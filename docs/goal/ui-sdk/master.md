# Zero UI SDK 抽取与浏览器迁移 — 运行时控制面板

> **入口文档**：`docs/goal/ui-sdk.md`（goal contract，稳定）。本文件是持续演进的增量控制面，**不是**一次性交付物。创建第一版只表示治理框架建立，**不表示**核心目标已被覆盖。
> **上游权威**：`docs/specs/ui-sdk-spec-rfc.md`（v1.6.1）。实现具体接口/模块/组件时必须查阅 spec 对应章节。
> **姊妹目标**：`docs/goal/rendering-compat.md`（只读引用，不得改写；其 `make test`/`make reftest`/`make product-smoke` 主线不得因本目标退化）。
> **工作分支**：`ui-sdk`。

---

## Active Milestone

**M2 — 浏览器首批组件迁移 + WebView adapter + text foundation 接入（spec §M2）**。依赖：M1（已完成）。

**✅ M1 UI SDK 核心骨架已收口（2026-07-01）**：23 个新 crate（22 通用 + 1 共享 + 2 耦合点 adapter-webview/chrome）全部存在、可编译、有无窗口单测；`cargo build --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all` 全净；DC-1 依赖隔离机械验证 PASS（`evidence/dep-isolation-20260630-234530.txt`）；DC-17 coverage 聚合 line 93.00% / function 93.12% / region 93.41%（≥85%，per-crate 除 foundation/text stub 外全部 ≥85%）。M1 过程/决策/证据已归档 `archive/m1-skeleton-complete.md`。

**M2 目标**（按 spec §M2 / goal §Ordered Next Milestones）：
1. 抽取滚动条/按钮/文本输入/菜单/popup/toolbar 到 `ui/widgets`；SearchField/SuggestionList/TabBar 等到 `ui/patterns`；新增 `browser-ui/chrome` 首批领域组件（spec §8.4.1A 映射）。
2. text foundation 真实接入 `ui/render` 与 `zero-webview`（复用 fontdue/swash/rustybuzz，DC-11 / TBD-8）；`ui/render` Scene → render-foundation 后端 trait（TBD-2）。
3. 定义 `BrowserChromeModel` + `BrowserAction` + desktop/tablet/phone shell 共享合约；browser menu/shortcut/context menu 接 `ui/commands`；PermissionPrompt/DownloadPanel/SiteInfoPanel 接 `ui/overlay`；Downloads/Bookmarks/History/TabOverview 接 `ui/collections`。
4. **逐组件灰度迁移**（shim / feature-flag），任意提交点浏览器可运行（DC-14 零退化硬门禁）；涉及渲染/布局变更跑 `make product-smoke`。

**铁律**：M2 触碰浏览器运行时，必须保持 `make test` / `make reftest` / `make product-smoke` 不退化（姊妹目标 `rendering-compat` 主线）。

