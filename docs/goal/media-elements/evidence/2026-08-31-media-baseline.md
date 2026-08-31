# WPT media-elements 首批基线报告（media-elements goal M1 / DC-1）

- **日期**: 2026-08-31（Rally 轮，库内时区）
- **WPT rev pin**: `315976933870b34d6ea30e3f6643403edae678ba`（与 fetch-dom-subset.sh / fetch-canvas-subset.sh 同 pin）
- **入口**: `make testharness-media`（fetch-media-subset.sh + zero-wpt-runner testharness-media，test-guard 包裹）
- **首批范围**: media-elements/ 顶层语义面 23 用例 + HTMLTrackElement 反射目录 7 用例 = 30 用例（判定标准 = 只断言 JS 可观察语义，不含真解码面；event_* 族 / autoplay / seeking/ 留待语义层落地追加）

## 总通过率：114/245 subtest = **46.5%**

| 状态 | 数量 | 说明 |
|---|---|---|
| Pass | 114 | 通过 |
| Fail | 77 | 失败（断言不满足/接口缺失） |
| Timeout | 13 | 用例级超时（completion 未回调——异步媒体事件依赖面） |
| PreconditionFailed | 41 | 实现可选断言前置失败（canPlayType 能力表为空表的直接后果） |

## 分类通过率

| 分类 | Pass | Fail | Timeout | PreconditionFailed |
|---|---|---|---|---|
| load 算法/事件面 | 8 | 0 | 4 | 0 |
| 其他（historical 面） | 55 | 1 | 0 | 0 |
| track 面 | 9 | 66 | 0 | 0 |
| 元数据/反射面 | 7 | 10 | 2 | 0 |
| API 语义（canPlayType） | 19 | 0 | 0 | 41 |
| 状态机 | 16 | 0 | 7 | 0 |

## 逐用例明细

| 用例 | Pass | Fail | Timeout | PreconditionFailed |
|---|---|---|---|---|
| error-codes/error.html | 2 | 0 | 1 | 0 |
| historical.html | 53 | 1 | 0 | 0 |
| interfaces/HTMLElement/HTMLMediaElement/addTextTrack.html | 2 | 9 | 0 | 0 |
| interfaces/HTMLElement/HTMLMediaElement/crossOrigin.html | 6 | 5 | 0 | 0 |
| interfaces/HTMLElement/HTMLMediaElement/textTracks.html | 0 | 1 | 0 | 0 |
| interfaces/HTMLElement/HTMLTrackElement/default.html | 0 | 7 | 0 | 0 |
| interfaces/HTMLElement/HTMLTrackElement/kind.html | 0 | 20 | 0 | 0 |
| interfaces/HTMLElement/HTMLTrackElement/label.html | 0 | 11 | 0 | 0 |
| interfaces/HTMLElement/HTMLTrackElement/readyState.html | 2 | 0 | 0 | 0 |
| interfaces/HTMLElement/HTMLTrackElement/src.html | 5 | 6 | 0 | 0 |
| interfaces/HTMLElement/HTMLTrackElement/srclang.html | 0 | 11 | 0 | 0 |
| interfaces/HTMLElement/HTMLTrackElement/track.html | 0 | 1 | 0 | 0 |
| location-of-the-media-resource/currentSrc.html | 2 | 0 | 1 | 0 |
| mime-types/canPlayType.html | 19 | 0 | 0 | 41 |
| networkState_during_loadstart.html | 0 | 0 | 1 | 0 |
| networkState_during_progress.html | 2 | 0 | 1 | 0 |
| networkState_initial.html | 2 | 0 | 0 | 0 |
| offsets-into-the-media-resource/currentTime.html | 0 | 2 | 1 | 0 |
| offsets-into-the-media-resource/duration.html | 1 | 0 | 0 | 0 |
| paused_false_during_play.html | 2 | 0 | 1 | 0 |
| paused_true_during_pause.html | 2 | 0 | 1 | 0 |
| playing-the-media-resource/playbackRate.html | 0 | 1 | 1 | 0 |
| preload_reflects_none_autoplay.html | 0 | 2 | 0 | 0 |
| readyState_during_canplay.html | 2 | 0 | 1 | 0 |
| readyState_during_canplaythrough.html | 2 | 0 | 1 | 0 |
| readyState_during_loadeddata.html | 2 | 0 | 1 | 0 |
| readyState_during_loadedmetadata.html | 2 | 0 | 1 | 0 |
| readyState_during_playing.html | 2 | 0 | 1 | 0 |
| readyState_initial.html | 2 | 0 | 0 | 0 |
| src_reflects_attribute_not_source_elements.html | 2 | 0 | 0 | 0 |

