# kc5r124 · @function 体条件规则组刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 431（b0 实测）→ 449（+18）**＝judge VERDICT PASS＋
verify VERIFY-PASS＝预注册通过 P2F=0、GONE=NEW=0。净 +18＝
function-conditionals **17 键**（22 键页 held 全清＝本刀主域
css-mixins-1 §conditional-rules）＋function-container-self **1 键**（CQ
链排除查询元素自身语义兑现）。43 页 521 键恒同、Timeout 恒同。

## 刀形

css-mixins-1 §conditional-rules（@function 体条件规则组）＝解析端 AST
条目树＋求值端条件展平双端刀：

**解析端（css-parser）**——`FunctionRule.decls: Vec<FunctionDecl>` →
`body: Vec<FunctionBodyItem>`（Decl | Conditional 组，组 kind 三臂
Supports/Media/Container）；`consume_function_body_items`：RBrace/Eof
终止、Semicolon 跳过、AtKeyword → `consume_function_conditional_group`
（**开头 advance 消耗 AtKeyword**——首版漏消耗致 prelude 吞 `@supports`
token 解析恒败整组 skip，嵌套树探针实锤后修复）、失败
`skip_function_conditional_extent` 整跳（prelude 深扫＋skip_simple_block
边界保形）；嵌套递归 `consume_function_body_items`。

**求值端（style-system）**——`flatten_function_body(items, cond, out)`：
Conditional 组 `function_cond_holds` 三臂判定（@supports →
`evaluate_supports_condition` 静态支持表；@media →
`evaluate_media_query × media_ctx_for`；@container → 容器链快照合成无名
`ContainerRule` 走 `evaluate_container_condition`——matcher 函数提
pub(crate)）；真组按源序原地展平（§4 last-wins 与平面声明一致）、假组
整跳、嵌套递归；None 语境（无视口/无容器链）臂 false＝规范「不可判定
→ 组不生效」。`CondEvalCtx { media, container }` 穿线
evaluate/substitute 全递归点（6 处）＋lib.rs 4.755 站组装。

**not 前缀否定（本刀衍生补域——嵌套两键 held 根因）**——判据
`(not (height: 99999999px))`（Nested @media）与 `(not (height = 75px))`
（Nested @container）首版整组被 skip：media 侧
`parse_conditions_from_inner` 冒号臂吃 "not (height: …)" 恒败
（feature="not (height"）返空 vec＝query 不匹配；container 侧
`parse_container_condition` 无 not 臂返 None。修＝
- `MediaCondition::Not(Box<...>)` 变体＋not 前缀臂（括号包裹
  media-condition，mediaqueries-5 §media conditions；内层不可解析＝未知
  条件整 query 不匹配）＋evaluate 取反臂——query 级 not 前缀由
  parse_single_media_query 的 negated 字段承载不经本变体
- `ContainerCondition::Not(Box<...>)` 变体＋parse_container_condition
  not 臂（递归解包，多层叠加语义成立；`not style(...)` 走 Style 变体
  自带 negated 不经本变体）＋matcher `eval_one` 解包计数取反（防御式
  while）
- **unknown 传播门**（compound roundtrip 钉测暴露）：`not (garbage)`
  （random() 函数值形）旧 garbage-Size 恒 false＝unknown 不应用；裸
  Not 变体直取反得 true＝unknown 语义缺失（ζ 刀 random-in-container
  恒不应用红线翻转险）——Not 域两门：值 px 不可解析／轴值缺席 → 整体
  false（早退不走尾取反＝unknown 天然保持）；范围形界 garbage 由 range
  臂早退承载
- css_wire 序列化 `Not` 臂外卷 `(not …)` 与括号包裹解析形恒等回卷
  （wire_container_compound_or_condition_roundtrip 钉测恒同；首轮红＝
  本刀初版序列化形缺外卷→修复后绿）
- @container 裸 not 前缀（parse_one_condition token 级臂仅
  `not style(...)`）维持原域不动＝后续刀候选

**container-self 语义兑现**（function-container-self 翻 P）——#target
自身是 CQ 容器但 @container 组不得对查询元素自身求值：container_chain
由 compute_styles_recursive「push 自身样式后、子递归前」维护 → 求值时
链顶恒为最近**祖先**容器（#container 100px）→ `(width = 100px)` 真 →
B（非 C 50px 自查询形）。判据注释逐字核对语义正确。

**指纹**：`css_wire.rs` `KC5R124_FN_COND_FACE`（ZW-KC5R124-FN-COND-FACE
#[used] 防 DCE）＋CARRY-K123 线承。

## 钉测（净增 9）

