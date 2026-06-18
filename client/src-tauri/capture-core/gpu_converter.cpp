#include "gpu_converter.h"
#include <d3dcompiler.h>
#include <iostream>

struct ShaderParams {
    uint32_t width;
    uint32_t height;
    uint32_t pad[2];
};

const char* kShaderSource = R"(
Texture2D<float4> InputTexture : register(t0);
SamplerState LinearSampler : register(s0);
RWTexture2D<uint> OutputTextureY : register(u0);
RWTexture2D<uint> OutputTextureU : register(u1);
RWTexture2D<uint> OutputTextureV : register(u2);

cbuffer Params : register(b0) {
    uint Width;
    uint Height;
    uint Pad0;
    uint Pad1;
};

[numthreads(8, 8, 1)]
void CSMain(uint3 threadID : SV_DispatchThreadID) {
    if (threadID.x >= Width || threadID.y >= Height) return;

    // Сэмплируем с билинейной фильтрацией из оригинальной текстуры (авто-масштабирование)
    float2 uv = (float2(threadID.xy) + 0.5f) / float2(Width, Height);
    float4 color = InputTexture.SampleLevel(LinearSampler, uv, 0);
    
    // Формула конвертации RGB -> YUV (ITU-R BT.601 Studio Range)
    // В HLSL при чтении текстуры компоненты автоматически приводятся к стандарту RGBA (r=.x, g=.y, b=.z)
    float r = color.r * 255.0f;
    float g = color.g * 255.0f;
    float b = color.b * 255.0f;

    float y = ((66.0f * r + 129.0f * g + 25.0f * b + 128.0f) / 256.0f) + 16.0f;
    OutputTextureY[threadID.xy] = uint(clamp(y, 0.0f, 255.0f));

    // Субдискретизация U и V (2x2)
    if ((threadID.x % 2 == 0) && (threadID.y % 2 == 0)) {
        // Берем центр блока 2x2 пикселей
        float2 uv_chroma = (float2(threadID.xy) + 1.0f) / float2(Width, Height);
        float4 chroma_color = InputTexture.SampleLevel(LinearSampler, uv_chroma, 0);
        float cr = chroma_color.r * 255.0f;
        float cg = chroma_color.g * 255.0f;
        float cb = chroma_color.b * 255.0f;

        float u = ((-38.0f * cr - 74.0f * cg + 112.0f * cb + 128.0f) / 256.0f) + 128.0f;
        float v = ((112.0f * cr - 94.0f * cg - 18.0f * cb + 128.0f) / 256.0f) + 128.0f;

        OutputTextureU[threadID.xy / 2] = uint(clamp(u, 0.0f, 255.0f));
        OutputTextureV[threadID.xy / 2] = uint(clamp(v, 0.0f, 255.0f));
    }
}
)";

GPUConverter::GPUConverter() {}

GPUConverter::~GPUConverter() {
    release();
}

bool GPUConverter::init(ID3D11Device* device, ID3D11DeviceContext* context, int width, int height) {
    device_ = device;
    context_ = context;
    width_ = width;
    height_ = height;

    device_->AddRef();
    context_->AddRef();

    if (!compile_shader()) {
        std::cerr << "[GPUConverter] Failed to compile compute shader" << std::endl;
        return false;
    }

    if (!create_resources()) {
        std::cerr << "[GPUConverter] Failed to create D3D11 resources" << std::endl;
        return false;
    }

    return true;
}

void GPUConverter::release() {
    if (compute_shader_) { compute_shader_->Release(); compute_shader_ = nullptr; }
    if (sampler_state_) { sampler_state_->Release(); sampler_state_ = nullptr; }
    if (constant_buffer_) { constant_buffer_->Release(); constant_buffer_ = nullptr; }

    if (input_copy_texture_) { input_copy_texture_->Release(); input_copy_texture_ = nullptr; }
    if (input_srv_) { input_srv_->Release(); input_srv_ = nullptr; }

    if (output_tex_y_) { output_tex_y_->Release(); output_tex_y_ = nullptr; }
    if (output_tex_u_) { output_tex_u_->Release(); output_tex_u_ = nullptr; }
    if (output_tex_v_) { output_tex_v_->Release(); output_tex_v_ = nullptr; }

    if (output_uav_y_) { output_uav_y_->Release(); output_uav_y_ = nullptr; }
    if (output_uav_u_) { output_uav_u_->Release(); output_uav_u_ = nullptr; }
    if (output_uav_v_) { output_uav_v_->Release(); output_uav_v_ = nullptr; }

    if (staging_tex_y_) { staging_tex_y_->Release(); staging_tex_y_ = nullptr; }
    if (staging_tex_u_) { staging_tex_u_->Release(); staging_tex_u_ = nullptr; }
    if (staging_tex_v_) { staging_tex_v_->Release(); staging_tex_v_ = nullptr; }

    if (context_) { context_->Release(); context_ = nullptr; }
    if (device_) { device_->Release(); device_ = nullptr; }
}

