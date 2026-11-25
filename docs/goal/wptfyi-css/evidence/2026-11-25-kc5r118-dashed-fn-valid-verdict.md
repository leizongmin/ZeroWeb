# kc5r118 · `<dashed-function>` 调用实参文法写面刀 verdict（2026-11-25）

## 结果

css-mixins suite **P 329 → 363（+34）**＝judge VERDICT PASS＋verify
VERIFY-PASS（预注册硬码一次性通过）＝**P2F=0、GONE=NEW=0**。套件域账，
引擎总账不动（页外域零扩散）。

## 刀形

css-mixins-1 §function-rule 调用实参文法＝34F 靶面全清（parsing 28＋
named-arg 6）＝写面拒收单源三通道接线，合法形路径零改（held-P 21＋9
全保＝纯拒收补齐零 P2F 面）：

**验证单源（`values/dashed_function.rs` 新文件）**——
`dashed_function_value_valid`：Tokenizer 流式递归下降（url_modifier_
declaration_valid 同型）；无调用词元恒真＝零漂移。文法规则全矩阵：
- 实参顶层逗号分隔；坏词元恒拒（`;`/`!`/`}`/`]`——declaration-value
  保留词元无文法名分）
- 空实参拒（`--fn(,)`/`--fn(a,,b)`——逗号段须含非空白 component）
- 块实参独占且块体非空（`123 {}`/`{}1` 拒；`{}`/`{ }` 拒；
  `{,}`/`{{}}`/`{[]}` 过——闭合词元不自证内容）
- 保留形拒（实参首 dashed-ident 且跨空白次 `:`——csswg-drafts#11749
  未来命名实参语法保留；`--fn(50px --myident:)` 过＝非首豁免）
- 嵌套 dashed 调用任意深度递归同文法；非 dashed 函数透明走行
  （其内部顶层逗号不切段）
- 词法期 Error 词元（bad-url）顶层拒；未闭合调用拒

**三通道接线**：

1. 样式表门：`parser/mod.rs` consume_declaration 值收集后——标准属性
   （非 `--` 前缀）坏调用整声明丢弃（None→skip_malformed_declaration
   恢复，css-syntax「bad declaration」同语义）；custom property 豁免
   （值是任意 token 流，guaranteed-invalid 只在 var() 替换期判定）。
2. 宿主 pre-arm：style-system `apply.rs` validate_decl_value——URLS 臂
   **前**属性无关预筛（`!prop.starts_with("--")` 豁免同口径）。
3. shim hub 臂：part06 `_zwNormalizeDeclValue` 头部——`--name(` 正则
   预筛（漏网转义名形由样式表门兜底）→宿主 `__zw_validate_decl_value`
   委托（'false'→null 拒）；`!important` 剥尾再验（font 臂同款正则）；
   元素 style（part03 T53）与规则样式 setter 两通道经 hub 自然覆盖。

**指纹**：`css_wire.rs` `KC5R118_DFN_VALID_FACE`（ZW-KC5R118-DFN-VALID-
FACE，#[used] 防 DCE）。

## 钉测（6）

css-parser `tests/dashed_fn_validate.rs` 5（WPT 双页全矩阵 64 形逐形
钉＋无调用恒真反例组＋嵌套递归/未闭合组＋样式表门丢弃/恢复/保形存储
＋custom property 豁免）＋style-system t54 模块 1（validate_decl_value
pre-arm 判定矩阵：10 拒形＋4 过形＋豁免/零漂移对照）＋engine part170
1（写面端到端五向：元素 style 矩阵/cssText 坏形丢弃同串恢复/样式表
replaceSync 通道/零漂移对照/!important 尾——important 存 priority 位
getter 不回显钉）。

## 门禁（三代腿）

