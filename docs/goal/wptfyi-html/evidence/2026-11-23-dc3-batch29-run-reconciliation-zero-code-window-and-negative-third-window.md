# DC-3 · 批二十九：run 45124287002 收账——零码刀窗全带内（face +1＝track-cues 交替带回 Pass＋批二十八勘误）＋负向两形第三窗零回位＝码形主导证据加强＋dom/ranges 单窗尖峰证伪＋CE·SD 第五窗持位

2026-11-23。run 45124287002（06:28:05Z dispatch，~06:34Z 起
~2h10m 止 08:44:38Z，快照 coverage-2026-11-23 同日覆写＝08:39:57Z
生成，aggregate manifest 恒 063eb7cae2 本窗零漂移）对批二十八 run
45109059524 同分片逐键 A/B。布防事故如实记：/tmp/batch29/autodl.sh
等待循环 60×120s＝2h 上限小于 run 时长（~2h16m），08:28Z 误判
timeout-no-marker 退出；RUN_DONE 出现后重跑幂等成功（08:58:38Z
下载 167 artifacts）＝**布防脚本等待窗必须 ≥ run 时长上界 + 余量**
入课。分片 checkout 逐 job 实测：**55 分片+gap 全提取＝40×
28ad1121f＋14×ab551b0b4＋1×3fa4c2d76**（后两者 docs-only 领先，
码图同 28ad1121f）＝全窗码图同 28ad1121f。窗口码变：**零码刀**
（28ad1121f 前窗基线 cadb865cb..28ad1121f 10 commits 全 docs(goal)/
chore(wpt)；32ef8f66e 快照数据写回非码）＝本窗全部变化零码归因。

## 目标级账

html 面 109,425 → **109,426（+1）＝净 +38,330/基线 71,096，验收
≥152,096 余 42,670**。+1 唯一构成＝track-cues-sorted-before-dispatch
**Fail→Pass**（artifact 键级实证：b27 Fail／b28 Fail／b29 Pass）。

## 批二十八勘误（批二十六课同族复发）

批二十八证据「track-cues 零变化＝Pass 持位第六演兑现」**有误**：
批二十八 artifact 该键实为 **Fail（P=0）持位第二演**——收账时未下
键验 status，把批二十六预测当了实测。「单键判定必须下 artifact
验键」（批二十六组行反推课）复发实证；组行/face 构成本身无误（批
二十八 +1 唯一构成 dialog 回位不受影响）。交替带真形七演：
F(b23) P(b24) F(b25) P(b26) F(b27) **F(b28)** P(b29)——b27→b28
破一拍后回 P＝带内震荡续证，不逐轮追判定维持。

## 负向扫溢出第三窗＝零回位零加深（零码窗下）

- **test262 51,907 持位**、**csp 199 持位**：本窗为零码刀窗，若
  timing 形应现回位机会（同窗 dom/ranges −441 大幅震荡对照）——
  两形三窗（b27 负向窗＋b28/b29 两零码窗）零回位零加深＝**timing
  回位假说弱化、码形主导证据加强**：test262 13 键与 csp 各键均为
  P→F 翻转且 regs 恒＝确定性位移形（非 TO/注册抖动）。
  **③③b 081d40b4a（test262 Function 构造器域 collateral 主嫌）／
  ③③a+③⑤ sink 域（csp timer/eval 主嫌）** 维持，证据链已批二十七
  移交对端，对端修复读数以本通道为准。
- **CE·SD 第五窗持位**（property-reflection-idl-setters 225P/2,685F
  等五键族 ce_sd changed entries = 0）。
- **DP①② 第二演持位**：domparsing/wc/sanitizer 三组行零键变化＝
  批二十八 +49 组行读数稳定兑现（sanitizer-custom-elements-
  constructor 8P 持位确认）；tt 套件 1,482 持位（交互窗键族同）。

## 停摆形第六演＝批二十八带外上穿证伪

- **dom/ranges 42,677 → 42,236（−441，回撤带内 42,173~42,420）**：
  dataChange 1,179→923、replaceData 1,146→961——批二十八 +306 带
  外上穿为**单窗尖峰**（带扩大候选证伪），既有带维持。六窗序列
  b24 −350/+96 → b25 +120/−220 → b26 −247/+223 → b27 +198/+5 →
  b28 +306/−125 → b29 −441/−20＝交替震荡续（零码窗大震荡＝timing
  形再证）；dom/ 邻账归 wptfyi-dom 只读。
- **editing/run 95,233 → 95,213（−20 带内 95,130~95,353）**：
  multitest +20／forwarddelete −56（regs 491→416）／delete +16。

## flake 带三观察

- onblur checkable-active-onblur-with-click **3P/4F 三连持位**
  （b27→b28→b29）＝水位下修形强化（b26 5P→三连 3P）；
- dialog modal-in-visibility-hidden 3P 持位（b28 3→b29 3＝两连，
  回稳候选）；
- track-cues 见勘误节（F→P 回位 +1 入 face）。

## 观察清单全持位

webappapis 1,540/207 files/subs 2,540 逐字同；per-global 6 注册
3P/3F names 全同；dat 170 键 shapes 148TO/19F/3 逐字节同（write TO
恒定续持）；interestfor 键族逐字同；variant 三键 DISTINCT regs/P/
nameset 569d8f/d83306/a823af 全同；合成 Fail 扫描 **103=103 零漂移**
（新旧两侧逐套件同）；css 组行 +16＝镜像震荡带（color **+34 精确
镜像回位**批二十八 −34、animations −17、borders −5、writing-modes
+4＝带内对称震荡定谳强化）；IndexedDB −1（批二十八 +1 timing 单键
镜像回撤）；web-animations +1。

## 快照闭合

组差合计 **−444**＝zw_passes 1,929,014 → 1,928,570 逐项闭合
（html +1、dom −441、editing −20、css +16、IndexedDB −1、
web-animations +1）；覆盖文件 74,189 恒；全过文件 63,640 恒
（track-cues F→P 文件级 +1 未现＝max(浏览器, ZW) 文件修正口径差
如实记，本地/CI 两侧同值）；分母恒 2,279,168／html 160,190；
本地旧 cache 复现 face 行 109,426 同值双证（本地全量分母 2,278,580
旧 cache 微漂另报不入账）。环比护栏未触发（face 正向、组差回落
属带内震荡）。

[逐文件 JSON](2026-11-23-dc3-batch29-run-reconciliation-zero-code-window-and-negative-third-window.json) ·
[批二十八证据](2026-11-23-dc3-batch28-run-reconciliation-dp12-verbatim-and-stall-band-breakout.md)
