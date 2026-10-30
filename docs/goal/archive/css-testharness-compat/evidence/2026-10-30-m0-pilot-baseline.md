# css-testharness-compat M0 — 试点通道与干跑基线（2026-10-30）

语料：上游 WPT `css/css-values` + `css/selectors` + 共享 helper `css/support`
@ pin 315976933870b34d6ea30e3f6643403edae678ba（`scripts/goals/35-css-testharness-compat.sh`
GOAL_FETCH_DEEP=1 git trees 深挖拉取，gitignored）。上游树清单：css-values
672 blobs / selectors 996 blobs / css-support 61 files（2026-10-29 实测）。

## 通道（DC-1 交付物）

- **fetch**：35 号脚本深挖（`fetch_dir_deep`）。本轮补两处基建：lib.sh 深挖
  分支补 `*.css` 扩展（浅拉分支早有，深挖此前缺位——css-values support/*.css
  资产面）；35 号 DIRS 补共享 helper 目录 `css/support`（runner script fetcher
  按 `/css/support/...` 绝对路径解析到 wpt-data——selectors focus-visible 解析
  族 52 案依赖 `parsing-testcommon.js`）。
- **runner**：`tests/wpt-runner/src/testharness_css_corpus.rs`（svg 模块先例，
  2000 行约束）——子命令 `testharness-css-values` / `testharness-selectors`，
  递归扫描 + `css_testharness_case_skipped` 内容规则筛减（manual / crash /
  `.https.` / helper 资产目录 / 非 testharness reftest 页 / `<iframe` 依赖），
  fetch 脚本头注释与谓词两侧同源防漂移。
- **Makefile**：`fetch-wpt-css-testharness` + `testharness-css-values` /
  `testharness-selectors`（全量，test-guard 包裹）+ `-one` 定向（FILTER 透传）。
- **分片运行器**：`tests/wpt-runner/scripts/run-css-testharness-corpus.sh`——
  每案独立 runner 进程（单进程长套件 RSS 线性累积 ~35MB/案，M1 治理前分片是
  守卫内唯一全量通道）+ 每案外部 90s 墙钟兜底（runner 内部 30s 超时拦不住同步
  死循环——`has-complexity.html` 100% CPU 自旋 >7min 实证）+ 断点续跑
  （`CSS_CORPUS_RESUME=1`）与并行分片（`CSS_CORPUS_SHARD=k/N`）。
  ⚠ 分片奇偶以各实例自身的 todo 序为准——不同脚本版本（如病态案名级跳过
  增删）会让两实例 todo 错位一档，奇偶互补变完全重叠：本轮 s0/s1 实测 152 案
  完全重叠、152 案漏跑，靠第三次续跑收口。同版本脚本 + 同 seeding 下才可分片。

## 形态分簇（DC-1 结论）

两试点目录均 **无 `.any.js`**（树清单 + 本地语料双确认）——`.any.js` window
变体通道为后续扩面目录预留（net-api 先例），本试点未触发。

| 目录 | html/htm 总数 | run-face | 非 testharness（reftest/ref 页） | helper 目录 | crash | iframe | manual | 其他 |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| css/css-values | 595 | **256** | 264 | 46 | 16 | 11 | 1 | 1（病态，见下） |
| css/selectors | 655 | **274** | 355 | 7 | 16 | 2 | 1 | — |

- run-face 占 html 总数 43%（css-values）/ 42%（selectors）——css/* 目录一半
  以上体量是 reftest 渲染断言面（rendering-compat 流域），内容规则自动分界。
- `.xht` 22 页（selectors，XML 文档）不入通道：runner testharness 管线按 HTML
  解析，XML 文档路径缺位（svg `.svg` 同口径）——重入条件 = XML 底座。
- 资产缺口：run-face 外链 helper 仅 3 个缺失（`/resources/accesskey.js`、
  `html/semantics/interestfor/.../invoker-utils.js`、
  `html/interaction/focus/.../utils.js`——3 案面，量小挂账不阻塞）；
  `/resources/testdriver*.js` 本地缺失但 runner shim 内建 testdriver 语义
  （0 Unsupported，案面按真实断言成败计）。
- **病态案 1**：`css/css-values/if-conditionals.html`——单案 Oilpan CagedHeap
  OOM（`Fatal process out of memory: Oilpan: CagedHeap reservation`，
  fuzz-repro-guard 口径复现，4G 单进程守卫杀树连累整轮分片）。CSS `if()`
  条件面引擎半支持 → 状态指数膨胀。分片脚本名级跳过（css-variables corpus
  脚本 `variable-exponential-blowup.html` 先例）；重入条件 = if() 求值内存
  语义修复。

## 干跑基线（DC-1，test-guard 守卫内全量）

| 目录 | run-face 案 | 子测试 | Pass | Fail | Timeout | 通过率 |
|---|---:|---:|---:|---:|---:|---:|
| css/css-values | 256 | 5,587 | 738 | 4,837 | 12 | **13.2%** |
| css/selectors | 274 | 4,256 | 2,001 | 2,249 | 6 | **47.0%** |
| **合计** | **530** | **9,843** | **2,739** | **7,086** | **18** | **27.8%** |

逐案四态计数：[2026-10-30-m0-css-values-baseline.json](2026-10-30-m0-css-values-baseline.json)
/ [2026-10-30-m0-selectors-baseline.json](2026-10-30-m0-selectors-baseline.json)。
零回归基线（首轮通过率只记录不设门禁，clipboard/csp 先例）。

与 coverage-2026-10-27 对账分母（css-values 10,364 / selectors 6,229 子测试）
的口径差：上游快照含 `.any.js` 变体矩阵与 coverage 时点滚动；本基线分母 =
本地 pin 语料 run-face 实测子测试（css-values 595 html / selectors 677 含
xht，树清单 2026-10-29），两账并行不互换。

## 失败簇（M1+ 修复面入口）

**selectors（2,249 Fail / 6 TO）**：

1. `assert_equals` 值不匹配 1,131——getComputedStyle 值语义 + 选择器匹配结果。
2. `':x' is not a valid selector` 833——解析覆盖缺口，集中在
   `:is()/:not()` 含复杂选择器入 `:has()`（invalidation
   is/not-pseudo-containing-complex-in-has 两案 468）+ `:heading()`（182）
   + `parsing/` parse-is-where / parse-part / parse-has-slotted。
3. `CSSStyleSheet.insertRule` undefined 101——CSSOM API 缺口。
4. Timeout 6——fullscreen/media/modal in-has invalidation + has-complexity
   （外部墙钟兜底杀）。

**css-values（4,837 Fail / 12 TO）**：

1. `computedStyleMap` 面大头：`X doesn't seem to be supported in the computed
   style`（scale / object-position / color / letter-spacing /
   background-image...）+ `e.computedStyleMap is not a function`——CSS Typed
   OM styleMap 属性支持缺口（css-typed-om 域，扩面目录 top15 之一）。
2. `assert_approx_equals: expected 10/20 got 0`——字号相对单位（ic/cap/ch 族）
   →px 换算缺位。
3. `ident()` / `attr()` 替换语义（ident-function-substitution 全案失败面）。
4. Timeout 12——`*-invalidation.html` 族 + if() 面同步长尾。

## M1 指针

- 长套件内存治理（DC-2）：RSS 归因小实验先行（分配器驻留 vs 引擎态；
  `if-conditionals.html` 单案 4.3GB 是独立于 ~35MB/案线性累积的病态峰值，
  归因实验不要混入）；方案选型在进程内分批 vs 子进程分片间。
- selectors 通道已可作为回归标尺（47.0% 基线）；css-values 13.2% 基线薄，
  修 Typed OM/unit 换算两簇即可显著抬升。

运行入口：`make testharness-css-values` / `make testharness-selectors`（全量
分片）、`make testharness-css-values-one FILTER=<sub>`（定向）。
