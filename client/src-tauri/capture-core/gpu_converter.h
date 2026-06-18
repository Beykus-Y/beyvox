#pragma once

#include <d3d11.h>
#include <stdint.h>

class GPUConverter {
public:
    GPUConverter();
    ~GPUConverter();

    bool init(ID3D11Device* device, ID3D11DeviceContext* context, int width, int height);
    void release();

    bool convert(ID3D11Texture2D* src_texture, uint8_t* y_plane, uint8_t* u_plane, uint8_t* v_plane);

private:
    ID3D11Device* device_ = nullptr;
    ID3D11DeviceContext* context_ = nullptr;

    int width_ = 0;
    int height_ = 0;

    ID3D11ComputeShader* compute_shader_ = nullptr;
    ID3D11SamplerState* sampler_state_ = nullptr;
    ID3D11Buffer* constant_buffer_ = nullptr;

    ID3D11Texture2D* input_copy_texture_ = nullptr;
    ID3D11ShaderResourceView* input_srv_ = nullptr;
    int input_copy_width_ = 0;
    int input_copy_height_ = 0;

    ID3D11Texture2D* output_tex_y_ = nullptr;
    ID3D11Texture2D* output_tex_u_ = nullptr;
    ID3D11Texture2D* output_tex_v_ = nullptr;

    ID3D11UnorderedAccessView* output_uav_y_ = nullptr;
    ID3D11UnorderedAccessView* output_uav_u_ = nullptr;
    ID3D11UnorderedAccessView* output_uav_v_ = nullptr;

    ID3D11Texture2D* staging_tex_y_ = nullptr;
    ID3D11Texture2D* staging_tex_u_ = nullptr;
    ID3D11Texture2D* staging_tex_v_ = nullptr;

    bool create_resources();
    bool compile_shader();
};
