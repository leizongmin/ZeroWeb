# video 帧像素表面 FIXME 立项评估——跨域大件勘根盘点 + 分期提案（挂用户门）

- 日期: 2026-11-14；基线: HEAD e1ce76c9d（s423 收口），驱动: 红池分诊 CONTINUE
  指令「video 帧像素表面 FIXME 跨域大件立项评估」。本账为**评估轮（零源码
  改动）**，产出分期提案挂用户门，不在未立项前施工。

## 0. 结论摘要

- **基建已大半在场**：zero-media 真解码（webm VP9/AV1 + mp4 H.264 feature）、
  webview `VideoPlayerRegistry` 真播放、runner 播放泵逐帧解码注入 ImageCache
  **与 `__zw_get_image_wire` 同键**——帧像素到达画布 wire 的管线只差**消费臂**。
- **缺口集中在三层**：①shim canvas 三源臂（drawImage/createImageBitmap 无
  video 臂——drawImage(video) 现抛 TypeError；createPattern 为 1×1 占位
  wire）+ `__zw_get_image_wire` 快照陈旧（页装载期克隆，播放期新帧 miss）；
  ②decode 驱动的 resize/playing 事件链缺席（settle 链零维也派 resize、
  资产缺席不派 error——part06.js:13593；canPlayType 元素臂已真值化，仅
  interface stub 臂口径不一致）；③语料资产
  `400x300-red-resize-200x150-green.{webm,mp4}` 未 fetch（在册白名单
  外，上游 rev pin 在案）。
- **在册池收益 = 2F**（resize-during-playback mp4/webm 双子测，唯一直接受
  益红面）；其余为基建价值（2dcontext canvas 域未来车道 + 行为诚实性）。
- **建议分期**：P1 canvas wire 消费臂（本车道低风险、1 切片）→ P2 语料
  资产 fetch + settle 维度真值/缺席 error 语义核对（小件；canPlayType 已
  大半真值化，仅 stub 臂对齐）→ P3 decode 驱动 resize/playing 事件链
  （跨域大头，media corpus 209 文件回归面）。P1/P2 合并一切片可先行；
  P3 独立立项。
- **决策请求**：P1+P2 是否立项施工（挂 GB-20261114-VIDEO）；P3 建议另开
  RFC 评审（状态机统一改动面大，风险不在本车道独担）。

## 1. 现状架构盘点（已验证事实）

### 1.1 解码与播放基建（零缺口）

- `zero-media`：`VideoTrackDecoder::open_media` 容器嗅探路由——webm
  （VP9 rusty_vp9 / AV1 dav1d）+ mp4（H.264 OpenH264，`decode-h264`
  feature）（crates/media/src/decode.rs / av_decode.rs / mp4_h264.rs）；
  `VideoPlayer::tick` 产 `DecodedVideoFrame` RGBA（crates/media/src/
  player.rs:119）。
- webview `VideoPlayerRegistry`（crates/webview/src/video_registry.rs）：
  `register_source` settle 登记 → `play(abs_src, now_ms)` 建 player 真解
  码（:497-560，创建即探 media_meta w/h/duration——videoWidth 真值链）；
  MSE 面在场。
- runner 播放泵（tests/wpt-runner/src/testharness.rs:9969-9983）：
  `is_any_playing` 门控 `tick_all(now_ms, image_cache)` + `audio_advance_all`，
  与桥 play(0) 时钟原点同契约。
- 帧注入同键（video_registry.rs:1057-1067）：`tick_all` 变更帧
  `image_cache.insert_with_key(ImageKey(registry_key(abs_src)))`，而
  `registry_key` = `image_resource_key(bare, None)`（:31-33）——**与
  `__zw_get_image_wire` 读侧同一键约定**。
- shim play() 桥已接（part06.js:13301 `__zwVideoBridge.play(_pAbs, 0)`；
  loop 面 :20834）；videoWidth/videoHeight 解码器探针真值链在场
  （part04.js:545-551 经 resourceState settle 链消费）。

### 1.2 缺口面（勘根定位）

1. **shim drawImage 无 video 臂**（part05.js:11425+）：臂序 = ImageBitmap
   → img（`naturalWidth > 0`）→ canvas 元素；video 元素 `naturalWidth`
   非数值 → 落 `throw new TypeError('drawImage: invalid image source')`
   （:11509）。spec #dom-context-2d-drawimage video 源 = 以当前帧为位图
   绘制。
2. **shim createImageBitmap 无 video 臂**（part05.js:9294
   `_zwImageBitmapSourceToWire`：Blob/ImageData/img/canvas 四臂）→ video
   落 null → `TypeError` rejection（:9469-9470）。spec #createimagebitmap
   video 源合法。
