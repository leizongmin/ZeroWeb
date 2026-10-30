# M3 批次 4：css-transforms / css-motion（css/motion）扩面入账（2026-10-30）

## 结论

M3 扩面批次 4 两目录过 M0 模板入账（top15 分母目录推进至 14/15；剩
css-grid / css-text 两个重量级独立轮次）：

- **运行面（批次 4 两目录合计）152 案 / 1,131 子测试 / 220 Pass（19.5%）**。
- **十四目录统一基线**：54,424 子测试 / 15,396 Pass（28.3%）。
- css-transforms 通过率 15.5%——computed 探测族 164（scale/rotate/translate/
  transform-origin/perspective-origin 等"该属性在 computed style 中不受
  支持"）、canonical 序列化 70（perspective/-webkit-transform）、
  createDiv 123（`/web-animations/testcommon.js` 缺失，WAAPI 划界归口）。
- css-motion 通过率 26.9%——offset-* computed 探测 122（offset-path 78
  为主）+ `offset` 简写序列化族 104。
- motion 扫描根 `css/motion`（上游目录名无 css- 前缀），runner 子命令
  `testharness-css-motion`，分片/parity 脚本按 selectors 先例加特例映射。

## 基线（单进程 `--json`，test-guard 4G/8G/2400s）

| 目录 | 案 | 子测试 | P | F | TO | 通过率 | 墙钟 | 峰值 RSS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| css-transforms | 107 | 740 | 115 | 622 | 3 | 15.5% | 51.6s | 382MB |
| css-motion | 45 | 391 | 105 | 286 | 0 | 26.9% | 7.9s | 193MB |

run-face 案数与 runner 谓词预估逐目录一致（107/107、45/45）；两目录均无
`.any.js`（形态同试点，无变体口径）。motion 上游 207 html 仅 45 页
testharness 形态（其余 reftest/interpolation 演示页出运行面）。

## 通道接线（M0 模板复用）

- runner 子命令 `testharness-css-transforms` / `testharness-css-motion`
  （`run_css_transforms_cases` / `run_css_motion_cases`，共享
  `run_css_corpus_dir` 底座）；Makefile 全量 + `-one` target ×2；35 号
  goal 脚本 DIRS 追加 `css/css-transforms` + `css/motion`。
- 分片脚本 `run-css-testharness-corpus.sh` 与 parity 脚本
  `verify-css-corpus-json-parity.sh`：`css-transforms` 走既有 `css-*`
  规则零改动；`motion` 加特例映射行（注释注明上游目录名无前缀，selectors
  先例）。

## 语料对账（上游 pin blob hash）

| 目录 | 上游 pin | 本地 | 内容漂移 | 处置 |
|---|---:|---:|---:|---|
| css/css-transforms | 1,314 blobs / 1,238 html | 1,314 文件 | **2 页陈旧 → 定向重拉后 0 mismatch** | 精选仓存量走幂等快路径（`已含用例，跳过深挖`）——文件数与 pin 恰好同数但 `animation/rotate-explicit-and-implicit-keyframes.html`、`support/transform-iframe-scroll-position-contents.html` 内容陈旧；"文件数一致 ≠ 内容一致"第三次实证（backgrounds 批次 3、fonts 批次 2 先例） |
| css/motion | 209 blobs / 207 html | 207 文件 | **0 mismatch** | 深挖全量首拉（本地原空）；缺口 = META.yml + WEB_FEATURES.yml 元数据（不在抓取扩展名内，批次 3 同口径） |

- 陈旧页重拉注意：`fetch_raw` 的 jsdelivr 通道返回 301 时 `curl -sf`
  不跟随会把重定向体（~205 字节）落盘——需 `curl -sfL`。本次实测踩中并
  复核（落盘字节 204/205 → 1,783/858，hash MATCH）。
- 幂等快路径残留风险进一步收窄口径：**本地文件数与 pin 树 blob 数相等
  仍不可免对账**（三次批次连续实证内容漂移 1–2 页/目录）。

## 资产引用甄别

真缺补拉 1 个：`css/css-transitions/support/helper.js`（11,494 B，
`transform-non-invertible-no-transition.html` run-face 页引用，跨目录
helper，上游 pin 存在）。其余弃拉并归类：

- `/web-animations/testcommon.js` 缺失 → **createDiv 123 子测**
  （2 页）失败，按批次 3 WAAPI 划界归口 timing-animation 底座（wpt-data
  无 web-animations 目录），不入补拉范围；
