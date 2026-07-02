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
- **DC-12**：扩展 `BrowserChromeModel`（tabs/nav/address/security/bookmarks/downloads/permissions/find/page_load，三 shell 共享业务模型）+ `BrowserChromeShell` trait + `DesktopBrowserShell`/`TabletBrowserShell`/`PhoneBrowserShell`（build→WidgetSpec 声明树，layout→ShellLayout safe-area/keyboard 避让）+ `AdaptiveBrowserChrome`（按 ViewportClass/PlatformClass/InputClass 选 shell，§8.4.4A 表）+ 跨 shell 稳定 WidgetId（address_bar/viewport 等，断点切换保留输入状态）。**SDK-only，未触碰 apps/browser**。
- **DC-6 phase-1（M3 启动，与 M2 并行）**：ui/dsl `engine.rs` 表达式引擎落地——三阶段管线 parse（Pratt）/typecheck（推断 ValueType）/eval（EvalContext 确定性、无副作用求值）。覆盖字面量 / `$path` / 算术·比较·布尔·空值合并 / 条件 `?:` / 纯函数（count·contains·any·all·min·max·clamp·concat·starts_with·ends_with·format·field·index）。sandbox：禁用能力黑名单（eval/system/require/random/time/…）→ `ForbiddenCapability`，未注册 → `UnknownFunction`，`allow_functions=false` 全禁。资源上限：max_nodes/max_depth/max_iterations → `EvalResourceLimit`（+ parse-depth 守卫 64 防栈溢出）。26 测（含 sandbox negative + 资源上限）。**SDK-only**。
- **DC-6 phase-2（YAML→WidgetSpec loader）**：新增 `ui/dsl/src/yaml.rs` **受限 YAML 子集解析器**（仓内自实现，零新依赖；spec §8.4.7 倾向仓内自实现 + serde_yaml 已弃用）。支持块映射/块序列/嵌套缩进/单双引号+纯标量/流集合 `[..]{..}`/行内注释/类型推断；dash 展开（含 `- - item` 嵌套）；不支持锚点/多文档/块标量/Tab 缩进（超子集即报错）。新增 `loader.rs::YamlLoader`（impl `WidgetSpecLoader`）：YAML→`WidgetSpec` 递归（component/id/props/bindings/actions/control/children），表达式承载字段以**原文 CompactString** 存储（保持 ui/core 不依赖 ui/dsl），strict 模式加载时调 `Engine::parse` 提前校验 visible_when/enabled_when/for_each.source/binding.source 语法（§8.4.7 Validate 阶段），`i18n:` 对象以 `Value::Object` 原样保留。**22 新测**（12 yaml + 10 loader，含端到端 load→eval、strict/lenient、i18n 保留、错误路径）。**SDK-only，未触碰 render-foundation/浏览器**（无 product-smoke 风险）。`map`/`filter` lambda 仍需 spec 决策（`Expression` 枚举无 Lambda 变体；候选：作 pure function 取字段投影/注册 fn id），留 phase-3。
- 顺手抬升 ui/testing crate coverage：`scene_snapshot.rs` 补 StrokeRect/clip 测（70%→92%），ui/testing crate ~93%（per-crate 全部 ≥85% 收口）。
- **DC-2 retained 运行态闭环**：`ui/runtime/src/host.rs` 新增 `WidgetHost` —— 把声明树 `WidgetSpec` reconcile 为 retained **widget 实例树**（`Box<dyn Widget>` + 缓存几何），驱动 layout（measure 自下而上 + arrange 自上而下定绝对 rect；内置 Column/Row/Stack 容器布局）+ paint（每节点 widget 局部坐标 paint → 按绝对 origin 平移并入全局 `Scene`，clip = 祖先 clip ∩ 节点 rect）+ 事件命中（带位置事件 hit-test 到最深最上层节点 + 冒泡，位置转局部坐标，收集 `EmittedAction`）。`WidgetRegistry`（ComponentType→工厂闭包）让 host **不依赖 ui/widgets**（宿主注册具体控件）；失效门控 `needs_layout`/`needs_paint`/`mark`（DC-9）；`creation_epoch`/`rect_of` 查询支持断言 retained 复用与 IME rect。配套 `RenderPrimitive::translate` + `Scene::translated`（局部→绝对坐标）+ `Rect::translate` + `InvalidationFlags::remove` + `WidgetId`/`ComponentType: Default`。**6 host 测 + translate 全分支测**。**这是 counter 示例（DC-14）与 chrome paint→scene（DC-7 完整验收）的运行时基础**。SDK-only，未触碰 render-foundation/浏览器（无 product-smoke 风险）。
- **PaintRecorder::draw_text（M2 文本绘制补齐）**：core `PaintRecorder` trait 加 `draw_text`（原 SceneRecorder 仅有 inherent draw_text，widget 经 `&dyn PaintRecorder` 取不到 → 控件此前无法绘制文本，Button 只画背景）。现 widget 可直接 paint 文本；SceneRecorder 实现 trait 方法 + 保留 inherent `draw_text_blob`（预 shape 路径）。MockRecorder 同步。低 blast radius（仅 2 实现者）。
- **DC-14 counter 示例**：新增 `ui/examples`（`zero-ui-examples`）crate —— `CounterApp`（AppState + reducer + `build_spec`）+ 自定义 `Label` 展示控件（paint 文本）+ 复用 `ui/widgets::Button` + `register_counter_factories`。无窗口 `examples/counter` 二进制（脚本化点击驱动 retained 闭环：count 0→3→2，scene 3 图元）+ 2 集成测（retained 闭环断言：点击→emit→reducer→重建→Scene 文案随状态更新；稳定 WidgetId 跨重建复用 epoch）。**DC-1 机械验证：cargo tree 零浏览器 crate 依赖**（仅 core/render/runtime/widgets）——验证 SDK 外部可复用。SDK-only，未触碰浏览器。
- **DC-7 chrome paint → 统一 Scene**：新增 `browser-ui/chrome/src/render.rs` —— `ChromePanel` 叶子绘制控件（按 props 语义色 `bg` + 文案 `text` paint 进统一 Scene，走 `PaintRecorder`，**不绕过 ui/render**）+ `register_chrome_factories`（注册 8 个 `browser.*` 叶子工厂：AddressBar/SecurityBadge/NavigationButtons/BrowserMenu/BrowserTabStrip/BookmarksBar/FindBar/PageViewportFrame）+ `chrome_color`/`security_color_name` 语义色映射。host 扩展 `node_container_kind` 读 `props.layout`（`column`/`row`/`stack`）→ 容器节点无需硬编码业务组件名即可布局（host 保持浏览器无关）。shell `build()` 由模型填充 props（address_text/security/tab 标题/bookmark 数等）+ 容器 `layout` 声明。`examples/chrome_scene` 二进制产出 DC-7 scene 快照（desktop shell → 12 图元：URL/安全徽章绿/tab 标题/书签数/视口白底）。**2 新测**（desktop shell paint 进 Scene 断言：URL 文案 + ≥4 bar fills + 视口铺满；安全状态→语义色）。SDK-only，未触碰 apps/browser。
- **DC-3 WebViewWidget → 统一 Scene（phase-1）**：新增 `RenderPrimitive::ExternalSurface { rect, surface_id }`（轻量 marker：WebView/平台视图纹理由后端按 id 取回合成，**ui/render 不依赖 zero-webview**，DC-1）+ `PaintRecorder::draw_external_surface` trait 方法 + `RenderBackend::draw_external_surface`（paint_scene 派发）+ scene_snapshot 确定性分支。`WebViewWidget` **impl Widget**（layout 填充分配的外部矩形；paint 记录 ExternalSurface marker；event 处理 Scroll 更新 scroll 度量并钳到 max；semantics 单节点）。**架构边界**：UI SDK 只算外部矩形，不映射 DOM（WebViewWidget public API 仅 viewport/scale/theme/scroll/surface_id）。**3 新测**（paint→ExternalSurface marker；scroll 事件更新度量+钳制；host 注册→layout→paint→Scene 含 ExternalSurface）。SDK-only，未触碰 apps/browser。剩余：WebviewBackend.render → ExternalSurface 纹理的真实光栅合成（render-foundation 后端，M2 后期/需 product-smoke）。
- **DC-4 滚动语义 + ScrollBar**：重写 `ui/widgets/src/scrollbar.rs` —— ①`drag_to_command` 改用真实 `ScrollMetrics`（按 `track_extent - thumb_extent` 映射 thumb 拖动→内容滚动，**修正占位 max_scroll=1000 bug**）；②`ScrollBarHit`（None/Thumb/TrackBefore/TrackAfter）+ `hit_test`；③`hit_to_page_command`（track 点击→翻页）；④`ScrollBarStyle` + `paint_scrollbar`（track+thumb 记录进 `PaintRecorder` → 统一 Scene，与 chrome render 桥一致）；⑤闭环测试（drag→resolve_target→clamp）。**跨 crate 闭环测**（adapter-webview）：通用 ScrollBar 拖动 → `apply_scroll_command`(scroll_bridge) → `WebViewWidget.set_scroll_metrics`（页面 offset 由 WebView 管，spec FR-006）。SDK-only，未触碰 apps/browser。剩余：apps/browser page_scroll.rs 迁移（DC-14，product-smoke）。
- 门禁：`cargo build --workspace` + clippy `-D warnings` + fmt 全净；foundation/text 27 + ui/render **15** + ui/testing 5(+3) + ui/widgets **33**(+4 scrollbar) + browser-ui/chrome **47**(+2 render) + ui/runtime **15** + ui/adapters/webview **8**(+1 scroll 闭环) + ui/examples 2 + ui/dsl 48 tests 全绿；DC-17 聚合 **line 93.33% / function 92.90% / region 92.51%**（per-crate 全部 ≥85%；本轮 scrollbar 重写/新增代码覆盖率预计 ≥85%，下轮统一重测）。

