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

### 2026-11-01 08:50 追记：锚复跑现 tip（`0d4966f9d`）状态账逐位一致 ×3——html-semantics 流 s198 fieldset type IDL 变更吸收

- push 前例行 `git pull --rebase` 吸入 html-semantics 流 `0674243a1`
  （s198 HTMLFieldSetElement.type IDL 常量臂——shim `part04.js` +5 行
  写通道，[其账](../../html-semantics-compat/evidence/2026-11-01-m2s198-fieldset-type-idl.md)）：
  shim 系语料执行路径代码，runner 二进制面前移，本流锚账
  `8604b13a7`（R5373 吸收，rebase 后为 `0d4966f9d`）先推送，随即按
  纪律对新 tip 复跑锚——release runner 重建 32.4s（shim 重编译入包，
  `make zero-wpt-runner-release`）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）三连跑，guard log 无击杀（仅 /usr/bin/time 摘要 50 B），
  runner EXIT=1 为预期挂账常规退出。
- **状态账 33,109 分母逐 (case, subtest) 对照零差异 ×3**（键集全等 +
  status 逐位相等；123 文件 / 25,788P + 7,320F + 1T）；RUN1/RUN3 对
  M1 终态锚**逐字节一致（cmp 全等，7,208,436 B）**；RUN2 首采 W4 案
  deadline 臂（`phase=2`）竞态翻转 1/3——与 07:50（RUN1 型）/08:35
  （RUN1 型）采样同款 runner 既有两臂竞态（归因未定谳，状态账零差异），
  按 07:50 修订验收口径**不构成回归**。采样面累计 2/9 轮命中该臂
  （s197/R5373/s198 三 tip 各 3 跑），维持低频竞态定性、不改变挂账
  归属。
- 至此锚窗口推进为 **edaed928a → 0d4966f9d**，实锚链 7065ea39f→
  1b6b9b291→65de5f28e→b0a6f9174→3f1399195→ba4c74fcb→edaed928a→
  0d4966f9d，33,109 分母零回归在现 tip 成立。
- **轮询面**：s198 自账 zero-engine **3,642P/1F**（ua_display_li 同键
  在册）+ the-fieldset-element 目录 22P/0F——fieldset type IDL 非
  li→Block 双面翻转，渲染流 D4 仍未裁决（渲染流 rally 已重启在跑但
  冻结门 `ua_display_tests.rs` 末触仍 `c41b0c750`）；p126 未回流
  main；git 无 D1/D2 决策新账面，维持待拍板（GB-20261031 征询在案，
  核验只认 git 账面 + 用户主动知会）。DC-1~4 四件 evidence 在位
  （后续无触及）。收口序列不变：ua_display_li 清零 → make test 全绿
  复证 → 锚按届时最终 tip 处置 → 归档迁移 + README/csv/脚本登记 →
  推送 → DONE；或 D2② 授权 standing exclusion 口径收口。临时审计件
  `/tmp/webcrypto-anchor-run{,2,3}-20261101-s198.json`（不入库）。

### 2026-11-01 09:09 追记：锚复跑现 tip（`f05b0efc1`）逐字节一致 ×3——net-api 流 S19 Headers 大小写保真变更吸收

- push 前例行 `git pull --rebase` 吸入 net-api 流 `f05b0efc1`（二期归档
  重入 S19：Headers 实例/XHR 异步源头名大小写线上保真补全，shim
  `part01.js` +31 行 / `part03.js` +6 行——`_hCase` 旁路四处补齐
  fetch init/Request-like input/Request ctor/XHR 异步通道，
  [其账](../../archive/net-api-compat-v2/evidence/2026-11-01-m2-s19-header-case-fidelity-reentry.md)）：
  shim 系语料执行路径代码，runner 二进制面前移，按纪律对新 tip 复跑
  锚——release runner 重建 25.8s（shim 重编译入包，
  `make zero-wpt-runner-release`）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）三连跑，guard log 零字节（无击杀），runner EXIT=1 为预期
  挂账常规退出。
- **RUN1/RUN2/RUN3 对 M1 终态锚逐字节一致（cmp 全等，7,208,436 B
  ×3）**——较 07:50（s197）/08:35（R5373）/08:50（s198）三轮
  （状态账逐位一致 + W4 deadline 臂竞态各采 1/3）更强：本轮三跑整
  文件字节全等、W4 竞态零采样（0/3）；123 文件 / 25,788P + 7,320F +
  1T；33,109 分母零回归在现 tip 成立。该提交自账 A/B 净账 0 增益/
  0 回归（net-api 面 12,078 行逐 subtest）与本流 33,109 分母面零
  交集——两侧零回归互证。
- 锚窗口推进为 **0d4966f9d → f05b0efc1**，实锚链 7065ea39f→
  1b6b9b291→65de5f28e→b0a6f9174→3f1399195→ba4c74fcb→edaed928a→
  0d4966f9d→f05b0efc1。
- **轮询面**：该提交仅触及 engine shim `part01/part03` + engine
  测试（`js_dom_bridge_tests` part93）+ net-api 归档 docs，未触渲染
  域——D4 冻结门 `ua_display_tests.rs` 末触仍 `c41b0c750`（渲染流
  rally 在跑）；ua_display_li 同键在册未清（该提交自账 engine lib
  3,645P/1F，同键渲染域在册红，stash 同败非新账）；p126 未回流
  main；git 无 D1/D2 决策新账面，维持待拍板（GB-20261031 征询在案，
  核验只认 git 账面 + 用户主动知会）。DC-1~4 四件 evidence 在位。
  收口序列不变：ua_display_li 清零 → make test 全绿复证 → 锚按届时
  最终 tip 处置 → 归档迁移 + README/csv/脚本登记 → 推送 → DONE；
  或 D2② 授权 standing exclusion 口径收口。临时审计件
  `/tmp/webcrypto-anchor-run{,2,3}-20261101-netapi.json`（不入库）。

