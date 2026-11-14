# DC-2 · d18 批：Selection collapse/extend/addRange 三道规范校验门

日期：2026-11-14。前置：d17 批（[窗口数学重分类](2026-11-14-dc2-window-math-reclassification.md)）。
基线：CI run 43240558696 selection-face 产物失败聚类——collapse/extend 面
"should not update anchorNode" 5,558F + "should not update focusNode" 2,038F、
addRange 面 "should not add Range" 78F + addRange-08/40/44/48/52/56 六文件
各 464F 全红（first addRange must do nothing expected 0 but got 1）。
本批一刀落 `crates/engine/src/js_dom_shim/part06.js` 主窗口 Selection 三方法。

## 修复内容（spec 依据 https://w3c.github.io/selection-api/）

共享判定 helper `_zwSelNodeInDoc`（`_zwNodeContains(document, node)` 上行，
selectAllChildren R3254-E2 同款语义）+ 三道门：

1. **addRange**（spec #dom-selection-addrange）：步骤 0 非 Range 参数 →
   TypeError（removeRange 同款先例）；步骤 1 range 两端点 root 不是本文档
   （detached/外文档）→ 完全 no-op；步骤 2 `rangeCount !== 0` → no-op
   （Chrome/Edge 行为入 spec——旧版整体替换 `_ranges` 两面全违反）。
2. **collapse**（spec #dom-selection-collapse）：步骤 4 `!document.contains(node)`
   → 完全 no-op。步骤序保持 spec：步骤 2 doctype → InvalidNodeTypeError、
   步骤 3 offset 越界 → IndexSizeError 已由既有 `setStart`（R42/R288）在
   `_ranges` 赋值前抛出（**抛错先于 no-op 分支**——语料 collapse.js 分支序
   doctype→offset→contains 与 spec 2→3→4 一致）。
3. **extend**（spec #dom-selection-extend）：步骤 1 in-doc 门 **先于**空
   selection InvalidStateError（detached + 空选区 = no-op 非抛错——extend.js
   首分支 + extend-exception.html 单案 only attached+empty 仍抛
   InvalidStateError，两面兼容）。doctype/offset 抛错由 setStart/setEnd
   在赋值前承担。

`setPosition` 为 collapse 别名自动获得同门。iframe 轻量 Selection
（part05）不在本批范围。

## 语料契约核对（修复前完成）

- `selection/collapse.js` testCollapse 分支序：doctype 抛 → offset 抛 →
  `!document.contains` 走 assertSelectionNoChange（六断言 + getRangeAt(0)
  身份）→ 有效路径 rangeCount/focus==anchor/addedRange 四边界不改写。
- `selection/extend.js` testExtend 分支序：contains **首**分支 → 空
  InvalidStateError → doctype → offset → 有效路径 anchor 不变（同 root）/
  focus=(node,offset)/`assert_not_equals(getRangeAt(0), originalRange)`
  （替换式新 range）。
- `selection/addRange.js`：testAddRange（非 selectable 端点 →
  testAddRangeDoesNothing 全 no-change 断言；selectable → identity 电池 +
  live-range 双向 mutation）+ 尾部 `if (selection.rangeCount > 0)
  testAddRangeDoesNothing(range2, "second")`——**对一切 range2**（含
  selectable）断言第二段 addRange 不改 selection = spec 步骤 2 直接语料面。
- addRange 步骤 2 的 P2F 面清点：engine 内部 2 处调用
  （part01d.js:2997 重绑 / part01f.js:911 契约恢复）均先 removeAllRanges；
  语料面 editing setup（implementation.js setSelectionForwards/
  movePreservingRanges/removeformat/plain-text-copy-paste/canvas
  selection-rects）全部 removeAllRanges 前置；selection 语料内
  Document-open.html（iframe 独立 Selection，本批不涉）、getRangeAt.html
  （仅 identity 断言）、removeAllRanges.html（前置清空）、dir-manual.html
  （manual 不自动跑）、addRange.tentative.html（fresh 空 selection）均无
  replace 语义依赖。

## 验证（同机同命令 before/after）

方法学：修复前二进制（02:12 release，含 20521a2ba 壳域选区臂标记、
不含 `_zwSelNodeInDoc`）跑基线，修复后重建（+1m26s 增量）同命令对拍；
`--json` 断言级 ledger（按测试名集合差）。

六文件基线→修复后（test-guard 4G/8G/1800s 包裹）：

| 文件 | pre P/F | post P/F | F2P | P2F | vanished |
|---|---|---|---|---|---|
| addRange-00 | 1392/232 | 1624/0 | 232 | 0 | 0 |
| addRange-08 | 0/464 | 232/0 | 232 | 0 | 232 |
| collapse-00 | 1485/1170 | 2655/0 | 1170 | 0 | 0 |
| collapse-30 | 2871/2262 | 5133/0 | 2262 | 0 | 0 |
| extend-00 | 989/1035 | 2024/0 | 1035 | 0 | 0 |
| extend-20 | 1161/1215 | 2376/0 | 1215 | 0 | 0 |