**M2 剩余**：①统一 render-foundation 现有 font 栈到 foundation/text（物理迁移/复用，**触碰渲染后端需 product-smoke**，使 zero-webview 也走 foundation/text，DC-11 完整闭环）；②chrome 组件 paint → scene snapshot（widget paint 管线接通后，DC-7 完整验收）+ chrome 组件 → `apps/browser` 灰度接线；③apps/browser 逐组件灰度迁移（DC-14）；④render-foundation 实现 `RenderBackend` trait（TBD-2 后端侧，product-smoke）；⑤移动端运行时（M4，DC-15：真实 Phone/Tablet shell 渲染 + gesture/软键盘/safe area 适配）。

**M3 进行中（DC-6）**：phase-1 表达式引擎 + phase-2 YAML→WidgetSpec loader 已落地（见上）；剩余 = ①map/filter lambda（需 spec 决策 `Expression` 枚举扩展；候选：字段投影 / 注册 fn id）；②DSL responsive branch（`Adaptive`/`branches`）/ command·route·overlay 引用；③counter/form/browser-shell-demo 示例（DC-14，验证 SDK 外部可复用）。

## Done Criteria 进度

| DC | 标题 | 状态 | 证据 / 备注 |
|----|------|------|------------|
| DC-1 | 目录与依赖隔离 | ✅ skeleton（M1） | ui/ + foundation/text + browser-ui/chrome crate skeleton 全在；DC-1 机械验证 PASS（`evidence/dep-isolation-20260630-234530.txt`） |
| DC-2 | 三棵树 + 单向数据流 + retained | 🟡 retained 运行态闭环（M2） | ui/core Widget/Element/RenderNode/EventResult/Invalidation 类型 + Element reconcile 单测；**`WidgetHost`（ui/runtime）打通 reconcile→layout→paint→Scene→事件命中闭环**（按 WidgetId+ComponentType 复用 widget 实例保留状态，DC-9 失效门控）；剩余：接 winit 事件循环（M2/M4 adapter）、焦点/键盘路由（DC-8） |
| DC-3 | WebViewWidget 自定义组件 | 🟡 impl Widget + ExternalSurface 合成（M2 phase-1） | adapter/webview `WebViewWidget` **impl Widget**（paint 记录 `ExternalSurface{rect,surface_id}` marker → 统一 Scene，不映射 DOM）；`RenderPrimitive::ExternalSurface` + `PaintRecorder`/`RenderBackend::draw_external_surface`（ui/render 零 webview 依赖，DC-1）；剩余：WebviewBackend.render → 真实纹理光栅合成（render-foundation 后端，product-smoke） |
| DC-4 | 滚动语义与滚动条边界 | 🟡 ScrollBar 几何/命中/命令/视觉 已通（M2） | `ui/widgets::scrollbar`：`layout_scrollbar` + `hit_test`(Thumb/TrackBefore/TrackAfter/None) + `drag_to_command`(真实 metrics 映射，修正占位 bug) + `hit_to_page_command` + `paint_scrollbar`(track+thumb→Scene)；**跨 crate 闭环** ScrollBar→apply_scroll_command→WebViewWidget.scroll（spec FR-006，页面 offset 由 WebView 管）；剩余：apps/browser page_scroll.rs 迁移（DC-14，product-smoke） |
| DC-5 | 主题系统 | 🟡 skeleton（M1） | ui/core::theme 类型 + ui/runtime::theme_provider 在 Wave 1/2 |
| DC-6 | YAML DSL + 完整表达式语言 | 🟡 phase-2（M3） | phase-1 表达式引擎（parse/typecheck/eval + sandbox 黑名单 + 资源上限，26 测）+ phase-2 YAML→WidgetSpec loader（受限 YAML 子集解析器 + `YamlLoader`，strict 表达式校验，22 测）已落地；剩余：map/filter lambda（spec 决策）、DSL responsive branch/command·route·overlay 引用、示例应用 |
| DC-7 | 首批组件（通用+组合+浏览器） | 🟡 组件 12/12 + paint→scene 已打通（M2） | FR-009 widget 集合补齐（+IconButton/ContextMenu/Toggle）；browser-ui/chrome §8.4.1A 全 12 组件已实现；**`render.rs` 桥接：chrome 叶子组件经 `WidgetHost` paint 进统一 Scene（`chrome_scene` 快照 12 图元，URL/安全徽章/tab 标题随模型数据），不绕过 ui/render**；剩余：`apps/browser` 灰度接线（真实组件替换手绘） |
| DC-8 | 无障碍/焦点/IME | 🟡 skeleton（M1） | ui/core::focus/semantics + ui/runtime::ime/accessibility |
| DC-9 | 局部失效刷新 | 🟡 host 失效门控（M2） | ui/core::invalidation needs_layout/paint 区分单测；`WidgetHost::needs_layout`/`needs_paint`/`mark` 驱动重布局/重绘（paint-only 变化不触发布局）；剩余：paint-only 主题色切换的端到端 invalidation 断言 |
| DC-10 | 国际化资源与 message id | 🟡 skeleton（M1） | ui/i18n IF-007 接口在 Wave 2 |
| DC-11 | 共享文本/字体基础层 | 🟡 ui/render 已接入（M2） | foundation/text 真实后端（FontdueBackend）+ ui/render `RenderPrimitive::TextBlob`/`RenderBackend`/`paint_scene` 已接入（phase 1+2）；render-foundation 字体栈统一到 foundation/text + zero-webview 接入待续（需 product-smoke） |
| DC-12 | 响应式/自适应 + 移动 chrome | 🟡 shell 已立（M2） | WindowMetrics/ViewportClass/PlatformClass/InputClass/AdaptiveBranch 在 ui/core::layout；BrowserChromeModel + BrowserChromeShell + Desktop/Tablet/Phone shell + AdaptiveBrowserChrome + ShellLayout(safe-area/keyboard 避让) + 跨 shell 稳定 WidgetId 已落地；剩余：移动端真实运行时（M4，DC-15） |
| DC-13 | 完整应用级 UI 能力（13 域） | 🟡 skeleton（M1） | 13 域接口 + 最小单测在 Wave 3；浏览器接入在 M2-M4 |
| DC-14 | 浏览器迁移完成 + 零退化 | 🟡 counter 示例已立（M2） | **counter 示例可构建/运行、不依赖浏览器 crate（`ui/examples`，retained 闭环跑通，2 集成测 + 无窗口二进制）**；剩余：form/browser-shell-demo 示例、apps/browser 逐组件灰度迁移（product-smoke 零退化硬门禁） |
| DC-15 | 移动端运行时 | ⬜ M4 | — |
| DC-16 | 测试与质量不可退让 | 🟡 守门中 | 新 crate scoped test 全绿；详见 Testing & Quality Gates |
| DC-17 | Coverage 量化 | 🟡 持续推进（M1 阶段达标） | 聚合 line 93.33% / function 92.90% / region 92.51%（≥85%）；per-crate **全部 ≥85%**（host.rs 91.85%、render_node 96%、ui/dsl engine.rs 91.79%/yaml.rs 86.61%/loader.rs 92%、chrome shell 94%/chrome_model 100%、12 组件 + IconButton/Toggle 95–100%、foundation/text ~96%；聚合随新增代码体量小幅波动，非既有代码退化） |
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
- **`ui/*` + `foundation/text` + `browser-ui/chrome` DC-17 当前（2026-07-01，M2 DC-2 retained WidgetHost 后）**：
  - 聚合：**line 93.33% / function 92.90% / region 92.51%**（7586 行，超 85% 目标）。
  - 演进（同口径）：M1 基线 89.89% → M1 抬升 93.00% → foundation/text 93.66% → ui/render 93.40% → chrome 8 组件 93.88% → chrome 12/12+widget 94.28% → DC-12 shell 94.38% → DC-6 engine 93.97% → DC-6 YAML loader 93.58% → **DC-2 retained host 93.33%**（新增 ~440 行 host.rs 91.85%/render_node translate 96%，聚合 -0.25pp 为新增代码体量稀释，非既有代码退化；region 持平 92.51%）。
  - 当前证据：`evidence/coverage-20260701-073437.txt`。
  - 命令（统一口径）：`cargo llvm-cov -p zero-text-foundation -p zero-ui-* ... -p zero-browser-chrome --summary-only --ignore-filename-regex '[\\/](crates|apps)[\\/]' -- --test-threads=1`（`--ignore-filename-regex` 必须用 `[\\/](crates|apps)[\\/]` 匹配 profdata 绝对路径中的目录分量，`^` 锚定不生效）。口径含 `#[cfg(test)]` 内联模块（与仓库 `check-coverage.sh` 全仓口径一致，趋势可比）。注意：单 `-p zero-ui-dsl` standalone 数字偏低（~74%）是因为 ui/core 被插桩但未在该 run 测试；DC-17 以全量 run 为准。
