# C2 切片 2——registry 客户端切换与宿主面端到端验证

**日期**：2026-10-25
**状态**：PASS（registry 开态全宿主面 + 端到端契约 + M1 双 feature 零回归）
**goal**：[media-runtime-isolation](../../master.md)
**设计**：[m3-prep-design DESIGN §10.4/§10.5](../2026-10-24-m3-prep-design/DESIGN.md)（偏差记 §10.8 偏差 9–12）

## 1. 版本与环境

| 项 | 值 |
|---|---|
| 基线 | C2 切片 1 落地后 main（`0ebbfda50`） |
| 平台 | Linux 6.18.33.2-microsoft-standard-WSL2，AMD Ryzen 7 9700X（16 线程），39GB RAM |
| 工具链 | rustc 1.99.1 / cargo 1.99.1，edition 2024 |
| 测量档 | release（与 2026-10-24 M1 基线同口径；debug 档解码慢 ~20× 不作界依据） |
| 隔离与守卫 | 编译阶段无阈值；执行阶段 `./target/test-guard --per-proc-mem 4 --total-mem 8`；clippy 走 `make guarded-clippy`（24G scope，-D warnings） |

## 2. 落地面

- **`HostMediaFacade`**（`crates/webview/src/video_registry_host.rs`，新文件，`media-process` 门控）：registry 命令面转发 `MediaHostClient`（settle 全量 8MiB 分片喂入），真值面读 push 镜像（DC-2 epoch 门控、未知键丢弃）；帧经 `tick_into` 排空注入 `ImageCache`（painter 同键）；纯视频 Ended+loop 回卷待办收集 + `flush_replays` 统一 play(0)（内核 replay 链，镜像时钟先归零防 seek 回流末）。
- **registry 双形态分流**（`video_registry.rs` 21 个公开面 `hosts(key)` 早退/并集形态）：登记不落内联字节（IF-006 不回落）；MSE 键不经登记天然内联；`tick_all` 并集形态保 MSE 内联 player 不被宿主排空短路。
- **client 面**（`media_proxy.rs`）：play/pause 回执返回 `MediaStateUpdateParams` 载荷（镜像即时真值，消除 250ms 心跳盲区）；新增 `set_rate`/`set_gain` fire-and-forget。
- **play 回执后即时排空**：内核 `SourceReady` 先于 `Playing` 回执同批出队——JS `play()` 后立读 duration/currentTime 即在真值；排空窗帧缓冲到下一 tick 注入（不丢首帧窗）。
- **M1 harness 时钟分叉**：`media-process` 开态真实节拍（16ms 对齐生产泵），关态合成加速钟原样。
- **桥拆分**：`video_registry_bridge.rs`（JS 门面 + native 回调注册，532 行）自 video_registry.rs（2212→1817 行，单文件行数纪律）拆出，外部路径经再导出不变。

## 3. 验证结果

### 3.1 端到端契约（真实二进制，debug 档，`--ignored` 5.0s 全绿）

`crates/webview/tests/media_host_contract.rs` 5 场景：控制面错误面 / 视频 play→元数据→帧→seek 代际→暂停冻结 / A-V / 纯音频 set_loop 内核回卷 / **hosted_registry_end_to_end**（registry 全宿主面：登记建档→play 回执即时 Playing→tick 排空帧入 ImageCache→duration 2.0/(320,240) 升级→seek 镜像时钟=resolved→暂停冻结→纯视频 loop 回卷观测（时钟 >1.5s 回 <0.5s 且保持播放态）→clear 清镜像）。

### 3.2 M1 release 档（`--test-threads=1`，宿主面 vs 内联基线）

| 场景 | 内联（2026-10-24 基线） | 宿主面（本轮） | 含义 |
|---|---|---|---|
| 纯视频 60s 墙钟 | 1079ms（合成加速 55×） | 59983ms（真实节拍 1×） | 内核真实墙钟推进 |
| 测试进程 CPU | 1080ms（cores 1.001） | 690ms（cores 0.012） | 解码 CPU 移出 renderer 进程（宿主进程 CPU 未计入） |
| 每 tick p50 | 431ns（解码+注入） | ~3.7µs（排空+注入） | renderer 侧纯消费成本 |
| 呈现帧 | 1440/1440 | 1440/1440 | 无丢帧 |
| 时钟保真 rate | 1.0（A1 对齐） | 0.99827（offset p95 49ms） | 宿主面同带 |
| seek 冷跳 15s | 18.2ms | 19.1ms | 内核窄化路径并源（差 <1ms） |
| A/V 漂移 | p50 11ms | p50 11ms | media 层不变 |

**内联零回归**：feature-off release 套件 5/5（19.99s），pure-video 1079ms / A4 18226/18243µs 与基线同量级（[m1baseline-inline-release.log](m1baseline-inline-release.log)）。

### 3.3 门禁

- workspace `make guarded-clippy`（-D warnings）过；
- facade 单测 7（合成事件面：镜像跃迁/旧代际丢弃/帧注入与缓冲时钟/回卷待办/错误面/SourceReady 升级/release 丢推）；
- registry 内联套件 31 + 桥面 1（feature-off）全绿；
- 全量 `make guarded-test`：<见 master.md 执行记录>。

## 4. 限制与未决

- **bg02-media 站点复评**未做（外网依赖，A2 时已记限制，归后续验证切片）。
- 宿主进程侧 CPU/内存未单独计量（测试进程 cpu_ms 只证 renderer 侧解耦；C4 资源审计切片补宿主计量）。
- shm 帧传输面未接（内核发射全内联，`media_shm_enabled()` 无生产调用方；facade FIXME 标注，帧路径切片补 /dev/shm 读面）。
- debug 档冷 seek 方差大（425–527ms），A4 界仅 release 口径（DESIGN §10.8-10）。

原始输出：[m1baseline-host-mode-release.log](m1baseline-host-mode-release.log)（宿主面 release 全套 5/5）。
