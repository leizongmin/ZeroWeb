# kc5r125 · 标准属性值域 `<dashed-function>` 替换刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 449（b0 实测）→ 458（+9）**＝judge VERDICT PASS＋
verify VERIFY-PASS＝预注册通过 P2F=0、GONE=NEW=0。净 +9＝
dashed-function-standard-property **4 键**（本刀主域 css-mixins-1 §3：
longhand／typed longhand／inherited longhand／typed shorthand）＋
function-attr **4 键**（「Return typed/untyped url() from function
(quoted)」——函数返回 url 值落地标准属性，域内收益）＋
function-invalidation **1 键**（「Appending a rule」——`width: --f()` 直
形同构）。43 页 521 键恒同、Timeout 恒同。

## 刀形

css-mixins-1 §3（`<dashed-function>` 标准属性值域替换）＝级联放行门＋
简写哨兵域＋不可解产物残 IACVT 臂三件刀：

**级联放行门（cascade.rs `is_cascade_value_valid`）**——提前放行名单补
`has_dashed_function_call(value)` 臂：替换函数同 var() 口径，级联面值域
文法不可判定（级联期无函数注册表/无元素语境），合法性待 4.755 站替换后
由 apply 消费裁决。断点实证：`width: --ten-px()` 原落 apply-on-dummy 被
拒 → 级联丢声明 → width=Auto（var() 间接形 `--w: --ten-px();
width: var(--w)` 已通＝探针双臂对照）。

**简写哨兵域（shorthand/mod.rs `expand_one` gate）**——pending 哨兵
gate 同构并入（kc5r88 random 臂先例）：简写值含 dashed call → 铸
`ZWSP_SENTINEL_PREFIX` 哨兵（族解析器拒函数 token，强展开丢整条——
padding 键探针实证）→ 4.6 `expand_pending_shorthands` 复燃幂等保形 →
4.755 站代入哨兵 raw → 4.85 二趟展开落 4 长属性。

**不可解产物残 IACVT 臂（lib.rs 4.755 站 Value 臂）**——非 `--` 消费属
性替换产物残调用形（`has_attr_word` attr 词／`has_dashed_function_call`
dashed 调用残）→ `fn_cycle_props` 级联移除＋gather 表移除＝消费声明
IACVT → 继承属性继承回落。定谳依据：guaranteed-invalid「保留原文」口径
系 custom property 值域宽松承载（值域内无害）；标准属性域 apply 拒＝槽
留默认非继承回落，与级联丢观测断裂（环 B 级联丢 → 继承回落
Url("parent") 实证）——本臂收口。css-values-5 §attr-security taint 拦截
全域（attr 产物 url 污染取 URL 属性域）挂账随函数内 attr 求值刀
（函数求值端 attr() 替换缺位＝产物原文回落，8 键 P2F 归因定谳）。

## 归因链（attr-tainted P2F 8→0 两轮修复）

- **v1 探针 P2F=8**（function-attr「Returned url() is attr-tainted」
  簇）：判据语义＝attr() 产物 url tainted → 拒用 → IACVT → 继承回落
  （父 `#parent { list-style-image: url(parent); }`）→ 与 expected 显式
  声明恒同。b0 的 Pass＝级联丢（raw apply false）→ 未声明 → 继承回落同
  观测（值语义正确到达，非空洞）。b1 放行后：函数求值端 attr() 不替换
  （产物原文）→ apply 拒 → 槽默认 None ≠ 继承回落 → Fail。
- **环 A/B/C 探针**（临时 probe_taint_tmp，收尾删）：环 B（级联丢
  `garbage-value()`）→ Url("parent") 继承 ✓；环 A（attr 产物形）→
  None；环 C（url(cat.png) 直活）→ Url("cat.png")。
- **修复一**（attr 词臂）→ P2F 8→1；残 1「initial」臂＝`--g()` 未注册
  嵌套残原文透传（custom 域口径）→ apply 拒槽默认。**修复二**（臂扩
  dashed call 残）→ P2F=0。

## 钉测（净增 9）

