# 刀 OM-19 判读收账 · typed-om keyword 面＋引擎受理族 7 靶（2026-11-22）

## 判读门（04:45:56 GATE PASS）

```
[css-typed-om]   keys b0=12499 b1=12499 P 12413->12439 F2P=7 P2F=0 new=0 gone=0
[css-values]     keys b0=1385  b1=1385  P 985->985    F2P=0 P2F=0 new=0 gone=0
[css-sizing]     keys b0=5700  b1=5700  P 3239->3239  F2P=0 P2F=0 new=0 gone=0
[cssom]          keys b0=1630  b1=1630  P 888->888    F2P=0 P2F=0 new=0 gone=0
[css-transforms] keys b0=5508  b1=5508  P 1910->1910  F2P=0 P2F=0 new=0 gone=0
[css-masking]    keys b0=6355  b1=6355  P 5520->5520  F2P=0 P2F=0 new=0 gone=0
[css-ui]         keys b0=1599  b1=1599  P 1000->1000  F2P=0 P2F=0 new=0 gone=0
TARGET-GAIN om19-keywordface F2P=7 (expect 7)
GRAND F2P=7 P2F=0 new=0 gone=0 excluded=21
OM19-GATE-PASS
```

- 恰集 7/7 全 F2P（预注册见 `.acceptance/wptfyi-css/NOTES-knife-om19-keywordface-claim.md`），
  无 TARGET-MISS、无 NON-TARGET-F2P、无 P2F-LIST。
- 六哨全 SET-IDENTICAL：css-values／css-sizing／cssom（OM-18 配方）＋
  css-transforms／css-masking／css-ui（本刀触面扩哨——perspective-origin
  none 受理／clip 文法／pointer-events 值域）。
- 账面自洽：css-typed-om P +26 ＝ 恰集 7 ＋ 排除域探针页翻转 19
  （zz-om19probe 6＋b 3＋c 2＋d 4＋zz-om17probe probe D 4；zz-* 出账域）。
  excluded=21＝19 F2P＋2 集合位移（探针页内）。

## 恰集 7 靶 → face 臂

| 页 | 靶 | 臂 |
|---|---|---|
| cssKeywordValue-invalid.any.js | 空 ctor TypeError | part10 ctor 空串门 |
| cssKeywordValue-value.any.js | 空 setter TypeError | part10 setter 空串门 |
| clip-path.html | 'clip' does not support inset(...) | apply.rs clip basic-shape 臂（OM-18 挂账承接） |
| color.html | currentcolor keyword | part10 declared 'color' currentcolor 保形（/-color$/ 尾缀域不含本名） |
| perspective-origin.html | none keyword | part10 declared none 保形＋apply_advanced none 受理 |
| pointer-events.html | bounding-box keyword | css-parser/style-system BoundingBox 值域（part10 map 已含，b0 写拒根因） |
| computed/get-position.html | Computed position sticky | part10 keyword map position += sticky |

## 电池与二进制锚

- 电池：`knife-om19-keywordface-battery.sh`，log
  `knife-om19-keywordface-battery-042612.log`（04:26:13 发射→04:45:56
  GATE-DONE，19m43s，14 腿全 rc=1 正常尾、无 LEG-FAIL）。
- patch md5 `636b3506f4588be8c6d72bd53d5c6b8f`（rebase b1fc0f8ea 后字节原样，
  preflight 快照双锚）。
- b0r md5 `7386dc4c1c…`／b1r md5 `f48abf6255…`（M2 distinct ✓）。
- FACE-MARKER 三 token b0r=0/b1r>0 全过：`刀 OM-19`（2）／
  `if (value === '')`（1）／`'relative','absolute','sticky'`（1）。
- 判读器：`knife-om19-keywordface-judge.py`（排除域＝zz-*＋css-values
  random flake，OM-18 同域）。

## 残面与挂账

- cssKeywordValue.tentative.html「IDL 转义序列化」1F 不入本刀
  （expected `"\\ Hello\\ World"` got `" Hello World"`——序列化转义层另刀）。
- perspective-origin none 计算值语义未建模（gCS 仍 initial '50% 50%'，
  FIXME 在 face 注记；语料主靶在 declared 面不受影响）。
- census 残池（om18-b1r＝HEAD 口径 css-typed-om 12499 键/367 页，全 Fail 86
  ＝zz 探针 46＋语料残池 40）：typed-om 余池 gCS typed 通道、IDL 转义、
  attached 族深水区维持前账。（勘误：本文件首版误承 OM-18 电池键数 12475。）

## 组合复锚（115adf54d，TEXT-INTERP face 并入后）

兄弟流刀 TEXT-INTERP（fa89cbb31，css+engine 域）在判读与推送窗内并入 main，
rebase 单冲突仅 master.md 累计行（来件 +13077 链缺 OM-18/OM-19、我方链缺
TEXT-INTERP，合并 tally +13103，双流条目全保留）；6 face 文件 hunk 域与其
不相交、净应用。组合态复锚：merged-runner md5 `e3195594…`（HEAD 115adf54d）
css-typed-om 单腿对 b1r JSON 全集对账＝恰集 7/7 Pass＋非排除域漂移 0。
产物 `results-knife-om19-keywordface/merged/css-typed-om.json`。