## 失败聚类（M1 切片 2 输入）

| # | 簇 | 症状 | 根因定性 | 量级 |
|---|---|---|---|---|
| F1 | **`<track>` IDL 反射面整体缺失** | `track.kind/label/srclang/default` 全 undefined；`track.track` 抛 `TextTrack is not defined` | part03 get trap 无 TRACK 分支；TextTrack/TextTrackList 接口未建 | 49 Fail（7 用例） |
| F2 | **HTMLMediaElement 元数据 IDL 缺失** | `currentTime/playbackRate/preload` 读返 undefined；`crossOrigin` 反射值无 anonymous/us e-credentials 归一 | 同上——media 段只有 R2835 四方法，无 IDL 属性面 | 9 Fail（4 用例） |
| F3 | **`addTextTrack`/`textTracks` 缺失** | `video.addTextTrack is not a function`；`textTracks.length` undefined | TextTrack 集合面未建（依赖 F1 的 TextTrack 接口） | 10 Fail（2 用例） |
| F4 | **媒体事件序列未派发**（headless 近似驱动缺口） | `loadstart/canplay/loadedmetadata` 等从不触发 → async_test 全部 case-level Timeout。注：Timeout 用例里的 2 Pass subtest 是外层同步 `test()` 包装（async_test 声明处），内层 async_test 恒 pending——通过率为表面值 | 宿主 FR-009 资源 settle 只派 img/track/source 的 load/error；media 专有事件序列（load 算法）未接 | 11 case Timeout |
| F5 | **canPlayType 空表的连锁** | 41 PreconditionFailed（assert_implements_optional）+ 少量 Fail | 空表本身是 spec 允许的保守实现（R2835 记录在案），但 `'audio/mp4'→''` 使 optional 断言前置失败——非 bug，能力表决策后自愈 | 41 PF |
| F6 | **`track.src` IDL 解析缺失** | 返原始属性串，未按 base 解析为绝对 URL；`\0` 未按 spec 剥离 | track.src 为 URL 属性（同 R2838 a.href 模式），未接 `__zw_parse_url` | 6 Fail |

## 与基线事实的对照（修正 2026-08-17 立项记录）

- ✅ **error/readyState/networkState 初值正确**（3 用例全绿）——FR-009 资源 settle 与常量面已备
- ✅ `src_reflects_attribute_not_source_elements`（R3040 反射底座）全绿
- ✅ historical 面 53/54（`TextTrackCue` 未按历史移除抛 TypeError——实际 ReferenceError，归 F1 联动）
- ⚠️ 立项记录「canPlayType 未核实」现已核实：R2835 恒返空串，语义正确但能力表为空（F5）

## 结论与下一步

1. **M1 完成**：基线 46.5%（114/245）成文；失败聚类 F1~F6 入账。
2. **M1 切片 3（下一轮）**：F2 元数据 IDL 反射面（currentTime=0/duration=NaN/playbackRate=1/preload/crossOrigin 归一）+ F6 track.src URL 解析——根因清楚、headless 值有 spec 定义（无解码器时的合法初值），轻量修复优先级最高。
3. **M2（其后）**：F1 track 反射 + TextTrack 最小面 → F3 集合面 → F4 load 算法骨架 + headless 近似事件序列（clearTimeout 面，需宿主资源 settle 扩展 media 事件）。

---

## M1 切片 3 增量（F2/F6 落地，2026-08-31 同轮）

commit `feat(engine): media metadata IDL face + track reflection`：

- **F2 修复**：AUDIO/VIDEO 元数据 IDL 面——currentTime/duration(NaN)/playbackRate/
  defaultPlaybackRate/volume（[0,1] clamp）/seeking/paused/ended/defaultMuted 初值与
  setter round-trip；preload 枚举反射（缺省 metadata）；crossOrigin 枚举反射
  （missing→null / ''·invalid→anonymous / use-credentials，setter null→removeAttribute）；
  has-trap 白名单（`'crossOrigin' in video` 可见）。
