# HTML、XHTML 与 DOM 接口控制面

**2026-11-20 · DC-1 完成（试跑 25,606/25,606 全过）**。11-20 快照基线 html 顶级目录
73,076/159,876（对 11-07 快照 71,096 漂 +1,980＝并行流增益与上游漂移，分母链完整）。
11 月 8 日 html-sem/web-components 分片缺失已核对：护栏拦下坏快照未发布，11-10 起各
周跑恢复，11-12+ 快照 custom-elements/shadow-dom/html-semantics 计分完整。
`html/the-xhtml-syntax` 实体矩阵通道落地并试跑通过：12 外层页 25,606 子测试全过、
逐文件数与浏览器分母逐位吻合、外部实体零取回实证（详见
[evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md)）。
机制注记：DOMParser XHTML 面经 HTML 解析回退兑现（内建实体表超集，断言面不可区分），
真 XML 文档载入管道未建挂 DC-2。

**卡点** 无。

**下一步** DC-2 按簇收敛：以 11-20 快照未覆盖子测试数逐簇扩语料与修复——
html/semantics 13,962、html/dom 13,415、html/rendering 11,717、html/syntax 6,298、
html/canvas 4,240、html/webappapis 3,672；`.xhtml` 文档载入路径（XML MIME 管道）
按簇评估是否建。每周跑将实体矩阵入账（catch-all-8），DC-3 对账需邻域哨兵。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 待执行 | 缺口按 11-20 快照：semantics 13,962 / dom 13,415 / rendering 11,717 / syntax 6,298 / canvas 4,240 / webappapis 3,672 |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
