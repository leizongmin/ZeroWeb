# Renderer 特权请求与进程信任边界

**版本**：v1.0 / 2026-10-20
**状态**：Active（启动门控：用户本轮只要求规划和脚本；等待后续明确启动）
**父目标**：[zero-web](zero-web.md)
**控制面**：[master.md](renderer-trust-boundary/master.md)

## 0. 执行摘要与 Mission

按 renderer 已被攻破的威胁模型，检查服务端如何绑定身份、拒绝跨站特权请求并限制 helper 系统权限。先建立真实消息权限矩阵，再复现和修复确认的问题。

拆分依据：2026-10-20 用户要求将 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md)形成 goal 与 scripts，开发后续投入。本 goal 建立验收目标，不宣称已经发现 ZeroWeb 漏洞。

## 基线与 Support Envelope

静态核查：`crates/engine/src/ws_bridge.rs` 有 `WsHost::connect`、`WsEvent::SetCookie`、`merge_handshake_cookie`；IPC 底座见 `crates/protocol/src/lib.rs`，接收方见 `apps/browser/src/process_backend.rs`。这些符号证明存在数据路径，不证明可越权。运行基线 **NOT_RUN**，启动后记录 HEAD 与相关差异。

范围：Cookie/HttpOnly、HSTS、存储、权限、自动化角色的可信身份；renderer/worker 与 browser/helper 请求权限；macOS/Linux 沙箱实际生效时机。

边界：复用 [security-hardening 历史成果](archive/security-hardening/master.md)和 [cookies-compat](cookies-compat.md)。页面规范兼容仍归兄弟 goal，本 goal负责被攻破进程的权限测试。跨站 iframe 架构变更遵守 navigation-compat 的既有门控；不自动改协议架构或关闭自动化能力。

碰撞：按 [run-rules §9–11](../rally/run-rules.md)记录 protocol、engine、net、browser 接收方与并行流的占用；共享面暂停一边，不硬解。只写本 goal 控制面；不继承通知、提交、推送授权。

## Done Criteria / 验收场景（P0）

| ID | 条件与动作 | 期望与验证 |
|---|---|---|
| DC-1 / IPC-01 | renderer 伪造 origin、URL、session、私密标志 | 服务端按已绑定身份判断；跨站请求拒绝。拟新增生产接收方集成测试，并记录允许/拒绝矩阵 |
| DC-2 / IPC-02 | 普通页面进程申请读取/改写 HttpOnly、HSTS、其他站点存储 | 特权操作不可执行；正常 HTTP/WS Cookie 流程继续正确。拟新增非法消息注入与合法流对照 |
| DC-3 / IPC-03 | 普通 renderer 冒充 WebDriver/CDP 特权角色 | 接收方不可由消息字段自授角色；真实授权自动化不回归。拟新增角色切换/重连测试 |
| DC-4 / IPC-04 | helper 启动线程、重启或遇缺少沙箱能力 | 权限/FD/文件与 socket 策略按实际平台契约生效；失败/降级行为明确。真实 helper 受限操作验收 |
| DC-5 | 修复后验收全部场景 | 常驻回归、适用质量门禁与生产 IPC 复测有据；报告未支持平台与剩余风险，不能用页面 JS 单测替代 |

未知沙箱能力与拒绝方式在 M1 核查，不先规定全平台强制退出；发现额外产品取舍记待决。

## 里程碑与实施交接

| 阶段 | 文件/职责与依赖 | 产物与首步验证 |
|---|---|---|
| M1 | protocol 定义 → browser 接收方 → net/security/storage 所有者 | 消息权限表、实际身份来源、合法/非法请求基线；命令和二进制哈希写 evidence |
| M2 | 现有接收路径与最近集成测试 | 最小反例→共享路径修复→常驻回归；先 cookie/HSTS，再会话/角色 |
| M3 | helper 启动与平台 sandbox | 权限/线程时序验收；调整需核对 Linux/macOS 双侧 |
| M4 | 本 goal 控制面 | DC 审计、适用检查、剩余项与清理证据 |

建议分批：权限盘点与回归；确认缺陷修复；平台沙箱。新能力复用 protocol 与既有 sandbox，测试注入器拟放最近生产 IPC 集成测试；不新增全局权限框架。每批可单独撤销本批修复，不删除证据。

## 执行与 Document Control

统一 CLI、运行守卫、证据协议和授权边界见 [启动说明](ladybird-followups/README.md)。脚本：`./scripts/rally-goal-renderer-trust-boundary.sh`；直接执行启动 Rally，加 `--dry-run` 仅预览。

入口仅目标变化时更新；进展写 master/evidence，里程碑只追加 archive。启动前未获授权不执行 M1 或内核改动。启动后默认 `CONTINUE: <下一可执行切片>`；真实阻塞 `BLOCK: <原因>`；全部 DC 和适用门禁有证才输出 `DONE`。