**🟢 M2 进展（2026-07-01）**：
- **DC-11 phase 1**：foundation/text 真实文本后端（`backend::FontdueBackend`，fontdue+rustybuzz 实现 FontProvider/TextShaper/TextMeasurer + 14 测，含 Ahem.ttf 真实 shaping/measure/换行/caret）。shaping/text_blob/text_measure stub 0→100%。
- **DC-11 phase 2 + TBD-2**：ui/render 接入 foundation/text —— 新增 `RenderPrimitive::TextBlob`（预 shape 文本单元，承载 `TextBlob`）+ `SceneRecorder::draw_text_blob`；新增 `backend::RenderBackend` trait + `paint_scene`（TBD-2：Scene→光栅后端抽象，ui/render 不直接依赖 render-foundation）；scene_snapshot 加 TextBlob 确定性分支；3 个 paint_scene 测试用真实 FontdueBackend shape→TextBlob→mock 后端验证派发。foundation/text 给 TextBlob 加 Serialize 并顶层导出。
- DC-1 复核：ui/render 经 foundation/text 仍**零浏览器业务 crate 依赖**（foundation/text 仅纯文本/字体栈 fontdue/rustybuzz/hashbrown/ttf-parser/unicode-*）。**本步 SDK-only，未触碰 render-foundation/浏览器**（无 product-smoke 风险）。
- **DC-7 批次**：browser-ui/chrome §8.4.1A 实现 6 个领域组件——`PageLoadIndicator`(ProgressIndicator)、`SecurityBadge`(Badge+Tooltip)、`BookmarksBar`(Toolbar)、`BrowserTabStrip`(TabBar)、`FindBar`(TextInputState+StatusBubble)、`AddressBar`(TextInputState+SuggestionList+SecurityBadge)。遵循 props+build+on_activate 模式（由通用 widgets/patterns 组合，输出进 UI scene）；加 zero-ui-patterns 依赖；19 个新测。**SDK-only，未触碰 apps/browser**（无 product-smoke 风险）。
- **DC-7 收尾（12/12）**：ui/widgets 补 FR-009 首批缺失 widget `IconButton`/`ContextMenu`(并入 menu.rs)/`Toggle`；BrowserAction 加 download actions（OpenDownload/CancelDownload/ShowDownload）；chrome 补 `BrowserMenu`(Menu+ContextMenu)、`PermissionPrompt`(DialogScaffold+Toggle)、`SiteInfoPanel`(Popover+ListView+权限切换)、`DownloadPanel`/`DownloadItemView`(Popover+ListView+ProgressIndicator)。**§8.4.1A 全 12 组件 + FR-009 widget 集合补齐**。
- 门禁：`cargo build --workspace` + clippy `-D warnings` + fmt 全净；foundation/text 27 + ui/render 10(+3 集成) + ui/testing 4 + ui/widgets 29 + browser-ui/chrome 35 tests 全绿；DC-17 聚合 line 94.28% / function 94.92% / region 94.27%（per-crate 全部 ≥85%；新组件/控件 95–100%）。

**M2 剩余**：①统一 render-foundation 现有 font 栈到 foundation/text（物理迁移/复用，**触碰渲染后端需 product-smoke**，使 zero-webview 也走 foundation/text，DC-11 完整闭环）；②chrome 组件 paint → scene snapshot（widget paint 管线接通后，DC-7 完整验收）+ chrome 组件 → `apps/browser` 灰度接线；③`BrowserChromeModel`+`BrowserAction` + desktop/tablet/phone shell（DC-12）；④apps/browser 逐组件灰度迁移（DC-14）；⑤render-foundation 实现 `RenderBackend` trait（TBD-2 后端侧，product-smoke）。

## Done Criteria 进度

