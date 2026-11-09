---
date: 2026-11-09
modules: engine
---

# FD walk 吸收臂设计前先逐迭代 trace 执行路径——静态模型把 br-branch 当落点，实际 walk cond1 已先吃掉 br

## 问题描述

mt 626（`<p>abc[]<br> </p> <p><br></p>` forwarddelete→inserttext，期望 `<p>abcd</p>`，实得 `<p>abcd</p> <p><br></p>`）：按静态读码模型判断块尾 br 走 `_zwDelForwardDeleteAction` 的 br-branch（walk 后 `childNodes[offset]` 为 br 的显式支路），据此在该支路内落「ws 吸收 + 空源块吸收」臂。探针 side-band trace（`globalThis.__zwA4Trace`）连跑四变体**全空**——连吸收成功的变体都没进新臂，臂挂在了不执行的域上。

## 根因分析

两层分歧：

1. **执行路径分歧**：FD walk 循环体 cond1（invisible 兄弟子移除）在 text 分支之前就吃掉了块尾 br（`_zwJusIsExtraneousLineBreak` 判 br 冗余 → invisible），walk 继续前进；块内尾 ws 经 cond4 下降进 text 节点后 break，text 分支把 ws **当字符单删**后直接 `return true`。br-branch 只服务行中可见 br 的删除断行场景，对本形不可达。静态模型拿「walk 出口」当「br 仍在原位」推演，漏算了 walk 途中的 DOM 变异。
2. **谓词语义分歧**：ws 文本残留的根因是 spec visible 对折叠空白走文档序**回看**（`_zwJusIsCollapsedWhitespaceNode`，`_zwJusPreviousNode` 链遇非空白文本即判行中）——walk 先移除 br 后，ws 的回看命中块内正文 `abc`，被误判行中可见；而 CSS/Chrome 语义是**前向**行界折叠（ws 后方到块界/br 之间无可见内容即折叠不可见）。

## 解决方案

- **方法论**：对 walk 型 spec 忠实移植代码设计新吸收臂前，先在循环体顶部加逐迭代 trace（迭代序号 + node/offset + 关键谓词值），用真执行路径对齐模型再选落点；side-band 数组（`globalThis.__zwXxxTrace`）+ 探针 fixture 消费已二次验证可行（closure 私有谓词引擎侧不可见，side-band 是唯一通道）。
- **落点修正**：吸收逻辑改为 walk cond1/cond2 的 invisible 判定 adjunct（`_zwA4TrailWsInvisible`：ws 文本 + 前向扫描遇块/br/耗尽即折叠真，遇非 ws 文本/含可见子树 inline 即假），`_zwJusIsWhitespaceNode` 父链 white-space 上下文判定使 pre 面天然不入（627 护栏零改动保绿）。不执行的 br-branch 吸收臂整体撤除。
- 判据选择教训：同名字谓词（collapsed whitespace / collapsed block prop / extraneous line break）语义各异且对「方向」敏感（回看 vs 前向），新支路判据不能照搬近名谓词，须按目标语义（CSS 行界折叠）直判。
