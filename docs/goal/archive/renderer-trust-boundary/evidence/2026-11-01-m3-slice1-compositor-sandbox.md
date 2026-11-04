# M3 切片 1：compositor 沙箱实测——landlock 从未生效双根因修复 + seccomp 测试假绿收口

**日期**：2026-11-01
**HEAD（基线）**：`7c681d5c6`
**范围**：`apps/compositor/src/landlock_linux.rs`（实现修复 + 1 新增常驻回归）、`apps/compositor/src/seccomp_linux.rs`（仅测试兜底）。

## 缺口

DC-4/M3 核查发现 compositor landlock（RFC 4.5-S3）自引入以来从未实际生效，两个独立缺陷叠加，均被 fail-open 语义掩盖：

1. **`landlock_add_rule` 第 4 参误传 size（主根因）**：内核原型 `landlock_add_rule(ruleset_fd, rule_type, rule_attr, flags)` 无 size 参；旧实现传 `size_of::<PathBeneathAttr>()` 落入 `flags` 位，非零一律 EINVAL——安装每次失败走 fail-open（`tracing::warn!` 继续运行），读/exec 保护从未落地。
2. **访问位错位（次根因）**：旧常量 `READ=1`/`WRITE=2` 按 ABI v1 实为 `EXECUTE`/`WRITE_FILE` 位——即便装上也只 handled exec/write，`READ_FILE`/`READ_DIR` 从未受限，读保护静默失效且无任何报错。

另修正前轮错误断言：`EXECUTE 不可授予目录 fd` 不成立。实测目录 fd 授 `EXECUTE|WRITE_FILE` 可正常 add_rule（EXECUTE 语义为治理树下文件执行）；目录规则 EINVAL 的旧结论是把缺陷 1 的 EINVAL 误归因到位定义上。

## 修复

- 位常量按 ABI v1 修正：`EXECUTE=1<<0`、`WRITE_FILE=1<<1`、`READ_FILE=1<<2`、`READ_DIR=1<<3`；目录授予集 `FS_RO_DIR=READ_FILE|READ_DIR`、`FS_RW_DIR=FS_RO_DIR|WRITE_FILE`；handled 保持四位置 0xF。
- `syscall_landlock_add_rule` 第 4 参改传 `0u32`（flags 保留 0），删除误传的 size 与多余尾参。
- 模块头注释重写为上述双根因（含证据日期），移除不成立的 EXECUTE/目录断言。
- seccomp_linux.rs 三个执行型子进程测试假绿收口：子进程 `--exact` 名称改全路径（短名零匹配时 libtest 空跑 exit 0），父进程 `.status()` 改 `.output()` 并断言 stdout 含 `running 1 test`（新 landlock 测试同款兜底）。

## 常驻回归（1 个新增）

- `landlock_restrict_blocks_reads_outside_allowlist`：子进程镜像生产前置（PR_SET_NO_NEW_PRIVS）后真实 `install_compositor_landlock()`，验收 `/dev/shm` 读写通过、白名单外 `/etc/passwd` 读取被拒（EACCES/EPERM）；create_ruleset ENOSYS/EPERM 按既有容忍口径 exit 4 跳过；父进程断言 stdout 非空跑。生产侧 GPU/非 GPU 规则路径共用同一安装函数。

## 排查记录（环境定性）

- 本开发环境为容器（`/.dockerenv`、`systemd-detect-virt`=container-other、PID 1 `Seccomp: 2`）。排查初期误判「容器 seccomp 拦截 add_rule 一律 EINVAL」；以 errno 判别矩阵证伪：`add_rule(fd=-1)`→EBADF、`rule_type=99`→EINVAL、坏指针/坏 flags→EINVAL——syscall 直达内核且按参数语义报错，landlock 在本环境完整可用。
- 仓外 C 探针端到端确证（kernel 6.18.25-xanmod1，ABI 7）：`create_ruleset(0xF)` + 三条 path_beneath 规则（/dev/shm RW、/tmp EXECUTE|WRITE_FILE 对照、字体 RO）+ `restrict_self` 全通；restrict 后 `/etc/passwd` 打开 EACCES、`/dev/shm` 写读通过、字体目录 readdir 通过。探针位于 `/tmp/lk-probe/`（仓外，按病态输入规矩独立目录，非 fuzz 负载）。

## 真实进程验证（browser-compositor-smoke compositor 腿）

`target/release/zero-browser --renderer=cpu`（`ZERO_BROWSER_PRODUCT_SMOKE=1`、DISPLAY=:20、RUST_LOG=info）实际运行：

- `compositor: seccomp 网络/exec 过滤已启用 (gpu=true)`、`compositor: landlock 文件系统沙箱已启用 (gpu=true)`——修复后首次真实安装成功，无「安装失败」降级日志。
- 帧管线全通：renderer 1 spawn → `frame_submitted` → `frame_committed`（zero-compositor）→ `frame_completed` → `compositor_bitmap_adopted` → `frame_captured source=compositor_bitmap fallback=false`；SMOKE_CAPTURE PNG 落盘（199,993 B）且 chrome/page SMOKE_REGION 签名齐全（页面文字渲染过，即 restrict 后字体已加载路径无回退）。
- 无 `SMOKE_FAILURE`/`panicked`/`Compositor disconnected`/`fallback=true`。

**顺带发现（记档不修，pre-existing）**：该脚本两腿按现码不可绿——① `ZW_COMPOSITOR_PROCESS=0` 已是死变量（`compositor_client::enabled()` 恒 `true`，浏览器始终走独立 compositor 进程），legacy 腿实际以 compositor 模式运行；② renderer 侧 `SMOKE_EVENT ... frame_published`（renderer/src/runtime.rs:4843）不出现在父进程日志，脚本两腿的 frame_published 断言无可匹配输入。两者均早于本切片（工作树仅两文件改动可证），属 compositor 交付面脚本与浏览器模式重构的漂移，不在本 goal 范围内修。

## 验证命令与结果（test-guard 包裹）

- `cargo test -p zero-compositor --lib` → 49 passed / 0 failed（含新 landlock 回归真实 restrict 路径与 seccomp 三执行型测试）
- `cargo clippy -p zero-compositor --all-targets -- -D warnings` → 零警告
- `cargo fmt -p zero-compositor -- --check` → clean（workspace fmt 有 apps/browser 既有 diff，不属本切片文件）

## 清理

无残留进程；smoke 产物在 `target/browser-compositor-smoke/`（gitignore 面）。探针目录 `/tmp/lk-probe/` 保留供复核，不入仓。
