# kc5r115 · @function CSSOM 结构化面刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 268 → 299（+31）**＝judge VERDICT PASS＋verify VERIFY-PASS
（预注册硬码一次性通过，零修订）＝**P2F=0、GONE=NEW=0**。套件域账，引擎总账
P=6840 不动（页外域零扩散）。

## 刀形

at-function-cssom.html 35 键全谱（32F 四失败形＋3P 红线）——CSSFunctionRule/
CSSFunctionDeclarations/CSSFunctionDescriptors 三接口物化＋canonical cssText
序列化域：

- css-parser `tokenizer.rs`：`escape_css_ident` pub（css-syntax-3
  #serialize-an-identifier——控制字符 codepoint escape `\9 ` 形/NUL→FFFD/
  非 ident code point 单字符转义空格 `\ ` 形/开头 digit 消歧）。
- css-parser `parser/function_prelude.rs`：`tokens_text_cssom`——Ident/
  Function/AtKeyword/Hash 名 escape 重铸；AST decoded 字符串级信息不可逆
  （`I\ dent` 与两 ident 文法串同形），escape 重编码下沉 token 消费期；
  ty/default 收集换用，at_rules.rs returns 收集接线；tokens_text 退役。
- engine `js_dom_bridge/callbacks.rs`：`__zw_parse_function_rule` k115 扩展
  ——type() 剥壳展开（单分量剥壳/联合形 `type(X | Y)` 保壳/**`type(*)`
  整段略（returns 侧）**）＋body 折叠（首现位置＋末值）＋canonical
  cssText/prelude/styleText 七字段 JSON。
- engine `js_dom_shim/part06.js`：`_makeFnRule` 结构化物化（cssRules 空体
  0/声明体恰 1 CSSFunctionDeclarations＋style CSSFunctionDescriptors——
  result getter/getPropertyValue/length/item/Symbol.iterator/cssText 读写
  setter 白名单重解析＋rule.cssText 重算）＋三接口全局壳＋回调缺席回落
  type-1 k114 形。
- css-parser lib.rs `ZW-KC5R115-FN-CSSOM-FACE` `#[used]` 指纹。

canonical 序列化 WPT 实证：联合形保 `type()` 包裹（`--ret-auto` 键
`returns type(<length> | auto)`——303 行单参保形断言，首跑 F2P=30 的根因，
`css_type_text` contains('|') 保壳修复后 31）；接口面（getParameters/
returnType）恒剥壳。

## 门禁

- css-parser：at_function_parse 12 钉（钉⑪ cssom canonical 序列化形新增
  ——TAB ident decoded/escape 分离＋空格 ident 收集期重编码）＋crate 全套件
  3402 过零失败＋clippy/fmt 净。
- engine：part168_k115_fn_cssom_pins 两钉（主钉 6 stage C1 cssText 13 case/
  C2 getParameters＋returnType/D1 Descriptors 读面/D2 setter mutation/
  E1 Empty cssRules 0＋instanceof 三壳/N1 嵌套 @supports 不抛＋回调缺席
  回落钉）——首跑 C2 挂（漏装 rule 级 getParameters/returnType）补后全绿；
  lib 4296 过＋3F＝账面既有三连红＋clippy 净。
- 双活轮仲裁：fedce64e/b39bb55e 并行同 face 碰撞——face 修复链归属
  fedce64e（联合形保壳），zombie 未接线 `type_css_text` dead code 收敛删除
  （clippy -D 实证）；gen8 二次构建换代（旧代 7fb3c257e844 探针 F2P=30
  作废＝接线缺位形），NOTES §六仲裁块双方账齐。

## 探针定谳

gen8 二代 bin `e42a869ce92e080f0ac77d4a1c120abb`（指纹
ZW-KC5R115-FN-CSSOM-FACE=1＋k114 线承 ZW-KC5R114-FN-PRELUDE-FACE=1）
全语料单腿（43 页 521 键）对 k114 b1 归档基线逐键差分：
**F2P=31、P2F=0、GONE=NEW=0、held-F 1**（`@supports in body`——replaceSync
`}` 切分器嵌套 brace 粒度域，k115 不扩切分器，预注册持）＝registry 31 键
程序化生成（f2p-registry.json，防转写漂移）。

## 电池双腿

- b0＝gen7 归档 bin `751768370409762be37e382bf9b35e6c`（k114 face）；rc0=0。
- b1＝gen8 二代；指纹=1＋线承=1；rc1=0。
- **P0=268 基线复现双证**：b0 重跑 JSON 与 k114 收账 b1 归档**字节恒同**
  （cmp 零差，md5 `a70cea3290fed452` 两侧一致）；b1 JSON 与探针产出 md5
  恒同（`2cd4d510c7a9f806`，双证）；P1=299 净 +31。
- status census：b0 {Fail 252/Pass 268/Timeout 1} → b1 {Fail 221/Pass 299/
  Timeout 1}；43 页 521 cases 双侧恒同；Timeout 键恒同。

## 判官（judge-r115.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 7/7 持；红线 14/14 持
  （cssText 3P＋attr-tainted 9P＋shadow/conditionals 锚 2P）。
- G3 F2P observed=31 knife=31/31 missing=0 out-of-registry=0（registry 内插）。
- G4 non-exempt pages diff=0；NET +31==+31 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订）。

## verify（verify-r115.py 独立复核）

恒等式 P1−P0==F2P−P2F（+31==31−0）＋关系预测 (31,31,0) 逐位命中＋presence
521==521＋pages 43==43＋status 值域 census 净＋Timeout 恒同＝**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=299。挂账池：@supports in body 切分器嵌套 brace 粒度刀
（held-F 1 键）＋@mixin 跨表/导入域（mixin-shadow-dom 3＋cross-stylesheet 1＋
from-import 2＋from-import-media 1）＋mixin-invalidation 3＋mixin-layers
层序 2＋m2c/m8/m15＋gCS IACVT ''/absent 统一刀＋animations 24＋narrowing＋
DC-3 对账周账 12-01。

脚本/NOTES/结果驻 `.acceptance/wptfyi-css/k115/`（gitignore 不入库；
battery-k115.sh/judge-r115.py/verify-r115.py/f2p-registry-r115.py/
probe-b1-full/results-k115-b{0,1}；/tmp/zw-k115/ 同灶）。