| DC | 标题 | 状态 | 证据 / 备注 |
|----|------|------|------------|
| DC-1 | 目录与依赖隔离 | ✅ skeleton（M1） | ui/ + foundation/text + browser-ui/chrome crate skeleton 全在；DC-1 机械验证 PASS（`evidence/dep-isolation-20260630-234530.txt`） |
| DC-2 | 三棵树 + 单向数据流 + retained | 🟡 skeleton（M1） | ui/core Widget/Element/RenderNode/EventResult/Invalidation 类型 + UiTree reconcile 单测；完整 retained 能力随 M2 迁移深化 |
| DC-3 | WebViewWidget 自定义组件 | ⬜ M2 | adapter/webview skeleton 在 Wave 4；真实接入 M2 |
| DC-4 | 滚动语义与滚动条边界 | ⬜ M2 | ScrollBar skeleton 在 Wave 2；迁移自 page_scroll.rs 在 M2 |
| DC-5 | 主题系统 | 🟡 skeleton（M1） | ui/core::theme 类型 + ui/runtime::theme_provider 在 Wave 1/2 |
| DC-6 | YAML DSL + 完整表达式语言 | ⬜ M3 | ui/dsl schema skeleton 在 Wave 3；完整表达式语言 M3 |
| DC-7 | 首批组件（通用+组合+浏览器） | 🟡 组件 12/12（M2） | FR-009 widget 集合补齐（+IconButton/ContextMenu/Toggle）；browser-ui/chrome §8.4.1A 全 12 组件已实现（props+build+on_activate+测试）；剩余：chrome 组件 paint→scene snapshot（widget paint 管线接通后）+ `apps/browser` 灰度接线 |
| DC-8 | 无障碍/焦点/IME | 🟡 skeleton（M1） | ui/core::focus/semantics + ui/runtime::ime/accessibility |
| DC-9 | 局部失效刷新 | 🟡 skeleton（M1） | ui/core::invalidation needs_layout/paint 区分单测在 Wave 1 |
| DC-10 | 国际化资源与 message id | 🟡 skeleton（M1） | ui/i18n IF-007 接口在 Wave 2 |
| DC-11 | 共享文本/字体基础层 | 🟡 ui/render 已接入（M2） | foundation/text 真实后端（FontdueBackend）+ ui/render `RenderPrimitive::TextBlob`/`RenderBackend`/`paint_scene` 已接入（phase 1+2）；render-foundation 字体栈统一到 foundation/text + zero-webview 接入待续（需 product-smoke） |
| DC-12 | 响应式/自适应 + 移动 chrome | ⬜ M2/M4 | WindowMetrics/ViewportClass 在 ui/core::layout（Wave 1）；shell 在 M2/M4 |
| DC-13 | 完整应用级 UI 能力（13 域） | 🟡 skeleton（M1） | 13 域接口 + 最小单测在 Wave 3；浏览器接入在 M2-M4 |
| DC-14 | 浏览器迁移完成 + 零退化 | ⬜ M2-M4 | 浏览器运行时本轮不触碰 |
| DC-15 | 移动端运行时 | ⬜ M4 | — |
| DC-16 | 测试与质量不可退让 | 🟡 守门中 | 新 crate scoped test 全绿；详见 Testing & Quality Gates |
| DC-17 | Coverage 量化 | 🟡 持续推进（M1 阶段达标） | 聚合 line 94.28% / function 94.92% / region 94.27%（≥85%）；per-crate **全部 ≥85%**（chrome 12 组件 + IconButton/Toggle 等新控件 95–100%、foundation/text ~96%） |
| DC-18 | 证据持久化与文档自洽 | 🟡 守门中 | evidence/ 持续落盘；本表与 Latest Evidence 自洽 |

图例：⬜ 未开始 / 🟡 进行中或 skeleton / ✅ 完成。

## Current Proven Baseline（首轮复核，2026-06-30）

复核结论与入口文档「Current Proven Baseline」一致，**均不存在**项确认：

- `ui/` 顶层目录 — **不存在**。
- `foundation/` — **不存在**（text foundation 待新建）。
- `browser-ui/` — **不存在**。
- `docs/goal/ui-sdk/{master.md,archive/,evidence/}` — **不存在**（本轮创建）。
- Widget/Element/Render 三棵树、retained 运行时、Theme/i18n/DSL/表达式语言 — **均不存在**。

已有可复用基础（SDK 将扩展而非重写）：

- `apps/browser/src`：自绘 UI 耦合在 `app.rs`/`app_render.rs`/`app_input.rs`/`page_scroll.rs`/`colors.rs`/`app_platform.rs`/`layout.rs` 等（合计约 19.7k 行；多个文件 >2000 行：`app_input.rs` 2910、`app_render.rs` 2816、`main.rs` 2346、`headless.rs` 1761 — 迁移时需注意拆分）。
- `crates/render-foundation`：渲染后端（GPU/CPU 图元 + `src/font/{cache,loader,shaper}` 字体栈 + image_cache）；依赖 wgpu/tiny-skia/resvg/png。
- `crates/host-runtime`：winit 0.30 窗口/事件/IME/surface。
- `crates/webview`：嵌入式 WebView 稳定边界。
- `crates/browser-shell`：浏览器业务状态（标签/书签/历史/设置/下载，UI-agnostic）。
- workspace 已声明依赖：`fontdue 0.9`/`swash 0.2`/`rustybuzz 0.20`/`unicode-bidi 0.3`/`taffy 0.7`(本地 patch)/`winit 0.30`/`serde`/`serde_json`/`thiserror`/`tracing`/`slotmap`/`hashbrown`/`compact_str`/`parking_lot`。

