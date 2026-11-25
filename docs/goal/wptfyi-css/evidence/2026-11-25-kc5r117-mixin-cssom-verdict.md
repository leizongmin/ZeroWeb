# kc5r117 · @mixin/@apply CSSOM 物化刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 303 → 329（+26）**＝judge VERDICT PASS＋verify
VERIFY-PASS（预注册硬码一次性通过）＝**P2F=0、GONE=NEW=0**。套件域账，
引擎总账不动（页外域零扩散）。

## 刀形

css-mixins-1 §cssom 双刀面（A 层序接线＋B CSSOM 物化）＝26F 靶面全清；
invalidation 3F mutation 生效形 held-F 不吃：

**A 层序接线（mixin-layers 1F）**——css-parser：

- `parser/at_rules.rs`：`register_layer_name`（层序表首现 push——声明序
  定级联层序，css-cascade-5 §cascading-utilities）＋`enter_layer_collect`
  （rank 分配）＋@layer 声明/块形收集上下文（pass-1 collect 层内 `@mixin`
  携 rank；`collect_layer` 暂切/恢复；匿名层不登记不追踪——unlayered 与
  既有行为逐位同）。
- `parser/mixins.rs`：`MixinDef.layer` 字段＋`UNLAYERED=u32::MAX`；
  `register_ordered` 层序优先裁决——更强层（rank 更大）不被弱层收集覆盖
  （层序先于文档序）；同层晚序胜（k116 聚合源序保持）；unlayered 最强
  （mixin-layers e3 形）。

**B1 条目形/展开形双载体（css-mixins-1 §cssom 求值域/枚举域分离）**：

- `ast.rs`：`StyleRule.apply_expanded` 字段——`@apply` 块编译出条目形
  （原文回读载体）与展开形（求值域声明）两条并存；展开形仅参与级联求值，
  CSSOM 枚举面（wire/shim）按此跳过。
- `parser/mod.rs`：`compile_parsed_style_rule` 条目形 push
  （`apply_declaration_entry`——property 固定 `"@apply"`；语句形 value=
  prelude；块形 value=`{prelude} { decls }` canonical、空体 `{ }` 单空格）；
  `compile_nested_at_rules` Apply 臂展开产物全量置位（含嵌套子规则）。
- style-system：`cascade.rs` 求值端按属性名跳过条目（钉
  k117_apply_entry_skipped_in_cascade——@apply 双条目＋color 真声明 →
  result 仅 color 槽）；`StyleRule` 字段新增 219 处测试构造点程序化补齐
  （python 平衡扫描器 215＋修正版重打 71＋matcher 4；fmt 门修复）。
- engine `css_wire.rs`：`style_rules_text` Style 臂展开形跳过（枚举面
  单条形）＋`@apply` 条目 cssText 多行形（mixin-cssom 逐字节 template
  literal：`#foo {\n  color: red;\n  @apply --m2 { color: green; }\n}`）；
  无 @apply 条目维持单行形（对端域 styleExists 单行钉不破）。

**B2 单源重建器（engine `js_dom_bridge/mixin_css.rs` 新文件）**：

- `parse_header` 名双形（`Token::Ident` / `Token::Function`——css-syntax-3
  function token 语义：`--m1(` 的名自带 `(` 语义）＋参数列表双入口
  （depth 基准统一）；`canonical_param` 三臂——源形 `type()` 壳剥壳、
  canonical 无壳 `<...>` 直传（wire 读面二次解析形）、default `:` 后
  原文保形。
- `scan_block` 裸声明聚合首插保源序（invalidation 前置
  `cssRules.length==2` 形）＋@layer 任何深度 extent 丢弃＋空 bare 不生成
  聚合 child＋at 块行集组装。
- canonical cssText 形（WPT 逐字节）：空参列表剥 ()、@result 内部行与
  头行平级、嵌套块单行、@contents 带分号、参数剥壳
  `--arg <length>: 1em`、空 contents 块 `{ }` 单空格。

**B3 shim 物化五件套（`js_dom_shim/part06.js`）**：

- `_ruleFromText` @apply 顶层门（SyntaxError DOMException——嵌套 FIXME
  挂）＋@mixin 臂（`__zw_parse_mixin_rule` 空返抛 SyntaxError）。
- CSSMixinRule/CSSResultRule 接口全局壳＋`_makeMixinRule`/
  `_makeMixinChildren`/`_makeMixinChild`（kind style/result/at/contents
  四类；style 形 `_makeRuleStyle` 从 cssText 解析声明＝child style 读面；
  祖先 cssText 冒泡重建 FIXME 挂＝invalidation held-F 域）。
- wire 分派 @mixin 臂（selectorText 标记形条目单源；失败跳过不炸）。
- replaceSync brace 深度感知切分（旧朴素 split('}') 把嵌套块切碎＝
  mixin-parsing 17F 硬门槛）＋尾 `}` 剥＋@mixin 空体保留门
  （`@mixin --m { }` 合法 cssRules=[]）。

**钉测 14**：css-parser mixin_layer 3（层序裁决）＋style-system cascade 1
＋engine css_wire k117_wire_tests 3（wire 条目/@apply 多行分形/参数剥壳
＋空 contents）＋part169 两钉（cssom pins C1-C6＋wire pins——后者含
s2 参数形 wire 读面通道钉）。

