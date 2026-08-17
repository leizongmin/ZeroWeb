# 编辑与 contenteditable — 运行时控制面板（master.md）

**入口文档**: [../editing-contenteditable.md](../editing-contenteditable.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-17（立项——M1 待启动）

---

## 当前状态

**专项定位**：键盘/编辑方向三拆之二。把 contenteditable 从属性反射（R3187）深化为可用
编辑基础（Selection/Range 可观察 + 键入落 DOM + 编辑事件 + execCommand 基础面），WPT
`selection`/`editing` 真实用例驱动。

**与兄弟 goal 的边界**：
- keyboard-default-actions — 非编辑宿主默认动作归其管；编辑宿主内按键归本目标（分发
  顺序：编辑宿主优先消费）
- keyboard-page-scrolling — 滚动键归其管
- js-dom — zero-dom range.rs 是其 deep-review 过的共享模型（不撞 dom_bindings 面）；
  Selection JS 绑定进 js_dom_shim 前先 `git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ contenteditable 反射：R3187（part01.js:328）枚举状态求值 getter/setter
- ✅ Range 模型：zero-dom range.rs（952 行，R3377 deep-review 确认健壮；insert_node 有
  文本节点字符偏移分裂底座；跨容器分支已知简化记录在案——本目标 Selection 面若逼出
  跨容器需求即到接线时点）
- ✅ 宿主选区底座：page_selection.rs（browser 侧文本选区基础设施）
- ✅ execCommand copy/cut 返 true 语义（part06.js:1432 桩——format 类不真应用）
- ⚠️ 编辑行为为零（键入/删除/换行不落 DOM）
- ⚠️ `window.getSelection()` 可观察面待摸底（M1 首项）
- ⚠️ beforeinput/input 事件缺失
- ⚠️ WPT `editing`/`selection` 未导入，无基线

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| E1 | WPT selection/editing 用例覆盖为零 | ⬜ M1 |
| E2 | Selection JS 可观察面未核实/缺失 | ⬜ M1 |
| E3 | 编辑行为管线（键入/删除/换行 → DOM）缺失 | ⬜ M2 |
| E4 | beforeinput/input 事件缺失 | ⬜ M2 |
| E5 | execCommand format 桩（不真应用） | ⬜ M3 |

## 下一步计划

1. **M1 切片 1**：`selection` 用例导入 + 基线（Selection 是编辑前置——先立可观察面）
2. **M1 切片 2**：getSelection 全 API 摸底/接线（zero-dom 选区模型）
3. **M1 切片 3**：`editing` 用例导入 + 失败聚类

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — selection 基线 + Selection 面摸底 | ⬜ 待启动 |
| M2 — 编辑行为管线 | ⬜ |
| M3 — execCommand 基础面 + 收尾 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿；clippy 零警告
- WPT selection/editing 面：无基线（未导入）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
