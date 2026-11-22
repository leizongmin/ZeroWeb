# DC-3 · 批二十四：run 45049069340 收账——per-global +3 兑现＋对端编辑刀首窗（editing +96 / dom −350 溢出）

2026-11-23。run 45049069340（1e0ea06ec，coverage-only，11-22 17:22:19Z 起
19:17:58Z 止）与批二十二 run 45028417732（cf288ecd2）同分片逐键 A/B；快照
coverage-2026-11-22（1c9560c24 19:17:55Z 回写，aligned a3830cd8f7，分母
恒 2,278,575）组行环比。

## 目标级账

html 面 109,404 → **109,408（+4）＝净 +38,312/基线 71,096，验收
≥152,096 余 42,688**。

## 批二十三闭环：per-global +3 精确兑现

- `per-global.window.js` 1 条合成 Fail（`load support
  object-association.js`）→ 6 注册/3P/3F——批二十三语料修复在 CI 销账，
  隔离律第 8 例闭环。
- webappapis 面 1,537 → 1,540（207 files 持位）。
- 残 3F＝`window.clientInformation` 双导航别名＋`document.open` 形——
  navigator 窗口别名刀位挂账不变（~3S，js_dom_shim 域）。

## +1 带外回摆（不归因窗内刀）

html/semantics 组行 +1：`track-cues-sorted-before-dispatch.html`
Fail→Pass（事件时序竞争敏感键，无窗内刀可归因）——flake 候选，下轮观察
回摆。

## 观察清单判读（四项全持位）

1. **write/write_single 110 键 TO 形恒定**：170 键 shapes 逐字节同
   （148 TO/19 Fail/3 Fail+Pass）。判定成立——document.write 刀位落地前
   不再逐轮追。
2. **interestfor −3 持位**：键族逐字同。
3. **variant 三键过滤形零漂移**：三键 DISTINCT 且 nameset md5 与批二十
   二逐字同（548dd3/d83306/a823af）。
4. **合成 Fail 扫描**：os error 2 形 html 面首次清零；fetch-module 形
   2 键持位（blob-url/repeated-imports，module-map 域）＋import-maps
   3 键持位；subsetTestByKey 0。

## 邻账（对端两刀兑现）

- **edf8ce921 TT 刀**：tt 套件 31 → 943（**+912 精确兑现**）、san 套件
  39 → 60（**+21 精确**）。快照组行 trusted-types 仅 +887、sanitizer-api
  持 39——差异精确归因：25P/21P 落在 wpt.fyi aligned stable 清单外路径
  （`eval-csp-tt-default-policy.html` 22P／`eval-csp-tt-no-default-policy.html`
  3P／`sethtml-with-trustedtypes.tentative.html` 21P，resolve None 入
  zw_only 计数）。**分母契约正确执行，非管线异常**；刀兑现全额，快照账
  按并集清单口径记账。本地 `wptfyi_gap.py coverage` 重跑（同 cache＋同
  产物）三 totals 逐字复现。
- **4fb594667 编辑路径刀首入 weekly 窗**：editing 面 110,212 →
  110,308（**+96**：editing/run +94、whitespaces +10、other −4、
  plaintext-only −4）＝正向兑现落其本域。

## 对端碰撞：dom/ 面 −350

- `Range-mutations-dataChange.html` 1,152 → 947（−205）、
  `Range-mutations-replaceData.html` 1,146 → 1,001（−145）。
- 形态：P→None 未注册（注册数 1153→948／1146→1002）＋页级 Timeout——
  页面中途停摆杀死后续 async_test 注册。
- 消失子测试名恰为 nodeValue/textContent/data/replaceData 直写形＝
  4fb594667（part01f.js nodeValue 直写改直调 deleteData/insertData）域。
- 本地当前树复现：FILTER 两键实跑均页级 Timeout（dataChange 666P＋1TO／
  replaceData 1048P＋1TO）。
- 归 wptfyi-dom 账本＋碰撞记录；机制证据链移交对端刀作者定谳，不入本流
  修复面。

## 其余带内波动（非本流域）

csp/script-src +4、css-borders −12、web-animations +4（interfaces/timing
各 +2）、shadow-dom/declarative −1——无窗内刀域对应，watch 不深挖。

## 面差对平

快照组差合计 **+632**＝zw_passes 1,930,093 → 1,930,725，13 组行变动逐项
闭合（webappapis +3、html/semantics +1、dom/ranges −350、editing 四组
+96、tt +887、csp +4、css-borders −12、web-animations +4、shadow-dom/declarative
−1）。

[逐文件 JSON](2026-11-23-dc3-batch24-run-reconciliation-perglobal-editing-knife-and-dom-collision.json) ·
[TO 面根因探针](2026-11-23-dc3-batch24-speculative-to-face-rootcause-probes.md) ·
[批二十二证据](2026-11-23-dc3-batch22-run-reconciliation-three-fixes-and-rt-spillover.md)
