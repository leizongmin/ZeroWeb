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

## animation + inline 站收口（session 972a02c4，twin 提交）

认领板 01:15/01:24 两站全量落地（B2 收口后续作，color 站 + apply 接线
修复为 B2 后新增，未入 B2 账）：

- **animation 插值站**（width 516F + color 412F 池）：
  - `interpolate_rule_width_value`：有限 repeat 展开/单值广播（`repeat(2,
    20px)`→`30px` 以 2 位输出）/双 auto 循环体 LCM 对齐+保形/leading-trailing
    段数各等门/单侧 auto 离散/负外推钳 0；项插值 px lerp。
  - `interpolate_rule_color_value`：结构语义同 width（展开/广播/LCM/段数门）；
    项插值走 `lerp_color` 预乘 alpha 空间（css-color-4 §15.1，引擎既有通道），
    端点经 css-parser 全名色表 + `resolve_color_current`，a≈1→`rgb(r, g, b)`
    否则 `rgba(…, a.2)`。
  - lenient 臂（离散翻转回落 t≥0.5→to）+ strict 臂（失配 None 不启动过渡，
    css-transitions §starting-of-transitions）双通道接 `column/row-rule-{width,
    color}`；`underlying_individual_value` 补 width 臂（neutral keyframe 底层
    列表，`rule_width_list_to_gcs` pub(crate) 单源复用）。
  - **apply 接线修复（B2 后根因定位）**：B2 账 anim/waapi 中点形全 F，got=
    `.target` 级联值——`apply_single_property` 无 rule-width/color 臂，插值
    串计算后被丢弃（tick 输出不落 style）。补双属性组列表臂（插值串经
    css-parser 列表文法回 `Vec<RuleWidthSegment>`/`Vec<RuleColorSegment>`
    typed 字段，gCS 串化单源规范化 "23.00px"→"23px"）。此臂为 CSS
    Animations/WAAPI 页级增益的前置（B3 预期主要增益池）。
- **inline 简写展开站**（css_style_declaration.rs）：写面 `expand_shorthands`
  展开落 longhand（简写名不落盘，Chromium cssText 同形态）+ 读面 longhand
  重组（2 列全等/triple 逐位/rule 双轴全等）+ 初始分量省略（width medium
  锚）+ 段结构对齐门（失配→''，csswg#12201 同款）+ 非法值拒收。
- 验证：engine lib **4,098/0F**（B2 收口 4,096 + color 6 钉 + apply 1 钉；
  width 7 钉 B2 前已入账）；fmt/clippy 净（clippy 唯一 exempt = es_module
  peer WIP too-many-arguments 8/7 参在册连坐，调用级 `-A` 放行核本域零告警）。
- 端点关键字子测结构性放弃维持判：插值页 neutral 端点期望 'black' 关键字
  原文 vs gCS rgb() 归一（color-computed 页已赢面）不可兼得，保 gCS 归一。

## B2 插值页 F 池分解（B3 对账基线）

刀 B 插值页（excl. inset 刀域外 1,344F、excl. at(1) 端点 echo 恒真形）
F 池 750 分解：

| 形态 | 机制 | F | 归属 |
|---|---|---|---|
| explicit | anim/waapi | 274 | apply 臂缺位（**已修**，B3 入账池） |
| neutral | anim/waapi | 110 | 同上（underlying 补臂已在 B2） |
| explicit | trans-prop | 120 | 部分失配离散真预期 + 部分可插值形待 B3 验 |
| neutral | trans-prop | 63 | **共享缺口**（下条） |
| explicit | trans-all | 120 | transition-all 展开 FIXME（`property == "all"` continue，预存在册） |
| neutral | trans-all | 63 | 同上 + 共享缺口叠加 |

**共享缺口登记（刀域外，预存在册）**：neutral CSS-Transitions 中点形全灭
（5 corpus 175F，column-count 成熟臂同形 0P/10F 三轮持位 B0/B1/B2 实证）。
gCS 登记点 `take_last_style` None 分支整跳（computed_style_cache.rs:905）——
harness flush 读后单 mutation 写形下 per-identity 快照链断点待探针定谳
（sel/handle 双通道 store 点覆盖差异嫌疑）。属 transition 通道共享路径，
非刀 B 域，另立站收口。

## B3 页级收账（净 +395 / 0 lost / 0 regressed）

