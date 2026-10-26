# wpt.fyi 差距分析

只读接入 [wpt.fyi](https://wpt.fyi) 公开数据，把 ZeroWeb 的 WPT 成绩与
chrome/firefox/safari stable run 放到同一口径下对比，产出可执行的修复优先级清单。
工具：`tests/wpt-runner/scripts/wptfyi_gap.py`（纯标准库 Python）。

## 回答的问题

「在大家都跑的同一批 WPT 测试上，哪些是三家浏览器全过、我们没有过的？」
这类 gap_core 案例是兼容性投入产出比最高的修复目标；反过来，
「三家都没过」的案例（上游测试脆或规范歧义）应当降优先级，避免白费功夫。

## 口径

- 浏览器基准：`wpt.fyi/api/runs` 的 aligned stable run——三家落在同一上游
  revision（`labels=stable&aligned=true`），summary_v2 从 run 的 `results_url`
  公开下载，无需鉴权。
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

报告是时点快照（wpt.fyi 每日有新 run），不入库；需要时重新生成。

## 已知限制

- **revision 漂移**：ZW 语料 pin 在 `3159769…` / `04067ce9…`，浏览器 run 是
  当前 master（每日滚动）。同路径下测试内容可能已变；对比是路径级对齐而非
  内容级对齐。
- **reftest 大语料无 pin**：`zeroweb-wpt-data`（tag 固定）导入自不同时点的
  上游 master，无单一 revision 可对齐；本工具当前只覆盖 testharness 输出。
- **子集覆盖**：只反映已导入语料的目录；全量 WPT 面的差距要等套件覆盖扩大。
- summary_v2 的 reftest 文件 `c=[0,0]`，文件级判定不受影响，但拿不到
  reftest 的子测试级细节。

## 后续方向（未排期）

- 阶段 2（外向）：向 wpt.fyi 注册 test runner，上传 wptreport 使 ZeroWeb 以
  `product=zeroweb` 上榜（参照 Servo 先例）——前提是套件覆盖有代表性。
- 将 gap_core 计数按目录聚合接入 `docs/compat/trends/wpt-suites.csv` 体系。
- reftest 语料补 revision 台账，纳入渲染面对比。
