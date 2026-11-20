# DC-1 · 基线核对与 XHTML 实体矩阵试跑

2026-11-20。wptfyi-html goal DC-1 两项核对一次完成：① 11 月 8 日 html-sem/web-components
分片缺失核对与产物恢复确认；② `html/the-xhtml-syntax` XHTML 实体矩阵真实 MIME/文档载入
路径试跑。

## 环境与口径（共同验收 #1）

- ZeroWeb commit：`4b0731a2b`（main，2026-11-20 rebase 后）；runner：release
  `zero-wpt-runner`（native V8 shim 通道），经 `target/test-guard` 包裹执行。
- 语料 pin：`315976933870b34d6ea30e3f6643403edae678ba`（goals/lib.sh 全局 WPT_REV，
  fetch 脚本 `tests/wpt-runner/scripts/goals/55-wptfyi-html.sh` 显式清单 21 文件）。
- 分母对照：wpt.fyi aligned stable chrome 158.0.8127.36 / firefox 160.0.1 /
  safari 27.2 @ `4b26db8978`（coverage-2026-11-20 头部台账）。
- 机器：Linux 6.18.58-x64v3-xanmod，8 并行 job，单案预算 30s（长超时案按
  `<meta name=timeout content=long>` 放宽 60s）。

## ① 11 月 8 日 html-sem/web-components 分片缺失核对

