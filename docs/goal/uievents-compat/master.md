# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-03（M2 片 1 落地）

## 当前状态

**M2 片 1 落地（2026-10-03）**：基线 230P → **343P（+113）**——uievents 17.4%→27.8%、
pointerevents 23.6%→33.9%（1024 subtests 口径）。跑法
`ZW_CORPUS_CASE_TIMEOUT_SECS=10 make testharness-uievents`（30s 缺省首跑 30min guard
超时——explicit_timeout 用例等真输入恒烧满额）。门禁：make test 19,603P/0F、
clippy -D warnings、fmt 全绿。

落地内容：① runner testdriver Actions stub 重写为上游 testdriver-actions.js 多源
API 面（addPointer/setPointer/addKeyboard + tick 对齐重放 + options-object 签名）；
② `__zw_dispatch_event` PointerEvent 分支（pointerdown/up/move/over/out/enter/
leave/cancel——`instanceof PointerEvent` 面）+ mouse 分支补 relatedTarget/detail/
button/buttons；③ 边界事件序（`__zw_pointer_move` 跨界 out/leave + over/enter，
enter/leave 非冒泡；out/over 任意命中目标变化都派）；④ click 组合序（连击
dblclick + UIEvent.detail 连击计数；mousedown/mouseup 跨目标 → click 到最近公共
祖先 `__zw_pointer_click_sequence`）；⑤ 非主键序（mousedown→contextmenu（右键）→
mouseup→auxclick，auxclick.detail 连击计数）；⑥ viewport origin 命中测试
（elementFromPoint host 缓存 + gBCR 几何近似 + body 背景传播 fallback）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | uievents + pointerevents corpus 导入 + 基线 | ✅ 2026-10-03 |
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | 🔄 片 1 已落地（+113）；残余 = mousemove-between（视口命中精度） |
| P3 | Pointer 生命周期 + capture 三方法 + enter/leave 边界序 | ⏳ M3 |
| P4 | touch-events / pointerlock / IME 组合挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1（2026-10-03）**：corpus 导入 + runner 通道 + 分类基线 1024 subtests 230P +
  suites CSV 回填（[evidence/2026-10-03-m1-baseline.md](evidence/2026-10-03-m1-baseline.md)）。
- **M2 片 1（2026-10-03）**：见当前状态。伴生修改：PlannedEvent.detail 通道
  （click/dblclick detail）；webview `pointer_over`/连击态（per-document 生命周期，
  导航重置 ×4 站）；shim `__zw_reset_form_state` 挂 `__zw_pointer_reset`。
  已验证无回归：testharness-html 全绿、keyboard 16/16 绿、web-components 4003P
  （稳态）、fullscreen 132P。

## 下一步计划

1. **M2 片 2**：uievents 残余面——mousemove-between（视口命中精度，需 host
   HitTestCache 在 runner 侧填充）、wheel 三案（wheel 源 scroll 命令——wheel 事件 +
   scroll 干预）、focus-events 四案（mousedown→focus 默认动作链）、interface
   keyboard-click（键盘激活无 pointer 序——需 Activate 来源标注）。
2. **M3**：Pointer capture 三方法语义化（got/lostpointercapture 派发、pending
   override、隐式释放、active pointer NotFoundError、捕获重定向 + compat mouse
   抑制）；pointerType pen/touch 全链（PlannedEvent.pointer_type 通道）；
   pointercancel；touch-action 解析/计算值核对。
3. **M4**：挂账定稿（touch-events / pointerlock / IME / touch-action 交互面）。

**待用户决策清单**：（空——pointerlock 挂账重入 = 用户点名）