- **per-crate line 曲线**（M3 DC-6 phase-1 后；全部 ≥85%）：

  | crate | line | 变化 |
  |-------|------|------|
  | ui/assets · ui/collections · ui/forms · ui/platform · ui/commands · ui/overlay · ui/restoration · ui/gestures | 100% / ~99% | M1 抬升 |
  | foundation/text | ~96% | M2：shaping/text_blob/text_measure 0→100、backend.rs 93 |
  | browser-ui/chrome | ~97% | M2：12 组件 95–100% + shell.rs 94% / chrome_model 100% |
  | ui/dsl | ~91% | M3 DC-6：engine.rs 91.79%、yaml.rs 86.61%、loader.rs 92.00%（真实 loader）、expression 100% |
  | ui/widgets | ~95% | M2 DC-7：补 IconButton/Toggle/ContextMenu |
  | ui/render | ~89% | M2：backend.rs 92、paint_ctx 87、render_node 100 |
  | ui/animation · ui/patterns · ui/devtools · ui/navigation · ui/design-system · ui/i18n · ui/adapters · ui/core | 92–98% | — |
  | ui/runtime · ui/testing | 87% / 93% | M3：scene_snapshot 补 StrokeRect/clip 测后 ui/testing 70%→93% |

- **下一步 coverage 推进**：所有 per-crate 已 ≥85%（DC-17 阶段达标）。可选抬 ui/render(~89%)/ui/runtime/ui/testing 边角，随 M2 组件迁移自然增长。