## Testing & Quality Gates（本机实测基线）

**⚠️ 本机 `make test` 基线状态：RED（环境性，非本目标引入）**

- 本轮首轮跑 `make test`（= `test-guard -- cargo test --workspace --exclude zero-render-foundation`）在 **`zero-script-sandbox` (lib test) 链接阶段失败**：MSVC `link.exe` 报 `LNK2019: 无法解析的外部符号 __imp_EventRegister / RegOpenKeyExW / ...`（`advapi32` 系符号），源自 `rusty_v8` 的 debug test 二进制。
- **关键定性**：`cargo build --release -p zero-script-sandbox` **成功**（0.86s）；仅 debug **test** 二进制链接失败。即 V8 库与环境本身正常，是 rusty_v8 debug-test 在本 MSVC BuildTools 上未自动链接 `advapi32` 的既有环境问题。
- **与本目标无关**：`zero-script-sandbox` 属禁止修改区（脚本 sandbox），且为环境/工具链问题，非代码/逻辑失败。CI（Linux，真后端）按 `ci.yml` 跑全量 `cargo test --workspace` 正常。
- **UI SDK 验证路径**（本轮及后续）：对新增 `ui/*` + `foundation/text` + `browser-ui/chrome` 使用 **scoped test-guard**：`./target/test-guard -- cargo test -p zero-ui-<crate>`（OOM 防护 + 不触碰 script-sandbox）；并用 `cargo build --workspace` 确认不破坏其它 crate 编译。最终门禁仍以 `make test` 为准（待环境恢复或 CI 验证）。
- 跟踪项：本机 script-sandbox debug-test 链接环境问题记入「未解决缺口」，不阻塞本目标推进（属环境，非 UI SDK 代码）。

**其它门禁**：单 `.rs` ≤ 2000 行；`cargo fmt` 无变更；clippy `-D warnings`；新增能力必须带测试；依赖仅 MIT/Apache-2.0/BSD 且最小化。

## Coverage 基线

- 全仓 floor（来自 `rendering-compat` 基线）：line 95.46% / function 96.94% / region 94.88%。本目标不得显著下降。
- **`ui/*` + `foundation/text` + `browser-ui/chrome` DC-17 当前（2026-07-01，M2 DC-7 收尾 12/12 + FR-009 widget 补齐后）**：
  - 聚合：**line 94.28% / function 94.92% / region 94.27%**（4719 行，超 85% 目标）。
  - 演进（同口径）：M1 基线 89.89% → M1 per-crate 抬升 93.00% → M2 foundation/text 真实后端 93.66% → M2 ui/render 接入 93.40% → M2 chrome §8.4.1A 8 组件 93.88% → M2 chrome 12/12 + widget 补齐 94.28%。
  - 当前证据：`evidence/coverage-20260701-031400.txt`。
  - 命令（统一口径）：`cargo llvm-cov -p zero-text-foundation -p zero-ui-* ... -p zero-browser-chrome --summary-only --ignore-filename-regex '[\\/](crates|apps)[\\/]' -- --test-threads=1`（`--ignore-filename-regex` 必须用 `[\\/](crates|apps)[\\/]` 匹配 profdata 绝对路径中的目录分量，`^` 锚定不生效）。口径含 `#[cfg(test)]` 内联模块（与仓库 `check-coverage.sh` 全仓口径一致，趋势可比）。
