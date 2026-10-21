# M3 事件目标化侦察——SVG 元素事件面现状（2026-10-21）

看护轮产出（M2 S1 源码面归基线原 session；本方只读侦察 + 账目）。范围：
goal 契约 M3「SVGGeometryElement 几何接口 + 事件目标化」的事件半边。

## 核心发现：通用派发路径对 SVG 元素已可用

v2 基线全 corpus（动画面外）**事件派发失败≈0**（唯一 'target' 命中是
getStartPositionOfChar 文本几何 API 缺失，S4 面）。架构原因：SVG 元素与 HTML
元素走同一 proxy/handle 事件管线（part05 四个 dispatchEvent 入口
window/document/元素 proxy/EventTarget.prototype；`_zwDispatchGuard` 统一
守卫 R106/R312），ns 无关。svg:a 派发 click 激活已由 navigation-compat M2-S6
通用激活路径收口（tagName 大写化命中 A 分支，2026-10-10 账本）。

**M3 无需为「事件目标化」新建派发管线**；残面如下。

## M3 事件残面清单（v2 实测定位）

| 残面 | 信号量 | 依赖 | 归属评估 |
|---|---|---|---|
| **形状命中测试**（elementFromPoint 返回 SVG 形状） | interact 108F（rect-hittest-002 53F / ellipse-hittest 24F / svg-pointer-events-bbox 23F / rect-001 4F / text-hittest 3 案 / small-big-path） | 形状几何——rect/ellipse 属性级 bbox 可解析近似；精确面涉 pointer-events fill 规则与 transform 合成 | 解析近似可做；bbox-vs-fill 语义面（svg-pointer-events-bbox）须与渲染边界核对（同 getBBox 家族） |
| 焦点/Tab 遍历 | focus-inside-hidden-svg-containers 3 案 + text/a Tab 面 | testdriver send_keys（runner 自备面按既有口径记账）+ SVG tabindex/focusable 语义 | 挂账候选 |
| SVGAnimationElement 事件接收 | animations/custom-events 6F | S1 表构造器 + SMIL 事件基（beginEvent/endEvent/repeatEvent） | S1 表收构造器；事件基触发归 SMIL 挂账 |
| image/script load/error 事件 | image-load-error-events T / script-load-error-events T | 资源加载事件管线 | Timeout 面待归因 |

## 结论

M3 主战场是 **SVGGeometryElement 几何**（getTotalLength 104F + getBBox 171F，
纯计算面优先）+ 命中测试解析近似；事件目标化本体已被通用路径覆盖，只需
interact 面复核 + 挂账记账。
