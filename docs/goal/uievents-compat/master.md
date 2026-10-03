# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-03（M3 落地）

## 当前状态

**M3 + 尾簇落地（2026-10-03）**：基线 230P → M2 片 1 343P → M3 375P →
**451P（+221 累计）**（run5：451P/611F/103TO/1NR）。
证据：[evidence/2026-10-03-m2-slice1.json](evidence/2026-10-03-m2-slice1.json)（M2 片 1）、
[evidence/2026-10-03-m2m3-after.json](evidence/2026-10-03-m2m3-after.json)（M3 后）、
[evidence/2026-10-03-m3-tail.json](evidence/2026-10-03-m3-tail.json)（M3 尾簇后全量）。
门禁：make test 19,610P/0F、clippy -D warnings、fmt 全绿。M3 尾簇（7561ba05a）：
element-origin 偏移语义（WebDriver 元素中心+offset 命中——pen/touch variant 面
解锁）+ getCoalescedEvents/getPredictedEvents stub（init 序列透传）。

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
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | 🔄 核心已落地；残余 = mousemove-between（视口命中精度）、wheel 三案、focus-events 四案、interface keyboard-click、uievents/mouse 尾簇（layerX/chorded buttons/image-map/mutation 序） |
| P3 | Pointer 生命周期 + capture 三方法 + enter/leave 边界序 | 🔄 核心已落地（含 coalesced/predicted stub）；残余 = mutation-at-down/up 事件序（after_target_removed 族 19F）、pointercancel/touch-action 交互面 |
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

## 下一步计划

1. **M2/M3 尾簇**（按 Throughput 排序）：a) mutation-at-down/up 事件序（PE spec
   pending pointer capture 处理 + target 移除后的事件路由——after_target_removed/
   appended/interleaved 族 ~40 subtests）；b) wheel 源 scroll 命令（wheel 三案）；
   c) mousedown→focus 默认动作链（focus-events 四案）；d) uievents/mouse 尾簇
   （layerX/layerY、chorded buttons 位、image-map 命中）。
2. **M4 收口**：touch-action 解析/计算值核对（parsing 三案 0P 待查）、
   touch-events / pointerlock / IME / touch-action 交互面挂账定稿、DC-4 全绿门禁
   （make test + clippy + fmt + reftest 零回归 + product-smoke）。

**待用户决策清单**：（空——pointerlock 挂账重入 = 用户点名）

