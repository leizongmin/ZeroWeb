# 键盘页面滚动 — 运行时控制面板（master.md）

**入口文档**: [../keyboard-page-scrolling.md](../keyboard-page-scrolling.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-17（立项——M1 待启动）

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
| P1 | 用例覆盖为零（上游键盘滚动用例稀缺——本地 reftest 补足策略） | ⬜ M1 |
| P2 | 键盘滚动分发层（键位→滚动量）缺失 | ⬜ M1 |
| P3 | 滚动目标判定 + 嵌套传播缺失 | ⬜ M2 |
| P4 | scrollIntoView 选项面 / scroll 事件联动未核实 | ⬜ M3 |

## 下一步计划

1. **M1 切片 1**：上游可执行用例导入 + 本地 reftest 骨架（滚动量断言——标明本地）
2. **M1 切片 2**：键盘滚动分发层骨架（根滚动先行）
3. **M1 切片 3**：失败聚类 → 修复队列

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/` 核对 js-dom 流
element scroll 段活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 基线建立 + 分发层骨架 | ⬜ 待启动 |
| M2 — 全键位 + 滚动目标判定 | ⬜ |
| M3 — scrollIntoView + 事件 + snap 交互收尾 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿；clippy 零警告
- 键盘滚用例面：无基线（未导入/未建）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
