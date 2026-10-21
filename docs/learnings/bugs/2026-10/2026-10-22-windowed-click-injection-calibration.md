---
date: 2026-10-22
modules: browser,renderer
---

# 窗口模式注入点击两坑：chrome 高 91px 坐标换算与空白区点击静默丢弃

## 问题

compositor-animation DC-3 验收（COMP-03）注入点击（`ZW_INJECT_CLICKS`）三击全部
"发生但无效"：browser 日志确认三次注入、暂停/取消/恢复链路全部工作，但页面点击
回执格恒为空白。初版落点按「chrome 高 ~40px」假设从 fixture CSS 坐标反推窗口坐标。

## 根因

两个独立因素叠加，均无报错静默失败：

1. **chrome 高度实测 91px，非 40px**。`TAB_BAR_TOP_INSET 6 + TAB_BAR_HEIGHT 34 +
   ADDRESS_BAR_HEIGHT 50 = 90`（`apps/browser/src/layout.rs` `TOOLBAR_HEIGHT`），
   实测 `doc = win − 91`（scale=1）。落点整体在文档空间偏高 ~50px，全部落入元素
   间空档。
2. **空白区点击静默不派发（预存在缺口）**。browser `dispatch_page_mousedown`
   命中 None → `DispatchDomEvent.selector=None`；renderer `dispatch_dom_at` 对
   None selector 走 `hit_test_element(x,y)` 兜底，仍 None 时直接返回默认结果——
   mousedown/mouseup/click 全部不到达页面 DOM，无 html/body 兜底（真实浏览器语义
   应派发至 html）。

定位路径（其他观测手段全部失灵时的有效组合）：ui_driver 对照实验（`--ui-driver-script`
JSONL `click` op 驱动同一 fixture 基线静态点击，排除注入器自身）确认链路级断裂；
再以临时 debug 插桩三点（browser `dispatch_dom_event_async` sent 行、renderer
`handle_dispatch_dom_event` recv 行、`dispatch_dom_at` 解析行）+ `ZW_LOG_LEVEL=debug
ZW_RENDERER_STDERR_ECHO=1` 拿到 renderer 实收坐标，反推 chrome 高。注意 renderer
stderr 必须开 echo 才可见，`ZW_LOG_LEVEL` 经进程 env 继承对 renderer 生效。

## 解决方案

- 注入/断言类 fixture 的窗口坐标一律按 `TOOLBAR_HEIGHT(+1)` 换算并在脚本注释标明
  实测依据；落点**必须瞄准元素**（空白区点击不派发是语义缺口不是坐标误差）。
- 观察时间线以 frame_dump 采集窗（512 帧 ≈ 8.2s@62fps）为硬上界反推：事件时刻、
  注入时刻、判定时刻全部落入窗内；注入锚（tab loaded）较页面 load 提前 0~2.5s
  （Δ）的摆动用「恒在状态窗内」「旋转绕中心任意相位命中」类几何设计吸收，不依赖
  固定延时。
- 分析脚本对时间窗做结构化识别（如 bbox 恒定段判定冻结期），不硬编码时间线常量。

完整验收证据：`docs/goal/compositor-animation-continuity/evidence/2026-10-22-m2s3d-dc3-acceptance/README.md`。