### 2026-11-01 09:20 追记：锚复跑现 tip（`e30ab8494`，代码 tip `a4f30d2a6`）逐字节一致 ×3——s199 + change-on-blur 变更吸收（上轮 rebase 窗口漏核补账）

- 复盘发现：上轮 push 前 rebase 吸入**两个语料执行路径代码提交**
  （07:05 `5bc4acc6d` zero-web 流 change-on-blur 双账去重闩——engine
  桥 + shim；09:04 `a4f30d2a6` html-semantics 流 s199 Space 键激活
  keydown preventDefault 抑制门——runner `testharness.rs` + shim
  `part04/part06` + `script_gen.rs`），上轮锚跑于 `f05b0efc1` 且
  push 前未复核 rebase 新基的代码前移，锚账与所推树存在窗口错位。
  本轮按纪律补账：对现 tip 复跑锚——release runner 重建 23.7s
  （runner/shim 重编译入包，`make zero-wpt-runner-release`）+
  test-guard 直包同参数（`--per-proc-mem 4 --total-mem 8
  --time-limit 1800`，`--json` 重定向）三连跑，guard log 零字节
  （无击杀），runner EXIT=1 为预期挂账常规退出。
- **RUN1/RUN2/RUN3 对 M1 终态锚逐字节一致（cmp 全等，7,208,436 B
  ×3）**：123 文件 / 25,788P + 7,320F + 1T；33,109 分母零回归在现
  tip 成立；W4 deadline 臂竞态连续第二轮零采样（累计 2/15 轮）。
  两提交自账零回归（s199 forms 全树 keyed diff F→P 恰 1 靶键零真
  P2F；change-on-blur 同一失焦只派一次 change）与本流 33,109 分母
  面零交集——两侧零回归互证。
- 锚窗口推进为 **f05b0efc1 → e30ab8494**，实锚链 ……→0d4966f9d→
  f05b0efc1→e30ab8494。流程补强：push 前 rebase 后必须重查
  `git diff <末锚tip>..HEAD -- crates/ tests/` 是否有执行路径前移，
  有则锚复跑先于 push（本轮起执行）。
- **轮询面**：渲染流 ZRG 巡检落档 ua_display_li bisect 定谳补充
  （[evidence](../rendering-compat/evidence/ua-display-li-bisect-attribution-2026-11-01.md)：
  first bad = `ee2dc953b` s161 gCS handle 通道，与本流 D2 归因一致；
  归属与收口路径不变——渲染流 D4 待裁决，D2 征询在案）；D4 冻结门
  `ua_display_tests.rs` 末触仍 `c41b0c750`；ua_display_li 同键在册
  未清；p126 未回流 main；git 无 D1/D2 决策新账面（GB-20261031 在
  案，核验只认 git 账面 + 用户主动知会）。收口序列不变。临时审计件
  `/tmp/webcrypto-anchor-run{,2,3}-20261101-s199blur.json`（不入库）。

### 2026-11-01 09:33 追记：锚复跑现 tip（`dd0eb2ec8`）状态账逐位一致 ×3——editing 流 sv-setup style IDL 变更吸收

- push 前例行 `git pull --rebase` 吸入 editing 流 `dd0eb2ec8`（物化元素
  补 style IDL——style-with-value 簇 setup 例外族收口，shim
  `part03.js` +136 行 + engine 测试 `part18.rs`，
  [其账](../../editing-compat/evidence/2026-11-01-sv-setup-exception-mstyle-idl.md)）：
  shim 系语料执行路径代码，runner 二进制面前移，按纪律对新 tip 复跑
  锚——release runner 重建 24.2s（shim 重编译入包，
  `make zero-wpt-runner-release`）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）三连跑，guard log 零字节（无击杀），runner EXIT=1 为预期
  挂账常规退出。
- **状态账 33,109 分母逐 (case, subtest) 对照零差异 ×3**（键集全等 +
  status 逐位相等；123 文件 / 25,788P + 7,320F + 1T）；RUN1/RUN3 对
  M1 终态锚**逐字节一致（cmp 全等，7,208,436 B）**；RUN2 首采 W4 案
  message 臂新变体（`completion callback was not called` 文本臂，
  非 phase=2 deadline 臂、非 phase=4 主臂）——程序化对照确认该案
  仅 message 字段差异、状态账零差异，与前几轮 deadline 臂同属
  runner 既有完成路径竞态（归因未定谳），按主判据**不构成回归**。
  采样面累计 3/18 轮（本轮 1/3，RUN2 型），维持低频竞态定性、不
  改变挂账归属。
- 锚窗口推进为 **e30ab8494 → dd0eb2ec8**，实锚链 ……→f05b0efc1→
  e30ab8494→dd0eb2ec8。该提交自账 style-with-value 簇 setup 例外
  族收口（editing 面），与本流 33,109 分母面零交集——两侧零回归
  互证。
- **轮询面**：D4 冻结门 `ua_display_tests.rs` 末触仍 `c41b0c750`
  （该提交仅触及 engine shim/测试 + editing docs，未触渲染域）；
  ua_display_li 同键在册未清；p126 未回流 main；git 无 D1/D2 决策
  新账面（GB-20261031 在案，核验只认 git 账面 + 用户主动知会）。
  收口序列不变。临时审计件
  `/tmp/webcrypto-anchor-run{,2,3}-20261101-svmstyle.json`（不入库）。

