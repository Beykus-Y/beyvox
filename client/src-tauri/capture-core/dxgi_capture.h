#pragma once

#include "gpu_converter.h"
#include <d3d11.h>
#include <dxgi1_2.h>
#include <stdint.h>

class DXGICapture {
public:
    DXGICapture();
    ~DXGICapture();

    bool init(int monitor_index, int out_width, int out_height);
    bool acquire_frame(uint8_t* y_plane, uint8_t* u_plane, uint8_t* v_plane);
    void release();

private:
    int monitor_index_ = 0;
    int out_width_ = 0;
    int out_height_ = 0;

    ID3D11Device* device_ = nullptr;
    ID3D11DeviceContext* context_ = nullptr;
    IDXGIOutputDuplication* duplication_ = nullptr;

    GPUConverter converter_;

    ID3D11Texture2D* last_frame_tex_ = nullptr;
    int last_frame_width_ = 0;
    int last_frame_height_ = 0;

    bool init_dxgi();
    bool reinit();
    void release_dxgi();
};
