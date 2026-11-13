# attr() font 槽写面拒收 + var 实参 deferred 形：写面豁免 + 后置重裁站

2026-11-14。g31 css-values 对账 −13 回归根因修复（洞 A 写面豁免 +
洞 B deferred 形后置重裁站），并复账 [bolder/lighter 批](2026-11-14-bolder-lighter-relative-weight.md)
终验链发现的账面问题。

## 1. 起点

- zw-g31-verify-0210 对账链（02:08 二进制，g27–g30 + g31 WIP）：css-values
  **3,920P**，对 v14-trigfix 基线（3,933P，11-13 23:19）**−13**：
  attr-all-types −8（attr 31/33/35/38/69 等 font-weight attr 形）+
  attr-argument-grammar −5（sub 0/1/2/12/14，`attr(var(--x))` 族）。
- 失败形态全部一致：`expected "10" but got "400"`——attr() 替换值在
  font-weight 槽被拒、回落初始 400。

## 2. 归因链

**分解探针**（zz-g31-fwattr，静态 `<style>` 与 setProperty 双站）：静态站
plain attr 全过（t3 与语料 attr 31 同形）、写面站全灭 → 定位写面。

**trigfix 通过的真相（对称失败）**：语料 helper 双侧对账
（`test_valid_attr` 以 expectedElem 的 gCS 为期望）。trigfix 时代
`parse_font_weight` 域 [100,900] 百倍数——`expected "10"` 同样不可解析，
双侧都回落 400 相等即过；attr 替换链从未被该 5 腿实证（argument-grammar
sub 9/13 当时已挂）。g29 连续域 `<number [1,1000]>` 让 expected 侧真出
"10"，对称打破，elem 侧缺陷暴露。

**洞 A（写面拒收，8 腿）**：g29 新增 font-*/g30 font-size-adjust 写面
文法分支（apply.rs `validate_decl_value`）只放行 math 形
（`is_math_function_value`），attr()/var() 替换函数文本落入
`apply_property_value` oracle → parse_font_weight 对替换前文本恒拒 →
setProperty 整体拒收 → 元素无声明 → 400。级联 plain attr 替换机制本身
完好（静态 t3 过；trigfix attr 153 union 形经 inline_decls 替换实证）。

**洞 B（deferred 形无重裁站，5 腿）**：`attr(var(--x))` 实参含 var()——
`parse_attr_call` 标 deferred（`has_substitution`），但
`eval_attr_call` 对 deferred 恒返 None，预级联站（lib.rs 2707
`substitute_attr_values_deferring` 前身）把 None 当 IACVT **丢整条声明**；
cascade.rs `is_cascade_value_valid` 注释所称「4.75 步」后置重裁站从未建成
——var 代入后无人重评。静态规则形（探针 s1–s3）同样死，属预存缺口
（非本轮回归），本轮一并收口。

## 3. 修复（分文件）

