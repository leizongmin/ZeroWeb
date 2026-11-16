# gap-rules 刀 B：width/style 列表文法 + rule 简写家族（二十五轮）

日期：2026-11-17 深夜。对应设计稿：[gap-rules-knife-design](2026-11-16-gap-rules-knife-design.md)
刀 B（估 +400~600F）全量落地 + gap-rule 联合简写文法（原刀 C 范围一并吸收）。
执行形态：rate-limit 多重 spawn（68e17414 四实例并发 resume），按认领板
`.acceptance/wptfyi-css/notes-gaprules-knife-b-claim.md` 对称收敛——css-parser
站两实例互补合流（width 档 + style/gap-rule 档），style-system 站单实例，
engine 站 01:04 停滞窗由 00:54 实例接手收口。

## 刀体四站

- **css-parser**：三新档 + 一共享接口化。
  - `parse_rule_width.rs`：`RuleWidthSegment{Plain,Repeat}`，条目复用 multicol
    既有 `ColumnRuleWidthValue`（keyword 保形）；`parse_line_width_item` 单
    token 门（括号感知——`calc(10px + 0.5em)` 实参空格合法、顶层空格分隔
    复合形拒）、`auto` 拒、% 拒（<length [0,∞]>）、负长拒、math 函数保
    Calc 树。钉 4。
  - `parse_rule_style.rs`：`RuleStyleSegment` 同构，`parse_line_style` +
    `line_style_str` canonical 小写。钉 4。
  - `parse_gap_rule.rs`：联合简写文法 `<gap-rule> = [<line-width> ||
    <line-style> || <line-color>]`——`GapRuleComponent` 三维度至多各一
    （段内重复同维拒）+ `*_raw` 原文字段（specified 投影保形：`hotpink`
    不归一 rgb()，shorthand.html 页钉）；`parse_gap_rule_list` + 三维投影
    `project_{widths,styles,colors}`（缺省补 medium/none/currentcolor，
    repeat 保形）。钉 4。
  - `parse_rule_color.rs` 共享接口化：`split_top_level_commas` pub、
    `parse_rule_repeat_count` pub、`RuleRepeatCount::to_css_head` pub
    （三列表档 + 联合档共用计数文法/切分/串化）。
- **style-system**：`column_rule_width`/`column_rule_style` 迁移 Vec 段 +
  `row_rule_width`/`row_rule_style` 新字段；registry/default/inherit/apply
  全臂；VALIDATED_PROPERTIES + width/style 族（简写裁决走 shorthand 展开）；
  math_slot_family_valid LengthOnly 行 +2；shorthand 地图 +4（row-rule/
  rule/rule-width/rule-style）+ 展开器 `expand_gap_rule`（GapRuleTarget
  Row/Column/Both，投影值落 longhand）+ `expand_rule_dimension`；apply
  臂（apply_advanced）rule 族简写 apply 全过才落（rule-color 臂同款）。
  computed.rs resolve 逐段 Length 臂（R907 同模式，row 同文法）+
  <length [0,∞]> 负计算值 clamp 0（`calc(10px - 0.5em)` → `0px`，computed
  页钉）。
- **engine**（gCS/paint）：
  - `rule_width_list_to_gcs`/`rule_style_list_to_gcs`：单段平值走既有标量
    路径（multicol 纯宽/纯样式声明串化零漂移：thin/medium/thick → used
    px 1/3/5 R2755 oracle 口径），列表/repeat 形结构串化。
  - row-rule-width/row-rule-style 读臂 + rule-width/rule-style 简写臂
    （row/column 串化全等口径，rule-color 臂同款）。
  - `gap_rule_recomb_to_css`：row-rule/rule 简写重组——width/style/color
    三维度逐段 zip（repeat 保形：段循环体逐项 `w s c` 拼接，WPT computed
    页 `repeat(auto, 5px solid rgb(0, 0, 255))` 恒等钉；三维度段数失配
    回落平铺）。
  - paint（text_multicol）：首段首宽/首样式提取（`rule_width_list_first_width`
    等，列表逐段渲染后置 FIXME 挂账）。
- **钉**：css-parser 12（width/style/gap-rule 各 4）。

## 单测与门禁

- css-parser 3,293/0F、style-system 2,548/0F、engine lib 4,077/0F 全绿。
- fmt 三 crate 净；clippy 三 crate 净（唯一 warning = es_module peer WIP
  在册连坐 too-many-arguments 8/7，非本刀域）。
- 途中修复三笔：width 条目顶层空白门括号感知化（`calc(10px + 0.5em)`
  valid 形误拒——top-level 空白 vs 函数实参空格）；gap_rule 投影色关键字
  保形（`*_raw` 原文字段，canonical rgb() 仅 computed 期）；clippy 三连
  （question_mark else-if 链、悬空 doc 注释、collapsible_if filter 化）。

## A/B 权威账

- B0 = `results-rulecolor-final`（刀 A 终态链，b1v3 binary `e2bb5d29` =
  HEAD 码态全等；01:00 docs-only 提交 3942d7892 不改码）。
- B1 = `results-gaprules-b1`（unit `zw-gaprules-b1` 01:17 发射，单写者
  独占窗，bin md5 见 bin-gaprules-b1/）。五 corpus：gaps/multicol/typed-om/
  shapes/masking。

