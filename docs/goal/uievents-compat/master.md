# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-04（M3 尾簇 2 落地）

## 当前状态

**M3 + 尾簇 1/2 落地（2026-10-04）**：基线 230P → M2 片 1 343P → M3 375P →
尾簇 1 451P → **457P（+227 累计）**（run6：457P/607F/103TO）。
证据：[evidence/2026-10-03-m2-slice1.json](evidence/2026-10-03-m2-slice1.json)（M2 片 1）、
[evidence/2026-10-03-m2m3-after.json](evidence/2026-10-03-m2m3-after.json)（M3 后）、
[evidence/2026-10-03-m3-tail.json](evidence/2026-10-03-m3-tail.json)（尾簇 1 后）。
门禁：make test 19,610P/0F、clippy -D warnings、fmt 全绿。M3 尾簇 1（7561ba05a）：
element-origin 偏移语义 + getCoalescedEvents/getPredictedEvents stub。
**M3 尾簇 2（2026-10-04，f3564b5c2）**：目标移除重定向——悬停/事件目标被页内
listener 移除后，下一指针事件经命中测试重定向（host elementFromPoint + gBCR 最小
包含盒近似），已移除元素不接收 out/leave、只补 over@新目标；touch（非 hoverable）
抬起悬停拆除（leave 锚 = 入队时祖先链快照首个连通近祖）+ touch move 步不派/down
隐含迁移；runner enqueue 携带 move 时点祖先链快照（up/down 的 enqueue 在前一命令
resolve 后、彼时元素可能已移除），`__zw_td_selector` 断连回退首个连通近祖。

**M3 内容**：Pointer Capture 三方法语义化（spec §9.3——非 active pointerId
NotFoundError、disconnected InvalidStateError、got/lostpointercapture 派发〔lost
异步〕、up/cancel 隐式释放）；捕获重定向（capture 期 pointer/mouse 系事件全量
重定向到捕获目标 + `__zw_pointer_move` 跨界抑制）；active pointer 状态机
（down 前置位——listener 内 setPointerCapture 可用）；Actions down/up 步逐步重放
（页内 pointerdown listener 的 setPointerCapture 影响后续 move/up 路由）；
`_makeEvent` 产物挂 Event.prototype（eventPhase/instanceof 面）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | uievents + pointerevents corpus 导入 + 基线 | ✅ 2026-10-03 |
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | 🔄 核心已落地；残余 = mousemove-between（视口命中精度）、wheel 三案、focus-events 四案、interface keyboard-click、uievents/mouse 尾簇（layerX/chorded buttons/image-map） |
| P3 | Pointer 生命周期 + capture 三方法 + enter/leave 边界序 | 🔄 核心已落地（含 coalesced/predicted stub + 目标移除重定向）；残余 = pointerup-remover variant（子测试间 re-append 后布局快照陈旧——gBCR 零 rect 致命中测试回退近祖）、after_target_appended/from_slot/interleaved 族、pointercancel/touch-action 交互面 |
| P4 | touch-events / pointerlock / IME 组合挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1（2026-10-03）**：corpus 导入 + runner 通道 + 分类基线 1024 subtests 230P +
  suites CSV 回填（[evidence/2026-10-03-m1-baseline.md](evidence/2026-10-03-m1-baseline.md)）。
- **M2 片 1（2026-10-03，5d5669118）**：Actions stub 重写为上游多源 API 面 +
  PointerEvent 分支 + 边界事件序 + click 组合序（dblclick/公共祖先/auxclick/
  contextmenu）+ viewport 命中测试。+113。
- **M3（2026-10-03，19d5bb1c4）**：Pointer Capture 语义化 + 捕获重定向 + active
  pointer 状态机 + Actions 逐步重放 + Event.prototype 面。+32。
- **M3 尾簇（2026-10-03，7561ba05a）**：element-origin 偏移语义（中心+offset 命中）
  + coalesced/predicted stub。+76。
- **M3 尾簇 2（2026-10-04，f3564b5c2）**：目标移除重定向（命中测试重定向 +
  dangling 边界面 + touch 悬停拆除 + 祖先链快照回退）。+6（after_target_removed
  pointerdown-remover 3 variant 转绿；uievents/mouse 17→19P、order-of-events
  6→7P）。

## 下一步计划

1. **M2/M3 尾簇**（按 Throughput 排序）：a) mutation 族残余——pointerup-remover
   variant（**根因已定位（2026-10-04 摸底轮）**：runner 页内 DOM 变更全落 shim 内部
   状态——不产 DomMutation/native 写——宿主管线布局停留在导航时点，re-append 元素
   gBCR 恒零盒 → send() 时点命中测试回退近祖。已验证 shim outerHTML 驱动的结构性
   刷新（render_html 全量 + persistent_handle_nodes 重绑）机制可行但**序列化滞后一
   turn** 且计划构建时点解析救不回——正解 = **命令执行时点重解析**（pointer 命令
   dequeue 时以 fresh 几何做 origin 中心+offset 命中，配合同款结构性刷新，一个提交
   内成套落地）；试做净差零已回退）；after_target_appended/from_slot/interleaved
   族；b) wheel 源 scroll 命令（wheel 三案）；c) mousedown→focus 默认动作链
   （focus-events 四案）；d) uievents/mouse 尾簇（layerX/layerY、chorded buttons 位、
   image-map 命中）。
2. **M4 收口**：touch-action 解析/计算值核对（parsing 三案 0P 待查）、
   touch-events / pointerlock / IME / touch-action 交互面挂账定稿、DC-4 全绿门禁
   （make test + clippy + fmt + reftest 零回归 + product-smoke）。

**待用户决策清单**：（空——pointerlock 挂账重入 = 用户点名）

