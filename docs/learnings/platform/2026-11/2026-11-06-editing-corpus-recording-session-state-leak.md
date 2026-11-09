---
date: 2026-11-06
modules: editing-compat,wpt_runner
---

# editing 语料 before 读面编码录音会话状态泄漏——孤立红行先做泄漏模拟对账再定读面 bug

## 问题描述

editing 语料（insertorderedlist/insertunorderedlist 等）中 `queryCommandValue("defaultparagraphseparator") before` 这类**有状态读面**的期望值不是浏览器默认常量，而是录音会话的顺序泄漏：每案 before 读 = 语料序中前一个 dps exec 案 set 进去的值，首案 before = 录音浏览器初态（"div"）。切片跑批时这条链与切片窗口交互，产生**每窗恰好一处**的孤立红行，形状上极易误判为读面实现缺陷（2026-11-06 dps 轮 iol `<p>[foo]<blockquote>…` qVb 行为例，浪费一轮归因才定谳）。

## 根因分析

录音浏览器不重置 dps 状态：语料序 case 543（set div）的 before="p" → case 548（set p）的 before="div" 逐对吻合，即期望值沿语料序构成一条状态机链。引擎侧 `__zwDefaultParagraphSeparator` 同样跨案持久 + 未设态回退 'div'，与录音语义同构——全量对账（56+84 案）吻合 55/56 + 82/82。唯一失配出现在切片窗界：`?1001-last` 等切片页每窗新开、state 重置回退 'div'，而 expected 沿用全文件会话链值（前案 set p），**窗界后首个有状态读面案必然失配**。

改回退默认迁就窗界首案是反向陷阱：回退改 'p' 后 iol −1/iul +1 净恶化，回退 'div' 是全量最优拟合，不可改。

## 解决方案

1. 有状态读面（qV/qS/qI before/after、dps、stylewithcss 等）出现孤立红行时，先做**泄漏模拟对账**：按语料序跑状态机（每 dps exec 案更新当前值，首案初态 'div'），逐案对照 log 实测 before 值；模型吻合 ≥55/56 即窗界伪影，除名不追，模拟不合才是真读面缺陷。
2. 判读器报 newF/P2F 时先核该案是否恰为切片窗界后首案（`?N-…` 窗口键 + 语料序位置）。
3. 同族参考：跨窗 straddle 伪影（qS/qVa/qSa 跨 `?N-` 边界 split 产生的 post-only Fail）与本文窗界状态重置同源——都是切片窗口 vs 录音全文件会话的状态语义差；单点定谳后按族签归因，不逐轮重查。切片协议本身见 [multitest 语料 full-file 单发模式失败](2026-11-05-multitest-corpus-full-file-mode-fails-use-slices.md)。
