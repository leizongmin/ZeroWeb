# kc5r116 · @mixin 跨表/导入域聚合刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 299 → 303（+4）**＝judge VERDICT PASS＋verify VERIFY-PASS
（预注册硬码一次性通过）＝**P2F=0、GONE=NEW=0**。套件域账，引擎总账不动
（页外域零扩散）。

## 刀形

k113 registry per-parse 生命周期跨不过样式表边界（cross-stylesheet 实锤
red 保持）；@import 取回文本已落 `_zwStyleImportTexts` 但 blob 路径不消费
（from-import red 保持）。k116 文档级聚合半刀：

- css-parser `parser/mixins.rs`：`MixinDef` 加 `order: u32`（聚合源序）＋
  `#[derive(Clone)]`；`register_ordered` 晚序保序裁决（同名晚序胜＝
  css-mixins-1 document order 最后定义胜；同序保持 insert 后者胜）＝k113
  `register_ordered` 带序化通用化；新 `MixinDefs`（`collect(order, input)`
  ——`!contains("@mixin")` 零成本 bail；`std::mem::take` 预填外部聚合；
  共享 pass-1 收集循环）。
- css-parser `parser/mod.rs`：`Parser.mixin_order` 字段；`parse_stylesheet`
  薄壳化；新 `parse_stylesheet_with_mixins(input, external, self_order)`
  ——双程门 `contains("@apply")` 保持，pass-1 预填外部聚合＋本表收集带
  self_order；`run_mixin_collect_pass` 收集循环共享化（pub(super) 转交桥）。
- css-parser `lib.rs`：`pub use parser::MixinDefs` 导出桥＋
  `ZW-KC5R116-MX-DOCSHEET-FACE` `#[used]` 指纹。
- engine `pipeline/mod.rs`：`style_eligible_text` 拆出（type/media/disabled
  三门＋文本提取）；新 `parse_sheets_with_doc_mixins` 两轮式（表文本序列
  →`MixinDefs.collect` 文档序带序聚合→逐表携注册 parse；memo 沿用门＝
  i==0 且聚合空或无 `@apply` 且 pre_parsed 在）；`collect_stylesheets_with_
  parsed_external` blob 分支（外部 css order 0＋各 style 三门文本）与
  `collect_interleaved_external` interleaved 分支（@import 条目先于 base
  sheet 文档序交错）双双接入两轮式。
- css-parser 钉测 5（aggregate 晚序胜/cross-sheet 展开/本表早序被聚合晚序
  压制/空聚合恒等/imported 文本 collect 展开——@result 直声明形提升为独立
  同选择器规则实证）；engine 钉测 2（blob 双 style 块聚合展开/interleaved
  imported 文本聚合）。

## 门禁

- css-parser：lib 3407 过零失败（mixin 30 钉含 5 新钉）＋clippy/fmt 净。
- engine：lib 4298 过＋3F＝账面既有三连红（s330/expando/p99s，与本刀零
  关）＋k116 两钉过＋clippy 净。

## 探针定谳

gen9 bin `65c7675cb7d51dcfba8c21c2fdac381b`（指纹
ZW-KC5R116-MX-DOCSHEET-FACE=1＋k115 线承 ZW-KC5R115-FN-CSSOM-FACE=1）
全语料单腿（43 页 521 键）对 k115 b1 归档基线逐键差分：
**F2P=4、P2F=0、GONE=NEW=0**＝registry 4 键程序化生成
（f2p-registry-r116.py，防转写漂移）。

四 F2P 案归因：

1. `mixin-cross-stylesheet.html :: CSS Mixins: Mixin from another
   stylesheet`——主刀面（跨表 registry 聚合）。
2. `mixin-from-import.html :: CSS Mixins: Mixin from an imported
   stylesheet`——@import 取回文本（s164 entries 通道）入聚合＋base 表
   @apply 展开。
3. `mixin-from-import-with-media-queries.html :: CSS Mixins: @import with
   media queries`——media prelude 键在 corpus viewport 条件为真，聚合直接
   覆盖（原挂账 media 条件刀在 corpus 形下不需要）。
4. `mixin-conditionals.html :: @container wrapping locals with different
   container is not cached`——探针外红利案；电池 b0 侧 Fail→b1 Pass
   双腿仲裁复现＝真效应（非 flake；聚合改变样式表解析序连带 container
   缓存键）。

## 电池双腿

- b0＝gen8 归档 bin `e42a869ce92e080f0ac77d4a1c120abb`（k115 face）；rc0=0。
- b1＝gen9；指纹=1＋线承=1；rc1=0。
- **P0=299 基线复现双证**：b0 重跑 JSON 与 k115 收账 b1 归档**字节恒同**
  （cmp 零差，md5 `2cd4d510c7a9f806` 两侧一致）；b1 JSON 与探针产出
  **字节恒同**（md5 `e865b5f06cdcbef5`，cmp 零差，双证）；P1=303 净 +4。
- status census：b0 {Fail 221/Pass 299/Timeout 1} → b1 {Fail 217/Pass 303/
  Timeout 1}；43 页 521 cases 双侧恒同；Timeout 键恒同。

## 判官（judge-r116.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 3/3 持（shadow
  DOM 域 3 键两臂皆 Fail）；红线 4/4 持（apply-top-level/mixin-basic/
  shadow 隔离/mixin-cycle）。
- G3 F2P observed=4 knife=4/4 missing=0 out-of-registry=0（registry 内插）。
- G4 non-exempt pages diff=0；NET +4==+4 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订——红线键名三处以 k115 b1
  归档 JSON 实测名修正后才发射，属发射前防漂移步骤非判读后修订）。

## verify（verify-r116.py 独立复核）

恒等式 P1−P0==F2P−P2F（+4==4−0）＋关系预测 (4,4,0) 逐位命中＋presence
521==521＋pages 43==43＋status 值域 census 净＋Timeout 恒同＝**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=303。挂账池更新：mixin-shadow-dom 3（shadow 级联面缺位
gCS 空串——域宽另立刀）＋from-import-media media 条件假形（corpus 页形已
过，条件不匹配页形未测；`_zwStylesheetImports` 无条件取回——html-semantics
s16x 共享通道域碰头协调挂账）＋mixin-invalidation 3＋mixin-layers 1＋
m2c/m8/m15＋gCS IACVT ''/absent 统一刀＋animations 24＋narrowing＋DC-3
对账周账 12-01。

脚本/结果驻 `.acceptance/wptfyi-css/k116/`（gitignore 不入库；
battery-k116.sh/probe-k116.sh/judge-r116.py/verify-r116.py/
f2p-registry-r116.py/probe-b1-full/results-k116-b{0,1}；/tmp/zw-k116/
同灶）。