### 2026-11-01 09:43 追记：锚复跑现 tip（`578240175`）状态账逐位一致 ×3——html-semantics 流 s200 键盘激活 click-only 事件序吸收

- push 前例行 `git pull --rebase` 吸入 html-semantics 流 `578240175`
  （s200 键盘激活 click-only 事件序——uievents keyboard-click-event
  4 子测收口，runner `testharness.rs` +21/-4 + html-semantics docs，
  [其账](../../html-semantics-compat/evidence/2026-11-01-m2s200-keyboard-click-only-activation.md)）：
  runner 系语料执行路径代码，二进制面前移，按纪律对新 tip 复跑锚——
  release runner 重建 24.85s（`make zero-wpt-runner-release`）+
  test-guard 直包同参数（`--per-proc-mem 4 --total-mem 8
  --time-limit 1800`，`--json` 重定向）三连跑，guard log 零字节
  （无击杀），runner EXIT=1 为预期挂账常规退出。
- **状态账 33,109 分母逐 (case, subtest) 对照零差异 ×3**（键集全等 +
  status 逐位相等；123 文件 / 25,788P + 7,320F + 1T）；RUN2/RUN3 对
  M1 终态锚**逐字节一致（cmp 全等，7,208,436 B）**；RUN1 采样 W4 案
  message 臂（`completion callback was not called` 文本臂，与 09:33
  轮 RUN2 型同臂、非新变体）——程序化对照确认仅 message 字段差异、
  状态账零差异，属 runner 既有完成路径竞态（归因未定谳），按主判据
  **不构成回归**。采样面累计 4/19 轮（本轮 1/3，RUN1 型），维持低频
  竞态定性、不改变挂账归属。
- 锚窗口推进为 **dd0eb2ec8 → 578240175**，实锚链 ……→e30ab8494→
  dd0eb2ec8→578240175。s200 自账 forms 全树 keyed diff 零 P2F 零键
  增减（uievents 面），与本流 33,109 分母面零交集——两侧零回归互证。
- **轮询面**：D4 冻结门 `ua_display_tests.rs` 末触仍 `c41b0c750`
  （s200 仅触 runner + html-semantics docs，未触渲染域）；
  ua_display_li 同键在册未清；git 无 D1/D2 决策新账面（GB-20261031
  在案，核验只认 git 账面 + 用户主动知会）。收口序列不变。临时
  审计件 `/tmp/webcrypto-anchor-run{1,2,3}-20261101-s200key.json`
  （不入库）。

### 2026-11-01 10:05 追记：锚复跑现 tip（`8e84cb9a2`）三跑全 cmp 全等 ×3——zero-web 流 t-p126 softnav 簇吸收，W4 本轮零采样

- push 前例行 `git pull --rebase` 吸入 zero-web 流 PR #265（t-p126
  softnav busy-loop 簇落 main：R98 原型 memo 化 `554f668a7` + DEF-1/DEF-2
  返修 `a6f2c5acd` + rustfmt `87fa201cc` + 两笔 main 入线 merge）：
  执行路径 7 文件 +1127/−77（`js_dom_bridge.rs` +429、`callbacks.rs`
  +286、`css_wire.rs` ±18、shim `part03.js` +51、`part05.js` +6、
  `part94.rs` 测试 +413）——engine bridge/shim 系语料执行路径代码，
  二进制面前移，按纪律对新 tip 复跑锚。
- 并行流协调：起跑前 ps 查兄弟进程发现 editing 流 `forecolor` run
  在跑（同 runner 二进制路径），轮询等待其收尾（约 9.5 min）后再起
  三连跑，避免 RUN1/RUN2 中途二进制被并行流重建换面；起跑前
  `make zero-wpt-runner-release` 复验 no-op（0.18s）确认二进制对
  HEAD 现行（本流重建 25.48s 在前）。
- test-guard 直包同参数（`--per-proc-mem 4 --total-mem 8
  --time-limit 1800`，`--json` 重定向）三连跑：runner EXIT=1 为预期
  挂账常规退出，guard log 零字节（无击杀），墙钟 90/92/88s 与历史
  持平。
- **RUN1/RUN2/RUN3 对 M1 终态锚全部逐字节一致（cmp 全等，7,208,436
  B）且三跑互等**：123 文件 / 33,109 子测试 = 25,788P + 7,320F + 1T
  状态账零差异——t-p126 簇对 webcrypto 语料面完全吸收。本轮 W4 零
  采样，采样面累计 4/20 轮，维持低频竞态定性。
- 锚窗口推进为 **e2910e933 → 8e84cb9a2**，实锚链 ……→dd0eb2ec8→
  578240175→e2910e933→8e84cb9a2。
- **轮询面**：D4 冻结门 `ua_display_tests.rs` 末触仍 `c41b0c750`
  （PR #265 仅触 engine bridge/shim/tests，未触渲染域 style-system）；
  ua_display_li 同键在册未清；git 无 D1/D2 决策新账面（GB-20261031
  在案，核验只认 git 账面 + 用户主动知会）。收口序列不变。临时审计
  件 `/tmp/webcrypto-anchor-20261101-1003-{r1,r2,r3}.json`（不入库）。

### 2026-11-01 10:47 追记：锚复跑现 tip（`ebdfe3939`）状态账零差异 ×3 + RUN1/RUN3 逐字节一致——html-semantics 流 s201 SVG 形状 gBCR 命中注入吸收