- 缺失事实：[run 42499865933](https://github.com/leizongmin/ZeroWeb/actions/runs/42499865933)
  （2026-11-08）`wptfyi-coverage (html-sem)` job 的 Run suites 步失败，
  `testharness-web-components.json` 产空；聚合护栏命中两条环比回归
  `custom-elements: 2819→0（−100%）`、`shadow-dom: 5111→0（−100%）`，退出码 3，
  **坏快照未回写 main**（护栏按设计拦截）。该分片同跑 html-semantics 套件，
  html/semantics 结果同批缺失。
- 恢复确认：11-10 起 run 42801058132 / 42843305476 / 42947347500（11-11）/
  43240558696（11-12）均 success，其中 11-12 run 的 html-sem 分片 job success；
  coverage-2026-11-12 快照 custom-elements 2,822 / shadow-dom 4,310 非零入账，
  html/semantics 计分恢复。受影响周跑产物即 11-08 候选报告——被护栏拦下未发布，
  无需另行回填；11-12 起已发布快照即为同 revision 链上的完整替代。
- 对本 goal 基线的影响：无。html 顶级目录 11-07 快照 71,096/159,323 → 11-20 快照
  73,076/159,876（并行 html-semantics 流增益 + 上游漂移，另账），分母分片链完整。

## ② XHTML 实体矩阵试跑

### 机制（上游 pin 逐文件实读）

`html/the-xhtml-syntax` 缺口 25,607 子测试 / 15 文件，构成：

- **12 × `xhtml-mathml-dtd-entity-{1..12}.htm`** = 25,606 子测试（计分面）。
  外层页 `<iframe src=support/xhtml-mathml-dtd-entity.htm>` + onload 调
  `contentWindow.run([mime, publicId, systemId, label])`；support 页 XHR 拉
  `entities.json`（2,231 项命名实体）后逐实体
  `DOMParser.parseFromString(doctype+串, mime)`，断言 `body#test` 存在、
  firstChild 为 Text（nodeType 3）、逐 charCode 等于实体期望字符。
  11 文件传 `application/xhtml+xml` + 各 XHTML/MathML/WAP DTD publicId
  （systemId 恒 `"foo"`），各测 2,125 项（`;` 结尾实体）；entity-10 传
  `text/html` + null doctype，测全 2,231 项。
- `xml-stylesheet-blocking.xhtml`：1 子测试（真 .xhtml 文档，XML MIME 页面载入）。
- 2 reftest 形（data-xhtml-with-dtd、adopt-while-parsing-001 及其 ref /
  adopt-while-parsing.xhtml）：本口径 reftest 无子测试不进分母。

### 通道落地

- runner 新通道 `testharness-xhtml-syntax`（`tests/wpt-runner/src/testharness_xhtml_syntax.rs`
  + main.rs 分发）：递归扫 `html/the-xhtml-syntax`，执行 `.htm`/`.html` testharness 形；
  零执行记账 = `support/` 助资产、rel=match reftest 形、`.xhtml` 真文档（runner 无
  XML MIME 文档载入管道，DC-2 门控）。Makefile `fetch-wpt-xhtml-syntax` /
  `testharness-xhtml-syntax`（TIME_LIMIT 默认 1200s）。
- CI 接入：`ci_discover_shards.py` 自检通过——新套件按缺省约定
  `fetch-wpt-xhtml-syntax` 入 catch-all-8（与 wai-aria 同片），零手调编辑即入周跑。

### 结果（12/12 文件全过，逐文件数与浏览器分母逐位吻合）

| 文件 | ZeroWeb P | 浏览器分母 c[1] |
|---|---:|---:|
| xhtml-mathml-dtd-entity-1/2/3/4/5/6/7/8/9/11/12.htm（11 文件） | 各 2,125 | 各 2,125 |
| xhtml-mathml-dtd-entity-10.htm（text/html 案） | 2,231 | 2,231 |
| **合计** | **25,606** | **25,606** |

Fail/Timeout/Unsupported 全零，rc=0；同批 text 输出轮（先跑）与 JSON 轮两次完整
跑结果一致。逐文件明细见
[2026-11-20-dc1-xhtml-entity-trial.json](2026-11-20-dc1-xhtml-entity-trial.json)。

引擎面兑现路径：iframe 载入 support 页（R115 iframe 管道 + M3 成熟/信任 load）→
子窗内联脚本执行 → 子窗 XHR 拉 entities.json → 跨窗 `parent.test/done`
（explicit_done harness）→ `DOMParser.parseFromString` 逐实体断言。

### 外部实体安全边界

- DOM 解析器对 DOCTYPE 只记 publicId/systemId 元数据
  （`crates/dom/src/parser.rs` append_doctype_to_document），无任何 DTD 取回路径
  （文件内无 fetch/request 调用）；实体解析走内建命名实体表。
- 试跑离线环境通过、systemId 恒为不存在的相对 URL `"foo"`——若发起外部取回，
  12 页 × 数千子测试必现挂起/失败；实测全过且整轮秒级完成，实证零外部请求。
- 结论：外部实体安全边界保持「不取回外部 DTD、只解析内建实体表」；与现有
  安全边界一致，未放宽。

### 机制注记（诚实记账）

`DOMParser.parseFromString` 当前对全部 MIME 按 HTML 解析
（`crates/engine/src/js_dom_shim/part02.js` 6399 起，R186 最小 XML 面：根元素
大小写保源 + namespaceURI 最小读取）。实体矩阵 25,606 子测试的全部可观察断言
（body#test 存在 / Text 节点 / charCode 逐位）由 HTML 命名实体解码满足——
HTML 实体表是 XHTML+MathML DTD 实体集的超集，两种解析路径对这些断言不可区分。
真 XML 解析器（命名空间完整语义、XML 良构错误文档 parsererror 面）未实现：
- 计分不受影响（分子=实跑 Pass 子测试）；
- 影响面 = 真 `.xhtml` 文档载入（xml-stylesheet-blocking.xhtml 1 子测试 +
  2 reftest 形）与未来命名空间敏感簇——挂 DC-2 门控，届时按簇决定是否建
  XML 文档管道。

## 对预算的投影

试跑面入账后（DC-3 周跑口径）html 顶级目录 73,076 → ≈98,682（+25,606），
对 11-07 基线净增 ≈27,586。目标 152,096 尚余 ≈53,400，下一批缺口（11-20 快照
未覆盖子测试数）：html/semantics 13,962、html/dom 13,415、html/rendering 11,717、
html/syntax 6,298、html/canvas 4,240、html/webappapis 3,672 等。