B1 全账（01:17:45 发射，01:23:39 DONE，runner md5
`8289d468493be92e9cceb351600fdf9a`；对账脚本
`.acceptance/wptfyi-css/reconcile-knife-b.py`，PAGE-ONLY 口径）：

| corpus | gain | lost | flips | NEW/GONE | pages |
|---|---|---|---|---|---|
| css-gaps | +252 | −6 | 0 | 0/0 | 75→75 |
| css-multicol | +7 | 0 | 0 | 0/0 | 91→91 |
| css-typed-om | 0 | 0 | 0 | 0/0 | 359→359 |
| css-shapes | 0 | 0 | 0 | 0/0 | 145→145 |
| css-masking | 0 | 0 | 0 | 0/0 | 61→61 |
| **合计** | **+259** | **−6** | 0 | 0/0 | — |

增益族分布（gaps 侧）：style computed +54、rule-width 插值 repeaters +50、
width invalid +27、rule-shorthand computed +26、width computed +24、
style invalid +18、rule-shorthand invalid +16、row-rule-width 插值 +16、
multiple-values-001/002 +16、bidirectional +3、col-rule-width +2；multicol
侧 width/style invalid+computed +7（零回归护栏下顺手增益）。

## 回归钉与修复（B1 → B2）

−6 全在 `gap-decorations-rule-shorthand-from-longhands.tentative.html`
row-rule 误配形。根因：B0 期 `row-rule` 未实现，`e.style.rowRule` 与 gCS
双返空串 vacuous 过；B1 gCS 臂落地后，重组器对三 longhand 结构不齐形
（Plain/Repeat 混段、repeat 计数异型、循环体长度不齐）产出平铺串而非
空串——csswg-drafts#12201 规定 longhand 不对齐时简写序列化为空串（页内
注释「the longhands do not line up」+ 6 形 expected `""` 钉）。

修复（computed_style.rs）：`gap_rule_lists_aligned`（pub(crate)）——段数
相等 + 逐段同为 Plain / 同为 Repeat 且计数折叠口径全等（`repeat(calc(5+3),…)`
≡ `repeat(8,…)`）+ 循环体长度全等；recomb 失配返空串（替代原平铺回落）；
`column_rule_shorthand_to_css` 同门（column 侧 6 形同钉，预存 Fail 转
Pass 预期 +6；multicol 单段形恒对齐，R2755 oracle 零漂移——multicol corpus
+7/0 已证）。row/column 侧 4 例对齐形（expected 非空）待 inline 简写站
（css_style_declaration 读面/写面，认领板 01:15/01:24 在册）落层后 B2 复核。

## B2 权威账（刀 B 收口）

B2 = B1 树态 + 结构门修复 + animation 插值站（认领板 01:24，
`interpolate_rule_width_value` 展开/广播/LCM/段数门 + 7 新钉）+ inline 站
01:30:27 快照（css_style_declaration.rs 部分落地）。01:32:13 发射
（unit `zw-gaprules-b2`，runner md5 `7eebe279020c3ab084cf1b1b54458ac6`），
01:39:18 DONE。

| corpus | gain | lost | flips | NEW/GONE | pages |
|---|---|---|---|---|---|
| css-gaps | +258 | 0 | 0 | 0/0 | 75→75 |
| css-multicol | +7 | 0 | 0 | 0/0 | 91→91 |
| css-typed-om | 0 | 0 | 0 | 0/0 | 359→359 |
| css-shapes | 0 | 0 | 0 | 0/0 | 145→145 |
| css-masking | 0 | 0 | 0 | 0/0 | 61→61 |
| **合计** | **+265** | **0** | 0 | 0/0 | — |

- B2−B1 delta = **+12 纯门修复归因**（−6 row 误配复位 + 6 column 误配
  Fail→Pass），其余全 corpus 逐键零漂移。
- from-longhands 页终态 12 Pass / 4 Fail：12 误配形全过；4 对齐形
  （row/column 各 2，expected 非空）留 inline 站收口（e.style.rowRule/
  columnRule 写读面）。
- animation 站本 corpus 页级增益 +0（钉级 127P 与页级接线残差——插值页
  B1 已吃端点形 +74，中点形需 B3 页级接线复核）。
- 收口门禁（树态含 animation+inline 站）：engine lib **4,096/0F**
  （4,077 + 双站新钉）、css-parser 3,293/0F、style-system 2,548/0F、
  fmt 三 crate 净、clippy 三 crate 净（唯一 warning=es_module peer WIP
  在册连坐）。

残面移交：inline 站收口（4 对齐形 + rule-shorthand.html 240F 池）、
animation 页级接线 B3 复核、color 真插值池 ~412F、刀 D inset-cap/junction
~1.7k、残面⑤ calc 代数插值全闭环。

## 残面队列（更新）

- color 真插值池 ~412F（列表插值器缺位）；width/style 真插值
  （rule-width-interpolation-repeaters 252F + row-rule-width 168F + 多值
  96F，需列表插值器同基建）。
- 刀 D：inset-cap/junction 几何族 ~1.7k 最大残面。
- 残面⑤：calc 代数插值全闭环（承重拒收边界重撤硬前置）。
