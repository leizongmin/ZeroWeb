# 媒体音频 — 运行时控制面板（master.md）

**入口文档**: [../media-audio.md](../media-audio.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（M0 收口——环境验证 + headless 验证策略成文，见 evidence/
2026-09-01-m0-environment-probe.md；cpal 编译面阻塞于 libasound2-dev 缺失，列入待用户决策）

---

## 当前状态

**专项定位**：媒体方向三拆之三（门控最深）。音频输出（解码→混音→设备）+ A/V 同步 +
volume/muted 真控制。**双重启动门控**：① M0 音频环境验证与验证策略成文（自持，
**已完成**）；② media-playback M0 解码选型 RFC 获批（解码路线绑定其选型）。

**M0 已收口（2026-09-01）**：
- 环境实测：内核层 HDA 声卡在；**ALSA dev 头缺失（libasound2-dev 未装）→ cpal 默认
  ALSA host 无法编译**；PulseAudio dev 在但 server 不可用（Connection refused）。
- 验证策略成立：**`AudioSink` trait 抽象 + 双实现**——`CpalSink`（feature-gated
  `audio-cpal`，设备面）+ `NullSink`（headless/CI 默认，可观测：写入帧数 + 过零率
  频域代理断言）。M1 验收 = NullSink 可观测断言（CI 常驻）+ CpalSink 人工冒烟（可选）。
- cpal 编译实测须装 `libasound2-dev`（系统级变更 → 待用户决策 D2；不阻塞 trait/
  NullSink 层设计与实施）。
- 证据：[evidence/2026-09-01-m0-environment-probe.md](evidence/2026-09-01-m0-environment-probe.md)

**与兄弟 goal 的边界**：
- media-playback — 视频/解码选型归其管；A/V 同步接口对齐（audio clock 主时钟——契约记录
  于两流 master.md）；**解码面依赖其 RFC 选型，输出面（AudioSink trait）与选型解耦可先行**
- media-elements — 语义面归其管；volume/muted 本目标接真增益（IDL 语义已由其 M3 扩批
  III 落地：非有限 TypeError/同值短路/queued volumechange/load 清 pending）
- js-dom — volume/muted 反射段共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时 + 2026-09-01 M0 探测更新）

### 现有实现

- ✅ 反射底座：muted/volume 属性反射（R3040 + M3 扩批 III IDL 语义全对齐）
- ✅ 时钟底座：rAF 帧驱动（P1a）——音频时钟对齐可挂
- ✅ 环境/策略底座：M0 收口（AudioSink trait + NullSink 验证策略成文）
- ⚠️ 零音频管线（无 cpal/音频依赖，无解码/混音/输出代码）——M1 实施项
- ⚠️ cpal 编译面阻塞于 libasound2-dev（待用户决策 D2）
- ⚠️ 选型未对齐（待 media-playback M0 落地）
- ✅ 音频 e2e 资产：`tests/fixtures/media/`（sample-mp3.mp3 / sample-ogg-opus.oga，
  ffmpeg 生成、来源清白、生成命令记录于该目录 README）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| A1 | 音频环境验证 + headless 验证策略 | ✅ M0 收口（2026-09-01） |
| A2 | 解码选型未对齐（外部门控：media-playback M0） | ⬜ 等待 |
| A3 | 零音频管线（解码/重采样/混音/输出） | ⬜ M1（双门控解除后；trait+NullSink 层可先行设计） |
| A4 | A/V 同步机制缺失 | ⬜ M2（依赖 media-playback M2 视频时钟） |
| A5 | 音频 e2e 资产 | ✅ fixture 已备（mp3/oga + mp4/webm 见 tests/fixtures/media/） |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | AudioContext（Web Audio）最小面可行性 RFC → 是否实施 | ⬜ M3 时点提交 |
| D2 | 安装 `libasound2-dev`（系统级 apt 变更）以解锁 cpal 编译验证 | ⬜ 新增（M0 探测发现；不阻塞 NullSink 层） |

## 下一步计划

1. **M1 前置（双门控部分解除）**：AudioSink trait + NullSink 可观测层设计稿
   （与 media-playback RFC 的接口对齐记录——驱动源替换不返工原则同款）。
2. **等 media-playback M0 RFC**：解码选型获批后启动 M1 解码/混音/输出管线。
3. **等 D2 拍板**：装包后补 cpal 编译实测（CpalSink 冒烟记录追加到 evidence）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 环境验证 + 验证策略（门控） | ✅ 完成（2026-09-01，cpal 编译实测面留待 D2） |
| M1 — 首个声音输出 | ⬜ 门控：media-playback 选型落地（M0 已解除一半） |
| M2 — A/V 同步 + 控制 | ⬜ |
| M3 — `<audio>` 全路径 + Web Audio 评估 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿；clippy 零警告（本目标至今零源码改动）
- 音频 e2e 面：NullSink 可观测断言形态已定（写入帧数/过零率）；fixture 已备
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- evidence：[evidence/2026-09-01-m0-environment-probe.md](evidence/2026-09-01-m0-environment-probe.md)
