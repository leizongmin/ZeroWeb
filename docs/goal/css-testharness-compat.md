# CSS testharness 面 — css/* 目录 CSSOM/style API 语料

**版本**: v1.8
**日期**: 2026-10-27（立项，coverage-2026-10-27 对账驱动）；2026-10-30（M0 收口）；2026-10-30（M1 收口）；2026-10-30（M2 收口）；2026-10-30（M3 批次 1 + DC-3 CI 闭环）；2026-10-30（M3 批次 2）；2026-10-30（M3 批次 3）；2026-10-30（M3 批次 4）；2026-10-30（M3 批次 5——top15 收口）
**状态**: Active——M0/M1/M2 完成；DC-3 已由 CI 快照闭环（运行 40935989794：css/css-variables 58 files / 232 passes）；M3 完成（**top15 分母目录 15/15 全部入账**，十六目录统一基线 65,157 子测试 17,747 Pass = 27.2%，批次 2–5 十目录 parity 全 IDENTICAL）；M4 进行中——css-face 三分片接线已合入（62064f318），首次 coverage 运行 40997740139 被姊妹流 weekly 运行并发取消（concurrency cancel-in-progress），DC-4 数据改由姊妹流运行 41000541561（HEAD 含接线，css-face 三分片随跑）产出，轮询收口中
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

**coverage-2026-10-29 复核**：css topdir 分母 195,334 子测试 / 35,319 文件
（上游清单滚动 +161），覆盖 3 文件 0 通过——全部落在 css/css-scroll-snap
（845 分母，其他通道顺带页）。试点两目录（css-values / selectors）覆盖
仍为 0，M0 前提不变。

**M0 基线（2026-10-30，DC-1 落成）**：试点两目录 run-face 530 案 9,843
子测试全量干跑——css-values 738/5,587 = 13.2%、selectors 2,001/4,256 =
47.0%，合计 27.8%。通道（子命令 + 内容筛减规则 + Makefile 分片 target）、
形态分簇（两目录均无 .any.js；非 testharness reftest 页占 55%+）、失败簇
与病态案（css-values if-conditionals.html Oilpan OOM 名级排除）详见
[evidence/2026-10-30-m0-pilot-baseline.md](css-testharness-compat/evidence/2026-10-30-m0-pilot-baseline.md)。
分母口径 = 本地 pin 语料 run-face 实测子测试（与 coverage 快照含 .any.js
变体矩阵的口径并行不互换）。

**M1 归因（2026-10-30，前提否证 + DC-2 落成）**：单进程长套件**不存在**
~35MB/案线性 RSS 累积——四组同进程序列（selectors i18n/parsing、
css-variables variable-s、css-values calc，197 案）实测斜率 0.06–0.82 MB/案，
单案 ~60–76MB 峰值被后续案复用；历史 css-variables >8.8GB 修正为病态案单案
4.9GB 抬升 Oilpan/V8 笼水位后按案平摊的口径错误。内存维度无需"套件内分批 /
子进程分片"治理；每案独立进程通道的真实价值是病态内存案与原生同步死循环案的
外部兜底隔离。DC-2 交付：分片通道 runner 形状 `--json`（`CSS_CORPUS_JSON_DIR`）
+ 恒等判据复验（子集 selectors 43/43、css-values 59/59 IDENTICAL）+ 管道
免特调核对 + **全量端到端**（selectors 274 案 2,001/2,249/6、css-values
256 案 738/4,837/12，与 M0 文本基线逐项一致）。详见
[evidence/2026-10-30-m1-rss-attribution.md](css-testharness-compat/evidence/2026-10-30-m1-rss-attribution.md)。

## Mission

