# M3 片 1 — Pointer Capture 状态机 + Actions 逐步重放（2026-10-03）

> **记账说明**：本测量对应 commit `19d5bb1c4` 的落地内容（该提交由并行 rally
> 会话在 22:26 收编本会话在途编辑落地，其记账的 375P 为收编前中途测量；本文
> 434P 为该提交内容完成态的全量复测）。碰头经过见文末。

**基线**：M2 片 1 后 343P（1024 subtests 口径）。**本轮后**：434P / 630F / 66TO / 1NR
（1131 subtests——前期早停 case 复跑出更多 subtests，唯一文件数不变 235）。
pointerevents 198P→383P（23.6%→40.7%）；uievents/click 0→9P、mouse 10→17P、
order-of-events 3→6P。跑法 `ZW_CORPUS_CASE_TIMEOUT_SECS=10 make testharness-uievents`
（capture 子集聚证：69P→130P）。

## 落地面

1. **pending capture override 状态机**（part04/part06——spec §9.3
   https://www.w3.org/TR/pointerevents2/）：`setPointerCapture` 只登记
   `_zwPtrState.pending`（NotFoundError/InvalidStateError 校验保留）；
   `hasPointerCapture` 读 pending（即刻 true/false 面）。
2. **Process Pending Pointer Capture**（part06 `_zwProcessPendingCapture`）——
   pointer 系事件派发前结算：pending≠现 override 时派 lostpointercapture@现
   override + 悬停跨界序恢复，再跨界序 + gotpointercapture@pending + 换防；
   `processingCapture` 重入 guard（跨界序自身派 pointer 系事件）。
3. **隐式释放**（pointerup/pointercancel 尾）——chorded buttons（buttons 归零判定）
   不释放；释放序 = lostpointercapture@捕获目标 → out/leave@捕获目标 →
   over/enter@真实悬停位（`hoverSel`/`overSel` 分离——捕获期逻辑位≠真实位）。
   pointerup listener 内的 setPointerCapture 登记 up 尾清除、无 got
   （pointerevent_setpointercapture_pointerup_mouse 断言面）。
4. **捕获重定向**（保留 M3 WIP 面）+ pointermove 前 process-pending（release 后
   事件回原命中元素——pointerevent_releasepointercapture_events_to_original_target）。
5. **touch 隐式捕获**：pointerType==='touch' 的 pointerdown 目标即 pending 捕获
   （spec direct manipulation 隐式捕获；pointerevent_element_haspointercapture
   ?touch expected_default_capture 面）。
6. **got/lostpointercapture 事件面**：bubbles/composed **true**、cancelable false、
   pressure 随键位、pointerType 随源（pointerevent_support.js assert_props helper
   断言面——此前 bubbles/composed false 全簇 fail）。
7. **Actions 逐步重放**（runner testdriver stub）——pointerDown/pointerUp 拆为
   `pointer_down`/`pointer_up` 独立宿主命令（取代 M2 折叠 click：页内 pointerdown
   listener 的 setPointerCapture 须影响后续 move/up 路由）；click/auxclick 组合
   （捕获落点优先、跨目标最近公共祖先、同目标连击 dblclick、非主键 auxclick）移入
   shim `__zw_pointer_down_sequence`/`__zw_pointer_up_sequence`；move-only 链不再补
   尾随 click（真实 Actions 语义——折叠期尾随 click 致悬停迁移重放、捕获族「无多余
   事件」断言面记入多余 pointermove）。pointerType/pointer_move buttons/pressure
   随键位透传（拖拽中 move.buttons 非零——same_element_twice 的 buttons 过滤面）。
8. **伴生根因修复**（`_makeEvent`，part03）：产物挂 `Event.prototype` 原型链——
   泛型派发事件（click 等）此前缺 AT_TARGET 常量与 instanceof Event 面
   （click_during_capture 的 `event.eventPhase == event.AT_TARGET` 过滤全簇 miss
   根因）。

## 验证

- 全量通道：434P/630F/66TO/1NR（本文件头部）；capture 子集 69P→130P（+61）。
- 门禁：`make test` 19,610P/0F、`cargo clippy --workspace --all-targets -- -D warnings`
  零告警、`cargo fmt --all -- --check` 无 diff。
- 已知残余：pointerrawupdate 族（.https 专用事件面）、click_during_parent_capture
  （跨 iframe 捕获）、chorded buttons 相位、pointercapture_in_frame（iframe）、
  pointercancel 触发链——M3 片 2 队列。

## 完整逐案结果

- 全量：/tmp/uievents-full2.log（rally 轮临时产物，计数已录本文）
- capture 子集前后对照：69P/93F/18TO → 130P/38F/11TO

## 碰头记录（run-rules §9/§10）

本会话（uievents-compat 推进轮）与用户侧常设 rally 会话**同树并发**：本会话完成
M3 capture 编辑并进入全量测量/make test 门禁期间，rally 会话于 22:26 将在途编辑
收编为 `19d5bb1c4`（bundle 其 part09.rs R3068 单测更新）+ `733c8fc47`（master.md
记账，375P 为其收编前中途测量），随后继续在 part05.js（coalesced/predicted stub）
与 testharness.rs（element-origin 中心 + offset 语义）推进「M3 尾簇」——两文件
存在其未提交在途编辑。本会话按 §9 暂停 engine/webview 域写入：只落本文档、
不在 master.md 与尾簇文件上并发写。测量差异归因：434P（本文，含 click/auxclick
组合目标捕获优先 + auxclick 公共祖先两处尾修）vs 375P（收编前中途树）。
