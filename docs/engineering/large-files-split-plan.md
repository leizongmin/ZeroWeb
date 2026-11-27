# 超长源码文件分阶段拆分计划

> 更新日期：2026-11-27
> 状态：已建档排期（不在当前 WPT 冲刺期中途拆解，待对应并行流收敛后按批落地）

## 背景与约束

[AGENTS.md](../../AGENTS.md) 要求单个源码文件不超过 2000 行（特别是 `.rs`），超限时按职责高内聚、低耦合拆分。

截至 2026-11-27 基线统计：

- **非测试 Rust 源码（`.rs`）**：共 **86 个文件**超过 2000 行，其中 18 个超过 5000 行，最大单文件为 [service_worker.rs](../../crates/script-sandbox/src/service_worker.rs)（11,804 行）与 [webview.rs](../../crates/webview/src/webview.rs)（11,680 行）。
- **DOM JS Shim（`crates/engine/src/js_dom_shim/*.js`）**：共 **12 个文件**超过 2000 行，其中 `part01.js`–`part06.js` 均在 15,880–37,609 行之间，最大为 [part03.js](../../crates/engine/src/js_dom_shim/part03.js)（37,609 行）。

当前 `wptfyi-dom`、`wptfyi-css`、`html-semantics`、`rendering-compat` 等并行流正在密集修改 `crates/engine`、`crates/style-system`、`crates/layout-engine` 及 `js_dom_shim`。**在冲刺中途对热点文件做大规模物理拆分，会直接造成跨分支高频 rebase 冲突。** 因此本计划采用「按并行流碰撞域分组、冻结后分批拆分、只拆结构不改语义」的节奏执行。

## 当前超限分布（按 crate）

统计口径：排除 `tests/`、`*_tests/`、`*_tests.rs`、`tests.rs` 后，行数 > 2000 的 Rust 源码文件。

| Crate / 目录 | 超限文件数 | 最大行数 | 合计行数 | 当前并行流热度 |
| --- | ---: | ---: | ---: | --- |
| `crates/engine` | 19 | 9,871 | 85,918 | 极高（`wptfyi-dom` / `html-semantics` / `rendering-compat`） |
| `crates/layout-engine` | 14 | 6,706 | 54,923 | 高（`wptfyi-css` / `layout-compat`） |
| `crates/style-system` | 10 | 6,718 | 37,728 | 高（`wptfyi-css` / `rendering-compat`） |
| `apps/browser` | 12 | 4,510 | 33,544 | 低（适合最先拆分） |
| `crates/script-sandbox` | 3 | 11,804 | 24,181 | 中（`wptfyi-dom` / SW / ES module） |
| `crates/css-parser` | 8 | 3,718 | 21,446 | 中（`wptfyi-css`） |
| `apps/renderer` | 3 | 8,313 | 18,048 | 低—中 |
| `crates/webview` | 2 | 11,680 | 14,727 | 低（适合早期拆分） |
| `crates/render-foundation` | 3 | 4,872 | 11,644 | 低—中 |
| `crates/page-runtime` | 2 | 6,594 | 8,619 | 低 |
| `crates/dom` | 3 | 3,411 | 8,544 | 中 |
| `crates/canvas` | 3 | 3,055 | 8,169 | 低 |
| `crates/net` | 2 | 2,227 | 4,422 | 低 |
| `crates/protocol` | 1 | 2,675 | 2,675 | 低 |
| `crates/taffy-local` | 1 | 2,667 | 2,667 | 低（上游 vendor 补丁，优先保持上游结构） |

## 头部重点文件清单（>5000 行）

### Rust 源码（Top 18）

