# M3 扩批 LIII — addtrack 时序三处 spec 对齐 + track-mode-not-changed-by-new-track 试导负结果

**日期**: 2026-09-05
**WPT rev**: 315976933870b34d6ea30e3f6643403edae678ba
**结果**: 209 用例 653 subtest，**629P / 0F / 24PF**（与 LII 基线逐位一致，零回归）；
试导件 track-mode-not-changed-by-new-track 回退维持排除（负结果归档）。

## 背景

LII 轮 track settle 同步化三方案全部负结果后，本轮转 runner 事件循环域归因的自主
可推进面：复核全部排除注记的有效性。发现 `track-mode-not-changed-by-new-track`
的排除注记（扩批 XXXII）与用例实际断言面不符：

- **注记所述**: 「textTracks 增量同步的 track 身份对拍（getElementById(x.track.track)
  same-object 面，身份表切片）」——实际用例无此断言。
- **实际断言面**: mode 稳态（新 track append 后既有 track 的 readyState/mode/cues
  不变）+ addtrack 身份链（`event.target === video.textTracks`、
  `event.track === textTracks[length-1]` 即 track3）。
- 所需资源 metadata.vtt / webvtt-file.vtt 均已在 fetch 清单；appendChild 钩子 →
  `_zwSyncTextTracksFromChildren` + `_zwScheduleChildTrackLoads` + addtrack 派发 +
  onaddtrack accessor 链（扩批 XII/XIII/XXXI）均已就位 → 具备试导条件。

## 试导过程与根因定位（探针实证）

首轮试导失败：`assert_equals` 对象身份不匹配。逐层探针（document.title 日志经
throw 注入 failure message）定位：

1. **阶段链全通**: canplaythrough（C）→ metadata onload（M）→ captions onload（K）
   → addtrack handler（T0）全到达——非触发链断裂。
2. **T0 收到错误实例**: `event.track === track3` 为 false，`event.track.mode` 为
   'hidden'（= track1 在 canplaythrough 内被设的值）——handler 收到的是 **track1 的
   addtrack**。根因：list 惰性建于 `textTracks` 首读（K-turn 的 line 54），首读时
   sync 对 holder 全量补派 [t1,t2,t3]，迟注册的 handler 先收到 t1 的事件。真实浏览器
   中 t1/t2 的 addtrack 在插入时刻（parse 期 / appendChild 任务）已派发完毕，迟到
   handler 只收 track3。
3. **三项 spec 对齐修复落地**（保持 629 绿）:
   - **addtrack 每实例一次幂等**（`_zwAddtrackFired` 门，sync added-dispatch +
     addTextTrack 路径）——重复读 textTracks 不再重派。
   - **observed 观察登记**（`_zwMarkTrackObserved`/`_zwIsTrackObserved`，part04
     appendChild 钩子标记）——parse 期入列、钩子未见过的 track 子不再由迟到首读
     补派（spec「addtrack at insertion」：其 addtrack 在解析任务期已派发完毕）。
   - **append 时刻建 list**（`_zwEnsureTextTrackList`，part04 appendChild 钩子在
     sync 前调用）——list 不再惰性于首读，append 期插入的 track 在 append turn 内
     排队 addtrack。
4. **进一步探针发现派发时点不稳定**: t2 的 addtrack timer 在 M-turn 注册（先于
   t2.onload 的任务），但 dispatch 却发生在 K-turn（listener 已注册后）——
   `dq:`（queue）/`disp:`（dispatch）日志分离实证 **queued task 的跨 execute 派发
   时点不受排队序保证**。microtask 承载时同样失序（上一 execute 的微任务滞留到
   之后某 execute 的 checkpoint 才派发）；改 setTimeout 承载（host 定时器逐 tick
   泵）后仍受泵 tick 合并影响。t2 的迟到派发抢占 handler 首事件 → track3 断言
   失败，与 LII「调度成功而 body 永缺 / 派发时序失真」同域。

## 结论

- **三项 spec 对齐改动保留**（629P 保绿，见下「改动清单」）——它们修正了真实的
  语义缺陷（迟读重放历史事件、list 惰性时序），对 40+ 既有 track 用例零回归。
- **track-mode-not-changed-by-new-track 回退维持排除**：其首断言依赖 addtrack
  queued task 的跨 execute 派发时点保证，属 **runner 事件循环统一（deep-structure）**
  域，与 track-remove-insert-ready-state 同域归档。runner 域内无进一步可自主
  修复面。

## 改动清单

- `crates/engine/src/js_dom_shim/part01.js`:
  - `_zwSyncTextTracksFromChildren` added-dispatch：`_zwAddtrackFired` 每实例一次
    幂等门 + `_zwIsTrackObserved` 观察登记判定（parse 期 track 子不补派）。
  - 新增 `_zwMarkTrackObserved` / `_zwIsTrackObserved` / `_zwEnsureTextTrackList`。
- `crates/engine/src/js_dom_shim/part01b.js`: `_zwFireTracksAdded` /
  `_zwFireTracksRemoved` 的 `_deferFire` 从 queueMicrotask 改为 setTimeout(0)
  承载（跨 execute 派发顺序稳定；queued task 语义不变）。
- `crates/engine/src/js_dom_shim/part03.js`: addTextTrack 路径补 `_zwAddtrackFired`
  标记（与 sync 幂等门同面）。
- `crates/engine/src/js_dom_shim/part04.js`: appendChild 钩子——track 子
  `_zwMarkTrackObserved` 登记 + `_zwEnsureTextTrackList` 建列（先于 sync）。
- `tests/wpt-runner/src/testharness.rs` / `scripts/fetch-media-subset.sh`:
  track-mode-not-changed-by-new-track 排除注记更新（旧「身份对拍」注记勘误 +
  LIII 负结果归因）。

## 验证

- `make testharness-media`: 209 cases / 653 subtest / **629P / 0F / 24PF**（与
  LII 基线逐位一致；24PF 为 canPlayType optional 中性面）。
- 机读版: `evidence/2026-09-05-media-addtrack-timing-liii.json`。
- `cargo fmt --all -- --check` 干净；`make guarded-clippy` 零警告；
  `make test` 全绿（后台完成）。
