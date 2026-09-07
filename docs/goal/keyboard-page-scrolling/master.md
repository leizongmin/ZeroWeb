# 键盘页面滚动 — 运行时控制面板（master.md）

**入口文档**: [../keyboard-page-scrolling.md](../keyboard-page-scrolling.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（P1 收口——snap/input 三案经 keyboard 套件可执行 + Timeout 根因定位（runner 无真帧循环 scrollend 语义），与 keyboard-default-actions M1 切片 3 同轮）

---

## 当前状态

**专项定位**：键盘/编辑方向三拆之三。页面级键盘滚动（PageUp/Space/Home/End/方向键 +
修饰变体）+ scrollIntoView + scroll 事件语义，WPT 真实用例 + 本地 reftest 驱动。滚动
管线底座新鲜（p1a element scroll + b12f9b67 钳位）。

**与兄弟 goal 的边界**：
- keyboard-default-actions — 控件默认动作归其管；本目标管滚动键（分发顺序：编辑宿主 →
  控件默认动作 → 滚动默认动作）
- editing-contenteditable — 编辑宿主内按键归其管
- rendering-compat — 滚动条 UI/overscroll 深化归其流域；scroll-snap **布局**已有，本目标
  只接键盘触发面
- js-dom — engine element scroll 段（p1a 产物）共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ 滚动管线：page_scroll.rs（滚轮路径）+ 根滚动 `min(layout, painted)` 钳位
  （b12f9b67，2026-08-16 CI 修复轮固化）+ 滚动后合成帧 e2e 断言
- ✅ 元素滚动：p1a element.scrollTop/scrollLeft + overflow 容器（2026-08-12 设计）
- ✅ scroll-snap 解析 + 渲染指示器（Tier 1 表 ✅）
- ⚠️ 键盘滚动默认动作（键位/滚动量/修饰变体）无系统分发
- ⚠️ 滚动目标判定（焦点→容器→根 + 嵌套传播）缺失
- ⚠️ scrollIntoView 选项面 / scroll 事件与键盘滚动联动待摸底
- ⚠️ 用例覆盖为零（上游 + 本地）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 用例覆盖为零（上游键盘滚动用例稀缺——本地 reftest 补足策略） | ✅ defer 解除（2026-09-07，keyboard-default-actions M1 切片 2/3 同轮）——snap/input 三案（keyboard.html/paged.html/scroll-padding-paged.html）已导入 keyboard 套件并可执行：paged 1P/1F/1Timeout + keyboard 1 Timeout + scroll-padding 1 Timeout；**Timeout 根因定位**：waitForScrollEndFallback 依赖 scroll 事件 + rAF 帧循环静默窗（runner 无真帧循环——`__ZW_RAF_FRAME_DRIVEN` OFF 路径 rAF 同步 stub，scroll 未发生时 promise 永挂 → testharness completion 不触发）；paged 1F = snap 容器 Space 键滚动目标判定（P3 跨域项同一根因）。本地单测 1 案 + 窗口滚动七断言 + scrollIntoView 五断言落地（evidence/2026-09-07-m1-keyboard-scroll-baseline.md）|
| P2 | 键盘滚动分发层（键位→滚动量）缺失 | ✅ 底座 R3254-M9 既有（keydown 回执驱动）+ 本地单测断言固化（Space/PageUp/PageDown/方向键/修饰变体/None 键位）|
| P3 | 滚动目标判定 + 嵌套传播缺失 | 🔶 M2 切片 1（2026-09-07，fad120776）：Ctrl+Home/End 修饰变体回执链接通；焦点→容器→根链依赖 renderer S3 layout 几何（跨域 defer——R3298 S2 注记协调点），非本流可闭环 |
| P4 | scrollIntoView 选项面 / scroll 事件联动未核实 | ✅ M3（2026-09-07）：R3060 选项面摸底完成（block start/end/center 公式 + smooth 简化已记录）；scroll 事件语义断言资产两件（窗口七断言 + scrollIntoView 五断言，标明本地）|

## 下一步计划

1. ~~**M1 切片 1**：上游可执行用例导入~~ ✅ 2026-09-07（defer 解除——Actions 键盘链 + send_keys 滚动键事件对落地，css-scroll-snap/input 三案导入 keyboard 套件；全部可执行、断言 F 聚类记录）
2. ~~**M1 切片 2**：键盘滚动分发层骨架~~ ✅ 2026-09-07（底座 R3254-M9 既有 + scroll_delta_for_key 映射单测固化——Space/PageUp/PageDown/Arrow/None 键位语义断言）
3. ~~**M1 切片 3**：失败聚类 → 修复队列~~ → 并入 M2。
4. ~~**M2 切片 1**：Ctrl+Home/End 修饰变体~~ ✅ 2026-09-07（commit fad120776；dispatch_ctrl_scroll_key 回执链 + e2e；browser 413 全绿）。M2 剩余：焦点→容器链（S3 跨域 defer，master.md 记录）——M2 全键位中本流可闭环部分已完成

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/` 核对 js-dom 流
element scroll 段活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 基线建立 + 分发层骨架 | ✅ 切片 1/2 完成（2026-09-07）——上游三案导入（6F=真滚动管线缺口）+ 分发映射单测；M2 滚动目标判定为主项 |
| M2 — 全键位 + 滚动目标判定 | 🔶 切片 1 ✅（Ctrl 变体，2026-09-07）；焦点→容器链 S3 跨域 defer（待渲染流域协调，master.md 记录）|
| M3 — scrollIntoView + 事件 + snap 交互收尾 | ✅ 断言资产落地（2026-09-07）；snap 键盘交互断言需 runner 真渲染 viewport（跨域协调项，M1 记录延续）|

## 验证基线

- 测试基线：立项时点全绿；clippy 零警告
- 键盘滚用例面：本地单测 1 案（分发层映射）+ 上游 snap/input 三案 defer 记录（evidence/2026-09-07-m1-keyboard-scroll-baseline.md）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

## Done Criteria 清点（2026-09-07 收尾状态）

| DC | 条目 | 状态 |
|---|---|---|
| DC-1 | 上游可执行用例导入 | ✅ snap/input 三案（keyboard 套件，全可执行；断言 F=真滚动管线缺口记录）|
| DC-1 | 本地 reftest/单测补足（标明本地） | ✅ 分发映射单测 + 窗口滚动七断言 + scrollIntoView 五断言 |
| DC-1 | 分类通过率报告 + evidence 持久化 | ✅ evidence/2026-09-07-m1-keyboard-scroll-baseline.md（含三轮追加）|
| DC-2 | 滚动键全键位（含修饰变体） | ✅ Space/±Shift/PageUp/PageDown/Arrows/Home/End/Ctrl+Home/End（回执链 preventDefault 阻断）|
| DC-2 | 焦点→容器→根判定 + 嵌套传播 | 🔶 跨域 defer——renderer S3 layout 几何（R3298 协调点），文档级/根级已闭合 |
| DC-3 | scrollIntoView 选项面 | ✅ R3060 block start/end/center 公式断言（smooth→instant 简化记录）；scrollIntoViewIfNeeded R3075 |
| DC-3 | 键盘滚动后 scroll 事件 + scrollX/Y 一致 | ✅ 窗口滚动七断言（事件 cancelable=false + 轴独立 + round-trip）|
| DC-4 | snap 容器键盘交互 | 🔶 断言需 runner 真渲染 viewport（跨域协调项；snap 布局在渲染器已有）|
| DC-5 | cargo test 全绿 / clippy 零警告 / 资产化 | ✅ 18,966 全绿（本轮）/ 零警告 / 本地资产标明 |

**收尾结论**：本流可闭环面全部落地；剩余两项（焦点→容器链、snap 断言）均为跨域
协调项（渲染流域），已记录于对应协调点注记——不阻塞本 goal 的流域收口判定。
