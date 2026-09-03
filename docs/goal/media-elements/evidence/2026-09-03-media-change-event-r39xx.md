# M3 扩批 XXI（2026-09-03）— TextTrackList change 事件广播（深结构项 D 组首个收口）

## 排除原因（既有记录）

track-change-event 需要「TextTrack → 所属 TextTrackList」反向链 + mode 变更时向
list 广播 change 事件——此前 TextTrackList 仅在 textTracks 首读时惰性创建，TextTrack
实例无 list 反向引用。

## 实施

1. **反向链回填**（三处）：`_zwSyncTextTracksFromChildren` 的 holder 同步段（track
   子 + manual 全量遍历回填 `track._zwOwnerList = list`）；textTracks getter 的
   list 首建处（既有 manual 段）；addTextTrack。
2. **addTextTrack 即时建 list**：spec 语义 track 一经创建即属于 media element 的
   track 列表（列表对象存在与否不依赖脚本是否访问过 textTracks）。上游用例时序
   （addTextTrack → mode='showing' → textTracks 首读）中 mode setter 跑在首读前
   ——惰性建 list 会使 setter 的 change 广播失联（首版修复 Timeout 根因）。
3. **change 广播**（part01b）：TextTrack mode setter **有效值变更**（同值/invalid
   不派）→ `_zwFireTracksChanged(list)` 异步派**基础 Event**('change')
   （无 track 属性——上游 hasOwnProperty('track') === false 断言面；target 为 list
   的 exposed proxy，同 addtrack 形态；queueMicrotask queued task）。

## 结果

- track-change-event 导入常驻（1 subtest 全绿；track 族 52 用例 2 连跑稳定）。
- testharness-media：**536P/0F/24PF（536/560 = 95.7%）**。
- 单测 `test_media_texttrack_list_change_broadcast_m3xxi`（派发面 + instanceof/
  无 track 属性/target 身份 + 同值/invalid 不派 + 再变更再派，2 断言组）。
- make test 66 套件 18804/0、clippy -D warnings 零警告、fmt 干净。
