# DC-2 · strike css-false 新裹标签 `<s>`→`<strike>`（2026-11-12）

六 format 命令元素臂 strike 项 Chrome delta 清账：strike 文件 css-false
innerHTML 失败 50 例全簇根因是新裹标签形——上游 vendored 原文 `s`，
WPT 语料期望以 Chrome 录形为准（`foo[bar]baz` css-false 期望
`foo<strike>[bar]</strike>baz`，data/strikethrough.js:73 实证）。
单行共享路径修复（`_zwVaForceValue` 落尾臂 createElement('s')→
createElement('strike')），UA 读面 s/strike 双标同名映射 line-through
（part01d :239、dissolve/allow 列表双标签）故 state 读/dissolve 面零
影响。

## 方法

同 usbase/usafter2 口径（test-guard/systemd scope 包裹，
`ZW_SCRIPT_TIMEOUT_MS=300000`），base = usafter2（P3 批入账树，
bb1c1256e 等价），after = dc2us3-runner（本刀）。面 1 `dc2us-face-run.sh`
（六 format 文件 + 8 探针）、面 2 `dc2us-mt-run.sh`（multitest + misc
哨兵）。

## 数字

面 1：usafter2 11,748/253 → us3 **11,785/216**：

| 文件 | base P/F | after P/F | ΔP |
|---|---|---|---:|
| strikethrough | 2067/80 | 2104/43 | +37 |
| bold/italic/sub/sup | 恒等 | 恒等 | 0 |
| underline | 2115/32 | 2115/32 | 0 |
| 探针 8（tstyle 1F、mtmini 1F、p2diag 5F 既有诊断） | 恒等 | 恒等 | 0 |

- 断言级：**F2P=37 / P2F=0 / newF=0**，37 翻转逐名恒等
  （removed-Fail 名单 = added-Pass 名单，diff 空）。
- 37 全为 css-false 新裹簇：裸文本/跨 styled-child/表格格/跨段落/
  局部分解再裹（`<s>foo[b<span>…]</span>z</s>` 中段 `<strike>ba</strike>`
  形）——原挂账 50 例簇中 37 例仅标签形一差即翻，余 13 例叠卷位/嵌套
  差仍红（见残余）。

面 2（multitest + misc 哨兵）：usafter2 10,310/24/1T → us3mt
**10,485/24/1T**：misc 401/22 恒等、multitest F=2/T=1 恒等，断言级
**丢失=0 / P2F=0 / newF=0**（fail 集逐名 diff 空），+175P 全为新增入账
（?9001-last 慢页门杀进度差 + multitest strike 序列随本刀翻绿，共享
断言零漂移）。

## 残余与下一步

- strike 43F / underline 32F 长尾两簇：① css-true UA 面载体折叠
  （`foo<u>[bar]</u>baz`+strike 期望 `<span style="text-decoration:underline
  line-through">` 单壳——u 元素值 push-down 入新裹后自身溶解；P3 span
  载体门不含 u/s/del/ins 元素载体）；② 多值 decoration 偏序删除
  （`foo<span style="text-decoration: underline line-through">b[a]r</span>`
  +underline：外层留存活 token、被摘段内层补 assert 被摘 token 的
  Chrome 雕刻形）；③ wrap 嵌套位（css-false 新裹应落保留载体壳内
  `<u><strike>[bar]</strike></u>`，我方外裹）。
- bold/italic 值面作者臂未接（语料零 `<style>` 消费面，挂后续）。
