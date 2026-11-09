---
date: 2026-11-09
modules: engine
---

# shim appendChild 隐式 detach 缺口状态相关——孤立探针同码全绿不等于实现正确，执行态分歧用窗口克隆+side-band trace 定位

## 问题描述

F6b N 族四键（multitest :2178/:2183/:2398/:2403，delete→inserttext 过幸存 script 壳）孤立探针 zz-probe-f6b 17/0 全绿，mt 语料却残红，got 形=期望形+`<sub>a</sub>` 幽灵重复尾（如 N1 期望 `foo<font size="1"><sub>a</sub></font>baz`，实得同形后再串一个 `<sub>a</sub>`）。立项时按「引擎产出形与语料期望形分歧」归入 handoff（语料形对齐刀）——方向错了。

## 根因分析

两层：

1. **变异点定位**（side-band trace，页面侧 wrapper 钩 `__zw_ce_inserttext_mutate`/`__zw_ce_f6b_post_insert`，POST-in/POST-out 树差出页）：全链单次 MUT、carry/tags 正确、POST-in 树已是正确重映射前中间态，重复尾在 `__zw_ce_f6b_post_insert` ② 异桶重放内部产生——唯一嫌疑即 `fontEl.appendChild(scriptEl)`。
2. **缺口机理**：shim appendChild 在该状态下不做隐式 detach（旧亲链不脱），元素双亲（新 font + 原 host）被 innerHTML 序列化两次——与 zz-probe-d4x X4 / F1'-B 前向并入先例同根（「DocumentFragment 无移动语义；元素侧同法保守」）。**缺口状态相关**：孤立页同码 appendChild 正常 move，mt 顺序执行面（typed 文本为 caret 锚节点态、数百案先行）不脱链。所以「孤立页过」只能证明「该态下没问题」，不能证明「实现无条件正确」。

## 解决方案

- **修复**：循 X4 先例——`appendChild` 前显式 `if (node.parentNode) node.parentNode.removeChild(node);`（本刀 part01 `__zw_ce_f6b_post_insert` 重放段）。显式 detach+append 在两态下语义恒等（正常 move 时 removeChild 是无感前移），不依赖未定义的隐式行为。凡是「先 insertBefore 到目标侧、再 appendChild 收编」的移动写法都要过这道检查。
- **方法论**：孤立探针全绿而语料红的分歧，先假设**执行态**差异而非实现/语料分歧——① multitest 全克隆窗口探针（同 harness 装配+subset 窗语义，variant 收窄）确认可复现；② 页面侧 wrapper side-band trace 在变异点前后各 dump 一次树态，把「产出形分歧」收敛到「哪一次 DOM 写产生重复」。在变异点粒度取证之前，不要把残红读作「期望序列化需要另做对齐」。