| 行数 | 文件 | 建议拆分边界 | 批次 |
| ---: | --- | --- | --- |
| 11,804 | `crates/script-sandbox/src/service_worker.rs` | 拆为 `lifecycle.rs` / `fetch_dispatch.rs` / `clients_api.rs` / `cache_bridge.rs` / `mod.rs` | Batch 2 |
| 11,680 | `crates/webview/src/webview.rs` | 按宿主职责拆为 `navigation.rs` / `input.rs` / `storage_cookies.rs` / `media_sw.rs` / `render_loop.rs` / `mod.rs` | Batch 1 |
| 9,871 | `crates/engine/src/animation.rs` | 拆为 `keyframes.rs` / `timeline.rs` / `interpolation.rs` / `transitions.rs` / `mod.rs` | Batch 3 |
| 8,577 | `crates/engine/src/paint/painter/mod.rs` | 按绘制阶段拆为 `background_border.rs` / `stacking.rs` / `replaced.rs` / `effects.rs` / `mod.rs` | Batch 3 |
| 8,313 | `apps/renderer/src/js_worker.rs` | 拆为 `event_loop.rs` / `dom_rpc.rs` / `timers_observers.rs` / `mod.rs` | Batch 2 |
| 7,551 | `crates/script-sandbox/src/es_module.rs` | 拆为 `import_map.rs` / `loader.rs` / `linker.rs` / `dynamic_import.rs` / `mod.rs` | Batch 2 |
| 7,230 | `crates/engine/src/js_dom_bridge/callbacks.rs` | 按 DOM 域拆为 `tree_mutation.rs` / `attr_class.rs` / `geometry_layout.rs` / `forms_media.rs` / `mod.rs` | Batch 3 |
| 7,202 | `crates/engine/src/pipeline/mod.rs` | 拆为 `parse.rs` / `style_layout.rs` / `script_exec.rs` / `incremental.rs` / `mod.rs` | Batch 3 |
| 6,996 | `crates/engine/src/js_dom_bridge/computed_style.rs` | 按 CSS 模块拆为 `box_model.rs` / `typography.rs` / `color_background.rs` / `layout_grid_flex.rs` / `mod.rs` | Batch 3 |
| 6,978 | `crates/engine/src/js_dom_bridge.rs` | 保留薄入口 `mod.rs`，下沉 shim 装配、事件派发与桥接状态至子模块 | Batch 3 |
| 6,718 | `crates/style-system/src/property/apply.rs` | 按属性族拆为 `box_sizing.rs` / `flexbox_grid.rs` / `typography.rs` / `visual_effects.rs` / `shorthands.rs` | Batch 4 |
| 6,706 | `crates/layout-engine/src/intrinsic_sizing.rs` | 拆为 `inline_measure.rs` / `block_measure.rs` / `table_grid_measure.rs` / `replaced.rs` | Batch 4 |
| 6,594 | `crates/page-runtime/src/service_worker_manager.rs` | 拆为 `registration.rs` / `scope_match.rs` / `update_check.rs` / `storage.rs` | Batch 1 |
| 6,380 | `crates/engine/src/paint/painter/text.rs` | 拆为 `shaping.rs` / `decorations.rs` / `selection_caret.rs` / `mod.rs` | Batch 3 |
| 6,284 | `crates/style-system/src/lib.rs` | 下沉 `ComputedStyle` 字段定义、默认值、继承逻辑与结构体方法到 `computed/` | Batch 4 |
| 5,908 | `apps/renderer/src/runtime.rs` | 拆为 `ipc_handler.rs` / `frame_scheduler.rs` / `session_state.rs` | Batch 2 |
| 5,901 | `crates/layout-engine/src/inline_finalization.rs` | 拆为 `line_break.rs` / `bidi_reorder.rs` / `vertical_align.rs` / `inline_box.rs` | Batch 4 |
| 5,719 | `crates/layout-engine/src/tree.rs` | 拆为 `box_gen.rs` / `anonymous_boxes.rs` / `table_fixup.rs` / `mod.rs` | Batch 4 |

### DOM JS Shim（`crates/engine/src/js_dom_shim/`）

| 行数 | 文件 | 拆解方向（待 `wptfyi-dom` / `html-semantics` 阶段收口后） |
| ---: | --- | --- |
| 37,609 | `part03.js` | 按接口域拆出 Proxy getter/setter、HTML 元素 IDL、表格/表单/媒体反射、原型链缓存 |
| 26,254 | `part06.js` | 按能力域拆出 Geometry/Range/Selection、CSSOM/StyleSheet、Observer 与事件补充 |
| 23,467 | `part05.js` | 按能力域拆出 Selector 复验、Worker/SharedWorker、Shadow DOM 与槽位分发 |
| 18,308 | `part04.js` | 按能力域拆出 TreeWalker/NodeIterator、Mutation/Range 与节点操作辅助 |
| 16,951 | `part01.js` | 按能力域拆出 Dialog/Popover/Focus、Document 核心与初始化引导 |
| 15,880 | `part02.js` | 按能力域拆出 ServiceWorker/Storage/Fetch/URL/Abort 与 Web APIs |

> 说明：`js_dom_shim` 采用顺序拼接加载，拆分时需在 `crates/engine/src/js_dom_bridge.rs` 保持加载顺序与作用域闭包语义完全一致，优先通过 `include_str!` 清单按功能模块切分。

## 分批执行策略

### Batch 1：低碰撞宿主与运行时层（可随时启动）

- 目标：`crates/webview`、`crates/page-runtime`、`apps/browser`、`crates/net`、`crates/protocol`、`crates/canvas`。
- 准入条件：无 WPT 热路径锁；单 PR 只拆 1 个文件或 1 个小目录。

### Batch 2：沙箱与渲染进程层

- 目标：`crates/script-sandbox`、`apps/renderer`、`crates/render-foundation`。
- 准入条件：`wptfyi-dom` 当前 Worker / ES Module 切片收口后执行。

### Batch 3：核心引擎与 JS DOM Bridge

- 目标：`crates/engine/src/{animation.rs,paint/,pipeline/,js_dom_bridge*}` 与 `crates/dom`。
- 准入条件：`html-semantics` 与 `wptfyi-dom` 达里程碑冻结窗后按模块逐个拆解。

### Batch 4：样式与布局管线 + JS DOM Shim

- 目标：`crates/style-system`、`crates/layout-engine`、`crates/css-parser` 及 `crates/engine/src/js_dom_shim/part01..06.js`。
- 准入条件：`wptfyi-css` / `layout-compat` / `rendering-compat` 进入阶段性平稳窗后执行。

### 例外：`crates/taffy-local`

- 仅同步上游补丁时按上游文件布局维护，不主动做偏离上游的目录重排。

## 每次拆分的硬性门禁

1. **纯结构提交**：拆分 PR 不夹带功能改动或顺手重构，便于审查与回滚。
2. **禁止新增超限文件**：拆分后的每个新模块不超过 2000 行，且公开 API 保留既有路径（必要时通过 `pub use` re-export 保持兼容）。
3. **验证命令**：每次拆分后按 [quality-gates.md](quality-gates.md) 执行 `cargo fmt --all -- --check`、受影响 crate 的 `target/test-guard` 全量单测及 `cargo clippy --workspace --all-targets -- -D warnings`。
4. **增量止血**：在对应批次落地前，新功能优先落入已有子模块或新建独立子文件，不再向已超 5000 行的单文件追加大块独立逻辑。
