# HTML、XHTML 与 DOM 接口控制面

**2026-11-20 · DC-2 批一完成（html/dom +1,888，共享键零回归）**。DC-1 实体矩阵
25,606/25,606 全过（[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md)）。
DC-2 批一查账先行（wpt.fyi 分母 72,400 对 CI 产物分子逐文件），缺口形态三臂：
variant 展开机械臂（innertext 12 variant 键 570 Pass，分母逐字对齐）＋ idlharness
解锁资产臂（`interfaces/html.idl` + 11 依赖 IDL 补拉，3 variant 键 1,397 Pass）＋
引擎臂挂账（IDL 成员面 ~4,425 Fail 清单即缺员表；innerText getter 渲染语义 ~576；
reflection-* 块缺生成 ~3,000）。html-syntax 套件 61,840 → 63,728 Pass、共享键零回归
（详见 [dc2 批一证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)）。
html/dom 分子 59,035 → 60,923，簇缺口 ≈11,527。

**卡点** 无。

**下一步** DC-2 续：html/rendering 簇（缺口 11,717、0/529 文件——语料盘点 + 新通道，
xhtml-syntax DC-1 先例零 CI 编辑）；html/syntax html5lib write-mode 面（分母 61
variant 键/浏览器，树构造管道评估）；IDL 成员面战役批二。DC-3 周跑对账需实体矩阵
（catch-all-8）+ 本批（syntax-dom 分片）入账与邻域哨兵。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；余簇缺口（11-20 快照）：semantics 13,962 / rendering 11,717 / syntax 6,298 / canvas 4,240 / webappapis 3,672 |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
