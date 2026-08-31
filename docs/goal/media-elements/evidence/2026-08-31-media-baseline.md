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
| F4 | **媒体事件序列未派发**（headless 近似驱动缺口） | `loadstart/canplay/loadedmetadata` 等从不触发 → async_test 全部 case-level Timeout（subtest 本身断言正确） | 宿主 FR-009 资源 settle 只派 img/track/source 的 load/error；media 专有事件序列（load 算法）未接 | 11 case Timeout |
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