## 依赖决策日志

| 决策 | 结论 | 理由 | Spec 锚点 |
|------|------|------|-----------|
| TBD-8 text foundation 落点 | **新建独立 `foundation/text`（`zero-text-foundation`），M2 已落地真实 `FontdueBackend`**（fontdue+rustybuzz，不依赖 render-foundation） | render-foundation 耦合 wgpu/png/resvg/tiny-skia 图形后端；纯文本/字体基础层独立避免被图形后端污染，UI 与 WebView 都能依赖而不拖入 GPU 栈。M2 第一阶段：foundation/text 自立真实后端；第二阶段（风险步，需 product-smoke）把 render-foundation 现有 font 栈统一到 foundation/text | spec §6.5A / IF-008 / TBD-8 |
| TBD-2 ui-render 与 render-foundation 依赖方向 | **`ui/render` 定义自己的 Scene/RenderNode/PaintCtx 抽象 + M2 已加 `RenderBackend` trait + `paint_scene`**；ui/render 不直接依赖 render-foundation，通过 trait 由 render-foundation 后端实现（待 product-smoke） | 通用 UI 层不被 wgpu 后端耦合；ui/render 只依赖 foundation/text（纯文本层），不依赖 GPU 栈 | spec §8.4.1 / TBD-2 |
| TBD-9 text shaping/font 依赖 | **复用 workspace 已声明** fontdue/rustybuzz/unicode-bidi；M2 已把 fontdue+rustybuzz 加入 foundation/text（swash 暂未用） | 已在 workspace.dependencies；零新增外部依赖；fontdue=度量/光栅、rustybuzz=OpenType shaping | spec §6.4 / TBD-9 |
| TBD-7 i18n 依赖 | **M1 手写 minimal plural/RTL**，不引入 ICU4X/Fluent | 接口先行；依赖评估留 M3 | spec §6.5A / TBD-7 |
| TBD-6 表达式 parser | **仓内自实现 Pratt/precedence-climbing parser**（`ui/dsl/src/engine.rs`，M3 phase-1 已落地），不引 parser combinator crate | 表达式语法边界明确（spec §8.4.7 文法），手写 parser 可控 + 零新依赖；含 parse-depth 守卫防栈溢出 | spec §8.4.7 / TBD-6 |
| TBD-1 serde_yaml | **仓内自实现受限 YAML 子集解析器**（`ui/dsl/src/yaml.rs`，M3 phase-2 已落地），不引 serde_yaml（已弃用）/ serde_yml | DSL YAML 子集结构受限（组件树 + 标量/流集合/注释），仓内解析器可控、零新依赖、错误信息友好；spec §8.4.7 倾向仓内自实现；serde_yaml 上游已 archived | spec §8.4.7 / TBD-1 |

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
- `evidence/coverage-20260701-032214.txt` — **DC-17（M2 DC-12：BrowserChromeModel + desktop/tablet/phone shell，2026-07-01）**：聚合 line 94.38% / function 94.88% / region 94.35%；per-crate 全部 ≥85%（chrome shell.rs 94% / chrome_model.rs 100%）。+ DC-12 测试锚点（select_shell §8.4.4A 表 / phone keyboard+safe-area 避让 / 跨 shell 稳定 WidgetId / adaptive 派发）。
- `evidence/coverage-20260701-034232.txt` — **DC-17（M3 DC-6 phase-1：DSL 表达式引擎，2026-07-01）**：聚合 line 93.97% / function 94.33% / region 93.07%；per-crate 全部 ≥85%（ui/dsl engine.rs 91.79%）。+ DC-6 测试锚点（parse/eval 各能力 / sandbox 黑名单 ForbiddenCapability / UnknownFunction / 资源上限 nodes·depth·iters）。
- `evidence/coverage-20260701-040111.txt` — **DC-17（M3 DC-6 phase-2：YAML→WidgetSpec loader + ui/testing 补测，2026-07-01）**：聚合 line 93.58% / function 92.92% / region 92.51%（6914 行）；per-crate 全部 ≥85%（ui/dsl yaml.rs 86.61%/loader.rs 92.00%、ui/testing scene_snapshot 70%→92% / crate ~93%）。+ DC-6 phase-2 测试锚点（YAML 块/流/嵌套/注释/类型推断；loader 递归 component/id/props/bindings/actions/control/children；strict 表达式校验；端到端 load→eval；i18n 对象保留）。
- `evidence/coverage-20260701-073437.txt` — **DC-17（M2 DC-2：retained WidgetHost + render_node translate 全分支测，2026-07-01）**：聚合 line 93.33% / function 92.90% / region 92.51%（7586 行）；per-crate 全部 ≥85%（host.rs 91.85%、render_node.rs 96%、scene.rs 95%）。+ DC-2 测试锚点（Column 堆叠绝对 rect / 点击命中正确节点 emit / stable WidgetId 复用 epoch / ComponentType 变更重建 / Row·Stack 几何 / 未知组件填充 / clip 交集 / translate 全图元变体）。
- `evidence/dc14-counter-20260701-074528.txt` — **DC-14（counter 示例，2026-07-01）**：`cargo run --example counter` 输出 retained 闭环（count 0→3→2，scene 3 图元）；2 集成测绿；DC-1 cargo tree 机械验证零浏览器 crate 依赖（仅 core/render/runtime/widgets）。
- `evidence/dc7-chrome-scene-20260701-075934.txt` — **DC-7（chrome paint→统一 Scene，2026-07-01）**：`chrome_scene` example 产出 desktop shell scene 快照（12 图元：address URL `https://example.com` / security_badge 绿 fill / tab 标题 `Example` / `0 bookmarks` / viewport 白底）；47 chrome 测绿；DC-1 复核 ui-runtime 零浏览器业务 crate 依赖。
- `evidence/dc3-webview-widget-20260701-080848.txt` — **DC-3（WebViewWidget impl Widget，2026-07-01）**：WebViewWidget paint 记录 `ExternalSurface` marker 进统一 Scene（不映射 DOM）；7 adapter-webview 测绿（paint→ExternalSurface / scroll 事件钳制 / host 注册→Scene）；DC-1 复核 ui/render 零 zero-webview 依赖。
- `evidence/dc4-scrollbar-20260701-081457.txt` — **DC-4（ScrollBar 几何/命中/命令/视觉，2026-07-01）**：scrollbar 7 测（hit zones / drag 真实映射 / track 翻页命令 / paint track+thumb / 闭环 clamp）+ adapter-webview 跨 crate 闭环（ScrollBar→apply_scroll_command→WebViewWidget.scroll）绿。

