---
date: 2026-10-23
modules: layout-engine, engine
---

# flatten 单 run 的 run_ws 表达不了混合 white-space——R5081「文本自身条目优先」不可推广到 inline flatten

## 问题描述

伪元素注入文本节点（inject_pseudo_text_nodes 把伪元素样式挂在文本节点条目上）经 inline 宿主 flatten 时，run white-space 读 owner 元素继承而非伪元素声明（content-174：span::before 的 nowrap 丢失，窄容器逐词折行）。按 R5081「run ws 读文本自身条目」的思路推广到 flatten 路径（取宿主直接文本子中首个带条目者的 white-space），content-174 有改善，但 content-175 从 ✓0.21% 回归到 ✗1.11%——corpus A/B 定谳净负，改动全撤。

## 根因

layout-engine 的 inline flatten 把宿主全部 inline 内容摊平成**一个 run、一个 run_ws**，而 white-space 是逐文本片段的属性：content-175 里宿主原有文本应按 normal 折叠、::after 注入的 `  \A` 应按 pre 保留——单 run_ws 无论取 owner（现状，对宿主文本正确、对伪元素文本错）还是取注入条目（对伪元素文本正确、对宿主文本错）都是近似。R5081 的「文本自身条目优先」成立的前提是**块容器直子每段文本独立成 run**，作用域天然一致；inline flatten 共享 run 的结构不满足该前提，推广即串味。

## 解决方案

放弃 ws 通道，改走盒化：伪元素带装饰（背景/边框）+ inline-static + 真文本 → zw-pseudo element 盒化（engine pipeline needs_box 第四臂，kill-switch ZW_PSEUDO_INLINE_DECORATED_BOX）。盒化后伪元素是真实 inline 元素，white-space/color/background 全走既有元素路径，run 归属天然分离（R5238，content-174 ✗1.72→✓0.00）。corpus A/B 是唯一可靠的取舍裁判：非盒化残余（1.72→1.40）看着像净改善，但同一次 A/B 暴露出另一案的 pass→fail，总量不动掩盖方向错误。

## 避免方式

改动「共享 run 属性从哪个节点读」之前，先问该属性在规范里是逐片段还是逐盒语义；逐片段属性（white-space/方向类）在共享 run 结构里换读取源都会伤另一半。凡做 R5081 式语义推广，必须带 16,590 案全量 A/B 再定去留，单案改善不足以收账。
