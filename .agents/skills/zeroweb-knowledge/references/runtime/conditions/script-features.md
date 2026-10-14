# 脚本后端、无引擎构建与 feature 并集

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未编译 feature 矩阵。 -->

## Module Structure

说明 browser、renderer、WebView、engine 与 page-runtime 的 Cargo feature 传播，以及 V8/QuickJS/无引擎模式。执行器生命周期详见 scripting；此节点只记录构建条件与真实选择点，不把静态 cfg 核对当编译成功。

### Decision Entry

`Cargo.toml` 的 workspace dependencies 与各 crate `[features]` 共同决定依赖传播；`apps/renderer/src/js_worker.rs`、`crates/script-sandbox/src/lib.rs` 和 `service_worker.rs::create_engine` 决定当前执行器选择。包级 default-features=false 不等于整个依赖图都关闭默认引擎。

## Branching Table

| 维度 | 条件 | 真实行为 | 来源 |
| --- | --- | --- | --- |
| renderer 默认 | 不显式禁默认 | default=["v8"]，转发 engine/webview/script-sandbox/page-runtime | `apps/renderer/Cargo.toml` |
| QuickJS-only | quickjs 且 not(v8) | 使用 QuickJS worker/执行器路径 | `crates/script-sandbox/src/lib.rs`, `apps/renderer/src/js_worker.rs` |
| 并集 | v8 与 quickjs 同时开 | 同名 WorkerRuntime 只导出 V8；QuickJSSandbox 无同名冲突仍导出 | `crates/script-sandbox/src/lib.rs` |
| 无引擎类型层 | v8/quickjs 都不开 | 线程/通道、SW 类型、import map 留在模块树，create_engine 返回 EngineUnavailable | `crates/script-sandbox/src/lib.rs`, `crates/script-sandbox/src/service_worker.rs` |
| engine 基础模式 | 无 script-runtime | optional script-sandbox 与绑定依赖不启用；v8/quickjs 都会开启 script-runtime | `crates/engine/Cargo.toml` |
| browser 产品 | 默认仅 windows-console | optional WebView 不启用，生产 tab_worker 用 stub；显式脚本 feature 另传播 | `apps/browser/Cargo.toml`, `apps/browser/src/main.rs` |
| test-support | browser test-support | 启用 WebView 与进程内 worker 模块；v8/quickjs 用 ?/ 传播已启用的 optional WebView | `apps/browser/Cargo.toml`, `apps/browser/src/main.rs` |
| Android renderer | android-renderer | 可选链接 zero-renderer；未启用时 JNI renderer role 不可执行 | `apps/android-browser/rust/Cargo.toml`, `apps/android-browser/rust/src/lib.rs` |
| V8 bench | script-sandbox bench required-features=["v8"] | QuickJS-only 不编译该 V8 专用 bench target | `crates/script-sandbox/Cargo.toml` |

## Affected Scope

| 模块 | 影响 |
| --- | --- |
| `apps/renderer/Cargo.toml` | 独立 peer 构建的引擎转发 |
| `apps/browser/Cargo.toml` | 发布/测试支持依赖边界 |
| `crates/webview/Cargo.toml` | 嵌入式脚本与 page-runtime 传播 |
| `crates/engine/Cargo.toml` | script-runtime 与原生绑定依赖 |
| `crates/page-runtime/Cargo.toml` | 公共类型依赖与引擎转发 |
| `crates/script-sandbox/src/lib.rs` | 无引擎模块保留与并集同名类型处理 |
| `apps/renderer/src/js_worker.rs` | V8 优先的执行器选择 |
| `apps/android-browser/rust/src/lib.rs` | renderer role 是否可调用 |
| `Makefile` | workspace 与独立应用构建、专用测试 target |

## Gotchas
- default-features=false 只影响该依赖边，Cargo 并集可能从其他消费者重新启用 V8；不能用 workspace 绿态证明单包 QuickJS-only 或无引擎状态。 (`Cargo.toml`, `apps/renderer/Cargo.toml`, `crates/page-runtime/Cargo.toml`, `crates/script-sandbox/Cargo.toml`)
- 无引擎类型层可保留协议不等于无引擎 renderer 可执行；renderer worker 的构造只实现 V8 或 QuickJS 条件分支，本次没有编译无引擎产品矩阵。 (`crates/script-sandbox/src/lib.rs`, `crates/script-sandbox/src/service_worker.rs`, `apps/renderer/src/js_worker.rs`)

## Decisions
- 发布 browser 的 WebView 改为 optional，进程内页面路径收进 test-support，避免产品入口创建内嵌页面 worker；历史依赖细节已继续变化，以当前 Cargo 与 stub 为准。 (`apps/browser/Cargo.toml`, `apps/browser/src/tab_worker_stub.rs`, `git:0f42fb011`)

## Branching Behavior
- V8+QuickJS 并集只对冲突的 quickjs_worker 模块及导出做 not(v8) 门控，不能把所有 QuickJS 类型都删掉；ES module 层仍可能引用 QuickJSSandbox。 (`crates/script-sandbox/src/lib.rs`)
- renderer 的脚本 feature 必须同时转发 WebView、engine、page-runtime 和 sandbox，不能只改直接依赖的 zero-script-sandbox，否则宿主绑定配置不同步。 (`apps/renderer/Cargo.toml`, `crates/webview/Cargo.toml`, `crates/page-runtime/Cargo.toml`)
- engine 的 v8 和 sandbox 的 v8 使用同版本 150.2.0，QuickJS 对齐 rquickjs 0.7/bindgen；绑定安装 escape hatch 需要同一引擎类型实例。 (`crates/engine/Cargo.toml`, `crates/script-sandbox/Cargo.toml`)
- Makefile 对 renderer 的 bin/lib 测试腿分别选择，相关回归钉在 runtime library 模块，不能仅由 --bin 测试腿覆盖推断全部角色契约已测。 (`Makefile`, `apps/renderer/src/lib.rs`, `apps/renderer/src/runtime.rs`)
- Android 可选 zero-renderer 使用该包默认 V8，android-renderer 本身不是 QuickJS 选择器；更换 Android 后端还需核对依赖 feature 转发，而非只改 JNI cfg。 (`apps/android-browser/rust/Cargo.toml`, `apps/renderer/Cargo.toml`)