- `../interpolation/resources/interpolation-test.js`（1 页
  transform-translate-composition.html）：解析目标
  `css-transforms/interpolation/resources/` 不在 pin 树，顶层
  `/interpolation/` 在 pin 亦 404——上游破损引用，typed-om
  comparisons.js 先例，弃拉留档；
- `common/reftest-wait.js` 等 64 页 reftest 域引用 → 渲染断言面划界
  （rendering-compat），不入本账；
- `/resources/testdriver*.js`：runner 装配层置空注入，无需落盘；
- 误报类：meta author 邮箱（`href="mailto:..."` 正则伪影）、CSS 负例
  字面量 `url(abc)`、`../not-found.png` 故意无效引用。

## 失败分簇（message 归一化：数字→N、串→'…'）

- **css-transforms（622 F / 109 簇）**：
  - computed 探测族 **164**（scale 39 + rotate 24 + transform-origin 24 +
    perspective-origin 22 + translate 20 + transform-box 6 + transform 4 +
    backface-visibility 3 + perspective/transform-style 各 1）——"属性在
    computed style 不受支持"，individual transform properties
    （rotate/scale/translate）+ origin 族 computed 值缺口，引擎可修域；
  - 泛型 assert_equals **197**（transform 插值/computed 长尾，含
    `rotate` 计算值输出 `rotate(30deg)` 而非 matrix 形态一类序列化差异）；
  - canonical 序列化 **70**（perspective/-webkit-transform 无单位值）；
  - createDiv **123**（WAAPI 归口，见资产甄别）；
  - interpolation-testcommon.js setup **32**（`target.style.setProperty`
    处 target.style undefined——helper 本地为 pin 版，runner DOM 缺口
    （template content / querySelector('.target') 路径待归因），引擎域
    挂账；motion 同簇 22）。
- **css-motion（286 F / 32 簇）**：
  - offset-* computed 探测 **122**（offset-path 78 + offset-position 16 +
    offset-anchor 15 + offset-distance 7 + offset-rotate 6）；
  - 序列化族 **104**（`offset` 简写 64 + canonical 47 + path canonical 4
    + offset-anchor canonical 3，含 `path("M N N H N")` 引号形态差异）；
  - interpolation helper setProperty **22**（同 transforms 簇根因）。

归口：**individual transform / offset-* computed 值与序列化长尾引擎域**，
两簇合计占两目录 Fail 的 46%；跨 helper 缺口（createDiv / setup）按
WAAPI/DOM 划界挂账不入本 goal 修复面。

## parity（分片 ≡ 单进程恒等判据）

全语料 parity（`make testharness-css-parity`，FILTER=目录前缀取全量）：

| 目录 | 案（single=sharded） | 判定 |
|---|---|---|
| css-transforms | 107=107 | **IDENTICAL** |
| motion | 45=45 | **IDENTICAL** |

DC-2 通道契约对新目录成立；批次 2/3/4 连续八目录 parity 无一状态残留。

## 验证矩阵

| 项 | 结果 |
| --- | --- |
| 语料对账（git blob hash vs 上游 pin） | transforms 2 陈旧页重拉后 0 mismatch；motion 0 mismatch（2 .yml 元数据口径缺口） |
| 缺口资产 | 真缺 1 个补拉（css-transitions helper.js）；弃拉类留档（WAAPI/渲染面/上游破损/误报） |
| 单进程基线 ×2 目录 | 见基线表 |
| run-face 案数 vs 谓词预估 | 107/107、45/45 逐目录一致 |
| parity 全语料 | 见上节 |
| fmt / clippy（-p zero-wpt-runner --all-targets -D warnings） | 干净（fmt 首查要求折叠 `run_css_transforms_cases` 签名，应用后复检通过） |
| runner bin 单测 | 267 passed / 0 failed（test-guard 6G/12G，17.8s） |

## 边界与遗留

- **top15 剩余**：css-grid（15,841 分母 / 上游 3,359 html）与 css-text
  （5,714 / 2,811 html）——重量级独立轮次（grid 上游语料体量与本仓渲染
  账重叠面需先划界盘点）。
- **interpolation helper setup 缺口**：32+22 子测的
  `target.style.setProperty` 引擎域归因（template.content /
  querySelector 路径）留给 engine 侧专项，不阻塞账目。
- **M4**：coverage 复测收口（DC-4）待 css-grid / css-text 入账后统一做，
  避免汇总行反复重写。
