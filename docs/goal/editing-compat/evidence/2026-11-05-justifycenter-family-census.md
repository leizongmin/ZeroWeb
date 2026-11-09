# ⑥ justifycenter 真实残差族谱普查（只读窗——零源码修改）

日期：2026-11-05　证据源：/tmp/qv-fix4/fix10-justifycenter.log（6,078P/125F，⑩ 轮门禁
基线；⑪ 审计口径 114 真实 + 11 切片伪影）　判读脚本口径：行级 fail + 首随行 expected/got

## 125F 面分布

compare innerHTML **105** + 读面 20（qV defaultparagraphseparator 7 / qV justifycenter 6 /
qS stylewithcss 4 / qS justifycenter 3；⑪ 伪影集中于此，残余真实读面随 ④ 机制域）。

## compare 105 结构签名分桶（expected↔got 标签序差集）

| 桶 | 行数 | 签名 |
|----|------|------|
| 1 | 30 | expected 多 1 枚 `br`，got 无 |
| 2 | 22 | got 多 2 对 `div`（含 table 16 行） |
| 3 | 20 | got 多 1 对 `div`（含 quasit 4 行） |
| 4 | 14 | 标签序同、attr 形不同 |
| 5 | 6 | expected 多 `br` 且 got 多 2 对 `div`（桶 1×2 复合） |
| 6 | 4 | expected `div`↔got `center`（壳换代） |
| 7 | 4 | 标签序同、文本/ws/选区标记位不同 |
| 8 | 1 | assert_in_array（多形族） |

## 机制归因（五头，含 got/expected 实证）

**J-A 既有对齐祖先壳改写缺失（桶 3+4 主部 + quasit + 部分 14，~38 行）**：Chrome 的
justifycenter 落点 = **改写既有块级/对齐祖先壳的面**，引擎一律在最内层新嵌 div 并保留
旧壳。三形：
- attr→style 换形：`<div align=center><p>[foo<p>bar}</div>` → expected
  `<div style="text-align:center">…`，got 保留 `align="center"` 原样（14 行 attr 桶主形）；
- 内层嵌套 vs 祖先改写：`<div align=justify><p>[foo<p>bar}</div>` → expected 同一 div
  换 `style=text-align:center`，got `<div align="justify"><div style="text-align:center">…`
  双层嵌套（桶 3 主形）；
- quasit 外裹+attr 摘除：`<quasit align=right><p>[SEL]</p></quasit>` → expected
  `<div style="text-align:center"><quasit><p>foo</p></quasit></div>`（quasit 摘
  align=right、上提外裹），got 内嵌且 attr 保留（4 行）。
（与 ③'-a 规则互补：对齐 face 命令的落点=最近对齐祖先壳的改写，非内层包裹。）

**J-B table 落点层级（16 行 ⊂ 桶 2）**：table/tbody/tr align 四变体全数——expected
把**整个 table 外裹** `<div style="text-align:center">`（对齐目标块=table 元素级），
got 逐 cell 内部段落各自包裹（块爬升不越过 table）。Chrome 的 blockExtend/目标块
选择把 table 当单块。

**J-C 跨壳合并边界 br 保留（桶 1+5+caret 族 ≈ 36 行）**：跨壳选中（`<center>[foo</center>bar]`
型、`<div align=center>{}</div>` caret 型）apply 后 expected 合并壳**保留壳边界为
`<br>`**（`foo<br>bar`），got 文本直接拼接 `foobar` 丢边界；caret 空内容型 expected
div 内容 `<br>` 占位。方向与 ③'-d ghost br 相反——同一条「块边界 ↔ br」原语，引擎在
apply 面吞掉 Chrome 要求保留的边界 br（③'-d 是 merge 面多插 br）。修 primitive 时
两面须一并核。

**J-D center→div 壳换代（桶 6，4 行）**：`<center><p>[foo<p>bar]</center><p>extra` →
expected center 元素整体换代为 `<div style="text-align:center">`，got center 保留不动
（J-A 的元素专用形：center 不可直接改 face，Chrome 走换代）。

**J-E ws/标记位微形（桶 7，4 行）**：` <p>` vs `<p> ` 空白载体位置、选区标记落点差异，
量小随 J-A 主刀顺带。

## 修复方向预记（下轮实施窗）

J-A/J-D 同刀（落点=最近对齐祖先壳改写 + center/quasit 换代/外裹规则 + attr→style 换
形）≈ 42 行主收益；J-B 独立刀（table 目标块层级）16 行；J-C 需与 ③'-d 共享 primitive
设计（边界↔br 双向）≈ 36 行。全部须避让并行轮 fix14 门禁窗口；J-A/J-B/J-C 刀后跑
justifycenter 全语料 + multitest 守卫腿 + enginelib 终核。
