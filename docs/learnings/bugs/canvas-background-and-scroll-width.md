# Canvas background and false horizontal overflow

日期：2026-08-09

相关模块：`zero-engine` paint、`zero-browser` page scroll

## 问题描述

长页面滚动后会露出宿主默认白色背景。页面包含 `overflow-x: scroll` 的宽内容时，浏览器还会错误显示整页横向滚动条。

## 根因分析

+ 根元素或 `body` 传播到 canvas 的背景只按 viewport 高度生成。宿主滚动页面图元后，首屏背景矩形也随内容移出视口。
+ paint 阶段会把 overflow 区域外的字形标记为 `glyph_id = 0`、`font_size = 0`，但文档宽度统计仍使用这些字形的原始 `x` 坐标，已裁掉的内容因此扩大整页宽度。
+ 图片为保持 crop 语义会保留未裁剪的 `rect`，并用 `clip` 记录实际可见范围。文档宽度统计若忽略 `clip`，图片异步加载后的最终帧会突然扩大整页宽度。
+ 浏览器绘制的是 overlay 滚动条，却仍从页面 viewport 扣除滚动条宽度。仅有垂直溢出时，原本正好等于内容区宽度的页面会被误判为横向溢出。
+ 多进程 renderer 的异步加载器把外链 CSS 缓存在 `WebView` 内。脚本修改 DOM 后若调用 `load_html(..., None)` 重绘，会清空这份 CSS，使 `pre` 等元素退回 UA 样式并产生页面级横向溢出。

## 解决方案

+ canvas 背景按布局树可见 overflow 范围与 viewport 的较大值生成，确保长页面滚动后背景连续。
+ 文档宽度统计忽略已失效的字形。
+ 图片使用 `rect` 与 `clip` 的可见交集计算文档宽度，完全裁掉的图片不参与统计。
+ overlay 滚动条不缩减页面 viewport；双滚动条只在轨道交汇处预留 corner。
+ 脚本 DOM 变更后的 renderer 重绘使用 `WebView::reload_html_after_script()`，保留异步加载阶段已缓存的外链 CSS。

## 验证

+ 长文档 canvas 背景高度覆盖完整文档。
+ 裁剪到不可见的远端字形不扩大文档宽度。
+ 原始矩形超宽但被 clip 限制的图片不扩大文档宽度。
+ 文档宽度等于 viewport 且仅纵向溢出时，不显示横向滚动条。
+ 脚本修改 DOM 后重绘仍保留缓存 CSS。
+ morning.work 静态产品页在 1024px 下通过结构检查。
