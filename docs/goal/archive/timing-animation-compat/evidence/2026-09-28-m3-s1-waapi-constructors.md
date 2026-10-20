# M3-S1 — WAAPI 构造器 + effect 桥

**日期**: 2026-09-28
**通道**: `make testharness-web-animations`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-28-m3-s1-web-animations.json](2026-09-28-m3-s1-web-animations.json)

## 结果（M1 基线 → M3-S1）

| 指标 | M1 基线 | M3-S1 |
|---|---|---|
| web-animations | 52P/901F/10T（963）| **338P/605F/10T（953）** |
| 全四 corpus 合计 | 319/1376（23.2%）| **945/1582 = 59.7%** |

注：web-animations 分母 963→953——部分案在关键帧参数处 fail-fast 提前退出，
注册 subtest 面缩小（诚实记账）。

## 实现（part03.js `_makeAnimation` 区升级 + part06 document.timeline）

- `KeyframeEffect(target, keyframes, options)`：可构造接口——target/composite/
  iterationComposite/pseudoElement 访问器、`getTiming()`（原始值：fill 'auto'、
  duration 'auto'）、`getComputedTiming()`（归一化 + endTime/activeDuration/
  progress/currentIteration）、`getKeyframes()`/`setKeyframes()`（数组形态 +
  property-indexed 形态统一展开、computedOffset 均分补齐、composite 缺省 'auto'）、
  `updateTiming()` 局部更新
- `Animation(effect, timeline)`：可构造接口——effect/timeline 访问器（timeline 缺省
  document.timeline 单例）、playState/pending/ready/finished、play/pause/cancel/
  finish/reverse/updatePlaybackRate/commitStyles/persist、onfinish/oncancel；
  R2965 瞬间完成状态机迁移（play → running → defer finished + fill 门末态持久化 +
  finished promise/onfinish；finish 负 rate → InvalidStateError）
- `DocumentTimeline(options)` + `document.timeline` 默认实例（currentTime =
  now() − originTime）——hr-time raf-coarsened-time 的 `DocumentTimeline is not
  defined` 同闭
- `el.animate()` 返真 Animation（part04 签名不变）+ getAnimations 注册表兼容；
  `duration` 非 spec 属性保留（R2965 旧形态与动画库消费兼容）

## headless 帧精度约束（DC-3 如实标注）

播放时序面（ready promise 时序、pause() 后 progress、逐帧 currentTime 推进）保持
「瞬间完成」近似：无真 vsync 时间轴，按真实缺口 Fail/Timeout 计入分母，不放容差。

## 余非 Pass 主簇（M3-S2 方向）

| 簇 | 案数 | 归属 |
|---|---|---|
| `progress/currentIteration` 相位计算（before-phase 期望 0 而非 null）| ~110 | M3-S2：timing-model 相位/进度算法（localTime=0 基线）|
| `Accessor not called`（构造期不得触样式访问器）| ~48 | M3-S2：构造路径样式副作用排查 |
| 关键帧参数校验不抛（processing-a-keyframes-argument）| ~29 | M3-S2：§5.4.15 校验（非法属性值/列表形态 TypeError）|
| 渲染效果断言（getComputedStyle 联动）| ~200 | rendering-compat 域记账（goal 边界）|

## 跨 corpus 零回归

hr-time 10/3/2、user-timing 561/8/1、performance-timeline 36/4/4 全部持平；
engine R2821/R2965 单测全绿（make test exit 0）。