- **per-crate line 曲线**（M2 DC-7 收尾后；全部 ≥85%）：

  | crate | line | 变化 |
  |-------|------|------|
  | ui/assets · ui/collections · ui/forms · ui/platform · ui/commands · ui/overlay · ui/restoration · ui/gestures · ui/dsl | 100% / ~99% | M1 抬升 |
  | foundation/text | ~96% | M2：shaping/text_blob/text_measure 0→100、backend.rs 93 |
  | browser-ui/chrome | ~99% | M2 DC-7：12 组件全部 95–100%（含本轮 browser_menu/permission_prompt/site_info_panel/download_panel） |
  | ui/widgets | ~95% | M2 DC-7：补 IconButton/Toggle/ContextMenu（新控件 100%） |
  | ui/render | ~89% | M2：backend.rs（RenderBackend+paint_scene）92、paint_ctx 87、render_node 100 |
  | ui/animation · ui/patterns · ui/devtools · ui/navigation · ui/design-system · ui/i18n · ui/adapters · ui/core | 92–98% | — |
  | ui/runtime · ui/testing | 85–88% | 可选继续抬边角 |

- **下一步 coverage 推进**：所有 per-crate 已 ≥85%（DC-17 阶段达标）。可选抬 ui/render(~89%)/ui/runtime/ui/testing 边角，随 M2 组件迁移自然增长。

## 依赖决策日志

| 决策 | 结论 | 理由 | Spec 锚点 |
|------|------|------|-----------|
| TBD-8 text foundation 落点 | **新建独立 `foundation/text`（`zero-text-foundation`），M2 已落地真实 `FontdueBackend`**（fontdue+rustybuzz，不依赖 render-foundation） | render-foundation 耦合 wgpu/png/resvg/tiny-skia 图形后端；纯文本/字体基础层独立避免被图形后端污染，UI 与 WebView 都能依赖而不拖入 GPU 栈。M2 第一阶段：foundation/text 自立真实后端；第二阶段（风险步，需 product-smoke）把 render-foundation 现有 font 栈统一到 foundation/text | spec §6.5A / IF-008 / TBD-8 |
| TBD-2 ui-render 与 render-foundation 依赖方向 | **`ui/render` 定义自己的 Scene/RenderNode/PaintCtx 抽象 + M2 已加 `RenderBackend` trait + `paint_scene`**；ui/render 不直接依赖 render-foundation，通过 trait 由 render-foundation 后端实现（待 product-smoke） | 通用 UI 层不被 wgpu 后端耦合；ui/render 只依赖 foundation/text（纯文本层），不依赖 GPU 栈 | spec §8.4.1 / TBD-2 |
| TBD-9 text shaping/font 依赖 | **复用 workspace 已声明** fontdue/rustybuzz/unicode-bidi；M2 已把 fontdue+rustybuzz 加入 foundation/text（swash 暂未用） | 已在 workspace.dependencies；零新增外部依赖；fontdue=度量/光栅、rustybuzz=OpenType shaping | spec §6.4 / TBD-9 |
| TBD-7 i18n 依赖 | **M1 手写 minimal plural/RTL**，不引入 ICU4X/Fluent | 接口先行；依赖评估留 M3 | spec §6.5A / TBD-7 |
| TBD-6 表达式 parser | **M3 评估**，M1 不涉及 | DSL 表达式在 M3 | TBD-6 |
| TBD-1 serde_yaml | **M3 评估**，M1 dsl 只立 schema skeleton | — | TBD-1 |

依赖自治条款（已与用户确认）：YAML 解析、表达式 parser、i18n、text shaping/font、布局均由执行器自主决策，硬约束 = 仅 MIT/Apache-2.0/BSD、最小化、优先复用 workspace 已有 crate、论证写入本表 + archive。

## Latest Evidence

