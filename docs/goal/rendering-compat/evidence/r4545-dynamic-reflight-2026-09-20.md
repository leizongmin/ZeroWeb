# R4545 证据：资产激活新红二案归因——JS 动态重渲染域（跨流记档，P 轮 0 net code）

- 日期：2026-09-20（R4545 P 轮——R4544 下轮方向②）
- 谱系：R4542 资产激活（green-256x256.png 补齐）暴露的两案新红归因勘察。

## 逐案取证

1. **display-none-inline-img（15.62%）**：页面 JS 于 `window.onload` 内嵌套两层
   `requestAnimationFrame` 两次 toggle img display（终态 inline 应可见）。ZW 渲染
   **全白**——第一次 toggle（none）已生效（初始 CSS 无 hidden，白页 = img 隐），第二层
   rAF 的恢复 toggle 未在截图前生效/重渲染。ref（静态）绿色 300×250 正常。
2. **dynamic-filter-changes-001（13.65%）**：`onload` handler `classList.add('filter')`
   （invert(100%)）+ `takeScreenshotDelayed(0)`。ZW 渲染**原绿**（class 未生效）；ref
   静态 invert = 洋红 ✓（**invert 滤镜本身在 ZW 静态路径工作正常**——排绘制域）。

## 归因（共同域）

两案共同面 = **JS 驱动的 DOM 变更 → 重渲染管线**（reftest-wait.js / rAF 链 /
onload handler / takeScreenshotDelayed 截图时序）：JS 变更后的样式失效与重排没有
可靠地在截图前生效（一案第一次变更生效、第二次丢失；另一案完全未生效）。taffy/绘制/
滤镜静态路径均已被对照排除。

**跨流边界**：该域属 zero-web 流（script-sandbox / engine JS 桥 / reftest harness 的
截图时序），渲染流不单方修（L1139 先例同型）。两案为 R4542 资产激活前「两侧同空假
通过」→ 激活后**诚实暴露的既有 gap**（非回归）；维持 1453 fail 记账，本 goal 的
known-reds 账本新增该域标记。

## 下轮方向

1. corner-shape 余 6 fail 深域勘察（backdrop-filter ×2/iframe/video/inset-shadow/
   render-corner-shape 多 variant）；
2. 守成轮（ZRG 低频巡检：本流 R4539-R4544 六连代码轮后建议守成）；
3. 或跨流移交记档复核（JS 重渲染域两案在 zero-web 流控制面记档的建议——本流仅记录）。