twin 提交 `71b07c7d8`（animation 插值器 + inline 简写展开 + apply 接线修复）
五 corpus 对 B2（results-gaprules-b2）收账。**双链 provenance 法证**：
zw-gaprules-b3（01:59:04 发射）构建窗 es_module peer WIP 在树（launch 日志
git status 在册 `M crates/script-sandbox/src/es_module.rs`）→ 二进制
`e3a46906…` WIP 扫入形（与 stash 窗干净构建 `58ec871f…` md5 异证；56s 快建 =
复用 WIP 态 script-sandbox 缓存，59s 干净建含 script-sandbox 重编），留作
b3wip 参照账。权威链 zw-gaprules-b3c（02:06:53 发射，unit 单写者独占窗）钉定
干净二进制 `58ec871f3041a8d12ce1510b795e2f1b`（无构建步，WIP 已回树但不进
二进制）。

权威账（b3c vs B2，02:12:27 GATE-DONE 全腿 rc=0）：

| corpus | gain | lost | flips | NEW/GONE | pages |
|---|---|---|---|---|---|
| css-gaps | +305 | −1 | 0 | 0/0 | 75→75 |
| css-multicol | +91 | 0 | 0 | 0/0 | 91→91 |
| css-typed-om | 0 | 0 | 0 | 0/0 | 359→359 |
| css-shapes | 0 | 0 | 0 | 0/0 | 145→145 |
| css-masking | 0 | 0 | 0 | 0/0 | 61→61 |
| **合计** | **+396** | **−1** | 0 | 0/0 | — |

净 **+395 / 0 页 lost / 0 regressed**。增益全落 B2 预期池（apply 臂缺位
池兑现）：gaps 7 页全在 animation/——row-rule-color 插值 +66、
row-rule-width 插值 +47、rule-color-repeaters-001 +89（含 −1 见下）、
rule-width-repeaters +59、rule-color-multiple-values-001 +15、
rule-width-multiple-values-001/002 +14/+14；multicol 2 页——
column-rule-color-interpolation +45、column-rule-width-interpolation +46。

- **−1 机理定谳（非语义回归，页 bug 侧）**：repeaters-001 WAAPI
  `column-rule-color from [repeat(auto, black, black)] to [repeat(auto,
  blue, red)] at (1.5)`——WPT 页 expected 串括号残缺笔误
  （`[repeat(auto, rgb(0, 0, 255), rgb(255, 0, 0)]` 缺右括号）。B2 的 Pass
  为 vacuous 假绿：apply 臂缺位时 got 与残缺 expected 同走我引擎
  parse→serialize 路径恒等（两侧 FAIL 消息反证：expected 侧序列化为
  `repeat(2, rgb(0,0,0), …)` 残缺形）；B3 apply 臂落地后 got 变真值
  `repeat(auto, rgb(0, 0, 255), rgb(255, 0, 0))`，揭穿假绿。同 case 的
  at(1) 形（正常 expected）B3 为 Fail→Pass 计入 +89。Transitions/
  Transitions-all 两 variant 该形 B2/B3 均 Fail（页 bug 持位，刀域外）。
