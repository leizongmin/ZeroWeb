# M3 扩批 XIX（2026-09-03）— track-cues-* 播放推进族续批：解码器 EOF 排空缺陷修复

## 修复的根因（宿主实证链条）

### 根因 1 — `VideoDecoder::next_frame` 的 EOF 提前滞留（zero-media decode.rs）

**症状**：fixture-mounted runner 下 `track-cues-enter-exit` 三连跑全部 Timeout，
march ended 事件在媒体时间 ~3.5s 触发（流长 6.035s）——cue3@4.0-5.0s 永不 enter。

**归因路径**（探针实证）：
1. 逐帧顺序解码 `test.webm`：167 帧后 `Ok(None)`，末帧 pts=5525——容器时长 6035ms
   还有 14 帧滞留（ffprobe 实测视频块 181 个、末块 pts 5990）。
2. demux 层独立验证：181 块全可喂入；直接喂 rusty_vp9 + pull-all：181 帧全出。
   pull-one-per-push（`VideoDecoder::next_frame` 的调度形态）只出 166 帧。
3. 残帧去向：pull-one 终止后 pre-flush drain +6、post-flush +9 → 合计 181。
   **帧不丢，是被 `eof` 标志挡住**：demux `Ok(false)` 分支 flush 后只 pull 一帧
   即置 `eof=true`，后续调用 `eof → Ok(None)` 提前返回。
4. 根本机制：rusty_vp9 的 hidden/alt-ref 帧（show_frame=0）解码后返回
   `Error::Again`（不产出）——每次消耗一个 pull 机会但其 pts 帧晚一个 demux 块
   浮现，形成 ~15 帧的**输出流水线滞后**。demux 末队列积压 15 帧，被 eof 提前吞掉。

**修复**：`draining` 中间态——demux 耗尽只置 `draining`，残余帧经 `drain_frame`
（Again=隐藏帧继续拉，Eof=队列真空才停）逐帧产出，队列真空才置 `eof`。seek 路径
双分支同步重置 `draining`。

### 根因 2 — `present_pending` 未来帧消费丢失（zero-media player.rs）

**症状**：修根因 1 后模拟真实泵节拍（~15ms tick），player 仍在 position≈3.8s 转
Ended（应 6.0s）。

**归因**：`present_pending` 的循环拉取在遇到 `pts > position` 的未来帧时
`get_or_insert(frame)` 把它**返回给调用方**（tick_all 渲染后丢弃）——该时间槽
永久丢失。粗 tick 背压下每个大步进 tick 消费一个未来帧，逐 tick 累积使解码器
提前耗尽。

**修复**：`VideoDecoder::un_read(frame)` 队首退回 API（`pending` 槽复用）——
`present_pending` 遇未来帧退回而非返回。修复后模拟：181 帧全呈现、position=6.0
才 Ended。

### 根因 3 — march pauseOnExit 暂停后置（shim part06.js）

上游 `track-cues-pause-on-exit` 在 onexit handler 内同步断言
`assert_true(video.paused)`——spec time-marches-on 的 pauseOnExit 暂停须在 handler
内可观察。旧实现先 `dispatchEvent('exit')` 后置 `ms.playing=false`，handler 读到
paused=false。修复：暂停（含桥 pause）先于 exit 派发；handler 内 `video.play()`
照常续播。

### 根因 4 — pending seek 补推缺 seekSync（shim part03.js）

seek-before-play 时序（pause-on-exit：`currentTime=4.0` 早于桥接通）在 play() 桥
命中后只记 `_zwLastMarchMs` 不跑 `_zwMediaSeekSync`——起点恰在 seek 目标上的
cue0@4.0-4.5 永不 enter（`start > lastMs` 恒假），其 exit/暂停面全部缺席。修复：
两条补推路径（同步命中 + 退避重试命中）均调 `_zwMediaSeekSync(_pKey)`。

## 结果

- **track-cues-enter-exit**（+2 subtest）与 **track-cues-pause-on-exit**（+1 subtest）
  导入常驻；track-cues-seeking 评估后维持排除（逐次 seek 链的 currentTime 同步
  落位 + activeCues 计数窗口依赖 seek 事件真值化）。
- testharness-media：**534P/0F/24PF（534/558 = 95.7%）**（+3 净涨零回归）；
  track-cues 5 用例 4 连跑稳定全绿。
- zero-media 48 单测全绿（新增回归测试
  `webm_sequential_decode_drains_hidden_tail_frames_r3936`——顺序解码末帧 pts
  贴近容器时长 + eof 幂等面）。
