# wpt.fyi 差距分析

只读接入 [wpt.fyi](https://wpt.fyi) 公开数据，把 ZeroWeb 的 WPT 成绩与
chrome/firefox/safari stable run 放到同一口径下对比，产出可执行的修复优先级清单，
以及 ZeroWeb 在全量 WPT 面上的真实通过率与缺口地图。
工具：`tests/wpt-runner/scripts/wptfyi_gap.py`（纯标准库 Python）。

## 回答的问题

「在大家都跑的同一批 WPT 测试上，哪些是三家浏览器全过、我们没有过的？」
这类 gap_core 案例是兼容性投入产出比最高的修复目标；反过来，
「三家都没过」的案例（上游测试脆或规范歧义）应当降优先级，避免白费功夫。

coverage 子命令回答另一个问题：「ZeroWeb 在全量 WPT 面上到底通过了多少、
真实缺口在哪」——分母取三家并集全量清单，未跑的测试直接记 0，得到全量
口径通过率与分目录缺口地图，指导语料覆盖扩大的排期。

## 口径

- 浏览器基准：`wpt.fyi/api/runs` 的 aligned stable run——三家落在同一上游
  revision（`labels=stable&aligned=true`），summary_v2 从 run 的 `results_url`
  公开下载，无需鉴权。
- 对照列：ladybird 只有 master/experimental nightly run（无 stable 频道），
  revision 不与三家对齐，因此**只进 coverage 快照的浏览器对照表与 `browsers`
  JSON，不进对账分母**；fetch 失败只降级警告，不阻塞三家 baseline。本地补拉
  对照列用 `fetch --refresh`（缓存完整性判定只看 aligned 三家）。
- 文件级「全过」：ZeroWeb 侧 = 该文件全部子测试 `Pass`（`Timeout`/`Fail` 均
  不算过，`Unsupported`/`NotRun`/`PreconditionFailed` 属中性也不算过）；
  浏览器侧 = summary 的 `s ∈ {O, P}`（OK=全部子测试过；PASS=单测试/reftest 过）。
  浏览器侧 `S`（SKIPPED，未执行）也按未全过记账——wpt.fyi 对 `.js` 源名本身
  会收录 `s='S'` 的占位条目，真实结果在生成变体上，不能当通过证据。
- **完整交集**：只对三家 summary 都对齐到同一文件的路径判定。路径对不上的
  （浏览器未跑、上游已改名）计入「交集外」，不强行判定。
- **变体解析**：runner 以 `.js` 源名上报（如 `foo.any.js`），wpt.fyi 收录生成
  后的 HTML 变体（`foo.any.html`、`foo.any.worker.html` 等）。带变体规则的
  路径变体优先（按执行 scope，window 的 `.any.html` 优先），且要求同一候选
  变体在三家都存在——杜绝三家各比各的生成文件；无候选全齐时才回退 exact
  源名比对。
- **环境受限单列**：runner 对静态语料无法执行的服务端行为打
  `zw-static-corpus:` 失败前缀。失败信息全部命中环境标记（`--env-marker`，
  默认 `zw-static-corpus`）的文件归入 `zw_env_limited`——这是语料边界而非
  引擎差距，不计入 gap。
- 分类结果：`gap_core`（三家全过、ZW 未全过）＞ `gap_partial`（部分全过）＞
  `both_fail`（交集内没有一方全过——上游测试脆、规范歧义或三家均未执行，
  降优先级）；另有 `zw_strong`（ZW 过、有浏览器没过，信息面）、`both_pass`、
  `zw_env_limited`。

## 用法

```bash
# 0. 语料 revision 台账（阶段 0：fetch 脚本 WPT_REV pin 汇总，无网络）
python3 tests/wpt-runner/scripts/wptfyi_gap.py revisions

# 1. 下载三浏览器 aligned stable summary（约 14MB×3，缓存到
#    tests/wpt-runner/wptfyi-cache/，已 gitignore；需代理时先导 HTTPS_PROXY）
python3 tests/wpt-runner/scripts/wptfyi_gap.py fetch

# 2. 捕获 ZeroWeb 侧结果（--json 放在子命令后；退出码非 0 属预期，JSON 已落盘）
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  ./target/release/zero-wpt-runner testharness-compression --json \
  > /tmp/zw-results/testharness-compression.json
# （release 构建用 make zero-wpt-runner-release；语料先 make fetch-wpt-compression）

# 3. 差距分析（markdown + json 双输出）
python3 tests/wpt-runner/scripts/wptfyi_gap.py gap \
  --results /tmp/zw-results --out /tmp/zw-wptfyi/gap-report.md

# 3'. 全量对账（同样的缓存 + 结果目录；未覆盖路径记 0）
python3 tests/wpt-runner/scripts/wptfyi_gap.py coverage \
  --results /tmp/zw-results --out /tmp/zw-wptfyi/coverage-report.md

# 离线自测（合成数据，不联网）
python3 tests/wpt-runner/scripts/wptfyi_gap.py self-test
```

## 首次实测（2026-10-26，revision `5af3c135c9`）

三个小套件验证管线（compression 18 案 / cookies 22 案 / webstorage 25 案）：

| 套件 | 交集 | gap_core | zw_env_limited | both_pass |
|---|---:|---:|---:|---:|
| testharness-compression | 18 | 14 | 0 | 4 |
| testharness-cookies | 22 | 1 | 17 | 4 |
| testharness-webstorage | 25 | 0 | 0 | 25 |

- **webstorage 与三大浏览器完全对齐**（25/25 both_pass）。
- **compression 是真差距富矿**：14 案三家全过而 ZW 部分子测试失败
  （如 `compression-multiple-chunks.any.js` 45/60）。
- cookies 的 17 案 0/N 失败全部是 `zw-static-corpus` 环境标记（HTTP-header
  驱动的 Set-Cookie 静态语料无法执行），被正确单列——若不区分，这 17 条会
  全部混入 gap_core，使总 gap_core 虚高到 32 条（14+18）。

## 定时刷新（weekly cron）

`.github/workflows/weekly.yml` 的 `wptfyi-gap` job 每周六自动执行上述全流程
（构建 runner → 跑三套件 → 拉 wpt.fyi aligned stable → 生成快照），并把
`gap-<日期>.md` + `.json` 追加提交回 `docs/compat/wptfyi/`；也可在 Actions
手动 dispatch（勾选 run_wptfyi_gap，跳过构建矩阵）。

- 快照是追加式时点数据：`.md` 供人读，`.json` 供程序消费（`meta.runs` 记
  浏览器版本与 revision，分套件/分目录聚合 + 逐文件六桶分类）；旧快照保留
  即趋势史。
- **统计口径数据源**：文档与网站（website/metrics.json 体系）后续统一以
  `docs/compat/wptfyi/gap-*.json` 为 wpt.fyi 口径取数点，不另行抓取。
- `tests/wpt-runner/wptfyi-cache/` 仍是本地缓存（gitignored）；CI 上每次
  全新拉取。快照目录在仓库内、不受 gitignore 影响。

手动跑全套件时注意：wpt.fyi 每日滚动新 run，同一份 ZeroWeb 结果隔天重跑
gap 会得到不同浏览器基准，跨日对比请引用快照文件而非重生成。

## 全量对账（coverage）

`coverage` 子命令把 ZeroWeb 实跑结果放到三家 aligned stable run 的**并集
全量清单**上对账——这是网站「全量 WPT 通过率」的统计口径：

- **分母**：三家 summary 并集路径（上游全量测试面，约 12.5 万文件 /
  226 万子测试），子测试基准取三家 `c[1]` 最大值；ZeroWeb 跑到的文件用
  `max(浏览器, ZW 实际)` 修正（浏览器全 skip 的文件 `c=[0,0]` 不代表上游
  没有子测试）。
- **分子**：ZeroWeb 实跑 `--json` 的 Pass 子测试数；**未覆盖路径记 0**
  （不强行跑）。同一文件被多个套件跑到取 `(total, passes)` 最大的一次，
  不双记。
- **缺口分解**：未覆盖子测试（扩语料导入）与已覆盖的失败子测试（修引擎）
  分开；失败中命中环境受限标记的部分单列（`zw_env_fails`），不计引擎缺口。
- **reftest 渲染面不在本口径**：浏览器侧 reftest 文件无子测试数
  （`c=[0,0]`），渲染面用 reftest 专用口径另行统计。

输出 `coverage-<日期>.md/.json`：总量（文件/子测试分母、覆盖数、通过数、
全量通过率）、浏览器对照（各家全量口径）、按 topdir 与二级目录的分组累计
（`groups`，含分母/覆盖/全过/通过/环境受限失败），JSON 供网站 metrics
接线消费。

周报快照：weekly cron 在三套件 gap 快照之外，每周把**全部已导入套件**
跑一遍产出全量对账快照（`.github/workflows/weekly.yml` 的
`wptfyi-coverage` matrix job），随覆盖扩大数字单调上涨。

首份基线（[coverage-2026-10-27](wptfyi/coverage-2026-10-27.md)，51 套件本地
全量跑）：全量 125,334 文件 / 2,264,919 子测试，ZeroWeb 覆盖 7,407 文件
（5.91%）、通过 164,266 子测试——**全量通过率 7.25%**（浏览器对照：chrome
97.13% / firefox 95.58% / safari 94.24% / ladybird 对照列 96.25%）。缺口两看：
~89% 分母在零覆盖目录（encoding legacy-mb 三族独占 50%，扩语料立项见
`tests/wpt-runner/scripts/goals/` 31-34 号）；已覆盖面的失败与环境受限单列
分组呈现。css-variables 套件因病态内存用例（单进程 >8.8GB，test-guard 8G
上限三跑被杀）暂未纳入，css 面基本记 0，属已知限制。

## 已知限制

- **revision 漂移**：ZW 语料 pin 在 `3159769…` / `04067ce9…`，浏览器 run 是
  当前 master（每日滚动）。同路径下测试内容可能已变；对比是路径级对齐而非
  内容级对齐。coverage 口径下漂移还表现为 ZW 侧子测试数多于浏览器基准——
  已按 `max(浏览器, ZW 实际)` 修正分母。
- **reftest 大语料无 pin**：`zeroweb-wpt-data`（tag 固定）导入自不同时点的
  上游 master，无单一 revision 可对齐；本工具当前只覆盖 testharness 输出。
- **子集覆盖**：coverage 的分母是全量清单，但分子只来自已导入语料的目录；
  未覆盖面在缺口表中以「未覆盖子测试」呈现——这正是缺口地图的用途。
- summary_v2 的 reftest 文件 `c=[0,0]`，文件级判定不受影响，但拿不到
  reftest 的子测试级细节。

## 后续方向（未排期）

- 阶段 2（外向，暂缓）：向 wpt.fyi 注册 test runner，上传 wptreport 使
  ZeroWeb 以 `product=zeroweb` 上榜（参照 Servo 先例）——前提是套件覆盖
  有代表性。
- 网站接入：`scripts/generate-website-metrics.py` 读 `docs/compat/wptfyi/`
  最新快照，在 website/metrics.json 增补 wpt.fyi 口径的两个数字——gap_core
  差距面（gap 快照）与全量对账通过率（coverage 快照）；缺口地图指导后续
  排期。`docs/compat/trends/wpt-suites.csv` 的 15 列契约如需扩列同步改
  trend_csv_check 镜像测试。
- 套件覆盖扩大：缺口表里「未覆盖子测试」最多的目录就是下一个导入目标；
  导入后改 workflow 的 shard 套件列表即可纳入 weekly 全量对账。
- reftest 语料补 revision 台账，纳入渲染面对比（coverage 口径之外的单列
  渲染面统计）。
