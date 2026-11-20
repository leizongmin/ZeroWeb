# DC-2 · html/rendering 批二：UA 样式面语料接入与通道落地

2026-11-20。DC-2 第二批落在 html/rendering 簇（11-20 快照缺口 11,717 子测试、0/529
文件计分）。本批交付语料 + 执行通道（批一先例：查账与排除面先行，引擎修复挂后续批）。
release runner 实测 150 文件 9,691 子测试，Pass 0 → **2,551**，簇缺口 11,717 → ≈9,166。

## 查账口径（与批一同链）

- 分母：wpt.fyi aligned stable chrome 158.0.8127.36 @ `4b26db8978`（本地缓存
  `tests/wpt-runner/wptfyi-cache/summary-chrome.json`）；html/rendering = 525 文件 /
  11,705 子测试，其中 testharness 形计分文件 158（chrome/firefox/safari 三家同数）。
- 语料：pin `31597693`（fetch 脚本头注释），git trees 逐 blob 核对 754/754 齐平
  （含 rendering 根级 5 文件——首版脚本只列 51 个子目录漏根级，403 断点续拉后补齐）。
  上次会话匿名 API 限流（403）中断在 41/51 目录，GITHUB_TOKEN 认证续拉完成。
- 分子：本地 release runner（test-guard 包裹，8 job，`--json`）。

## 变更（两文件 + Makefile + 语料）

1. `tests/wpt-runner/scripts/goals/56-wptfyi-html-rendering.sh`——51 子目录
   （fetch_dir_html 幂等标记防限流）+ 根级 5 文件 + 5 png 表格资产显式单文件。
2. `tests/wpt-runner/src/testharness_html_rendering.rs`——独立模块（2000 行约束 +
   xhtml-syntax 先例），复用 `run_testharness_html` 执行底座。排除面：support/
   resources/reference/crashtests 助资产、`-ref/-notref` 参照页、`.xhtml` 真文档
   （无 XML 管道，FIXME DC-2+）、无 testharness 引用页（reftest 主体页无子测试
   不进分母）。单测 `rendering_pages_run_and_assets_skip` 锁排除规则。
3. `main.rs` match 臂 + Makefile `fetch-wpt-html-rendering` /
   `testharness-html-rendering`（1800s 墙钟）。CI 零编辑：discover 正则
   `"testharness-([a-z0-9-]+)"\s*=>` 命中新臂，fetch 命中 `fetch-wpt-<suite>` 约定，
   catch-all 自动入片（xhtml-syntax DC-1 同款）。

**共享键零回归**：本批对既有套件零代码触点（纯新增模块/match 臂/make 目标），
既有 shard 键面结构性不变。

## 结果（150/158 计分文件执行）

| 面 | 数值 |
|---|---:|
| 执行文件 | 150（余 8 键见下） |
| 子测试 | 9,691（Pass 2,551 / Fail 7,132 / Timeout 8） |
| 分母逐字对齐键 | 138/150（子测试计数与 wpt.fyi den 严格相等） |
| 簇分子 | 0 → 2,551（缺口 11,717 → ≈9,166） |

8 键未执行（den 合计 88）：1 `.xhtml` 排除面（den 5）+ 2 `.window.html` 键 =
本地 `.window.js` 源的 wptrunner 生成形（den 10，通道不执行 .js 形，FIXME）+
5 键上游漂移（den 78）：`unicode-bidi-ua-rules-iso-8859-8`（pin 只剩
`unicode-bidi-ua-rules.html`）、`img-empty-alt-replaced-block`（pin 为
`img-empty-alt-replaced.html`）、`cellspacing-dynamic-within-display-none` /
`img-aspect-ratio-alt-text` / `getComputedStyle-hidden`（pin 子树全无）——
分母 rev `4b26db8978` 与语料 pin `31597693` 的漂移形（批一"上游漂移"同类）。

