# DC-2 g17/g17b 批：computed 通道 calc/calc-mix canonical 串化 + 配对伪通过转真

日期：2026-11-12　刀：engine 桥 `length_to_css` Calc 臂单面（两提交：
9dbe26c80 主刀 +72 行刀身、1e496148f 单参 min/max 恒等与 Abs 非负塌缩
补臂 +72/-2）；前置 g15 批收口（2,903P）。

## 1. 根因与实现面

- `js_dom_bridge/computed_style.rs` `length_to_css` 的 Calc 臂对一切含 %
  的 calc 值串化为空串——getComputedStyle 全部 calc-mix computed 断言
  `got ""`（勘根见 .acceptance 笔记 notes-g15b-computed-serialize-rooting.md）。
- 主刀（css-values-4 §10.11 computed canonical 口径）：全静态叶直接求值
  为 px；含 % 叶经加法脊项化（Add/Subtract 展平、calc-mix 字面权重项化
  clamp+缺失填充、全零塌缩取首项维度族）→ canonical 拼装：%
  项前置、零 px 项丢弃（零 % 项保留）、单项解包、多项保 calc(...)；
  不可 canonical 形（Expr 权重非零、TreeCounting 值项、混族非零权重、
  多参 min/max 含 %、乘除含 %）fail-closed 返空串不变。
- 补臂（g17 暴露后）：min()/max() 单参恒等塌缩（css-values-4 §10.4）；
  abs() 项全非负塌缩（|Σ| = Σ 门控，可能为负 fail-closed；hypot(A)
  单参形 parse 期已塌缩为 Abs）。

## 2. 关键发现：g15 的 58 案空串对空串伪通过

g15 在 minmax-length-percent-computed（30P）与 hypot-pow-sqrt-computed
（41P）的配对断言中，凡含 % 的形双侧皆串化空串 → `"" == ""` 伪通过。
主刀把 calc() 侧诚实化后 min()/abs() 侧仍 bail → 配对失配翻红 28 案
（g17 全量账 2,953P 较 g15 净 +50 表面含 -28 损失）；补臂把配对双侧都
接通，伪通过全部转真通过。

## 3. 全量账（css-values 单套件，256 页）

| 轮 | 二进制态 | P | vs g15 |
|----|----------|---|--------|
| g15 基线 | g14b+g15 刀 | 2,903 | — |
| g17 | 9dbe26c80（SIB+主刀） | 2,953 | +50 |
| g17b | 1e496148f + 他流在制 random 刀 | **3,032** | **+129** |

- g17 分页：增益 8 页 +78（calc-mix-serialize +35、calc-mix-computed +16、
  minmax-length-serialize +12、border-radius +8/+2、minmax-percent-serialize
  +3、letter-spacing +1、gradient +1）；翻红 -28（伪通过机制，§2）。
- g17b 分页 vs g17：percent-computed 4→30、hypot 39→41（补臂回收）、
  minmax/percentage serialize 与 calc-invalid/size-parsing 散页 +14
  （percent-serialize +3、percentage computed/serialize +2/+2、
  calc-invalid-parsing +4、calc-size-parsing +3）、
  random-invalid +39 与 attr-all-types -2（他流在制 random() 刀混入本轮
  二进制，归其账面）；**calc 族逐页零损失 vs g15**。
- 单元测试 7/7 绿（静态求值/塌缩族/% 保留/混合项/bail/min-max 恒等/
  abs 门）。

## 4. 法证与事故附记

- 06:43 他流原始六套件批与 06:44:52 二进制落盘竞窗撕裂（values 前后段
  异二进制、transcript 379/594 截断），其 -1585 账面为半截 JSON 合并
  伪影；他流 06:52 自行清理并重启。本流隔离探针证 06:44:52 二进制
  calc 族零回归，撕裂账撤销。
- 他流 06:52 16b 头部-only 账（净 +14、零回归）与其 07:07 组合态重跑
  （bin-g16 副本 = 本流 9dbe26c80 二进制 md5 同一）互证 g17 账
  （2953 vs 2953、256 页逐案零差）。
- 他流 random() 刀未提交面混入 g17b 账：+39/-2 两页归其名下，本批
  权威数以 calc 族页为准；其入库后如需净账可按同法分轮。

## 5. 挂账

- width used 通道 %-形需 layout 解析（桥层无 layout，Tier 2）。
- 多参 min/max 数值解析（minmax-length-percent-computed 10 案 + 
  minmax-length-computed 残余）。
- sibling-index 求值期 DOM 注入（他流挂账在册）。
