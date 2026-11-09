# M1 通道扩面 + 双面基线（DC-2/DC-3 首遍，2026-11-05）

## 通道决策（DC-2「或新增子命令，M1 rally 轮定」）

**复用既有通道，不新增子命令**：ranges → `testharness-dom`（`DOM_TEST_SUBDIRS`
本含 `dom/ranges`，js-dom R42 先例）、html/dom → `testharness-html-syntax`
（`HTML_SYNTAX_CORPUS_SUBDIRS` 本含 `html/dom`，40 号先例）。理由：两通道
已拥有父目录扫描面与 CI 分片位（weekly `syntax-dom` shard），扩面=追加
扫描目录 + per-dir skip，零 CI matrix 增量。

## 代码变化

- `tests/wpt-runner/src/testharness.rs`
  - `DOM_TEST_SUBDIRS` += `dom/ranges/tentative`（OpaqueRange/createValueRange
    提案面 19 案；crashtests/ 不入扫描）。
  - `ranges_case_skipped`（新）：仅作用于 `dom/ranges` 前缀——crashtests/
    目录双保险 + 无 testharness 引用页 3 案（2 根目录 crash 页 +
    `Range-test-iframe.html` 辅助页）排除；6 个旧式 iframe 案维持既有行为
    （R115 iframe 管道消费，不回退）。
  - `HTML_SYNTAX_CORPUS_SUBDIRS` += 10 叶子目录（documents/
    dom-tree-accessors 及其 document.getElementsByName 嵌套、resource-
    metadata-management、partial-updates/tentative 及 fragment/、elements/
    五子域）；既有 `html_syntax_case_skipped` 规则零改动（ref/manual/
    iframe/worker/rel=match 内容级排除直接覆盖新面 ~130 个 reftest 形态
    dir_auto/dir-isolation 页）。
- `tests/wpt-runner/scripts/fetch-dom-subset.sh`：SUBDIRS +=
  `dom/ranges/tentative`（镜像扫描面）。
- `tests/wpt-runner/scripts/goals/40-html-syntax-compat.sh`：尾部显式补拉
  10 个 depth-2 叶子目录（fetch_dir_html depth<1 不达，support/ 先例）。
- Makefile：`fetch-wpt-dom`/`testharness-dom`/`fetch-wpt-html-syntax`/
  `testharness-html-syntax` 四 target 既有入口零改动（fetch 依赖自动覆盖
  新语料）；weekly `syntax-dom` shard 预算不变（本地实测 html-syntax 全通道
  ~7min、dom ranges 面 ~8min，3600s×2 系数余量足）。

## ranges 面（testharness-dom，dom/ranges 前缀）

```
before: make testharness-dom FILTER=dom/ranges TIME_LIMIT=2700（扩面前二进制）
after : ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 2700 -- \
        ./target/release/zero-wpt-runner testharness-dom dom/ranges --json
```

| 轮 | 案路径 | 子测试 | P | F | T | 通过率 |
|---|---:|---:|---:|---:|---:|---:|
| before（44 顶层文件） | 45 | 40,812 | 40,511 | 296 | 5 | 99.28% |
| after（+19 tentative，−3 非断言页） | 61 | 41,253 | 40,684 | 566 | 3 | 98.62% |

- before 原始 log：[2026-11-05-ranges-before-expansion.txt.gz](2026-11-05-ranges-before-expansion.txt.gz)
- after 全量 JSON：[2026-11-05-ranges-after-expansion.json.gz](2026-11-05-ranges-after-expansion.json.gz)

**零回归实证**：非 mutations 确定性面 39,114 子测试逐条 diff
before/after——**0 翻转、0 新增、0 缺失**。mutations 族仅截断形状漂移
（appendData/insertData 本轮跑完整 384P/382P，dataChange 10s 预算截断点
漂移 288F→369F），无任何 P→F。

**扩面分类（DC-3 口径）**：
- tentative 19 案 189 子测试 **0P/189F 全部接口缺失**——`OpaqueRange`/
  `createValueRange`（WICG 提案）引擎未实现，TypeError 即败；归 M3
  「接口存在性图」挂账，不做语义收敛。
