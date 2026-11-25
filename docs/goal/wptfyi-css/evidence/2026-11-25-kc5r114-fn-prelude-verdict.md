# kc5r114 · @function 定义文法刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 216 → 268（+52）**＝judge VERDICT PASS＋verify VERIFY-PASS
（预注册硬码一次性通过，零修订）＝**P2F=0、GONE=NEW=0**。套件域账，引擎总账
P=6840 不动。

## 刀形

css-mixins-1 §2.1.1 `<function-parameter-list>` 严格文法：

- css-parser `parser/function_prelude.rs` 新模块——参数列表 token 级状态机
  （空段拒/尾逗号容忍）、类型分量白名单 15 名＋乘数 `#`/`+`＋transform-list
  禁乘数、type() 联合、裸 ident、default 类型核验、returns 文法。
- css-parser `parser/at_rules.rs` `consume_function_rule` 接线（字符串路径退役）。
- engine `js_dom_bridge/callbacks.rs` `__zw_parse_function_rule` 单源回调
  （整规则/仅 prelude 双调用形；非法返空串）。
- engine `js_dom_shim/part06.js` replaceSync `@function` 空体保留门（非法 →
  今日空体丢弃形恒同 0 规则；合法 → `_ruleFromText` 保形物化）＋insertRule
  写面 SyntaxError DOMException 臂（合法 fall-through 通用 type-1 保形）。
- css-parser lib.rs `ZW-KC5R114-FN-PRELUDE-FACE` `#[used]` 指纹。

调试期修复三笔（探针前定谳）：

1. `parse_type_component` 裸 Ident/`*` 分支消耗数漏置（`i` 未置 1 →
   `parse_bare_type_spec` consumed≠len 误拒——裸 ident 与 type() 内分量全拒根因）。
2. default 类型核验 Function 形早退（单 token 切片模式 `[Token::Function(_)]`
   不匹配含括号体的多 token 组件切片——`random(...)` 语料钉假拒根因）；改为
   Function 形组件级早退保守接受。
3. 乘数位 `#` tokenizer Error 形本地映射（裸 `#` tokenizer 产
   `Error("Unexpected '#'")`——`tokenizer_coverage2` 钉锁该形，全局面不动，
   仅乘数位认作 `#`）。

## 门禁

- css-parser：at_function_parse 11 钉全绿（WPT 全页镜像 48 valid/38 invalid
  表驱动＋k109 承 9 钉）＋crate 全套件零失败＋clippy --all-targets -D warnings
  零警＋fmt 净。
- engine：part167_k114_fn_prelude_cssom_pins 6 stage 全绿（构造表 valid 空体
  保留/invalid 不抛 0 规则/cssText 保形/insertRule SyntaxError/非 @function
  空体粒度不变/`@function-bar` 异名不误捕）＋lib 4294 过＋3F＝账面既有
  三连红（s330/p99s/expando——html-semantics-compat.md 在册，非本刀）＋
  clippy/fmt 净。

## 探针定谳

gen7 bin `751768370409762be37e382bf9b35e6c`（指纹
ZW-KC5R114-FN-PRELUDE-FACE=1＋k113 线承 ZW-KC5R113-MIXIN-APPLY-FACE=1）
全语料单腿（43 页 521 键）对 k113 b1 归档基线逐键差分：
**F2P=52、P2F=0、GONE=NEW=0**——at-function-parsing 48（全页主刀面）＋
dashed-function-eval 4（defaults 家族核心：Single/Multiple parameters with
defaults、Default referencing another (defaulted) parameter）。function-attr
attr() default 候选未翻（held-F 钉）。

## 电池双腿

- b0＝gen6 归档 bin `ff91c12852f51c000301b17beeb9750b`（k113 face）；rc0=0。
- b1＝gen7；指纹=1；rc1=0。
- **P0=216 基线复现双证**：b0 重跑 JSON 与 k113 收账 b1 归档**字节恒同**
  （md5 `4483c2b381c1` 两侧一致）；P1=268 净 +52。
- status census：b0 {Fail 304/Pass 216/Timeout 1} → b1 {Fail 252/Pass 268/
  Timeout 1}；Timeout 键双侧恒同。

## 判官（judge-r114.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 20/20 持；红线 14/14 持
  （cssText 3P＋attr-tainted 9P＋shadow/conditionals 锚 2P）。
- G3 F2P observed=52 knife=52/52 missing=0 out-of-registry=0。
- G4 non-exempt pages diff=0；NET +52==+52 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订）。

## verify（verify-r114.py 独立复核）

恒等式 P1−P0==F2P−P2F（+52==52−0）＋关系预测 (52,52,0) 逐位命中＋presence
521==521＋status 值域 census 净＋Timeout 恒同＝**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=268。k115 候选挂账池：CSSFunctionRule 接口面
（at-function-cssom 32F——instanceof/getters/CSSFunctionDescriptors/cssText
canonical 序列化，js bridge 域）＋@mixin 跨表/导入域＋mixin-invalidation 3＋
gCS IACVT 统一刀＋animations 24＋narrowing＋DC-3 对账周账 12-01。

脚本/NOTES/结果驻 `.acceptance/wptfyi-css/k114/`（gitignore 不入库；
battery-k114.sh/judge-r114.py/verify-r114.py/probe-b1-full/results-k114-b{0,1}）。