3. **createPattern video 臂为占位 wire**（part05.js:10543-10570，s420）：
   `readyState >= 2` 门内以 `'1:1;0,0,0,0'` 1×1 透明 wire 建 pattern——
   污染语义面已真（双 URL OR 判），帧内容面未真（FIXME :10534 明示
   drawImage/createImageBitmap 同缺）。
4. **`__zw_get_image_wire` 快照陈旧**（webview.rs:4054-4056）：
   `image_cache.snapshot_entries()` 在 `run_page_scripts_impl` 装载期**一
   次性克隆**；播放期注入的新帧 miss 快照 → 回落 fetch 臂
   （:4090-4137）→ `decode_image_bytes` 只解码图像容器，video 容器解不
   出 → 空 wire。**video wire 臂须读活注册表/活缓存**，非快照。
5. **canPlayType 臂位口径不一致**（part03.js:32078 元素臂已真值化——
   webm/ogg/mp4 能力表 M4g-d + M3 切片 1；part03.js:23791 interface
   stub 臂仍恒 `''`）——元素主面无缺口，stub 臂对齐留 P2 小件。
6. **decode 驱动 resize/playing 事件链缺席**：resize 事件（videoWidth/
   videoHeight 变化时派，spec #event-media-resize）当前 settle 模拟链只
   覆盖 loadedmetadata 族；播放中维度变化（resize-during-playback 断言
   ②）无真值来源。playing 事件链现状未核（media corpus 单列）。
7. **语料资产缺席**：`/media/400x300-red-resize-200x150-green.{webm,mp4}`
   不在 wpt-data/media（16 文件白名单，fetch-media-subset.sh:473-506）；
   上游 rev pin 3159769 在案，资产 fetch 未验（GitHub raw 本轮直连超
   时——外部事实降级：**资产在 pin rev 是否存在待 fetch 时核**）。

### 1.3 在册池直接受益面量化（solo s424 实测修正）

- `the-video-element/resize-during-playback.html` 2F（mp4/webm 双子测，
  s423 双基线均稳定 Fail）。**solo s424 实测红形**（评估轮补探针）：
  canPlayType 两容器**均已真值放行**（part03.js:32115 能力表——webm VP9
  / mp4 H.264 M3 切片 1 落地，§1.2-5「canPlayType 恒 ''」仅部分臂位，
  元素臂已真值化）→ `wait_for(['resize'])` 解析（settle 链
  part06.js:13593 **video settle 即派 resize，零维也派**）→ 断言①
  `videoWidth expected 400 but got 0`（资产缺席 → 解码探针无真值）→
  Fail。**勘根附注**：资产缺席仍派 resize（不发 error）系 settle 链
  语义缺口（spec 资源获取失败 → error 事件），P2 范围应一并核对。
- 翻绿路径：断言① = 资产 fetch + settle 维度真值（P2，探针
  `source_media_meta`/media_meta 链在场，大概率随资产到位翻绿）；断言②
  = 播放中维度变化第二次 resize（P3 真播放驱动，泵无 dims→shim 事件桥
  不翻）。**子测级翻绿须 P3 全链**；P2 单独可翻断言①半面。
- 在册池其余 video 面已清：security.pattern video ×6（s420 占位臂绿）、
  video-ended settle 真值（s417）、video-loading-lazy-load-when-visible
  归渲染流（s423 排除）。**P1 单独在册池翻绿 0**——价值在行为诚实性 +
  2dcontext 域基建；P2 翻半面（断言①×2 子测仍 Fail）；全清须 P3。

## 2. 分期提案

### P1——canvas wire video 消费臂（本车道，低风险，1 切片）

**范围**：
- webview `__zw_get_image_wire` video 分支：src 经 `registry_key` 查活
  `VideoPlayerRegistry`（`video_players()` 锁）当前帧（presented 或
  latest decoded）→ `encode_image_wire` 同格式输出；注册表 miss（未
  settle/未 play）返空串走既有回落语义。
- shim drawImage video 臂：tag VIDEO + `readyState >= 2` 门 →
  `__zw_get_image_wire(currentSrc)` → 既有 3/5/9 参三 op 同路；污染面复
  用 s420 双 URL OR 判（`_zwCanvasSrcCrossOrigin(src)` + redirect.py
  静态解析）；`readyState < 2` no-op（spec 无帧不绘制）。
- shim createImageBitmap video 臂：`_zwImageBitmapSourceToWire` 第五臂
  同 wire 路径；`_zwBitmapUnclean` 同 s420 污染旗标传播。
