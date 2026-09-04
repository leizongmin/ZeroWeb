# 媒体播放 — 运行时控制面板（master.md）

**入口文档**: [../media-playback.md](../media-playback.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-04（**GB-20260904 巡检——D3/D4 征询跟进**：cronjob 待决策
卡点专项合并征询（与 media-audio D3 同批）msg `om_x100b669923cd64a4c3e335615ed3d9f`
——D3 待批复跟进提醒 + D4 待点名随批告知，两行状态已补征询凭据。无代码变更。）
**（注：下方「最后更新」2026-09-04 泵时钟块为前轮记录，保留作历史）**
**最后更新**: 2026-09-04（**泵时钟生产路径注入（扩批 XXV 收口的 tab_worker 面）**：
tab_js_worker `SetVideoPlayers` 此前以 clock=None 注册宿主桥——shim 桥 play 恒传
nowMs=0，而 tab_worker 泵 tick 用 `pump_epoch.elapsed()`（原点错位：worker 启动后
首次桥 play 的首拍 delta=泵全程 → 位置瞬跳流末——runner 侧扩批 XXV 同款缺陷在
生产路径的残留）。修复：pump_clock（Arc<AtomicU64>）提前至 WebView 构建前创建 +
`SetVideoPlayers` 命令携带 + js_worker 注册传入 + 泵循环每拍 store——桥 play 锚与
泵 tick 同源。renderer 路径维持 None（D4 泵架构未决，「登记但不推进」现状不变，
非回归）。make test 18874/0、clippy/fmt 干净。此前同日：**M3 fixture-mounted
runner 播放面切片 12 落地（扩批 XXIX）**——HAVE_NOTHING 期 play() 挂起语义（spec dom-media-play 步 6）：play()
readyState==0 且有候选（按身份分派判据——handle-only 无 src 无 settle 序列
可达保持既有 queued task resolve 契约）→ 记 `_zwPlayPendingEvents` 挂起不派
事件；`_zwMediaLoadSequence` readyState 1 处补派 play、3 处（canplay 后）派
playing + promise settle——事件严格序 play→canplay→playing→canplaythrough。
既有「play 先行」同步态断言零回归（engine 契约测试同步序更新：settle 走
microtask 事件同步可观察——旧 queued task 宏任务面空 log 断言不再成立）。
media-elements 570P/0F/24PF（+10 净涨零回归——ready-states/autoplay 导入，
audio+video 各 5 子测：autoplaying flag 与 play()/pause()/load() 交互 +
事件严格序）。
此前同日：**M3 fixture-mounted runner 播放面切片 11 落地（扩批
XXVIII）**——currentTime-move-within-document 导入（同文档移动不重置播放：
seek(10) 后 appendChild 移动 paused=false + currentTime 保持——headless 时钟
推进面现成，零改动导入）+ track-mode-triggers-loading 导入（metadata track
mode 触发加载——扩批 XV 既有面零改动）+ track-remove-quickly /
-by-setting-innerHTML 导入（track 移除不 crash smoke 面）+ **track 空 src 语义
两处补全**（src=''空串 error settle code 4——spec「fail with attribute 之空
URL」；removeAttribute('src') 触发 track 重调度——与 setAttribute 对称）；
fixture 增 movie_300.webm（VP9 300s）。media-elements 560P/0F/24PF（+4 净涨
零回归）。track-element-src-change-error / -src-aborted-load 维持排除：
「加载中移除 src」in-flight 中断时序 headless 不可复现（settle 同步 microtask
无 in-flight 窗口，实证 settings.vtt onload 恒先于 removeAttribute）/
WPT trickle pipe 机制不可复现。此前同日：
**切片 10（扩批 XXVII）**——media fragment #t= 起点解析（settle 加载序列内 currentTime 初始化：
hash 内 & 分隔 k=v 对取 t=、percent-decode、npt: 前缀可选、start,end 取 start、
HH:MM:SS.ms/MM:SS.ms/SS；settle url 携带 hash 面已被 registry_key strip 兼容）+
**headless 播放时钟推进**（march 内非 bridgeOn 播放按 performance.now 墙钟差 ×
playbackRate 推进 ms.currentTime，clock 基点记 play 时——此前 headless 播放无
推进面，autoplay 驱动的播放 currentTime 恒 0）+ **周期 timeupdate**（march 内
nowMs > lastMs 时 250ms 节流派发——spec time updates 播放推进面，此前页面在
播放期无 timeupdate 可收）。media-elements 556P/0F/24PF（+4 净涨零回归——
media_fragment_seek + autoplay-with-broken-track 导入）。
此前同日：**M3 fixture-mounted runner 播放面切片 9 落地（扩批
XXVI）**——seekable/buffered TimeRanges headless 近似 getter 落地
（`__zwMediaSeekableRanges` 共享面：readyState>=1 后 [0,duration] 单区间；
duration 解析序 桥真值 → _mediaState → settle durationMs → headless 600；
HAVE_NOTHING 空集合 + IndexSizeError；has-trap 白名单补列）+ **currentTime
setter seek 语义补全**：clamp 到 seekable 范围（spec seek 步 5——镜像写 clamp
后值；duration 未知只 clamp 下限 0）+ seeking/timeupdate/seeked 同一排队任务
序派发（seeking 异步——后挂 onseeking 可达；seeking 翻 false 先于 timeupdate
——Chromium 可观察语义；事件序 [seeking, timeupdate, seeked]）。
media-elements 552P/0F/24PF（+9 净涨零回归——seeking/ 三件 + volume_nonfinite
导入；buffered/seekable「上游无断言用例」的旧注记失效——seeking/ 目录即断言
面，DC-3 buffered/seekable 注记项收口）。
此前 2026-09-03：**M3 fixture-mounted runner 播放面切片 8 落地（扩批
XXV）**——loop-from-ended.tentative 导入 + 四处播放面缺陷收口：
① **registry Ended→play 解码器重建**——`play` 的 player Ended 态经 sources/
av_sources 留存字节 `reset()` 重建 + 伴生轨游标/静默线归零（此前直接置 Playing
下一拍即再 Ended——解码器单向流耗尽，重头播放从未真正工作）；
② **seek 游标 clamp**——av entry seek 后游标 clamp 到 player clamp 后位置
（语义层以 headless 600 算的 seek 目标可超真实流长，audio clock 主时钟游标超界
把视频位置拉出流末）；
③ **泵时钟注入**——`install_playback_bridge_with_clock`：桥 play 的 nowMs=0
（shim 无钟）翻译为宿主泵时钟现值，播放锚与 tick 同源（原点错位使首拍
delta=泵全程、位置瞬间跳到流末）；
④ **shim ended 态 play 语义**——play() 命中 `_zwEndedDispatched` 标记时派
seeking/seeked 回最早位置（spec「ended playback」步 6.4 在 play 入口生效；
loop setter 翻 ended 后 ms.ended 不可靠，以 march 非 loop 分支标记为判定面）+
loop=true IDL setter 翻回 ended=false（looping 媒体不能是 ended）。
media-elements 543P/0F/24PF（+1 净涨零回归——loop-from-ended 导入）。
此前同日：**切片 7（扩批 XXIV）**——loop 属性真面 + played TimeRanges：registry `set_loop`（音频 entry
流末回卷——`restart()` 解码器重建 + 游标归零 + 播放态保持；伴生轨同面；
`reached_end` 标志补音频面 isEnded 驱动源——此前音频流末对桥不可见）+
`registry_key` 规范化（strip query/fragment——WPT cache-buster query 与 runner/
shim 两侧 URL 编码差异同键命中；bridge play 的 audio_guess 同面——`.oga?...`
此前恒 miss 使音频条目永不登记）+ shim loop IDL setter/getter + march Ended 面
loop 分叉（seek(0)+play + seeking/seeked 派发非 ended）+ played TimeRanges
（march 采样 `_zwPlayedRanges` → getter TimeRanges 形状；loop 尾段计入）+
duration getter settle 竞态兜底。**setLoop 桥回调参数索引修复**
（args.get(2)→get(1)——门面传 2 参，此前 on 恒 false 使 loop 真面从未生效）。
media-elements 542P/0F/24PF（+2 净涨零回归——played-loop /
audio_loop_seek_to_eos 导入）。此前同日：**M3 fixture-mounted runner 播放面切片 6 落地**——
media load invoke 重置面收口：`_zwMediaScheduleLoad` invoke 入口重置
`_resourceStates[key]` + invoke 步 6 位置重置（currentTime=0 / HAVE_NOTHING / 
`_zwMediaTimeKnown` 失效）+ invoke 重置 track 子产物 cue（addTextTrack 产物
排除）+ settle 的 media/track 元素 load/error 派发改 `_zwMediaFire`
（handle-only 元素 on\* expando handler 兜底）；track-active-cues 导入——
**B 组排除件全清**；media-elements 540P/0F/24PF。此前同日：**切片 5**——
play() 桥 src 读身份分派（handle 身份走 registry 现值——createElement 媒体元素
形态的桥失联修复）+ march 遍历面统一（addTextTrack 产物 cue 调度）+ disabled
gate + cuechange 派发；media-elements 539P/0F/24PF。此前同日：**切片 4**——
HAVE_NOTHING 期 seek 挂起语义：currentTime setter readyState 0 时挂
`_zwSeekDeferred`，`_zwMediaLoadSequence` readyState 0→1 翻转时补跑 seek 算法
（spec「default playback start position」）；track-cues-seeking 导入；media-elements
535P/0F/24PF。此前同日：**M3 fixture-mounted runner 播放面切片 3 落地**——
解码器 EOF 排空缺陷修复：`VideoDecoder::next_frame` draining 中间态（demux 尽后
排空 hidden/alt-ref 帧滞后队列才报流末）+ `VideoPlayer::present_pending` 未来帧
`un_read` 队首退回——此前 position < duration 即提前 Ended（fixture-mounted
WPT 流的最小暴露面：test.webm 30fps + 15 个 alt-ref hidden 帧）；media-elements
534P/0F/24PF（+3 净涨——track-cues-enter-exit / pause-on-exit 解除排除）。
此前同日：**M3 fixture-mounted runner 播放面切片 2 落地**——
track-cues-* 播放推进族解锁：runner 播放桥前置 + 逐 tick 动态源登记 + shim play()
latest-wins 读/退避重试/pending seek 补推 + registry 字节留存/is_ended 桥面 +
march 区间捕获/事件时间序/ended 面；media-elements 531P/0F/24PF（+2 净涨）。
此前 2026-09-02：**M3 fixture-mounted runner 播放面切片 1 落地**——webview
`install_playback_bridge` + wpt-runner 播放泵/源登记 + shim `_zwMediaTimeMarchesOn`
cue 调度钩子；webm A_OPUS 解码切片（WebmOpusAudioTrack + registry codec 泛化 +
canPlayType webm-opus 扩表）；media-elements 529P/0F 零回归。此前同日：**M3 AV1
解码切片落地 + H.264 立项 RFC 起草**——
AV1：dav1d 绑定 feature `decode-av1` + VideoCodec 自路由 + fixture 48 帧全解
（ffmpeg 参照 ±15 窗）；H.264：[h264-increment-project-spec-rfc.md](../../specs/h264-increment-project-spec-rfc.md)
Proposed 态——D-RFC-3a 专利授权链 / 3b OpenH264 分发形态 / 3c AAC 随期 三决策
点**待用户批复**（D-RFC-3「单独立项」决议的立项评估文档，批准前不动源码）。
此前 2026-09-01：D2 获批（选 A：libdav1d-dev 1.5.1 在位，pkg-config 发现，
apt 清单已记入 development/linux-macos.md））

---

## 当前状态

**专项定位**：媒体方向三拆之二（门控流）。视频解码与帧渲染——「占位框 → 能播放」的一跳。
**M0 已收口**：RFC 获批（2026-09-01，路线 C「VP9/AV1 开源先行 + 进程内 crate」）。
**M3 AV1 解码切片已落地（2026-09-02，D-RFC-2）**：`crates/media` 新模块
`av1_decode`（feature `decode-av1` 门控）——dav1d 安全 Rust 绑定（系统 libdav1d
1.5.1）+ Matroska V_AV1 轨 low-overhead OBU 喂入；`decode.rs` VideoCodec enum
codec 自路由（新 `open_webm`：V_VP9 → rusty_vp9 / V_AV1 → dav1d，feature 关闭
回落 NoVideoTrack 占位面；`open_webm_vp9` 原样保留零回归）；YUV→RGBA 提为通用
`planes_to_rgba`（VP9/AV1 共用，M2 色度面单点维护）；webview `video_registry.play`
切换 `open_webm`（生产播放面 codec 无关）。fixture `sample-webm-av1.webm` 48 帧
全解、PTS 单调、首帧 RGB 均值与 ffmpeg 7.1.5 RGBA 参照（123.26）同窗对齐 ±15。
media 40 单测（default）/ 42（decode-av1）全绿、webview 678 全绿、clippy 双态零警告。
**AV1 settle 探针接通 + canPlayType 扩表（2026-09-02 补片，跨 goal 联动兑现）**：
async_load `probe_video_media_meta` 从 `open_webm_vp9` 切 `open_webm`（与播放面
同一 codec 自路由入口）——修复「play 可路由而 settle 探针 VP9-only」的分叉
（AV1 源 settle 后 duration 真值 + 首帧注入缺失）；webview `decode-av1` feature
转发 + AV1 settle e2e（`video_settle_av1_first_frame_and_truth_m3`，feature-gated）；
media-elements canPlayType 能力表扩 av1（video/webm → probably——M4g-d
「新增解码面同步扩表」注记兑现）。media-elements 面 510P 维持零回归。
（**M1a/M1b/M2a 切片 2~5b/M2b/M2c/M2 切片 C/D/E 历史过程段**——解码管线/帧上屏/
  播放驱动/真值注入/registry/桥接线/色彩面/音频面/A-V 同步各片明细——已归档至
  [archive/2026-09-04_m3-fixture-mounted-slices.md](archive/2026-09-04_m3-fixture-mounted-slices.md)；
  实测基线节保留各片交付面的现状清单。）

**与兄弟 goal 的边界**：
- media-elements — 语义面（状态机/事件/canPlayType）归其管；本目标产出 readyState 真实
  驱动接口（`VideoClock` trait，其 headless 近似驱动届时替换，语义层不返工——RFC §3.1）
- media-audio — 音频输出/A/V 同步归其管（其 M0 已收口，AudioSink trait 验证策略成立）；
  本目标首期静音播放（video clock 驱动），音频解码面 M2c 经其 AudioSink 接入
- js-dom — 媒体反射段共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项 + 2026-09-01 M0/M1a/M1b 更新）

### 现有实现

- ✅ **解码管线（M1a）**：`crates/media`（`zero-media` crate）——webm/Matroska
  demux（`matroska-demuxer 0.8`，纯 Rust）+ VP9 解码（`rusty_vp9 0.1`，纯 Rust 零 C
  依赖）+ YUV→RGBA（BT.601，8/10/12bit 与 4:2:0/4:2:2/4:4:4 面宽）；单测 5 + doctest 1
  常驻（fixture 帧数/PTS 单调/像素窗口/确定性/拒收非 webm）
- ✅ **帧上屏通路（M1b）**：painter `paint_video_element`（ImagePrimitive，解码像素
  gate）→ pipeline video 固有尺寸段 → layout-engine replaced sizing 白名单 →
  wpt-runner `load_video_first_frames`（harness 侧解码注入）；e2e 双测常驻
  （`m1b_video_first_frame_renders_to_framebuffer` 正例 + undecodable 负例）——
  证据：[evidence/2026-09-01-m1b-first-frame-on-screen.md](evidence/2026-09-01-m1b-first-frame-on-screen.md)
- ✅ **播放驱动接口（M1a 定义）**：`VideoClock` trait（currentTime/duration/is_playing/
  playbackRate）——M2a player 模块实现、语义层对接点
- ✅ 架构先例：image-decoder 独立进程（D1）+ zero-protocol IPC（`ImageDecodeParams/
  Result` 字节进 RGBA 出——视频解码进程升级时同构扩展）+ compositor（C2）
- ✅ 渲染通路：canvas 像素 → 页面图元桥接（R3268）——M1b 已按同款两段式落地
- ✅ event loop 帧驱动：rAF（P1a）——播放时钟可挂
- ✅ **e2e 资产已备**（V5 闭合）：`tests/fixtures/media/` 四 fixture（h264+aac mp4 /
  vp9 webm / mp3 / opus oga，ffmpeg 生成、来源清白、生成命令入 README）
- ✅ crate 生态调研数据：symphonia 0.6（纯 Rust 容器+音频）/ dav1d 0.11（AV1 绑定）/
  openh264 0.9 / ffmpeg-next 9.0 / rav1e 0.8（crates.io 实测版本）；M1a 实测补充——
  `rusty_vp9 0.1.1`（纯 Rust VP9，Apache-2.0，MSRV 1.85）首帧与 ffmpeg 逐字节一致，
  `matroska-demuxer 0.8.1`（Zlib OR MIT OR Apache-2.0）API 干净（双许可证均兼容工作区 MIT）；
  M2c opus 面实测补充——`opus-decoder 0.1.1`（纯 Rust RFC 6716/8251 decoder，零 unsafe
  零 FFI，仅依赖 thiserror，MIT OR Apache-2.0，MSRV 1.85，conformance 测试常驻）入
  workspace 依赖
- ✅ **生产侧帧注入 + 播放桥（M2a 切片 4/5 + M2c 后续 A/B）**：settle 首帧注入
  ImageCache + 源字节登记；`VideoPlayerRegistry` + `__zwVideoBridge` 宿主桥 +
  tab_worker 帧泵（切片 5a/5b）；settle 登记生产链路补全 + renderer 多进程路径
  SetVideoPlayers 对齐（M2c 后续切片 A/B）——双路径（tabworker/renderer）一致性
- ✅ **音频播放面（M2c 后续 A/B）**：`<audio>` settle 登记 → 桥 play → 音频泵实时
  节奏解码写 NullSink + volume/muted 增益联动（IDL setter 桥推 + play 起播同步）+
  seek 追赶区静默；导航释放（DC-4）
- ✅ **色彩面（M2 切片 C）**：WebM Colour 解析 + identity/BT.709/BT.601 矩阵 +
  limited/full 值域自适应转换；色度采样索引与值域两处旧缺陷修复——与 ffmpeg
  swscale 参照对齐；replaced-element-003 unmask 收口
- ✅ **A/V 同步面（M2 切片 D+E）**：webm 双轨伴生音频（OGG 重封装 → symphonia）+
  audio clock 主时钟（视频帧调度 sync_to_media_time 对齐音频游标）+ currentTime
  组合时钟 + seek 双轨对齐（media-audio M2 契约兑现——drift 构造校正）
- ⚠️ AV1（dav1d 绑定）与 H.264 未引入——M3（D-RFC-2 / D-RFC-3 决议）

## 深结构缺口发现（2026-09-02，Web Audio 多进程接线巡检）

**renderer 路径无播放泵**（记录不修——深结构）：browser tab_worker 主循环有 1ms
帧泵/音频泵（`is_any_playing` 门 → `tick_all` + `audio_advance_all` + WebAudio
`wa.advance`）；renderer 路径 M2c 切片 ⑤ 仅对齐了**桥面**（`__zwVideoBridge` 注入
js_worker）而**从未 tick**——renderer 的 VideoPlayerRegistry Arc 注入后无消费方，
play 真值面在 renderer 播放请求可登记但帧/音频永不推进（`is_any_playing` 恒真
即自旋、无泵即永不推进——实际为「play 登记后静默」）。Web Audio 同理：本轮已补
`SetWebAudio`/`__zwWA*` 注入（桥面一致性），但 `WebAudioRegistry.advance` 无
renderer 主循环节拍驱动。**修复方向**：renderer 主循环（`runtime.run`，当前
事件驱动无固定节拍）引入播放泵节拍或迁移独立泵线程——架构决策域（进程内线程
模型 vs 事件循环节拍），待用户点名后实施；在此前 renderer 路径播放面维持
「登记但不推进」现状（与 M2c 切片 ⑤ 交付态一致，非回归）。

## DC 达成审计（2026-09-02，对照入口文档 Done Criteria 逐项核验）

**DC-1（选型 RFC 已批准并落地）✅**：主 RFC 获批（D-RFC-1/2/3 三决议，2026-09-01）；
实现与 RFC 一致（路线 C：进程内 crate + feature gate——`decode-av1` 默认关、VP9
纯 Rust 主线）；偏离处零（dav1d 为 RFC §4 明示的 M3 面）。H.264 增量按 D-RFC-3
「单独立项」决议起草独立 RFC（Proposed，待批复）——不属 DC-1 范围。

**DC-2（首个视频端到端播放）✅**：① 真实 fixture → demux → 解码 → 首帧上屏
（M1a/M1b，`load_video_first_frames` + settle e2e 双测常驻）；② 连续播放
（M2a VideoPlayer：帧率驱动/currentTime 推进/play/pause/seek/ended——单测 6 件
+ 桥 e2e）；③ 帧渲染走页面图元通路（painter `paint_video_element` →
ImagePrimitive，与 canvas R3268 同通路）。

**DC-3（语义驱动真值化）✅（buffered/seekable 注记）**：duration 真值注入链全通
（M2a 切片 2——容器时长 → settle → shim `_zwMediaLoadSequence`）；readyState 由
settle 事实驱动（headless 加载序列推进 HAVE_METADATA→HAVE_ENOUGH_DATA）；videoWidth/
videoHeight 解码器探针真值（M2a 切片 3）。**buffered/seekable TimeRanges headless
近似面已收口（2026-09-04 扩批 XXVI）**——`__zwMediaSeekableRanges`（readyState>=1
后 [0,duration] 单区间 + IndexSizeError）+ seeking/ 三件断言用例导入（此前「上游
无断言用例」注记失效——seeking/ 目录即断言面）；真值化依赖真解码流的缓冲区间
追踪（随播放面背压优化一并评估，记录为后续项）。

**DC-4（多格式 + 稳定性）🔄（余 WPT 子集导入）**：① 选型面内容器/编解码 e2e
（VP9 单轨/双轨 + AV1 decode→settle→play→canPlayType 全链 ✅）；② 资源生命周期
（`prepare_document_state` 清空注册表 + `clear()` 单测——DC-4 导航释放面 ✅）；
③ **上游 WPT 可执行子集导入未启动**（master.md 下一步 #1 尾项——runner 桥注入
可行性分析已记：WPT corpus 无 settle 真源，注入后行为零变化；待 fixture-mounted
runner 播放用例面评估，随 D-RFC-3 批复状态一并决策）。

**DC-5（测试与质量不可退让）✅**：make test 18694 全绿（2026-09-02 组合树实测）、
clippy 零警告、每切片带单测 + e2e/fixture 资产化（AV1 全流单测 + settle e2e +
桥 roundtrip）。

**结论**：DC-1/2/3/5 满足；DC-4 余 WPT 可执行子集导入一项（外部门控：D-RFC-3
批复影响其形态——H.264 批准则 mp4 面 WPT 用例可随实施导入，不批准则 webm 面
维持 headless 饱和态）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| V1 | 解码路线选型（专利/依赖/架构三维） | ✅ RFC 获批（路线 C，2026-09-01） |
| V2 | 零解码管线（demux/解码/帧转换） | ✅ M1a 落地（2026-09-01，`zero-media` crate） |
| V3 | 播放驱动（帧率时钟/seek/ended）缺失 | ✅ M2a + M2b 落地（时钟/play/pause/ended/精确 seek/playbackRate 变速桥接） |
| V4 | readyState 真值驱动接口未建 | ✅ 5b 落地：duration/videoWidth/currentTime 真值 + 宿主桥 play/pause + 帧泵（readyState 推进面仍 headless 序列——语义层契约不返工） |
| V5 | 播放 e2e 资产为零 | ✅ fixture 已落地 + M1b 帧上屏 e2e 常驻 |
| V6 | 帧上屏通路（video 元素盒 → 图元）缺失 | ✅ M1b（harness 全链）+ 切片 4（生产 settle 注入）+ 切片 5b（播放帧泵） |
| V7 | 色度元数据精化（Colour 元素/limited-range/BT.709） | ✅ M2 切片 C 落地（2026-09-01）——replaced-element-003 unmask 收口，reftest-upstream 83.6% |
| V8 | A/V 同步（audio clock 主时钟）缺失 | ✅ M2 切片 D+E 落地（2026-09-01）——webm 双轨伴生音频 + 视频帧调度对齐音频游标 + currentTime 组合时钟 + seek 双轨对齐（media-audio M2 契约） |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | **RFC 审批**（路线 C：VP9/AV1 开源先行 + 进程内 crate；附 D-RFC-2 AV1 时点、
  D-RFC-3 H.264 增量立项——见 RFC §5） | ✅ 获批（2026-09-01）——三项决议见 RFC §5 |
| D2 | **AV1 dav1d 依赖引入方式**（M3 解锁前置）：本机实测——系统有 libdav1d7 运行时
  （.so.7）但**无 libdav1d-dev 头**（pkg-config 找不到）；`dav1d 0.11` crate 的
  dav1d-sys 走 system_deps：优先 pkg-config 系统库，缺则**从源码构建**（git clone
  videolan/dav1d + meson + ninja——本机 meson/ninja 均未装）。两条路都需系统级安装
  （`apt install libdav1d-dev` 或 `apt install meson ninja`），按 run-rules 须用户
  批准；三平台 CI 构建矩阵成本同 RFC §6 风险面 | ✅ 获批选 A（2026-09-01，
  GB-20260901 批复）——`libdav1d-dev 1.5.1-1` 已装，pkg-config 发现 dav1d 1.5.1；
  apt 清单已记入 [docs/development/linux-macos.md](../../development/linux-macos.md)（不阻塞 M3 其余面——
  WPT 子集导入可先行） |
| D3 | **H.264/AAC 增量立项批复（D-RFC-3a/3b/3c）**——立项 RFC 已起草
  （[h264-increment-project-spec-rfc.md](../../specs/h264-increment-project-spec-rfc.md)，
  2026-09-02 Proposed）：推荐路线 A（Cisco OpenH264 `openh264 0.9` 安全 Rust 绑定 +
  symphonia aac feature 扩展；本机探针实证 48/48 帧解码）；三决策点：
  **3a** 专利授权链是否接受（核心门禁——MPEG-LA/Via 池面，Cisco AVC Patent Trust
  License 授权链 + 源码编译态确定性注记，本 RFC 不构成法律意见）；
  **3b** OpenH264 分发形态（① 构建期源码编译【推荐，与路线 C 轻依赖一致】/
  ② 官方预编译二进制——授权链最强但分发矩阵成本回潮）；
  **3c** AAC 是否随期（推荐随期——symphonia feature 扩展成本 ≈0）。
  为何需用户：专利/授权属 Mission 级决策（run-rules rule 11 + 主 RFC D-RFC-3
  「单独立项」决议），agent 不可代判 | ⏳ 待批复（2026-09-02 起草，飞书已征询
  msg `om_x100b664d8a6f44b0dee3398474de92b` + GB-20260904 巡检合并跟进 msg
  `om_x100b669923cd64a4c3e335615ed3d9f`；批准前不动源码；不批准亦请明示
  「维持不实施」以便归档） |
| D4 | **renderer 路径播放泵架构决策**（2026-09-02 深结构缺口发现，2026-09-02 巡检
  补入决策表）：browser tab_worker 主循环有 1ms 帧泵/音频泵（`is_any_playing` 门
  → `tick_all` + `audio_advance_all` + WebAudio `wa.advance`），renderer 路径桥面
  已对齐（VideoPlayerRegistry Arc + `SetWebAudio`/`__zwWA*` 注入）但**主循环无节拍
  驱动 advance**——play 登记后帧/音频永不推进（「登记但不推进」现状，非回归）。
  修复须架构决策：进程内独立泵线程 vs 事件循环节拍（renderer `runtime.run` 当前
  事件驱动无固定节拍）
  为何需用户：多进程线程模型属架构决策域（run-rules rule 11 深结构），待点名后
  实施 | ⬜ 待点名（此前仅记于「深结构缺口发现」块，决策表不可见——2026-09-02
  巡检补登；GB-20260904 巡检合并征询 msg `om_x100b669923cd64a4c3e335615ed3d9f`
  已随 D3 一并提醒；不影响 tab_worker 路径现有功能） |

## 下一步计划

1. **M3 多格式收尾**（当前首选）：~~AV1~~ ✅ 2026-09-02 落地（解码切片 +
   codec 自路由 + fixture 48 帧全解——见当前状态）；**H.264 立项 RFC 已起草
   （2026-09-02，[h264-increment-project-spec-rfc.md](../../specs/h264-increment-project-spec-rfc.md)
   ——Proposed 态，D-RFC-3a（专利授权链）/3b（OpenH264 分发形态）/3c（AAC 随期）
   三决策点待用户批复，批准前不动源码）**；上游 WPT 可执行子集导入。
   **M3 预备资产已落库（2026-09-01）**——
   `sample-webm-av1.webm`（libaom-av1 生成，README 命令记录）；matroska-demuxer
   实测可枚举 V_AV1 轨（CodecPrivate 在）——demux 面就绪，解码切片
   （dav1d 绑定 + `open_webm` track 路由 V_AV1）直接以本资产验证。
   **runner 桥注入可行性分析（2026-09-01）**：wpt-runner 沙箱可注入
   `register_video_bridge_callbacks`（tab_worker 同款）+ take_probe 泵 tick——
   但 WPT corpus 无 settle 真源（play() 桥 play 返 false 回落 headless），注入后
   行为零变化、无新增可跑用例；待 D2（AV1 fixture 资产面）落地后一并评估
   fixture-mounted runner 播放用例面。
   **切片 1 ~ 11 落地明细（2026-09-02 ~ 2026-09-04）**——runner 播放桥前置/
   动态源登记/play() 退避重试/EOF 排空/loop 真面/Ended→play 收口/seekable 面/
   media fragment/headless 时钟/同文档移动等各片明细——已归档至
   [archive/2026-09-04_m3-fixture-mounted-slices.md](archive/2026-09-04_m3-fixture-mounted-slices.md)
   （累计口径：media-elements 603P/0F/24PF；每片 evidence/单测见各 commit）。
   fragmented-mp4-end）。
2. ~~**A/V 同步精化余项**~~ ✅ 2026-09-01 收口：ended 面回归守卫落地
   （切片 F——伴音流末 video player 走到 Ended、泵停）；音频设备面（CpalSink
   真出声）挂 media-audio M1 可选切片。
3. ~~**opus 解码选型注记**~~ ✅ 2026-09-01 落地（`opus-decoder 0.1.1` 纯 Rust 面——
   评估结论：libopus 绑定族全部违反路线 C；pure-Rust 候选对比后选 opus-decoder
  （RFC 8251 conformant + conformance 常驻 + 零依赖）；音频输出格式面收口为
   mp3 + vorbis + opus 三编解码）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 解码器选型 RFC（门控） | ✅ 完成并获批（2026-09-01，路线 C） |
| M1 — 首个视频帧上屏 | ✅ 完成（2026-09-01：M1a 解码管线 + M1b 帧上屏通路 + e2e 常驻） |
| M2 — 连续播放 + 语义驱动 | 🔄 M2a + M2b + M2c + 切片 C/D/E/F 收口（播放/真值/桥/帧泵/seek/变速 + 音频面生产链路/增益/导航释放/renderer 对齐 + 色彩面全对齐 + A/V 同步 audio clock 主时钟 + A/V pair ended 面回归守卫）；余音频设备面（media-audio M1 CpalSink，可选） |
| M3 — 多格式 + 稳定 + 收尾 | 🔄 AV1 ✅（2026-09-02，D-RFC-2：解码/settle/播放/canPlayType 全链 + fixture e2e）；余 H.264 立项（D-RFC-3，RFC Proposed 待批复）+ WPT 可执行子集导入（外部门控：随 D-RFC-3 批复状态决策形态） |

## 验证基线

- 测试基线：`make test` 全绿（zero-media default 23 单测 + 1 doctest；engine 2539
  含桥契约测试；webview 667 含桥 e2e + registry 4 + settle e2e 2；browser 411 under
  xvfb）；clippy 零警告；testharness-media 372P/0F/41PF 基线维持（桥 feature-detect
  回落面零回归实证）
- 解码正确性锚点：fixture `sample-webm-vp9.webm` 首帧与 ffmpeg 7.1.5 rawvideo 参照
  逐字节一致（YUV 面）；全流 48 帧（2s @ 24fps）PTS 单调（0→1958ms）
- 上屏 e2e 锚点：帧区 RGB 均值 108-138（testsrc2 ≈123.3，M2 切片 C 后与 ffmpeg
  swscale 一致）+ 帧界外白底 + 不可解码 src 占位负例；reftest-upstream
  13981/16730（**83.6%**，replaced-element-003 unmask 已收口 ✓）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

## 归档

- [archive/2026-09-01_m0-m2-slices.md](archive/2026-09-01_m0-m2-slices.md) —
  M0 门控收口与 M1/M2 切片全链过程记录（只追加不修改；本控制面保留最新态）。
- [archive/2026-09-04_m3-fixture-mounted-slices.md](archive/2026-09-04_m3-fixture-mounted-slices.md) —
  M1a~M2b 历史过程段 + fixture-mounted 切片 1~11 落地明细归档
  （2026-09-04 治理切片；本控制面保留头链切片 5~12 摘要与 DC 审计/决策表）。
