# Pointer Lock 兼容 — requestPointerLock / 指针捕获语义（门控 goal）

**版本**: v1.1
**日期**: 2026-10-18
**状态**: **M1 完成（DC-1 ✅）·BLOCK 于 M2 门控**——捕获管线 RFC 须用户点名
批准后启动；M1 甄别报告见
[evidence/2026-10-18-m1-baseline.md](pointerlock-compat/evidence/2026-10-18-m1-baseline.md)
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

## 当前状态（2026-10-18 M1 收口）

- **M1 完成**：corpus 导入 27/27 测试资产（pin 31597693）+ census + 可执行
  子集甄别 + runner 通道（`make testharness-pointerlock`）+ 基线落账 +
  csv planned 行回填（active，1/4 = 25.0%）。
- **关键盘点发现**：API 面**已存在**——R2939（2026-08-07，Fullscreen 同轮）
  在 js_dom_shim 落地 spec-alike 锁态（requestPointerLock Promise grant/deny、
  exitPointerLock void、pointerLockElement、change/error 事件 + on* handler、
  host `__zw_pointer_lock_enabled` deny 开关）。缺口五项（transient activation
  门 / deny TypeError≠NotAllowedError / 位移流 / unadjustedMovement / 全屏
  联动+移除解锁+retarget）已列入甄别报告 M2 RFC 范围输入。
- **卡点（BLOCK）**：DC-2 捕获管线 RFC 须**用户点名批准**——按任务约定不代替
  用户决策。点名后从 RFC（参考 lei-spec-rfc + dnd M3 管线 RFC 先例
  `docs/goal/dnd-compat/m3-drag-input-pipeline-spec-rfc.md`，可合并征询减
  重复评审）起步。
- **跨域 residual**：`Event.NONE` 等 Event 接口 phase 常量缺失
  （constructor.html s1 红）归 dom/events 域（uievents 50 residual），不占
  本 goal 分母口径。

## Mission

M1 完成 corpus census 与可执行子集甄别（API 面存在性/异常语义可静态执行的
部分）；捕获管线（相对位移流/unadjustedMovement/全屏联动语义）出 RFC 后按
用户点名推进。排除：mouse lock 之外的指针 API（编号 50 域）、游戏手柄（远期池）。

---

## Done Criteria

- **DC-1（自主）✅ 2026-10-18** 导入 + census + 可执行子集甄别报告 + csv
  planned 行回填——见 [M1 基线](pointerlock-compat/evidence/2026-10-18-m1-baseline.md)
- **DC-2（门控，须点名）** 捕获管线 RFC 征询通过后实现收口
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归

## 里程碑

M1 盘点（自主，**✅ 完成**）→ M2 管线 RFC（**用户门控，BLOCK 中**）→
M3 实现与收口（门控）。

## 依赖约束（run-rules §9）

host-runtime 光标/输入面与 keyboard 系遗产（已归档）同域；与 28-dnd-compat
的输入注入管线 RFC 合并征询可减重复评审。