bool GPUConverter::compile_shader() {
    ID3DBlob* shader_blob = nullptr;
    ID3DBlob* error_blob = nullptr;

    HRESULT hr = D3DCompile(
        kShaderSource,
        strlen(kShaderSource),
        "CSMain",
        nullptr,
        nullptr,
        "CSMain",
        "cs_5_0",
        D3DCOMPILE_OPTIMIZATION_LEVEL3,
        0,
        &shader_blob,
        &error_blob
    );

    if (FAILED(hr)) {
        if (error_blob) {
            std::cerr << "[GPUConverter] Shader compile error: "
                      << (char*)error_blob->GetBufferPointer() << std::endl;
            error_blob->Release();
        }
        return false;
    }
    // Release warnings blob even on success (D3DCompile can set it with S_OK)
    if (error_blob) { error_blob->Release(); error_blob = nullptr; }

    hr = device_->CreateComputeShader(
        shader_blob->GetBufferPointer(),
        shader_blob->GetBufferSize(),
        nullptr,
        &compute_shader_
    );

    shader_blob->Release();
    return SUCCEEDED(hr);
}

bool GPUConverter::create_resources() {
    // 1. Создание Sampler State для билинейного масштабирования
    D3D11_SAMPLER_DESC sampler_desc = {};
    sampler_desc.Filter = D3D11_FILTER_MIN_MAG_MIP_LINEAR;
    sampler_desc.AddressU = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler_desc.AddressV = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler_desc.AddressW = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler_desc.ComparisonFunc = D3D11_COMPARISON_NEVER;
    sampler_desc.MinLOD = 0;
    sampler_desc.MaxLOD = D3D11_FLOAT32_MAX;

    HRESULT hr = device_->CreateSamplerState(&sampler_desc, &sampler_state_);
    if (FAILED(hr)) return false;

    // 2. Создание Constant Buffer для размеров
    D3D11_BUFFER_DESC cb_desc = {};
    cb_desc.ByteWidth = sizeof(ShaderParams);
    cb_desc.Usage = D3D11_USAGE_DYNAMIC;
    cb_desc.BindFlags = D3D11_BIND_CONSTANT_BUFFER;
    cb_desc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;

    hr = device_->CreateBuffer(&cb_desc, nullptr, &constant_buffer_);
    if (FAILED(hr)) return false;

    // Заполнение Constant Buffer
    D3D11_MAPPED_SUBRESOURCE mapped;
    hr = context_->Map(constant_buffer_, 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped);
    if (SUCCEEDED(hr)) {
        ShaderParams* params = (ShaderParams*)mapped.pData;
        params->width = width_;
        params->height = height_;
        context_->Unmap(constant_buffer_, 0);
    } else {
        return false;
    }

    // 3. Создание выходных текстур (UAV) Y, U, V
    auto create_uav_texture = [&](int w, int h, ID3D11Texture2D** tex, ID3D11UnorderedAccessView** uav) -> bool {
        D3D11_TEXTURE2D_DESC desc = {};
        desc.Width = w;
        desc.Height = h;
        desc.MipLevels = 1;
        desc.ArraySize = 1;
        desc.Format = DXGI_FORMAT_R8_UINT;
        desc.SampleDesc.Count = 1;
        desc.Usage = D3D11_USAGE_DEFAULT;
        desc.BindFlags = D3D11_BIND_UNORDERED_ACCESS;

        HRESULT hr = device_->CreateTexture2D(&desc, nullptr, tex);
        if (FAILED(hr)) return false;

        D3D11_UNORDERED_ACCESS_VIEW_DESC uav_desc = {};
        uav_desc.Format = desc.Format;
        uav_desc.ViewDimension = D3D11_UAV_DIMENSION_TEXTURE2D;

        hr = device_->CreateUnorderedAccessView(*tex, &uav_desc, uav);
        return SUCCEEDED(hr);
    };

    if (!create_uav_texture(width_, height_, &output_tex_y_, &output_uav_y_)) return false;
    if (!create_uav_texture(width_ / 2, height_ / 2, &output_tex_u_, &output_uav_u_)) return false;
    if (!create_uav_texture(width_ / 2, height_ / 2, &output_tex_v_, &output_uav_v_)) return false;

    // 4. Создание Staging текстур для чтения результатов на CPU
    auto create_staging_texture = [&](int w, int h, ID3D11Texture2D** tex) -> bool {
        D3D11_TEXTURE2D_DESC desc = {};
        desc.Width = w;
        desc.Height = h;
        desc.MipLevels = 1;
        desc.ArraySize = 1;
        desc.Format = DXGI_FORMAT_R8_UINT;
        desc.SampleDesc.Count = 1;
        desc.Usage = D3D11_USAGE_STAGING;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;

        HRESULT hr = device_->CreateTexture2D(&desc, nullptr, tex);
        return SUCCEEDED(hr);
    };

    if (!create_staging_texture(width_, height_, &staging_tex_y_)) return false;
    if (!create_staging_texture(width_ / 2, height_ / 2, &staging_tex_u_)) return false;
    if (!create_staging_texture(width_ / 2, height_ / 2, &staging_tex_v_)) return false;

    return true;
}