- 本轮 `git pull --rebase` 吸入 html-semantics 流 `ebdfe3939`（s201 SVG
  形状 gBCR 命中注入——inert-svg-hittest 2/2 收口，engine
  `hit_test.rs` ±21 + 新 `hit_test_svg.rs` +310 + `lib.rs` +1，共 3 文件
  +331/−1，[其账](../../html-semantics-compat/evidence/2026-11-01-m2s201-svg-shape-gbcr-hit-test.md)）：
  engine 系语料执行路径代码，runner 二进制面前移，按纪律对新 tip 复跑
  锚——release runner 重建 36.8s（engine 重编译入包，
  `make zero-wpt-runner-release`）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）三连跑。
- 并行流协调：起跑前 ps 查兄弟进程发现 editing 流单案 run 在跑
  （`testharness-editing-other join-different-white-space-style`，10:24
  起、timeout 1500s 包裹），90s 间隔轮询至 10:39 收尾后再重建三连跑，
  避免中途重建换掉在跑二进制面。
- guard log 三跑零字节（无击杀），runner EXIT=1 为预期挂账常规退出，
  墙钟 93/88/89s 与历史持平。
- **状态账 33,109 分母逐 (case, subtest) 对照零差异 ×3**（键集全等 +
  status 逐位相等；123 文件 / 25,788P + 7,320F + 1T）；RUN1/RUN3 对
  M1 终态锚**逐字节一致（cmp 全等，7,208,436 B）**；RUN2 W4 案
  deadline 臂采样（`phase=2`「completion callback was not called」，
  非 phase=4 主臂）——程序化对照确认仅 message 字段差异、状态账零
  差异，与 07:50/09:33/09:43 各轮同属 runner 既有完成路径竞态（归因
  未定谳），按主判据**不构成回归**。采样面累计 5/23 轮（本轮 1/3，
  RUN2 型），维持低频竞态定性。
- s201 为 SVG 命中测试域新增，对 webcrypto 语料面完全吸收（零状态
  变化即证）；锚窗口推进为 **8e84cb9a2 → ebdfe3939**，实锚链
  ……→578240175→e2910e933→8e84cb9a2→ebdfe3939。
- **轮询面**：D4 冻结门 `ua_display_tests.rs` 末触仍 `c41b0c750`
  （s201 仅触 engine hit_test 域 + html-semantics docs，未触渲染域
  style-system）；ua_display_li 同键在册未清；PR #266 仍 OPEN 未合
  入；git 无 D1/D2 决策新账面（GB-20261031 在案，核验只认 git 账面
  + 用户主动知会）。收口序列不变。临时审计件
  `/tmp/webcrypto-anchor-20261101-s201svg-{r1,r2,r3}.json`（不入库）。

### 2026-11-01 11:07 追记：锚复跑现 tip（`a5a11cd07`）三跑全 cmp 全等 ×3——html-semantics 流 s202 legend/option form IDL 变更吸收，W4 本轮零采样

- 本轮 `git pull --rebase` 吸入 html-semantics 流 `a5a11cd07`（s202
  legend/option form IDL 反射——the-legend-element 5F + option-form
  1F 收口，shim `part03.js` +37/`part05.js` +32，
  [其账](../../html-semantics-compat/evidence/2026-11-01-m2s202-legend-option-form-idl.md)）：
  shim 系语料执行路径代码，runner 二进制面前移，按纪律对新 tip 复跑
  锚——release runner 重建 26.56s（shim 重编译入包，
  `make zero-wpt-runner-release`）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）三连跑，起跑前 ps 核查无兄弟 cargo/runner 在跑、两 clone
  target 目录各自独立（无跨库换面面）。
- guard log 三跑零字节（无击杀），runner EXIT=1 为预期挂账常规退出，
  墙钟 ~90s/跑与历史持平。
- **RUN1/RUN2/RUN3 对 M1 终态锚全部逐字节一致（gunzip 后 cmp 全等，
  7,208,436 B ×3）且三跑互等**：123 文件 / 33,109 子测试 = 25,788P +
  7,320F + 1T 状态账零差异（程序化计数复核 ×3）——s202 对
  webcrypto 语料面完全吸收。W4 本轮零采样，采样面累计 5/26 轮，
  维持低频竞态定性。
- 锚窗口推进为 **ebdfe3939 → a5a11cd07**，实锚链 ……→8e84cb9a2→
  ebdfe3939→a5a11cd07，33,109 分母零回归在现 tip 成立。
- **轮询面**：s202 自账 zero-engine **3,664P/1F**（ua_display_li 渲染
  流域在册键同位）；D4 冻结门 `ua_display_tests.rs` 末触仍
  `c41b0c750`（s202 仅触 engine shim + html-semantics docs，未触渲染
  域 style-system）；ua_display_li 同键在册未清；git 无 D1/D2 决策
  新账面（GB-20261031 在案，核验只认 git 账面 + 用户主动知会）。
  收口序列不变：ua_display_li 清零 → make test 全绿复证 → 锚按届时
  最终 tip 处置 → 归档迁移 + README/csv/脚本登记 → 推送 → DONE；
  或 D2② 授权 standing exclusion 口径收口。临时审计件
  `/tmp/webcrypto-anchor-20261101-s202legend-{r1,r2,r3}.json`
  （不入库）。

### 2026-11-01 11:32 追记：锚复跑现 tip（`66f4871c1`）三跑全 cmp 全等 ×3——html-semantics 流 s203 number 值消毒变更吸收，W4 本轮零采样

