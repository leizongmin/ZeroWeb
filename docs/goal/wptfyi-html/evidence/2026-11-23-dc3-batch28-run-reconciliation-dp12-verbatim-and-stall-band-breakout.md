# DC-3 · 批二十八：run 45109059524 收账——DP①② +49 组行兑现（domparsing +17／CE +24 双精确＋sanitizer +8 清单外正溢出）＋负向两形第二窗零回位持位＋dom/ranges 停摆带带外上穿 +306＋face +1（dialog 回位）

2026-11-23。run 45109059524（03:58:35Z dispatch，04:07:58Z 起
~1h47m 止 05:55:21Z，快照 coverage-2026-11-23 同日覆写＝05:50:59Z
生成，aggregate manifest 恒 063eb7cae2 无漂移）对批二十七 run
45091003913 同分片逐键 A/B。分片 checkout 逐 job 实测（批二十七课）：
**55 覆盖分片+gap 全提取＝39×314099bde＋14×9b2c8416f＋1×cadb865cb**
（后两者 docs-only 领先，码内容同 314099bde）＋1 job fallback 误抓
Cargo.lock cache key（b8a594ea5，不在对象库＝批二十七课复发实证，
对象存在性检查兜底）。窗口码变：**唯一码刀 DP①② 4c1816d14**
（domparsing .xhtml 入扫＋CustomElementRegistry 可构造，对端自报语料
+41P）——4c1816d14..314099bde 区间全 docs/chore(wpt) 零码；7542fd567
快照数据写回非码。批二十七「DP①② 窗外归批二十八」兑现。

## 目标级账

html 面 109,424 → **109,425（+1）＝净 +38,329/基线 71,096，验收
≥152,096 余 42,671**。+1 唯一构成＝the-dialog-element/
modal-dialog-in-visibility-hidden 2→3（批二十七 P→F 回位＝新形
flake 交替带第二演：b26 3→b27 2→b28 3，续 watch）。

## 邻账兑现：DP①② 三面双向账（组行合计 +49 vs 自报 +41）

- **domparsing 组行 121→138（+17）＝DP① 自报 +17 逐字精确**。套件
  51→56 files、154→171P：五 .xhtml 新键全注册——insert_adjacent_html
  +12P/30、innerhtml-03 +3P/8、createContextualFragment +2P/4、
  innerhtml-01 0P/2、xml-serialization 0P/17，另 stream-html-custom-
  element-sanitizer regs 1→4（0P）。覆盖文件 +5、全过文件 +6 同源。
- **custom-elements 组行 2,843→2,867（+24）＝DP② 自报 +24 组行级
  精确**；套件 wc 11 键 +23（差 1＝组行/套件 regs 口径差，单窗不深
  挖）。10 registries 键全落 DP② 域：importNode +7P（regs 1→20）、
  registry-define-get-etc +7P、define +3P、multi-register +2P、
  createElement/createElementNS 各 +2P（regs 1→10）、Construct +1P；
  effective-global-registry regs 1→66／initialize regs 1→27 新注册
  0P；define-upgrade-criteria P 恒 1、regs 14→11（13F+1P →
  9F+1P+1TO＝3 子测试消失＋1 F→TO，对端自报 4F→页级 TO 形的 CI
  regs 口径差如实记）。
- **sanitizer-api 组行 281→289（+8）＝DP② CE 域清单外正溢出**：
  sanitizer-custom-elements-constructor 0→8P（regs 1→8）。对端自报
  该页得分净零与 CI +8 冲突＝环境/语料差如实记，不深挖。
- CE 域合计：组行 +24（清单内）＋san +8（清单外）＝+32；对端自报
  CE +24——差全在清单外溢出，两口径各自闭合（批二十四同款）。

## 负向扫溢出第二窗＝零回位零加深（判读窗顺延批二十九）

- **test262 51,907 持位**：13 键 Function 构造器域 P→F 全数持位，
  regs 恒——③③b collateral 嫌疑第二窗零回位零加深＝既非定时回位
  也非继续加深，单窗不定谳期满一窗，判读窗顺延（批二十九第三窗）。