tests/function_standard_property.rs 新模块 9 钉：longhand 直形／typed
longhand／inherited longhand（color 落地＋子继承）／shorthand pending
展开（4 槽）／var() 间接形守成／未注册名 IACVT 初始（Auto）／attr 词产
物残 IACVT 继承回落／未注册嵌套残 IACVT 继承回落／源序 last-wins 跨简
写展开。临时探针 probe_stdlong_tmp／probe_taint_tmp 已删＋撤册。

## 门禁（v3 终态）

- css-parser lib **3415P/0F**；style-system lib **2709P/0F**（2700 基＋
  9 钉）；engine lib **4310P/3F**（既有对端红名单恒同：s330 慢性/
  expando/p99s 对端 k102 刀γ——k122/k123/k124 账本一致）；clippy 0；
  fmt 0。
- fmt 假绿一次＝管道吞退出码（`cargo fmt --check | tail && echo CLEAN`
  的 RC 是 tail 的）——改 `> file; RC=$?` 直取后收敛，diff 全在临时探
  针文件（收尾删）。

## gen 代际

- **v1** 687b9df6（首版 face；指纹数组 24B 编译错——串实长 23——修复）
- **v2** 08403f9c（attr 词臂）
- **v3** d683c5759f1a14ff0e5a23c706a8843d（臂扩 dashed call 残＋9 钉终
  态定档）；指纹 `ZW-KC5R125-STDLONG-FACE=1＋CARRY-K124=1`（strings 直
  查，三代恒 1）。

## 探针与电池（各一代）

- 探针（gen18 v3 全语料 css-mixins 单腿，锁键 zw-k125-battery.lock 同
  域）：43p/521k census {Pass 458/Fail 62/Timeout 1}；vs k124 b1 逐键差
  分 **F2P=9 P2F=0 GONE=NEW=0**＝registry 程序化生成
  （f2p-registry-r125.py）。
- 电池双腿（同锁）：b0＝k124 归档 gen17 v3
  `432c797b1d0ab6c790e19062c864efee` rc0=0 **b0 ≡ k124-b1 逐键恒同
  True**（status+message 全等，pristine 复现 449）；b1＝gen18 v3
  `d683c5759f1a14ff0e5a23c706a8843d` rc1=0 **b1 ≡ probe 逐键恒同 True**
  （521 含 message）。census b0 {P 449/F 71/T 1} → b1 {P 458/F 62/T 1}。

## 判官（judge-r125.py registry 驱动硬码）

G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0；G2 P2F=0（豁免空）held-F
10/10 持（shadow 3＋invalidation 3＋eval 残 2＋local-if-substitution
inherit 1＋m5 1——k124 11 键集删 inherited longhand 翻 P 出集）＋红线
4/4 持；G3 F2P observed=9 knife=9/9 ±0 集外 0；G4 non-exempt diff=0；
NET 449→458 +9==9。**VERDICT PASS**。
（归因核：F2P 第 9 键实名「Appending a rule」非「Prepending a
rule」——后者 Fail→Fail 恒 F 未动，held 列表行扫读易混，程序化重对质
定谳。）

## verify（verify-r125.py 独立复核）

恒等式 P1−P0==F2P−P2F（+9==9−0）＋关系预测 (9,9,0) 逐位命中＋presence
521==521＋pages 43==43＋status census 净＋Timeout 恒同＝**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins **P=458**（gen18 v3 d683c575 确定性锚；下一轮电池 b0＝
gen18 v3）。挂账池更新：**standard-property 簇清零**（4 F2P＋5 守成
P）＋attr-tainted 回落臂兑现（8 键 P2F 修复回归 P）＝函数内 attr() 求
值＋css-values-5 §attr-security taint 拦截全域（attr 产物 url 污染取
URL 属性域）候选刀＋function-attr 残 held 6（attr(type(...))/type(*)/
default parameter/local variable）＋invalidation 残 4（prepending/
deleting/then-deleting/mutation 形 3——CSSOM 域）＋eval 残 2（revert-
layer/revert-rule 真级联回溯域）＋local-if-substitution inherit 残 1
（if() style() 条件关键字局部解释域）＋shadow-dom 3＋cycles 4＋@apply
嵌套块 FIXME＋@container 裸 not 前缀＋m5 裸声明覆盖序＋其余 carry：
from-import-media＋m2c/m8/m15＋gCS IACVT ''/absent 统一刀＋animations
24＋narrowing＋DC-3 对账周账 12-01。