- 门禁腿 1（第一代）三处红全归因修复：css-parser 编译 4（测试类型
  注解 `&Vec` 误标 `Vec<_>`）＋style-system 1F（`--func({})` 误过——
  块收尾 RBrace 于 pre-match depth==1 自证块体非空，标记排除闭合词元
  RBrace/RParen 后净）＋指纹字节数失察（25 字节标 `[u8; 24]`）＋
  engine 钉 1F（important 回读期望错——inline style priority 位不回
  显，属性 getter 语义，钉测期望适配）。
- 门禁腿 2（第二代）：css-parser lib **3415P/0F**（3410 基＋钉 5）＋
  style-system lib **2643P/0F**（2642 基＋钉 1）＋clippy **0**＋fmt
  **0**；engine lib **4306P/3F**＝既有红精确对号（s330/expando/p99s
  三键，名单与 k117 门禁腿 2 恒同——非本刀引入）。
- 门禁腿 3（第三代）：钉测期望修复后 engine 4306P/3F 复核恒同＝全绿。

## 探针定谳（一代）

gen11（md5 `3155ad3da3a916da8c89f6e0edbdea3a`，指纹
ZW-KC5R118-DFN-VALID-FACE=1＋CARRY-K117=1）全语料单腿（43 页 521 键）
对 k117 b1 终态逐键差分：**F2P=34、P2F=0、GONE=NEW=0**＝预期面精确
全翻（parsing 28＋named-arg 6）＝registry 34 键程序化生成
（f2p-registry-r118.py，防转写漂移）。分面账：parsing F2P=28
held-P=21（合法形全保）；named-arg F2P=6 held-P=9。探针窗 329→363。

## 电池双腿

- b0＝k117 归档 bin `b774dbf377f14007c00fa64a41b878b4`
  （ZW-KC5R117-MX-CSSOM-FACE）；rc0=0。
- b1＝gen11；指纹=1＋CARRY=1；rc1=0。
- **P0=329 pristine 双证**：b0 重跑 JSON 与 k117 收账 b1 **逐键恒同**
  （程序化全量比对 True）；b1 JSON 与探针产出**逐键恒同**（同 gen11
  bin 两独立轮全等 True）；P1=363 净 +34。
- status census：b0 {Fail 191/Pass 329/Timeout 1} → b1 {Fail 157/Pass
  363/Timeout 1}；43 页 521 cases 双侧恒同；Timeout 键恒同。

## 判官（judge-r118.py 预注册硬码）

- G1 腿 RC 0/0；G5 keys 521==521 GONE=NEW=0。
- G2 P2F total=0 out-of-exemption=0（豁免集空）；held-F 6/6 持
  （shadow DOM 3＋invalidation 3，两臂皆 Fail）；红线 4/4 持
  （apply-top-level/mixin-basic/shadow 隔离/mixin-cycle）。
- G3 F2P observed=34 knife=34/34 missing=0 out-of-registry=0（registry
  内插）。
- G4 non-exempt pages diff=0；NET +34==+34 恒等式。
- **VERDICT PASS**（一次性通过，预注册零修订）。

## verify（verify-r118.py 独立复核）

恒等式 P1−P0==F2P−P2F（+34==34−0）＋关系预测 (34,34,0) 逐位命中＋
presence 521==521＋pages 43==43＋status 值域 census 净＋Timeout 恒同＝
**VERIFY-PASS**。

## 新基与挂账

新基 css-mixins P=363。刀域（dashed-function-parsing/named-arg）**0
held-F**＝本簇清池。挂账池更新：mixin-invalidation 3（mutation 生效形
held）＋mixin-shadow-dom 3（carry，shadow 级联面缺位）＋@apply 嵌套块
FIXME＋其余 carry：from-import-media 条件假形＋m2c/m8/m15＋gCS IACVT
''/absent 统一刀＋animations 24＋narrowing＋DC-3 对账周账 12-01。

脚本/结果驻 `.acceptance/wptfyi-css/k118/`（gitignore 不入库；battery-
k118.sh/probe-r118.sh/judge-r118.py/verify-r118.py/f2p-registry-r118.py/
bins；/tmp/zw-k118/ 同灶＋probe-b1-full/results-k118-b{0,1}）。
