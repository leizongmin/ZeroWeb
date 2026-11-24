# kc5r86 verdict——flip 端点偏移集合成员门＋选择器归侧单刀（F2P=5＝3 精确＋2 race 族／P2F=0）

日期：2026-11-24。face＝**js_anim_registry.rs 单文件刀**（+72/−11，md5
584a8833，'zw-flip-orderless-2026-11-24' `#[used] static [u8;28]` 指纹）：

- `flip_ok` 端点偏移门：位置式检查（`keyframes[0]` 须 0%）改**集合成员判定**
  （`offs.contains(&0.0) && offs.contains(&1.0)`，兼排 0/0 同侧退化）。
- `flip_pairs` 归侧：数组位（首帧→from/次帧→to）改**真按选择器**（0%→from、
  100%→to），修与自身 doc「按帧选择器归侧」的矛盾。
- query_core fallback 行为不动（`endpoint_hit && open_interval && !flip_adopted`
  原样）；normal 序条目行为零变化（id=7 auto→20 门形不变直证）。
- 常驻 env 门调试探针（ZW_BOUNDARY_DEBUG，house 先例 ZW_ANIM_VALIDATION_DEBUG 形）：
  REG 行（id/blocks/flip_ok/pairs 逐块 dump）＋Q 行（prop/ehit/ointerval/eased）。
- spec：css-animations-1 #keyframes（缺帧＝underlying）＋css-values-4 §8.5.6。

bins：b0＝r86-b0-runner **143dc725**（HEAD 9fdacffb7 基线＝r85 face＋对端 dom
刀⑦；strings 'zw-flip-orderless'=0/'kc5r85'=2）、b1＝r86-b1-runner **0b3e141e**
（'zw-flip-orderless'=1/'kc5r85'=2）。电池发射行 FACE-REG-RS 584a8833 与收账时
树面恒等；构建零警告。

## 根因（草案B 探针证伪→dump 定谳）

interpolation-testcommon.js neutral keyframe 不入数组→harness 发单 to 帧→shim
（part03.js `_zwWaapiSpecKeyframes`）**预读 underlying 材料化为真实 0% 帧追加在
to 帧之后**（数组序反置；r83「{} neutralKeyframe 材料化先于守卫③」先例新面）→
flip_ok 位置式检查恒拒＝neutral 族 flip 通道全灭；同时 flip_pairs 数组位归侧
对反置数组把 from/to 值互换（潜在错配，本次一并修正）。eased 通道本身精确
（dump：raw="20" ehit=true eased=1.0 prog=0.5 直读；引擎单元钉
apply_timing_function(0.5, StepStart)==1.0）。缺陷仅在端点采纳判定：eased==1.0
精确终点落地被 fallback 当「离散 echo」拒→回落静态 underlying echo→at(1) 行
漏 to。草案B（single_block len==1 门）死臂证伪：本族材料化后恒 2 实块。
草案A（from 侧合成进 flip_eased_adopts）布防轮已否决（构造性 P2F 波，
interpolable at∈(0.5,1) 误采纳）。

## 探针门（发射前，双页双臂）

probe-r86.sh（锁 /tmp/zw-kc5r86-battery.lock，CSS_CORPUS_ONLY_SUBSTR 双页）：
b0 形两页 at(1) WAAPI 行 F→b1 形两行 P＋中程行全 P（eased 精确性实证位），
全行 diff 恰＝2 目标行翻面零漂移＝**PASS**（首跑 FAIL＝草案B 死臂，dump 修正
后复跑过门）。指纹课（r82 先例复发）：首版 `#[allow(dead_code)] &str` marker
被 release DCE 剥离（strings=0，interim bin 69a328f4）→`#[used] [u8;28]` 形
过门。

## 电池（六腿全 RC=0，19:52:17–20:16:36）

systemd transient（MemoryMax=24G/RuntimeMaxSec=7200）＋flock 独立键＋PRE-
BATTERY-QUIET census（proc cwd 精确匹配）＋JSON_DIR 独占写者。逐 corpus：

| corpus | b0 | b1 | Δ |
|---|---|---|---|
| css-multicol | P=1408/F=205（1613 键） | P=1410/F=203 | F2P=2 精确 |
| css-values | P=6680/F=3200（9882 键） | 恒等 | SET-IDENTICAL |
| css-transitions | P=1141/F=345（1505 页键） | P=1144/F=345（1508） | F2P=3 |

F2P=5＝multicol 2 精确（column-count neutral→[20]、column-rule-color
neutral→[rgb(70, 170, 70)] 均 at(1) WAAPI）＋transitions 1 精确（z-index
neutral→[5] at(1) WAAPI）＋transitions 2 race 族（properties-value-003，
report-only）。**P2F=0 全语料硬零**。values 软带 [0,112] 落零——calc-size/
interpolate-size 族 to 原串 canonicalize 失配回落 echo 持红（预注册可能向兑现），
零 churn。watch 挂账 z-index [auto]→[10]@1.0 持红未翻（键内他红行＝指纹非
完美类）。

## judge/verify（as-run FAIL 不回填→修订门 PASS＋独立双算 VERIFY-PASS）

- as-run judge（judge-r86-asrun.out md5 93ececfb 留证）：**FAIL:G3-css-transitions**
  ——F2P=3≠1，extra 2 键全在 properties-value-003 页。
- **race 族四证据归档**：①同 bin 3× 复判——column-count b0 F/F/P（**基线可达**，
  刀无关性直证）＋b1 P/P/P；border-radius b0 F/F/F＋b1 F/P/F 双侧不稳；②r80
  预注册本页＝race 采样族（无关刀 b1 bin 同页 3×F→P 先例）；③r85 boundary-
  sampling 同页族 '/ values' 单姊妹翻转签名（同页 4 radius 姊妹＋/events 孪生
  通道全持红）；④r80 G6 race 条款先例。修订 judge/verify 加 RACE_FAMILY 条款
  （仅豁免 F2P 精确计数，**P2F 硬门不豁免**）→修订门 **VERDICT PASS**。
- verify 独立双算（元组集合代数，不共享 judge 辅助）**VERIFY-PASS**：tuple
  13000/13003，恒等式 P1−P0==F2P−P2F 三 corpus 全等（multicol +2、values 0、
  transitions +3），预注册三键元组直证命中，hard-P2F=0。
- G6 report-only：event-dispatch NEW×3＋animate-with-color-mix GONE×1（r82/r83
  presence 对先例）。

## 残面移交

①inline 面 `_zwMStyleProxy` 同族缺陷（element inline 通道，需 corpus 读者预
注册）；②column-wrap-interpolation.html 补页（守卫③ regex 六关键字不含
wrap/nowrap，r85 承继）；④transitions timing 边界族条款预注册化；⑤transitions
corpus neutral 族其余键逐键勘形（auto→[10] 等）；⑥properties-value-003
'/ events' 慢性红带续观（r85 勘误在案）。

净指标：wpt.fyi CSS 全语料 +3（multicol +2、transitions +1 精确；race 族 2 不
计入净值主张）。证据链：NOTES-kc5r86.md §零-§五（.acceptance/wptfyi-css/kc5r86/，
本地）；judge-r86.out/verify-r86.out/电池 log /tmp/zw-kc5r86-battery.log（本地）。
