# M3：coverage 复测收口（DC-4）+ 门禁记账

日期：2026-10-29。过程工件在 `/tmp/zw-form-url-m3-cov/`（输入目录符号链接，
不入库）与 `/tmp/zw-form-url-m2/`（M2 全量 JSON，coverage 输入）。

## coverage 账本再生成

- **输入**（55 文件）：31 号 DC-4 的 54 通道实跑 JSON（其 DC-4 本地 results
  目录，复用未再跑）+ 本 goal M2 全量通道
  `testharness-form-url-encoding.json`（86,770,893 B，766,959 子测试）。
- **命令**：`wptfyi_gap.py coverage --results <dir> --out
  docs/compat/wptfyi/coverage-2026-10-29.md`（默认 cache
  `tests/wpt-runner/wptfyi-cache` + env-marker `zw-static-corpus`；systemd
  user scope 8G/1800s 守卫）。账本 `.md` + 同名 `.json` 原地再生成（31 号
  5d59e6514 先例：日期账本为当前快照，csv 行载历史）。
- **cron 碰撞记账**：weekly cron（`ec93030d9`，14:39Z）在本轮本地再生成前
  已以 CI 侧结果覆盖过该账本——浏览器面刷新至 07ce6e987a 新 run，但 CI
  mb-decode/form-encode 分片彼时 tests:0 全 Timeout（22b817e55 修复前），
  头条仅 15.84%。本轮再生成以本地 54 通道高质量结果 + 既有浏览器口径
  （ffc4686d11）为准，push 时 rebase 冲突按本地快照保留解决；下轮 cron 将
  以新浏览器口径 + 修复后 CI 结果再基线（趋势账 csv 各行不受影响）。

## 结果（全量头条增量入账）

| 指标 | 前（31 号 DC-4 后） | 后（本轮） |
|---|---:|---:|
| 全量 WPT 子测试通过率 | 24.19% | **58.02%** |
| 通过子测试 | 548,502 | 1,315,460 |
| 覆盖文件 | 8,627 | 9,444 |

- **增量精确对账**：Δ通过 = 1,315,460 − 548,502 = **766,958**，与本 goal
  M2 通道基线（766,958P）逐位一致——其他通道零扰动。
- **encoding topdir**：31.38% → **97.93%**（361,578 → 1,128,536 通过，未覆盖
  790,816 → 23,858，覆盖文件 418 → 1,235）；`legacy-mb-{japanese,korean,
  tchinese}` 三行移出未覆盖榜。条目数 9,034 → 9,851（+817 = 本通道文件条目）。
- **口径如实单列**（DC-4）：账本 encoding 分母 1,152,394 为 wpt.fyi 全
  encoding 域，含范围外文件（单字节遗留编码等，入口文档划界不拉语料）——
  残余未覆盖 23,858 即落在该面，属「扩语料导入」缺口而非引擎缺口。本 goal
  面分母维持 M0 钉死的 766,959 单一口径，与账本两账口径差沿 M0 记账不变。
- csv：`docs/compat/trends/wpt-suites.csv` 追加 completed 行
  （766,959/766,958），210 行 planned 占位保留（追加不改史）。

## 门禁与单测红面定性（诚实记账）

`make test`（test-guard + DISPLAY=:20）rc=2，11 FAILED，逐项定性：

- **10 项 = js_dom_bridge fetch/Request/Response/iframe-SW 桥接环境性失败面**：
  - 25 号流 s140 当日已 stash-对照**定谳 HEAD 基线 8F**（fetch_sync_wire/
    cross_origin_origin/forbidden_headers/passes_credentials/
    iframe_service_worker_messages/headers_guard_r3223/body_readers_r2982/
    response_request_constructors，+details flake 间歇第 9），勘根挂账归
    25 号（s141 候选③「恢复质量门禁基线」）。
  - 本轮直接 A/B：M2 两文件（text_encoding.rs/part03.js）checkout 回
    42c890d81 态、同命令同环境隔离复跑 11 测试 → **10F 同集合同形态**
    （`Invalid URL: /request` at `_zwRequestValidate`、`undefined` vs
    wire 串），实证红面与 M2 文件内容无关；t8g_npo 单测该态通过。
  - **漂移观察**：本轮 10F 较 s140 名册多 2 成员
    （`test_fetch_response_binary_body_r3021`、
    `test_request_signal_passthrough_r3045`），归入 25 号在册勘根项面，
    本 goal 不重复施工。
- **1 项 `test_t8g_npo_iframe_scan_no_collection_leak` = 负载敏感墙钟
  flake**：阈值断言 <4000ms，源码注释自载「全量套件并行负载下边缘
  2.0-2.3s……套件增重后并行窗口越线」；隔离复跑 HEAD 态 **0.67s ok**
  （42c890d81 态同）——make test 全 workspace 并行负载越线，非回归。

结论：红面 11 项**零项可归因本 goal**（10 环境性在册 + 1 负载 flake）。
「cargo test 全过」门禁按在册环境性失败面口径如实记账（同日归档
net-api-compat-v2「在册并行流红」同口径先例）；本 goal 施工面门禁：
M2 evidence 记 fmt 干净 + clippy `-D warnings` 通过 + 单测定向全绿。

## DC-4 判定核对

- coverage 对账 encode 面清零路径成立：not-run 余量（767,673 记 0 分）转
  已覆盖，账本 legacy-mb 未覆盖清零 ✓
- 通过率如实单列：面基线 766,958/766,959 单列；账本头条 58.02% 全量口径，
  encoding topdir 97.93%，残余未覆盖归语料边界 ✓
- 全量头条增量入账：24.19% → 58.02%，csv completed 行落账 ✓

## 遗留挂账

- `iso2022jp-encode-form-errors-stateful` 单子测试 Timeout（唯一非 P）：
  链路含 data: URL iframe 脚本执行 + 子→父 postMessage，runner data: iframe
  不走脚本通道（M0/M1/M2 三轮实测 600s Timeout）。acceptCharset/编码语义已
  全部落地，重入条件 = runner data: URL iframe 脚本通道落地后单案复跑。
  已落入口文档 Final Output Protocol 与归档 README 登记行。