12 键计数缺口 = Timeout 截断：svg-embedded-sizing 6 键（iframe/embed/object
SVG 子文档完成回调不触发，den ~1,294 只入 1-2）、table-attribute / resets
页级超时（-7 / -35）、pixel-length-attributes（561 Fail 面 + 截断）、
baseline-alignment（wedge）、field-sizing 2 键。

## 根因探针（批三挂账；2026-11-20 晚间探针 6-8 已修正第 1 项归因）

探针页（本地临时，未提交）经 `testharness-html-rendering` 通道实测：

1. **同任务 JS 创建元素不可见＝最大单根因（修正定谳；推翻本文件首版"`*`
   选择器漏命名空间"归因）**——test-ua-stylesheet.js helper 在**同一任务内**
   同步完成 clone→`refs.append(clone)`→`querySelectorAll('#refs *')`→逐属性
   compare；而 document 级 QSA 求值的是 Rust DOM（变异管线跨任务界排空），
   同任务创建子树尚未入账 → `refEls` 空 → 全部断言对幽灵空样式
   （`expected "" but got "0px"/"none"/…` ≈4,400 Fail；lists×2 键 138/2,070、
   headings-styles 0/561 等）。探针实证链：
   - `refs.querySelectorAll('*')` = 3（元素级子树 QSA 走 JS live 视图，命名空间
     克隆全数命中；`urn:not-html` 类型选择器 `#refs ul` = 0 属 spec 正确面——
     HTML 文档类型选择器限 HTML 命名空间，勿"修"）→ 选择器引擎 `*` 匹配
     **无罪**，首版归因撤回；
   - `document.querySelectorAll('#refs *')` = 0/1（Rust 视图）：普通
     `createElement('div')` 产物同样不可见（探针 8 plainIn=0）——**非命名空间
     特异**，是同任务变异排空时序；
   - JS live 视图嵌套正确（`ns.appendChild(li)` 后 parentNode/childNodes 恒
     正确），Rust 视图同任务恒缺席——两视图分裂是失败机制。
   解锁刀位＝engine shim/bridge 同任务 DOM 可见性（QSA/getComputedStyle 前
   排空或视图统一），属 js_dom_bridge 深面（零-web 流核心域、在途 WIP 所在），
   需跨流碰头/用户点名后动刀，本 goal 流不单方面改。
2. **iframe 子文档 defaultView 面（114 Fail）**——pixel-length-attributes /
   unmapped-attributes 调 `document.defaultView.getComputedStyle`（子文档），
   报 `reading 'getComputedStyle'` on null；`Subframe should be in quirks mode`
   断言同面。
3. **svg-embedded-sizing 完成面（den ~1,294）**——iframe/embed/object 嵌 SVG
   子文档加载不完成。
4. **dimension attributes 映射残面**——dimension-attributes 958/1,720 过，
   762 Fail 为 width/height/align 属性 → presentational hints 元素面缺口
   （页面经 iframe 映射，与 2 同域）。
5. **UA 样式表值差（跨任务可见面，独立可修）**——`expected "inline" got
   "block"` ×330、`isolate`、margin 16px 等；静态 DOM 页（unicode-bidi、
   form-controls/resets 等）在 test() 内断言，跨任务已见，属 UA 样式表
   取值面（rendering 流域），不被 1 阻塞。

## 验证清单

- release 构建 ✓；单测 1/1 ✓（`cargo test -p zero-wpt-runner rendering_pages`，
  test-guard --compile-first）。
- fmt：两文件 rustfmt 后 0 diff（workspace 余量 fmt diff 全在 crates/engine，
  并行流在途工作面，本批不触）。
- clippy：`-p zero-wpt-runner --all-targets -D warnings` ✓；deps 上
  zero-engine ×11 + zero-css-parser ×1 为并行流面/旧存量 lint
  （`match_result_ok` 出自 94057657b），以 `-A` 放行 deps 后 runner crate
  零告警，本批不修他流域。
- 语料齐平：git trees 754/754；fetch 脚本幂等重跑 rc=0。

[逐文件 JSON](2026-11-20-dc2-rendering-batch2.json) · [批一证据](2026-11-20-dc2-dom-batch1-variants-idlharness.md)
