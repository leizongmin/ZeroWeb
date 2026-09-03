# M3 扩批 XXII（2026-09-03）— B 组排除件复评：disabled gate + cuechange 面三案收口

## 复评范围

B 组排除件（「依赖真播放钟推进」族）在 change 广播（XXI）与播放推进基建（XVI~XX）
之后的逐件复评。

## 实施

1. **march disabled gate**（part06）：disabled track 跳过 cue 调度并静默清空
   active 集合（spec time-marches-on 步 2「disabled mode → abort these steps」）。
2. **march 遍历面统一**（part06）：track 子产物（_elementTextTrack）与 addTextTrack
   产物（_textTracksCache[mediaKey].tracks）统一遍历 + 身份去重——此前 addTextTrack
   产物的 cue 永不 enter/exit（track-remove-active-cue 的 addTextTrack + addCue +
   play 形态 Timeout 根因）。
3. **cuechange 派发**（part06）：本 tick 有 enter/exit 派发的 track 异步派单次
   cuechange（spec time-marches-on 步 8），同步转发到 track 元素
   （HTMLTrackElement.oncuechange 监听面——track._zwOwnerEl，part01b 暴露）。
4. **play() 桥 src 读身份分派**（part03）：handle 身份（createElement 产物）走
   registry 现值 `__zw_get_attr_handle`——`__zw_get_attr_lw` 是 sel 文档查询，
   对 null sel 恒空 → bridgeSrc 空 → 桥失联 + 重试不启动。

## 导入（+4 净涨）

- track-disabled（disabled gate 面）
- no-cuechange-before-play（播放前不派 cuechange——march 仅 playing 态跑天然满足；
  EventWatcher + promise_test 框架面验证）
- track-remove-active-cue（active cue 移除无 crash）
- ~~track-active-cues~~ **维持排除**（EventWatcher 双 promise wait + createElement
  track 动态 src + VTT settle 竞速的三方耦合，eventCount==3 窗口不稳定——归因
  详见下方「待复评」）

## 结果

- testharness-media：**539P/0F/24PF（539/563 = 95.7%）**；track 族 55 用例 3 连跑
  稳定。engine 2575 全绿。
- **待复评**：track-active-cues（onload + oncuechange + oncanplaythrough 三事件
  计数窗口）——宿主实证各事件均已可达（settle load 派发 / march enter / cuechange
  ownerEl 转发），但 EventWatcher 的 promise settle 与三方竞速的窗口未收敛，随泵
  节拍精化复评。
