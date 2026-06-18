#include "capture_core.h"
#include "dxgi_capture.h"
#include <memory>
#include <mutex>
#include <dxgi.h>

static std::unique_ptr<DXGICapture> g_capture = nullptr;
static std::mutex g_mutex;

extern "C" {
    CAPTURE_API bool capture_core_detect_gpu(uint32_t* vendor_id) {
        if (!vendor_id) return false;

        IDXGIFactory* factory = nullptr;
        HRESULT hr = CreateDXGIFactory(__uuidof(IDXGIFactory), (void**)&factory);
        if (FAILED(hr)) return false;

        IDXGIAdapter* adapter = nullptr;
        hr = factory->EnumAdapters(0, &adapter);
        factory->Release();
        if (FAILED(hr)) return false;

        DXGI_ADAPTER_DESC desc;
        hr = adapter->GetDesc(&desc);
        adapter->Release();
        if (FAILED(hr)) return false;

        *vendor_id = desc.VendorId;
        return true;
    }

    CAPTURE_API bool capture_core_init(int monitor_index, int out_width, int out_height) {
        std::lock_guard<std::mutex> lock(g_mutex);

        if (g_capture) {
            g_capture->release();
            g_capture = nullptr;
        }

        g_capture = std::make_unique<DXGICapture>();
        if (!g_capture->init(monitor_index, out_width, out_height)) {
            g_capture = nullptr;
            return false;
        }
        return true;
    }

    CAPTURE_API bool capture_core_acquire_frame(uint8_t* y_plane, uint8_t* u_plane, uint8_t* v_plane) {
        std::lock_guard<std::mutex> lock(g_mutex);
        if (!g_capture) return false;
        return g_capture->acquire_frame(y_plane, u_plane, v_plane);
    }

    CAPTURE_API void capture_core_release() {
        std::lock_guard<std::mutex> lock(g_mutex);
        if (g_capture) {
            g_capture->release();
            g_capture = nullptr;
        }
    }
}
