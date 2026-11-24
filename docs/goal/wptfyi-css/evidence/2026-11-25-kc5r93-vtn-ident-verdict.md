# kc5r93 verdict——view-transition-name 属性＋ident() 函数刀（9 触点＋property/ident 模块）（2026-11-25）

## 结论

- **刀面**（指纹 `ZW-KC5R93-VTN-FACE`，电池代 b1 bin md5 14794f737b1eefd5f08c3ff90580e1cc）：
  1. 新模块 `crates/style-system/src/property/ident.rs`（css-values-5 §ident
     通用资产）：`ident_call_outer_args`（平衡括号剥壳，string 内 `)` 不误剥）
     ＋`ident_declaration_value_valid`（外壳 `<declaration-value>`：顶层无逗号/
     无 `{}`，string 豁免）＋`eval_ident_concat`（`<ident-arg>+` 拼接：string
     去引号/custom-ident 原文/dashed-ident/`<integer>` 字面＋math 函数
     `eval_int_math_value` 复用）＋`serialize_custom_ident`（CSSOM serialize
     an identifier 数字开头 `\33 ` 形）＋`is_serialized_ident_form`（serialize
     自产形裸值反向通道，存原样）＋`is_custom_ident_token`（裸 ident 文法
     最小面）。
  2. computed_style.rs 字段 `pub view_transition_name: String`＋default "none"
    （css-view-transitions-1 §view-transition-name-prop，Inherited: no——
     inherit_property **不加臂**，与 math-depth 反向形）。
  3. registry.rs：initial_value `OptionalString(Some("none"))`＋
     known_properties 注册（CSS.supports 属性名面＋`all` 展开清单 +1）。
  4. inherit.rs apply_initial_value 复位臂。
  5. apply.rs 求值臂：none ∥ ident() 剥壳求值（失败→IACVT→gCS 'none'）∥
     裸 custom-ident＋serialize 转义形。
  6. apply.rs validate_decl_value 写面文法门（白名单 +1 臂 109）：none/裸
     ident/转义形/ident() 外壳三形收；ident-arg 外形无效判留给 computed 期
    （round-trip as specified——12 键依赖原文 store）。
  7. matcher/mod.rs vtn supports 臂（as-run 修订 ①——布防「不动」落空）。
  8. engine js_dom_bridge.rs decl_supported vtn 前置臂（as-run 修订 ②——
     尾门 apply-on-dummy computed 语义裁决使 `CSS.supports('vtn',
     'ident(5px)')` 假阴性；interpolate-size 先例同型）。
  9. engine js_dom_bridge/computed_style.rs gCS 串化臂＋css-parser
     random_integer_typed 清单 +1（§9.1.2 integer 位 floor——r92 教训
     预判兑现，本轮非「零新码」而 +1 清单行）＋lib.rs `#[used]` 指纹。
- **布防预测账 as-run 勘误（三缺口修复＋一真增益，如实记）**：
  | 预测 | as-run | 判 |
  |---|---|---|
  | matcher 不动（supports 语料零键） | computed 页 `CSS.supports` 前置门走 matcher→extended 表默认拒 | 修订 ① |
  | decl_supported 尾门通用通道即过 | 尾门＝known＋apply-on-dummy（computed 语义），外壳形恒拒 | 修订 ② |
  | round-trip 12 键假阳面钉住 | round-trip **链**二次赋值 serialized `\\33 myident` 裸值臂拒 | 修订 ③ |
  | substitution 页 9 键全持 F | calc 包装 sibling-index 形翻 P（真增益：ident 拼接×既有 tree-counting math 求值协同；裸形持 F 与布防一致） | 翻红即查结案 |
  | F2P=20 | **=21**（20＋真增益） | 修订 |
  | NET +20（6740） | **+21（6741）** | 修订 |

## 探针门（probe-impl 双臂三轮，b0 f8b7b3d03c44 指纹=0／b1 14794f73 指纹=1）

| 门 | b0 | b1 | 判 |
|---|---|---|---|
| E1 拼接形（ab/\33 x/--a） | Fail | Pass | 双翻 |
| E2 无效形 'none'×2＋写面拒 | Fail | Pass | 双翻 |
| E3 calc integer '--p3' | Fail | Pass | 双翻 |
| E4 裸 ident/dashed/round-trip 空格/none | Fail | Pass | 双翻 |
| E5 auto 异元素异 draw | Fail | Pass | 双翻 |
| E6 property-index-scoped 同值 | Fail | Pass | 双翻 |
| E7 serialize 转义形裸值（修订 ③ 钉面） | Fail | Pass | 双翻 |
| E8 supports 外壳文法（修订 ①② 钉面） | Fail | Pass | 双翻 |

