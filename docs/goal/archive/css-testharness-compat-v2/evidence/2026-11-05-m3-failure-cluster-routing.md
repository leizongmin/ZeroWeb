# M3 失败簇承接归口——grid 探测簇与 color 计算值面（2026-11-05）

一期收口挂账的四簇中，两簇处理结论如下落账；anchor-size 就地修与十六目录
零回归对照两件的状态见末节。

## 归口总表

| 一期挂账簇 | 体量 | 结论 | 去向 |
|---|---:|---|---|
| color Color-4 计算值面 | 7,199 | 非单点，挂账不翻案 | color 域专项（engine computed 值管线） |
| grid 布局探测簇 | 3,280 | 布局引擎缺口，具名挂账 | layout goal（立项条件已触发，待拍板） |
| anchor-size() 序列化参数序 | ~2,500 | 引擎可修单点，就地修 | 本 goal M3，进行中 |
| text 断行 approx 面 | ~543 | 已在一期归口断行语义 | 维持一期归口，本 goal 不动 |

## grid 探测簇（3,280）→ layout goal

一期 css-grid 台账（archive
`2026-10-30-m3-batch5-grid-text.md`）把 6,393 个失败分成两类：

- **布局探测 3,280 + Timeout 76**：check-layout-th 的
  `data-expected-*`/`data-offset-*` 与 client 尺寸断言，在未实现的 grid
  布局引擎上回退或等待——布局引擎缺口的直接账面。
- **解析/序列化长尾 ~1,700**：computed 序列化枚举 893、computed 探测
  483、canonical 序列化 375、泛型 assert_equals 1,062、assert_true 210
  中的非布局部分——延续 CSSOM 既有归口，不随布局账走。

处理结论维持 M0 甄别（`2026-11-04-m0-triage-and-gate.md` §7）：

1. **具名挂账 layout goal，不在本 goal 就地修**。立项触发条件已实测
   成立：布局耦合池 22,566 子测试（B3 十三目录 19,286 + 一期 grid
   3,280），按解析份额 30% 上限折算后布局探测地址池下限仍 > 15k，超过
   10k 立项线。测算与立项建议已通报，goal 文档待用户拍板后立项。
2. **B3 十三目录照常导入入账**：本 goal 修到序列化/声明值层为止，目录
   里解析份额照常计门，探测面失败不就地修。
3. **账本去向**：B3 探测面失败簇默认记 rendering-compat 控制面（一期
   grid 探测簇同款）；批次 4 干跑基线（B3 十三目录）出分簇数据后按实际
   簇形修正，修正写进批次 4 台账。

## color Color-4 计算值面（7,199）→ 维持一期归口

一期 css-color 分簇（archive
`2026-10-30-m3-batch1-color-align-sizing.md`）：失败 9,763 中
**7,199（73.7%）**为同一簇——`color doesn't seem to be supported in
the computed style`。getComputedStyle 对 color-mix、oklch、light-dark
等 Color-4 形态没有 computed 支持，断言在计算值读取处直接失败。次簇为
canonical serialization 579、序列化 assert_equals 838、color-mix 色域
换算误差 ~21。

归口评估结论——**承接一期归口，不翻案**：

1. 该簇是 engine computed 值管线域（Color-4 解析、色域换算、computed
   序列化一套），不是可就地修的单点。一期已定性「引擎工作项评估归
   color 域专项，不在本 goal 修引擎」，本轮维持。
2. **不进 DC-4 数值门**：门公式只含新目录导入收益（基线 17,923 + B1
   1,672 + B2 525 + B3 386），一期目录内的修复收益本就不在门内，挂账
   不改变门数学。
3. 挂账形态与 typed-om 未实现域同款：具名挂 color 域专项，后续 goal
   评估切入点（computed 串行化单点优先于整套色域换算）。
4. 次簇 canonical serialization 579 属 CSSOM 序列化既有归口，不随
   本簇挂账。

## anchor-size（~2,500）就地修与十六目录零回归——已落地

- **anchor-size() 参数序**：已就地修（2026-11-05，提交 6be1aae94）：
  zero-css-parser 值级 canonical 归一 + engine 三处声明值写入路径接线，
  `--*` 按 spec 跳过；css-anchor-position 通道 5,205P/40.6% →
  7,457P/58.2%（+2,252），残留 12 为 calc/fallback 单位归一相邻簇归
  CSSOM 既有账。台账见
  [2026-11-05-m3-anchor-size-serialization](2026-11-05-m3-anchor-size-serialization.md)。
- **十六目录零回归对照**（DC-3 末件）：已执行（2026-11-05）——逐
  subtest 对照一期 weekly CI 基线，修复触达面 0 回归；对照中出现的
  385 子测回归经无修复对照二进制集合级 A/B 证明与修复无关（main 六日
  他流演化，记对端账）。台账见
  [2026-11-05-m3-sixteen-zero-regression](2026-11-05-m3-sixteen-zero-regression.md)。

## 证据

- 一期 grid/text 分簇：[archive 2026-10-30-m3-batch5-grid-text](../css-testharness-compat/evidence/2026-10-30-m3-batch5-grid-text.md)
- 一期 color 分簇：[archive 2026-10-30-m3-batch1-color-align-sizing](../css-testharness-compat/evidence/2026-10-30-m3-batch1-color-align-sizing.md)
- 布局池测算与 B3 目录清单：[2026-11-04-m0-triage-and-gate](2026-11-04-m0-triage-and-gate.md) §7
- anchor-size 一期定性：[archive 2026-10-30-m3-batch3-anchor-position-masking-backgrounds](../css-testharness-compat/evidence/2026-10-30-m3-batch3-anchor-position-masking-backgrounds.md)
