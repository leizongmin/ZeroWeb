# M3 批次 2：css-typed-om / css-fonts / css-shapes 扩面入账（2026-10-30）

## 结论

M3 扩面批次 2 三目录过 M0 模板入账（top15 分母目录推进至 9/15）：

- **运行面（批次 2 三目录合计）656 案 / 11,029 子测试 / 3,990 Pass（36.2%）**。
- **九目录统一基线**：38,853 子测试 / 9,583 Pass（24.7%）。
- **css-typed-om 单簇独大：CSS Typed OM API 面整体未实现**
  （`CSSKeywordValue` / `CSSUnitValue` / `CSSStyleValue` /
  `computedStyleMap` / `CSS.px` 等入口全 undefined，~617/750 子测试直接报
  ReferenceError/TypeError）——引擎能力缺口按 goal 划界归口挂账，不在本
  goal 修引擎。
- css-fonts / css-shapes 通过率带 45.6% / 31.9%，失败簇归口见下。

## 基线（单进程 `--json`，test-guard 4G/8G/2400s）

| 目录 | 案 | 子测试 | P | F | TO | 通过率 | 墙钟 | 峰值 RSS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| css-typed-om | 348 | 750 | 25 | 725 | 0 | 3.3% | 31s | 131MB |
| css-fonts | 163 | 4,981 | 2,273 | 2,704 | 4 | 45.6% | 1m39s | 914MB |
| css-shapes | 145 | 5,298 | 1,692 | 3,605 | 1 | 31.9% | 31s | 433MB |

计时注：本轮与兄弟流（editing-compat，ZeroWeb-3 clone）重负载并行，墙钟
含 CPU 争用虚高；计数维度不受影响。run-face 案数与 runner 谓词预估一致
（348/163/145 逐目录相等）；typed-om 子测试密度极低（2.2/案）系页脚本
早期抛错中断已注册测试回报，非通道缺账。

## 通道接线（M0 模板复用，零脚本改动）

- runner 子命令 `testharness-css-typed-om` / `testharness-css-fonts` /
  `testharness-css-shapes`（`run_css_*_cases` → `run_css_corpus_dir`，契约
  同批次 1）。
- 分片/parity 脚本 `css-*` 规则映射与 Makefile 全量 + `-one` target ×3；
  35 号 goal 脚本 DIRS 追加三目录。

## 语料对账（上游 pin blob hash，SHA-1(`blob <len>\0`+content)）

| 目录 | 上游 pin | 本地 | 内容漂移 | 缺口处置 |
|---|---:|---:|---:|---|
| css/css-typed-om | 395 blobs / 364 html | 367 文件 | **0 mismatch** | 深挖全量首拉（原空目录）；缺口 = 3 .yml 元数据 + 11 `.any.js` 变体（通道不收 `.any.js`，M0 形态规则，计入口账） |
| css/css-fonts | 2,499 blobs / 833 html | 937 文件 | **0 mismatch** | 精选仓 476 页 + 本轮定向补拉 357 页（幂等快路径漏差——见下） |
| css/css-shapes | 620 blobs / 520 html | 546 文件 | **0 mismatch** | 深挖全量首拉（原空目录）；缺口 = 7 .yml + 30 未引用 png + 1 README |

- **fonts 幂等快路径缺口**：`fetch_dir_deep` 对已含顶层 .html 的目录跳过
  （FORCE=1 全量重拉不划算），精选仓 476 页与 pin 833 页差 357 页 +
  资产。本轮按树清单（`git trees?recursive=1`）定向补拉 357 html + 1
  .headers + 1 .css + 4 .js。**fresh checkout 复现口径**：单跑 35 号脚本
  只会得到精选仓 476 页子集；补差需按 pin 树清单逐文件对差重跑（M4 若
  CI 接线 fonts 前须先脚本化此补差，当前 misc 分片不含 fonts，无 CI 漂移）。
- 缺口资产补拉 52 个（46 直拉 + 6 CSS 转义路径）：
  - 6 个 avar 变字体文件名含 `[opsz,wdth,wght]`（CSS `url()` 内 `\,` 转
    义、真名含逗号）——URL 需百分号编码后点拉，fetch_raw 直拼会 404。
  - typed-om `the-stylepropertymap/properties/` 页面资产（svg/png/cur）。
  - css-values `resources/ChTestZeroWidthZero.woff`（试点目录存量页引用，
    顺带补齐）。
