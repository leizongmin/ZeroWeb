# M3 扩批 XXIII（2026-09-03）— media load invoke 重置面收口（track-active-cues 解除排除）

## 排除原因（既有记录）

track-active-cues 列于 B 组排除件（EventWatcher 三方竞速窗口注记）；扩批 XXII 落地
disabled gate + cuechange 派发后复评，首轮仍 Timeout。

## 根因定位（dormant 插桩实证）

经 `__zwPauseWatch`（`ZW_MEDIA_SEEK_DEBUG` 环境变量门控 console→tracing 通道）在
settle / load invoke / march 三处埋点 + 宿主泵侧 skip-playing 快照，时序还原：

1. `eventCount==3`（canplaythrough）→ 用例置 `video.src=''`（成功加载后的**二次
   调度**）→ 追踪见 `LOAD-INVOKE-STOP`（播放中止段跑通），但 **settle-dispatch
   video error 缺席** → onerror 永不 → done 永不 → Timeout。

两个独立根因（A 与 B 相互掩蔽——首轮先修 A 后 Timeout 转 Fail 才暴露 B）：

- **根因 A — settle 幂等门误吞**：`video.src=''` 走 IDL setter →
  `_zwMediaScheduleLoad(isEmptySrc=true)` → 续段 `_zwSettleResourceKey(...,'error',4)`。
  但 `_resourceStates[key]` 仍持有首次成功加载的 settled 条目——settle 首行
  「每资源只 settle 一次」幂等门**直接 return false**，error 永不提交。
  `delete _resourceStates[key]` 只在 IDL `load()`（part03）与 src 移除（null setter）
  分支存在；`src=''` 二次赋值路径无重置。
- **根因 B — handle-only 元素 on\* handler 派发断链**：`createElement('video')` 产物
  的 `video.onerror = fn` 经 part04 set trap 的 R2933 分支**同时**写入 `_onHandlers`
  与 `_listenerStore[key]`——listener 路径可触发（trackElement.onload 亦然）。但
  `video` 是 handle-only（`_elKey` 为 `@__n0` 形态），settle 的
  `_dispatchWithBubble(key, sel, null, ev)` 传 **handle=null**，listener 键失配恒
  0 命中；`_zwMediaFire` 的 on\* expando 兜底分支才是 handle-only 唯一可达通路，
  settle 未走它。

## 实施（shim part06）

1. **invoke 重置 settle 面**：`_zwMediaScheduleLoad` 入口处
   `delete _resourceStates[key]`（spec 资源选择算法 invoke 步「await a stable
   state」前资源状态归零）——IDL load() 已先行清，统一到调度入口后 src= setter /
   setAttribute 路径同语义。
2. **invoke 步 6 位置重置**（spec dom-media-load「set the current playback
   position to 0 ... set readyState to HAVE_NOTHING」）：`readyState >= 1` 时
   `currentTime = 0` + `_zwMediaTimeKnown = false`（activeCues headless gate 复位）。
3. **invoke 重置 track 子产物 cue**：关联文本轨道（`_textTracksCache[key].tracks`
   中 `_zwOwnerEl` 在位的 track 子产物）`_zwClearCues()`——cue@0-5 在位置 0 仍合法
   active，须清 cue 才满足「unloaded 后无 active cue」；**addTextTrack 产物排除**
   （无 URL 面，cue 不随 media load 重置——TextTrack/activeCues「video playing」
   断言面零回归约束）。
4. **settle 的 media/track 元素 load/error 派发改 `_zwMediaFire`**（img/source 外
   其余 tag 保持原路径——source 的 error 派在 source 元素上由 sourceChild 分支
   自理）：listener 未命中时 on\* expando handler 兜底，handle-only 元素断链修复。

WIP 清理：上一 session 限额中断前试推的 track settle 续段 microtask→macrotask
改动经归因**与本三案无关**（根因 A/B 均在 media 面），已回退原 queueMicrotask 模型；
`__zwPauseWatch` 插桩与 `ZW_MEDIA_SEEK_DEBUG` runner 门控验证完毕后移除。

## 结果

- track-active-cues 导入常驻（1 subtest 全绿；FILTER 单跑 3 连跑稳定）。
- testharness-media：**540P/0F/24PF（540/564 = 95.7%）**（+1 净涨零回归）。
- 单测 `test_media_load_invoke_reset_face_m3xxiii`（invoke 重置复 settle error
  面 + 位置归零/activeCues 清空面 + audio onerror expando 兜底面，3 断言组）。
- make test 66 套件 18806/0、clippy -D warnings 零警告（v8 + quickjs 双态）、fmt 干净。
