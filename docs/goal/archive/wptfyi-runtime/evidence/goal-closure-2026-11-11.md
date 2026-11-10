# goal 收口取证（clippy / build / test，HEAD c6960416f + 归档提交）

**2026-11-11** · DC-3 完成条件的守卫取证项。机器 46G，同窗并行流
（ZeroWeb-2 clone 渲染流定向 layout-engine 测试、本仓 html-semantics 流定向
testharness）共享 CPU；全程走 `make guarded-*`（systemd scope 24G）与
test-guard 包裹。

## fmt

`cargo fmt --all -- --check` 初跑 4 文件漂移（均已提交进 main、工作区无
未提交改动）：`crates/css-parser/src/tests/r2320_math_specified_serialize_tests.rs`
（`9f25a0fa4`，css 流）、`crates/css-parser/src/values/serialize_math.rs`
（同上）、`crates/script-sandbox/src/v8_runtime.rs`（`91ab28c56`，本流
source-phase）、`crates/style-system/src/property/apply.rs`（css 流 grid 批）。
全部为 rustfmt 标准折行/use 排序，无语义变化；`cargo fmt --all` 修复后
复检 clean。跨流格式漂移归因各流（未跑 fmt 即提交），本次统一收口修复。

## clippy

`make guarded-clippy`（= `cargo clippy --workspace --all-targets --
-D warnings`）**exit 0 干净**，50.29s（缓存热）。quickjs feature 组合的
clippy 因下述 make test 中止语义未跑（本流变更仅注释路径，不涉 feature
组合敏感面）。

## build

`make build`（workspace --exclude zero-browser + `-p zero-browser`）**exit 0
全绿**。`zero-engine (lib)` 有 7 个 dead_code rustc warning
（`HOST_CALL_TRACE` / `HOST_CALL_TRACE_FILE` / `HostCallTraceGuard` /
`HOST_CALL_TRACE_MS` / `arm` 等，非 clippy `-D warnings` 面），build 门允许
warning，如实记录不处理。

## test

`make guarded-test`（fmt 前置 + standalone binaries 刷新 + test-guard v8
workspace 腿）：**v8 workspace 腿 3833P/7F**（`js_dom_bridge` 集成集，
26.00s），make 按首腿失败中止（中止语义在册），后续腿（renderer bin、
browser bin/tests、quickjs 测试段）未跑。

7F 归因——**与 11-09 起各流在册的「组合态既有集」逐名同构，非本流引入、
非新失败**：

| 失败 | 在册证据 |
|---|---|
| img_runtime_deferred_settle_r5275 | 11-07 shadow-dom M2 clean HEAD `bb48bdf55` 逐案确定性 FAILED 法证在册（img/fetch/websocket/service-worker 异步事件循环面） |
| svg_img_settle_density_r5277 | 同上 |
| test_fetch_filtered_response_type_matrix | 同上 |
| test_iframe_service_worker_messages_dispatch_to_iframe_container | 同上 |
| test_t8k_websocket_error_nonterminal | 同上 |
| test_t8k_websocket_event_flow_open_message_close | 同上 |
| test_response_body_readable_stream_r2967 | 11-09 起在册：editing-compat f6b 名册门 ABORT 记录 + html-semantics s360（`57caa86f2` R2967 ReadableStream 引入，streams 引擎面在册债，与 DC-2 streams 16 案挂账同族） |

对照：html-semantics s360 证据（11-09）同一 7F 名册逐名同构、DC-2
ShadowRealm 证据（11-10）「engine lib 7F 集逐字一致」。

## 环境卫生

发现 11-07 遗留孤儿 `make testharness-svg`（PID 2026885，cwd 本仓，PPID=1，
futex 等待已消失的子进程 4 天，日志 `/tmp/svg-m1-anim.log` 停在 11-07
11:02）——前轮进程树被连坐杀死后的 make 尸体，精确 PID 清理。