style-system tests/function_conditionals.rs 新模块 9 钉：@supports 真臂
覆盖顶层／假臂整跳守成／源序 last-wins／组内 locals 覆盖／@media 视口
真臂／@container 静态几何真臂／**嵌套 @media（not 形）**／**嵌套
@container（not 形＋= 等值）**。engine part168_k115_fn_cssom_pins N1 锚
推进：cssRules 0→1（k124 前组解析破坏连外层 result 一并吞＝0；k124 后
组进 body 树外层 result 保留＝1；WPT 期望 2 子规则物化域仍 held-F 理由
改写注记——非掩盖回归）。probe_body_tmp.rs 调试探针删除＋撤册。

## 门禁（两代）

- **v2（not 补域后）**：css-parser lib **3415P/0F**＋style-system lib
  **2700P/0F**（2692 基＋k124 净增 8）＋engine lib **4310P/3F**＋clippy
  0＋fmt 0。
- 首轮 4F＝cqmq compound_or（本刀 Not 序列化形缺外卷）＋3 既有对端红；
  修复后 3F 收敛。fmt 首判 103 字符超 100 系预判错——rustfmt.toml
  max_width=120，cargo fmt 反向折叠（展开→单行）复检 0。
- **3 红 face 零涉硬证据**：pathspec stash 全 face 13 文件→HEAD 态
  s330/expando/p99s 3/3 红复现→pop 复位（stash@{0} 3582ddbf；pop 后
  status 复核 13M＋1 新增原样）＝HEAD 既有红；k123 账本「名单与 k122
  腿恒同——非本刀引入」。对端归因在案：s330 慢性 27+ 连红
  （html-semantics-compat s747）；p99s＝对端 k102 刀γ %槽 resolvable
  扩域新红首演（m2s747 证据）。

## 探针与电池（一代）

- **gen17 三代**：v1 e5e5730c（首版 face）→ v2 077e7f04（not 补域预
  fmt）→ **v3 432c797b1d0ab6c790e19062c864efee（fmt 后定档）**；指纹
  ZW-KC5R124-FN-COND-FACE=1＋CARRY-K123=1 三代恒 1（strings 直查）。
- 探针（v3 全语料单腿，锁键 zw-k124-battery.registry 同域）：43p/521k
  census {Pass 449/Fail 71/Timeout 1}；vs k123 b1 逐键差分 **F2P=18
  P2F=0 GONE=NEW=0**＝registry 18 键程序化生成
  （f2p-registry-r124.py）——function-conditionals 17（22 键页 held 清
  零）＋function-container-self 1。
- 电池双腿（锁键 zw-k124-battery.lock）：b0＝k123 归档 gen16
  `b1d40280cf829906bfef248eff560b4e` rc0=0 **b0 status ≡ k123-b1 逐键恒
  同 True**（pristine 复现 431）；b1＝gen17 v3 rc1=0 **b1≡probe 逐键恒
  同 True**（521 含 message；gen17 两独立轮全等）。census b0 {F 89/P
  431/T 1} → b1 {F 71/P 449/T 1}。

## 判官（judge-r124.py registry 驱动硬码）

G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0；G2 P2F=0（豁免空）held-F
11/11 持（shadow 3＋invalidation 3＋eval 残 2＋local-inherit 残 2＋m5
1）＋红线 4/4 持；G3 F2P observed=18 knife=18/18 ±0 集外 0；G4
non-exempt diff=0；NET 431→449 +18==18。**VERDICT PASS**。
（held-F 首轮 4 bad＝键名转写漂移——eval 2＋local-inherit 2 页路径写
成 mixins/ 实为 functions/ 子目录，键名以语料 JSON 实名修正后 0 bad；
修正前先查双臂实态 None＝不存在键而非行为翻转，未冒计入账。）

## verify（verify-r124.py 独立复核）

恒等式 P1−P0==F2P−P2F（+18==18−0）＋关系预测 (18,18,0) 逐位命中＋
presence 521==521＋pages 43==43＋status census 净＋Timeout 恒同＝
**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins **P=449**（gen17 v3 432c797b 确定性锚；下一轮电池 b0＝
gen17 v3）。挂账池更新：**conditionals 清零**（22 键 17 F2P＋5 守成
P）＝m5 mixin-conditionals 裸声明覆盖序移规则域刀候选＋eval 残 2
（revert-layer/revert-rule keyword left unresolved on result
descriptor——真级联回溯域）＋local-inherit 残 2（dashed-function-
standard-property inherited longhand＋local-if-substitution CSS-wide
keyword inherit——同页 typed longhand/shorthand 3 键同域簇候选）＋
invalidation 3＋shadow-dom 3＋attr 10F＋cycles 4＋@apply 嵌套块 FIXME
＋@container 裸 not 前缀（parse_one_condition token 臂）＋其余 carry：
from-import-media＋m2c/m8/m15＋gCS IACVT ''/absent 统一刀＋animations
24＋narrowing＋DC-3 对账周账 12-01。
