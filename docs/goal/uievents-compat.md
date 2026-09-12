# UI/指针事件兼容 — uievents / pointerevents

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（已立项，未启动——启动顺序由用户点名）
**执行模式**: WPT 驱动（上游 uievents + pointerevents corpus 为验收标尺）+ 语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——输入事件语义面）

> **说明**
> 本文档是 ZeroWeb「UI/指针事件兼容」专项目标执行契约。宿主输入链路（host-runtime
> winit）已把真实输入送进页面；本目标收敛 JS 侧事件语义（事件序/坐标/捕获/合成
> 事件派发），使拖拽/手势类交互正确。
>
> **▶ 拆分动机（2026-09-12 用户决策，第二批立项）**：① 鼠标事件面已有
> （UIEvent 47/MouseEvent 52 处）但 pointer 语义弱（PointerEvent 6 处）——现代
> 交互（拖拽/笔输入/多点）以 Pointer Events 为标准底座；② 与 keyboard 两 goal
> （已归档）同属输入语义域，打法已验证。
>
> **▶ 基线事实（2026-09-12 实测，js_dom_shim grep）**：
> - MouseEvent 52 处 / UIEvent 47 处——鼠标事件面存在
> - **PointerEvent 6 处 / setPointerCapture 3 处**——pointer 语义面薄
> - keyboard 域已收口（keyboard-default-actions 18/27 + scrolling 8/8 完成面）——
>     事件派发机制先例可复用
> - **WPT corpora**：`uievents/`（事件序/坐标/click 语义）、`pointerevents/`
>     （pointer 事件/capture/触控点）

---

## Mission

以 **WPT uievents + pointerevents 真实用例为验收标准**，收敛鼠标事件语义
（事件序/坐标/click 组合）与 Pointer Events 语义（pointerdown/move/up 生命周期、
setPointerCapture/hasPointerCapture、pointerId/多触点评估），使手势与拖拽交互
行为与主流浏览器一致。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **宿主链路只消费**：winit 输入→engine 派发链路不改协议；语义修齐在 JS/派发层
3. **合成事件优先**：WPT 多为合成事件（dispatchEvent）——先修派发语义再对齐宿主映射

覆盖范围：MouseEvent/UIEvent 语义修齐 / click/auxclick/dblclick 组合序 /
PointerEvent 生命周期 / setPointerCapture/hasPointerCapture/releasePointerCapture /
pointerenter/leave 边界序 / touch-events 评估记账。

### 排除（明确不在范围内）

- **touch-events 套件** —— 触控硬件路径宿主未接入，评估后挂账
- **pointerlock / 指针锁定** —— 宿主鼠标捕获模式域，挂账（重入条件 = 用户点名）
- **IME 组合事件** —— host-runtime IME 面已存在，组合语义挂账
- **拖放 DnD** —— web-api-batch2 已排除挂账（宿主拖拽管线深依赖）

---

## Support Envelope

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | Pointer/UI/Mouse 事件语义面 | part 系 JS 语义 |
| engine | 事件派发序与捕获机制 | 不改 host-runtime 输入协议 |
| WPT 资产 | uievents + pointerevents 子集导入 | fetch 脚本 + 账本 |

**依赖约束（run-rules §9）**：与 keyboard 两 goal（已归档）——事件派发机制先例
复用、键域不重复；与 host-runtime——输入协议只消费不改；与 workers-compat——
无共享面。

---

## Done Criteria

- [ ] **DC-1**：uievents + pointerevents corpus 可执行子集导入 + 分类基线 +
      suites CSV 回填
- [ ] **DC-2**：鼠标事件语义（事件序/坐标/click 组合）修齐
- [ ] **DC-3**：Pointer Events 生命周期 + capture 三方法 + enter/leave 边界序修齐
- [ ] **DC-4**：`make test` 全绿 + clippy `-D warnings` + fmt + reftest 零回归

## 活跃里程碑

**M1** 导入基线（corpus fetch 用编号脚本 `tests/wpt-runner/scripts/goals/50-uievents-compat.sh`（索引 README；幂等续拉，GitHub API 限流时部分目录跳过属预期））→ **M2** 鼠标事件序 → **M3** Pointer 语义 → **M4** 收口
（touch/pointerlock/IME 挂账定稿）。

## Final Output Protocol

`DONE`（DC-1~4 全满足 + 挂账定稿）/ `CONTINUE: <下一步>`（默认）/ `BLOCK: <原因>`。

## Document Control / Archive Policy

入口文档实质变化才改；控制平面 `docs/goal/uievents-compat/master.md`；
archive/ 只追加；evidence/ 持续追加。
