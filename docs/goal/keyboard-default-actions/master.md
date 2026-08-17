# 键盘默认动作 — 运行时控制面板（master.md）

**入口文档**: [../keyboard-default-actions.md](../keyboard-default-actions.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-17（立项——M1 待启动）

---

## 当前状态

**专项定位**：键盘/编辑方向三拆之一（form-validation 拆分时用户已点名的「其次键盘默认
动作」）。HTML 控件键盘默认动作（Enter 提交/空格激活/Esc/select 导航）补齐，WPT 真实
用例驱动。与 form-validation（提交校验管线）天然衔接。

**与兄弟 goal 的边界**：
- editing-contenteditable — 编辑宿主内的键（键入/删除/换行）归其管；本目标管非编辑宿主
  默认动作。分发顺序：编辑宿主优先消费
- keyboard-page-scrolling — 滚动键归其管
- form-validation — Enter 提交走其 interactive validation 管线（不重建，衔接）
- js-dom — 碰 js_dom_shim 事件段前先 `git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ FocusManager：Tab 导航 + tabindex 排序 + 13 单测
- ✅ html_actions submit 路径 + form-validation requestSubmit 阻断
- ✅ dialog 状态机（R3290：show/showModal/close + open 反射）
- ⚠️ keydown 默认动作分发层（按控件类型 + 按键）无系统性实现
- ⚠️ implicit submission / 激活键（空格 vs Enter）/ select 键盘导航缺失
- ⚠️ WPT 键盘默认动作上游用例覆盖为零

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| K1 | WPT 用例覆盖为零 | ⬜ M1 |
| K2 | 默认动作分发层缺失 | ⬜ M1 |
| K3 | implicit submission（Enter 提交规则）缺失 | ⬜ M2 |
| K4 | 激活键语义（空格/Enter → click 合成 + 两键差异）缺失 | ⬜ M2 |
| K5 | select 键盘导航（展开/移动/type-ahead）缺失 | ⬜ M3 |

## 下一步计划

1. **M1 切片 1**：WPT 键盘交互用例导入 + 基线（零源码改动）
2. **M1 切片 2**：失败聚类 → 修复队列
3. **M1 切片 3**：默认动作分发层骨架

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + 分发层骨架 | ⬜ 待启动 |
| M2 — 表单键与激活 | ⬜ |
| M3 — select 导航 + radio/checkbox + 事件序 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿（13,192+）；clippy 零警告
- WPT 键盘默认动作面：无基线（未导入）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
