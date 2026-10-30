# M3 批次 1：css-color / css-align / css-sizing 扩面入账 + 根级 support 文件修复（2026-10-30）

## 结论

M3 扩面首批三目录过 M0 模板入账，并顺带修复一个掩蔽真实账目的语料基建缺口：

- **运行面（六目录合计）1,052 案 / 27,824 子测试 / 5,593 Pass（20.1%）**——
  批次 1 三目录新增 464 案 / 17,443 子测试 / 2,604 Pass。
- **根级 support 文件 `resources/check-layout-th.js` 缺失**（上游根
  resources/ 的 data-\* 布局断言库，css 面各目录合计 201 页引用）曾使
  align/sizing 的 checkLayout 族页脚本报 `checkLayout is not defined` 或
  挂起超时——修复前 align 99 TO + sizing 54 TO（153 个超时子测试）、两目录
  合计少报 2,921 个子测试；补拉后 **Timeout 全部清零**，页跑完出真实断言。
- **DC-3 的 CI 判据正式落账**：workflow_dispatch coverage 运行
  40935989794（2026-10-29T19:36Z 聚合）`css/css-variables` 组
  `covered_files=58`（判据 ≈58 ✓）/ `zw_passes=232`（本地 239，−7 为 CI
  runner 动画子测时序差异）/ `zw_env_fails=0`。css 顶目录自 0 入账为
  61 files / 232 passes / 195,338 分母 / 0.12%。
- 全语料 parity：color 62/62、align 234/234 **IDENTICAL**；sizing 出 6 个
  status-diff（2 案 6 子测，`stretch/` 布局断言族）——归因为单进程跨案
  状态残留（见下），非通道集成漂移，M2 Timeout 先例口径记账。

## 通道接线（M0 模板复用）

- runner 子命令 `testharness-css-color` / `testharness-css-align` /
  `testharness-css-sizing`（`testharness_css_corpus.rs::run_css_*_cases` →
  `run_css_corpus_dir`，契约同 `cmd_testharness_css_values`）。
- 分片脚本 / parity 脚本的 CORPUS→SUBCMD 映射泛化为规则（`css-*` →
  `testharness-<corpus>`，selectors 历史命名保留特例）——后续目录零脚本
  改动接入。
- Makefile 全量 target ×3（3600s）+ `-one` 定向 target ×3（300s）；
  35 号 goal 脚本 DIRS 追加三目录（均 FORCE=1 首拉防幂等快路径漏补）。

## 语料对账（上游 pin 全量一致）

git blob hash 逐文件对照（SHA-1(`blob <len>\0`+content)，零下载核内容）：

| 目录 | 上游 pin .html | 本地 | 差异处置 |
|---|---:|---:|---|
| css/css-color | 460 | 460 | 精选仓 4 页陈旧 → 补拉至逐字节一致 |
| css/css-align | 342 | 342 | 精选仓无此目录 → 深挖全量首拉 |
| css/css-sizing | 865 | 867 | 21 页陈旧 + 4 缺失补齐；本地多 2 页为
  R4919 自建 reftest 对（`intrinsic-nested-font-inline-block-zw-001*`，
  reftest 形态不入 harness 运行面） |

精选仓 v1.10 与 pin 的内容漂移（21+4 页）说明"文件数一致 ≠ 内容一致"——
M2 的 268/268 清单对账未覆盖此维度，本轮起以 blob hash 为对账口径。

## 根级 support 文件修复（check-layout-th.js）

- **发现路径**：sizing 基线分簇出现 `checkLayout is not defined` ×29 →
  全 css 面扫引用：align 102 / sizing 89 / values 9 / variables 1 页
  （color/selectors 0）。
- **修复**：`fetch_raw resources/check-layout-th.js`（10,265B，自包含仅
  依赖 testharness.js，pin hash 一致）；35 号 goal 脚本 `goals_fetch_all`
  后定向补拉（不入 DIRS——上游根 resources/ 目录体量大，按需点拉）。
