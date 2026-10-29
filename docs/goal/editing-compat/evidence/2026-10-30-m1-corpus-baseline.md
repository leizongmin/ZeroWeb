# editing-compat M1 — 语料通道与全量基线（2026-10-30）

语料：上游 WPT `editing/run` + `editing/other` + 共享依赖 `editing/include`
（implementation.js / tests.js / editor-test-utils.js / reset.css 等 6 文件）+
`editing/data`（per-command 期望矩阵 35 文件）@ pin 315976933870b34d6ea30e3f6643403edae678ba
（`scripts/goals/33-editing-compat.sh` GOAL_FETCH_DEEP=1 git trees 深挖，gitignored）。
pin 处上游全树（2026-10-30 实测）：run 43 案 + other 96 案均平铺、已拉全；
`crashtests/`（145 案病态输入）与 `manual/`（9 案真交互）不入通道，
`edit-context/` `plaintext-only/` `whitespaces/` `chrome-compat/` 不在 DC-1
语料范围（后续扩面另立项）。`run/createlink-with-selecting-img.html` 引用的
`/img/lion.svg` 在 pin 快照上游即不存在（根树无 img/），不补拉，相关子测试按
缺资源计失败。

## 通道（DC-2 交付物）

- **runner**：`tests/wpt-runner/src/testharness_editing_corpus.rs`
  （testharness_css_corpus M0 先例）——子命令 `testharness-editing-run` /
  `testharness-editing-other`，递归扫描 + `editing_case_skipped` 内容规则筛减
  （manual / `.https.` / helper 资产目录 / 非 testharness 页 / `<iframe`
  依赖），fetch 脚本头注释与谓词两侧同源防漂移。`<meta name=variant>` 逐
  query 跑（R329 同款——run/ 22 案与 other/ list-editing 案为 ?N-M / ?key=val
  分片矩阵，与上游 dashboard 对齐）。
- **Makefile**：`fetch-wpt-editing` + `testharness-editing-run` /
  `testharness-editing-other`（全量分片，test-guard 包裹）+ `-one` 定向
  （FILTER 透传）。
- **分片运行器**：`tests/wpt-runner/scripts/run-editing-testharness-corpus.sh`
  ——每案独立 runner 进程（单进程长套件 RSS 线性累积，css M0 同根因同结论）
  + 每案外部墙钟兜底（run 420s / other 150s；内部 30s 超时拦不住同步自旋）+
  断点续跑（`EDITING_CORPUS_RESUME=1`）与并行分片（`EDITING_CORPUS_SHARD=k/N`）。

## 形态分簇

| 目录 | html 总数 | run-face | iframe | https | manual |
|---|---:|---:|---:|---:|---:|
| editing/run | 43 | **42** | 1（undo-redo.html） | — | — |
| editing/other | 96 | **86** | 8 | 2 | — |

other/ 的 8 个 iframe 案（delete-in-child-of-head/html、insertparagraph-in-child-of-*、
typing-around-link-element-at-*、exec-command-without-editable-element、
insertparagraph-in-non-splittable）计入口账，重入条件 = runner iframe 文档管道。
testdriver 依赖案未排除——runner shim 内建 testdriver 语义，案面按真实断言
成败计（baseline 实证 justify/outdent/insert-list preserving-selection 族
314 子测试全绿）。

## 全量基线（DC-3，test-guard 守卫内，双分片并行 ~15 分钟）

| 通道 | 案 | 子测试 | Pass | Fail | Timeout | 通过率 |
|---|---:|---:|---:|---:|---:|---:|
| editing/run | 42 | 96,663 | 7 | 96,656 | 0 | **0.007%** |
| editing/other | 86 | 10,073 | 2,316 | 7,747 | 10 | **23.0%** |
| **合计** | **128** | **106,736** | **2,323** | **104,403** | **10** | **2.18%** |

