#pragma once

#ifdef CAPTURECORE_EXPORTS
#define CAPTURE_API __declspec(dllexport)
#else
#define CAPTURE_API __declspec(dllimport)
#endif

#include <stdint.h>

extern "C" {
    // Детекция видеокарты
    CAPTURE_API bool capture_core_detect_gpu(uint32_t* vendor_id);

    // Инициализация захвата экрана
    // monitor_index: индекс монитора (0 - основной)
    // out_width, out_height: целевое разрешение для трансляции
    CAPTURE_API bool capture_core_init(int monitor_index, int out_width, int out_height);

    // Получение кадра
    // Заполняет плоскости Y, U, V (I420). 
    // Размеры буферов: Y = out_width * out_height, U и V = (out_width / 2) * (out_height / 2)
    // Возвращает true, если получен новый кадр, и false, если кадра нет или произошла ошибка
    CAPTURE_API bool capture_core_acquire_frame(uint8_t* y_plane, uint8_t* u_plane, uint8_t* v_plane);

    // Остановка захвата и освобождение ресурсов
    CAPTURE_API void capture_core_release();
}