让 css/* 目录的 testharness 形态语料可运行并入账 coverage，分三件事：

1. **试点通道**（M0）：css-values + selectors 两目录语料导入 + runner 通道
   + 干跑基线——testharness 面测的是 CSSOM API（`getComputedStyle` /
   `CSSStyleDeclaration` / 支持性判断），与渲染面 95%+ 不是同一能力域，
   通过率带只能实测；试点产出「通道 → 基线 → 分簇」模板供后续目录复用。
2. **长套件 runner 通道治理**（M1）：立项时假设"单进程 RSS 线性累积
   ~35MB/案"是 css-variables 全语料被杀（>8.8GB）的根因；2026-10-30 归因
   实验否证该前提（四序列 0.06–0.82MB/案，8.8GB 实为病态案 4.9GB 抬升笼
   水位的平摊误读，见 evidence）。真实的每案隔离需求是病态内存案与原生同步
   死循环案的外部兜底（每案独立进程通道已覆盖）；M1 剩余实质 = 分片通道产出
   runner 形状 `--json` + 合并等价恒等判据 + coverage 管道免特调消费——
   weekly 其余长套件同享。
3. **css-variables 解禁**（M2，2026-10-30 完成）：病态案
   （`variable-exponential-blowup.html` 单案 4.9GB 杀树实证；`*-crash.html`
   按仓库 fuzz-repro-guard 口径）在 runner skip 规则单点排除
   （共享谓词 `PATHO_CASES`），R5114 通道迁入 css 共享底座（skip 规则单点 +
   非 testharness reftest 页出运行面——coverage 分母不虚增），分片脚本补
   css-variables 分支，weekly CI misc 分片接线（fetch 入口
   `fetch-wpt-css-variables`）。运行面 58 案 532 子测试（239 Pass =
   44.9%），三通道计数一致，coverage 管道 0 suspicious。实测详见
   [evidence/2026-10-30-m2-css-variables-unban.md](css-testharness-compat/evidence/2026-10-30-m2-css-variables-unban.md)。

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
  misc 分片，快照 css/css-variables 组 covered>0。（2026-10-30 全落：本地
  判据 + CI 快照运行 40935989794——css/css-variables covered_files=58 /
  zw_passes=232 / zw_env_fails=0，css 顶目录自 0 入账。）
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

- **M0**：试点通道（DC-1，2026-10-30 完成）——语料 fetch（35 号脚本）→
  runner 子命令 + skip 规则 → 干跑基线与形态分簇（`.html` 直跑 / `.any.js`
  window 变体 / support 资产依赖甄别）。
- **M1**：通道治理收口（DC-2）——RSS 归因实验已完成并否证线性累积前提
  （2026-10-30）；选型结论 = 内存维度不治理，保留每案隔离通道；交付分片
  JSON 通道 + 恒等复验 + 管道契约，全量端到端回填收口。
- **M2**：css-variables 解禁（DC-3，2026-10-30 完成）——通道迁移共享底座 +
  病态案谓词排除 + misc 分片接线；运行面 58 案 532 子测试 239 Pass，
  三通道计数一致，语料与上游 pin 对齐（268/268）。
- **M3**：按簇扩面——top15 顺序逐目录过 M0 模板，每目录独立基线与
  失败簇归口；cssom-view / geometry 到此阶段先协调。（批次 1 完成
  2026-10-30：css-color/css-align/css-sizing 入账，六目录统一基线
  1,052 案 27,824 子测试 5,593 Pass = 20.1%；顺带修复根 support 文件
  `resources/check-layout-th.js` 缺失——153 个挂起超时清零、2,921 子测试
  解掩。证据：evidence/2026-10-30-m3-batch1-color-align-sizing.md。
  批次 2 完成 2026-10-30：css-typed-om/css-fonts/css-shapes 入账，九目录
  统一基线 38,853 子测试 9,583 Pass = 24.7%，parity 全 IDENTICAL；
  typed-om 单簇 = Typed OM API 面整体未实现（96.7% Fail，引擎缺口挂账）；
  fonts 幂等快路径漏差以 blob-hash 定向补拉 357 页收口（fresh checkout
  复现口径见 evidence，M4 CI 接线前须脚本化）。证据：
  evidence/2026-10-30-m3-batch2-typed-om-fonts-shapes.md。批次 3 完成
  2026-10-30：css-anchor-position/css-masking/css-backgrounds 入账，十二
  目录统一基线 53,293 子测试 15,176 Pass = 28.5%，parity 全 IDENTICAL；
  anchor-size() 序列化参数序颠倒 ~2,500 为最大可修单点（CSSOM 序列化域
  挂账）；本地深挖首拉须 GITHUB_TOKEN 认证（未认证 60 req/h 会 403 降级
  跳过）；backgrounds 精选仓 1 页陈旧重拉对齐 pin。证据：
  evidence/2026-10-30-m3-batch3-anchor-position-masking-backgrounds.md。
  批次 4 完成 2026-10-30：css-transforms / css-motion 入账，十四目录
  统一基线 54,424 子测试 15,396 Pass = 28.3%，parity 全 IDENTICAL；
  transforms 精选仓 2 页陈旧定向重拉（jsdelivr 301 须 `curl -sfL`，
  "文件数一致 ≠ 内容一致"第三次实证）；`/web-animations/testcommon.js`
  缺失的 createDiv 簇按 WAAPI 划界归口；interpolation-testcommon.js
  setup 的 target.style undefined 簇（32+22 子测）引擎域挂账。证据：
  evidence/2026-10-30-m3-batch4-transforms-motion.md。
  批次 5 完成 2026-10-30（**top15 收口 15/15**）：css-grid/css-text
  入账，十六目录统一基线 65,157 子测试 17,747 Pass = 27.2%，parity 全
  IDENTICAL；grid 布局探测簇 3,280（布局引擎未实现挂账，体量 top15
  之最）+ text 断行 approx 543 / Selection×text-transform ~390 归引擎
  专项；CDN 突发限流（jsdelivr 403）后补拉改走 git blobs API（3,313
  文件零失败）——通道脚本化与 fonts 差补一并挂 M4 前债务。证据：
  evidence/2026-10-30-m3-batch5-grid-text.md。）
- **M4**：coverage 复测收口（DC-4）+ wpt-suites.csv 汇总行。

## 依赖约束（run-rules §9）

- runner 内存治理不改每案 WebView 生命周期语义（`WebView` !Send，同线程
  创建+销毁）；分片/子进程方案核对 `--json` 输出契约与 test-guard 内存
  上限口径（per-proc 与 total 两级）。
- 通道落地的 skip 规则两侧同源（fetch 脚本与 runner 谓词），html-semantics
  防漂移先例。
- css 值语义修复若触 `zero-engine`/`style-system`，与 rendering-compat
  的 reftest 回归门（716/716）同窗验证，防两账互相打红。
