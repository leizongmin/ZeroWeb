# M1 — 键盘滚动分发层基线（2026-09-07）

**基线资产构成**（goal DC-1 允许形态：上游稀缺 → 本地单测标明「本地」补足）：

1. **上游勘察**（2026-09-07，WPT_REV 315976933870）：`css/css-scroll-snap/input/`
   有 keyboard.html / paged.html / scroll-padding-paged.html 三案——但均依赖
   testdriver Actions **键盘链**（addKey/send，`meta name=flags content=should`）
   + 真渲染 snap 布局断言。runner 的 Actions stub 仅 pointer（R142），键盘链是
   keyboard-default-actions goal M1 切片 2 的修复队列（跨 goal 共享基建，见其
   evidence）；**上游三案待键盘链落地后导入**（defer 有据，非静默）。
2. **本地单测**（标明本地）：`scroll_delta_for_key_maps_keyboard_scroll_keys_r3254_kp1`
   （apps/browser/src/tests.rs）——分发层键位→滚动量映射断言：
   - Space/Shift+Space = ±0.85 视口高；PageDown/PageUp = ±0.85 视口
   - ArrowDown/ArrowUp = ±40 CSS px × page_render_scale
   - Home/End = None（立即 to_top/to_bottom 路径，非回执 delta）
   - 非滚动键（a/Enter）= None
   - 底座：R3254-M9 keydown 回执驱动（PendingTabAction::ScrollViewport，页面
     preventDefault 可阻止；focused_text_input 守卫）。
3. **测试访问器**：`scroll_delta_for_key_for_test` / `page_render_scale_for_test`
   （app.rs，#[cfg(test)]，既有 for_test 模式）。

**已知限制**（记录）：滚动目标判定（焦点→可滚动容器→根的冒泡链）未实现——当前
delta 一律作用于根视口（缺口 P3，M2）；Ctrl+Home/End 修饰变体未实现。

**执行入口**：`cargo test -p zero-browser scroll_delta_for_key_maps`（仓库
make test 全量含之）。
