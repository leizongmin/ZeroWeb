---
date: 2026-08-20
modules: zero-script-sandbox, zero-page-runtime, zero-browser
---

# workspace feature unification 掩盖单包构建缺 JS 引擎 feature

## 问题

`cargo build -p zero-browser`（`browser.ps1` / Makefile / 打包脚本的构建方式）报 `zero-script-sandbox` 的 `compile_error!("至少需要启用一个JS引擎feature")`，而 `cargo build/test --workspace` 一直通过。

## 根因

两层叠加：

1. workspace 根对 `zero-script-sandbox` 等内部 crate 的声明全是 `default-features = false`，`zero-browser` 的 default 只有 `windows-console`（无引擎）。`6ff4bc311`（service-workers browser owner）把 browser 对 script-sandbox 的依赖从 optional 变为硬依赖后，单包构建图中没有任何 crate 给它开 `v8`/`quickjs`。
2. workspace 级构建时，其他成员（`tests/wpt-runner`、`tests/integration` 对 script-sandbox 的**带默认 feature** 依赖）经 feature unification 把 `v8` 统一了进来——构建绿，掩盖了单包构建已经断裂。

本质冲突：browser 主进程按架构**不应链接 JS 引擎**（引擎只在 renderer），而 `zero-page-runtime::ServiceWorkerManager` 当时直接在 manager 内创建 `ServiceWorkerRuntime`（真引擎），browser 的 SW owner 调它在主进程求值——架构走偏在编译层的表达。

## 解决

按"引擎只在 renderer"的目标架构重构（2026-08-20）：

- `zero-script-sandbox` 允许无引擎编译：`threaded_runtime`/`service_worker` 模块本就引擎无关（仅 `create_engine` 按特性选择引擎），解开门控并给无引擎构建补 `create_engine → Err(EngineUnavailable)` 降级；删除 `compile_error!`。
- `zero-page-runtime` 把 manager 的 runtime 面抽成 `ServiceWorkerRuntimeHost` trait：`LocalServiceWorkerHost`（进程内引擎，webview/WPT/单测）+ browser 侧 IPC 实现（命令转发 renderer、事件注入回 manager）。
- 求值在 renderer 进程执行，`-p zero-browser` 生产依赖图（`cargo tree -e normal -i v8`）不含 v8/rquickjs。

## 如何避免

- 内部 crate 用 `default-features = false` + feature 转发时，**任何新增的硬依赖都必须在单包构建（`cargo check -p <crate>`）下验证**，workspace 构建（unification）不能作为唯一门禁。
- 可选重型依赖（引擎、GPU）从 optional 变硬依赖前，先确认接收方 crate 的 default feature 是否会开启它；不会则保持 optional 或在消费方 feature 中显式转发。
