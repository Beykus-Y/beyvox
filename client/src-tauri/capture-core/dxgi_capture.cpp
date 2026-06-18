#include "dxgi_capture.h"
#include <iostream>

DXGICapture::DXGICapture() {}

DXGICapture::~DXGICapture() {
    release();
}

bool DXGICapture::init(int monitor_index, int out_width, int out_height) {
    monitor_index_ = monitor_index;
    out_width_ = out_width;
    out_height_ = out_height;

    return init_dxgi();
}

bool DXGICapture::init_dxgi() {
    release_dxgi();

    // 1. Создание D3D11 Device и Context
    D3D_FEATURE_LEVEL feature_levels[] = { D3D_FEATURE_LEVEL_11_0 };
    D3D_FEATURE_LEVEL feature_level;
    UINT flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
#ifdef _DEBUG
    flags |= D3D11_CREATE_DEVICE_DEBUG;
#endif

    HRESULT hr = D3D11CreateDevice(
        nullptr,
        D3D_DRIVER_TYPE_HARDWARE,
        nullptr,
        flags,
        feature_levels,
        1,
        D3D11_SDK_VERSION,
        &device_,
        &feature_level,
        &context_
    );

    if (FAILED(hr)) {
        std::cerr << "[DXGICapture] Failed to create D3D11 Device: " << hr << std::endl;
        return false;
    }

    // 2. Получение DXGI интерфейсов
    IDXGIDevice* dxgi_device = nullptr;
    hr = device_->QueryInterface(__uuidof(IDXGIDevice), (void**)&dxgi_device);
    if (FAILED(hr)) return false;

    IDXGIAdapter* dxgi_adapter = nullptr;
    hr = dxgi_device->GetParent(__uuidof(IDXGIAdapter), (void**)&dxgi_adapter);
    dxgi_device->Release();
    if (FAILED(hr)) return false;

    IDXGIOutput* dxgi_output = nullptr;
    hr = dxgi_adapter->EnumOutputs(monitor_index_, &dxgi_output);
    dxgi_adapter->Release();
    if (FAILED(hr)) {
        std::cerr << "[DXGICapture] Failed to enum output " << monitor_index_ << ": " << hr << std::endl;
        return false;
    }

    IDXGIOutput1* dxgi_output1 = nullptr;
    hr = dxgi_output->QueryInterface(__uuidof(IDXGIOutput1), (void**)&dxgi_output1);
    dxgi_output->Release();
    if (FAILED(hr)) return false;

    // 3. Дублирование экрана (Desktop Duplication)
    hr = dxgi_output1->DuplicateOutput(device_, &duplication_);
    dxgi_output1->Release();
    if (FAILED(hr)) {
        std::cerr << "[DXGICapture] DuplicateOutput failed: " << hr << std::endl;
        return false;
    }

    // 4. Инициализация GPU-конвертера
    if (!converter_.init(device_, context_, out_width_, out_height_)) {
        std::cerr << "[DXGICapture] Failed to init GPU converter" << std::endl;
        return false;
    }

    return true;
}

bool DXGICapture::reinit() {
    std::cout << "[DXGICapture] Reinitializing DXGI Desktop Duplication..." << std::endl;
    release_dxgi();
    
    // Даем системе время перестроиться (например, при смене режима UAC или разрешения)
    Sleep(100); 
    
    return init_dxgi();
}

void DXGICapture::release_dxgi() {
    converter_.release();

    if (last_frame_tex_) { last_frame_tex_->Release(); last_frame_tex_ = nullptr; }
    if (duplication_) { duplication_->Release(); duplication_ = nullptr; }
    if (context_) { context_->Release(); context_ = nullptr; }
    if (device_) { device_->Release(); device_ = nullptr; }
}

void DXGICapture::release() {
    release_dxgi();
}

bool DXGICapture::acquire_frame(uint8_t* y_plane, uint8_t* u_plane, uint8_t* v_plane) {
    if (!duplication_) {
        if (!reinit()) {
            // Backoff — prevents CPU spin during UAC prompt or resolution change
            Sleep(200);
            return false;
        }
    }

    IDXGIResource* frame_resource = nullptr;
    DXGI_OUTDUPL_FRAME_INFO frame_info;

    // Получаем следующий кадр (таймаут 50мс)
    HRESULT hr = duplication_->AcquireNextFrame(50, &frame_info, &frame_resource);

    if (hr == DXGI_ERROR_ACCESS_LOST) {
        // Доступ потерян (UAC окно, смена разрешения, блокировка экрана)
        if (!reinit()) {
            Sleep(200);
        }
        return false;
    }

    if (hr == DXGI_ERROR_WAIT_TIMEOUT) {
        // Новых кадров нет (статичное изображение)
        if (last_frame_tex_) {
            // Повторяем конвертацию предыдущего кадра в новые буферы Rust
            return converter_.convert(last_frame_tex_, y_plane, u_plane, v_plane);
        }
        return false;
    }

    if (FAILED(hr)) {
        return false;
    }

    // Извлекаем D3D11 текстуру из ресурса
    ID3D11Texture2D* acquired_tex = nullptr;
    hr = frame_resource->QueryInterface(__uuidof(ID3D11Texture2D), (void**)&acquired_tex);
    frame_resource->Release();

    if (FAILED(hr)) {
        duplication_->ReleaseFrame();
        return false;
    }

    // Копируем во внутреннюю текстуру last_frame_tex_, чтобы отпустить системный кадр как можно быстрее
    D3D11_TEXTURE2D_DESC desc;
    acquired_tex->GetDesc(&desc);

    // Пересоздаём при смене разрешения экрана (CopyResource требует одинаковые размеры)
    bool needs_realloc = !last_frame_tex_ ||
        (int)desc.Width != last_frame_width_ ||
        (int)desc.Height != last_frame_height_;

    if (needs_realloc) {
        if (last_frame_tex_) { last_frame_tex_->Release(); last_frame_tex_ = nullptr; }

        D3D11_TEXTURE2D_DESC copy_desc = desc;
        copy_desc.Usage = D3D11_USAGE_DEFAULT;
        copy_desc.BindFlags = D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_RENDER_TARGET;
        copy_desc.CPUAccessFlags = 0;
        copy_desc.MiscFlags = 0;

        HRESULT create_hr = device_->CreateTexture2D(&copy_desc, nullptr, &last_frame_tex_);
        if (FAILED(create_hr)) {
            acquired_tex->Release();
            duplication_->ReleaseFrame();
            return false;
        }
        last_frame_width_ = (int)desc.Width;
        last_frame_height_ = (int)desc.Height;
    }

    context_->CopyResource(last_frame_tex_, acquired_tex);
    acquired_tex->Release();

    // Освобождаем системный кадр
    duplication_->ReleaseFrame();

    // Запускаем GPU-масштабирование и конвертацию
    return converter_.convert(last_frame_tex_, y_plane, u_plane, v_plane);
}
