# M2：css-variables 解禁——通道迁移、病态案谓词排除与三通道计数一致（2026-10-30）

## 结论

css-variables harness 面解禁入账（DC-3 本地判据全落）：

- **运行面 58 案 / 532 子测试：239 Pass / 282 Fail / 11 Timeout / 0 Unsupported**
  （239/532 = 44.9%）；0 空子测试案、coverage 管道口径 0 suspicious。
- 病态案 `variable-exponential-blowup.html`（单案 4.9GB，M1 归因实证）与
  `*-crash.html` 在 runner 谓词单点排除（`css_testharness_case_skipped`
  `PATHO_CASES`）；谓词探针复验：两案即时返回 `[]`，无内存事件。
- 三通道计数一致：单进程 `--json`（CI 形态，1m58s / 峰值 RSS 983MB）≡ 分片
  文本通道（make target 全量）≡ 分片 JSON 通道（parity 子集 10/10 IDENTICAL）。
- 历史单进程 >8.8GB 三跑被杀的账目在本轮收口：排除病态案后单进程全量
  983MB 峰值，4G 单进程守卫余量充足——CI misc 分片单进程形态安全。
- 11 Timeout 全部集中在 animation/transition 页（每案 1 个异步动画子测试
  超时）——engine transition/animation 事件面缺口，M3 失败分簇素材。

## 通道迁移（R5114 通道 → css 共享底座）

`testharness-css-variables` 子命令自 `testharness.rs` 的 R5114 专用通道
（`run_corpus_subdirs` + `css_variables_case_skipped`）迁入 M0/M1 共享底座
（`testharness_css_corpus.rs::run_css_corpus_dir`）：

- skip 规则单点：共享谓词统一 manual / crash / 病态案名 / .https / helper
  目录 / 非 testharness 页 / iframe 七类规则，两侧（fetch 脚本头注释）同源
  防漂移；R5114 谓词、`CSS_VARIABLES_SUBDIRS` 与
  `run-css-variables-corpus.sh`（头注释含已被 M1 否证的"35MB/案线性累积"
  前提）随之删除。
- 运行面语义修正：R5114 通道对非 testharness 的 reftest 形态页（本地精选
  语料 202 页）照跑并产出 runner 合成单 Pass 子测试——在 coverage 管道
  （`cmd_coverage` 按案 `max(universe, zw_total)` 记分母）会把每页虚增 1 个
  分母子测试并计入 covered_files。共享谓词的"无 testharness 引用"规则把
  这些页出运行面，入账面 = 真实 harness 断言面（与 wpt.fyi testharness
  通道同口径）。
- 分片脚本 `run-css-testharness-corpus.sh` 补 `css-variables` SUBCMD 分支；
  Makefile `testharness-css-variables` 指向共享脚本（获得外部墙钟兜底、
  断点续跑、多分片并行与 JSON 通道）；M1 parity 复验脚本同步支持
  css-variables。原 R5114 分片脚本删除。

## 数据源盘点（上游对账）

- 本地 `fetch-wpt-data` 精选仓（v1.10）css-variables 目录 261 页，对照上游
  pin `3159769338`（git trees recursive 清单）**268 个 .html blob**：缺
  2 用例页 + 5 helper 资产（reference/resources/support 目录），本轮补齐
  ——语料与上游 pin 全量一致（268/268）。
- harness 形态（源含 `testharness` 引用）**61 页**；其中 1 页病态案 +
  2 页 iframe 依赖被谓词排除 → 运行面 58 案。
- coverage 快照上游分母（coverage-2026-10-29：css/css-variables
  den_files 250 / den_subtests 584 / covered 0）里 ~192 个 0 子测试文件即
  reftest 形态页（wpt.fyi manifest 收录全部跑过的文件）；584 子测试全部
  落在 harness 形态页——共享底座运行面不损失任何可入账子测试。
- fetch 入口整合：`fetch-css-variables-subset.sh`（R5114，只补 harness
  公共资源）扩展为用例树深挖补拉（goals/lib.sh `fetch_dir_deep`，同
  pin、目录幂等），weekly CI misc 分片以 `fetch-wpt-css-variables` 为
  fetch 入口；35 号 goal 脚本 DIRS 同步加入 `css/css-variables`（goal
  级自足重建通道）。

## 验证矩阵

| 项 | 结果 |
| --- | --- |
| fmt / clippy（-p zero-wpt-runner --all-targets -D warnings） | 干净 |
| runner bin 单测（trend_csv_check 等） | 265/265 |
| 谓词探针：variable-exponential-blowup / long-variable-reference-crash | `[]`，即时返回（守卫内，无内存事件） |
| parity 子集（variable-animation 10 案） | 集合恒等 IDENTICAL |
| 全量单进程 `--json`（CI 形态，test-guard 4G/8G/2400s） | 58 案 532 子测试，1m58s，峰值 RSS 983MB，rc=1（存在非 Pass，记录模式预期） |
| `make testharness-css-variables` 全量分片文本通道 | 58 案 239/282/11/0——与单进程逐项一致 |
| coverage 管道消费（`merge_zw_results` 严格口径） | 58 files，0 suspicious，/css/css-variables/* 全部解析 |
| M0 试点面回归（selectors parse-anplusb 探针） | 112 子测试，与 M0 基线一致 |
| wpt-suites.csv 生成器契约（trend_csv_check） | 8/8 |

注：Timeout 子测试两通道计数一致（11），说明单进程内部 per-case 超时与
分片外部墙钟在本语料无分歧（无原生同步死循环形态案）。

## 边界与遗留

- **DC-3 的 CI 快照判据**（weekly coverage 快照 css/css-variables 组
  covered>0）待下一次 weekly cron（或 workflow_dispatch
  run_wptfyi_coverage=true）落账——本地已验证 misc 分片接线、fetch 入口与
  管道消费三环节；正式快照以 CI 为准。
- M0 基线不受影响：css-values / selectors 运行面零变化（PATHO_CASES 对
  两目录为空集；if-conditionals.html 仍为分片脚本快筛 + 谓词双保险，
  M0 基线经分片通道记录，口径不变）。
- 11 Timeout（animation/transition）与 282 Fail 的分簇入 M3（扩面 +
  失败簇分析），本轮只记账不展开。
