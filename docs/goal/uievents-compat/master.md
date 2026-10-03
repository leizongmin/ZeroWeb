# UI/指针事件兼容 — 运行时控制面板（master.md）

**入口文档**: [../uievents-compat.md](../uievents-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-03（M1 收口）

## 当前状态

**M1 收口（2026-10-03）**：corpus 全量导入（uievents 60 + pointerevents 200 案）、
runner 通道 `testharness-uievents`（递归扫描 + 跳过规则 + variant 展开）、分类基线
1024 subtests（230P / 675F / 73TO / 46NR——uievents 17.4%、pointerevents 23.6%）。
证据：[evidence/2026-10-03-m1-baseline.md](evidence/2026-10-03-m1-baseline.md)。
suites CSV 已回填（active 行，1024/230）。

**M2 主根因**（基线聚类）：runner testdriver Actions stub 指针链退化
（pointerDown/Up no-op、send() 折叠单 click）+ 宿主派发面缺 detail/button/
relatedTarget/dblclick/PointerEvent 类型 + 边界事件（over/out/enter/leave）零合成。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | uievents + pointerevents corpus 导入 + 基线 | ✅ 2026-10-03 |
| P2 | 鼠标事件序/坐标/click 组合语义修齐 | ⏳ M2（主根因 = Actions 指针链退化） |
| P3 | Pointer 生命周期 + capture 三方法 + enter/leave 边界序 | ⏳ M3 |
| P4 | touch-events / pointerlock / IME 组合挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1（2026-10-03）**：50 号 fetch 脚本幂等续拉 + order-of-events 二级子目录追加；
  runner `testharness-uievents` 子命令 + `uievents_case_skipped` 跳过规则
  （manual ×8 / pointerlock ×9 / crashtests / resources / legacy-domevents——evidence
  清单在册）；Makefile `fetch-wpt-uievents`/`testharness-uievents`；分类基线 +
  suites CSV 回填。

## 下一步计划

1. **M2 切片 1**：Actions 指针链步骤化（runner 注入 stub 记录 move/down/up → 宿主
   命令 `pointer_move`/`pointer_down`/`pointer_up`，keyboard keydown/keyup 命令先例
   同构）+ `__zw_dispatch_event` mouse 分支补 detail/button/buttons/relatedTarget
   + dblclick 组合序——驱动面 click-order/mouse_buttons_back_forward/mouseover-out。
2. **M2 切片 2**：边界事件合成（mouseover/out + mouseenter/leave 非冒泡祖先链 +
   relatedTarget）——驱动面 mouse 边界族。
3. **M3**：PointerEvent 类型分支 + pointer 边界序 + capture 三方法
   （got/lostpointercapture、pending override、隐式释放、NotFoundError）。

**待用户决策清单**：（空——touch-action 交互面与 textInput 族在 M2 尾评估记账，
pointerlock 挂账重入 = 用户点名）
