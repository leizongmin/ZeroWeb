// NAPI bridge: Rust C ABI ↔ ArkTS
// HarmonyOS native module registration.
//
// ArkTS import:  import native from 'libzeroui.so'
// Then call:     native.initRuntime(w, h, density)
//                native.windowSizeChange(w, h, density, textScale, safeT, safeR, safeB, safeL, isPortrait)
//                native.dispatchTouch(touchId, x, y, action)
//                native.backPressed()
//                native.inputMethodChange(x, y, w, h, visible)
//                native.isRuntimeReady()
//                native.pumpEvents()
//                native.shutdown()

#include <napi/native_api.h>
#include <hilog/log.h>

#undef LOG_DOMAIN
#undef LOG_TAG
#define LOG_DOMAIN 0x3200
#define LOG_TAG "ZeroBrowser"

// ── Rust C ABI declarations ────────────────────────────────────────────────

extern "C" {
    void harmonyos_init_runtime(float width, float height, float density);
    void harmonyos_window_size_change(float width, float height,
        float density, float text_scale,
        float safe_top, float safe_right, float safe_bottom, float safe_left,
        uint32_t is_portrait);
    void harmonyos_dispatch_touch(uint32_t touch_id, float x, float y, uint32_t action);
    void harmonyos_back_pressed();
    void harmonyos_input_method_change(float keyboard_x, float keyboard_y,
        float keyboard_w, float keyboard_h, uint32_t is_visible);
    uint32_t harmonyos_is_runtime_ready();
    void harmonyos_pump_events();
    void harmonyos_shutdown();
}

// ── NAPI wrappers ──────────────────────────────────────────────────────────

static napi_value NapiInitRuntime(napi_env env, napi_callback_info info) {
    size_t argc = 3;
    napi_value args[3];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    double w = 390, h = 844, density = 3.0;
    if (argc > 0) napi_get_value_double(env, args[0], &w);
    if (argc > 1) napi_get_value_double(env, args[1], &h);
    if (argc > 2) napi_get_value_double(env, args[2], &density);
    harmonyos_init_runtime((float)w, (float)h, (float)density);
    OH_LOG_INFO(LOG_APP, "ZeroBrowser runtime initialized [%dx%d, density=%0.1f]", (int)w, (int)h, density);
    return nullptr;
}

static napi_value NapiWindowSizeChange(napi_env env, napi_callback_info info) {
    size_t argc = 9;
    napi_value args[9];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    double vals[9] = {0};
    for (int i = 0; i < (int)argc && i < 9; i++) napi_get_value_double(env, args[i], &vals[i]);
    uint32_t portrait = (argc > 8) ? (uint32_t)vals[8] : 1;
    harmonyos_window_size_change(
        (float)vals[0], (float)vals[1], (float)vals[2], (float)vals[3],
        (float)vals[4], (float)vals[5], (float)vals[6], (float)vals[7], portrait);
    return nullptr;
}

static napi_value NapiDispatchTouch(napi_env env, napi_callback_info info) {
    size_t argc = 4;
    napi_value args[4];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    double touchId = 0, x = 0, y = 0, action = 0;
    if (argc > 0) napi_get_value_double(env, args[0], &touchId);
    if (argc > 1) napi_get_value_double(env, args[1], &x);
    if (argc > 2) napi_get_value_double(env, args[2], &y);
    if (argc > 3) napi_get_value_double(env, args[3], &action);
    harmonyos_dispatch_touch((uint32_t)touchId, (float)x, (float)y, (uint32_t)action);
    return nullptr;
}

static napi_value NapiBackPressed(napi_env env, napi_callback_info info) {
    harmonyos_back_pressed();
    return nullptr;
}

static napi_value NapiInputMethodChange(napi_env env, napi_callback_info info) {
    size_t argc = 5;
    napi_value args[5];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    double vals[5] = {0};
    for (int i = 0; i < (int)argc && i < 5; i++) napi_get_value_double(env, args[i], &vals[i]);
    harmonyos_input_method_change(
        (float)vals[0], (float)vals[1], (float)vals[2], (float)vals[3], (uint32_t)vals[4]);
    return nullptr;
}

static napi_value NapiIsRuntimeReady(napi_env env, napi_callback_info info) {
    napi_value result;
    napi_create_int32(env, (int32_t)harmonyos_is_runtime_ready(), &result);
    return result;
}

static napi_value NapiPumpEvents(napi_env env, napi_callback_info info) {
    harmonyos_pump_events();
    return nullptr;
}

static napi_value NapiShutdown(napi_env env, napi_callback_info info) {
    harmonyos_shutdown();
    return nullptr;
}

// ── Module registration ────────────────────────────────────────────────────

EXTERN_C_START
static napi_value Init(napi_env env, napi_value exports) {
    napi_property_descriptor desc[] = {
        { "initRuntime",       nullptr, NapiInitRuntime,       nullptr, nullptr, nullptr, napi_default, nullptr },
        { "windowSizeChange",  nullptr, NapiWindowSizeChange,  nullptr, nullptr, nullptr, napi_default, nullptr },
        { "dispatchTouch",     nullptr, NapiDispatchTouch,     nullptr, nullptr, nullptr, napi_default, nullptr },
        { "backPressed",       nullptr, NapiBackPressed,       nullptr, nullptr, nullptr, napi_default, nullptr },
        { "inputMethodChange", nullptr, NapiInputMethodChange, nullptr, nullptr, nullptr, napi_default, nullptr },
        { "isRuntimeReady",    nullptr, NapiIsRuntimeReady,    nullptr, nullptr, nullptr, napi_default, nullptr },
        { "pumpEvents",        nullptr, NapiPumpEvents,        nullptr, nullptr, nullptr, napi_default, nullptr },
        { "shutdown",          nullptr, NapiShutdown,          nullptr, nullptr, nullptr, napi_default, nullptr },
    };
    napi_define_properties(env, exports, sizeof(desc) / sizeof(desc[0]), desc);
    return exports;
}
EXTERN_C_END

static napi_module demoModule = {
    .nm_version = 1,
    .nm_flags = 0,
    .nm_filename = nullptr,
    .nm_register_func = Init,
    .nm_modname = "zeroui",
    .nm_priv = nullptr,
    .reserved = { 0 },
};

extern "C" __attribute__((constructor)) void RegisterZerouiModule(void) {
    napi_module_register(&demoModule);
}
