# M3 切片 6：media-host 最小沙箱（决策包 ⑤ / ③A-media-host + ④C）

日期：2026-11-02。决策依据：[2026-11-02 决策包](../2026-11-02-decision-package-media-host-devtools.md) ⑤（拍板 A：③A-media-host 最小档案，镜像切片 3 模板）+ ④C（新档案 fail-closed）。

## 交付面

- `apps/media-host/src/sandbox.rs`：镜像 image-decoder 切片 3 模板——seccomp 黑名单（网络 socket 全族 + execve/execveat/fork/vfork/ptrace；BPF UAPI 本地 repr(C) 定义保 android 目标，全 Linux 单一代码路径）+ landlock 空 allowed-set（handled 四访问位 + 零规则 restrict，全 fs 拒绝，已打开 fd 不受影响；自设 `PR_SET_NO_NEW_PRIVS`）；env 清洗（LD_PRELOAD/LD_LIBRARY_PATH/DYLD_*）；三 env 开关 `ZW_MEDIA_HOST_SANDBOX/_SECCOMP/_LANDLOCK` 默认开；opt-out 时 stderr 显式「未沙箱运行」警告（降级可见）。
- 音频档：`audio_runtime_rules()` 按 `audio-cpal` feature 门控——启用时 landlock 授予 `/dev/snd` 读写 + 读目录，默认档空 allowed-set。media-host 新增转发 feature `audio-cpal = ["zero-media/audio-cpal"]`（核查全仓无消费者启用该 feature，默认档即生产档）。FIXME 记档：若启用 audio-cpal 且音频栈走 PulseAudio unix socket，seccomp AF_UNIX 黑名单会阻断连接——需按真实音频后端扩 seccomp 白名单或走 tcp 模块，留待有消费者时验收。
- fail-closed（④C）：`main` 在 stdin 读线程 spawn 之前（sandbox 时序契约：任何子线程之前）调用 `apply_early_if_enabled()`，失败 `exit(1)`——renderer 侧呈现为媒体错误面，拒绝静默裸奔。
- 日志口径与 image-decoder 对齐：tracing 升 INFO，安装成功面（seccomp/landlock 档位）进 stderr，供真实进程验收。
- 操作面补齐：`ZW_MEDIA_HOST_SANDBOX/_SECCOMP/_LANDLOCK/_BIN` 四枚注册 `zero-runtime-config` 表 + `docs/runtime-environment.md` 4 行（`ZW_MEDIA_HOST_BIN` 为 webview `media_proxy.rs` 既有消费、此前未注册的补登记；后三枚为本切片新增消费）。

## 验证（test-guard 守卫）

- `cargo test -p zero-media-host`：集成 2P/0F——① 真实二进制默认沙箱下 register→data→play 消息族往返至 `MediaStateUpdate(Playing)`（stderr 断言 seccomp/landlock 安装成功日志在、未沙箱警告不在；安装失败会 exit(1) 使 recv 命中 EOF，无假绿面）；② `ZW_MEDIA_HOST_SANDBOX=0` 豁免面照常服务（MediaReady 握手）+ 显式未沙箱警告。lib 19P/0F——含执行型子探针（fs exec 拒 / execve 拒 / /dev/shm 写拒 / stdio 存活）与非法 ruleset `Err` 上抛。
- 生产链路腿：`cargo test -p zero-webview --test media_host_contract` 1P（`real_host_lifecycle_and_error_faces`：生产 resolve 链 spawn 真实二进制生命周期 + 错误面）；同目录 `media_m1_baseline`/`media_c4_paired` 全为 `#[ignore]` 长跑（60s+ 实时，evidence 协议显式运行），按设计不计入默认验证。
- `cargo fmt --all -- --check` clean；workspace guarded-clippy `-D warnings` 清零；`cargo check -p zero-media-host --target aarch64-linux-android` 通过（android 编译腿成立，优于切片 4 静态核对口径）。

## 残余

- audio-cpal 档零运行时验收（无消费者启用 feature；PulseAudio socket 面见上 FIXME）。
- macOS 真机验收环境受限维持记录（seccomp/landlock Linux-only，降级日志明确）。
- media-host 处 on-demand spawn（无媒体内容不拉起），生产 GUI 长跑复测归 media-runtime 链路既有证据协议，本切片以真实二进制集成测试钉行为。
