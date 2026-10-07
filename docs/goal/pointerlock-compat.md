# Pointer Lock 兼容 — requestPointerLock / 指针捕获语义（门控 goal）

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active·**门控**（M1 盘点自主；指针捕获管线须用户点名批准——99-webgl
门控模式）
**执行模式**: WPT 驱动（上游 pointerlock corpus 为验收标尺）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——交互面）

> **▶ 拆分动机（2026-10-08 html5test 缺口立项，用户「立项」）**：uievents goal
> （编号 50，已收口）挂账面之一；canvas 游戏/3D 场景的基础 API。corpus 体量小，
> M1 盘点成本极低，先立项占位。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：element.requestPointerLock/
> document.exitPointerLock/pointerlockchange/pointerlockerror 面存在性未盘点；
> 指针捕获本质依赖 host 光标管线（隐藏系统光标 + 相对位移流）——host-runtime
> 面，缺即门控。

## Mission

M1 完成 corpus census 与可执行子集甄别（API 面存在性/异常语义可静态执行的
部分）；捕获管线（相对位移流/unadjustedMovement/全屏联动语义）出 RFC 后按
用户点名推进。排除：mouse lock 之外的指针 API（编号 50 域）、游戏手柄（远期池）。

---

## Done Criteria

- **DC-1（自主）** 导入 + census + 可执行子集甄别报告 + csv planned 行回填
- **DC-2（门控，须点名）** 捕获管线 RFC 征询通过后实现收口
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归

## 里程碑

M1 盘点（自主）→ M2 管线 RFC（**用户门控**）→ M3 实现与收口（门控）。

## 依赖约束（run-rules §9）

host-runtime 光标/输入面与 keyboard 系遗产（已归档）同域；与 28-dnd-compat
的输入注入管线 RFC 合并征询可减重复评审。