- 本轮 `git pull --rebase` 吸入 html-semantics 流 `66f4871c1`（s203
  number 值消毒文法收口——`_zwSanitizeInputValue` number 臂换 spec
  valid floating-point number 文法，shim `part01.js` +10/−1，
  [其账](../../html-semantics-compat/evidence/2026-11-01-m2s203-number-value-sanitization.md)）：
  shim 系语料执行路径代码，runner 二进制面前移，按纪律对新 tip 复跑
  锚——release runner 重建 25.66s（shim 重编译入包，
  `make zero-wpt-runner-release`）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）三连跑。
- **换面括查强化**：起跑时兄弟 run 在跑（`testharness-html-semantics`
  733% CPU，11:19 起跑）——`/proc` cwd 读被沙箱挡，改用二进制 mtime
  旁证归库（ZeroWeb-3 runner 11:07:25 建、ZeroWeb-2 的 10:58:23 系本流
  上轮重建产物，兄弟起跑晚于前者 → 归 ZeroWeb-3，target 目录独立）；
  另加二进制 sha256 括查（每跑后复验，三跑全程 `289d771e…` 未变）+
  锚 cmp 本身为终极换面探测器。guard log 三跑零字节（无击杀），
  runner EXIT=1 为预期挂账常规退出，墙钟 93s/90s/97s（兄弟 CPU 竞争
  下与历史持平）。
- **RUN1/RUN2/RUN3 对 M1 终态锚全部逐字节一致（gunzip 后 cmp 全等，
  7,208,436 B ×3）且三跑互等**：123 文件 / 33,109 子测试 = 25,788P +
  7,320F + 1T 状态账零差异（程序化计数复核）——s203 对 webcrypto
  语料面完全吸收。W4 案处预期 Timeout 臂，本轮零采样，采样面累计
  5/27 轮，维持低频竞态定性。
- 锚窗口推进为 **a5a11cd07 → 66f4871c1**，实锚链 ……→ebdfe3939→
  a5a11cd07→66f4871c1，33,109 分母零回归在现 tip 成立。
- **轮询面**：s203 自账 zero-engine **3,664P/1F**（ua_display_li 渲染
  流域在册键同位，commit 正文自证）；D4 冻结门 `ua_display_tests.rs`
  末触仍 `c41b0c750`（s203 仅触 engine shim + html-semantics docs）；
  git 无 D1/D2 决策新账面（GB-20261031 在案，核验只认 git 账面 +
  用户主动知会）。收口序列不变：ua_display_li 清零 → make test 全绿
  复证 → 锚按届时最终 tip 处置 → 归档迁移 + README/csv/脚本登记 →
  推送 → DONE；或 D2② 授权 standing exclusion 口径收口。临时审计件
  `/tmp/webcrypto-anchor-20261101-s203number-{r1,r2,r3}.json`
  （不入库）。

### 2026-11-01 11:44 追记：锚复跑现 tip（`a88ab50c6`）三跑全 cmp 全等 ×3——渲染流 R5374 abspos 表 available-size 钳 CB 吸收，W4 本轮零采样

- 期间数轮零变化轮询后，本轮 `git pull --rebase` 吸入渲染流
  `a88ab50c6`（R5374 abspos 表 available-size 钳 CB——css-tables-3
  §abspos/crbug 1122428，layout-engine table 域新增
  `clamp_abspos_table_available_width` 挂 real-table 臂 + kill-switch
  `ZW_ABSPOS_TABLE_AVAIL_CLAMP` + `test_r5374_*` ×2）：
  layout-engine 系语料执行路径代码，runner 二进制面前移，按纪律对新
  tip 复跑锚——release runner 重建 28.75s（layout-engine 重编译入包，
  `make zero-wpt-runner-release`）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向）三连跑，起跑前 ps 核查无兄弟 cargo/runner 在跑，二进制
  sha256 每跑后复验三跑全程 `dc8aedd8…` 未变。guard log 三跑零字节
  （无击杀），runner EXIT=1 为预期挂账常规退出，墙钟 91s/88s/87s。
- **RUN1/RUN2/RUN3 对 M1 终态锚全部逐字节一致（gunzip 后 cmp 全等，
  7,208,436 B ×3）且三跑互等**：123 文件 / 33,109 子测试 = 25,788P +
  7,320F + 1T 状态账零差异（程序化计数复核）——R5374 对 webcrypto
  语料面完全吸收。W4 案处预期 Timeout 臂，本轮零采样，采样面累计
  5/28 轮，维持低频竞态定性。
- 锚窗口推进为 **66f4871c1 → a88ab50c6**，实锚链 ……→a5a11cd07→
  66f4871c1→a88ab50c6，33,109 分母零回归在现 tip 成立。
- **轮询面**：R5374 自账 make test **3,664P/1F = ua_display_li D4
  既有账**（渲染流同树自验）+ layout-engine lib 2308P/0F + corpus
  757F→756F FIXED=1 REGRESSED=∅ + product-smoke PASS + bench-gate
  定向 GATE PASS；D4 冻结门 `ua_display_tests.rs` 末触仍
  `c41b0c750`（R5374 仅触 layout-engine table 域 + rendering-compat
  docs，未触渲染域 style-system 冻结面）；ua_display_li 同键在册
  未清；git 无 D1/D2 决策新账面（GB-20261031 在案，核验只认 git
  账面 + 用户主动知会）。收口序列不变：ua_display_li 清零 → make
  test 全绿复证 → 锚按届时最终 tip 处置 → 归档迁移 + README/csv/
  脚本登记 → 推送 → DONE；或 D2② 授权 standing exclusion 口径收口。
  临时审计件
  `/tmp/webcrypto-anchor-20261101-r5374abspos-{r1,r2,r3}.json`
  （不入库）。

