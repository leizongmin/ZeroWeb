# kc5r128 verdict——function-layer 刀：@function 级联层 rank＋collect 递归 @layer（2026-11-26）

## 结论

- **刀域**（css-mixins-1 §2.1.1「The name of a @function rule is a tree-scoped
  name. If more than one @function exists for a given name, then the rule in
  the stronger cascade layer wins, and rules defined later win within the same
  layer」；层序 css-cascade-5 层序列表首现序）：
  1. **css-parser ast.rs**：`FunctionRule` 加 `pub layer: u32`（k117
     `MixinDef.layer` 同型；构造点唯一 at_rules.rs `consume_function_rule`）。
  2. **css-parser at_rules.rs**：构造携 `layer: self.collect_layer`（k117
     @mixin 同源 collect_layer 字段）＋`enter_layer_collect` 匿名层真 rank——
     兑现 k117 FIXME「匿名层按出现序入层序列表」：空名时以确定性合成哨兵名
     `\u{0}anon{mixin_order}:{anon_layer_seq}` 占层序表真位（NUL 前缀不可能
     与 ident 撞；确定性使命名使 pass-1/pass-2 同块同名判重命中＝rank 一致；
     `mixin_order` 分量跨表隔离；Parser 新字段 `anon_layer_seq` 块序计数）。
  3. **css-parser mod.rs**：`parse_stylesheet_with_mixins` 正式 parse Parser
     预填 `layer_seq = external.layer_seq.clone()`（pass-1 同源——@function/
     @mixin 构造携的 rank 才是全局层序表位次；单表路径 external 空＝零变）。
  4. **style-system custom_function.rs**：`collect_custom_functions` 递归
     `Rule::Layer` 块（新增 `collect_functions_in_rules`）＋rank 裁决——
     `existing.layer > fr.layer → keep`（强层不被弱层覆盖；同 rank 落
     insert＝后扫胜，扫描天然文档序）；@media 等条件块不递归（@property
     同型口径）；FIXME「cascade layer 维度本层未建模」兑现摘除。
  5. **engine css_wire.rs**：指纹 `KC5R128_FN_LAYER_FACE`（
     ZW-KC5R128-FNLAYER-FACE #[used] 23B）。k127 指纹源码存续＝gen21 bin
     CARRY-K127 strings=1。
- **电池代**：b1＝gen21 `r128-fnlayer-runner`（md5
  db8cb4d1be250751f05f3568d4431f19，FACE=1＋CARRY-K127=1 双指纹独立重核）；
  b0＝k127 归档 bin `r127-fnattr-fixpt-runner`（md5 1627cc92，
  ZW-KC5R127-FIXPT-FACE=1）洁净锚。
- **judge VERDICT PASS＋verify VERIFY-PASS**＝预注册通过：
  - G1 双腿 RC=0（b0 00:23:24 / b1 00:23:37 各 13s）。
  - G5 键 presence 521==521 GONE=0 NEW=0；pages 43==43。
  - G2 P2F=0（豁免集空）；held-F 显式 10 键两臂恒 Fail；红线 4 键恒 Pass。
  - G3 F2P observed=5 knife=5 out-of-registry=0 missing=0（registry 程序化
    生成防转写漂移）。
  - G4 SET-IDENTICAL 非豁免页 diff=0。
  - NET css-mixins **P 471 → 476 = +5**（b0 census {Pass 471/Fail 49/
    Timeout 1}＝k127 b1 终态净值 pristine 精确复现；b1 {Pass 476/Fail 44/
    Timeout 1}）；净恒等式 +5 == F2P − P2F == 5 − 0。
- **F2P=5 全落 function-layer.html**（判据 7 template 中 5 Fail 全翻，2
  守成 Pass 恒同）：Single function within anonymous layer／Last anonymous
  layer wins／Single named layer／Named layers／Named layers, reordered。
  刀前失败形态统一 `expected "Xpx" but got "--f()"`（原文残留＝注册表不含
  layer 内定义）。
- **判据页全谱锚定**：「Unlayered styles win」两序形（顶层在前/在后均胜）
  由 unlayered=u32::MAX 强于所有 rank 兑现——匿名层不能用 UNLAYERED 简化，
  必须占真实层序位；「Named layers, reordered」（`@layer theme, base;` 预
  声明使 base 首现序更强、文档序 base 块在前仍胜）由全局层序表位次裁决兑现。
- **P2F 面防核**：匿名层 rank 化对 @mixin 通道（k117 刀面）连带受益零扰——
  mixin-layers.html 判据 --m1..--m4 全为命名层竞争或匿名单例，无匿名同名
  竞争形；function-in-media.html（@media 内 @function 定义形）不在本套
  语料，条件块不递归口径保持。function-layer.html 哨兵登记不与命名层
  竞争（NUL 前缀），命名层判据形零漂移。
- **钉测**：css-parser `function_rule_layer_rank`（6 断言形：unlayered
  MAX／匿名单块 0／匿名两块 0/1／声明预置序 theme=0 base=1／命名块继后
  rank 1）＝at_function_parse 13P/0F；style-system
  `collect_layer_rank_arbitration`（9 断言形：匿名单层收集生效／匿名两块
  后块胜／顶层胜两序／命名层文档序／预声明序／@media 不递归锚／跨表
  unlayered 胜）＝lib 2723P/0F。
- **门禁**＝css-parser 3415P/0F＋style-system **2723P/0F**（2722 基＋1 新钉）
  ＋engine 4310P/3F＝既有对端红名单恒定（dialog_focus_s330/expando_r3046/
  p99s_inline_custom_property——k127 账面同名恒定；stash 归因定谳＝裸 HEAD
  恒红同三键非本刀引入）＋clippy RC=0＋fmt RC=0（首轮：fmt 折行 1＋失孤
  import `MixinDef` 1，修复复扫 0）。
- **坑录**：css-parser 钉测 helper `function_rules` 只滤顶层 rules——层内
  Function 在 `Rule::Layer` 内不在顶层，helper 补 `Rule::Layer` 递归后
  13/13 全绿；嵌套层形 `@layer a { @layer b { @function } }` 正式 parse
  不物化（k117 @mixin 同形态既有域，判据页无此形，不在本刀范围）。
- **挂账**：嵌套 @layer 内 @function/@mixin 物化缺位（内层块规则丢失——
  判据域外，FIXME 维持）；跨表层序比较在非聚合通道（表独立 rank）的
  layered-vs-layered 语义是近似（判据域单表，聚合通道已全局化）；held-F
  10 键 carry（shadow DOM 3／invalidation 3／eval revert 2／local-if
  inherit 1／mixin-conditionals m5 1）。
- 判据页 <https://drafts.csswg.org/css-mixins-1/#function-rule>（§2.1.1）。

## 证据链

- 探针：probe-b1-full census {Pass 476/Fail 44/Timeout 1}（43 页 521 键）；
  registry 差分 GONE=0 NEW=0 F2P=5 P2F=0（f2p-registry-r128.py）。
- 电池：battery-k128.log LEG b0 RC=0 / LEG b1 RC=0；B0=1627cc92（FACE=1）
  B1=db8cb4d1（FACE=1 CARRY-K127=1）。
- judge：G1..G5 全门 PASS；verify：(5,5,0) 关系预测逐位＋净恒等式。
- 工件：/tmp/zw-k128/（gates/build/probe/battery 四脚本＋logs＋registry）；
  归档 bin .acceptance/wptfyi-css/k128/bins/r128-fnlayer-runner；电池结果
  .acceptance/wptfyi-css/k128/results-k128-b{0,1}/。