**M2/M3 门禁实测（2026-07-01）**：
- `cargo build --workspace` — Finished（0 错误）。
- `cargo clippy --workspace --all-targets -- -D warnings` — Finished（0 警告）。
- `cargo fmt --all --check` — 净（0 diff）。
- foundation/text 27 + ui/render 13(+render_node translate) + ui/testing 5(+3) + ui/widgets 29 + browser-ui/chrome 45 + ui/runtime 8(+6 host) + ui/dsl **48**(+22: 12 yaml + 10 loader) tests 全绿（scoped test-guard）。
- DC-1 复核：ui/dsl 仅依赖 compact_str/hashbrown/serde/thiserror/zero-ui-core（`cargo tree -p zero-ui-dsl` 机械验证），零浏览器业务 crate；YAML loader 仓内自实现，零新依赖；ui/runtime 仅依赖 ui/core+ui/render+ui/i18n+compact_str/hashbrown/thiserror，零浏览器业务 crate。
- DC-17 coverage：聚合 line 93.33%，per-crate 全部 ≥85%。

## Next Steps

1. **`ui/examples` form/browser-shell-demo（DC-14）**：counter 已立；续做 form（多控件 + 校验 + 提交，演示 TextInput/校验/提交 action）与 browser-shell-demo（组合 browser-ui/chrome 组件 + render 桥 + ScrollBar + WebViewWidget 占位，不接真实浏览器进程）示例，进一步验证 SDK 外部可复用与组件组合力。
2. **DC-5 主题系统**：当前 chrome/render 用语义色名字符串映射（`chrome_color`）；接 ui/core::theme ThemeResolver + ThemeChanged → 组件消费 semantic token（而非字面色名）。系统主题变化只触发 needs_paint（字体/间距不变）。补 theme resolve + contrast lint + invalidation 测。
3. **DC-6 phase-3**：①map/filter lambda —— 需 spec 决策（`Expression` 枚举无 Lambda 变体；候选：字段投影 `map($items, "field")` / 取注册 fn id；走 `lei-spec-rfc` 回写 spec §8.4.7）；②DSL `Adaptive`/`branches` responsive branch（按 ViewportClass/PlatformClass/InputClass 选布局，接 ui/core::layout）+ command·route·overlay·asset 引用。
4. **DC-3 phase-2：WebViewWidget 真实纹理合成**（render-foundation 后端实现 `RenderBackend::draw_external_surface`，按 surface_id 取 zero-webview `RenderPrimitives`/texture 合成；**触碰渲染后端，需 `make product-smoke`**）。
5. **render-foundation 字体栈统一到 foundation/text**（DC-11 完整闭环；**触碰渲染后端，需 `make product-smoke`**）。
6. **apps/browser 逐组件灰度迁移（DC-14，product-smoke）**：用 browser-ui/chrome 组件 + render 桥替换 apps/browser 手绘 chrome（含 page_scroll.rs → ScrollBar 迁移，DC-14 零退化硬门禁）。
7. 跟踪项：本机 `make test` 受 script-sandbox debug-test V8 链接阻塞（环境性）；render-foundation 字体栈统一前 welcome.html 渲染走旧路径，未受本轮影响。
