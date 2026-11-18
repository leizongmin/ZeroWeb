---
date: 2026-11-15
modules: engine/animation, wpt-runner
---

# AnimationClock.tick 会烧死 fill-forwards 状态——端点捕获必须走无副作用路径

## 问题描述

WAAPI「动画值进 gCS」刀（js_anim_registry）首版在动画注册时用一次 `tick(id, 0)`
探针捕获端点原文，供后续求值做 fallback 检测。探针后整个动画在任意 seek 点的
求值全部恒返回终帧：探针页 at(0)/at(0.5) 读值与 at(1) 相同（opacity 0.25→0.75
在 at=0 读出 0.75），corpus 出现约 28 条 neutral/at(0) 面 REGRESSED。

## 根因分析

`AnimationClock::tick` 不是纯求值函数——`t ≥ duration` 时它把
`AnimationState::finished` 置 true，而 fill forwards 分支的后续路径是：

```rust
if anim.finished {
    // fill forwards 恒走 interpolate_to_end
    continue;
}
```

`finished` 一经置位没有任何复位路径（seek/set_hold 只改 paused_at）。因此注册时
对同一 clock_id 的任何一次 tick（哪怕传 t=0，若 duration=0 或进度被钳制判定为
结束）都可能把状态推进到 finished，此后所有求值被永久钉在终帧。

这是可复现的 API 陷阱：任何「先探测一次再正式求值」的调用序都会污染正式求值；
对同一动画的多次 tick 也不幂等。

## 解决方案

1. 端点/边界信息捕获一律走无副作用路径：直接读已解析的 `@keyframes` AST
   （`KeyframesRule.keyframes[].declarations[]` 的声明值原文）。`block_to_point`
   对 `d.value` 是保真 clone，AST 声明串与 tick 端点输出同源同格式，可安全
   作为 fallback 检测基准。
2. 需要重复求值的场景（注册表查询每次都 tick）：接受现状——fill forwards 下
   已开始动画的重复 tick 只推进时间不重复置位 finished；**禁止**对可能到达
   `t ≥ duration` 的时间点做「试探性」tick。
3. 若后续需要真幂等求值（如声明式 CSS Animations 进 gCS），需给 tick 增加
   `peek` 形态（不改状态只算插值）——届时在 js_anim_registry 挂 FIXME 对接。

## 验证

AST 直读端点改造后探针页 at(0)/at(0.5)/at(1)/cancel/cancel-after 全绿；
五组 corpus A/B REGRESSED 245 → 9 → 0。
