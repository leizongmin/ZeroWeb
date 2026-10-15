---
date: 2026-10-15
modules: crates/layout-engine,crates/engine
---

# 内联 SVG（替换元素）在 shrink-to-fit/auto 尺寸链里 used width 被算成 0——google 首页 logo 不绘制的根因方向

## 问题描述

true innerText 修复（PR #178）后，ZeroWeb 打开 `https://www.google.com` 首页能完整
渲染文字、顶栏、搜索表单，但**中央 Google logo 不绘制**。对照：Chrome 同条件该区域
彩色像素约 6.8k，ZeroWeb 三次独立截图该区域彩色像素全为 0。

logo 不是网络图片，而是页面里**内联 `<svg class="ESTs9d" width="272" height="92"
viewBox="0 0 272 92">` + 带 `fill="#3186FF"` 等的彩色 `<path>`**。DOM/布局读数：

- `.ESTs9d`：计算样式 `display:inline; width:auto; height:92px; max-width:100%`，
  `getBoundingClientRect() = [640,258,0,92]` → **宽 0，高 92 正确**。
- 父级 `.naW5gc`（flex 项的 block）同样 `width:auto`、rect 宽 0；祖先 flex 容器
  `[0,0,1280,~]` 满宽且在视口内。

即 ZeroWeb 在 google 的 flex/auto + `max-width:100%` 尺寸链里，把内联 SVG 替换元素
的 used width 算成 **0**，仅高度按属性尺寸算对，于是整块不绘制。

## 根因分析（方向已定位，最小复现未落地）

关键对立证据把"能力缺失"和"布局缺陷"分开：

- **SVG fill 绘制能力正常**：同一内联 SVG（`width="272"` + 彩色 `fill` path）放进
  本地 fixture 的普通块流 / flex 容器 / flex + `max-width:100%` 三种形态，ZeroWeb 都
  按 272px 正确绘制（像素分析 blue 25000 / red 20094 / green 878）。→ 不是 SVG 光栅化
  或 fill 不支持。
- 因此 collapse 是**特定布局上下文**问题：google 页面上 `.ESTs9d` 的 `width` 计算为
  `auto`（未把 `width` 属性映射成 used width），配合父级 shrink-to-fit / flex 项尺寸
  回环，ZeroWeb 把 used width 解析成 0。实际触发协作的精确 CSS 声明仍未取到。

**排查受阻点（可复用方法学）**：

1. google 首页 logo 的精确 CSS 由 JS 注入且体量巨大（`<style>` 合计数百 KB）。ZeroWeb
   V8 对"拼接全部 style 的 textContent 后求值"报 `IPC 通道已关闭` / `Runtime.evaluate
   timeout`——不能靠一次大求值抓规则；改"逐 `<style>`、只扫命中 class 的规则块"仍会
   timeout。
2. 同一渲染下，页面里 `Array.from(document.images).map(...)` 迭代会把 google 的侵入式
   属性 getter 触发成 `TypeError: Cannot convert a Symbol value to a string`；因此在
   实站探针里**不要迭代 `document.images` 全集**，改逐个容错点名读取目标元素。
3. CDP `DOM.getDocument`/`CSS.getMatchedStylesForNode` 在当前 headless build 不稳定
   （`No node with given id: 1`），不能依赖 CSS 域取规则。

这些使"取出 `.ESTs9d` 精确 CSS 制成最小 WPT 复现"这一步受阻，卡住修复落地的回归
测试前置（AGENTS「每修复必带测试」），故按任务无进展规则先暂停该簇，未伪造修复。

## 解决方案 / 下一步

该缺陷**尚未修复**，当前是"诊断 + 根因方向"。要把它做成可合并 PR，必须先：

1. 拿到 `.ESTs9d` 及祖先容器触发 `width:auto→0` 的精确 CSS 声明（拟用手动把 google
   首页内联 `<style>` 落到本地文件、用静态 fixture 逐个声明做 A/B，绕过 V8 超大求值）。
2. 固化成最小 WPT 用例（flex 容器 + block 包装 + 带 `width` 属性的内联 SVG + 精确
   `max-width:auto` 声明组合），确定性复现 used width = 0。
3. 定位 ZeroWeb 布局引擎里替换元素 intrinsic/autom 尺寸的解析回环节点后修复，并以该
   WPT 为回归、对照 Chrome 验收。

## 为什么记

这是「渲染修复不一定 = SVG 能力缺」的典型反例：现象是 logo 空白，根因在布局层
replaced-element 尺寸解析，而非光栅化。同类的"先排除能力缺、再定位上下文特定布局"
对照法（本地同形状 fixture + 像素量化 vs Chrome）可复用到后续 real-site 渲染缺陷。