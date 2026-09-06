# 键盘页面滚动 — 运行时控制面板（master.md）

**入口文档**: [../keyboard-page-scrolling.md](../keyboard-page-scrolling.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（M2 切片 1 完成——Ctrl+Home/End 修饰变体回执链；焦点→容器链跨域 defer 记录）

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
| P1 | 用例覆盖为零（上游键盘滚动用例稀缺——本地 reftest 补足策略） | 🔶 上游 snap/input 三案勘察 defer（依赖 testdriver Actions 键盘链——keyboard-default-actions M1 切片 2 共享基建）；本地单测 1 案落地（evidence/2026-09-07-m1-keyboard-scroll-baseline.md）|
| P2 | 键盘滚动分发层（键位→滚动量）缺失 | ✅ 底座 R3254-M9 既有（keydown 回执驱动）+ 本地单测断言固化（Space/PageUp/PageDown/方向键/修饰变体/None 键位）|
| P3 | 滚动目标判定 + 嵌套传播缺失 | 🔶 M2 切片 1（2026-09-07，fad120776）：Ctrl+Home/End 修饰变体回执链接通；焦点→容器→根链依赖 renderer S3 layout 几何（跨域 defer——R3298 S2 注记协调点），非本流可闭环 |
| P4 | scrollIntoView 选项面 / scroll 事件联动未核实 | ⬜ M3 |

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
| M3 — scrollIntoView + 事件 + snap 交互收尾 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿；clippy 零警告
- 键盘滚用例面：本地单测 1 案（分发层映射）+ 上游 snap/input 三案 defer 记录（evidence/2026-09-07-m1-keyboard-scroll-baseline.md）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
