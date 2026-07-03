# ZeroBrowser HarmonyOS App

HarmonyOS 工程脚手架，将 `zero-ui-adapter-harmonyos` Rust 库打包为 HAP，在 HarmonyOS 设备/模拟器上启动 ZeroBrowser UI SDK 运行时。

## 架构

```
EntryAbility.ets  ←── NAPI ←── libzeroui.so  ←── Rust adapter
     │                                                    │
     ├── WindowStage (生命周期)                               ├── HarmonyOSRuntime (PlatformRuntime impl)
     ├── TouchEvent → nativeDispatchTouch                    ├── event_map (OHOS → UiEvent)
     ├── KeyEvent → nativeDispatchKey                        ├── ffi.rs (C ABI)
     └── RenderLoop (60fps pumpEvents)                       └── napi_init.cpp (C++ NAPI bridge)
```

两层桥接：
1. **Rust .so** → C ABI 函数（`harmonyos_*`）
2. **C++ NAPI bridge**（`napi_init.cpp`）→ 注册为 NAPI 模块 `zeroui` → ArkTS `import native from 'libzeroui.so'`

## 快速开始

### 前提条件

- DevEco Studio 已安装
- 环境变量 `DEVECO_SDK_HOME` = `C:\Program Files\Huawei\DevEco Studio\sdk`
- Rust target `aarch64-unknown-linux-ohos`

### CLI 构建

```powershell
.\build.ps1            # debug 构建
.\build.ps1 release    # release 构建
```

构建过程：
1. `cargo build --target aarch64-unknown-linux-ohos` 交叉编译 Rust → `libzero_ui_adapter_harmonyos.so` (~14 MB)
2. 复制 `.so` 到 `entry/libs/arm64-v8a/`
3. `hvigorw assembleHap` 打包 HAP (~21 KB)

### DevEco Studio 构建

用 DevEco Studio 打开此目录，点击 **Build → Build Hap(s)**。

首次打开时 DevEco Studio 会自动 migrate 项目配置。

### 安装和运行

```powershell
# 安装到设备（需要 hdc 在 PATH 或 SDK 的 toolchains 目录下）
hdc install entry\build\default\outputs\default\entry-default-unsigned.hap

# 启动
hdc shell aa start -a EntryAbility -b com.zeroweb.ui

# 查看日志
hdc hilog -T ZeroBrowser
```

## 项目结构

```
harmonyos-app/
├── build.ps1                         # 一键 CLI 构建脚本
├── hvigorfile.ts                     # 根 hvigor 任务定义
├── oh-package.json5                  # 根包配置
├── build-profile.json5               # 根构建配置（products + modules）
├── .gitignore                        # 忽略 build/、entry/libs/、.hvigor/
├── hvigor/
│   └── hvigor-config.json5           # hvigor 引擎配置
├── AppScope/
│   ├── app.json5                     # 应用 scope 声明（bundleName 等）
│   └── resources/base/media/
│       └── app_icon.png              # 占位图标（1x1px PNG）
└── entry/
    ├── hvigorfile.ts                 # 模块 hvigor 任务定义
    ├── oh-package.json5              # 模块包配置
    ├── build-profile.json5           # 模块构建配置（apiType: stageMode）
    ├── libs/arm64-v8a/               # Rust .so（build.ps1 生成，已 gitignore）
    │   └── libzero_ui_adapter_harmonyos.so
    └── src/main/
        ├── module.json5              # 模块/Ability 声明
        ├── cpp/
        │   ├── CMakeLists.txt        # Native 代码构建配置
        │   └── napi_init.cpp         # NAPI C++ 桥接（Rust ←→ ArkTS）
        ├── ets/
        │   ├── entryability/
        │   │   └── EntryAbility.ets  # 主 Ability（生命周期 + 渲染循环）
        │   └── pages/
        │       └── Index.ets         # 主页面（Touch 事件路由）
        └── resources/
            ├── base/
            │   ├── element/
            │   │   ├── string.json   # 字符串资源
            │   │   └── color.json    # 颜色资源
            │   ├── media/
            │   │   ├── app_icon.png  # 占位图标
            │   │   └── start_icon.png
            │   └── profile/
            │       └── main_pages.json
```

## 测试

### Rust 侧单元测试

```powershell
cd ..\..\..\..\
cargo test -p zero-ui-adapter-harmonyos
```

覆盖：event_map（触摸·键盘·度量·软键盘·back·主题）、runtime（retained 闭环·back·度量·事件队列）、ffi（raw 事件→UiEvent·队列·度量更新）

### 集成测试（需设备/模拟器）

```powershell
# 构建并安装
.\build.ps1
hdc install entry\build\default\outputs\default\entry-default-unsigned.hap
hdc shell aa start -a EntryAbility -b com.zeroweb.ui
hdc hilog -T ZeroBrowser    # 确认运行时初始化日志
```

## 环境变量

| 变量 | 值 | 说明 |
|------|----|------|
| `DEVECO_SDK_HOME` | `C:\Program Files\Huawei\DevEco Studio\sdk` | HarmonyOS SDK 目录 |
| `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER` | SDK LLVM clang 路径 | Rust 交叉编译 linker（build.ps1 自动设置） |

## 注意事项

- **NAPI 模块验证**：HAP 构建时会提示 "module for 'libzeroui.so' is not verified"——这是正常的，当前没有 `.d.ts` 类型声明文件。后续可添加 `entry/src/main/cpp/types/libzeroui/index.d.ts` 消除。
- **未签名**：CLI 构建产出 `entry-default-unsigned.hap`。正式发布需在 DevEco Studio 中配置签名。
- **densityDPI 兼容**：`EntryAbility.ets` 使用硬编码 density=3.0，因为 `WindowProperties.densityDPI` 在不同 API 版本可能不可用。