- 已实现 API 语义正确率（top-level 面）：**40,684/41,064 = 99.07%**；
  唯一语义错误簇 = `Range-mutations-dataChange`（CharacterData 突变时
  Range 边界调整，369F@截断）+ mutations 尾簇 insertBefore/appendChild
  各 4F——归 M2/M3 收敛候选。

## html/dom 面（testharness-html-syntax 全通道）

```
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 5400 -- \
    ./target/release/zero-wpt-runner testharness-html-syntax --json
```

| 面 | 案路径 | 子测试 | P | F | T | 通过率 |
|---|---:|---:|---:|---:|---:|---:|
| 旧面（40 号既有扫描域） | 124 | 61,270 | 61,202 | 61 | 6+1U | 99.89% |
| 新面（10 叶子目录） | 149 | 1,106 | 469 | 636 | 1 | 42.41% |
| 全通道 | 273 | 62,376 | 61,671 | 697 | 7+1U | 98.87% |

**旧面零回归（DC-4 上半）**：本轮零引擎改动（diff 仅 runner 扫描面/脚本/
文档），旧面 61F 全部落在 40 号挂账域（render-blocking 56 + html5lib 3 +
DOMContentLoaded-defer 1 + unclosed-svg-script 2 + historical 1）——与归档
goal 残差账同域，无新增失败域。coverage 复测（DC-4 下半）在 M3 收口轮经
weekly 全量对账实测。

**新面分域账**：

| 域 | P | F | T | 通过率 |
|---|---:|---:|---:|---:|
| documents/dom-tree-accessors | 83 | 130 | | 39.0% |
| documents/resource-metadata-management | 10 | 9 | | 52.6% |
| elements/（五子域合计） | 363 | 441 | 1 | 45.1% |
| partial-updates/tentative | 13 | 56 | | 18.8% |

**新面失败簇首遍归因（636F）**：
1. **innerText 面 ~207F**（`the-innertext-and-outertext-properties`）：
   `innerText` getter 未实现（恒 undefined）——渲染依赖型接口，收敛需
   layout 联动，归 M3 与渲染流域划界后处置。
2. **partial-updates 56F**：`streamAppendHTMLUnsafe`（WICG declarative
   partial updates 提案）未实现——提案面挂账不收敛。
3. **提案 API 零散缺失 ~14F**：`setHTMLUnsafe`/`setHTML`/`Sanitizer`
   （Sanitizer API 提案）——同上挂账。
4. **documents 语义簇 ~130F**：`document.forms/embeds/plugins` live
   collection item()/namedItem 语义 + `document.lastModified` 解析 +
   `document.title` 生命周期——**M2/M3 主收敛候选**（标准面，非提案）。
5. **dir/dir_auto 动态语义 ~10F**（`expected "rtl" but got "ltr"` 等）——
   dir 属性 computed 语义，M2 归口（渲染面 dir_auto reftest 形态页已被
   skip 规则正确排除不入此账）。
6. **dataset/DOMStringMap 3F**——小簇顺手收敛候选。

## 接口存在性图（M2 输入，DC-3「接口缺失与语义错误分列」）

- ranges tentative：189/189 接口缺失（OpaqueRange/createValueRange/
  highlights 等 WICG 提案族）。
- html/dom 新面：62F 显式接口缺失形态（`X is not a function/defined`，
  大头 innerText 47 + streamAppendHTMLUnsafe 6 + DOMStringMap 3），
  574F 为断言级语义失败（上述簇 1/4/5）。

## 结构性缺口挂账（本轮不扩）

- `.window.js` 7 案（dataset-binding/dir-assorted/dir-auto-dynamic-changes/
  dir-auto-form-associated/lang-attribute/lang-attribute-shadow/
  multiple-text-nodes）——flat 扫描仅取 .html/.any.js；共享扫描器加
  .window.js 会横扫其他通道 207 个同形态文件（未授权面扩张），需独立
  分发机制，M2 甄别后再动。
- `.xhtml` 12 案（document.title-02/04、getByName ×6、resource-metadata ×3、
  cdata-iframe.xhtml）——XML 文档管道案，同上挂账。
- `html/dom` 顶层 `*.headers` sidecar 6 个（wptserve 语义）非案面，不计。

## 运行安全

两轮 JSON 捕获均经 `./target/test-guard --per-proc-mem 4 --total-mem 8
--time-limit ...` 包裹（与 Makefile target 同参数），无裸跑。
