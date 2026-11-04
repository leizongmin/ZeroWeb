# M2 切片 6：M1 权限矩阵服务端权威绑定面常驻回归补齐 + AutomationResponse tab 绑定先验后取修复

**日期**：2026-11-01
**HEAD（基线）**：`63247f2b6`（决策包提交，与上轮收口态一致；本轮起点 `git pull --rebase` 无新提交）
**性质**：测试补齐为主（DC-1/DC-3 服务端面），附一处最小共享路径修复。

## 背景与选择依据

决策包 ①②③④ 均待用户拍板（无新决策落定），但 DC-1 的验收文本含「拟新增生产接收方集成测试」——M1 矩阵「已由服务端权威绑定」各行中仅 StorageOp 面（切片 1）与 SW/IndexedDB 权威生命周期（既有 owner tests）有常驻回归，其余行只有代码核查记录。本切片把矩阵各行升级为常驻回归（goal 文档建议分批「权限盘点与回归」的回归半边），全部不 spawn 真实 renderer，与决策门禁无交集。

## 新增常驻回归（`apps/browser/src/process_backend/identity_binding_tests.rs`，7 用例）

| 用例 | 钉住矩阵行 | DC |
|---|---|---|
| `renderer_automation_request_falls_to_receiver_sink_without_execution` | renderer 发 `AutomationRequest` 落接收端 `_ => {}` 丢弃，不可自授自动化角色 | DC-3 |
| `automation_response_never_delivered_to_other_tab` | `AutomationResponse` 跨 tab 绑定校验 | DC-3 |
| `service_worker_register_rejects_document_url_diverging_from_committed_authority` | SW Register 自称 document_url 与已提交权威不符 → InvalidArgument 拒绝（对照臂：诚实注册进正常管线） | DC-1 |
| `navigation_start_with_stale_epoch_is_rejected` | NavigationStarted 陈旧 epoch 拒收，不进 begin_navigation/stage | DC-1 |
| `navigation_start_diverging_from_staged_pending_is_rejected` | NavigationStarted 与 staged pending 不匹配拒收且不覆盖 | DC-1 |
| `navigation_start_with_next_epoch_begins_navigation_and_stages_pending` | 合法导航对照臂 | DC-1 |
| `fetch_request_internal_headers_never_reach_origin`（真实 socket） | renderer FetchRequest 自带 `X-Zero-*` 内部头不出 origin；普通头照常转发（请求侧；响应侧已有 `spoofed_resource_type_header_cannot_hijack_download_route`） | DC-1/DC-2 |

未补钉面：IndexedDB 跨 renderer poll/ack 拒绝由 `ConnectionKey{renderer_id, connection_id}` 所有权键 + 既有 cross_renderer_* 真进程用例结构性覆盖，不另加钉。

## 修复：`take_automation_response` tab 校验先于消费（process_backend.rs）

新用例 `automation_response_never_delivered_to_other_tab` 的对照臂暴露既有缺陷：原实现 `remove(&request_id)` **先消费后校验 tab**——被攻破 renderer 按可预测消息 id 伪造其他 tab 在途请求的回执时，一次跨 tab `take` 即把条目消费丢弃，合法 renderer 稍后到达的真回执被毁（跨 tab 自动化拒绝服务；tab 校验本身挡住了错误结果交付，故仅为可用性面）。修复为 get → tab 校验 → 不匹配 warn 返回 None（条目保留，等真回执 insert 覆盖）→ 匹配才 remove。

**反例证据（修复前）**：同测试 FAILED——`同 tab 合法回执照常交付（对照臂）`（跨 tab 拒绝臂通过、随后同 tab 取回 None，证条目已被误消费）。修复后 7/7 PASS。生产等待方（tab_manager parity 循环与各测试 helper）对 None 均为继续轮询语义，happy path 不变；全量 bin 套件中重度消费该路径的 service_worker_owner_tests 全绿佐证。

## 验证

- `cargo test -p zero-browser --bin zero-browser identity_binding`（test-guard --compile-first，per-proc 4G/total 8G/900s）→ **7 passed; 0 failed**
- `cargo clippy -p zero-browser --all-targets -- -D warnings` → 零警告（初版 1 处 question_mark 已改）
- `cargo fmt -p zero-browser -- --check` → clean
- 全量 `cargo test -p zero-browser --bin zero-browser`（同守卫，1800s）→ 707P/24F（首轮）/10F（次轮低负载），归因见下

## 全量失败归因（§10 纪律，clean main stash 复测）

当前 shell 无 DISPLAY（`:20` 用户桌面已在 M4 复测轮事故中丢失，未恢复），winit/compositor 依赖用例在无显示环境超时失败：

- 确定性失败（clean main 复测同 FAIL，与本切片无关）：`typing_in_clicked_input_updates_page_html`、`gpu_compositor_path_dispatches_input_events_to_form_controls`、`clicking_checkbox_without_page_text_publishes_updated_snapshot`、`repeated_new_tabs_keep_live_renderers_and_page_frames`（恰 120s 超时）+ 既有红 2（`paste_and_go_blurs_and_clears_preedit`、`form_fixture_complete_multiprocess_semantics`）。
- 负载敏感：`page_context_menu_view_source_opens_view_source_tab_via_fetch` 首轮（load 4.67）FAIL，低负载带本切片 diff 复测 PASS。
- 本切片 7 用例在两轮全量中均绿；SW owner 真进程用例全绿。

**经验记录**：此前切片「bin 全量 715–722P/2F」的绿灯含 `:20` 桌面在场前提；该显示已不可用后，bin 全量基线应按「clean main stash 对照」归因，不得与历史数字直接比较。后续 GUI 依赖用例如需常绿，应经 `ZW_GUARD_RUNNER_PREFIX="xvfb-run -a"`（仅 --compile-first 路径生效）提供一次性 display。

## 清理

无残留进程；stash 两进两出已全部 pop（`git stash list` 无本切片残留）；`/tmp/fail_list_r6.txt` 为临时归因产物不入仓。
