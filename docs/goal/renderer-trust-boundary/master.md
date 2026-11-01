# Renderer 信任边界控制面

**日期**：2026-11-01
**状态**：Active（M1 盘点完成；M2 切片 1–5 已交付；M3 切片 1–2 已交付；M4 前置基线 + 已交付修复面 DC-5 生产 IPC 复测已交付）。
**当前卡点**：四项待用户/设计决策：① CDP 默认强制 token（破坏既有自动化客户端）；② `__zw*` 测试钩子族页面可见（fullscreen 激活门可被 `__zwUserActivate`/`__zwSetPermission` 双路自授绕过）——修复需测试模式判定，涉及 shim part02/04/06 与 html-semantics 流工作文件正面重叠；③ helper 完整 OS sandbox（renderer 现为 RFC P3 占位、image-decoder 无钩子）是否补齐；④ 沙箱安装失败平台策略 A/B/C（推荐 C，见 [M4 前置](evidence/2026-11-01-m4-preflight-platform-policy-dc-audit.md)）。
**下一步**：用户对 ①②③④ 拍板后执行对应修复/设计切片；全部落定后 M4 收口（决策面修复复测 + 剩余风险报告；已交付面的 DC-5 复测见 [DC-5 复测](evidence/2026-11-01-m4-dc5-production-ipc-retest.md)）；DRI3 显示补验与 macOS 真机验收为环境受限在册项。
**入口**：[目标契约](../renderer-trust-boundary.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 消息身份、站点、角色与存储所有者矩阵 | 已完成，见 [evidence](evidence/2026-11-01-m1-matrix-storageop-fix.md) | DONE（2026-11-01） |
| ~~StorageOp 信任消息 origin~~ | 已修复，服务端绑定 committed origin + 常驻回归 | DONE（2026-11-01） |
| ~~CDP 端口 DNS rebinding（Host 头非本机名放行）~~ | 已修复，Chromium 同款 verify_host + 真实 socket 回归 | DONE（2026-11-01） |
| CDP 默认无 token、空 origin 白名单=全允许 | opt-in `--remote-debugging-token` 已交付（[切片 4](evidence/2026-11-01-m2-slice4-cdp-token-opt-in.md)）；默认强制 token **待用户决策** | opt-in DONE，默认待决 |
| ~~renderer 本地 cookie jar 分叉~~ | 已修复：`ClearHttpCookies` 广播同步（协议末尾追加变体）；Cookie 头伪造经核验为非缺口（JS R3221 已按 fetch spec 挡） | DONE（2026-11-01） |
| `__zw*` 测试钩子族页面可见（`__zwUserActivate` 直接置瞬态激活 + `__zwSetPermission` 自授权，fullscreen 激活门双路可绕） | 设计切片：测试模式判定（renderer automation flag / runner 装配）+ shim 三文件接线 + GUI 真实输入激活来源核验；与 html-semantics 流 shim 文件碰撞待协调 | 待用户拍板装配策略 |
| PermissionManager 死代码（app 内零调用点） | 与上条合并设计：真权限决策上收 browser 或明确降级文档 | 待设计 |
| ~~多进程 HSTS 不注册（兼容缺口）~~ | 已修复：fetch_proxy document 响应落 per-tab SecurityContext + 2 常驻回归 | DONE（2026-11-01） |
| ~~compositor landlock 从未生效（add_rule flags 误传 size + 访问位错位）~~ | 已修复：位/flags 修正 + 真实 restrict 回归 + 真实进程验收（[M3 切片 1](evidence/2026-11-01-m3-slice1-compositor-sandbox.md)） | DONE（2026-11-01） |
| ~~fd_socket_linux 无 SO_PEERCRED（M1 矩阵第 5 项）~~ | 已修复：socket 0600 + 双侧 SO_PEERCRED euid 校验 + 每发布 128-bit token（wire 不变）+ 2 常驻回归（[M3 切片 2](evidence/2026-11-01-m3-slice2-fd-socket-peer-trust.md)） | DONE（2026-11-01） |
| helper 完整 OS sandbox：renderer 现为 RFC P3 占位（仅 env 剥离）、image-decoder 无任何钩子 | 设计决策（升新能力切片与否）→ 补齐后做 helper 受限操作验收（DC-4 余项）；线程时序契约已审计成立（切片 2） | 待用户决策 ③ |
| 平台缺沙箱能力的产品策略 | 选项已列（A 维持 fail-open / B compositor fail-closed / C 随决策 ③ 统一拍板，推荐 C），见 [M4 前置](evidence/2026-11-01-m4-preflight-platform-policy-dc-audit.md) | 待用户决定 ④ |
| fd 通道真实进程 dma-buf 端到端补验 | 需 DRI3-capable 显示（Xvfb 无 DRI3，静默回退 RGBA；本容器硬件显示为用户桌面不可动） | 环境受限（M4 剩余风险在册） |
| browser-compositor-smoke 脚本漂移（非本 goal）：`ZW_COMPOSITOR_PROCESS` 死变量 + renderer `frame_published` 事件不入父日志，两腿按现码不可绿 | compositor 交付面核对脚本与模式重构的契约漂移后修脚本或断言 | 已记档（M3 切片 1 顺带发现，pre-existing） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 权限矩阵与双向基线 | DONE（2026-11-01） | [matrix](evidence/2026-11-01-m1-matrix-storageop-fix.md) |
| M2 | 确认反例、共享路径修复与回归 | 切片 5/多 完成（StorageOp、CDP Host、ClearHttpCookies、CDP opt-in token、多进程 HSTS） | [切片 1](evidence/2026-11-01-m1-matrix-storageop-fix.md) / [切片 2](evidence/2026-11-01-m2-slice2-cdp-host-rebinding.md) / [切片 3](evidence/2026-11-01-m2-slice3-clear-http-cookies.md) / [切片 4](evidence/2026-11-01-m2-slice4-cdp-token-opt-in.md) / [切片 5](evidence/2026-11-01-m2-slice5-multiprocess-hsts.md) |
| M3 | helper 实际权限与时序验收 | 切片 1–2 完成（compositor S2/S3 实测 + fd 通道对端信任收口 + helper 时序审计）；余 helper 受限操作验收（依赖决策 ③）与 DRI3 补验 | [切片 1](evidence/2026-11-01-m3-slice1-compositor-sandbox.md) / [切片 2](evidence/2026-11-01-m3-slice2-fd-socket-peer-trust.md) |
| M4 | DC 审计、门禁、清理和剩余风险 | 前置基线交付（平台策略选项 + DC 审计快照 + macOS 代码级对照 + 清理核查）；已交付修复面 DC-5 生产 IPC 复测交付；收口待决策 ①②③④ | [前置](evidence/2026-11-01-m4-preflight-platform-policy-dc-audit.md) / [DC-5 复测](evidence/2026-11-01-m4-dc5-production-ipc-retest.md) |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | IPC-01-forged-origin-session | PARTIAL（导航 epoch/SW 权威矩阵有据；集成测试覆盖 StorageOp 面） | matrix |
| DC-2 | IPC-02-cookie-hsts-privilege | PARTIAL（StorageOp、cookie jar 同步、多进程 HSTS 注册已修附回归；权限 stub 未修；Cookie 头伪造判定非缺口） | 切片 1/3/5 |
| DC-3 | IPC-03-automation-role | PARTIAL（服务端不可消息自授角色已核；CDP rebinding 已堵、opt-in token 已交付，默认 token 待决） | 切片 2/4 |
| DC-4 | IPC-04-sandbox-thread-order | PARTIAL（compositor seccomp/landlock 真实 restrict 与失败降级实测；fd 通道对端信任收口；helper 线程时序契约审计成立 + macOS 代码级对照——helper 受限操作验收依赖决策 ③，DRI3 补验/真机验收环境受限） | [切片 1](evidence/2026-11-01-m3-slice1-compositor-sandbox.md) / [切片 2](evidence/2026-11-01-m3-slice2-fd-socket-peer-trust.md) / [M4 前置](evidence/2026-11-01-m4-preflight-platform-policy-dc-audit.md) |
| DC-5 | 全部回归与生产接收方复测 | PARTIAL（已交付修复面生产 IPC 复测有报：headless CDP 8/8 + GUI 清除链 3/3；余决策面复测/HSTS 生产面/DRI3 补验收口时补） | [DC-5 复测](evidence/2026-11-01-m4-dc5-production-ipc-retest.md) |

## 碰撞与执行记录

候选共享面：protocol、engine/ws、browser、net/security/storage。2026-11-01 核对：并行流为 website 流（`website/`+docs，不重叠）与渲染流（ZeroWeb-2 独立 clone）；两切片只改 `apps/browser`（bin 目标 process_backend + headless）与本 goal 文档，无共享面碰撞。

2026-11-01 M2 切片 1：`handle_storage_op` origin 改服务端绑定（同 IndexedDB owner 模型）；新增 `storage_op_owner_tests` 两测试，修复前语义 FAILED/修复后 PASS 反例证据在册。zero-browser bin 全量 715P/2F（两 F 经干净 main stash 复测归因为既有红：paste_and_go_blurs_and_clears_preedit、form_fixture_complete_multiprocess_semantics，与本切片无关）。消息字段 `params.origin` 保留不删（bincode 位置编码 wire 兼容）。

2026-11-01 M2 切片 2：CDP Host 头本机校验（verify_host，IP 字面量/localhost 才放行，缺 Host 放行，畸形 fail closed）堵 DNS rebinding；HTTP 发现 403、WS 升级拒连。三个常驻回归含真实 socket 双向用例。干净 main stash 连跑两轮对照归因：既有红 2 个确定性 + 3 个负载 flaky（两环境共现，无本切片独有成员）；clippy 17 条 dead_code 警告均不在本切片文件。生产脚本消费者（browser-launch-smoke/run-oracle-capture/comp04-probe）全为 127.0.0.1/localhost Host 形态，核对不破坏。CDP 默认 token 记待决。

2026-11-01 M2 切片 3：`ClearHttpCookies` 协议末尾追加变体 + renderer handler 清本地 jar + browser `broadcast_to_renderers` 广播接线 `apply_settings_clear_cookies`；单进程路 no-op。三常驻回归（protocol roundtrip / browser 广播桩 / renderer jar 清空）。cookie jar 分叉缺口收口；Cookie 头伪造经核验降级为非缺口（JS R3221 forbidden-header 已挡 + browser HTTP 无 jar 注入面，进程级攻击者无 IPC 面可护）。zero-protocol 333P/0F、zero-renderer 363P/0F、zero-browser bin 718P/3F（3F 均为切片 1/2 已归因成员）；clippy 触及文件零警告。

2026-11-01 M2 切片 4：`--remote-debugging-token` opt-in（与 port 同款双形态解析）经既有 `with_security` 装配，默认零变化；复用首消息 `-32001` 认证路径；stale `#[allow(dead_code)]` 清理（with_security/new/with_token 获生产调用点，with_origin 保留）。e2e 回归（tungstenite 真实 WS：无 token -32001、带 token 正常应答）。bin 全量 720P/2F（均既有归因成员）。切片 4a 定性：`__zwUserActivate`（part06.js:446）与 `__zwSetPermission`（part02.js:8656）均页面无条件可见，fullscreen 激活门（part04.js:4966）双路可绕——`__zw*` 测试钩子族装配策略升为设计切片记缺口表（与 html-semantics 流 shim 文件碰撞，§9 不硬解）。

2026-11-01 M2 切片 5：多进程 HSTS 注册——`TabFetchProxy::note_hsts_response`（https-only 门，镜像 webview.rs:2571）接 `drain()` 响应完成臂，仅 `resource_type == "document"`（P11 显式头契约）落 per-tab SecurityContext；browser 消费面 `check_resource_url` 无条件执行 HSTS 升级故无 mixed_content 开关错位。per-tab `HstsStore` 粒度与单进程 per-WebView 一致，真浏览器为全局存储记后续对齐项。两常驻回归（对齐 mixed_content_gate.rs 口径 + 回环 HTTP STS 头端到端 https-only 门，钉 status==200 防 Err 假绿）。bin 全量 722P/2F（失败集与切片 4 完全一致，无本切片独有成员）；clippy 零警告。

2026-11-01 M3 切片 1：compositor landlock 从未生效双根因修复——① `landlock_add_rule` 第 4 参误传 size 落入 flags 位（内核无 size 参）→ 安装永远 EINVAL 走 fail-open；② 访问位错位（READ=1 实为 EXECUTE 位）→ 读从未受限且无报错。修正位定义与 flags=0，修正前轮「EXECUTE 不可授予目录 fd」误断言（实测目录+EXECUTE 可装）。errno 判别矩阵证伪中途「容器 seccomp 拦截」假设（fd=-1→EBADF 证 syscall 直达内核），landlock 在本容器环境完整可用。新增真实 restrict 常驻回归；seccomp 三执行型测试补 --exact 全路径 + stdout 兜底收口假绿。真实进程验收：landlock/seccomp (gpu=true) 首次安装成功无降级，帧管线全通无 fallback。zero-compositor 49P/0F、clippy -D warnings 零警告、fmt clean。顺带发现 browser-compositor-smoke 脚本两处 pre-existing 漂移记档不修（死变量 + frame_published 不入父日志）。

2026-11-01 M3 切片 2：fd 传递通道对端信任收口（M1 矩阵第 5 项）——`fd_socket_linux.rs` 三层防线：socket 0600、双侧 SO_PEERCRED euid 校验（accept 侧跳过陌生连接继续等、connect 侧拒绝报错；macOS 无 SO_PEERCRED 记注释由 ①③ 保障）、socket 名加每发布 /dev/urandom 128-bit token（经既有描述符字段传输，wire 不变）。2 新增常驻回归（token 唯一性/格式、窗口内 0600 mode），既有 roundtrip+R3340 fd 计数 3 测试原样通过。helper 时序审计：renderer apply_early 为 role 入口首语句（先于一切线程）、重启重放成立；image-decoder 无任何钩子记缺口（决策 ③）。环境发现（重要）：X :20 实为用户实时 xfce 桌面，GUI smoke 开窗污染桌面且结果不可靠——本轮在 :20 的 sandbox on/off 分层 bisect（误判 seccomp/landlock 致卡）全部作废，同命令在自起干净 Xvfb 一次通过；Xvfb 无 DRI3，gpu-dmabuf 真实交付无法触发（静默回退 RGBA），fd 通道真实进程 dma-buf 端到端补验记 M4 环境受限项。验证：zero-protocol 335P/0F、zero-compositor 49P/0F、clippy -D warnings 零警告、fmt clean；干净 Xvfb 上 CPU+GPU 两腿真实进程 exit 0 无失败信号。

2026-11-01 M4 前置：平台缺沙箱能力产品策略选项分析（A 维持 fail-open / B compositor fail-closed / C 随决策 ③ 统一拍板，推荐 C）+ DC-1..5 审计快照（均 PARTIAL/NOT_RUN，剩余项全部映射到决策 ①②③④ 与环境受限项）+ macOS 代码级对照（seccomp/landlock/fd 通道均编译门控 Linux-only，降级日志明确；真机验收环境不可达在册）+ 清理核查（无残留进程/Xvfb）。唯一代码触点：fd_socket_linux.rs 一处注释按模块实际 Linux 编译门控修正表述（无行为变化）。决策 ①②③④ 全部落定前无可推进的修复面。

2026-11-01 M4 DC-5 复测（已交付修复面）：不等决策 ①②③④，对已交付面先行生产接收方复测——headless CDP 腿 8/8（evil Host 拒绝/本机放行、JS storage set/read、token 实例发现开门 + 无 token -32001 + 带 token 应答、document.cookie set/read）+ GUI 腿 ClearHttpCookies 全语义链 3/3（Set-Cookie→WS 握手带 Cookie 证 jar 写读；Ctrl+T 输 zero://settings/clear/cookies 证 App 拦截；双 renderer 子进程同刻日志证广播到达）。发现四项如实记档：JS Web Storage 每文档 stub（既有功能缺口非本 goal）、HTTP 请求不附 cookie 为 t8k-delta F1 既有设计（首版 img probe 观察面选错改 WS 握手头）、headless Page.navigate 到 zero://settings/* 阻塞 mux ~15s（自动化面缺口）、evil Host 403/RST 竞速（拒绝语义不受影响）。同腿顺带复核 compositor seccomp+landlock 零降级与 renderer P3 占位现状锚点。环境事故如实记档：早前 `pkill -x Xvfb` 清理误杀 :20 用户实时桌面 Xvfb（非本任务所起），会话无法盲恢复已报告用户；教训固化「只 kill 自起实例 PID，禁按名模式杀 Xvfb」。