- `evidence/test-20260630-223559.txt` — 首轮 `make test` 实测：RED（script-sandbox debug-test V8 链接环境失败；release 构建 + CI 绿；定性见上）。
- `evidence/dep-isolation-20260630-234530.txt` — **DC-1 机械验证 PASS**：22 通用 crate 零浏览器依赖；adapter-webview→zero-webview；chrome→ui/*+browser-shell+adapter-webview。
- `evidence/capability-matrix-20260630-234530.md` — M1 能力矩阵 + DC skeleton 证据锚点 + 未解决缺口。
- `evidence/coverage-20260701-021806.txt` — DC-17 抬升前基线（2026-07-01）：聚合 line 89.89% / function 89.98% / region 90.65%。
- `evidence/coverage-20260701-024441.txt` — **DC-17（M2 foundation/text 真实后端接入后，2026-07-01）**：聚合 line 93.66% / function 93.92% / region 93.79%；per-crate 全部 ≥85%（foundation/text ~96%，三个 stub 0→100%）。
- `evidence/coverage-20260701-025355.txt` — **DC-17（M2 DC-11 phase 2：ui/render 接入 foundation/text，2026-07-01）**：聚合 line 93.40% / function 93.87% / region 93.58%；per-crate 全部 ≥85%（ui/render 新增 backend.rs/paint_ctx TextBlob 路径 ~89%）。
- `evidence/coverage-20260701-030502.txt` — **DC-17（M2 DC-7：browser-ui/chrome §8.4.1A 组件批次，2026-07-01）**：聚合 line 93.88% / function 94.46% / region 93.96%；per-crate 全部 ≥85%（chrome 新 6 组件 96–100%）。+ DC-7 6 组件 component test 锚点（address_bar/find_bar/browser_tab_strip/bookmarks_bar/security_badge/page_load_indicator）。
- `evidence/coverage-20260701-031400.txt` — **DC-17（M2 DC-7 收尾 12/12 + FR-009 widget 补齐，2026-07-01）**：聚合 line 94.28% / function 94.92% / region 94.27%；per-crate 全部 ≥85%（chrome 12 组件 + IconButton/Toggle/ContextMenu 新控件 95–100%）。

**M2 门禁实测（2026-07-01）**：
- `cargo build --workspace` — Finished（0 错误）。
- `cargo clippy --workspace --all-targets -- -D warnings` — Finished（0 警告）。
- `cargo fmt --all --check` — 净（0 diff）。
- foundation/text 27 + ui/render 10(+3 集成) + ui/testing 4 + ui/widgets 29 + browser-ui/chrome 35 tests 全绿（scoped test-guard）。
- DC-1 复核：chrome 经 zero-ui-widgets/patterns 仍是允许耦合点；ui/* 通用 crate 不受影响。
- DC-17 coverage：聚合 94.28%，per-crate 全部 ≥85%。

## Next Steps

1. **render-foundation 字体栈统一到 foundation/text**（DC-11 完整闭环；**触碰渲染后端，需 `make product-smoke`**）：让 `crates/render-foundation/src/font` 复用 foundation/text（或 re-export `FontdueBackend`），使 zero-webview 也走 foundation/text；并让 render-foundation 实现 ui/render 的 `RenderBackend` trait（TBD-2 后端侧）。M2 风险步骤，须 product-smoke 守 welcome.html 不退化。
2. **`BrowserChromeModel` + `BrowserAction`** desktop/tablet/phone shell 共享合约（DC-12）：把 12 个 chrome 组件编排进 shell 模型，按 WindowMetrics/ViewportClass 选 desktop/tablet/phone 布局。
3. **chrome 组件 paint → scene snapshot**：通用 widget 的 Widget::paint 管线接通后，把 chrome 组件渲染到 Scene 并 golden snapshot（DC-7 完整验收）。
4. **apps/browser 逐组件灰度迁移**（shim/feature-flag，DC-14 零退化；涉及渲染/布局变更跑 `make product-smoke`）。
5. `ui/examples`（counter/form/browser-shell-demo）随 M3 落地（DC-14）。
6. 跟踪项：本机 `make test` 受 script-sandbox debug-test V8 链接阻塞（环境性）；render-foundation 字体栈统一前 welcome.html 渲染走旧路径，未受本轮影响。