### 2026-11-01 11:53 追记：锚复跑现 tip（`61720d309`）三跑全 cmp 全等 ×3——html-semantics 流 s204 pattern IDL + v flag 吸收，W4 本轮零采样

- R5374 锚账 push 后 rebase 窗口吸入 html-semantics 流 `61720d309`
  （s204 pattern IDL 反射 + v flag 编译判定——shim `part03.js`
  +34/−5、`part05.js` +14，[其账](../../html-semantics-compat/evidence/2026-11-01-m2s204-pattern-idl-v-flag.md)）：
  push 前重查流程（09:20 补强）核证 shim 系语料执行路径二进制面
  再前移，刚推锚账即陈旧，随即按纪律对新执行 tip 复跑——release
  runner 重建 25.51s（shim 重编译入包）+ test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`）三连跑，
  起跑前 ps 核查无兄弟 run 在跑，二进制 sha256 三跑全程
  `95dadfa4…` 未变。guard log 三跑零字节，EXIT=1 预期，墙钟
  89s/88s/89s。
- **RUN1/RUN2/RUN3 对 M1 终态锚全部逐字节一致（gunzip 后 cmp 全等，
  7,208,436 B ×3）且三跑互等**：123 文件 / 33,109 子测试 = 25,788P +
  7,320F + 1T 状态账零差异——s204 对 webcrypto 语料面完全吸收。
  W4 案处预期 Timeout 臂，本轮零采样，采样面累计 5/29 轮。
- 锚窗口推进为 **a88ab50c6 → 61720d309**，实锚链 ……→66f4871c1→
  a88ab50c6→61720d309，33,109 分母零回归在现执行 tip 成立。
- **轮询面**：s204 自账 zero-engine **3,664P/1F**（ua_display_li
  渲染流域在册键同位）；D4 冻结门 `ua_display_tests.rs` 末触仍
  `c41b0c750`（s204 仅触 engine shim + html-semantics docs）；
  git 无 D1/D2 决策新账面。收口序列不变。临时审计件
  `/tmp/webcrypto-anchor-20261101-s204pattern-{r1,r2,r3}.json`
  （不入库）。

### 2026-11-01 12:01 追记：锚复跑现执行 tip（`ee2ee9ec1`）三跑全 cmp 全等 ×3——html-semantics 流 s205 style-with-value 查询面 + zero-web 流 ClearHttpCookies 吸收，W4 本轮零采样

- s204 锚账 push 后 rebase 窗口再吸入两笔：html-semantics 流
  `ee2ee9ec1`（s205 style-with-value 簇值查询面——gCS plain 四属性
  解析器 + 五值命令引擎面接线，shim `part01c.js` +417/`part06.js`
  +21/−3）+ zero-web 流 `b5ef2f0e8`（ClearHttpCookies 广播，protocol
  message +4 与 serialize 测试，非 runner 链路面）。push 前重查流程
  核证 shim 系语料执行路径二进制面再度前移，随即按纪律对新执行 tip
  复跑（本轮第二次）——release runner 重建 35.9s（shim 重编译入包）
  + test-guard 直包同参数（`--per-proc-mem 4 --total-mem 8
  --time-limit 1800`）三连跑，起跑前 ps 核查无兄弟 run 在跑，二进制
  sha256 三跑全程 `ff63b9d7…` 未变。guard log 三跑零字节，EXIT=1
  预期，墙钟 87s/101s/90s。
- **RUN1/RUN2/RUN3 对 M1 终态锚全部逐字节一致（gunzip 后 cmp 全等，
  7,208,436 B ×3）且三跑互等**：123 文件 / 33,109 子测试 = 25,788P +
  7,320F + 1T 状态账零差异——s205 对 webcrypto 语料面完全吸收。
  W4 案处预期 Timeout 臂，本轮零采样，采样面累计 5/30 轮。
- 锚窗口推进为 **61720d309 → ee2ee9ec1**，实锚链 ……→a88ab50c6→
  61720d309→ee2ee9ec1，33,109 分母零回归在现执行 tip 成立。
- **轮询面**：s205 自账 zero-engine **3,664P+1F**（ua_display_li
  渲染流域在册键同位）；D4 冻结门 `ua_display_tests.rs` 末触仍
  `c41b0c750`（s205/b5ef2f0e8 均未触渲染域 style-system 冻结面）；
  同窗口 site-opt PR #266 合流簇（`4745773c9` 等 5 笔，T36/H3
  hotnav）经 `git diff ee2ee9ec1..origin/main -- crates/ tests/`
  核证**非执行路径**（仅 apps/browser、apps/renderer、网站 docs），
  锚账不因之复跑；git 无 D1/D2 决策新账面。收口序列不变。临时
  审计件 `/tmp/webcrypto-anchor-20261101-s205styleval-{r1,r2,r3}.json`
  （不入库）。

### 2026-11-01 12:15 追记：D4 双面裁决现身 git 账面——用户提交 `bb4cba524` 推上 `origin/site-opt/s24-flip-red`（未合入 main），本 goal 按碰撞纪律不抢着陆

- pull 核证新分支 `origin/site-opt/s24-flip-red`：单提交 `bb4cba524`
  （作者+提交者均为用户 LEI Zongmin，2026-11-01 12:13:03 +0800，观测时
  距推送约 1 分钟）——「UA display 双面裁决 li=list-item/canvas=inline +
  布局域连锁修复」，即 D2 记账的修复路径原样现身：`ua_default_display`
  host 表 li→ListItem、canvas→Inline + display_str 注入补显式
  `ListItem => "list-item"` 臂 + tree.rs（+28/−1，canvas atomic inline
  排除两处 leaf-path 判据）/ inline_finalization.rs（+18/−1，
  is_inline_canvas 喂两处 ib_sizes 收集点）/ sizing.rs（+5/−1，
  intrinsic sizing gate 补 ListItem）三处布局连锁修复；触及冻结门
  `ua_display_tests.rs`（+17/−10）；另附 learnings
  `docs/learnings/bugs/2026-11/2026-11-01-ua-display-dual-surface-divergence.md`。
- commit message 自附全量门禁走查：layout-engine lib 2306P/0F（翻转
  暴露的 3 既有红全收）、style-system 2469P/0F、integration 794P/0F ×3、
  make reftest 724P/1F（唯一红=float-shrink-inline-children 基线预存，
  stash A/B 双臂逐位同败）、product-smoke RC=0、touched crates clippy
  RC=0、fmt 0、make test 全腿走查真实套件全绿（4 预存红逐一在案 + 2
  负载敏感 flake 复跑绿）。**ua_display_li 预期随合入清零**（s24 flip
  常驻红即其跟踪位）。
- **未合入 main**；观测时兄弟会话活跃（双 `claude -p --resume` 进程在
  跑，分支名 site-opt 前缀归属对端流），按 run-rules §9 碰头纪律本
  goal 不抢着陆、不代跑分支门禁（避免与对端会话构建竞争）。**合入
  main 即触发 D2① 收口序列**：make test 全绿复证 → 锚按最终 tip 复跑
  ×3（含 push 前 rebase 窗口重查）→ 归档迁移 + README/csv/脚本登记 →
  推送 → DONE。
- **main 侧锚面无变化**：HEAD 仍 `492e9fb2e`，`git diff
  ee2ee9ec1..HEAD -- crates/ tests/` 为空，冻结门末触仍 `c41b0c750`，
  M1 终态锚（7,208,436 B，25,788P+7,320F+1T ×3）对 main 现执行 tip
  仍现行，不复跑；D1 无新账面。

### 2026-11-01 12:27 追记：锚复跑现执行 tip（`7c681d5c6`）——s205 setCustomValidity 换行归一吸收，RUN1/RUN2 整文件全等、RUN3 状态账全等 + W4 采样臂现形（累计 6/31 轮）

- 本 goal 上笔账（12:15 D4 分支现身）push 时 rebase 窗口吸入
  html-semantics 流 `7c681d5c6`（s205 setCustomValidity 换行归一——shim
  `part03.js`/`part04.js` 双存入点 \r\n|\r→\n + bridge_tests `part08.rs`
  +15，crates/ 执行路径前移，shim 入 runner 包），按纪律对现执行 tip
  复跑锚 ×3：release runner 重建 24.4s + test-guard 直包同参数
  （`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，`--json`
  重定向），起跑前 ps 核查无兄弟 run/孤儿进程，二进制 sha256
  `624a6412…` 三跑全程未变。guard log 三跑零字节（零击杀），EXIT=1
  预期，墙钟 89.5s/88.7s/92.1s。
