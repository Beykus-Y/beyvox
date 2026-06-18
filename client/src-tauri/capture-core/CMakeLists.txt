cmake_minimum_required(VERSION 3.12)
project(CaptureCore LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_definitions(-DUNICODE -D_UNICODE -DCAPTURECORE_EXPORTS)

# Source files
set(SOURCES
    capture_core.h
    capture_core.cpp
    dxgi_capture.h
    dxgi_capture.cpp
    gpu_converter.h
    gpu_converter.cpp
)

# Build DLL
add_library(CaptureCore SHARED ${SOURCES})

# Target properties (ensure prefix-less CaptureCore.dll on Windows)
set_target_properties(CaptureCore PROPERTIES
    PREFIX ""
    SUFFIX ".dll"
)

# Link libraries
target_link_libraries(CaptureCore PRIVATE
    d3d11
    dxgi
    d3dcompiler
)

# Build Test executable
add_executable(CaptureTest test_main.cpp)
target_link_libraries(CaptureTest PRIVATE CaptureCore)

# Copy DLL to Test output directory automatically on build
add_custom_command(TARGET CaptureTest POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
    $<TARGET_FILE:CaptureCore>
    $<TARGET_FILE_DIR:CaptureTest>/CaptureCore.dll
)
