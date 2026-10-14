# 构建、测试与提交门禁

运行构建、测试、clippy，或准备提交时必须读取本文。按实际影响选择检查；命令列在这里不表示每次都要执行。提交、推送、发布仍需各自授权。

## 常用入口

- 开发环境：[Linux/macOS](../development/linux-macos.md)、[Windows](../development/windows.md)。
- 准备 V8：`make setup-rusty-v8`，按平台下载；Windows 可设置 `RUSTY_V8_ARCHIVE` 指向匹配版本和架构的预编译库。
- 启动浏览器：`make browser`（release + GPU，先构建 browser、renderer、compositor、image-decoder）；仅构建用 `make browser-build`。
- WebView demo：`cargo run --bin webview-demo`。
- 构建工作区：`make build`，准备 V8 后分别构建其余 workspace 和 `zero-browser`，避免 workspace feature 合并掩盖浏览器单包依赖问题。
- 完整测试：`make test`，先构建配套子进程，再经 test-guard 执行，包含 QuickJS 检查；矩阵以 [Makefile](../../Makefile) 为准。
- 准备 WPT 数据：`make fetch-wpt-data`；首次运行依赖真实字体等资产的 WPT runner 测试前执行，`make reftest` 会自动调用。
- WPT reftest：`make reftest`，先 release 构建，再经 test-guard 执行。
- 定向上游 reftest：`make reftest-upstream FILTER=css-tables`。
- 基准：[scripts/run-benchmarks.sh](../../scripts/run-benchmarks.sh)；覆盖率：[scripts/check-coverage.sh](../../scripts/check-coverage.sh)。
- clippy：`cargo clippy --workspace --all-targets -- -D warnings`。
- Android：`make android-preflight`、`make android-apk`、`make android-release-apk`；renderer feature 前置条件见 [Android 构建说明](../../apps/android-browser/rust/README.md)。

## 定向测试与规范回归

测试框架为 Rust 内置 `#[test]`。先准备守卫：`make target/test-guard`；macOS/Linux 定向示例：

```bash
./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-dom
```

选择集成 target 加 `--test <target名>`；选择具体测试加名称过滤器；显示输出加 `-- --nocapture`。`--compile-first` 先在不设内存阈值的阶段编译，再保护测试运行。

多进程测试必须先构建与当前源码及 feature 一致的 renderer/compositor/image-decoder，不能复用旧版本子进程；全量入口已包含此步骤。

CSS、布局、绘制或 Web API 语义兼容性修复必须附带对应 WPT/reftest。优先用 `make import-wpt TEST=<上游用例> REF=<参照页> NOTE="Rxxxx 修复"` 导入 `tests/wpt-runner` 常驻断言集，并记入 `imported-tests.txt`；无法导入时至少补等价本地 reftest/单测。用例与修复同一提交。该约定来自 2026-08-07 调研 P1/P4：通过的上游用例进入常驻 CI，避免已修行为退化。

## 按影响选择检查

禁止因文件不在白名单、扩展名不是 `.md` 或准备提交，就自动执行浏览器全量检查。未改变产品代码、构建依赖或运行资源时，不执行无关的 cargo fmt/clippy、浏览器构建、workspace 测试、reftest、基准和覆盖率。

| 范围 | 相关验证 |
|---|---|
| `docs/**`、`**/*.md`、`evolution-drafts/**` | 文档链接、结构及实际修改的数据/配置语法 |
| `.agents/**`、`.claude/**`、`.trae/**`、`.cursorrules` | 规则与引用一致性、JSON/YAML；Skill 脚本只做受影响检查或测试 |
| `website/**` | HTML/CSS/JS/JSON、资源链接；行为变化需要时预览或交互检查 |
| `.github/**` | 工作流 YAML、CODEOWNERS 及受影响工作流/脚本逻辑 |
| 根 `LICENSE`、`banner.svg`、`.gitignore` | 文本、SVG 或忽略规则，不因无 `.md` 后缀启动 Rust 检查 |

目录豁免以未影响浏览器产品为前提。文件若被产品编译、运行或测试直接引用，按实际消费者验证。`examples/**`、测试 fixture/基线、`scripts/**`、`tools/**`、`.config/**` 不整目录豁免，也不默认全量测试：独立工具只测工具，测试资产只跑受影响用例。`build-support/**` 产品构建代码、Cargo 依赖/feature 和产品运行资源须验证受影响构建与行为。混合提交按实际产品变化决定门禁，不因附带文档或 Skill 扩大范围。

适用于当前变更的已完成检查直接复用；只有新改动、失败或新影响证据才补测，不能为提交重复跑昂贵测试。上述豁免仍须 `git diff --check`、适用检查及 `lei-pre-commit-guard` PASS。

Rust 产品代码提交前必须：

- 用 `make test` 通过 workspace 测试。
- `cargo fmt --all -- --check` 无 diff；有 diff 先用 `cargo fmt --all` 修复。
- `cargo clippy --workspace --all-targets -- -D warnings` 无 warning/error，保持与 CI 同等严格。
- 默认 v8 feature 因环境无法编译时，至少在可编译 feature 下跑 clippy（如 `--no-default-features --features quickjs`），并在提交说明注明覆盖范围。

此前未在本地跑严格 clippy，曾使 `register_callback` 的 `Box<dyn Fn>` 触发 `type_complexity`，造成全平台 CI 失败。

## 提交前安全扫描

执行 `git commit` 前必须调用 `lei-pre-commit-guard` 扫描暂存区。PASS 允许进入提交步骤；BLOCK 输出发现并等待用户确认，或修复后重新扫描。用户明确要求跳过可豁免。扫描通过不自动授予提交权限。

## CI 覆盖

[PR Precheck](../../.github/workflows/pr-precheck.yml) 对非 Draft PR 执行格式和 workspace check，忽略纯 Markdown/docs 变更。[主 CI](../../.github/workflows/ci.yml) 由手动触发或 OWNER 批准 PR 触发，包含多平台 clippy、测试、启动冒烟和 Android 构建；macos-x86_64 跳过测试执行，桌面 release 构建需手动启用。实际范围以工作流条件和矩阵为准。

## 运行安全

无人值守（Rally、CI、长期自动执行）的测试、构建和 clippy 必须由“进程树内存上限 + 墙钟超时”包裹，禁止裸跑 cargo test、npm test、pytest、go test、make 等原始入口。单个死循环或无限 realloc 会触发系统 OOM，连带回收父 Agent、tmux 或整个流程；包裹器应只杀失控进程树。

本仓入口与阈值见 [Rally 运行规则](../rally/run-rules.md)、[OOM 防护](../rally/oom-guard.md) 和 [Makefile](../../Makefile)。`make test` / `make reftest` 经 `scripts/test-guard.rs` 包裹运行阶段，但 Makefile 的磁盘守卫不代表所有编译阶段都有内存与超时保护。Linux 可用 `make dev-guard` 或既有 guarded 入口作 cgroup 兜底；macOS 使用 test-guard。其他语言沿用适合环境的内存/超时守卫，不绕过既有入口。

病态输入实验在交互式会话同样适用：复现、最小化 fuzz 工件（`timeout-*` / `OOM-*`）或研究解析器/算法对病态输入的行为，一律走 [scripts/fuzz-repro-guard.sh](../../scripts/fuzz-repro-guard.sh)。先无限制预编译，再用 8G 地址空间上限和墙钟超时执行；不可省略预编译，`ulimit -v` 会连链接器一起限制。2026-10-07 两个约 35GB 最小化进程把 46GB 机器拖到系统 OOM，连带杀死 `oom_score_adj=500` 的远程入口服务；有人观察不代表安全。