- **testdriver\*.js 不补**：runner 装配层对 testdriver script 标签置空
  注入（testharness.rs 装配管道，宿主触摸注入面），本地无需落盘；
  `accesskey.js`（3 页引用）无失败簇证据，不扩范围。
- **前后对照**（单进程 `--json`，test-guard 4G/8G/2400s）：

| 目录 | 修复前 | 修复后 |
|---|---|---|
| css-align | 3,000 子测试 646P/2255F/**99TO**，1m54s，907MB | 4,540 子测试 657P/3883F/**0TO**，41s，302MB |
| css-sizing | 513 子测试 126P/333F/**54TO**，2m10s，869MB | 1,894 子测试 701P/1193F/**0TO**，22s，244MB |
| css-color | 11,009 子测试 1246P/9763F/0TO | 逐项一致（0 引用页——对照组 ✓） |

153 个 Timeout 全为 `testharness completed without reporting registered
tests`（页脚本 checkLayout 报错中断，已注册测试永不回报）；修复后少报的
2,921 个子测试入账，F 随之上升（跑完暴露真实断言失败）。

## 基线（修复后统一状态，六目录）

| 目录 | 案 | 子测试 | P | F | TO | 通过率 | 墙钟 | 峰值 RSS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| css-values | 256 | 5,587 | 746 | 4,829 | 12 | 13.4% | 10m32s | 507MB |
| selectors | 274 | 4,257 | 2,002 | 2,249 | 5 | 47.0% | 2m30s | 938MB |
| css-variables | 58 | 537 | 241 | 285 | 11 | 44.9% | 1m57s | 354MB |
| css-color | 62 | 11,009 | 1,246 | 9,763 | 0 | 11.3% | 8.1s | 306MB |
| css-align | 234 | 4,540 | 657 | 3,883 | 0 | 14.5% | 41s | 302MB |
| css-sizing | 168 | 1,894 | 701 | 1,193 | 0 | 37.0% | 22s | 244MB |

试点两目录较 M0/M2 记录微移（values 738→746P：8 个 Fail→Pass 翻转；
variables 239/532→241/537；selectors 2,001→2,002P）——check-layout 页在
试点目录同样受益，M3 起六目录以本表为统一基线。计时注：values 10m32s
系与 parity 分片臂并行跑（CPU 争用，M1 干跑 2m47s），墙钟数字虚高仅
计时维度，计数维度不受影响。

## 失败分簇（message 归一化：数字→N、串→'…'）

- **css-color（9,763 F / 817 簇）**：一簇独大——`color doesn't seem to be
  supported in the computed style` **7,199（73.7%）**：getComputedStyle 对
  现代色彩形态（color-mix/oklch/light-dark 等 Color-4）不支持 → 计算值面
  缺口；次簇 canonical serialization 579 + 序列化 assert_equals 838 +
  color-mix 色域换算误差族（hsl/hwb/lch 各 ~7）。引擎工作面收敛于
  **Color-4 计算值/序列化**。
- **css-align（3,883 F / 182 簇）**：指定值展开/序列化族 **~1,345**
  （align-items 391 + align-self 378 + shorthand serialization 289 +
  align-content 231 + justify-\* 105——place-\* 展开与 normal 规则）+
  checkLayout 布局几何族 **~848**（abspos align-self 的 height/width
  data-expected 断言）。
- **css-sizing（1,193 F / 147 簇）**：aspect-ratio 解析/序列化族 **~145**
  + checkLayout 布局几何族 **~300+**（stretch/block-height 族）+
  contain-intrinsic-size 23。
- **归口**：布局几何两簇（align/sizing）与 rendering-compat 的 zero-engine
  布局面同域，修复须同窗回归两账（碰撞管理节约定）；checkLayout 族此后
  为 css 面常驻形态，跨目录簇计数入 M4 汇总。

## parity 与跨案状态残留（sizing stretch 族 6 子测）

全语料 parity（单进程 `--json` ≡ 分片通道，(case, 子测名→status) 集合
恒等）：

- color 62/62、align 234/234 **IDENTICAL**。
- sizing：`stretch/block-height-001.html` 5 子测 +
  `stretch/stretch-table-001.html` 1 子测 **single=Fail / sharded=Pass**。
- 归因实验（三步，全部同实现二进制）：
  1. 两案 standalone 单进程（新鲜进程首案）= 分片臂结果一致（同 6 子测
     Pass）→ 新鲜进程语境两通道同值；
  2. 全量单进程重跑 ×2 逐子测 diff = **0 差异** → 非竞态，确定性；
  3. 结论：**单进程跨案状态残留**——全量跑序中先行页留下的引擎态使
     stretch 几何断言翻转（2/1,894 子测 = 0.3%）。属 engine 页面加载顺序
     依赖类发现，归失败分簇账本（布局几何域），不在通道层修；
     M2 Timeout 先例口径记为通道语义差异。
- 基础设施注意：parity 循环里 `| tail -6` 会吞判定行（本轮实证）——
  复跑时直接落全量输出。

## DC-3 CI 快照（判据闭环）

workflow_dispatch（run_wptfyi_coverage=true）运行 **40935989794**：
17 shard success / 6 skipped / 全 run success（2026-10-29T19:00Z 起，
misc 分片 1,795s）；聚合 coverage-2026-10-29：

- `css/css-variables`：covered_files **58** / zw_passes **232** /
  full_pass_files 16 / den 250 files·588 subtests / zw_env_fails 0。
- css 顶目录行：覆盖 61 files / 232 passes / 195,338 分母 / 0.12%
  （分母含 0 子测试 reftest 形态页，M2 已录口径）。
- 本地-CI 通过差（239→232）：动画/transition 异步子测在 CI runner 时序下
  提前触内部 30s 超时截断（532→515 子测试同因）；案数 58=58 一致。
- misc 分片当前只含 css-variables；values/selectors/color 等仍在本地
  run-face 口径——**weekly 分片批量接线是 M4 工作项**（预算数据点：
  三目录合计单进程 ~71s 本地，CI ~2-3 倍）。

## 验证矩阵

| 项 | 结果 |
| --- | --- |
| fmt / clippy（-p zero-wpt-runner --all-targets -D warnings） | 干净 |
| runner bin 单测（trend_csv_check 等） | 265/265 |
| 语料对账（git blob hash vs 上游 pin） | color 460/460、align 342/342、sizing 865/865 全一致 |
| check-layout-th.js hash | pin 一致（10,265B） |
| 全量单进程基线 ×6 目录 | 见基线表；sizing 重跑 0 差异（确定性） |
| parity 全语料 | color/align IDENTICAL；sizing 6 子测状态残留（已归因） |
| 对照组：color 修复前后 | 逐项一致（0 checkLayout 引用） |
| wpt-suites.csv 生成器契约（trend_csv_check） | 8/8 |
| DC-3 CI 快照 | 40935989794：58 files / 232 passes，判据闭环 |

## 边界与遗留

- **M3 剩余**：top15 其余目录（css-grid 15,841 / css-text 5,714 等重量级
  需独立轮次）；cssom-view / geometry 落地前按 run-rules §9 与
  rendering-compat 协调划界。
- **失败簇归口**：color 的 Color-4 计算值面（7,199 簇）是六目录最大单簇
  ——引擎工作项评估归 color 域专项，不在本 goal 修引擎；align/sizing 的
  指定值展开族与布局几何族分别归 CSSOM 序列化与渲染域挂账。
- **M4（DC-4）**：weekly coverage 分片批量接线（values/selectors/color/
  align/sizing 入 shard，含 fetch 入口与时间预算）→ 正式 css 面复测快照
  → wpt-suites.csv 汇总行收口。
- 跨流记一笔：本轮修复兄弟流（editing-compat）csv 数据行的裸逗号
  （dc719ecf0 引入，trend_csv_check 抓到），已按生成器契约改为分号——
  修复随本轮提交，master.md 碰撞管理节有对账。
