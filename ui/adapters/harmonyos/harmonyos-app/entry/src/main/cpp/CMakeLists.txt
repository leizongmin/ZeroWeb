cmake_minimum_required(VERSION 3.4.1)
project(zeroui)

# Pre-built Rust shared library (built by cargo ndk / cargo build --target aarch64-unknown-linux-ohos)
add_library(zero_ui_adapter_harmonyos SHARED IMPORTED)
set_target_properties(zero_ui_adapter_harmonyos PROPERTIES
    IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/../../libs/${OHOS_ARCH}/libzero_ui_adapter_harmonyos.so
)

# NAPI bridge library (thin wrapper around Rust .so)
add_library(zeroui SHARED napi_init.cpp)
target_link_libraries(zeroui PUBLIC
    libace_napi.z.so
    libhilog_ndk.z.so
    zero_ui_adapter_harmonyos
)
