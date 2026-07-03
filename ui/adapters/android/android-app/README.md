# ZeroBrowser Android App

Android 工程脚手架，将 `zero-ui-adapter-android` Rust 库打包为 APK，在 Android 设备/模拟器上启动 ZeroBrowser UI SDK 运行时。

## 架构

```
MainActivity.kt  ←── JNI ←── libzero_ui_adapter_android.so  ←── Rust adapter
     │                                                           │
     ├── SurfaceView (渲染表面)                                    ├── AndroidRuntime (PlatformRuntime impl)
     ├── onTouchEvent → nativeDispatchTouch                        ├── event_map (Android → UiEvent)
     ├── onKeyDown/Up → nativeDispatchKey                          └── ffi.rs (JNI C ABI)
     ├── onBackPressed → nativeBackPressed
     ├── onApplyWindowInsets → nativeSoftKeyboard
     └── RenderLoop (60fps pumpEvents)
```

## 快速开始

### 前提条件

- Android SDK（`ANDROID_HOME` 指向 SDK 目录）
- Android NDK 30+（`ANDROID_NDK_HOME` 指向 NDK 目录）
- Rust target `aarch64-linux-android`
- `cargo-ndk`（`cargo install cargo-ndk`）
- Java 17+
- 代理（本工程 Gradle 使用代理 `127.0.0.1:7078`，编辑 `gradle.properties` 修改）

### 构建

```powershell
.\build.ps1            # debug 构建
.\build.ps1 release    # release 构建
```

构建过程：
1. `cargo ndk -t arm64-v8a` 交叉编译 Rust → `libzero_ui_adapter_android.so` (~14 MB)
2. 复制 `.so` 到 `app/src/main/jniLibs/arm64-v8a/`
3. `gradlew assembleDebug` 打包 APK (~14 MB)

### 安装和运行

```powershell
# 安装到设备
adb install -r app\build\outputs\apk\debug\app-debug.apk

# 启动
adb shell am start -n com.zeroweb.ui/.MainActivity

# 查看日志
adb logcat -s ZeroBrowser:V
```

### 首次初始化

如果 `gradle-wrapper.jar` 缺失或损坏：

```powershell
.\bootstrap.ps1     # 从官方 Gradle GitHub 下载 wrapper jar
```

## 项目结构

```
android-app/
├── build.ps1                    # 一键构建脚本
├── bootstrap.ps1                # Gradle wrapper 下载
├── gradlew.bat                  # Gradle wrapper
├── settings.gradle.kts          # 项目设置
├── gradle.properties            # Gradle 属性（含代理）
├── build.gradle.kts             # 根构建配置
├── .gitignore                   # 忽略 build/、.gradle/、jniLibs/
├── .gitattributes               # *.jar binary
├── gradle/wrapper/              # Gradle wrapper（gradle-wrapper.jar 已提交）
│   ├── gradle-wrapper.jar       # 43 KB
│   └── gradle-wrapper.properties
└── app/
    ├── build.gradle.kts         # 模块构建配置（compileSdk 36, minSdk 26）
    └── src/main/
        ├── AndroidManifest.xml  # 应用声明
        ├── jniLibs/arm64-v8a/   # Rust .so（build.ps1 生成，已 gitignore）
        │   └── libzero_ui_adapter_android.so
        ├── java/com/zeroweb/ui/
        │   └── MainActivity.kt  # 主 Activity（JNI 桥接 + 事件路由）
        └── res/                 # 资源文件
```

## 测试

### Rust 侧单元测试

```powershell
cd ..\..\..\..\
cargo test -p zero-ui-adapter-android
```

覆盖：event_map（触摸·键盘·度量·软键盘·back·主题）、runtime（retained 闭环·back·度量·事件队列）、ffi（raw 事件→UiEvent·队列·度量更新）

### 集成测试（需设备）

```powershell
# 构建并安装到模拟器
.\build.ps1
adb install -r app\build\outputs\apk\debug\app-debug.apk
adb shell am start -n com.zeroweb.ui/.MainActivity
adb logcat -s ZeroBrowser:V    # 确认 "Runtime initialized" 日志
```

## 环境变量

| 变量 | 示例值 | 说明 |
|------|--------|------|
| `ANDROID_HOME` | `%LOCALAPPDATA%\Android\Sdk` | Android SDK 目录 |
| `ANDROID_NDK_HOME` | `%ANDROID_HOME%\ndk\30.0.14904198` | NDK 目录 |
| `JAVA_HOME` | `C:\Program Files\Microsoft\jdk-17.0.16.8-hotspot` | JDK 目录 |
| `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER` | NDK clang 路径 | Rust 交叉编译 linker（build.ps1 自动设置） |