- **误报甄别（不拉，均上游一致态）**：CSS 测试值字面量（`url(filters.svg)
  blur(4px)` 等解析断言串、shapes `url(someimage.png)` 属性值）非资源依
  赖；故意坏 URL 负例（`parsing/foo.ttf`、`not a valid url/bar.ttf`、
  `should be quoted.ttf`、裸扩展名引用——上游树亦无此文件）；
  `css/css-typed-om/resources/comparisons.js`（3 页 script 依赖）**pin rev
  上游 404**——引用它的 3 页上游即坏页，失败按账记录。

## 失败分簇（message 归一化：数字→N、串→'…'）

- **css-typed-om（725 F）**：单簇独大——CSS Typed OM API 未实现
  （`CSSKeywordValue is not defined` 227 + 属性链 undefined 180 +
  `CSS[unit] not a function` 44 + `CSSUnitValue` 41 +
  `computedStyleMap` 34 + `CSSStyleValue` 21 + 其余 MathValue/
  UnparsedValue/Matrix 族 ~120）。引擎工作面收敛于 **Typed OM API 入口
  面**（`CSS.*` 工厂 + `Element.computedStyleMap()`）；达标前该目录账面
  即引擎差距账。
- **css-fonts（2,704 F / 710 簇）**：广而散——序列化/canonical 族
  （assert_equals 360 + canonical serialization 346 + cssText 59 +
  round-trip 54）、computed-style 探测族（`font doesn't seem to be
  supported` 309 + font-palette 32）、CSSRule 规则类型枚举 117
  （`bad rule type`——FONT_FEATURE_VALUES_RULE 等未入枚举）、undefined
  属性读 154、font-palette/size-adjust/variation 各小簇。归口：**CSSOM
  字体属性序列化与 @font-face 规则面**。
- **css-shapes（3,605 F / 37 簇，高度集中）**：
  - 合法双分量 position 基本形状值被拒 **~2,800**（`circle(at Npx N%)`
    / `ellipse(at N% Npx)` 等期望 canonical 串实得 ""——解析器不接受
    two-position 形态，css-values position 解析同源）；
  - 无效值未拒 **~536**（`shape-image-threshold: auto/10px`、
    `shape-margin: none/10`、`circle(at center left 1px)` 等应拒绝而接受
    ——解析校验面）；
  - computed-style 探测 **~157**（shape-outside 93 + clip-path 49 +
    threshold/margin 11）。
  归口：**basic-shape position 解析与序列化**（渲染面 shape 绘制归
  rendering-compat，本账只含解析/序列化断言）。

## parity（分片 ≡ 单进程恒等判据）

全语料 parity（`make testharness-css-parity`，FILTER=目录前缀取全量，单进程
`--json` ≡ 分片通道按 (case, 子测名→status) 集合恒等）：

| 目录 | 案（single=sharded） | 判定 |
|---|---|---|
| css-typed-om | 348=348 | **IDENTICAL** |
| css-fonts | 163=163 | **IDENTICAL** |
| css-shapes | 145=145 | **IDENTICAL** |

批次 1 sizing 的 6 子测跨案状态残留本批未复现（typed-om 0 TO、fonts 4 TO
与 shapes 1 TO 在分片臂 90s 外部墙钟内完成，语义差异未触发）。DC-2 通道
契约对三新目录成立。

## 验证矩阵

| 项 | 结果 |
| --- | --- |
| 语料对账（git blob hash vs 上游 pin） | 三目录 0 mismatch（typed-om 367、fonts 937、shapes 545 逐字节一致） |
| 缺口资产 | 52 个补拉到位；误报类甄别留档（上游一致态） |
| 单进程基线 ×3 目录 | 见基线表 |
| run-face 案数 vs 谓词预估 | 348/163/145 逐目录一致 |
| parity 全语料 | typed-om/fonts/shapes 全 IDENTICAL（348/163/145 案） |
| fmt / clippy（-p zero-wpt-runner --all-targets -D warnings） | 干净 |
| runner bin 单测 | 265/265 |

## 边界与遗留

- **typed-om 引擎缺口**：Typed OM API 面整体未实现是本 goal 至今最大单
  目录失败簇（96.7% Fail）；按划界不入本 goal 修，挂账待 CSSOM API 专项
  评估（M4 汇总时并入跨目录簇账本）。
- **fonts 补差脚本化**：35 号脚本幂等快路径对精选仓存量目录不补差——
  M4 CI 接线 fonts 前须把 blob-hash 补差落进脚本（当前仅本地一次性完成）。
- **M3 剩余**：top15 其余 6 目录（css-grid 15,841 / css-anchor-position
  13,657 / css-masking 6,399 / css-backgrounds 6,223 / css-text 5,714 /
  css-transforms 5,516）+ motion（4,959）；css-grid / css-text 重量级独立
  轮次。