探针页自身 bug 一轮：E2c 复用元素拒收后回读**前值**（拒收语义＝不覆盖）
——独立元素修正；DUMP 值回显定谳法同 r92。

## judge/verify 双算

- judge-r93.py **VERDICT PASS**：G1 双腿 RC=0/0（log 追加代际段取尾部代）；
  G5 9882/9882 GONE/NEW=0；G2 P2F=0＋sentinels=3（wrapper 余 3 键）持 F＋
  held-F 显式 8 键持；G3 F2P=21 刀内精确 out-of-registry=0 missing=0；
  G4 非豁免页 diff=0（豁免=random＋ident-function-parsing/computed＋
  knife 集）；NET 6720→6741 = +21。
- verify-r93.py **VERIFY-PASS**：presence 恒等＋恒等式 P1−P0=21 ==
  F2P−P2F=21−0＋基线 6720 逐位钉死。
- G4 豁免判据修订（布防预注册兑现）：ident-function 两页页名无 random 词
  但 33 键必变；substitution 页不豁免（12 恒形键仍判＝恒形硬证）。

## 三代 bin 考古（同日三 build）

13c1f6f7739b0cf513bfde0fff9506cc（探针二跑代——三缺口修复前，作废）→
14794f737b1eefd5f08c3ff90580e1cc（电池代）。VTN/MATHDEPTH/VARIND 三指纹
并存=1。b0=r92-mathdepth f8b7b3d03c449b8e2d997d6e4ce08f4d 恒（Pass=6720
与 r92 收账逐位一致）。build3 一度以 `cargo|tail` 管道丢失败 exit（E0603
mod ident private）——旧 bin 差点冒充新代，pipefail 形态重 build 纠正。

## 门禁

- clippy `-p zero-css-parser -p zero-style-system -p zero-engine
  --all-targets -- -D warnings`（systemd scope 24G）**RC=0**。
- fmt 归因（收账轮 `cargo fmt --check` 全扫定谳——布防「hunk 区与债区
  互斥」对既有债面成立、对本刀自有面落空，如实记）：本刀自有债 4 位点
  （ident.rs ×3＋apply.rs:1858 求值臂 if 合行）rustfmt pre-commit 清偿＝
  纯排版零语义（电池代 bin 14794f73 构建于清偿前，提交源与 bin 源仅排版差）；
  残面 10 位点（random.rs 8＋engine computed_style.rs:32 use 折行＋
  lib.rs:3375）全落既有债区、与刀面 hunk 互斥，维持挂账（fmt 清偿笔
  独立提交）。
- cascade-precheck 6 run 定谳（css-cascade all-prop-revert-layer 页）：
  b0 98/99/98、b1 95/98/101——两臂分布互叠（r92 同款 HashMap 迭代序
  flake），无 vtn 注册新回归；watch 项结案，工件在 cascade-precheck/。

## 残面移交

1. wrapper 族余 3 键刀序：font-palette ＞ animation-timeline ＞ color
   相对色。ident() 基座（写面外壳＋apply 求值＋serialize 转义）为通用
   css-values-5 资产，后续 ident() 消费属性刀复用。
2. substitution 页余 8 键（var()/attr()/taint/sibling-index 裸形域外持 F）
   ——attr()/var() 基座刀域。
3. css-cascade all-prop-revert-layer watch 项（known +1 波动）——
   cascade-precheck 6 run 定谳（见 .acceptance 工件）。
4. fmt 清偿笔（10 处既有债独立提交，r92 挂账延续）。
5. DC-3 对账周账（12-01 周窗口）。

## 证据链（.acceptance/wptfyi-css/kc5r93/ 本地）

NOTES-kc5r93 §零-§五／probe-impl-r93.sh＋log（三轮双臂 RC=0/0）＋
probe-impl-{b0,b1}-css-values.json（八门 8F/8P）／battery-r93.sh＋log
（指纹 B1=1 三代并存、B0=0、PROBE-PAGE-CLEAN=0、双腿 RC=0/0）／
results-k93-{b0,b1}/css-values.json 满量 257 页/9882 case（b0=6720 r92
收账逐位）／judge-r93.py VERDICT PASS／verify-r93.py VERIFY-PASS／
cascade-precheck.sh＋log／bins/r93-vtn-runner（14794f73）＋b0-r92-full.md5
＋b1-r93-full.md5；代账 kc5r89/bins/GENERATIONS.txt r93-vtn 行。
