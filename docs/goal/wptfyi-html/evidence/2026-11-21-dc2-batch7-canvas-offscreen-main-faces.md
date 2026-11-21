# DC-2 · 批七：canvas offscreen 主线程面整批接入（语料修复 + 扫描链扩展 + 记账）

2026-11-21。批七落地：offscreen 主线程 `.html` 面（~1,368 文件此前只作 `.worker.js`
变体在 worker 套件执行、`.html` 分母键从未执行）链入 `testharness-canvas` 主套件 +
offscreen 顶层链入口（此前死配置）+ worker 套件补 2 个顶层 `.worker.js` +
wide-gamut-canvas 语料修复。release runner 实测套件净 **+1,062 Pass**
（canvas 1,134 → 2,190；canvas-worker 1,044 → 1,050），共享键零回归（main 1,331
键 + worker 1,010 键逐键状态序列与批前账逐字一致）；对账口径（映射 summary-chrome
分母）净 **+1,046 Pass**（2,163 → 3,209），覆盖分母键 **+1,252**（2,209 → 3,461）。

## 变更（一文件 + 语料修复）

1. `tests/wpt-runner/src/testharness.rs`——
   - `run_canvas_cases` 扫描链加收 `CANVAS_OFFSCREEN_SUBDIRS`（18 目录，`.html`
     主线程面；`.worker.js` 仍归 canvas-worker 套件）+ `html/canvas/offscreen`
     顶层伪目录入口（`CANVAS_TOP_LEVEL_FILES` 的 offscreen 4 条目此前无扫描
     路径——死配置）；
   - worker 套件加收 `CANVAS_WORKER_TOP_LEVEL_FILES`（offscreen 顶层
     `.worker.js` 2 键，目录扫描不覆盖）；
   - worker 用例执行收敛为 `run_canvas_worker_case`（行为保持——主页面格式串
     逐字同源，仅去重两处内联）；
   - 单元断言 `canvas_scan_chains_cover_offscreen_faces`（顶层键直下形 +
     目录清单不相交防双计）。
2. 语料修复（gitignored 不入提交）：`offscreen/wide-gamut-canvas` 8 文件
   （4 main + 4 worker）——分母与 pin 315976933 均有此目录，本地 10-27 抓取时
   Contents API 列目录静默失败（脚本 404 容忍不报错）致目录空；raw 直拉补齐。
   CI 全新 checkout 走 `fetch_dir_html` 正常路径无需此修复。

## 首跑插曲：pin 缺失键回退

批七初版曾把 `element/2d.text-root-font-relative-units.html`（em/cap 相对字号面，
wpt.fyi 分母键 11 子测试）加进抓取清单与顶层白名单——raw 404 实证该文件上游
2026-11-01（ccd75f87062f）才新增，晚于语料 pin 315976933。按批六先例
（语料单 pin 不混版）回退该条目，11 子测试挂「pin 缺失」随周跑另报。

## 结果（release runner，test-guard 包裹，全量双套件 before/after）

| 面 | 变更前 | 变更后 |
|---|---:|---:|
| canvas 键 | 1,331 | 2,707（+1,376） |
| canvas Pass | 1,134 | 2,190（+1,056） |
| canvas Fail | 108 | 210（+102，全在新增键） |
| canvas NotRun | 239 | 588（+349，全在新增键 reftest-format 标记） |
| canvas-worker 键 | 1,010 | 1,016（+6 全 Pass） |
| canvas-worker Pass | 1,044 | 1,050（+6） |
| Timeout | 1 | 1（共享键不变） |

- 共享键零回归：main 1,331/1,331、worker 1,010/1,010，逐键状态序列与批前一致。
- 新增键：main 1,376 = Pass 1,056 + Fail 102 + NotRun 349（reftest-format/
  -expected 文件诚实标记，分母 0 子测试不计账）；worker 6 全 Pass（顶层 2 +
  wide-gamut worker 4）。
- 对账口径：summary-chrome @4b26db8978（分母 4,074 键 6,251 子测试，chrome 过
  5,483）映射执行键 2,209 → 3,461（+1,252），映射 Pass 2,163 → 3,209
  （**+1,046**）；执行超出分母的本地键 Pass 15 → 31（+16，溢出诚实记账——pin
  较新/分母侧无对应键）。基线映射 2,163 与 coverage-2026-11-20 快照 2,154 差
  9 子测试为 11-20 CI runner 与当前 main 的版本漂移，非本批效果。

## 分母分类账（html/canvas 4,074 键全量对平）

| 分类 | 键 | 子测试 | chrome 过 |
|---|---:|---:|---:|
| 已执行（本批后） | 3,461 | 3,455 | 3,103 |
| manual 门（范围外自述式） | 469 | 2,473 | 2,314 |
| pin 缺失（315976933 无此文件） | 135 | 293 | 42 |
| canvas 顶层（无扫描路径） | 7 | 19 | 19 |
| element 顶层（pin 缺失 + reftest 形） | 2 | 11 | 5 |

- pin 缺失 135 键明细：text 80/221（2d.text.shape.* tentative 族为 2026-11 上游
  新面）+ wide-gamut-canvas 9/32 + path-objects 21/21 + fill-and-stroke-styles
  21/16 + canvas-context 3/2 + video 1/1（媒体面范围外）。
- canvas 顶层 7 键：color.window 15 + historical ×3 + canvas-css-random +
  2 crash（0 子测试）——顶层散键无扫描链，后续微批可挂。
- 对平核验：3,461 + 469 + 135 + 7 + 2 = 4,074（逐键不重不漏）。

## 新增键 Fail 战役图（102 Fail，刀位储备）

- **offscreen/text 65**——TextMetrics DOM 关联面：getIndexFromOffset null 取值
  40 + getSelectionRects 列表长 24 + lang 继承 1；
- offscreen/drawing-images-to-the-canvas 15 + shadows 8 + fill-and-stroke-styles
  ~10——`createImageBitmap` scale/resize 选项门（"不支持的 source size 选项"）；
- offscreen/fill-and-stroke-styles 12——relative color 解析 2（`color(srgb …)`
  作 fillStyle 输入）+ pattern 面；
- set-proprietary-font-names-001-crash 1（crashtest 形态 ReferenceError）+
  path-objects 1。

## 溢出与 OffscreenCanvas 门注记

offscreen 主线程面基于 R3312 主线程 OffscreenCanvas（part05.js）执行——与
element 面同引擎码路径，整批 73% 子测试 Pass 率与 element 面相当；worker 影子
上下文无 host 桥接的面（drawImage 图像源经 createImageBitmap 门）在两形态同根因。

[逐文件 JSON](2026-11-21-dc2-batch7-canvas-offscreen-main-faces.json) ·
[批六证据](2026-11-21-dc2-batch6-semantics-complement.md)
