# JavaScript、DOM 绑定与脚本沙箱

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 V8/QuickJS 沙箱、原生 DOM 身份、页面任务检查点、module、Dedicated Worker、通用 Web API 与 WASM。宿主入口分别为 WebView、renderer JS worker、browser tab worker 和 WPT；页面对象、线程数据源及测试 fixture 并不相同。

### User Task Navigation

| 用户任务 | 源码入口 | 相关知识 |
| --- | --- | --- |
| 脚本超时、回调增长、退出卡住 | `script-sandbox/src/*runtime.rs`、`renderer/src/js_worker.rs` | [执行器](runtimes/index.md) |
| query 返回旧对象、apply 后读空 | `dom_bindings`、`js_dom_bridge`、`webview.rs` | [DOM 桥](dom-binding/index.md) |
| observer、Promise、输入先后次序 | `page_scripts.rs`、`timer_bridge.rs`、`user_actions.rs` | [事件循环](event-loop/index.md) |
| fetch/storage/Service Worker | `fetch_bridge.rs` 等宿主桥 | [Web 数据](../web-data/index.md) |
| Canvas、媒体 | 页面桥与对应底座 | [图形](../graphics/index.md)、[媒体](../media/index.md) |
| feature 与宿主模式差异 | Cargo 门控、运行时开关 | [脚本分支](../runtime/conditions/script-features.md) |

验证入口为 [zeroweb-verify](../../../zeroweb-verify/SKILL.md) 及各叶子的实际测试文件。本验证入口为并行创建的未提交资产，不归本页 HEAD；本域本轮仅来源/内容自查，未实跑产品，待独立语义核验。

## Architecture
- 页面 JS 能力是引擎、native DOM、shim 与宿主 bridge 的组合；只安装原生模板或只构造某 API 对象，不能证明宿主能力及异步完成通路存在。 (`crates/engine/src/dom_bindings/mod.rs`, `crates/engine/src/js_dom_shim/part05.js`, `crates/webview/src/webview.rs`)
- 脚本执行结果与宿主状态落定是两阶段：JS callback/microtask 完成后还要 drain/apply 并刷新查询或几何，跨线程路径不共享 WebView 的 live Rc。 (`apps/renderer/src/page_scripts.rs`, `apps/renderer/src/js_worker.rs`, `crates/webview/src/webview.rs`)

## Child Knowledge Nodes
- [脚本执行器与线程运行时](runtimes/index.md) — 回调表、Context、超时、线程等待和退出。
- [原生 DOM 与 shim 协作](dom-binding/index.md) — 对象身份、快照/活文档、集合与 CE 接线。
- [页面事件循环与用户动作](event-loop/index.md) — execute/apply、observer、生命周期与可取消默认动作。
- [ES module、import map 与动态脚本](modules.md) — 依赖解析、动态 import 缓存、预取收口。
- [Dedicated Worker 与消息传递](workers.md) — 页面近似 Worker 与 Rust 独立运行时的区别。
- [Web API shim、宿主桥与实现边界](web-platform-apis.md) — 编码、压缩、随机源、FileReader 与 messaging。
- [非页面 WASM 沙箱与页面 WASM 的边界](wasm.md) — 后端、自动桥、JS 重入与原生页面能力。