- 六文件合计 **F2P=6,146、P2F=0、new-fail=0**；除 addRange-08 外分母
  恒等。addRange-08 的 232 vanished =「second addRange」测试在修复后
  **不再创建**（harness `if (selection.rangeCount > 0)` 门——spec 步骤 1
  使 first addRange 正确 no-op，rangeCount 保持 0，分母收缩非回归；
  与 Chrome 行为同形）。
- 基线与 CI 聚类逐文件恒等（232/464/1170/2262/1035/1215），协议确定性
  证实。
- 常驻探针 `zz-probe-d18sel.html`（selection/ 目录，临时不入库）3 case：
  no-op 族（collapse/extend detached + addRange detached/foreign 完全
  不改 selection + 拒收 range 不被改写）、抛错序族（doctype
  InvalidNodeTypeError / 越界 IndexSizeError / detached+越界先抛后 no-op /
  extend detached+空选区不抛 / extend attached+空选区 InvalidStateError）、
  addRange 族（foreign 拒收 / 单 range 门保持 identity / live-range
  strong reference / collapse(null) 清空 / addRange(null) TypeError）。
  修复后 3/3 绿。

## 引擎 lib 钉测

`cargo test -p zero-engine --lib -- selection js_dom`（test-guard +
独立 CARGO_TARGET_DIR=/tmp/d13tgt）：**1583 passed / 7 failed**。7F 逐一
对上 wptfyi-css master.md 碰撞记录的 main 既有欠账定谳集
（img/svg settle + fetch/iframe-SW/response-stream/t8k-websocket 族，
clean HEAD be6703d58 冷编译同 7 全红）——与本批零交集，非本批引入。

## selection 全语料 ledger（收口）

CI 11-12 快照 JSON（run 43240558696 sel 产物）vs 本批 post 全跑
（test-guard 4G/8G/10800s，101 文件）：

- pre 35,470P/16,004F → post 47,147P/2,803F，**F2P=11,481、P2F=0、
  new-fail=0**，vanished=1,740（addRange second 测试不再创建，同六文件
  机理），净 -13,201F。
- 分布：addRange-00..56 十六窗（合计 +3,010）、collapse-00/15/30/45
  （+5,772）、extend-00/20/40（+2,340）、editing/other/restoration.html
  （+5，restoration 链走 selection API）。post 残余 2,803F 在
  exec-command/typing-around-link/insertparagraph tentative 族与
  modify 等非本批域。
- 口径注意：CI JSON 为 11-12 二进制产物，跨二进制对拍含 11-12→11-14
  期间其他批次效应；受控同机同命令六文件 before/after（F2P=6,146）为
  本批净账下界，全量 F2P 为上界包络。

## editing face 六文件 + 哨兵（收口）

face 六文件 prefix（HEAD 无门，stash 对拍）vs post ledger：**P2F=0、
F2P=0、new-fail=0，失败逐名恒等**——bold 9F、italic 3F、strike 4F、
underline 3F、sub/sup 0F，与 d17 终账全一致除 bold 一条：

- bold 较 d17 账 8F 多 1F：`[foo<span class=notbold>bar</span>baz]`
  sw=true compare innerHTML（?2001-3000 窗；expected 内层 span 含
  `style="font-weight:bold"`，got 内层为裸 span）。prefix 跑（无门）
  同红同形，**定谳 HEAD 既有欠账，非本批引入**；溯源指向 02:12 前落地
  批（20521a2ba 壳域选区臂 / 8add42713 ip M1 一带），后续批 bisect
  定谳。d17 账 bold 8F 勘误为 HEAD 基线 9F。
- 哨兵电池（FILTER=zz-probe 53 页）prefix vs post ledger：**P2F=0、
  new-fail=0**；d18sel 探针 3/3 绿。f6bwin（并行流新探针）两窗两跑均
  页级超时（prefix 0 案完成 / post 398 案完成，load 深度差非门面）；
  fncorpus 两窗 Timeout 恒在（d15 已知）。并行流持续向共享 wpt-data
  加页致组成漂移，早期 34=34 名集判据作废，本批起哨兵判读改 ledger
  集合判读（P2F/new-fail）。
- 隔离探针 zz-probe-d18boldnb（notbold 条目孤立复现）产物碎形，缺
  conformance runner 上下文，不可用于归因（归因改由 prefix 全窗跑
  承担）；文件临时不入库，与前例同。

## input-events 全语料对拍（收口）

CI 11-12 快照 vs 本批 post（23 文件）：**逐名恒等——210P/907F，
F2P=0、P2F=0、vanished=0、new-fail=0**。契约清点中标记的顺序 addRange
依赖点未产生任何位移（该面残余 907F 为既有红账，与本批无交集）；
跨二进制口径局限同上，但零位移本身亦证 11-12→11-14 该面无主链漂移。

## 账目

- 断言级：同机受控六文件 F2P=6,146 / P2F=0；selection 全语料对拍
  F2P=11,481 / P2F=0 / vanished=1,740（跨二进制包络口径）；face 与
  哨兵零回归。
- 估算面（CI 聚类口径）：selection 面 collapse/extend/addRange 三族
  残余 F ≈ 13.1k 全部落本批修复域；weekly 快照（DC-3）落账待下周跑。
- wpt.fyi 对账：待下周快照（selection 面 Pass 数预期大幅上移）。