- **csp 199 持位**：script-src-1_4_1/unsafe-eval/eval-scripts-set
  各键持位——timer/eval 签名嫌疑与页级 Timeout 并存第二窗持位，
  码 vs timing 分流同样顺延。

## CE·SD −2,772 第四窗持位

property-reflection-idl-setters 225P/2,685F、property-reflection、
DSD-attachment/basic、clonable、reactions/Node 五键族逐字同（ce_sd
changed entries = 0）。DP② registries 域不触 property-reflection
族（wc +24 全在 custom-elements/registries/ 内实证）；对端修复
读数通道不变。

## 停摆形第五演＝dom/ranges 带外上穿

- **dom/ranges 42,371 → 42,677（+306，旧带 42,173~42,420 上缘外穿
  +257）**：dataChange 983→1,179（破批二十五水位 1,052 创新高）、
  replaceData 1,036→1,146。窗内零码关联（DP①② 不触 CharacterData
  域）＝timing 形续证；五窗序列 b24 −350/+96 → b25 +120/−220 →
  b26 −247/+223 → b27 +198/+5 → b28 +306/−125＝交替震荡续，带
  扩大候选如实记（新读数带暂记 42,173~42,677，续窗确认）。
- **editing/run 95,358 → 95,233（−125 带内）**：multitest ?9001-last
  1,092→1,037（regs 全撤）、forwarddelete ?7001-last 371→406（regs
  446→491 全恢复）、delete ?7001-last 675→570（regs 756→617）。
- dom/ 邻账归 wptfyi-dom，本流只读。

## flake 带三观察

- onblur checkable-active-onblur-with-click **3P/4F（regs 7）＝批
  二十七 3P 水位保持第二演**＝水位下修候选强化（b26 5P→b27 3P→
  b28 3P）；
- dialog 回位见目标级账（交替带第二演）；
- track-cues 零变化＝**Pass 持位第六演兑现**（批二十六预测，仅记
  不追）。

## 观察清单全持位

webappapis 1,540/207 files/subs 2,540 逐字同；per-global 6 注册
3P/3F names 全同；dat 170 键 shapes 148TO/19F/3 逐字节同（write TO
恒定续持）；interestfor 键族逐字同；variant 三键 DISTINCT regs/P/
nameset 569d8f/d83306/a823af 全同（同值检测例行）；合成 Fail 扫描
**103=103 零漂移**（逐套件计数全同，新旧两侧皆扫）；tt 套件
**1,482 持位**（179 files 1482P/536F/64TO 逐字同批二十七＝③③b/
③⑤ 收敛后持位首演，交互窗 default-policy 族持位）；shadow-dom
组行 −1＝DSD-opt-in regs 119→116（批二十七 XHR document.open 新
注册 3 子测试回撤＝timing 带第二演）；css 组行 −30＝镜像震荡带
（color −34 regs 3753→3720、borders +5、animations +1、values +1、
writing-modes −3、transitions 页推进 regs 16→30 全 F、forms
checkmark 两键镜像互换）带内不深挖；IndexedDB +1（idbcursor_
iterating Timeout→Pass 单键 timing 带）。

## 快照闭合

组差合计 **+201**＝zw_passes 1,928,813 → 1,929,014 逐项闭合
（html +1、domparsing +17、CE +24、san +8、dom +306、editing
−125、css −30、shadow-dom −1、IndexedDB +1）；覆盖文件 74,184 →
74,189（+5＝domparsing xhtml 入扫）；全过文件 63,634 → 63,640。
**分母恒定**：CI manifest 恒 063eb7cae2（2,279,168；html 160,190
持位）＝本窗无漂移；本地旧 cache 复现 face 行 109,425 同值双证
（本地全量分母 2,278,580 为旧 cache 微漂，另报不入账）。环比护栏
未触发（本窗正向）。

[逐文件 JSON](2026-11-23-dc3-batch28-run-reconciliation-dp12-verbatim-and-stall-band-breakout.json) ·
[批二十七证据](2026-11-23-dc3-batch27-run-reconciliation-tt-dual-knife-verbatim-and-negative-spillover.md)
