# Renderer 信任边界控制面

**日期**：2026-11-01
**状态**：Active（M1 已完成盘点；M2 切片 1–3 已交付）。StorageOp origin 服务端绑定、CDP DNS rebinding 防护、ClearHttpCookies jar 同步清理，均附常驻回归。
**当前卡点**：CDP 默认强制 token 需用户决策（会破坏既有无 token 自动化客户端=目标边界禁止的"关闭自动化能力"），其余切片可继续。
**下一步**：M2 切片 4——权限决策在 renderer（`__zwSetPermission` 页面可自授权 fullscreen/clipboard 门；`PermissionManager` 在 app 内零调用点为死代码），核实 part02.js:8656 注册表消费点后定最小修复。
**入口**：[目标契约](../renderer-trust-boundary.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 消息身份、站点、角色与存储所有者矩阵 | 已完成，见 [evidence](evidence/2026-11-01-m1-matrix-storageop-fix.md) | DONE（2026-11-01） |
| ~~StorageOp 信任消息 origin~~ | 已修复，服务端绑定 committed origin + 常驻回归 | DONE（2026-11-01） |
| ~~CDP 端口 DNS rebinding（Host 头非本机名放行）~~ | 已修复，Chromium 同款 verify_host + 真实 socket 回归 | DONE（2026-11-01） |
| CDP 默认无 token、空 origin 白名单=全允许 | **待用户决策**：默认强制 token 属关闭既有自动化能力，需授权后实施；可先补 `--remote-debugging-token` 类 opt-in 参数 | 待决 |
| ~~renderer 本地 cookie jar 分叉~~ | 已修复：`ClearHttpCookies` 广播同步（协议末尾追加变体）；Cookie 头伪造经核验为非缺口（JS R3221 已按 fetch spec 挡） | DONE（2026-11-01） |
| 权限决策在 renderer（PermissionManager 死代码、__zwSetPermission 页面可自授权） | 核实 fullscreen/clipboard 消费点 | NOT_STARTED（切片 4 候选） |
| 多进程 HSTS 不注册（兼容缺口） | browser fetch_proxy 补 register 路径 | NOT_STARTED |
| helper 沙箱/线程时序与平台降级 | 实测 macOS/Linux，记录 FD/socket/文件权限 | NOT_STARTED（M3） |
| 平台缺沙箱能力的产品策略 | M3 证据后列选项，必要时用户决定 | 待核查 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 权限矩阵与双向基线 | DONE（2026-11-01） | [matrix](evidence/2026-11-01-m1-matrix-storageop-fix.md) |
| M2 | 确认反例、共享路径修复与回归 | 切片 3/多 完成（StorageOp、CDP Host、ClearHttpCookies） | [切片 1](evidence/2026-11-01-m1-matrix-storageop-fix.md) / [切片 2](evidence/2026-11-01-m2-slice2-cdp-host-rebinding.md) / [切片 3](evidence/2026-11-01-m2-slice3-clear-http-cookies.md) |
| M3 | helper 实际权限与时序验收 | NOT_STARTED | 无 |
| M4 | DC 审计、门禁、清理和剩余风险 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | IPC-01-forged-origin-session | PARTIAL（导航 epoch/SW 权威矩阵有据；集成测试覆盖 StorageOp 面） | matrix |
| DC-2 | IPC-02-cookie-hsts-privilege | PARTIAL（StorageOp、cookie jar 同步已修附回归；权限 stub 未修；Cookie 头伪造判定非缺口） | 切片 1/3 |
| DC-3 | IPC-03-automation-role | PARTIAL（服务端不可消息自授角色已核；CDP rebinding 已堵，默认 token 待决） | 切片 2 |
| DC-4 | IPC-04-sandbox-thread-order | NOT_RUN | 无 |
| DC-5 | 全部回归与生产接收方复测 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：protocol、engine/ws、browser、net/security/storage。2026-11-01 核对：并行流为 website 流（`website/`+docs，不重叠）与渲染流（ZeroWeb-2 独立 clone）；两切片只改 `apps/browser`（bin 目标 process_backend + headless）与本 goal 文档，无共享面碰撞。

2026-11-01 M2 切片 1：`handle_storage_op` origin 改服务端绑定（同 IndexedDB owner 模型）；新增 `storage_op_owner_tests` 两测试，修复前语义 FAILED/修复后 PASS 反例证据在册。zero-browser bin 全量 715P/2F（两 F 经干净 main stash 复测归因为既有红：paste_and_go_blurs_and_clears_preedit、form_fixture_complete_multiprocess_semantics，与本切片无关）。消息字段 `params.origin` 保留不删（bincode 位置编码 wire 兼容）。

2026-11-01 M2 切片 2：CDP Host 头本机校验（verify_host，IP 字面量/localhost 才放行，缺 Host 放行，畸形 fail closed）堵 DNS rebinding；HTTP 发现 403、WS 升级拒连。三个常驻回归含真实 socket 双向用例。干净 main stash 连跑两轮对照归因：既有红 2 个确定性 + 3 个负载 flaky（两环境共现，无本切片独有成员）；clippy 17 条 dead_code 警告均不在本切片文件。生产脚本消费者（browser-launch-smoke/run-oracle-capture/comp04-probe）全为 127.0.0.1/localhost Host 形态，核对不破坏。CDP 默认 token 记待决。

2026-11-01 M2 切片 3：`ClearHttpCookies` 协议末尾追加变体 + renderer handler 清本地 jar + browser `broadcast_to_renderers` 广播接线 `apply_settings_clear_cookies`；单进程路 no-op。三常驻回归（protocol roundtrip / browser 广播桩 / renderer jar 清空）。cookie jar 分叉缺口收口；Cookie 头伪造经核验降级为非缺口（JS R3221 forbidden-header 已挡 + browser HTTP 无 jar 注入面，进程级攻击者无 IPC 面可护）。zero-protocol 333P/0F、zero-renderer 363P/0F、zero-browser bin 718P/3F（3F 均为切片 1/2 已归因成员）；clippy 触及文件零警告。

