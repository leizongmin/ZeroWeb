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
