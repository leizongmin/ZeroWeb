# DC-2 · bold 数值权重边界 + cef 门 rc 面（d6 批，2026-11-12）

P2 主循环 ②：bold 值面清账——decl 批残余 84F 中的数值权重边界簇落地，
加六 format 命令 cef 门 rc=false 化。face 1（六 format 文件 + 12 哨兵
探针）断言级 **净增 +60 零回归零漂移**（F2P=60 / P2F=0 / newF=0，
bold 84F→42F）。

## 修复面

`part01d` 三臂 + `part06` 一门（录形 = data/bold.js 原文）：

- **bold 并值候选排除 strong**：`_zwVaMergeable` 对 bold 命令的 strong
  邻接恒不并入——vendored isSimpleModifiable 同判 strong/b 致误并
  （bold.js:270-282 两翼 fresh b、:285-287 仅并前 b、:298 css-false 仅
  并后 b 录形直译）；b 候选不受此门（:287/:300 并入 b）。
- **bold ADD 恒不切分数值壳祖先**：newValue 为激活值（bold/600-900）
  时 chain 走查跳过带 bold 指定值祖先——非激活数值壳（100-500/normal）
  与激活数值壳（600/800）均保壳内裹（:593-700 数值权重、:926-1005
  邻接 span 簇 `<span 500>fo[o</span><span 600>b]ar</span>` 两壳原形 +
  壳内载体录形）；toggle-off（normal）切分面不受此门。
- **bold toggle-off peel 幸存壳重物化**：css-false 且壳 decl ≡700 转 b
  剥 decl（:479-483 两翼 fresh b）；600/800/900 非等值壳与 css-true 恒
  保 span 形（:484/:489/:700 录形）；tag 面 b/strong 无 decl 不入。
- **cef 门 rc=false 化**（part01d apply 面 + part06 rc 面同判）：选区
  任一边界位置落 contenteditable=false 子树 → 整条命令 no-op 且
  execCommand 返 **false**（bold.js:103-167 录形 rc=[false] 零 DOM；
  implementation.js :579-582 enabled 前提——`<span cef>foo<span cet>
  ba[r</span>b]az</span>` host2 爬升取内层不走事件块）；editing host
  缘与中段穿越（边界均在可编辑区）不触发（对照 :103 录形 rc=true）。

## 方法

同树前后双二进制对跑（base = d6c-runner@HEAD 90cbcc889 无本刀、after =
d6d-runner@工作树，systemd scope 守卫构建，`ZW_SCRIPT_TIMEOUT_MS=
300000`），断言级（全子测试名）前后 diff。面 1 = 六 format 文件 +
12 探针（既有 10 + 新增 zz-probe-d6cef 3 case rc 判据、
zz-probe-d6diag 26 case 数值权重矩阵）；面 2 = multitest + misc 哨兵
同基线双跑。

rebase 复验：pull 入组合态两 commit（webview CSP 门，不触 engine
shim）后守卫重建 runner，bold + 双 d6 探针三文件重跑，3,076 断言行
与 pull 前 after 逐名恒等。

## 数字

面 1：base 11,870/185 → after **11,930/125**：

| 文件 | base P/F | after P/F | ΔP |
|---|---|---|---:|
| bold | 2964/84 | 3006/42 | +42 |
| italic/underline/strikethrough | 恒等（24/4/16F） | 恒等 | 0 |
| subscript/superscript | 恒等（13/13F） | 恒等 | 0 |
| zz-probe-d6cef | 1/2 | 3/0 | +2（rc 判据逐案翻转） |
| zz-probe-d6diag | 4/22 | 20/6 | +16 |
| tstyle/mtmini/p2diag 既有诊断 | 1F/1F/5F | 恒等 | 0 |

- 断言级：**F2P=60 / P2F=0 / newF=0**，断言名集逐名恒等（base/after
  only diff 均空），共享断言零漂移。

面 2（multitest + misc 哨兵）：base 10,458/24/1T → after 10,454/24/1T：

- misc 401/22 断言级恒等（423 行逐名 diff 空）。
- multitest F=2/T=1 恒名（fail 集 24 行逐名恒等）；P 差 -4 全落
  `?9001-last` 慢页门杀进度差（base 1,059 vs after 1,055，300s script
  门非确定性，承 carrier-fold 批口径）；`?9001-last` 之外断言名集恒等。

## 残余与下一步

- bold 残余 42F 三簇（d6diag 余红 6 case 即代表）：① notbold 类指定值
  读面（`<b><span class=notbold>` 族 qS before/after + innerHTML）；
  ② 跨 merge 邻接形（`<i><b>foo</b></i>[bar]<b>baz</b>` 族 +
  `foo[<b>b]ar</b>` partial 边界）；③ 数值壳边缘（`<span style=
  "font-weight: 700">foo[barbaz</span>}` 700/900 partial + b#purple
  toggle-off）。
- italic 24F / sub+sup 26F / u/s 20F 面未动（master ② 清单）。
