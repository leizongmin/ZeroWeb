# 门禁执行与收口状态（2026-10-31，DC-1~4 落账后）

## 时序与 HEAD

- DC-1~4 证据提交：`70694a7ed`（基线 HEAD `d56743004`，docs-only）。
- push 前 `git pull --rebase` 吸入并行流引擎修复（s172/r113），落点
  **`5894374c1`**（含 `eabdd5efa` r113 期望同步、`8d29b5d6a` s172 getElementById
  根修等），已推送。
- 本 goal 全程**零代码变更**（工作区 diff 仅 docs；runner 引擎/shim 未动）。

## 门禁结果（最终 HEAD `5894374c1`）

| 门禁 | 结果 | 证据 |
|---|---|---|
| cargo build | ✅ | release（zero-wpt-runner 重建 1m25s）+ dev profile 全部 Finished |
| cargo clippy --workspace --all-targets -- -D warnings | ✅ 全绿 | test-guard 包装（--per-proc-mem 4 --total-mem 8 --time-limit 3600），0 error / 0 warning |
| cargo test（make test） | ❌ **10F** | 3614 passed / 10 failed，全部集中在 `js_dom_bridge::tests` fetch/Request/XHR/ua_display 簇（见下） |
| webcrypto 通道锚复验 | ✅ | run3（rebase 后 HEAD 重建 runner）与 run2（pre-rebase）`--json` 输出**逐字节一致**（cmp 全等），33,109 分母锚在最终 HEAD 成立 |

## make test 10F 归因：并行流既有账，非本 goal

- pre-rebase 首跑：**11F**（3612 passed）。rebase 后：**10F**——差 1 例即
  `test_style_sheet_from_handle_style_element_r113`，与并行流提交
  `eabdd5efa`（「r113 期望同步增刀 resolved……引擎基线 11F→10F」）精确对账。
- 余 10F 清单：fetch_sync_wire_preserves_response_contract、
  test_fetch_appends_cross_origin_request_origin、
  test_fetch_forbidden_headers_r3221_r3222、
  test_fetch_passes_request_credentials_to_host、
  test_iframe_service_worker_messages_dispatch_to_iframe_container、
  test_request_body_readers_r2982、test_request_headers_guard_r3223、
  test_request_signal_passthrough_r3045、
  test_response_request_constructors_r2968、
  test_ua_display_li_list_item_canvas_inline_s24_flip。
- 归属：全部为 fetch/Request/XHR/UA-display 域——`html-semantics-compat`
  并行流（s 系列账本，远端 `1026ee0c6` 自述「引擎基线 11F→10F」）正在清的
  引擎既有基线。与 webcrypto 域不相交，本 goal 不越界代修（Mission 边界 +
  精准修改准则）。

## DONE 门禁缺口与收口条件

DONE 允许条件要求 `cargo test` 全过；当前唯一缺口是上述 10F 并行流账面。
收口归档（入口 + 控制面迁 `docs/goal/archive/`、归档 README 登记、
rally-goal-26 脚本对齐、wpt-suites.csv 收行）**待该基线清零后执行**：

1. 轮询并行流进度（`git fetch` + 引擎基线账面）；
2. fetch/Request 簇清零后复跑 `make test` 全绿 + clippy 复验；
3. 执行收口归档迁移与登记，最终提交推送。

## 12:55 追记：10F 清零驱动核查（收口前提变化）

- ps / tmux / crontab 三面核查（12:55）：`rally run
  docs/goal/html-semantics-compat.md` **无进程**——在跑 rally 仅
  editing 流（ZeroWeb-3，Oct 29 起）与本流（ZeroWeb-2，10:09 起）；
  tmux 无 server、用户 crontab 空。s 系 shim 推进止于 10:52
  `3b25bb2c7`，此后远端 main 仅 rendering 流（11:22 `544616de4`、
  12:29 `66200548f`、12:45 `7039931b0`）与本流 docs（11:18/11:24）推进。
- 12:29/12:45 两笔 rendering 推进均只触及 `tests/wpt-runner/src/reftest/`
  与 rendering docs，不触及 engine lib unit 测试面——10F 组合态在当前
  tip `7039931b0` 无变化依据，未消耗测试跑复证。
- 收口前提「并行流在清 fetch/Request 簇」的驱动缺位：处置路径登记
  master.md 待用户决策表 **D2**（① 等 goal-25 重启清零 / ② standing
  exclusion 口径授权收口，先例 net-api request-cache 98F）。本 goal 仍
  不越界代修；期间维持低成本轮询（`git fetch` + s 系账面），基线清零
  即当场收口。

### 12:58 修正：s 系活跃但慢（非驱动缺位）

- 12:53 `02a689410` s173（declarative shadow prepass，双靶 F→P）落地
  ——12:55 的 ps 核查落在 s 系轮次间隙，12:53 推进证明流仍活跃；初判
  「驱动缺位」修正为「活跃但慢」（s172→s173 间隔 ~2h15m）。
