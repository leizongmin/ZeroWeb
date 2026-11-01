# M1 消息权限矩阵 + M2 切片 1（StorageOp origin 服务端绑定）

**日期**：2026-11-01
**HEAD（分析起点）**：`b997b80f76c64cb025868d76a2433dea97f3fb1e`（rebase 至 origin/main a5a11cd07 之上）
**平台**：Linux x86_64（6.18 xanmod）
**守卫**：`./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900/1800`（test-guard sha256 前缀 `9cb9ecdd632a7d76`）

## M1：renderer→browser 消息权限矩阵（生产接收方 process_backend 分发实测核对）

身份绑定总原则：wire 层无身份位（`IpcMessage` 仅 id+kind）；进程身份 = 进程拓扑（browser spawn renderer，每 renderer 专属 stdin/stdout 管道 + 专属读线程），消息归属由通道对端推导；`tab_to_renderer` 由 browser 维护，tab_id 不上 wire。

### 已由服务端权威绑定（允许/拒绝有据）

| 消息 | 特权领域 | 身份来源 | 防护 |
|---|---|---|---|
| NavigationStarted/Committed | 存储键权威 | epoch+URL 双匹配 | 不匹配拒收（process_backend.rs:1304-1356） |
| IndexedDbRequest | IndexedDB | `indexed_db_origins[renderer_id]`（commit 时从 pending url 推导，:1361-1364） | commit 前拒；origin 不看消息字段；跨 renderer poll/ack 拒 |
| ServiceWorkerRequest/HostEvent | SW 注册域/client/private | `committed_document_urls[renderer_id]` 权威覆盖；client_id 强制 `{rid}:{epoch}:{cid}` 前缀；private 由 browser `private_tabs` 注入 | 消息自称 url 被 server 值覆盖 |
| AutomationResponse | 自动化回报 | 仅此一条自动化消息被消费（:2274-2276） | renderer 发 AutomationRequest 落 `_ => {}` 丢弃（apply_inbound_message 末臂）——不可由消息自授角色（DC-3 服务端成立） |
| FetchRequest | 网络/cookie 附着 | cookie/Origin 由 browser per-tab `page_origin()` 决定（fetch_proxy.rs） | 仅剥 `X-Zero-*` 内部头 |
| ServiceWorkerHostEvent private | 私密标志 | browser `private_tabs.contains(tab_id)` | 消息不携带 private |

### 协议中不存在 renderer 可达入口（不可执行=安全）

cookie 读写（含 HttpOnly）、HSTS 读写、permission 授予、session/private 标志：全部留在 browser/net/security 侧，无 IPC 消息种类触达。

### M1 确认缺陷（本切片修复）

**`StorageOp`（message.rs:933-944）**：唯一以消息字段 `params.origin` 决定存储分区的消息。`handle_storage_op`（修复前 process_backend.rs:1241-1249）直接 `storage.local_storage(&params.origin)`，无与已提交 origin 的比对——被攻破 renderer 一条消息即可按任意 origin 读写 browser 持久 localStorage/sessionStorage（DC-2 反例成立）。该路径无生产发送者（renderer 侧 runtime.rs:3730 仅回传 browser 从未发出的消息；活实现是页面内 JS shim），修复风险低。

### 记录在案的剩余风险（未在本切片处理）

1. **CDP 默认无 token**：`HeadlessSecurityConfig` 默认空 token + 空 origin 白名单=全允许（headless/security.rs:29-72；`with_security` 生产零调用）——本机任意进程可连 `127.0.0.1:port` 全量 CDP。DC-3 候选切片。
2. **renderer 本地 cookie jar 分叉**：`zero://settings/clear/cookies` 清不掉 renderer jar（app.rs:4245 注释自认）；renderer 可在 `FetchRequest.headers` 自带 Cookie 头直发 origin（仅初请求无敏感头过滤）。DC-2 候选切片。
3. **多进程 HSTS 不注册**：browser `fetch_proxy` 无 `register_hsts` 调用，多进程模式 HSTS 只剩内置预加载表（写路径仅 webview 单进程可达）——兼容缺口而非越权。
4. **权限决策在 renderer**：`PermissionManager` 在 app 内零调用点（死代码）；页面可调 `__zwSetPermission` 全局 stub 自授权 fullscreen/clipboard 门。DC-2/DC-1 候选。
5. **fd_socket_linux 无 SO_PEERCRED**：fd 传递 socket 单连接无对端凭证校验。M3 范围。

## M2 切片 1：StorageOp origin 服务端绑定

**改动**：`handle_storage_op` 增加 renderer_id 参数，origin 改从 `indexed_db_origins[renderer_id]` 读取（commit 前拒绝并 warn）；分发点传入 rid。消息字段 `params.origin` 保留（bincode 位置编码 wire 兼容）但服务端忽略。与 IndexedDB owner 模型一致（https://storage.spec.whatwg.org/#storage-keys）。

**常驻回归**：`apps/browser/src/process_backend/storage_op_owner_tests.rs`（bin 目标）：
- `storage_op_binds_to_committed_origin_not_message_field`
- `storage_op_rejected_before_navigation_commit`

**反例证据（修复前语义）**：临时还原 `params.origin` 直用后两测试均 FAILED——
- `storage_op_binds_to_committed_origin_not_message_field ... FAILED`（值落入攻击者 origin 分区）
- `storage_op_rejected_before_navigation_commit ... FAILED`（commit 前写入成功）
恢复修复后同二测试 PASS。

**验证命令与结果**：
- `cargo test -p zero-browser --bin zero-browser storage_op` → `3 passed; 0 failed`（含相邻 SW cache 测试）
- 全量 `cargo test -p zero-browser --bin zero-browser` → `715 passed; 2 failed; 1 ignored`（34.7s）
- `cargo clippy -p zero-browser --bins` → 0 warning

**失败归因（干净 main b997b80f7 stash 复测）**：
- `tests::paste_and_go_blurs_and_clears_preedit`：干净树 FAILED、带修复 FAILED——既有红，与本切片无关。
- `tests::form_fixture_complete_multiprocess_semantics`：干净树 FAILED、带修复 FAILED——既有红，与本切片无关。
- 首轮全量另有 2 例失败未在第二轮复现（并行负载 flaky，两轮失败集不同），未计入。

**说明**：`make test` 全量未跑——已知 engine lib 9F 既有红会在中途中止后续腿（见 memory 与 rendering 流记录），本切片影响面为 zero-browser bin，定向守卫运行已覆盖；合入前由 make test 既有编排兜底。

## 清理

临时还原补丁已恢复（`/tmp/pb_fixed.rs` 中间副本已删除）；无残留进程/目录。既有 stash（wip-justify-cluster-*、rendering-stream-content-race）非本流资产，未触碰。
