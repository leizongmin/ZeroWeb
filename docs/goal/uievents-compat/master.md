# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-09-12（立项）

## 当前状态

鼠标事件面已有（UIEvent 47/MouseEvent 52 处）、pointer 语义薄（PointerEvent 6 处）。
合成事件（dispatchEvent）语义先行，宿主输入协议只消费不改。keyboard 两 goal
（已归档）事件派发机制先例复用。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | uievents + pointerevents corpus 导入 + 基线 | ⏳ M1 纯资产 |
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | ⏳ M2 |
| P3 | Pointer 生命周期 + capture 三方法 + enter/leave 边界序 | ⏳ M3 |
| P4 | touch-events / pointerlock / IME 组合挂账定稿 | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：corpus fetch + 导入 + 基线（goals/50 编号脚本可跑 fetch 步）+ suites CSV 回填

**待用户决策清单**：（空——启动顺序由用户点名）