- **F1 部分修复**：HTMLTrackElement 反射——kind（缺省 subtitles / invalid→metadata /
  大小写归一）、label/srclang（DOMString）、default（布尔）、src（URL 属性绝对解析 +
  C0/space 剥离）+ 全部 setter（移除同步 R122 实例层）。
- **F6 附带修复**：`<a href="">.href` 空串按 URL spec 解析为页面绝对 URL（属性存在但空
  ≠ 属性缺失）。
- **新增单测**：`test_media_metadata_idl_face_r388`（5 组断言：初值面 / round-trip 与
  clamp / 枚举反射 / track 反射 / src 绝对解析）。

### 增量后总通过率：179/245 subtest = **73.1%**（基线 46.5% → **+26.6pp**）

| 状态 | 基线 | 切片 3 后 | 变化 |
|---|---|---|---|
| Pass | 114 | 179 | **+65** |
| Fail | 77 | 12 | **-65** |
| Timeout | 13 | 13 | 0（F4 域，M2） |
| PreconditionFailed | 41 | 41 | 0（F5 域，能力表决策后自愈） |

### 余账（12 Fail，全部 M3 TextTrack 家族）

- addTextTrack 9 Fail（`video.addTextTrack is not a function`——需 TextTrack 接口）
- textTracks 1 Fail（集合面）
- track.track 1 Fail（TextTrack 构造器）
- historical 1 Fail（TextTrackCue `new` 应抛 TypeError——现 ReferenceError；TextTrack
  接口落地时顺带闭合）

**验证**：make test 65 套件全绿、`cargo fmt --all -- --check` 干净、
`cargo clippy --workspace --all-targets -- -D warnings` 零警告。

---

## M2 增量（F4 media 事件序列 headless 近似驱动，2026-08-31 同轮）

- **实现**：
  - part06 `_zwSettleResourceSelector` 泛化为 `_zwSettleResourceKey`（handle/sel 双身份）；
    audio/video settle 成功即派 headless 事件序列：loadstart → progress → durationchange +
    loadedmetadata（readyState=HAVE_METADATA，duration 定值 600）→ loadeddata → canplay →
    canplaythrough（HAVE_ENOUGH_DATA）；autoplay 属性续派 play → playing；序列前后
    networkState LOADING→IDLE。
  - 动态 `.src=`（JS 设置）：shim 侧 setTimeout(0) 加载模拟（runner 无媒体 fetch 通路）；
    精确空串 src → 仅派 loadstart（资源选择失败语义，currentSrc 恒 ''）。
  - play()/pause() 语义面：playing 态镜像 + play/playing/pause 事件派发（幂等）。
  - currentTime setter：readyState≥1 时 seeking=true + seeking/timeupdate 派发，seeked
    异步回落；volume/playbackRate setter 派 volumechange/ratechange。
  - `_mediaFireSel`/`_zwMediaFire` 带 on* 属性兜底派发（detached handle 元素键位防护）。
  - src IDL getter（AUDIO/VIDEO/TRACK）：绝对 URL 解析 + C0/space 剥离。
  - FR-009 集成测试契约更新（readyState 0→4 = 新 headless 契约；error 路径断言不变，
    资源失败用例零改动通过 = A/B 佐证）。
- **新增单测**：`test_media_load_event_sequence_r389`（定时器排程/双路径派发/readyState/
  networkState 断言）。

### M2 后总通过率：214/269 subtest = **79.6%**（基线 46.5% → 切片3 73.1% → **M2 79.6%**）

| 状态 | 基线 | 切片3 | M2 | 变化 |
|---|---|---|---|---|
| Pass | 114 | 179 | 214 | **+35** |
| Fail | 77 | 12 | 12 | 0（TextTrack 家族） |
| Timeout | 13 | 13 | **2** | **-11**（F4 基本闭合） |
| PreconditionFailed | 41 | 41 | 41 | 0（能力表决策后自愈） |

### 余账（2 Timeout + 12 Fail）

- Timeout：currentSrc（source 子元素插入触发资源选择 = DOM mutation 面，defer 至 M3 后）、
  error（错误码语义 = 真错误注入面，随解码层）
- Fail：12 全部 TextTrack 家族（addTextTrack 9 + textTracks 1 + track 1 + historical 1）

**验证**：make test 65 套件全绿、fmt 干净、strict clippy 零警告。
