# M3-S2 — WAAPI timing-model 相位/进度 + 关键帧解析校验

**日期**: 2026-09-28
**通道**: `make testharness-web-animations`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-28-m3-s2-web-animations.json](2026-09-28-m3-s2-web-animations.json)

## 结果（M3-S1 → M3-S2）

| 指标 | M3-S1 | M3-S2 |
|---|---|---|
| web-animations | 338P/605F/10T（953）| **483P/452F/11T（946）** |
| 全四 corpus 合计 | 945/1582（59.7%）| **1090/1575 = 69.2%** |

分母 953→946：部分案 fail-fast 提前（校验抛错后不再注册后续 subtest），诚实记账。

## 修齐的语义（part03.js）

1. **相位/进度算法**（§4.9 calculations）——`_zwEffComputedTiming` 从持有 animation 的
   localTime（currentTime − startTime，null 按 0）解析 phase / progress /
   currentIteration：activeTime 的 fill 门（none → before/after 相位 progress null，
   assert_phase 的判相机制）、overall/directed progress（normal/reverse/alternate/
   alternate-reverse + iterationStart）、currentIteration、零时长/零迭代分支、
   phase/localTime 字段。解锁 current-iteration 33P / simple-iteration-progress 35P /
   active-time 5P / phases-and-states 3P / local-time 2P。
2. **effect ↔ animation 双向接线**——`effect._timing._animation` 反向持有
   （M3-S1 曾误挂在 effect 对象上，timing 读取落空 → progress 恒 null，探针勘定）；
   play() 不再强制 startTime=0（spec play-pending unresolved 语义保持，null 在
   计算中按 0 处理两全）。
3. **关键帧解析校验**（§5.4.15）——animatable 白名单（~120 属性；白名单外不读取
   值、不落关键帧——processing-001「Accessor not called」断言非可动画属性 0 次访问）、
   offset ∈ [0,1] 校验、easing 语法校验（linear/ease 族/cubic-bezier x∈[0,1]/
   steps 关键字族/linear(...)）、composite 枚举校验；**全部属性读取之后**统一抛
   TypeError（processing-002「All properties were read before throwing」断言——
   height/width 读写计数 2/4 后才抛 easing 错）。

## 余非 Pass 主簇（M3-S3/M4 方向）

| 簇 | 案数 | 归属 |
|---|---|---|
| 渲染效果断言（getComputedStyle 联动 / opacity 终态）| ~200 | rendering-compat 域记账（goal 边界：JS API 面 vs 渲染计算）|
| progress 边界（startTime=0 恰好压线 / transformed-progress 方向化）| ~40 | M3-S3 精修 |
| play/pause 状态机时序（running vs idle/finished）| ~24 | headless 瞬间完成近似，如实标注（DC-3）|
| 关键帧序列化 canonicalization（CSS 值归一）| ~30 | M3-S3 |

## 跨 corpus 零回归

hr-time 10/3/2、user-timing 561/8/1、performance-timeline 36/4/4 全部持平；
make test exit 0（engine 单测全绿）、fmt/clippy 干净。
