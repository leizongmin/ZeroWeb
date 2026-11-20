# HTML、XHTML 与 DOM 接口控制面

**2026-11-20 · DC-2 批二完成（html/rendering 通道落地，Pass +2,551）**。批一
html/dom +1,888 零回归（[dc2 批一证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)）；
批二接入 html/rendering 语料（pin `31597693` 754 文件齐平，GITHUB_TOKEN 断点续拉）
＋独立执行通道，CI 零编辑（catch-all 正则命中）。release runner 150 文件 9,691
子测试：Pass 0 → 2,551，簇缺口 11,717 → ≈9,166；138/150 键分母逐字对齐，
既有套件零代码触点。根因探针定谳最大单根因：选择器引擎 `*` 全称选择器漏
非 HTML 命名空间元素（spec 要求全命名空间匹配）→ test-ua-stylesheet helper
对幽灵空样式比较 ≈4,400 Fail——批三主刀位
（[dc2 批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md)）。

**卡点** 无（周跑入账待 catch-all 分片首跑；engine/css-parser 存量 clippy/fmt
红为并行流面，非本 goal 域）。

**下一步** 批三主刀：选择器引擎 `*`/后代遍历补非 HTML 命名空间匹配（勿动类型
选择器的 HTML 限域——spec 正确面），预期 UA 样式面 ≈4,400 回收；次刀 iframe
子文档 defaultView/getComputedStyle（114 Fail）与 svg-embedded-sizing 完成面
（den ~1,294）、dimension attributes 映射残面（762）。DC-3 周跑对账需实体矩阵
（catch-all-8）+ rendering 分片入账与邻域哨兵。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；余簇缺口（11-20 快照）：semantics 13,962 / rendering ≈9,166 / syntax 6,298 / canvas 4,240 / webappapis 3,672 |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8，rendering 通道待 catch-all 首跑入账 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