## 门禁（两代腿＋既有红对号）

- 门禁腿 1（第一代）四处红全归因修复：css-parser 1F
  （`test_invalid_invocations_skip_expansion`——k113 求值域断言
  `no_decls` 与 k117 条目形定义域冲突：条目形恒物化（解析层无法预判
  展开合法性，CSSOM 原文回读不过滤），断言适配为过滤 `@apply` 条目后
  核展开声明面空）＋clippy 1（`let...else` → `?`）＋fmt 多文件。
- 门禁腿 2（第二代）：css-parser lib **3410P/0F**＋style-system lib
  **2642P/0F**＋clippy **0**＋fmt **0**；engine lib **4305P/3F**＝既有红
  对号入座（s330/expando/p99s 三键——对端 ZeroWeb-3 s760 收账文档
  `fail=3＝s330 四十连红＋p99s 十四红＋expando 基线形③三承` 实证：
  无 k117 face 的对端树上已红＝跨流既有红，非本刀引入；名单恒同经
  独立重跑核实）。
- 钉测腿 r117e：**9P 全绿**（css-parser 3＋style-system 1＋engine 5，
  RC 全 0）。

## 探针定谳（两代）

第一代 gen10（md5 `057d8ebb28136716b5bee54385f0de7f`，指纹=1＋线承=1）
全语料单腿（43 页 521 键）对 k116 b1 终态逐键差分：**F2P=25、P2F=0、
GONE=NEW=0**——cssom 面 7 翻 **1 held**：
`serialization of @mixin with parameters` got `--arg, --other-arg` vs
expected `--arg <length>: 1em, --other-arg`。

**canonical 再解析漏网形**（本轮独立发现修复）：wire 读面 shim 对
canonical cssText 调 `__zw_parse_mixin_rule` 二次解析——输入 type 注解
已是剥壳后直传文本 `<length>`（Delim 三 token，无 type() 壳），不进
源形 Function 臂 → params 退化名单。钉测盲区＝part169 全走 replaceSync
写面通道（源形直通），wire 读面通道（HTML 静态 style 标签 →
document.styleSheets）零钉覆盖。修复＝`canonical_param` 加 `Delim('<')`
直传臂＋wire 钉补 s2 参数形（paramsText＋cssText canonical 幂等断言）。

第二代 gen10（md5 `b774dbf377f14007c00fa64a41b878b4`）：**F2P=26、
P2F=0、GONE=NEW=0**＝预期面精确全翻（layers 1＋cssom 8＋parsing 17）
＝registry 26 键程序化生成（f2p-registry-r117.py，防转写漂移）。
invalidation 3 键 held-F（mutation 生效形）。

## 电池双腿

- b0＝k116 归档 bin `65c7675cb7d51dcfba8c21c2fdac381b`
  （ZW-KC5R116-MX-DOCSHEET-FACE）；rc0=0。
- b1＝gen10；指纹 ZW-KC5R117-MX-CSSOM-FACE=1＋CARRY-K116=1；rc1=0。
- **P0=303 pristine 双证**：b0 重跑 JSON 与 k116 收账 b1 **字节恒同**
  （cmp 零差，md5 `e865b5f06cdcbef5` 两侧一致）；b1 JSON 与探针产出
  **字节恒同**（md5 `2ce7aa6962fd06d1`，cmp 零差）；P1=329 净 +26。
- status census：b0 {Fail 217/Pass 303/Timeout 1} → b1 {Fail 191/Pass
  329/Timeout 1}；43 页 521 cases 双侧恒同；Timeout 键恒同。

## 判官（judge-r117.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 6/6 持
  （shadow DOM 域 3 键 k116 carry＋invalidation 3 键本刀 held——两臂
  皆 Fail）；红线 4/4 持（apply-top-level/mixin-basic/shadow 隔离/
  mixin-cycle）。
- G3 F2P observed=26 knife=26/26 missing=0 out-of-registry=0（registry
  内插）。
- G4 non-exempt pages diff=0；NET +26==+26 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订）。

## verify（verify-r117.py 独立复核）

恒等式 P1−P0==F2P−P2F（+26==26−0）＋关系预测 (26,26,0) 逐位命中＋
presence 521==521＋pages 43==43＋status 值域 census 净＋Timeout 恒同＝
**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=329。挂账池更新：mixin-invalidation 3（held-F 本刀
——style 写面冒泡重建 FIXME(compat) 挂 part06.js）＋mixin-shadow-dom 3
（carry，shadow 级联面缺位）＋@apply 嵌套块 FIXME（_ruleFromText 顶层
门域）＋其余 carry：from-import-media 条件假形（对端 s16x 协调）＋
m2c/m8/m15＋gCS IACVT ''/absent 统一刀＋animations 24＋narrowing＋
DC-3 对账周账 12-01。

脚本/结果驻 `.acceptance/wptfyi-css/k117/`（gitignore 不入库；
battery-k117.sh/probe-r117.sh/judge-r117.py/verify-r117.py/
f2p-registry-r117.py/probe-b1-full/results-k117-b{0,1}/bins；/tmp/zw-k117/
同灶）。
