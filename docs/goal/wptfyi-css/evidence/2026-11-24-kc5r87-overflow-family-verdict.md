# kc5r87 verdict——overflow 读面家族反查＋写面展开名单双刀（F2P=19＝16 精确＋3 validity 增益／P2F=0）

日期：2026-11-24。face＝**css_style_declaration.rs 单文件双刀**（+58 行，md5
b77665dd，`#[used] static` 'zw-overflow-family-2026-11-24' 指纹）：

- **读面家族反查臂**：`gap_rule_shorthand_reconstruct_from_style` 分派链加
  overflow 独立臂（font/transition/mask/animation-range/corner 同型）＋新 fn
  `overflow_family_value`——longhand 写（raw 双 longhand 声明）→简写 getter
  折叠重组（x==y→x else "x y"，css-overflow-3 serialization 形）；简写写（raw
  'overflow: x y' 声明）→longhand getter 拆槽；css-wide keyword 单 token 双槽
  同值透传；双源缺 → ''（清空臂）。臂早退在 gap_rule_reconstruct 通用双槽
  全等门之前（rule-width/style/color 失配 None 是 css-gaps 语料钉形，零变化）。
- **写面展开名单臂**：`gap_rule_shorthand_longhands` 加 "overflow" →
  ["overflow-x","overflow-y"]——简写写经 `__zw_style_shorthand_expand` 任意名
  直达展开落双槽＝声明组 longhand 化零残留（Chrome 声明块语义）。
- spec：css-overflow-3 §propdef-overflow（双值语法＋serialization 折叠形）。

bins：b0＝r86-b0-runner **143dc725**（r86 基线复用，'zw-overflow-family'=0）、
b1＝r87-b1-runner **bceff8b5**（双刀终版；'zw-overflow-family'=1；首版单臂代
际 dbc508b3 探针 FAIL 留档）。电池发射行 FACE-RS b77665dd 与收账树面恒等；
两代构建零警告。

## 机制与实施轮证伪（布防单臂→双刀）

受害页 css-overflow/overflow-shorthand-001.html（16 键 `overflow: X Y works`
全对组合）。布防预注册读面单臂刀；首版探针 FAIL（b1 仍 16F）——三分最小探针
页（回调注册面／回调直调面／CSSOM 面全 Pass）定谳链路完好，dump 失败消息
`expected "auto" but got "auto auto"` 指向 **④ 简写写原文残留**：布防推理
「原文落库直读已通」只对 x≠y 键成立（expected=原串），x==y 键 expected=折叠
单值而原串 "auto auto" 直读红；且下一轮 longhand 写不覆盖残留简写声明（非
Chrome 声明块语义）。第二刀（写面展开落双槽零残留）合成闭环→探针门复跑
PASS（b0 16F→b1 16P，零中程漂移）。

## 探针门与哨兵

probe-r87.sh（锁 /tmp/zw-kc5r87-battery.lock）四腿：overflow-shorthand-001
b0 16F→b1 16P；random-item-nested 哨兵双侧 2F2P 恒形（'expands in a shorthand'
＋'nested inside random-item' 双红＝margin/random 深水面未触证明；布防哨兵形
1F3P 系笔误，已按 r86 实账修）。

## 电池（双腿全 RC=0，20:43:06–20:46:49）

systemd transient（MemoryMax=24G/RuntimeMaxSec=7200）＋flock 独立键＋PRE-
BATTERY-QUIET census＋JSON_DIR 独占写者。css-overflow 218 页 1122 键：
**b0 P=312/F=808 → b1 P=331/F=789＝+19 Pass**。P2F=0 硬零；GONE/NEW=0
（预注册 scrollWidthHeight-negative-margin-002 test 名漂移担忧落空——该页
不在 corpus testharness 记账集，勘误入账）。

## judge/verify（as-run FAIL 不回填→修订门 PASS＋独立双算 VERIFY-PASS）

- as-run judge（judge-r87-asrun.out md5 f10b1b03 留证）：**FAIL:G3**——F2P=
  19≠16，extra 3 键 overflow-valid.html 'should set the property value'。
- **EXTRA-GAIN 条款归档**（非 race 族）：①同 bin 3× 复判 overflow-valid 全 P
  零 Fail（稳定构造性）；②归因直证＝写面展开臂同通道增益（展开落库后
  setProperty＋serialization canonical 形对）；③预注册残面普查漏项如实记
  （受害形状判据按互读筛，validity 页族 'should set' 形未入普查）。修订门
  **VERDICT PASS**。
- verify 独立双算（元组集合代数）**VERIFY-PASS**：F2P=19 P2F=0，恒等式
  P1−P0==F2P−P2F（19==19），pred-16 直证命中。

净指标：wpt.fyi CSS 全语料 +19（css-overflow）。证据链：NOTES-kc5r87.md §零-
§六（.acceptance/wptfyi-css/kc5r87/，本地）；judge-r87.out/verify-r87.out/
电池 log /tmp/zw-kc5r87-battery.log（本地）。

## 残面移交

②column-wrap-interpolation.html 补页；④transitions timing 边界族条款预注册
化；⑤transitions neutral 族余键勘形；⑥properties-value-003 '/ events' 慢性
红带续观；⑦style-system margin expand random() 族 pending 函数值放行
（random-item-nested 2F，gCS 通道，is_margin_rect_value 拒函数 token，R2873
var pending 形同构）；⑧inline 面双槽属性同族刀面（overscroll-behavior 等，
corpus 普查零受害页暂不认领）。
