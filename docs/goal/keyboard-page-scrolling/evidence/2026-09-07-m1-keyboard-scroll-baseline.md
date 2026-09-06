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

---

# M2 切片 1 — Ctrl+Home/End 修饰变体（2026-09-07，同日追加）

**修复**（apps/browser）：Ctrl+Home/End 修饰变体接通 keydown 回执链——
`dispatch_ctrl_scroll_key`（app_input.rs）：keydown 派发到页面（可 preventDefault），
未取消则 `PendingTabAction::ScrollViewport` 滚到文档顶/底（delta 派发时按当前滚动
状态计算）。此前 Ctrl+Home/End 落入无修饰快捷键块立即滚动（无页面 keydown、无
阻断机会）；ctrl 分支 match 此前只处理 Tab/PageUp/PageDown。

**差异语义**：纯 Home/End = 无修饰快捷键立即滚动（无 keydown 派发——历史行为，
Chromium 同款）；Ctrl+变体 = 回执链（页面可消费）。

**e2e**：`ctrl_home_end_scroll_to_top_and_bottom_via_receipt`（tall page →
PageDown 到中间 → Ctrl+Home 回顶（回执链）→ Ctrl+End 到底）。browser 413 全绿。

**M2 剩余与跨域记录**：
- 焦点→可滚动容器→根判定链：依赖 renderer S3 layout 几何暴露（渲染流域协调点，
  R3298 S2 注记已记录）——跨域 defer，非本流可闭环。
- Ctrl+Left/Right（word jump）：文本编辑域（editing goal 范围）defer。
- snap 三案断言复评：需 runner 侧真实布局滚动管线（scroll-snap 布局在渲染器已有，
  runner testharness 无真渲染 viewport——跨域协调，记录于 master.md 待决策项）。
