# DC-3 · 批十八：scripting-1 模块脚本 .any.js window 形语料＋通道接线（准备轮）

2026-11-22。范围＝语料拉取闭包＋runner 扫描域接线＋本地基线测量；
coverage run **待批十七 run 45008050513 收账后派发**（concurrency
cancel-in-progress 在飞取消风险）。

## 通道决策（工作区边界三连）

1. script 25（fetch-wpt-html-semantics）属 **html-semantics-compat 活跃流**
  （GOAL=html-semantics-compat），本流不得触碰——scripting-1 语料补档只能
  走 59 脚本（本流 syntax-dom 分片通道）。
2. html-semantics 套件虽扫 scripting-1 根（HTML_SEMANTICS_TEST_SUBDIRS 第 7
  扫描根），但 .any.js 源是 opt-in 形态、25 脚本不设 GOAL_PULL_ANY_JS——
  html-sem 分片无此语料，本轮经 59 脚本＋html-syntax 套件接入**无双计**。
3. `.html` 形不拉：scripting-1 .html 已由 25 脚本带入 html-sem 分片计分，
  59 脚本若 fetch_dir_html 整目录则聚合金重复计账（批六防重同款）——
  显式 `fetch_raw` 单文件拉取，只拉 .any.js＋支撑闭包。

## 接线面（34 fetch_raw ＋ 5 扫描叶）

- runner：`HTML_SYNTAX_CORPUS_SUBDIRS` 加收 5 平铺叶（json-module /
  module/dynamic-import / module/dynamic-import/microtasks /
  module/top-level-await / text-module）。批十八块注释记账防重与挂账逻辑。
- 子树 27 源中 **14 源 window 形**接入（META: global= 含 window 或无 META 行
  缺省全全局——top-level-await 先例）；**12 源 worker-only 不拉**（microtasks
  7、import-meta 3、dynamic-import/microtasks 另 2——META 门不生成 window 键、
  分母亦无 .any.html，worker 通道域挂账 worker-lane 批次整批议）。
- 支撑闭包 20 文件显式拉取：json-module 5 json、dynamic-import/microtasks
  ticker+empty-module、top-level-await sibling 6 js、text-module bom 3 txt
  ＋file 四形。`/common/utils.js` 借道 58 脚本（同分片各自自足）；
  serve-custom-response.py／module-fetch-tester.py 走 py-route（site.rs
  s267/s346 臂已实现）不入盘（批十二先例）；serve-js-then-text.py／
  serve-text-then-js.py **无 py-route 臂未拉**——text-module/repeated-imports
  MIME 缓存面预期 Fail 记账，行为批议。
- 验证：`bash -n` 通过；脚本实跑 exit=0（幂等跳过＋bom txt ×3 新拉落盘），
  闭包 20 文件全在盘 python 逐路径核对。

## 本地基线（test-guard 包裹，FILTER=scripting-1，release 1m22s＋run ~2min）

14 键注册 **254 子测试＝234 Pass / 20 Fail**；chrome 分母 14 键
**249/260**。逐键对照（P/F 本地 · chrome P/S）：

| 键 | 本地 P/F | chrome P/S | 差异注记 |
|---|---|---|---|
| json-module/charset-bom | 2/1 | 3/3 | BOM strip 失败（编码刀位） |
| json-module/invalid-content-type | 100/0 | 106/106 | 本地注册 −6＝30s 案预算尾掉（CI 同预算） |
| json-module/non-object | 5/0 | 5/5 | 兑现 |
| json-module/repeated-imports | 4/4 | 8/8 | MIME/module-map 缓存族 |
| dynamic-import/blob-url | 1/5 | 6/6 | blob: URL 模块图未支持 |
| dynamic-import/concurrent-failed-imports | 0/2 | 2/2 | chrome 全过面（module-map fetch 缓存语义） |
| dynamic-import/microtasks/basic | 2/1 | 2/3 | microtask drain 语义 |
| dynamic-import/microtasks/with-import-assertions | 1/0 | 1/1 | 兑现 |
| dynamic-import/repeated-imports | 3/1 | 4/4 | MIME mismatch 族 |
| top-level-await/sibling-imports-not-blocked | 2/0 | 2/2 | 兑现（缺省全局门首例） |
| text-module/charset-bom | 2/1 | 3/3 | BOM strip 失败 |
| text-module/ignore-content-type | 106/0 | 96/106 | 本地超 chrome（?ct= 全忽略语义） |
| text-module/ignore-extension | 4/0 | 4/4 | 兑现 |
| text-module/repeated-imports | 2/5 | 7/7 | serve-*-then-*.py 无臂族 |

Fail 结构（20）＝blob: URL 模块图 5＋MIME/module-map 缓存族 10＋BOM strip 2
＋microtask drain 1＋其余 2——首跑基线记账（契约：Fail 多为预期不作为 land
门禁），blob-url 与 module-map 缓存族列行为刀位候补。

本地跑注记：FILTER=scripting-1 同时卷入同目录 56 .html 键（25 脚本语料本地
在盘、html-semantics-compat 流计分域）——本地测量噪声；CI syntax-dom 分片
无此语料，不双计。本地 P 面含该噪声（414 Pass 总），批十八面以上表 14 键
为准。

## CI 预期（run 待派）

syntax-dom 分片 html/semantics 组 **+~234P**（chrome 天花板 249/260；
invalid-content-type 案预算尾掉与 text-module 超 chrome 面随 run 实证收口）。
run 派发前置条件：批十七 45008050513 收账（快照落 main）后执行。

[逐文件 JSON](2026-11-22-dc3-batch18-scripting1-module-anyjs-corpus.json) ·
[批十七证据](2026-11-22-dc3-batch17-webidlparser-alias-and-rerun.md) ·
[前置账](2026-11-22-dc3-goal-accounting-and-corpus-runway.md)
