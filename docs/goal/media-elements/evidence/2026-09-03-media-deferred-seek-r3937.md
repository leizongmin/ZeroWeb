# M3 扩批 XX（2026-09-03）— HAVE_NOTHING 期 seek 挂起语义 + track-cues-seeking 导入

## 根因

`track-cues-seeking`（onseeked 计数链 + activeCues 递增断言）此前 Timeout：
track.onload 内 `video.src=` → `currentTime=0.5` 立即执行，此时动态 src 的 settle
（`_zwMediaLoadSequence`）尚未跑——`readyState === 0`，currentTime setter 的 seek
门（`readyState >= 1`）关闭 → seeking/seeked 永不派发 → onseeked 永不触发 →
`t.done()` 永不到达。

spec 语义（https://html.spec.whatwg.org/multipage/media.html#dom-media-seek）：
HAVE_NOTHING 期 seek 不立即跑 seek 算法，但「set the default playback start
position to that time」——元数据就绪后从该位置起播；Chromium 可观察面：
seeking/seeked 在 loadedmetadata 后照常派发。旧实现直接静默丢弃（既无挂起也无
default start position 面）。

## 修复

- **part05 currentTime setter**：`readyState === 0` 时挂 `_zwSeekDeferred = true`
  （值已由 setter 写入 `_mediaState.currentTime`——起播位置即该值）。
- **part06 `_zwMediaLoadSequence`**：`readyState = 1` 翻转处消费 `_zwSeekDeferred`
  ——补跑 seek 算法（seeking 事件 + seeked 异步回落 + `_zwMediaSeekSync` cue
  active 面同步），幂等（单次消费）。

## 结果

- track-cues-seeking 导入常驻（+1 subtest 全绿；track-cues 6 用例 3 连跑稳定）。
- testharness-media：**535P/0F/24PF（535/559 = 95.7%）**。
- 关联面回归确认：event_* 116P、resource-selection 23P、networkState 10P、
  currentTime 3P 全绿（seek 门与加载序列改动的影响面）；engine 2572 全绿。