bool GPUConverter::convert(ID3D11Texture2D* src_texture, uint8_t* y_plane, uint8_t* u_plane, uint8_t* v_plane) {
    if (!device_ || !context_ || !src_texture) return false;

    // Получаем параметры исходной текстуры
    D3D11_TEXTURE2D_DESC src_desc;
    src_texture->GetDesc(&src_desc);

    // Пересоздаём при смене разрешения источника (CopyResource требует одинаковые размеры)
    bool needs_realloc = !input_copy_texture_ ||
        input_copy_width_ != (int)src_desc.Width ||
        input_copy_height_ != (int)src_desc.Height;

    if (needs_realloc) {
        if (input_srv_) { input_srv_->Release(); input_srv_ = nullptr; }
        if (input_copy_texture_) { input_copy_texture_->Release(); input_copy_texture_ = nullptr; }

        D3D11_TEXTURE2D_DESC desc = src_desc;
        desc.Usage = D3D11_USAGE_DEFAULT;
        desc.BindFlags = D3D11_BIND_SHADER_RESOURCE;
        desc.CPUAccessFlags = 0;
        desc.MiscFlags = 0;

        HRESULT hr = device_->CreateTexture2D(&desc, nullptr, &input_copy_texture_);
        if (FAILED(hr)) return false;

        D3D11_SHADER_RESOURCE_VIEW_DESC srv_desc = {};
        srv_desc.Format = desc.Format;
        srv_desc.ViewDimension = D3D11_SRV_DIMENSION_TEXTURE2D;
        srv_desc.Texture2D.MipLevels = 1;

        hr = device_->CreateShaderResourceView(input_copy_texture_, &srv_desc, &input_srv_);
        if (FAILED(hr)) return false;

        input_copy_width_ = (int)src_desc.Width;
        input_copy_height_ = (int)src_desc.Height;
    }

    // Копируем кадр захвата в нашу текстуру-посредник (GPU-to-GPU)
    context_->CopyResource(input_copy_texture_, src_texture);

    // Привязываем ресурсы к Compute Shader
    context_->CSSetShader(compute_shader_, nullptr, 0);
    context_->CSSetShaderResources(0, 1, &input_srv_);
    context_->CSSetSamplers(0, 1, &sampler_state_);
    context_->CSSetConstantBuffers(0, 1, &constant_buffer_);

    ID3D11UnorderedAccessView* uavs[] = { output_uav_y_, output_uav_u_, output_uav_v_ };
    context_->CSSetUnorderedAccessViews(0, 3, uavs, nullptr);

    // Запускаем Compute Shader (размер групп: 8x8)
    int groups_x = (width_ + 7) / 8;
    int groups_y = (height_ + 7) / 8;
    context_->Dispatch(groups_x, groups_y, 1);

    // Отвязываем ресурсы
    ID3D11UnorderedAccessView* null_uavs[] = { nullptr, nullptr, nullptr };
    context_->CSSetUnorderedAccessViews(0, 3, null_uavs, nullptr);
    ID3D11ShaderResourceView* null_srvs[] = { nullptr };
    context_->CSSetShaderResources(0, 1, null_srvs);

    // Копируем результаты в Staging текстуры (GPU-to-GPU)
    context_->CopyResource(staging_tex_y_, output_tex_y_);
    context_->CopyResource(staging_tex_u_, output_tex_u_);
    context_->CopyResource(staging_tex_v_, output_tex_v_);

    // Вспомогательная лямбда-функция для чтения плоскостей (строчно, с учетом Pitch)
    auto copy_plane_to_ram = [&](ID3D11Texture2D* staging_tex, int w, int h, uint8_t* dest_plane) -> bool {
        D3D11_MAPPED_SUBRESOURCE mapped;
        HRESULT hr = context_->Map(staging_tex, 0, D3D11_MAP_READ, 0, &mapped);
        if (FAILED(hr)) return false;

        uint8_t* src_data = (uint8_t*)mapped.pData;
        for (int r = 0; r < h; ++r) {
            memcpy(dest_plane + r * w, src_data + r * mapped.RowPitch, w);
        }

        context_->Unmap(staging_tex, 0);
        return true;
    };

    if (!copy_plane_to_ram(staging_tex_y_, width_, height_, y_plane)) return false;
    if (!copy_plane_to_ram(staging_tex_u_, width_ / 2, height_ / 2, u_plane)) return false;
    if (!copy_plane_to_ram(staging_tex_v_, width_ / 2, height_ / 2, v_plane)) return false;

    return true;
}
