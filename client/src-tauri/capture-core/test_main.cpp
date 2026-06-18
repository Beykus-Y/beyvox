#include "capture_core.h"
#include <iostream>
#include <fstream>
#include <vector>
#include <chrono>
#include <thread>

int main() {
    std::cout << "[CaptureTest] Starting screen capture test..." << std::endl;

    // 1. Детекция GPU
    uint32_t vendor_id = 0;
    if (capture_core_detect_gpu(&vendor_id)) {
        std::cout << "[CaptureTest] Detected GPU Vendor ID: 0x" << std::hex << vendor_id << std::dec << std::endl;
        switch (vendor_id) {
            case 0x10DE: std::cout << "[CaptureTest] Vendor: NVIDIA (NVENC will be used)" << std::endl; break;
            case 0x1002:
            case 0x1022: std::cout << "[CaptureTest] Vendor: AMD" << std::endl; break;
            case 0x8086: std::cout << "[CaptureTest] Vendor: Intel" << std::endl; break;
            default: std::cout << "[CaptureTest] Vendor: Unknown" << std::endl; break;
        }
    } else {
        std::cerr << "[CaptureTest] Failed to detect GPU Vendor ID" << std::endl;
    }

    // 2. Инициализация захвата (1920x1080, монитор 0)
    int width = 1920;
    int height = 1080;
    if (!capture_core_init(0, width, height)) {
        std::cerr << "[CaptureTest] Failed to initialize screen capture" << std::endl;
        return 1;
    }
    std::cout << "[CaptureTest] Capture initialized successfully at " << width << "x" << height << std::endl;

    // Выделяем буферы под I420
    std::vector<uint8_t> y_plane(width * height);
    std::vector<uint8_t> u_plane((width / 2) * (height / 2));
    std::vector<uint8_t> v_plane((width / 2) * (height / 2));

    // Открываем файл для записи дампов кадров
    std::ofstream out_file("output.yuv", std::ios::binary);
    if (!out_file) {
        std::cerr << "[CaptureTest] Failed to create output.yuv" << std::endl;
        capture_core_release();
        return 1;
    }

    const int total_frames = 100;
    int captured_frames = 0;
    double total_latency_ms = 0;
    double max_latency_ms = 0;

    std::cout << "[CaptureTest] Capturing " << total_frames << " frames to output.yuv..." << std::endl;

    for (int i = 0; i < total_frames; ++i) {
        auto start = std::chrono::high_resolution_clock::now();

        bool success = capture_core_acquire_frame(y_plane.data(), u_plane.data(), v_plane.data());

        auto end = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double, std::milli> latency = end - start;

        if (success) {
            captured_frames++;
            double lat_ms = latency.count();
            total_latency_ms += lat_ms;
            if (lat_ms > max_latency_ms) {
                max_latency_ms = lat_ms;
            }

            // Записываем кадр в YUV420p формате
            out_file.write((char*)y_plane.data(), y_plane.size());
            out_file.write((char*)u_plane.data(), u_plane.size());
            out_file.write((char*)v_plane.data(), v_plane.size());
        }

        // Имитируем ~60 FPS
        std::this_thread::sleep_for(std::chrono::milliseconds(16));
    }

    out_file.close();
    capture_core_release();

    std::cout << "[CaptureTest] Done!" << std::endl;
    std::cout << "[CaptureTest] Total captured frames: " << captured_frames << " / " << total_frames << std::endl;
    if (captured_frames > 0) {
        std::cout << "[CaptureTest] Average GPU Processing + PCIe Copy Latency: " 
                  << (total_latency_ms / captured_frames) << " ms" << std::endl;
        std::cout << "[CaptureTest] Max Latency: " << max_latency_ms << " ms" << std::endl;
    }

    std::cout << "[CaptureTest] Saved raw video to output.yuv" << std::endl;
    std::cout << "[CaptureTest] You can play it using: ffplay -f rawvideo -pixel_format yuv420p -video_size 1920x1080 output.yuv" << std::endl;

    return 0;
}
