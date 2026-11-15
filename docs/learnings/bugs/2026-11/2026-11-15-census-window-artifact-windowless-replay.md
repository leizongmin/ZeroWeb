---
date: 2026-11-15
modules: engine,testing
---

# 批级 census 的稳定残红可能是窗切伪影：无窗单页复放配方定谳

## 问题描述

editing/run ip 语料按 1000 行窗切跑批，C4 垫刀后批级判读残 15F，且三轮跑批逐键稳定恒一。按 census 族桶归因得出 qV 读面 8 键＋C4②③ 各 1 键的「债」账——qV 8 因此挂账等域决策，C4②③ 立了「毒化前驱 bisect」待办。若照账开工，两边都注定失败：qV 读数无 bug 可修，C4②③ 的上下文毒根本不存在。逐案窗键核对后定谳：15F＝真红 5＋窗伪影 10，qV 8 全部是伪影。

## 根因

harness（include/tests.js runConformanceTest :5556）的执行结构给了伪影两个通道：

1. before/after query 是裸调用（无条件执行），execCommand 只活在 exec-return 子测里，而窗过滤可以把整组子测跳过；compare 对的是 exec 留下的 DOM。窗切口落在单案子测区间内时，exec 在窗 N 页跑过、compare 和 qV 却在窗 N+1 页跑——那是全新 document：defaultparagraphseparator 未置（读数恒 'div'）、host 未执行，got 就是纯 setup 形，签名＝no-op 加默认值读数。
2. 页首 carry 链重置让 before 读到 fresh 态。

确定性陷阱让伪影伪装成引擎债：切口固定、行布局固定，每轮跑出同一批伪影，于是 15F 三轮逐键恒一。「残红稳定」不是「残红真实」的证据，它只说明可复现。

## 解决方案

- 批级残红普查（census）必须逐案核对窗键：同一案的 exec-return 行与 compare/qV 行挂不同窗号，就是伪影（本次 bq、ulP 两案都是这种形态）；窗键前后一致且实跑的才是引擎债。no-op 签名（got 与 setup 输入形逐字相同）和默认值读数（qV 恒 'div'）是辅助指纹。
- 定谳配方：把涉事案 slice 成无窗单页探针复放（本次 zz-b4ul-pfx.html＝slice(0,302) 单页，实测 5014P/1F/0T）——伪影行应转绿、真红行保持红；红/绿分布对上预测即机制闭环。探针验证的是 harness 行为，不是引擎修复，不要把它当门禁腿。
- census 要把「族桶计数」和「真红分解」分开记：窗伪影 10（qV 8＋C4②③ 各 1）从债账除名，只记伪影不试图修；真红 5（C5 2＋I4 2＋I5 1）才是下刀输入。行级审计先例见[控制面挂账失败族开工前先核窗界分布](2026-11-07-window-straddle-accounting-audit.md)——本条补批级 census 面与无窗复放配方。
