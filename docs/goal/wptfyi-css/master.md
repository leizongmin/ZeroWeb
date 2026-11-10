# CSS 子测试控制面

**2026-11-10 · Active**。目标净增 153,000 项（绝对线 182,392）。11-09 发布
快照基线 29,958 / 196,461（15.25%），待回收 152,434。
**DC-1 完成**：四焦点目录本地重跑复现快照（typed-om 25 / color 1,293 逐位
一致），缺口三分类入账——运行面断言失败 ~139.5k（Chrome 同面过 95%）、
未接语料 ~8.1k+缺文件 ~10.5k、通道外形态余量。
**DC-2 首批完成**：Typed OM API 入口面（part10 ~1,810 行：CSSStyleValue
类族 + CSS 工厂 + StylePropertyMap + attribute/computed/rule styleMap +
225 属性语法表；part06 styleMap 挂钩）——css-typed-om 簇全量重跑
**Pass 25 → 9830**（365 页，65 页零失败），残余 2,383 归四桶（简写长槽
缺面/computed 颜色管线/computed transform/`.any.js` 通道）。
**卡点**：无阻塞。
**下一步**：DC-2 次批 color-mix()/现代色函数 computed value 通路（color
目录单簇 7,199）；再批未 fetch 目录语料扩面（transitions/animations/
om-view/view-transitions/geometry/properties-values-api 等 ~8.1k）；
`.any.js` 变体通道（typed-om 588 子测试）。

[执行契约](../wptfyi-css.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | **完成** | [基线与三分类](evidence/2026-11-10-dc1-baseline-and-classification.md)；快照基线 29,958/196,461（[coverage-2026-11-09](../../compat/wptfyi/coverage-2026-11-09.md)） |
| DC-2 按簇收敛 | 首批完成 | [Typed OM API 入口面](evidence/2026-11-10-dc2-part10-typed-om-api.md)：typed-om 簇 25 → 9830；次批 computed 颜色管线 |
| DC-3 全量对账 | 待执行 | 验收线 182,392 项通过，同 revision 周跑产物待补 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
