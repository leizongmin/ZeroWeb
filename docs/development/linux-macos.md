# Linux 和 macOS 开发环境

本文档说明在 Linux 和 macOS 上构建、运行和测试 ZeroWeb 所需的环境。项目同时支持 x86_64 和 AArch64；`rusty_v8` 下载脚本会根据当前 Rust host target 选择对应的预构建产物。

## 1. 安装基础工具

两个平台都需要：

1. Rust `1.85` 或更新版本，并安装格式化和静态检查组件：

   ```bash
   rustup toolchain install stable
   rustup default stable
   rustup component add rustfmt clippy
   ```

2. Node.js 20 或更新版本。Chrome 一致性脚本和 WPT 辅助脚本需要 Node.js。
3. GNU Make、Git、Bash 和 curl。

验证基础工具：

```bash
rustc --version
cargo --version
node --version
make --version
git --version
curl --version
```

## 2. 安装 Linux 系统依赖

Ubuntu/Debian 环境建议安装与 CI 相同的桌面和构建依赖，并补充 QuickJS bindgen 所需的 `libclang`：

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  git \
  pkg-config \
  libssl-dev \
  libclang-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libfontconfig1-dev \
  libwayland-dev \
  libx11-dev \
  libxrandr-dev \
  libxi-dev \
  libgl1-mesa-dev \
  mesa-vulkan-drivers
```

`libclang-dev` 提供 `rquickjs-sys` 的 bindgen 构建脚本需要的动态库。`mesa-vulkan-drivers` 提供 wgpu Vulkan 后端；缺失时 GPU 渲染可能回退到 GL 或 llvmpipe。

需要检查 Vulkan 设备枚举时，可额外安装 `vulkan-tools`：

```bash
sudo apt-get install -y vulkan-tools
vulkaninfo --summary
```

GPU 无头测试默认优先软件适配器以保证确定性。验证指定硬件 ICD 时，可显式设置 `VK_ICD_FILENAMES`，例如：

```bash
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/intel_icd.json \
  cargo test -p zero-render-foundation --lib gpu:: -- --test-threads=1
```

非 Ubuntu/Debian 发行版需要安装等价的 OpenSSL、libclang、X11、Wayland、fontconfig 和 OpenGL 开发包。

## 3. 安装 macOS 系统依赖

先安装 Xcode Command Line Tools：

```bash
xcode-select --install
```

它提供编译器、系统 SDK、Git 和 Make。Node.js 可通过官方安装包、版本管理器或 Homebrew 安装：

```bash
brew install node
```

默认情况下，Xcode 工具链中的 libclang 足以构建 QuickJS feature。若 bindgen 仍报告找不到 libclang，可安装 Homebrew LLVM，并在当前终端指向其动态库目录：

```bash
brew install llvm
export LIBCLANG_PATH="$(brew --prefix llvm)/lib"
cargo check -p zero-script-sandbox --no-default-features --features quickjs
```

不要把本机 Homebrew 路径写入仓库配置。Intel Mac 和 Apple Silicon 的 Homebrew 根目录不同，应始终通过 `brew --prefix` 获取。

## 4. 准备 rusty_v8

首次执行默认 V8 构建前运行：

```bash
make setup-rusty-v8
```

脚本从 `Cargo.lock` 读取 `rusty_v8` 版本，根据当前 Rust host target 下载对应的 release archive，并创建项目内链接。默认缓存目录为：

```text
${XDG_CACHE_HOME:-$HOME/.cache}/zero-web/rusty_v8
```

一般无需手工设置环境变量。使用镜像、自定义缓存或交叉目标时，可以覆盖：

```bash
RUSTY_V8_MIRROR=<release-base-url> make setup-rusty-v8
RUSTY_V8_CACHE_DIR=<cache-dir> make setup-rusty-v8
RUSTY_V8_TARGET=<rust-target-triple> make setup-rusty-v8
```

`make build`、`make browser` 和 `make browser-cpu` 会自动执行这一步。

## 5. 构建、运行和测试

```bash
# 准备 rusty_v8 并构建整个 workspace
make build

# GPU 模式启动浏览器
make browser

# CPU + scale 1.0 的 WPT 对齐模式
make browser-cpu

# 完整测试门禁
make test

# Workspace 静态检查
cargo clippy --workspace --all-targets -- -D warnings
```

无人值守测试必须使用 `make test`，不要绕过项目的 `test-guard`。首次运行 reftest 时，`make reftest` 会自动准备 WPT 数据。

## 6. 常见问题

### `Unable to find libclang`

Linux 确认已安装 `libclang-dev`，必要时用发行版工具定位 `libclang.so`。macOS 先确认 Xcode Command Line Tools 可用；仍失败时按第 3 节安装 Homebrew LLVM，并设置当前终端的 `LIBCLANG_PATH`。

### `rusty_v8` 下载或链接失败

单独运行 `make setup-rusty-v8` 查看版本、target 和下载诊断。确认 GitHub release 可访问，且 `rustc -vV` 输出的 host target 与本机架构一致。下载中断后脚本会保留临时文件，并在下次运行时继续下载。

### Linux 找不到 Vulkan adapter

确认已安装 `mesa-vulkan-drivers`，再用 `vulkaninfo --summary` 检查设备。无硬件 GPU 的环境可以使用 lavapipe；常规 `make test` 会先探测 adapter，不可用时跳过仅依赖 adapter 的测试。

### macOS 首次运行被 Gatekeeper 阻止

本地 `make browser` 直接运行构建产物，不需要应用签名。只有下载或分发 `.app` 时涉及 Gatekeeper、Developer ID 签名和公证，详见 [README 打包说明](../../README.md#4-打包为可分发产物)。
