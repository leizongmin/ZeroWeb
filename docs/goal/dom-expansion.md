# DOM 扩面 — dom/ranges + html/dom

**版本**: v1.3
**日期**: 2026-10-27（立项，coverage 对账驱动）；2026-11-05（M1 收口）；2026-11-05（M2 dataChange 簇收口）；2026-11-05（M3 documents 簇收口）
**状态**: Active——M1/M2 完成；M3 documents 簇收口（136 F→P：html/dom/documents 93P/139F→229P/3F，全 dom+html-syntax 双通道 0 回归；连带修复 named-proxy named-miss 扫描、iframe body getter、document.remove 缺席三面），剩 coverage 复测 + dataset 3F/mutations 尾簇
**执行模式**: WPT 驱动（上游 corpus 为验收标尺）+ 语料扩面 + 接口完备性收敛
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——DOM 面）

## coverage 对账（2026-11-05 更新：基线 coverage-2026-11-02 为复测锚）

| WPT 面 | 全量分母（子测试） | M1 前覆盖 | M1 后通道面 | 说明 |
|---|---:|---:|---|---|
| dom/ranges | 44,726 | 44/73 文件 | 63/73 文件（+19 tentative） | 已实现 API 99.97%（M2 dataChange 收口后，M1 时 99.07%）；tentative 189F 全接口缺失挂账 |
| html/dom | 72,326 | 81/396 文件 | ~230/396 文件（+149 案路径） | 旧面 99.89% 零回归；新面 42.41%（636F 六簇） |

M1 前列来自立项日报（0 覆盖口径过时——js-dom R42 已入 44 顶层案、40 号已入
81 文件，立项日对账未扣除既有通道）；精确锚见 `docs/compat/wptfyi/coverage-2026-11-02.json`
（dom/ranges zw_passes 40,582/44,726、html/dom 58,627/72,326）。

## Mission

两面一体推进：dom/ranges 是 Range API（DOM Standard §5-7）完备性——中小域
（73 文件）但子测试密度高，预期引擎工作为主；html/dom 是 HTML 元素与
Document 的 DOM 接口面（394 文件，含各元素 historical/idlharness 面）——
40 号 html-syntax-compat 当时按 parser 语义划界只收了子集，本 goal 承接
剩余接口面。

## Done Criteria

- **DC-1（语料全量）**：34 号脚本拉全 dom/ranges + html/dom（deep + any.js；
  html/dom 的 idlharness.any.js 为 build 期生成资产上游不存在，拉取记 1 个
  失败属预期，不追）。
- **DC-2（runner 通道）**：ranges 归 testharness-dom 通道扩语料面、html/dom
  归 testharness-html-syntax 通道扩语料面（或新增子命令，M1 rally 轮定）；
  per-dir skip 规则 + Makefile target 同步。
- **DC-3（基线）**：两面分类通过率分别落 `docs/goal/dom-expansion/evidence/`；
  接口缺失（Unsupported）与语义错误（Fail）分列；
  `docs/compat/trends/wpt-suites.csv` planned 行转数据行。
- **DC-4（coverage 复测）**：两面未覆盖子测试清零路径成立；对 40 号已收口
  面零回归（html/dom 扩面不得动 parser 语义通道既有结果）。

## 里程碑

- **M1**：语料预置 + 通道扩面 + Makefile。——**完成（2026-11-05）**：语料
  对账上游 pin 全量（见 evidence/2026-11-05-m1-corpus-audit.md）；通道复用
  既有两通道扩面 + per-dir skip 规则（ranges_case_skipped +
  HTML_SYNTAX_CORPUS_SUBDIRS 十叶子），Makefile 四 target 零改动自覆盖，
  见 evidence/2026-11-05-m1-channel-expansion.md。
- **M2**：基线分类（ranges：已实现 API 的语义正确率；html/dom：接口存在性
  图）。——首遍随 M1 落账（ranges 99.07%、html/dom 新面 42.41% 六簇 +
  接口缺失/语义分列）。dataChange 369F 簇已收口（2026-11-05）：shim 三域
  （part04/proxy、part06/textEl、part05/parsed sel）data/nodeValue/textContent
  setter 补 replace-data 边界调整 + accessor 化，定向套件 400P/0F，全 dom
  通道 A/B 零回归、html 通道字节级不变，见
  evidence/2026-11-05-m2-datachange-fix.md；待做：documents 簇
  （forms/lastModified/title ~130F）逐 API 归因。
- **M3**：可收敛簇修齐或挂账 + coverage 复测收口。——documents 语义簇
  **收口（2026-11-05）**：html/dom/documents 93P/139F→229P/3F（136 F→P，
  残余 3F 系 runner 管道挂账：embeds/plugins 树序、scripts live、
  lastModified header）；全 dom 通道 A/B 0 回归（唯一连带回归
  remove-unscopable 当轮修复，终版 6/6）、html-syntax 通道 0 回归
  +136F→P；连带修复 named-proxy `__zw` 门（named-miss 二次方扫描，
  remMs 28 倍）、iframe 文档 body 回落（Aryeh iframe 页复活）、
  document.remove 缺席（ChildNode mixin 不含 Document），见
  evidence/2026-11-05-m3-documents-cluster.md。待做：coverage 复测、
  dataset 3F、mutations 尾簇 8F；挂账候选：innerText 面（渲染
  依赖，与渲染流域划界）、partial-updates/Sanitizer/OpaqueRange 提案面、
  .window.js/.xhtml 形态缺口。

## 依赖约束（run-rules §9）

- Range 语义与 editing（33 号）共享 selection/Range 底座，收敛轮核对两域
  基线互不回归。
- html/dom 接口面与 25 号 html-semantics-compat（主攻中）的语义面划界：
  本 goal 只收 DOM 接口行为，事件/语义行为仍归 25 号；跨界用例在 M2 归口。
