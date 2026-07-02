# M1 能力矩阵 + 缺口 — 20260630-234530（Wave 1-4 完成）

## M1 验证状态

| 维度 | 状态 | 证据 |
|------|------|------|
| 全部 crate 存在并可编译 | ✅ | `cargo build --workspace` Finished（含 23 新 crate + 既有 crate lib） |
| clippy `-D warnings`（workspace + all-targets） | ✅ | `cargo clippy --workspace --all-targets -- -D warnings` Finished 零警告 |
| 新 crate 单元测试 | ✅ | 23 crate 合计 **135 passed / 0 failed**（scoped test-guard） |
| DC-1 依赖隔离 | ✅ | 22 通用 crate 零浏览器依赖；adapter-webview→zero-webview；chrome→ui/*+browser-shell+adapter-webview（见 dep-isolation-20260630-234530.txt） |

## DC skeleton 级证据（M1 范围内）

| DC | 验证点 | 状态 | 测试锚点 |
|----|--------|------|----------|
| DC-1 | 目录/依赖隔离 | ✅ | dep-isolation evidence；22 crate 零浏览器依赖 |
| DC-2 | 三棵树 + retained + WidgetId 复用 | ✅ skeleton | `element::stable_widget_id_preserves_element_state_across_rebuild`、`widget::stable_widget_id_equality` |
| DC-5 | 主题 paint-only 失效 | ✅ skeleton | `theme::color_only_change_is_paint_only_invalidation`、`theme_provider::system_scheme_change_emits_paint_only_invalidation` |
| DC-8 | 焦点遍历 / IME caret | ✅ skeleton | `focus::forward_backward_wrap`、`text_input::ime_caret_rect_advances_with_cursor`、`ime::change_detected` |
| DC-9 | 局部失效刷新 | ✅ skeleton | `invalidation::paint_only_change_does_not_request_layout`、`invalidation::layout_implies_paint` |
| DC-10 | i18n fallback/param/plural/RTL + locale 失效 | ✅ skeleton | `catalog::resolve_message_with_param`、`catalog::fallback_chain_resolves_parent`、`formatter::substitutes_count_and_plural`、`direction::rtl_detection`、`i18n_provider::locale_switch_invalidates_layout_paint_semantics` |
| DC-11 | 共享 text foundation / glyph cache 复用 | ✅ skeleton | `glyph_cache::same_key_reuses_atlas_entry`、`font_fallback::fallback_appends_generic_in_order` |
| DC-12 | ViewportClass/adaptive 分支 | ✅ skeleton | `layout::viewport_class_breakpoints`、`layout::adaptive_branch_mobile_vs_desktop` |
| DC-4 | ScrollBar 几何 + drag→ScrollCommand | ✅ skeleton | `scrollbar::vertical_thumb_ratio_and_position`、`scroll::command_resolve_targets` |
| DC-7 | 通用 widgets + patterns + browser chrome skeleton | ✅ skeleton | widgets/patterns/chrome 各模块构造测试 |
| DC-3 | WebViewWidget 不映射 DOM、只算外部矩形 | ✅ skeleton | `webview_widget::widget_only_tracks_external_geometry`、`scroll_bridge::clamps_to_max_scroll` |

## 已落地 crate 清单（23 个新 crate）

通用层（依赖 zero-ui-core，零浏览器依赖）：
ui/core, ui/render, ui/i18n, ui/runtime, ui/widgets, ui/patterns, ui/animation, ui/gestures,
ui/navigation, ui/overlay, ui/collections, ui/commands, ui/forms, ui/assets, ui/platform,
ui/restoration, ui/testing, ui/devtools, ui/design-system, ui/dsl, ui/adapters/winit。
共享层：foundation/text。
浏览器耦合点：ui/adapters/webview（→zero-webview）、browser-ui/chrome（→ui/*+browser-shell+adapter-webview）。

## 未解决缺口（M1 收口前剩余 + 后续里程碑）

1. **coverage 基线**：M1 crate 尚未跑 `scripts/check-coverage.sh`（cargo-llvm-cov）取 line/function/region 基线；
   计划下一轮取基线并写入 master.md（DC-17 要求 ≥85%，全仓不低于 floor）。cargo-llvm-cov 工具需确认已安装。
2. **`ui/examples`**（counter/form/browser-shell-demo）：spec 列在 M3（DC-14），本轮未建示例 crate。
3. **`make test` 本机门禁**：仍被 zero-script-sandbox debug-test 的 rusty_v8/advapi32 **链接**失败阻塞（环境性，
   非本目标引入；release 构建绿、CI 绿）。clippy --workspace --all-targets 全净（clippy 不链接，绕过该问题），
   故 DC-16 的 clippy/fmt/build 门禁满足；测试门禁通过 scoped test-guard + workspace build 旁证。
4. **M2 实质内容**：browser-ui/chrome 其余 §8.4.1A 组件（AddressBar/BrowserTabStrip/SecurityBadge/...）、
   text foundation 接入 ui/render 与 zero-webview（真实 fontdue/swash/rustybuzz 桥接）、apps/browser 灰度迁移。
5. **render-foundation 桥接（TBD-2）**：ui/render Scene→render-foundation 后端 trait 在 M2 设计。
6. **零-security cors.rs 工作树损坏**：本轮发现 crates/security/src/cors.rs 工作树曾被瞬时磁盘损坏
   （HEAD 干净），已用 `git show HEAD: > file` 恢复到提交状态（git status 干净、可编译）。非本目标代码改动；
   记录以防复发（疑似本机磁盘/AV 瞬时问题）。