- **RUN1/RUN2 对 M1 终态锚整文件逐字节一致**（gunzip 后 cmp 全等，
  7,208,436 B）且 RUN1==RUN2（sha256 同值 `40ea3c8e…`）；**RUN3 整
  文件 cmp 在 W4 案 message 文本处分歧**——`algorithm-discards-context`
  1T 竞态本轮采样：RUN3 走 completion-callback-not-called 臂（phase=2、
  test_wait=false），锚走 completed-without-reporting 臂（phase=4），
  **status 双方均 Timeout，33,109 条 (case, subtest) 状态账逐位对照
  0 差异**（25,788P + 7,320F + 1T），按「状态账逐位为主判据」锚成立；
  W4 采样面累计 **6/31 轮**。
- 锚窗口推进为 **ee2ee9ec1 → 7c681d5c6**（s205 setCustomValidity
  全吸收），33,109 分母零回归在现执行 tip 成立。
- 轮询面其余：D4 裁决分支 push 窗口内 force-update `bb4cba524` →
  `11f6770a4`——核证为同一 6 文件修复 rebase 至 `492e9fb2e`（作者
  日期 12:13:03 不变、内容 +168/−18 同），非内容变化，仍未合入 main；
  main 侧冻结门末触仍 `c41b0c750`，ua_display_li 同键在册未清；D1
  无新账面。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-s205setcv-tip7c681d5c6-{r1,r2,r3}.json`
  （不入库）。

### 2026-11-01 13:45 追记：锚复跑现执行 tip（`583ebe1df`）三跑整文件对 M1 终态锚 cmp 全等 ×3——渲染流 R5375 max-content 内容基测量吸收，W4 本轮零采样

- 本 goal 上笔账（12:27）push 后 rebase 窗口吸入渲染流 `583ebe1df`
  （R5375 max-content 内容基测量——layout-engine `engine/sizing.rs`
  +16/−1、`intrinsic_sizing.rs` +9、新增 `tests/
  r4149_width_keyword_clamp_tests.rs`，crates/ 执行路径前移，
  [其账](../../rendering-compat/evidence/r5375-max-content-content-based-2026-11-01.md)）：
  按 `git diff 7c681d5c6..583ebe1df -- crates/ tests/` 非空核证后，
  按纪律对现执行 tip 复跑锚 ×3——release runner 重建 30.3s
  （layout-engine 重编译入包，`make zero-wpt-runner-release`，二进制
  sha256 `f16be429…`）+ test-guard 直包同参数（`--per-proc-mem 4
  --total-mem 8 --time-limit 1800`，`--json` 重定向），起跑前 ps
  核查无兄弟 run/孤儿进程。guard log 三跑零字节（零击杀），EXIT=1
  预期，墙钟 90.6s/90.9s/89.4s。
- **RUN1/RUN2/RUN3 对 M1 终态锚全部逐字节一致（gunzip 后 cmp 全等，
  7,208,436 B ×3）且三跑互等**（sha256 同值 `40ea3c8e…`）：123 文件 /
  33,109 子测试 = 25,788P + 7,320F + 1T 状态账零差异——R5375 对
  webcrypto 语料面完全吸收。W4 案处预期 Timeout 臂，本轮零采样，
  采样面累计 **7/32 轮**。
- 锚窗口推进为 **7c681d5c6 → 583ebe1df**，实锚链 ……→ee2ee9ec1→
  7c681d5c6→583ebe1df，33,109 分母零回归在现执行 tip 成立。
- **轮询面**：D4 裁决分支/PR #267 仍 OPEN、head 仍 `11f6770a4`、未合
  入 main，合入即触发 D2① 收口序列；冻结门 `ua_display_tests.rs`
  末触仍 `c41b0c750`（R5375 仅触 layout-engine sizing 面 + browser
  headless tests + rendering docs，未触 style-system 冻结面），
  ua_display_li 在册未清；git 无 D1/D2 决策新账面。收口序列不变：
  ua_display_li 清零 → make test 全绿复证 → 锚按届时最终 tip 处置 →
  归档迁移 + README/csv/脚本登记 → 推送 → DONE。临时审计件
  `/tmp/webcrypto-anchor-run-20261101-r5375-tip583ebe1df-{r1,r2,r3}.json`
  （不入库）。

### 2026-11-01 13:52 追记：上笔账 push 后 rebase 窗口吸入 renderer-trust-boundary 流 `e58201648`（compositor landlock 双根因修复）——核证非执行路径，锚账不复跑

- `git diff 583ebe1df..e58201648 -- crates/ tests/` 为空（仅
  `apps/compositor/` landlock/seccomp 沙箱 2 文件 + renderer-trust-
  boundary docs 2 文件，apps/ 不入 release runner 二进制面，12:01 轮
  PR #266 簇同先例）；现执行路径仍锚 `583ebe1df`，13:45 锚账（×3
  cmp 全等）现行不变。冻结门末触仍 `c41b0c750`，本地 == origin ==
  `1fcf8967f`。轮询面无其他变化，D2①/D1 无新账面。

### 2026-11-01 14:04 追记：锚复跑现执行 tip（`e08a4febc`）状态账零差异 ×3（整文件分歧仅 W4 采样臂文本，累计 8/33 轮）——html-semantics 流 s206 output value/defaultValue 活读模型吸收

- 本 goal 上笔账（13:52）push 后 rebase 窗口吸入 html-semantics 流
  `e08a4febc`（s206 output value/defaultValue 活读模型——后代突变活读 +
  reset 清 override，mutations 2F 收口，[其账](../../html-semantics-compat/evidence/2026-11-01-m2s206-output-value-defaultvalue-live-read.md)；
  engine `js_dom_bridge_tests/part09.rs` +92、shim `part03.js`/`part04.js`
  +61/−26，crates/ 执行路径前移）：按
  `git diff 8174304d5..e08a4febc -- crates/ tests/` 非空核证后，按纪律
  对现执行 tip 复跑锚 ×3——release runner 重建 24.4s（shim 重编译入包，
  `make zero-wpt-runner-release`，二进制 sha256 `f8e4ef5f…`）+ test-guard
  直包同参数（`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，
  `--json` stdout 重定向），起跑前 ps 核查 ZeroWeb 面无兄弟 run/孤儿
  进程。guard log 三跑零字节（零击杀），EXIT=1 预期，墙钟 ≈90s ×3
  （JSON 产物 mtime 间隔 91s/90s）。
