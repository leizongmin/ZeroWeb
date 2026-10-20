# M3-S3 — WAAPI 收口（hold time / 边界 / 校验 / commitStyles）

**日期**: 2026-09-28
**通道**: `make testharness-web-animations`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-28-m3-s3-web-animations.json](2026-09-28-m3-s3-web-animations.json)

## 结果（M3-S2 → M3-S3）

| 指标 | M3-S2 | M3-S3 |
|---|---|---|
| web-animations | 483P/452F/11T（946）| **502P/437F/12T（951）** |
| 全四 corpus 合计 | 1090/1575（69.2%）| **1108/1580 = 70.1%** |

## 修齐的语义（part03.js）

1. **play() hold time**——同步置 `currentTime = 0`（fresh/finished 重播；spec：play 设
   hold time、startTime 保持 unresolved）——updateTiming「animation in progress」族
   （局部 delay 推进：`{delay:-50}` 后 progress 0.5 等 ×16）+ the-current-time +
   update-and-send-events 解锁。
2. **currentIteration 边界**——activeTime == activeDuration（after 相位 fill 门）→
   末迭代索引 `iterations − 1`（§4.9.4 overallProgress 恰为 iterations 时取边界值）。
3. **`KeyframeEffect.target` setter**（spec 可写；target.html「only a getter」簇）。
4. **updateTiming 数值校验**——delay/endDelay 须有限、iterationStart/iterations 须
   有限且 ≥ 0、duration 非负（NaN/±Infinity/负值 → TypeError，gBadDelayValues 等
   簇）。
5. **commitStyles InvalidStateError**——finished 且 fill 非 forwards/both、或无关联
   target → 抛（spec 前置条件；剩余 pseudo-element / display:none / not-rendered
   条件需渲染侧知识，按真实缺口记账）。

## 余非 Pass（M3 收口余量 + 域边界）

| 簇 | 案数 | 归属 |
|---|---|---|
| playState 派生（startTime/currentTime/rate 组合语义）| ~24 | **派生重构会与瞬间完成状态机冲突**（300+ subtests 与 R2965 单测依赖）——headless 近似如实标注（DC-3）|
| 渲染效果断言（getComputedStyle/opacity 终态）| ~200 | rendering-compat 域记账 |
| progress 压线边界 / 序列化 canonicalization | ~50 | M3 残余精修（低密度）|
| commitStyles 渲染条件（pseudo/display:none）×5 | rendering-compat 域记账 |

## 跨 corpus 零回归

hr-time 10/3/2、user-timing 561/8/1、performance-timeline 36/4/4 全部持平；
make test exit 0（engine 单测全绿）、fmt/clippy 干净。

## M3 收口判定

M3（WAAPI）主体完成：52→502P（基线的 9.7 倍）；构造器/时序模型/解析校验/hold time
语义面闭合。剩余为渲染域记账 + headless 近似约束 + 低密度边界精修，进一步收敛
需渲染管线联动（rendering-compat 域）或异步状态机重构（深结构，收益密度低）。
下一步 M4：DC 逐项判定 + 挂账定稿。