- s173 触及 `js_dom_bridge.rs` + shim part04/05（shadow 域），零
  fetch/Request 面；其自账 engine lib **3615P/10F**（+1 钉测，10F 同键
  = fetch/request/response 契约族 9 + ua_display_li s24）——10F 清零
  尚未进入 s 系已完成的刀位，ETA 未知，D2 维持待拍板。

### 14:52 轮询追记：s174/s175 落地，10F 同键未清

- `git pull --rebase` 吸入 s 系两轮：s174 13:30 `47370ed94`（webview 静态
  外链 CSS entries 化）+ s175 14:45 `b5fbe98b9`（detached 节点选择器身份
  盗用根修，`unique_selector_for_node` 可达门）+ 两轮并行合并复证记
  （`bbe005e67`/`d50631d28`）。节奏 s173 12:53 → s174 13:31 → s175 14:48
  ≈ 1h15m/轮，流活跃。
- s175 自账及合并树复证 engine lib **3617P/10F**——10F 与本 goal
  [门禁账](#make-test-10f-归因并行流既有账非本-goal) 同键（fetch/Request
  契约族 9 + ua_display_li），未清；s 系刀位仍在 shadow/dialog/dom 域，
  fetch/Request 簇未进入队列可见部分，ETA 仍未知。
- 本轮组合态 tip `d50631d28`：本 goal 零代码变更不受扰；33,109 分母锚
  仍系 `5894374c1` 复验账（s173/s174/s175 引擎变更后按收口序列在 ① 命中
  时于当时 tip 复跑，不提前消耗全量语料跑）。D2 维持待拍板 + 低成本轮询。

### 17:52 轮询追记：s176 落地，10F 同键未清，节奏放缓

- s176 17:45 `3bbc7ca56`（shadow 域定位选择器寻址全链路，shadow-slots
  LOST 20→2P/18F）；节奏 s175 14:48 → s176 17:45 ≈ 3h/轮（s173→s175
  为 ~1h15m/轮，s176 刀位为引擎侧 shadow 寻址物化重活，轮时拉长）。
- s176 自账 engine lib **3619P/10F**（3617 + 2 钉测；10F 基线同键：
  [s176 账](../../html-semantics-compat/evidence/2026-10-31-m2s176-shadow-addressing.md)）
  ——10F 未清；s177 队列仍是 shadow 域（命令式 attachShadow + 宿主
  childlist 变异），fetch/Request 簇（s 系候选③「恢复质量门禁基线」）
  仍未入队可见部分，ETA 未知。
- 期间并行流编辑/渲染簇 5+3 笔（typing-state/inline 盒 wrap/void caret/
  RTL/paint）经 `git pull --rebase` 吸入，均不触 fetch/Request 键；本 goal
  仍零代码变更，组合态 tip `3bbc7ca56`。D2 维持待拍板 + 低成本轮询。
- 18:40 s177 `bd040de1c`（多进程 js_worker 外链 CSS entries 化）为纯
  renderer/webview 轮——**engine 代码零 diff，engine lib 免跑**（
  [s177 账](../../html-semantics-compat/evidence/2026-10-31-m2s177-renderer-worker-css-entries.md)），
  10F 基线结构性延续；renderer lib 首立全量基线 354P/5F（含 renderer 侧
  fetch 契约族 3F，与 engine fetch 9F 同族异面）归 s 流；其下一步候选
  （forms 402F / interactive 44F / renderer page_scripts 2F）仍非 fetch 簇。

### 19:45 轮询追记：s179 进 fetch 簇清零刀——10F → 2F，收口路径显形

- s179 19:38 `a3768d3e7`（fetch 契约族 16 红勘定：M2-S3 构造校验链测试面
  三代际对齐 A/B/C 分类 + `Response.clone` 丢 Set-Cookie 真实 shim 根修，
  [s179 账](../../html-semantics-compat/evidence/2026-10-31-m2s179-fetch-contract-family.md)）
  ——**engine lib 3625P/10F → 3633P/2F**（fetch 族 8 转绿），browser bin
  tab_js_worker 39P/0F、renderer lib 361P/0F 全绿。
- 余 2F 归属清楚：①`test_iframe_service_worker_messages_dispatch_to_
  iframe_container`（iframe XHR credentialed-open，真实 shim bug，s 系
  **下刀位 ⑥**，根修方向已记 XHR spec §4.3）；②`test_ua_display_li_
  list_item_canvas_inline_s24_flip`（渲染流 UA sheet li/canvas 翻转钉测，
  s 系记「渲染流在册/style-system 跨流」不越界）。
- D2①（fetch/Request 簇清零→当场收口）进入射程：s180 落地 iframe XHR
  根修 + 渲染流清 ua_display_li 后 make test 即应全绿。期间继续低成本
  轮询；本 goal 仍零代码变更，组合态 tip `a3768d3e7`。

### 20:12 追记：fetch 簇全清 + 残余 1F 精确定性 + 锚复跑启动

- s180 19:50 `2e260d532`（XHR credentialed-open 根修——URL 凭证 send 层
  提取为 Basic Authorization）：**engine lib 3635P/1F**（fetch 族全清，
  唯余 ua_display_li；renderer lib 361P/0F、browser bin 39P/0F）。
- **本流独立复测交叉核证**（`make guarded-test`，tip `a3768d3e7`+s181）：
  engine lib **3635P/1F** 与 s180 自账一致；唯一 FAIL 输出——
  `li 计算 display 应为 'list-item'` 实得 `"block"`（part32.rs:1029）。
- **残余 1F 根因定性**（git 追溯）：slice25 `af677d7f0`（10-03，渲染流）
  翻 shim 面 `_zwUaDisplay` li→list-item 时钉测绿（彼时裸 createElement
  无 host 通道，回落 shim 表）；s161 `ee2dc953b`（10-30 23:21，s 系）新增
  handle gCS 通道（`__zw_get_computed_style_by_handle`）后，裸 createElement
  产物改路由 host 面——host 面 `ua_default_display("li")=Block` 系渲染流
  **D4 在册待裁决冻结值**（style-system `ua_display_tests.rs` D4 注：
  「双面裁决前冻结 host 现状，未来修复须同提交双面翻转并带
  reftest/product-smoke 门控」）。即残余 1F = 跨流通道交互 × 渲染流
  D4 待裁决面，非任何单流简单欠账；修复归属渲染流 D4 裁决落地（双面
  同提交翻转），ETA 未知（用户裁决门禁，本 goal 不越界）。
- **webcrypto 锚复跑完成（tip `6eb020fc2`，20:15）**：release runner 重建
  35s + test-guard 包裹全语料跑（`--json`，产物 7.2MB）——对 M1 终态锚
  （m6-residual `2026-10-31-m1-baseline-full.json.gz`）**逐字节一致
  （cmp 全等）**：123 文件 / 25,788P / 7,320F / 1T 逐位相等。今日
  s161~s181 全部并行 shim/引擎变更（gCS handle 通道、clone Set-Cookie
  根修、XHR credentialed-open 根修、typing-state、inline 盒 wrap、void
  caret、RTL、local-attachment 等）后 **33,109 分母零回归锚成立**；
  runner EXIT=1 为带预期 7,320F 挂账的常规退出（log 无 guard 击杀）。
  产物：`/tmp/webcrypto-anchor-run-20261031-2010.json`（临时审计件，
  不入库）。clippy 复验同 tip 完成（20:20）：`make guarded-clippy`
  （`cargo clippy --workspace --all-targets -- -D warnings`，24G scope）
  **EXIT=0 全绿**。
- **收口前置就绪度（tip `6eb020fc2` + 本流 docs）**：DC-1~4 ✅（5894374c1
  在档）+ 锚逐字节一致 ✅ + clippy ✅ + 独立复测 3635P/1F ✅——唯余
  ua_display_li 1F（渲染流 D4 待裁决冻结面）。ua_display_li 清零后收口
  序列 = make test 全绿复证 → 锚按最终 tip 复跑（若 runner 二进制面未
  变可引用本锚账）→ 归档迁移 + README/csv/脚本登记 → 推送 → DONE。

### 2026-11-01 03:10 追记：锚现 tip（dc49d1e7c）复验逐字节一致——s182~s194 零回归核证

- **锚复跑（tip `dc49d1e7c`）**：release runner 重建 32.92s（make
  `zero-wpt-runner-release`）+ test-guard 直包全语料跑
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json` stdout
  重定向取干净产物）——**对 M1 终态锚
  （`2026-10-31-m1-baseline-full.json.gz`）逐字节一致（cmp 全等，
  7,208,436 B）**：123 文件 / 33,109 子测试 = 25,788P + 7,320F + 1T。
  guard log 零字节（无击杀），runner EXIT=1 为带预期 7,320F 挂账的
  常规退出。临时审计件 `/tmp/webcrypto-anchor-run-20261101-0308.json`
  （不入库）；链路核证：上轮 `20261031-2010` 审计件与 M1 gz 亦逐字节
  一致，锚链连续。
- **覆盖窗口**：`6eb020fc2` → `dc49d1e7c` 间 s182~s194（s185
  form.elements 集合层、s186 click 分派、s187/s188 color/frame、s189
  range、s190 date/time 消毒、s192 input.list、s193 fieldset
  willValidate、s194 email/url/tel 消毒 + IDN、CE entry 快路、
  removeChild 单遍化等引擎/shim 变更）+ 渲染流 R5368/R5369（float
  收缩适配/嵌套 float clear 预测等布局修复）——全部在语料执行路径上，
  **33,109 分母零回归锚在现 tip 重新成立**（上轮锚账 `6eb020fc2`
  不再是最新 tip，本次刷新消掉陈旧窗口）。
- **轮询面（本轮）**：s194 账 engine lib **3642P/1F（在册同键
  ua_display_li）**；渲染流 R5368（2026-11-01）记账 make test 既有红
  中止（ua_display_li，对端 D4 残余账）→ workspace 腿 --skip 补跑
  ——**D4 双面翻转仍未裁决**。D1/D2 维持待用户拍板
  （GB-20261031 征询在案），本 goal 零代码变更不变。
- **收口序列不变**：ua_display_li 清零（渲染流 D4 裁决落地）→ make
  test 全绿复证 → 锚按届时最终 tip 复跑（若 runner 二进制面未再变可
  引用本次账）→ 归档迁移 + README/csv/脚本登记 → 推送 → DONE；或
  D2② 授权 standing exclusion 口径收口。

### 2026-11-01 03:40 追记：锚随 tip 前移复验（e4e4ebc09）——justify shim 簇吸收

- **锚复跑（tip `e4e4ebc09`，release runner 重建 24.29s）**：test-guard
  直包同参数——对 M1 终态锚**逐字节一致（cmp 全等，7,208,436 B）**，
  guard log 零字节，EXIT=1 为预期挂账常规退出。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-0335.json`（不入库）。
- **动机**：`b171ddbf9` justify 首簇新增 shim `part01c.js`（+1290 行）+
  `part06.js` + bridge ——语料执行路径面，其自证门禁（justifycenter/
  bold 真页 + engine lib 零位移）未覆盖 webcrypto 语料；本次复跑消掉
  dc49d1e7c→e4e4ebc09 的锚窗口。DC-1~4 证据链在位性同步复核（四件
  evidence 均在 `5894374c1` 提交，工作树干净）。
- **轮询面无变化**：远端无新 s 系/渲染流推进；ua_display_li（渲染流
  D4）与 D1/D2 维持原状。

### 2026-11-01 03:50 追记：轮询零变化——锚账按代码面同一性延用，飞书回读通道证伪

- **轮询面**：远端零新提交（tip 维持 `463ad8642`），树 clean；
  `e4e4ebc09..463ad8642` 仅 `f2e90d44d`（wptfyi 快照 cron）与本 goal
  docs，**零代码变更**——e4e4ebc09 的锚逐字节一致账按代码面同一性
  延用，本轮不复跑。快照 cron 对 WebCrypto 行数字零位移（diff 无
  +/- 触及 WebCrypto 行），DC-4 引用口径不变。
- **D4/D1/D2 原状**：渲染流 R5369（2026-11-01 02:15）账面仍列
  ua_display_li D4 为对端既有待裁决，未落地；仓库无用户决策记录。
- **飞书回读通道证伪（后续轮次不再重试）**：D1/D2 征询回复无法无人
  值守读取——bot 身份 `+messages-mget` 对已发 P2P 消息返回空、
  `+messages-search` 缺 `search:message` scope（app 未申请）、user
  身份读 P2P 需交互式 device-code 授权。回复核验通道收敛为 git
  账面 + 用户主动知会两条。

### 2026-11-01 03:55 追记：锚账延用窗口随 tip 前移至 f6c507297——82eb31b67 为 #[cfg(test)] 面，零二进制影响

- 本流 docs 提交（`f6c507297`）rebase 时吸收并行流 `82eb31b67`
  （wpt-runner legacy-mb chars 语料缺失假红修复）：改动全部位于
  `wpt_file_loader.rs:402` `#[cfg(test)] mod tests` 内（删 checked
  计数与尾断言），不进 runner 二进制、不在语料执行路径；legacy-mb
  chars 族与 WebCryptoAPI 语料（.any.js/.window.js testharness）亦非
  同一加载通道。e4e4ebc09 的锚逐字节一致账按二进制面同一性延用至现
  tip `f6c507297`，本轮不复跑。远端同步（origin/main =
  `f6c507297`），树 clean。

### 2026-11-01 04:00 追记：锚复跑现 tip（7065ea39f）逐字节一致——s195 engine 变更吸收

- 03:55 追记落账后远端再进 `88b1904f1`（s195 focus 时序域 radio 焦点
  导航，engine/shim 语料执行路径代码）——按纪律对现 tip `7065ea39f`
  复跑锚：release runner 重建 26.92s + test-guard 直包
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json` 重
  定向）——**对 M1 终态锚逐字节一致（cmp 全等，7,208,436 B）**，
  guard log 零字节，runner EXIT=1 为预期挂账常规退出。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-0359-7065ea39f.json`（不入库）。
- 至此锚窗口推进为 **e4e4ebc09 → 7065ea39f**（s182~s195 全部引擎/
  shim 变更 + 渲染流 R5368/R5369 + wpt-runner test 面修复全吸收），
  33,109 分母零回归在现 tip 成立。轮询面其余无变化：ua_display_li
  （渲染流 D4）与 D1/D2 维持待拍板。

### 2026-11-01 04:35 追记：锚窗口随 rebase 前移至 1b6b9b291 逐字节一致——s196 engine perf 变更吸收

- 04:25 追记落账 push 前的 `git pull --rebase` 吸入并行流两笔：
  `7d172a211`（s196 `_zwMEl` 安装面共享化，engine/shim 语料执行路径
  perf 变更，自证七语料全零位移）+ `ddc8668c4`（docs 维护轮，零代码）
  ——代码面再次前移，按纪律对现 tip `1b6b9b291` 复跑锚：release
  runner 重建 25.24s + test-guard 直包同参数——**对 M1 终态锚逐字节
  一致（cmp 全等，7,208,436 B）**，guard log 零字节，EXIT=1 为预期
  挂账常规退出。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-s196-1b6b9b291.json`（不入库）。
- 至此锚窗口推进为 **7065ea39f → 1b6b9b291**（R5370 + s196 + docs
  维护轮全吸收），33,109 分母零回归在现 tip 成立。轮询面无变化：
  ua_display_li（渲染流 D4，s196 自账 engine lib 3642P/1F 同键）与
  D1/D2 维持待拍板。

### 2026-11-01 04:25 追记：锚复跑现 tip（650eab54e）逐字节一致——渲染流 R5370 layout 变更吸收

- `git pull --rebase` 吸入渲染流 R5370 两笔：`7c2826ffd`（table/
  inline-table 内 OOF 子不生成匿名 cell + 不入空 grid 收缩，
  layout-engine 语料执行路径代码 + 常驻测试 2pin）+ `650eab54e`
  （R5370 轮记账 docs）——代码面前移，按纪律对现 tip `650eab54e`
  复跑锚：release runner 重建 28.61s + test-guard 直包
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）——**对 M1 终态锚逐字节一致（cmp 全等，7,208,436 B）**，
  计数复核 123 文件 / 25,788P + 7,320F + 1T = 33,109；guard log
  零字节，runner EXIT=1 为预期挂账常规退出。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-r5370-650eab54e.json`（不入库）。
- 至此锚窗口推进为 **7065ea39f → 650eab54e**（R5370 table OOF child
  排除 + r5370 钉测全吸收），33,109 分母零回归在现 tip 成立。
- **轮询面**：R5370 自账 make test workspace 腿 **19250P/0F** 仍以
  `--skip ua_display_li` 跑（对端 D4 既有红 stash 同位复现）——
  **ua_display_li D4 双面翻转仍未裁决**，D1/D2 维持待拍板
  （GB-20261031 征询在案，核验只认 git 账面 + 用户主动知会）。
  收口序列不变：ua_display_li 清零 → make test 全绿复证 → 锚按届时
  最终 tip 复跑（若 runner 二进制面未再变可引用本次账）→ 归档迁移
  + README/csv/脚本登记 → 推送 → DONE；或 D2② 授权 standing
  exclusion 口径收口。

### 2026-11-01 05:31 追记：轮询轮——锚账按二进制面同一性延用至 f6c42dd03，docs-only 三笔零代码

- 轮询核验：锚 tip `1b6b9b291` → 现 tip `f6c42dd03` 三笔（`811ee5192`
  / `674a4112f` / `f6c42dd03`，渲染流 R5371 定谳记账 + 1101 竞速重校
  + R5371 四案定谳）`git diff --stat` 全为 docs 面（CHANGELOG /
  README / ROADMAP / goal docs 共 7 文件 35+/11−，无 .rs、无
  Cargo.toml、不在 runner 或语料执行路径）——runner 二进制面未变，
  锚逐字节一致账（`1b6b9b291`，2026-11-01 04:35 复跑）**按二进制面
  同一性延用至 `f6c42dd03`，本轮不复跑**（`f6c507297` docs-only
  先例同判）。远端同步（origin/main = `f6c42dd03`），树 clean。
- 轮询面无变化：渲染流 R5371 为零代码定谳轮（IFC float 行缩短深域
  等四案挂账），ua_display_li（D4 双面翻转）在 R5368~R5370 门禁账
  仍以 `--skip` 既有红在册，D4 裁决与 D1/D2 拍板均未见 git 账面。
  收口序列不变：ua_display_li 清零 → make test 全绿复证 → 锚按届时
  最终 tip 处置 → 归档迁移 + README/csv/脚本登记 → 推送 → DONE；
  或 D2② 授权 standing exclusion 口径收口。

### 2026-11-01 06:05 追记：轮询轮——锚账延用至 42e662bf5，docs-only 两笔零代码；DC-1~4 证据复核在位

- 轮询核验：锚 tip `1b6b9b291` → 现 tip `42e662bf5` 五笔全 docs 面
  （`811ee5192` / `674a4112f` / `f6c42dd03` 已账 + 本轮新增两笔：
  `a63f2f263` 上轮台账 2 文件 22+/3−、`42e662bf5` learnings 索引
  重建 + skill 更新 2 文件 15+/3−，无 .rs、无 Cargo.toml、不在
  runner 或语料执行路径）——runner 二进制面未变，锚逐字节一致账
  （`1b6b9b291`，04:35 复跑）**按二进制面同一性延用至
  `42e662bf5`，本轮不复跑**。远端同步（origin/main =
  `42e662bf5`），树 clean。
- DC-1~4 证据在位复核（应用户要求逐项核对文件与关键数字）：DC-1
  `2026-10-31-m1-corpus-and-baseline.md`（pin 全量 202 blob / 198
  文件在库 0 失败）；DC-2 `2026-10-31-dc2-promotion-screening.md`
  （上游 `cb6cc97a`，`.tentative.` 50→0）；DC-3 同 M1 文档 + 原始
  账 `2026-10-31-m1-baseline-full.json.gz`（25,788P+7,320F+1T，对
  一期终态锚 `archive/webcrypto-compat/evidence/2026-10-17-m6-residual-full.json.gz`
  状态变化 0 例）；DC-4 `2026-10-31-dc4-coverage-attribution.md`
  （与官方快照逐位一致，差值 12,541P + 265 文件逐桶归因）。四件
  证据文件 + 里程碑归档 `archive/2026-10-31-m1-m2.md` 均在库。
- 轮询面无变化：`42e662bf5` 为 learnings 维护轮零代码；渲染流无
  li→Block 双面翻转落地（`git log --grep` 全账仅 ledger 记账，
  R5370 门禁仍 `--skip ua_display_li` 对端 D4 既有红），D4 裁决与
  D1/D2 拍板均未见 git 账面。收口序列不变。

### 2026-11-01 06:20 追记：锚复跑现 tip（65de5f28e）逐字节一致——editing 流 justify 查询面兜底变更吸收

- `git pull --rebase` 吸入 editing 流 `65de5f28e`（justify 查询面
  uplink 断裂兜底——setter 孤儿 wrapper 读数链修复，`js_dom_shim/
  part01c.js` +105 行，jr/jf +220P 各零回归）：shim 面语料执行路径
  代码，runner 二进制面前移，按纪律对现 tip 复跑锚——release runner
  重建 25.71s（engine 重编译入包）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）——**对 M1 终态锚逐字节一致（cmp 全等，7,208,436 B）**，
  计数复核 123 文件 / 25,788P + 7,320F + 1T = 33,109；guard log
  零字节（无击杀），runner EXIT=1 为预期挂账常规退出。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-justify-65de5f28e.json`（不入库）。
- 至此锚窗口推进为 **1b6b9b291 → 65de5f28e**（上轮台账 + justify
  查询面兜底全吸收），33,109 分母零回归在现 tip 成立。
- **轮询面**：justify 变更自账 engine lib **3,642P/1F**（s24 已知红
  同键逐名 identical，[其账](../../editing-compat/evidence/2026-11-01-justify-query-face-orphan-uplink.md)）
  ——ua_display_li 未清，渲染流 D4 双面翻转仍未裁决；D1/D2 维持
  待拍板。DC-1~4 四件 evidence 在位复核（均系 `5894374c1` 提交，
  后续无触及）。收口序列不变：ua_display_li 清零 → make test 全绿
  复证 → 锚按届时最终 tip 处置 → 归档迁移 + README/csv/脚本登记
  → 推送 → DONE；或 D2② 授权 standing exclusion 口径收口。

### 2026-11-01 06:27 追记：锚复跑现 tip（b0a6f9174）逐字节一致——html-semantics 流 s196 `:active` 伪类通道吸收

- `git pull --rebase` 吸入 html-semantics 流 `b0a6f9174`（s196
  `:active` 伪类通道——dom `active_element` 槽 + engine
  `js_dom_bridge`/`selector_match` 查询面注入 + shim `part04.js`
  +8/`part06.js` +104 行，定向 8F 翻绿 3 + flake 域 3）：shim +
  查询/选择器匹配面系语料执行路径代码，runner 二进制面前移，按
  纪律对现 tip 复跑锚——release runner 重建 30.59s + test-guard
  直包同参数（`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，
  `--json` 重定向，跑时 1m26s）——**对 M1 终态锚逐字节一致
  （cmp 全等，7,208,436 B）**，计数复核 123 文件 / 25,788P +
  7,320F + 1T = 33,109；guard log 零字节（无击杀），runner EXIT=1
  为预期挂账常规退出。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-s196.json`（不入库）。
- 至此锚窗口推进为 **65de5f28e → b0a6f9174**，33,109 分母零回归
  在现 tip 成立。
- **轮询面**：s196 自账 zero-engine **3,642P/1F**（ua_display_li
  同键在册，[其账](../../html-semantics-compat/evidence/2026-11-01-m2s196-active-pseudo-channel.md)）
  + zero-dom 880P/0F——`:active` 通道非 li→Block 双面翻转，渲染流
  D4 仍未裁决；git 无 D1/D2 决策新账面，维持待拍板。收口序列不变：
  ua_display_li 清零 → make test 全绿复证 → 锚按届时最终 tip 处置
  → 归档迁移 + README/csv/脚本登记 → 推送 → DONE；或 D2② 授权
  standing exclusion 口径收口。

### 2026-11-01 07:15 追记：锚复跑现 tip（`3f1399195`）逐字节一致——渲染流 R5372 abspos 嵌套 CB margin 变更吸收

- `git pull --rebase` 吸入渲染流 R5372 两笔：`9bef8608a`（abspos.rs
  `resolve_abspos_against_nested_cb_inner_m` 长度 inset 锚定写点补
  margin 项——CSS2 §10.3.7/§10.6.4 定位方程，layout-engine 语料执行
  路径代码 +31 行 + 常驻测试 `r5372_abspos_nested_cb_margin_tests`
  7pin）+ `3f1399195`（R5372 轮记账 docs）——代码面前移，按纪律对现
  tip `3f1399195` 复跑锚：release runner 重建 29.75s（layout-engine
  重编译入包）+ test-guard 直包同参数（`--per-proc-mem 4
  --total-mem 8 --time-limit 1800`，`--json` 重定向，跑时 1m28s）
  ——**对 M1 基线锚逐字节一致（cmp 全等，7,208,436 B）**，计数复核
  123 文件 / 25,788P + 7,320F + 1T = 33,109；guard log 零字节
  （无击杀），runner EXIT=1 为预期挂账常规退出。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-r5372-3f1399195.json`（不入库）。
- 至此锚窗口推进为 **b0a6f9174 → 3f1399195**（s196 后 docs 两笔按
  二进制面同一性延用 + R5372 两笔实跑吸收），33,109 分母零回归在现
  tip 成立。实锚链 7065ea39f→1b6b9b291→65de5f28e→b0a6f9174→3f1399195。
- **轮询面**：R5372 自账 make test workspace **3642P/1F** = ua_display_li
  D4 既有账（stash 干净 main 同位复现）——abspos margin 变更非
  li→Block 双面翻转，渲染流 D4 仍未裁决；git 无 D1/D2 决策新账面，
  维持待拍板（GB-20261031 征询在案，核验只认 git 账面 + 用户主动
  知会）。DC-1~4 四件 evidence 在位（后续无触及）。收口序列不变：
  ua_display_li 清零 → make test 全绿复证 → 锚按届时最终 tip 处置
  → 归档迁移 + README/csv/脚本登记 → 推送 → DONE；或 D2② 授权
  standing exclusion 口径收口。

### 2026-11-01 07:50 追记：锚复跑现 tip（`ba4c74fcb`）状态账逐位一致 ×3——html-semantics 流 s197 fieldset disabled 焦点域吸收 + W4 超时诊断臂竞态首采

- `git pull --rebase` 吸入 html-semantics 流 `ba4c74fcb`（s197
  fieldset disabled 置位焦点内含检查——shim `part03/04/05/06.js` 净
  +110/−4 行写通道钩，[其账](../../html-semantics-compat/evidence/2026-11-01-m2s197-fieldset-disable-focus-reset.md)）：
  shim 系语料执行路径代码，runner 二进制面前移，按纪律对现 tip 复跑
  锚——release runner 重建 25.55s（engine/shim 重编译入包）+
  test-guard 直包同参数（`--per-proc-mem 4 --total-mem 8
  --time-limit 1800`，`--json` 重定向）三连跑，guard log 零字节
  （无击杀），runner EXIT=1 为预期挂账常规退出。
- **状态账 33,109 分母逐 (case, subtest) 对照零差异 ×3**（键集全等 +
  status 逐位相等；123 文件 / 25,788P + 7,320F + 1T）；RUN2/RUN3 对
  M1 终态锚**逐字节一致（cmp 全等，7,208,436 B）**，墙钟 1m27.3s /
  1m26.5s 与历史持平。
- **新发现（W4 追加定性，不改变挂账归属）**：RUN1 首采
  `algorithm-discards-context.https.window.js` 超时诊断走 **deadline
  臂**（`testharness completion callback was not called (..., phase=2,
  test_wait=false)`——`CORPUS_CASE_TIMEOUT` 30s 燃尽击杀），RUN2/RUN3
  回落既有 **completion 臂**（phase=4 空结果，与锚逐字节）。两臂均为
  runner 既有代码路径（`testharness.rs` deadline 检查 vs completion
  空结果臂），30s 收敛边界竞态，本 tip 采样频率 1/3，此前锚链 6+ 连续
  采样未见翻转；归因未定谳（s197 全局写通道钩开销推移分布 vs 既有低频
  竞态首次命中）。全语料墙钟无净变化，两臂均记 Timeout 同键，状态账
  不受影响，**W4 定性维持**（runner/shim DOM 域边界——iframe 跨帧
  注入 + `top` 回调链），33,109 零回归底线无损。**后续锚验收口径**：
  状态账逐位一致为主判据；逐字节一致在采样复现时并列记录（RUN1 型
  竞态翻转不构成回归）。
- 至此锚窗口推进为 **3f1399195 → ba4c74fcb**，实锚链 7065ea39f→
  1b6b9b291→65de5f28e→b0a6f9174→3f1399195→ba4c74fcb，33,109 分母
  零回归在现 tip 成立。
- **轮询面**：s197 自账 zero-engine **3,642P/1F**（ua_display_li 同键
  在册）+ zero-dom 886P/0F 逐位——fieldset 焦点域变更非 li→Block
  双面翻转，渲染流 D4 仍未裁决（渲染流 rally 进程未在跑，D4 落地依赖
  其重启，已飞书补充告知）；git 无 D1/D2 决策新账面，维持待拍板
  （GB-20261031 征询在案，核验只认 git 账面 + 用户主动知会）。DC-1~4
  四件 evidence 在位（后续无触及）。收口序列不变：ua_display_li 清零
  → make test 全绿复证 → 锚按届时最终 tip 处置 → 归档迁移 +
  README/csv/脚本登记 → 推送 → DONE；或 D2② 授权 standing exclusion
  口径收口。临时审计件
  `/tmp/webcrypto-anchor-run{,2,3}-20261101-s197.json`（不入库）。

### 2026-11-01 08:35 追记：锚复跑现 tip（`edaed928a`）状态账逐位一致 ×3——渲染流 R5373 父高度回填塌缩屏障变更吸收

- `git pull --rebase` 吸入渲染流 R5373 单笔 `edaed928a`（R1743 父高度
  回填 extent 计入塌缩屏障——`postprocess.rs` R1743 fold 单写点按
  pb/bb 框边屏障二选一，layout-engine 语料执行路径 +36/−10 行 +
  常驻测试 `r5373_backfill_barrier_margin_tests` ×4（+151 行）+
  rendering-compat 记账 docs；[其账](../../rendering-compat/evidence/r5373-backfill-barrier-margin-2026-11-01.md)）：
  代码面前移，按纪律对现 tip 复跑锚——release runner 重建 31.4s
  （layout-engine 重编译入包，`make zero-wpt-runner-release`）+
  test-guard 直包同参数（`--per-proc-mem 4 --total-mem 8
  --time-limit 1800`，`--json` 重定向）三连跑，墙钟 94.5s / 92.0s /
  110.8s，guard log 无击杀（仅 /usr/bin/time 摘要 50-51 B），runner
  EXIT=1 为预期挂账常规退出。
- **状态账 33,109 分母逐 (case, subtest) 对照零差异 ×3**（键集全等 +
  status 逐位相等；123 文件 / 25,788P + 7,320F + 1T）；RUN2/RUN3 对
  M1 终态锚**逐字节一致（cmp 全等，7,208,436 B）**；RUN1 首采 W4 案
  deadline 臂（`phase=2`「completion callback was not called」）
  竞态翻转 1/3——与 07:50 首采同款 runner 既有两臂竞态（归因未定谳，
  状态账零差异），按 07:50 修订验收口径**不构成回归**（状态账逐位为
  主判据，逐字节在采样复现时并列记录）。
- 至此锚窗口推进为 **ba4c74fcb → edaed928a**，实锚链 7065ea39f→
  1b6b9b291→65de5f28e→b0a6f9174→3f1399195→ba4c74fcb→edaed928a，
  33,109 分母零回归在现 tip 成立。
- **轮询面**：R5373 自账 make test workspace **3642P/1F** = ua_display_li
  D4 既有账同位同签名（渲染流同树自验）+ 渲染 corpus 760F→757F
  FIXED=3 REGRESSED=∅ + layout-engine lib 2306P/0F + bench-gate 定向
  GATE PASS——**渲染流 rally 已重启在跑**（R5373 落地即证），但冻结门
  `ua_display_tests.rs` 末触仍 `c41b0c750`（D4 双面翻转未落地）；
  p126 未回流 main（merge-base 核证）；git 无 D1/D2 决策新账面，维持
  待拍板（GB-20261031 征询在案，核验只认 git 账面 + 用户主动知会）。
  DC-1~4 四件 evidence 在位（后续无触及）。收口序列不变：ua_display_li
  清零 → make test 全绿复证 → 锚按届时最终 tip 处置 → 归档迁移 +
  README/csv/脚本登记 → 推送 → DONE；或 D2② 授权 standing exclusion
  口径收口。临时审计件
  `/tmp/webcrypto-anchor-run{,2,3}-20261101-r5373.json`（不入库）。