逐案四态计数：[2026-10-30-m1-run-baseline.json](2026-10-30-m1-run-baseline.json)
/ [2026-10-30-m1-other-baseline.json](2026-10-30-m1-other-baseline.json)。
零回归基线（首轮通过率只记录不设门禁，css M0 先例）。与 coverage-2026-10-27
对账分母（113,084 子测试）的口径差同 css M0：上游快照含 coverage 时点滚动；
本基线分母 = 本地 pin 语料 run-face 实测子测试，两账并行不互换。

## 失败簇（M2 收敛面入口）

**editing/run（96,656 Fail）——单根因 setup 级联 99.96%**：

96,614 条 Fail 是同一断言 `Setup must not throw an exception`。根因已最小化
定位（[2026-10-30-innerhtml-materialization-probe.html](2026-10-30-innerhtml-materialization-probe.html)
六探针，F3/F4/F5 Fail、F1/F2/F6 Pass）：**innerHTML 产物宿主侧身份失步**——
`container.innerHTML = "<div contenteditable>…"` 后，`getElementById` 能命中
（host id_map 有节点）、JS 属性 API 可读（本地视图），但 `Element.matches`
连 tag-only 选择器都不命中（宿主句柄解析不到该元素）；`createElement +
setAttribute` 路径正常。上游 runConformanceTest 首行
`document.querySelector("div[contenteditable]")` 返 null → `setupDiv(null)`
抛 TypeError → 该案全部子测试记 setup 失败。M2 首轮修此共享路径（js_dom_shim
innerHTML 物化层——R81/R92/R95 双视图），run/ 面预期一次性解锁真实分类。
残余 ~42 Fail 为 caretnavigation / insertlinebreak / typing 真实语义缺口。

**editing/other（7,747 Fail / 10 Timeout）——按簇**：

1. **white-space 编辑 join/插入面 ~4,700**：`join-different-white-space-style-*`
   （4,062）+ `insertparagraph/insertlinebreak-with-white-space-style` +
   `join-pre-and-other-block`——delete/forwarddelete/insertparagraph 跨
   white-space 样式边界的保留与合并语义（"white-space should (not) be
   preserved by <span> elements" 断言族）。
2. **insertparagraph 分段语义 ~700**：段落拆分 / 新段落插入 / linefeed 插入
   断言族（含 editing-host-cannot-have-div、void-element 面）。
3. **execCommand 一般语义 474**：`exec-command-with-text-editor.tentative.html`
   （P1,332 / F474——execCommand 值语义与属性保持）。
4. **物化身份缺口外溢 ~240**：`Unhandled rejection` 158 + `Cannot read
   properties of null (reading 'getAttribute')` 80——与 run/ 面同根因
   （querySelector 返 null 后续解引用），M2 首轮一并复验。
5. **Timeout 10**：delete-in-last-*-list-item 6 子测试自旋 + 4 案外部墙钟杀
   （delete-without-unwrapping / insertparagraph-in-inline-editing-host /
   join-different-white-space-style-paragraphs / inserttext-last-mutation，
   150s）——list/inline 编辑宿主的 delete 循环疑似不终止，收敛轮先归因。

18 案全绿（justify/outdent/insert-list preserving-selection 314 子测试 +
textcontrol 面等）——execCommand 选择保持与 textcontrol 语义已可用。

## M2 指针

- 首轮 = innerHTML 物化身份失步（共享路径，修后 run/ 面重跑基线；跨域核对
  dom-expansion 34 号与 html-syntax 通道基线不回归，run-rules §9）。
- 第二层按 white-space join → insertparagraph → execCommand 语义顺序归簇立项。
- 自旋案（list delete 循环、4 案墙钟杀）先归因再决定是否按病态口径名级排除。

运行入口：`make testharness-editing-run` / `make testharness-editing-other`
（全量分片）、`make testharness-editing-run-one FILTER=<sub>`（定向）。