- **RUN1/RUN2/RUN3 三跑互等逐字节一致**（sha256 同值 `7cdea628…`，
  7,208,497 B ×3）；对 M1 终态锚整文件 cmp +61 B 分歧——**分歧仅在
  W4 案 `algorithm-discards-context` Timeout 消息文本**：本轮三跑同落
  phase=2 采样臂变体（`testharness completion callback was not called
  (…phase":2…)`，锚为 phase=4 变体 `completed without reporting
  registered tests`；12:27 轮同先例），其余全部 status + 非 Timeout
  message 结构化比对 **status-diffs=0**：123 文件 / 33,109 子测试 =
  25,788P + 7,320F + 1T 状态账零差异——s206 对 webcrypto 语料面完全
  吸收。W4 案完成路径竞态采样臂现形，采样面累计 **8/33 轮**。
- 锚窗口推进为 **583ebe1df → e08a4febc**，实锚链 ……→ee2ee9ec1→
  7c681d5c6→583ebe1df→e08a4febc，33,109 分母零回归在现执行 tip 成立。
- **轮询面**：D4 裁决分支/PR #267 仍 OPEN、head 仍 `11f6770a4`、未合
  入 main，合入即触发 D2① 收口序列；冻结门 `ua_display_tests.rs`
  末触仍 `c41b0c750`（s206 仅触 engine bridge 测试 + shim part03/04，
  未触 style-system 冻结面），ua_display_li 在册未清；git 无 D1/D2
  决策新账面。收口序列不变：ua_display_li 清零 → make test 全绿复证
  → 锚按届时最终 tip 处置 → 归档迁移 + README/csv/脚本登记 → 推送 →
  DONE。临时审计件
  `/tmp/webcrypto-anchor-run{1,2,3}-20261101-s206-tipe08a4febc.json`
  （不入库）。