- **apply.rs**（洞 A）：font-*/font-size-adjust 写面分支豁免集补
  `value_has_substitution_function`（既有 math 门豁免集，attr(/var(/env(
  等 9 needle 子串扫描）——替换函数形放行 store 原文，computed 期级联
  替换后按属性文法重裁。+validate 豁免钉面单测（attr type(<number>)/
  var()/legacy number/font-stretch/font-size-adjust 形 + `auto` 拒面
  不受影响）。
- **attr.rs**（洞 B 前半）：`substitute_attr_values_checked` 扫描体提为
  `substitute_attr_values_impl`（defer 参数），新增
  `substitute_attr_values_deferring`——deferred 调用不求值不拒、原文
  保留（混合形具体调用照常替换）；严格变体语义不变（写面/后置站）。
  +单测钉（延迟保留/混合/具体 IACVT 拒/严格变体对照 6 断言）。
- **lib.rs**（洞 B 后半）：预级联站换 deferring 变体（deferred 形存活）；
  4.6 简写展开与 4.7 random 之间新建 **4.75 attr 重裁站**——
  `resolve_var_in_cascaded` 后重扫含 attr 词的非 custom 值：有部数档案走
  `substitute_attr_values_readjudicate`（结构冻结重裁），无档案走严格
  `substitute_attr_values_checked`：文法 invalid / IACVT → 丢整条
  声明；url 污染 + 取 URL 属性 → 同预级联站 attr-security 口径拒。每
  元素独立（attribute 值依赖），置 outcome 缓存之后（author_attr_active
  已绕缓存重算）。
- **结构冻结刀（arg_parts 档案，attr.rs + lib.rs）**：4.75 首版全量重评
  让 argument-grammar sub 8 翻红——`--x: data-foo type(<number>), 10` +
  `attr(var(--x))` 形，var 代入后 attr 调用凭空长出 fallback 实参，首版
  重评照单全收。css-values-5 语义：attr() 实参部数在声明 parse 期定型，
  var 代入只能填槽不能新增实参分隔。落地：AttrCall 增 `arg_parts`
  （parse 期顶层逗号切分部数），deferring 变体第三返回元带 deferred
  调用部数档案，预级联站按属性存 `attr_deferred_parts`（FIXME：跨层
  同名 prop 按扫描序拼接近似）；4.75 站按档案 ordinal 冻结重裁
  （`substitute_attr_values_readjudicate`，档案外调用不受约束）——部数
  漂移即整值 IACVT。+单测
  `readjudicate_freezes_argument_structure`（部数 [2] 拒/[1] 放行/
  档案外正常求值/`attr(!)` 照拒）。

## 4. 验证

- 分解探针修复后 **14/15**（唯一残挂为探针自造腿期望错误：`--x` 未定义
  → var 替换失败 → IACVT → 400 为正确 spec 行为；语料同形 sub 0 经
  setProperty 复刻腿通过）。探针跑后删除不入库。
- 单测：zero-css-parser **3,190 passed / 0 failed**（+1 deferring 钉测
  +1 结构冻结钉测）、zero-style-system **2,533 passed / 0 failed**
  （+validate 豁免钉断言）。clippy 双 crate `--deny warnings` 零告警；
  rustfmt clean。
- 定向复跑（语料子串过滤，修复后二进制）：attr-argument-grammar
  **14/15**（唯一残挂 sub 9）、attr-all-types **159/177**（+8 回位）。
- 排障记：debug 测试 `tree_counting_fold_number_and_unpack` SIGILL——
  stash 对照干净 HEAD 复现（预存）、release profile 同测通过 → 增量编译
  缓存毒化，`cargo clean -p zero-css-parser` 后全绿。非本批面，已沉淀
  [learning](../../../docs/learnings/bugs/2026-11/2026-11-14-debug-test-sigill-incremental-cache.md)。
- 七套件对账链 zw-g31-final（修复后二进制，diff 对 g31-plainarm）：见
  §6 终账。

## 5. 残余挂账

- attr-argument-grammar sub 9（custom property 值内 attr 形
  `--x: attr(var(--y), 3px)`）：custom property 站 attr 语义（2953 站
  严格 None 即丢）未动，trigfix 起在册预存失败，本轮零漂移保留。
- attr-all-types 其余 18F（trigfix 在册：<color> 型别 10 腿、type(*)
  嵌套 attr 4 腿、non-existent 族 4 腿）——色值/嵌套域非本批面。

## 6. 终账（zw-g31-final，2026-11-14）

三大师件页级对账（diff 对 results-g31-plainarm）：

- **css-values 3,920 → 3,935（+15，页级零回归）**：仅两页上涨——
  attr-all-types 150→159（+9 = 回归 8 腿全收复 + 1 腿超 trigfix）、
  attr-argument-grammar 8→14（+6 = 回归 5 腿全收复 + sub 13 超
  trigfix 收复，唯余预存 sub 9 挂）。对 trigfix 基线 3,933 净 **+2**
  全收复并反超；argument-grammar 15 腿期望 14/15。
- **css-color 10,972 → 10,972**：66 页逐页恒一（g13 锚）。
- **css-fonts 2,502**（plainarm 2,524 含已删探针页 22P；除探针逐页
  恒一）= bolder/lighter 批终账（2,472 + Rust 臂 8 + JS 臂 22）。
- 四小套件页级零漂移（zw-g31-small4 补跑账）：transitions **200**P
  （118 页）/animations **238**P（123 页）/transforms **229**P（107 页）/
  easing **51**P（12 页），四套件与 plainarm 逐页恒一、页数全齐。
  排障注：zw-g31-final 后段曾被 target-disk-guard 三级清库撞断（重型
  入口与在跑链同时发射的时序失误，runner 被清致后三套件缺席）；三大件
  账撞断前已落盘，四小套件由补跑链恢复，终账以此处数字为准。

单测终数：css-parser 3,190 / style-system 2,533 全绿；clippy
--deny warnings 零告警；rustfmt clean。
