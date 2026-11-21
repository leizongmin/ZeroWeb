# DC-2 · 批九：html/infrastructure + 小簇搭车（obsolete/links/meta/scripting/select）语料接入

2026-11-21。html/infrastructure（约 8,000 文件的最大基础簇）此前整簇 0 执行，
本批新通道 `testharness-html-infra` 首次接入：26 目录 + 根级散文件
（meta/refresh-time.html、select×3、.webmanifest×6 等）按 pin
315976933870b34d6ea30e3f6643403edae678ba 拉取，obsolete/links/meta/scripting/
select 五小簇搭车同通道。release runner 实测：132 键 / 1,158 子测试，
**Pass 623** / Fail 381 / Timeout 152 / PreconditionFailed 2，22 键全绿；
对账口径净 **+623 Pass**，覆盖分母键 +132。

## 变更（四文件）

1. `tests/wpt-runner/scripts/goals/62-wptfyi-html-infra.sh`（新）——
   infrastructure 主体 + 五小簇 fetch；`GOAL_PULL_ANY_JS=1` 补
   structured-data 4 个 .any.js（fetch_dir_html 幂等标记不覆盖后拉形态）。
2. `tests/wpt-runner/src/testharness_html_infra.rs`（新，326 行）——
   三 lane（Html / WindowJs / DedicatedWorkerJs）+ skip 谓词（-manual/-ref/
   support//resources/ 门 + testharness 引用门 + META global 收窄 + 内容门）
   + 单元测试 ×2。
3. `tests/wpt-runner/src/main.rs`——mod 接线 + `cmd_testharness_html_infra`。
4. `Makefile`——`testharness-html-infra` target（test-guard
   --per-proc-mem 4 --total-mem 8 --time-limit 1800）。

## 执行形态

- `case_variants` variant 展开：query-encoding 5 编码页 × 16 面板 = 80 个
  `?include=` 键 + `?encoding=` 5 键。
- 三形态映射：`.window.js`→`.window.html`（3 键）、`.any.js`→`.any.html`
  （4 键）、`.any.worker.js`→`.any.worker.html`（4 键，dedicated 腿）。
- utf-16be/le 页 BOM-aware 解码（`read_corpus_to_string`）→ 32 个 utf-16
  变体键入账（非 BOM 解码下整批误判 no-testharness）。
- 主文档 `{{GET[encoding]}}` per-request 注入复用
  `wpt_apply_get_query_substitutions`（query 取自 case_name，上游 wptserve
  语义），含防御性单元测试。

## 结果（release runner，test-guard 包裹，8 jobs，墙钟约 15 min）

| 面 | 值 |
|---|---:|
| 键 | 132 |
| 子测试 | 1,158 |
| Pass | **623** |
| Fail | 381 |
| Timeout | 152 |
| PreconditionFailed | 2 |
| 全绿键 | 22 |

- `?include=url` 五编码页全过——编码声明 → URL 序列化主链已通。
- structured-data `serialization-via-idb.any.js` 全过；obsolete
  document-color-02/03/04、marquee scrollamount/scrolldelay、
  links quirk-origin-check 2 键过。
- PreconditionFailed 2 = `messagechannel.any.js` /
  `window-postmessage.window.js` 各 1——前置检查 ReadableStream 可转移
  （engine 尚不支持 ReadableStream transfer），非引擎行为错误。

## 对账口径（summary-chrome @4b26db8978）

六簇分母 193 键 / 1,736 子测试（chrome 过 1,636）：此前 0 执行，本批净
**+623 Pass**，覆盖分母键 +132。通道不相交核验：主套件 DIRS（html/dom、
html/syntax、html/webappapis 等 85 根）与批一至批八各通道均无本批六簇根，
无重复计账。

## 分母分类账（193 键全量对平）

| 分类 | 键 | 子测试 | chrome 过 |
|---|---:|---:|---:|
| 已执行（本批面） | 132 | 1,219 | 1,141 |
| 内容门挂账（iframe 族 33 键 + worker/open 族 9 键） | 42 | 197 | 179 |
| service/sharedworker 生成形 lane 缺位 | 4 | 282 | 282 |
| .xhtml（historical.sub.xhtml） | 1 | 20 | 20 |
| pin 缺失（404 实证） | 6 | 18 | 14 |
| 0 子测试记账键 | 8 | 0 | 0 |

- 对平核验：132 + 42 + 4 + 1 + 6 + 8 = 193 键；1,219 + 197 + 282 + 20 +
  18 + 0 = 1,736 子测试；1,141 + 179 + 282 + 20 + 14 + 0 = 1,636 chrome 过。
- 内容门 42 键 = iframe 子文档管道同根因（master 卡点，批四 107 + 批五 39 +
  批六 12 键同门）；worker/open 族 9 键（new Worker/SharedWorker/
  serviceWorker/window.open）同属跨文档/worker 执行面。
- lane 缺位 4 键：messagechannel.any.{service,shared}worker.html（137+137）
  + no-coop-coep.https.any.{service,shared}worker.html（4+4）——service/
  sharedworker 生成形通道未建，最大单项挂账 282 子测试。
- pin 缺失 6 键明细见 [逐文件 JSON](2026-11-21-dc2-batch9-infra-small-clusters.json)
  （detached-root-gc、window-external 等，上游晚于 pin 新增，按批六先例
  不混版）。

## 战役图（381 F + 152 TO，刀位储备）

- **样式表载入链挂起（105 TO）**——query-encoding `?include=css` 五编码页
  各 21 子测试整页 TO：带查询串 `link rel=stylesheet` 载入链挂起，engine
  stylesheet fetch/加载管域。另 `?include=svg` 各 3 TO（15）同链。
- **HTMLAllCollection 语义（40 F）**——htmlallcollection.html：可调用对象
  （typeof / call）/ named 属性 / length 语义面。
- **query-encoding 序列化差（139 F）**——`?include=scheme`（windows-1251/
  1252 各 31 + utf-8/utf-16be/utf-16le 各 4）+ `attributes.sub.html?
  encoding=` 三编码 69：scheme/表单属性 URL 编码依赖序列化。
- **structured clone/transfer 深面（47 F）**——messagechannel.any.worker.js
  21 + transfer-errors.window.js 13 + structured-cloning-error-stack
  .window.js 8 + window-postmessage.window.js 5。

## 门禁注记

workspace clippy `-D warnings` 门禁当前被渲染流域提交破坏（css-parser/
style-system/layout-engine，归因见 master 碰撞记录），本批门禁降级为 scoped
`cargo clippy -p zero-wpt-runner --all-targets`：exit 0，本批文件零警告；
单元测试 2/2 过。

[逐文件 JSON](2026-11-21-dc2-batch9-infra-small-clusters.json) ·
[批八证据](2026-11-21-dc2-batch8-syntax-html5lib-dat.md)
