# 媒体播放 — 运行时控制面板（master.md）

**入口文档**: [../media-playback.md](../media-playback.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（M0 四切片全部完成——RFC 已起草待用户审批
[docs/specs/video-decode-playback-spec-rfc.md](../../specs/video-decode-playback-spec-rfc.md)，
fixture 已落地；批准前不动源码）

---

## 当前状态

**专项定位**：媒体方向三拆之二（门控流）。视频解码与帧渲染——「占位框 → 能播放」的一跳。
**M0 已完成**：RFC 起草（三路线对比 + 推荐路线 C「VP9/AV1 开源先行 + 进程内 crate」+
分阶段里程碑 + 决策点 + 风险回滚），**待用户审批**（D-RFC-1~3）——批准后解锁 M1+。

**与兄弟 goal 的边界**：
- media-elements — 语义面（状态机/事件/canPlayType）归其管；本目标产出 readyState 真实
  驱动接口（`VideoClock` trait，其 headless 近似驱动届时替换，语义层不返工——RFC §3.1）
- media-audio — 音频输出/A/V 同步归其管（其 M0 已收口，AudioSink trait 验证策略成立）；
  本目标首期静音播放（video clock 驱动），音频解码面 M2c 经其 AudioSink 接入
- js-dom — 媒体反射段共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项 + 2026-09-01 M0 更新）

### 现有实现

- ✅ 架构先例：image-decoder 独立进程（D1）+ zero-protocol IPC（`ImageDecodeParams/
  Result` 字节进 RGBA 出——视频解码进程升级时同构扩展）+ compositor（C2）
- ✅ 渲染通路：canvas 像素 → 页面图元桥接（R3268）——帧位图可走同款
- ✅ event loop 帧驱动：rAF（P1a）——播放时钟可挂
- ✅ **e2e 资产已备**（V5 闭合）：`tests/fixtures/media/` 四 fixture（h264+aac mp4 /
  vp9 webm / mp3 / opus oga，ffmpeg 生成、来源清白、生成命令入 README）
- ✅ crate 生态调研数据：symphonia 0.6（纯 Rust 容器+音频）/ dav1d 0.11（AV1 绑定）/
  openh264 0.9 / ffmpeg-next 9.0 / rav1e 0.8（crates.io 实测版本）
- ⚠️ 零解码能力（无视频解码依赖与管线）——M1 实施项（门控中）
- ⚠️ 选型待批（RFC 草案已交）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| V1 | 解码路线选型（专利/依赖/架构三维） | ✅ 调研完成 → **RFC 待批**（D-RFC-1） |
| V2 | 零解码管线（demux/解码/帧转换） | ⬜ M1（门控：RFC 批准） |
| V3 | 播放驱动（帧率时钟/seek/ended）缺失 | ⬜ M2 |
| V4 | readyState 真值驱动接口未建 | ⬜ M2（VideoClock trait——M1a 先行定义） |
| V5 | 播放 e2e 资产为零 | ✅ fixture 已落地（2026-09-01） |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | **RFC 审批**（路线 C：VP9/AV1 开源先行 + 进程内 crate；附 D-RFC-2 AV1 时点、
  D-RFC-3 H.264 增量立项——见 RFC §5） | ⬜ **RFC 已起草待批——批准后解锁 M1+** |

## 下一步计划

1. **等用户审批 RFC**（本仓 Mission 级门禁，不自主解锁）。
2. 批准后 **M1a**：webm demux + VP9 解码 + YUV→RGBA（fixture 帧哈希单测）
   + `VideoClock` trait 先行定义。
3. 批准后 **M1b**：首帧上屏（video 元素盒渲染通路，R3268 同款）+ e2e 常驻。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 解码器选型 RFC（门控） | ✅ 四切片完成（2026-09-01）——**RFC 待批** |
| M1 — 首个视频帧上屏 | ⬜ 门控：RFC 批准 |
| M2 — 连续播放 + 语义驱动 | ⬜ |
| M3 — 多格式 + 稳定 + 收尾 | ⬜（含 H.264 增量决策点 D-RFC-3） |

## 验证基线

- 测试基线：立项时点全绿；clippy 零警告（本目标至今零源码改动）
- 播放 e2e 面：fixture 四件已入仓（`tests/fixtures/media/`）；e2e 断言形态见 RFC §3.3
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
