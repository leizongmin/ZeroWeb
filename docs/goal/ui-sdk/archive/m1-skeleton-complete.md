# Archive — M1 UI SDK 核心骨架完成（2026-07-01 收口）

> M1 = 把 spec §FR-002 列出的全部 crate 立起来（接口边界 + 无窗口单测），让通用 UI SDK 架构在编译期与测试期成立，**不触碰浏览器运行时**。本归档记录过程、决策、证据；当前状态以 `master.md` 为准。

## 交付范围（全部完成）

23 个新通用/共享 crate + 1 个浏览器耦合点 crate，按 spec §8.4.1 落地 skeleton + 接口 + 最小无窗口单测：

- **`ui/core`**（zero-ui-core）：geometry / event / widget(WidgetSpec·WidgetId·Widget trait + Mount/Update/Event/Layout/Paint/Semantics ctx) / element(reconcile，stable WidgetId 保留状态) / action(EventResult·ActionId) / binding(Props·Value·StatePath·Binding) / theme(Color·ThemeId + resolver skeleton) / focus / semantics(SemanticsNode·flags) / invalidation(needs_layout/paint/semantics/composite) / layout(WindowMetrics·ViewportClass·adaptive branch) / scroll(ScrollCommand)。31 tests。
- **`foundation/text`**（zero-text-foundation）：FontProvider / TextShaper / TextMeasurer / GlyphCache / bidi / font_fallback / font_request / grapheme / line_break 接口 + 最小实现（复用 workspace 已声明依赖，零新增）；shaping/text_blob/text_measure 为 M1 stub（M2 真实现）。12 tests。
- **`ui/render`**：Scene / SceneEntry / RenderNode / RenderPrimitive(FillRect·StrokeRect·Text) / Layer / PaintCtx(SceneRecorder) / clip / hit_test。6 单测 + 3 集成测。
- **`ui/runtime`**：UiTree(reconcile + pending invalidation 聚合) / UiApp / ThemeProvider / I18nRuntime / ImeController / AccessibilityTree / Scheduler / PlatformRuntime 抽象（不泄漏 winit 类型）。7 单测 + 3 集成测。
- **`ui/widgets`**：Button / IconButton / TextInput / Toolbar / Menu / ContextMenu / Popup / Popover / ListView / Badge / Tooltip / ScrollBar / ProgressIndicator / Tabs skeleton。24 tests。
- **`ui/patterns`**：SearchField / SuggestionList / CommandPalette / DataList / StatusBubble / TabBar / DialogScaffold。7 tests。
- **`ui/i18n`**：locale / catalog / fallback / formatter / plural / direction(RTL) / message id（手写 minimal，未引入 ICU4X/Fluent）。12 tests。
- **13 应用级能力域**：animation(clock/curve/tween，fake clock) / gestures(TapRecognizer·arena) / navigation(route stack) / overlay(OverlayHost·Toast) / collections(lazy list·selection) / commands(Shortcut·CommandDispatcher) / forms(validator) / assets / platform(clipboard 等 service) / restoration(RestorationStore) / testing(scene/semantics snapshot·FakeClock) / devtools(inspector·timeline) / design-system(Zero default pack skeleton)。每域 1–4 tests。
- **`ui/dsl`**：WidgetSpec schema skeleton + Expression AST（Literal/Path/Unary/Binary/Conditional/Call/Array/Object + node_count 资源上限钩子）；完整表达式 parse/validate/typecheck/eval + sandbox 在 M3。4 tests。
- **`ui/adapters/{winit,webview}`** + **`browser-ui/chrome`**：skeleton + 依赖边界（adapter-webview→zero-webview；chrome→ui/*+browser-shell+adapter-webview）。adapter-winit event_map/runtime；adapter-webview WebViewWidget + scroll_bridge；chrome browser_action/navigation_buttons/page_viewport。每 crate 4–5 tests。

## 推进节奏（4 波 + 本轮收尾）

| 波次 | 范围 | commit |
|------|------|--------|
| Wave 1 | bootstrap 控制面 + ui/core + foundation/text | `9f149353` |
| Wave 2 | render + i18n + runtime + widgets + patterns | `8492b1c0` |
| Wave 3 | 13 能力域 + dsl skeleton | `6e762931` |
| Wave 4 | adapters + browser-ui/chrome + DC-1 dep-isolation evidence | `c4ffef81` |
| 收尾 1 | render/runtime/testing 覆盖率集成测试（补齐中断工作） | `ba45b5f8` |
| 收尾 2 | DC-17 coverage 基线（aggregate line 89.89%） | `14a44bdf` |
| 收尾 3 | per-crate coverage 抬升（aggregate line 89.89%→93.00%） | `75919307` |

## 关键决策（详见 master.md 依赖决策日志）

- TBD-8：text foundation 独立 `foundation/text`，M2 桥接 render-foundation font 实现。
- TBD-2：ui/render 自立 Scene/RenderNode 抽象，M2 trait 桥接 render-foundation 后端。
- TBD-9：复用 workspace 已声明 fontdue/swash/rustybuzz/unicode-bidi，零新增依赖。
- TBD-7：i18n M1 手写 minimal plural/RTL，ICU4X/Fluent 评估留 M3。
- DC-17 口径：`--ignore-filename-regex '[\\/](crates|apps)[\\/]'` 排除依赖污染；`^` 锚定对 profdata 绝对路径不生效，必须匹配目录分量。

## 质量门禁（2026-07-01 收口实测）

- `cargo build --workspace` — 0 错误。
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 警告。
- `cargo fmt --all --check` — 净。
- 新 crate scoped test-guard（test-guard.rs OOM 包裹）— 全绿（core 31 / widgets 24 / i18n·foundation/text 12 / render 6+3 / runtime 7+3 / testing 3+1 / patterns 7 / …）。
- DC-1 依赖隔离机械验证 PASS（`evidence/dep-isolation-20260630-234530.txt`）：22 通用 crate 零浏览器依赖；adapter-webview→zero-webview；chrome→ui/*+browser-shell+adapter-webview。
- DC-17 coverage：聚合 line 93.00% / function 93.12% / region 93.41%（≥85%）；per-crate 除 foundation/text stub 外全部 ≥85%（`evidence/coverage-20260701-022603.txt`）。
- `make test` 本机仍 RED（script-sandbox debug-test V8/advapi32 链接环境失败，非本目标引入；release 绿、CI 绿）。跟踪项，不阻塞。

## Done Criteria skeleton 级证据

- DC-1 目录与依赖隔离：✅（skeleton 全在 + 机械验证）。
- DC-2 三棵树 + 单向数据流 + retained：🟡 skeleton（Widget/Element/Render·Scene 类型 + UiTree reconcile + invalidation 单测；DC-2 完整能力如 paint-only 不触发布局已在 core::invalidation 单测覆盖 skeleton 级）。
- DC-5 主题系统：🟡 skeleton（Color/ThemeId + ThemeResolver；系统主题变化→paint-only invalidation 单测）。
- DC-8 无障碍/焦点/IME：🟡 skeleton（SemanticsNode + focus traversal + ImeController change detection）。
- DC-9 局部失效刷新：🟡 skeleton（needs_layout vs needs_paint 区分单测）。
- DC-10 i18n：🟡 skeleton（IF-007 全：locale/catalog/fallback/plural/RTL/diagnostic）。
- DC-11 text foundation：🟡 skeleton（IF-008 接口在；真实共享接入 M2）。
- DC-13 13 能力域：🟡 skeleton（接口 + 最小单测；浏览器接入 M2–M4）。
- DC-17 coverage：🟡 M1 阶段达标（聚合 93%）。
- DC-3/4/6/7/12/14/15：⬜ 在 M2–M4。

## 延后项（明确不在 M1）

- 浏览器 chrome 组件迁移、`apps/browser` 灰度迁移 → M2。
- text foundation 真实 fontdue/swash/rustybuzz 桥接 + ui/render Scene→render-foundation trait → M2（DC-11、TBD-2）。
- 完整 DSL 表达式语言（parse/validate/typecheck/eval + sandbox negative）→ M3（DC-6）。
- `ui/examples`（counter/form/browser-shell-demo）→ M3（DC-14）。
- 跨平台 runtime adapter + 移动端可运行后端 + design-system 风格包 → M4（DC-15）。
- foundation/text 三 stub（shaping/text_blob/text_measure）真实实现 → M2 后补 coverage。