- **−1 归因补强（session 972a02c4 探针复核，修正上文机理细节）**：残缺串
  在我引擎 `setProperty` 即拒收——单页探针（corpus 过滤通道，/tmp
  zw-probe-malformed，探针页已删不入库）实证：合法串
  `repeat(auto, rgb(0, 0, 255), rgb(255, 0, 0))` gCS 往返全等；缺尾括号串
  拒收后元素无声明、落初值 `rgb(0, 0, 0)`（非残缺串 parse→serialize 产物）。
  故 expected 元素读数 `repeat(2, rgb(0, 0, 0), rgb(255, 0, 0))` 实为拒收后
  回落页 `<style>` `.target` 基础级联值 `column-rule-color: repeat(2,
  black, red)`（页原文在册）；B2 期 WAAPI actual 未被动画、同读该基础值，
  两侧恒等成假绿。CSSA 通道同 case 六 t 全读基础值（B3 消息
  `expected "repeat ( auto , …" but got "repeat ( 2 , …" 五 Fail +
  1.5 假绿持位）实证 CSSA keyframes 通道独立残面。分类不变：上游 typo ×
  我方动画值转正确，非语义回归非 flake。
- **−1 外证闭环（2026-11-17 02:2x-02:40，harness 源码 + wpt.fyi + b3d 实测）**：
  ① harness `interpolation-testcommon.js` L210-224：WAAPI variant 的
  animate() keyframes **只用 case 级 from/to**（页 L61-62，双带右括号合法），
  残缺 L69 串只进 expected 元素 `setProperty`（measure() L385），从不进
  keyframes 通道——「keyframes 非法值丢弃 → to 键失效」修复方向对本子测
  无输入可丢（b3d 实测证：02:30 发射批 vs B3 五 corpus 页/子测集合全等、
  status 翻转 0，本实例独立复验非转抄；源 mtime 02:27:22 < 构建 02:29:57，
  二进制含终态校验仍零翻转）。② 外证：wpt.fyi API 直取 chrome-160.0.8157.0
  （2026-11-16，rev 3d8756fa）该文件整体 O 276/276 全过，而上游 master
  （同日取证）L69 typo 原样——Chrome 全过的唯一自洽形 = `repeat(auto,…)`
  在 setProperty 与 keyframes 双通道均拒收（L68 合法串 expected 也必落基
  级联）→ 双 vacuous pass；此为 Chrome parser 缺口（css-gaps-1 文法允许
  auto），非语义 pass 非 parity 目标——勿为 parity 扩成拒收 repeat(auto)
  （会翻转我方 at(1) 真 Pass，净 −2 真回归）。分类终态：上游 typo ×
  我方超集动画，账面 +395 持位。
- **s419 中性双证**：b3wip vs b3c 五 corpus 全 SET-IDENTICAL（逐子测
  0/0/0、页集合全等）——二进制 md5 异（`e3a46906` vs `58ec871f`）而结果
  逐项全等，script-sandbox es_module peer WIP 对五 CSS corpus 零位移；
  b3wip 参照账与 b3c 权威账互证升级。
- runner 层注：b3/b3c shapes/masking 腿 log `timeout=1`（case 墙钟边沿），
  JSON 终态 TIMEOUT 子测 0=0（B2 同 0），对账零账面影响。
- **inline 站余量复核**：from-longhands 页 4 对齐形 B3 零位移（12P/16
  持位，column/row-rule plain/repeat 双形仍 Fail）——写读面已落但 gCS
  简写重组在该形未翻绿，残面留队；rule-shorthand.html 240F 池零位移。
- **预期性归因修正（B2 F 池表 → B3 实测）**：B2 表把 anim/waapi 记为同一
  「apply 臂缺位池」，B3 实测只兑现 WAAPI 半；CSS Animations 通道 45F 全形
  got 恒底层值（row-rule-color 页 45 Fail got 签名 100% = `rgb(255,255,0)`，
  4 Pass 全为负 t/t=0 免插值 vacuous 形）——机理：WAAPI 有 js_anim_registry
  旁路时钟咨询 gCS（本刀 apply 臂生效面），声明式 CSS Animations 走渲染管线
  `apply_animation_overrides`（pipeline/mod.rs:4494，插值只落渲染层 styles
  副本），gCS 级联回调零动画钟参与（js_anim_registry.rs 文件头断点注释原话）
  ——CSS Animations harness 形态为纯声明式（共享 style 追加 @keyframes +
  target 设 `animationName/Duration:100s/Delay:-50s/TimingFunction=easing(at)`
  后直读 gCS，无 pause/seek）。独立刀面：声明式动画 gCS 咨询通道（镜像
  js_anim_registry 架构）。
- 单测/门禁：twin 提交时 engine lib 4,098/0F 已证（见上节），本轮纯收账
  零码变。

## 残面队列（B3 后更新）

- ~~color 真插值池 ~412F~~：**B3 已收账**（apply 臂兑现入 +395 账）。
- ~~width/style 真插值池 anim/waapi 384F~~：**B3 已收账**（WAAPI 通道
  主增益；CSS Animations 通道部分入账）。
- **9 增益页残余 ~715F census**（B3C 通道分解，下刀对账基线）：
  CSS Animations ~302（@keyframes 路径中点形——apply 臂只修 anim/waapi
  tick 落盘，keyframes 通道独立残面）、CSS Transitions with transition
  (all/shorthand) ~241（transition-all 展开 FIXME + neutral 快照链叠加）、
  Web Animations ~118、纯 CSS Transitions ~30、transition-behavior/
  -property 变体 ~24。
- inline 站余量：from-longhands 4 对齐形（B3 复核零位移，gCS 简写重组
  该形未翻绿待探针）+ rule-shorthand.html 240F 池。
- 共享缺口：neutral CSS-Transitions 快照链（B2 F 池分解表在册）。
- 刀 D：inset-cap/junction 几何族 ~1.7k 最大残面。
- 残面⑤：calc 代数插值全闭环（承重拒收边界重撤硬前置）。
