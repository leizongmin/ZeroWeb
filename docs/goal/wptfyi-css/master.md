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
**DC-2 次批完成**：css-color declared/computed 通路（part11 色值管线接线 +
part06 null 即拒 + part01 computed `has` 陷阱/宿主抢答覆盖）——css-color 簇
62 页 **Pass 1,293 → 7,229**（+5,936），逐页不降核对 33 增/28 平/1 回归
（relative-color -14 记账，次批后半恢复）；T48 五钉 Blink 对齐翻正。
**DC-2 次批后半完成**：computed 真色彩数学管线（part11-computed-color.js
1,871 行接线：13 空间全精度矩阵/relative color 解析/color-mix 混合/none
策略/序列化）+ part03 raw 影子/part01c raw 读取分流 + NeedsPlain 关键字
排除——css-color 簇 63 页 **Pass 7,229 → 9,318**（+2,089），逐页不降
**9 增/54 平/0 回归**（前轮 -14 豁免页恢复持平，computed-relative 0→1,100、
computed-mix 0→930）。
**DC-2 再批完成**：declared 声明面归一（part11-computed-color.js 覆盖段
+527/-17：relative origin 折叠/color-mix method 与权重规范化/calc 规范形/
rest alpha 行为表/浅面基座委托）——css-color 簇 63 页 **Pass 9,318 →
10,530**（+1,212），逐页不降 **2 增/61 平/0 回归**（contrast 透传修复后
恢复）。残余 480 归三桶：color-layers 语法面（93 invalid + 10 valid）、
light-dark、杂项 computed（alpha-invalid、system-color 族等）。
**卡点**：无阻塞。
**下一步**：DC-2 再批未 fetch 目录语料扩面（~8.1k）；`.any.js` 变体通道
（588）；color-layers/light-dark 建模；DC-3 全量对账。

[执行契约](../wptfyi-css.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | **完成** | [基线与三分类](evidence/2026-11-10-dc1-baseline-and-classification.md)；快照基线 29,958/196,461（[coverage-2026-11-09](../../compat/wptfyi/coverage-2026-11-09.md)） |
| DC-2 按簇收敛 | 再批完成 | [Typed OM API 入口面](evidence/2026-11-10-dc2-part10-typed-om-api.md)：typed-om 簇 25 → 9830；[css-color 色值管线](evidence/2026-11-10-dc2-part11-color-pipeline.md)：color 簇 1,293 → 7,229；[computed 数学管线](evidence/2026-11-10-dc2-part11-computed-color.md)：color 簇 7,229 → 9,318；[declared 声明面归一](evidence/2026-11-10-dc2-declared-normalization.md)：color 簇 9,318 → 10,530 |
| DC-3 全量对账 | 待执行 | 验收线 182,392 项通过，同 revision 周跑产物待补 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
