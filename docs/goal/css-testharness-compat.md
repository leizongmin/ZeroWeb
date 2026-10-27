# CSS testharness 面 — css/* 目录 CSSOM/style API 语料

**版本**: v1.0
**日期**: 2026-10-27（立项，coverage-2026-10-27 对账驱动）
**状态**: Active——M0 未启动
**执行模式**: WPT 驱动（上游 css/* testharness corpus 为验收标尺）+ runner 基建（套件内存治理与 coverage 分片管道）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——CSS API 面）

## coverage 对账（coverage-2026-10-27 基线）

css topdir 全量分母 **195,173 子测试 / 89 个二级目录，ZeroWeb 覆盖 0**。
与 css-variables 病态案无关——缺口主体是 testharness 形态语料未导入
（本地 `wpt-data/css/` 各目录只有 reftest 清单文件，rendering-compat 的
16,594 案渲染账不在本 coverage 口径）。分母 top15（合计 66%）：

| 目录 | 分母（子测试） | 文件 |
|---|---:|---:|
| css/css-grid | 15,841 | 2,517 |
| css/css-anchor-position | 13,657 | 535 |
| css/css-typed-om | 13,445 | 411 |
| css/css-color | 12,714 | 380 |
| css/css-values | 10,364 | 519 |
| css/css-fonts | 8,665 | 561 |
| css/css-shapes | 7,527 | 385 |
| css/css-masking | 6,399 | 569 |
| css/css-sizing | 6,258 | 811 |
| css/selectors | 6,229 | 833 |
| css/css-backgrounds | 6,223 | 875 |
| css/css-text | 5,714 | 1,983 |
| css/css-transforms | 5,516 | 931 |
| css/css-align | 5,311 | 297 |
| css/motion | 4,959 | 148 |

css/css-variables（584 子测试 / 249 文件）是唯一已有 runner 通道的 css
testharness 套件，因内存问题未进 CI coverage（见 M2）。

## Mission

让 css/* 目录的 testharness 形态语料可运行并入账 coverage，分三件事：

1. **试点通道**（M0）：css-values + selectors 两目录语料导入 + runner 通道
   + 干跑基线——testharness 面测的是 CSSOM API（`getComputedStyle` /
   `CSSStyleDeclaration` / 支持性判断），与渲染面 95%+ 不是同一能力域，
   通过率带只能实测；试点产出「通道 → 基线 → 分簇」模板供后续目录复用。
2. **runner 套件内存治理**（M1）：单进程长套件 RSS 线性累积（每案独立
   WebView/V8 分配器驻留 ~35MB/案）是 css-variables 全语料三跑被杀
   （单进程 >8.8GB，test-guard 8G 上限）的根因，也是一切扩面套件进
   CI coverage 的前置。治理方案（套件内分批/子进程分片 + coverage 管道
   消费分片结果）全局受益——weekly 其余长套件同享。
3. **css-variables 解禁**（M2）：病态案（`variable-exponential-blowup.html`
   单案 4.9GB 杀树实证；`*-crash.html` 按仓库 fuzz-repro-guard 口径本就
   不入常驻运行面）在 runner skip 规则单点排除，既有分片运行器
   （`tests/wpt-runner/scripts/run-css-variables-corpus.sh`，每案独立进程）
   的执行面经 M1 管道接回 coverage misc 分片，584 子测试入账。

## Done Criteria

- **DC-1（试点基线）**：css-values + selectors 通道落地（子命令 + per-dir
  skip 规则 + Makefile target，test-guard 包裹），全量干跑分类通过率
  （文本 + JSON）落 `docs/goal/css-testharness-compat/evidence/`；
  `docs/compat/trends/wpt-suites.csv` 回填数据行。
- **DC-2（内存治理）**：长套件在 test-guard 守卫内可完整产出 `--json`
  结果且 coverage 管道可消费（分片结果合并等价于单进程输出——
  `(case, 子测名→status)` 集合恒等判据，perf/wpt-runner-testharness-speed
  A/B 先例）；weekly coverage matrix 无需逐套件特调即可纳入新套件。
- **DC-3（css-variables 入账）**：病态案排除后 css-variables 进 coverage
  misc 分片，快照 css/css-variables 组 covered>0。
- **DC-4（coverage 复测）**：重跑 coverage 对账，css 面从 0 到有数字；
  试点目录未覆盖子测试清零路径成立。

## 划界

- **渲染断言面归 rendering-compat**：reftest 渲染账（16,594 案）不在本
  goal；本 goal 只做 testharness 语义断言面。同一目录两账并存（如
  css-backgrounds），按文件形态天然分界。
- **cssom-view / geometry**：README 预留面曾点名与 rendering-compat 工作
  面重叠（DOMRect/scroll 断言双域），纳入本 goal 但排 M3+，落地前按
  run-rules §9 与 rendering-compat 控制面协调划界。
- **动态/时序面**：css-transitions / css-animations 断言依赖 WAAPI 通道
  （timing-animation-compat 已收口的 web-animations 底座），导入后失败簇
  优先归口该底座挂账，不在本 goal 修引擎。
- **病态输入**：`*-crash.html`、指数膨胀案按 fuzz-repro-guard 口径不入
  常驻运行面（css-variables 先例），skip 规则单点维护。

## 里程碑

- **M0**：试点通道（DC-1）——语料 fetch（35 号脚本）→ runner 子命令 +
  skip 规则 → 干跑基线与形态分簇（`.html` 直跑 / `.any.js` window 变体
  / support 资产依赖甄别）。
- **M1**：内存治理（DC-2）——方案选型（进程内分批 vs 子进程分片）先做
  RSS 归因小实验（驻留来自分配器还是引擎态），再落地；coverage 管道
  分片消费同切片交付。
- **M2**：css-variables 解禁（DC-3）。
- **M3**：按簇扩面——top15 顺序逐目录过 M0 模板，每目录独立基线与
  失败簇归口；cssom-view / geometry 到此阶段先协调。
- **M4**：coverage 复测收口（DC-4）+ wpt-suites.csv 汇总行。

## 依赖约束（run-rules §9）

- runner 内存治理不改每案 WebView 生命周期语义（`WebView` !Send，同线程
  创建+销毁）；分片/子进程方案核对 `--json` 输出契约与 test-guard 内存
  上限口径（per-proc 与 total 两级）。
- 通道落地的 skip 规则两侧同源（fetch 脚本与 runner 谓词），html-semantics
  防漂移先例。
- css 值语义修复若触 `zero-engine`/`style-system`，与 rendering-compat
  的 reftest 回归门（716/716）同窗验证，防两账互相打红。
