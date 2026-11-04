# M4 前置：平台缺沙箱能力产品策略选项 + DC 审计基线

**日期**：2026-11-01
**HEAD（基线）**：`4752a00df`（M3 切片 2 push 后）
**性质**：分析与审计（无生产代码变更；唯一代码触点是 M3 切片 2 遗留的一处注释准确性修正，见文末）。

## 一、平台缺沙箱能力的产品策略——选项分析（缺口表在册首步「M3 证据后列选项」）

**M3 实证事实**（切片 1/2 evidence）：

- landlock 在典型 Linux 全链可用：本容器（kernel 6.18，ABI 7）create_ruleset/add_rule/restrict_self 实测通，restrict 后读保护 EACCES 实证；seccomp 黑名单过滤器同容器安装生效。
- 不可用形态三档：ENOSYS（内核无 landlock，<5.13）、EPERM（环境拦 create_ruleset，如强 seccomp 容器 profile）、EINVAL（常量/flags 错误或环境拦 add_rule——切片 1 修复后常量正确，剩余 EINVAL 形态均属环境）。
- 现行失败语义：fail-open（`tracing::warn!` 继续运行），`ZW_COMPOSITOR_SANDBOX/SECCOMP/LANDLOCK` 三个 kill-switch 默认开。
- macOS：landlock/seccomp 模块编译门控 Linux-only，运行时打「仅 Linux 可用，已跳过」info（sandbox.rs:60/83）——降级行为明确。

**选项**：

| 选项 | 语义 | 优点 | 缺点 |
|---|---|---|---|
| A 维持现状 | compositor 安装失败 fail-open + warn 日志 | 可用性优先；老内核/受限容器不受影响；与 M1 决策「不先规定全平台强制退出」一致 | 安全敏感部署可能静默裸奔（日志可观测但无阻断） |
| B compositor fail-closed | 安装失败即退出进程 | 安全确定性强 | 老内核/受限容器上浏览器不可用；沙箱当前是纵深防御层（seccomp 与 landlock 互为备份、任一失败另一仍在），单层失败即退与其防御地位不成比例 |
| C 分层延迟拍板（推荐） | compositor 维持 A；helper（renderer/image-decoder）沙箱（决策 ③）落地时统一设计各进程 fail 语义（Chrome 模型 renderer fail-closed 是在其 renderer 有沙箱之后） | 决策成本最低、可逆；避免为尚不存在的 helper 沙箱预设语义；与 ③ 合并一次拍板 | 短期内无 fail-closed 强度（对纵深防御层可接受） |

**待用户决定**：A/B/C 择一（推荐 C）。

## 二、DC 审计基线（M4 收口前快照）

| DC | 现状 | 证据 | 剩余项与门控 |
|---|---|---|---|
| DC-1 | PARTIAL | M1 矩阵 + StorageOp 常驻回归（切片 1） | 权限 stub 面（`PermissionManager` 死代码 + `__zwSetPermission` 自授）＝决策 ② |
| DC-2 | PARTIAL | StorageOp/cookie jar/多进程 HSTS 常驻回归（切片 1/3/5） | `__zw*` 钩子族页面可见面＝决策 ② |
| DC-3 | PARTIAL | CDP Host 校验 + opt-in token e2e 回归（切片 2/4） | CDP 默认强制 token＝决策 ① |
| DC-4 | PARTIAL | compositor S2/S3 真实 restrict + 真实进程验收（M3 切片 1）；fd 通道对端信任收口 + helper 时序审计（M3 切片 2） | helper 受限操作验收＝决策 ③（renderer/image-decoder 现无 OS 沙箱可验收）；fd 通道 DRI3 显示补验＝环境受限；macOS 真机验收＝环境不可达（本环境无 macOS 主机，代码级对照见下） |
| DC-5 | NOT_RUN | 各切片常驻回归在册（protocol 335P / compositor 49P / browser bin 定向） | 全场景生产 IPC 复测与收口报告，待 ①②③+平台策略落定后执行 |

**已跑适用门禁**（本 goal 两轮代码交付）：触及包定向 test-guard 测试（zero-protocol lib 335P/0F、zero-compositor lib 49P/0F、zero-browser bin dmabuf 定向 1P/0F）、`cargo clippy -p <touched> --all-targets -- -D warnings` 零警告、`cargo fmt -p <touched> -- --check` clean、真实进程 smoke（干净 Xvfb，CPU+GPU renderer 两腿 exit 0 无 fallback）、lei-pre-commit-guard PASS ×2、push 前 rebase 干净。**未跑与原因**：全量 `make test`——main 为并行流组合态，engine lib 9F 既有红（跨流归因在册）中止后续腿，对本 goal 触及面无增量信号（§10 归因纪律）；DRI3 真实 dma-buf 端到端——容器无 DRI3 显示（Xvfb 无 DRI3、:0 为用户桌面不可动）。

## 三、macOS 代码级对照（环境不可达，静态核查）

- compositor：seccomp/landlock 模块 `#[cfg(target_os = "linux")]` 编译门控（sandbox.rs:3-9）；macOS 运行路径为 env 剥离 + 两条「仅 Linux 可用，已跳过」info（:60/:83）——失败/降级行为明确，符合 DC-4「不先规定全平台强制退出」。
- renderer：`apply_early_if_enabled` 双平台运行（env 剥离，无平台分支）；时序契约（role 入口首语句先于线程）与平台无关。
- fd 通道：`fd_socket_linux` 模块整体 `#[cfg(target_os = "linux")]` 门控（protocol lib.rs:23-24）——macOS 无该 socket 面，dma-buf 交付构建点（frame_shm.rs）同为 Linux-only；模块内非 Linux 防御分支不参与编译。
- 真机 macOS 行为验收（安装日志、进程功能无回退）记环境不可达，列 M4 剩余风险。

## 四、清理核查（M4 项）

无残留 zero-browser/zero-compositor/zero-renderer 进程与一次性 Xvfb；smoke 产物在 `target/browser-compositor-smoke/`（gitignore 面）；排查探针保留于仓外 `/tmp/lk-probe/`（不入仓，evidence 已注明供复核）。

## 附：代码触点

fd_socket_linux.rs `peer_same_user` 文档注释一处：原注释「macOS 无对应结构，恒 true」未说明模块本身被 Linux 门控（非 Linux 分支实为门控抬升时的防御分支，不参与 macOS 构建）——按实际编译事实修正表述，无行为变化。