- createPattern video 臂升级：占位 wire 换真 wire（readyState 门/污染面
  不动）。

**不改动**：img/canvas/ImageBitmap 既有臂、settle 模拟链、媒体状态机、
canPlayType（P2）、事件派发（P3）。

**验证**：shim 规范序拼接 node --check + unit（video wire 桥测三断言
面：play 后 wire 尺寸=解码帧尺寸、未 play 返空、污染传播）+ solo
the-canvas-element 目录零回归（132P 基线）+ 全域 sweep ×2 join 双基线
零 new 红。**在册池预期位移 0**（无帧内容断言红面）。

**风险**：低——纯加臂；既有 132 绿面（pattern 占位臂消费方）需盯
createPattern 升级后的 wire 尺寸变化（1×1 → 真帧），pattern.repeat 铺
贴面若语料有内容断言需复测（勘根未见，施工时 solo 验证收口）。

### P2——语料资产 + settle 真值面（小件，与 P1 同切片或紧随）

- canPlayType stub 臂对齐：part03.js:23791 interface 构造面 stub 恒 ''
  与元素臂真值表对齐（小刀；元素臂 M4g-d 真值表不动）。
- settle 链资产缺席语义核对：现缺资产派 resize 不派 error
  （part06.js:13593 无维门）——按 spec 资源获取失败 → error 事件核对
  settle 缺席臂（防 P2 资产到位前红形误读，亦系行为诚实性面）。
- 语料资产：fetch-media-subset.sh MEDIA_FILES 白名单追加
  `media/400x300-red-resize-200x150-green.webm` + `.mp4`（s227 流程：
  上游 sparse clone fetch + 脚本挂账同提交；pin rev 资产存在性 fetch
  时核验）。
- **风险**：资产到位 + settle 维度真值 → resize-during-playback 断言①
  翻绿但断言②仍红（P3 缺口，子测仍 Fail）；settle 缺席 error 臂改动波
  及 media corpus 缺资产行为面（209 文件——testharness-media 目标单跑
  核）。

### P3——decode 驱动 resize/playing 事件链（跨域大头，建议独立 RFC）

- **范围**：registry 播放泵维度变化检测（presented 帧 w/h ≠ 前值 → 派
  resize）+ shim 事件桥（host callback 或探针轮询面）+ play() 成功 →
  playing 事件真值链统一；media corpus event_* 族（209 文件在
  testharness-media 目标）模拟 settle 与真解码双轨并存面须逐一核对——
  **本 sweep pairs 不含 media corpus，回归面在 media goal 车道**。
- **收益**：resize-during-playback 2F 翻绿（P1+P2+P3 全链）+ media
  corpus 播放推进族若干潜在翻绿。
- **风险**：高——media-elements goal 档案库绿面建立在 settle 模拟 +
  真解码混合态上；状态机统一是跨 zero-media/webview/engine 三 crate
  改动，需独立 RFC + 双车道（html-semantics + media goal）联签验证。
  **本评估建议 P3 不并入本车道切片，另开评审。**

## 3. 决策请求（用户门 GB-20261114-VIDEO）

1. **P1+P2 立项与否**（建议立项：本车道内、低风险、行为诚实性收益明
   确；在册池位移 0 须明示接受）。
2. **P3 处置**（建议另开 RFC 不并入；若要求一并立项，须接受 media
   goal 车道联签验证与更大回归窗口）。
3. 决策记录后按 P1→P2→验证→落账推进；P3 决策维持挂账不阻塞 P1/P2。

## 4. 评估口径与证据锚

- 勘根基于 HEAD e1ce76c9d 源码直读（行号见 §1.1/§1.2）+ s424 评估轮
  solo 实测 9 文件（resize-during-playback 红形实测修正 §1.3；日志
  /tmp/s424/solo-*.log，runner 重建 2m14s exit 0）。
- canPlayType 现状修正：part03.js:32078 元素臂已真值化（webm/ogg/mp4
  能力表，M4g-d + M3 切片 1）；「恒 ''」仅存于 part03.js:23791 接口
  stub 臂（HTMLMediaElement interface 构造面）——§1.2-5 缺口降级为
  **部分臂位口径不一致**，P2 范围收窄为 stub 臂对齐。
- 外部事实降级项：上游 pin rev resize 资产存在性（fetch 时核）、
  playing 事件链现状（P3 RFC 时核）、Blink canPlayType 精确矩阵（P2
  施工时按需核）。
- 关联在册：s420 账本（pattern 占位臂 + 污染双 URL）、s417（settle 真
  值链）、part05.js:10534 FIXME（本评估标的）。
