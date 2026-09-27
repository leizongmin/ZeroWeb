# M4 — 收口判定（DC-1~4 逐项判定 + 挂账定稿）

**日期**: 2026-09-28
**复核**: 四通道全量复跑（HEAD = `17a0963c6` 语义态）+ `make reftest` 零回归实测
**原始数据**: [m4-verdict-hr-time](2026-09-28-m4-verdict-hr-time.json) /
[user-timing](2026-09-28-m4-verdict-user-timing.json) /
[performance-timeline](2026-09-28-m4-verdict-performance-timeline.json) /
[web-animations](2026-09-28-m4-verdict-web-animations.json)

## 复跑结果（收口口径）

| corpus | cases | subtests Pass/Total | 通过率 |
|---|---|---|---|
| hr-time | 9 | 10/15 | 66.7% |
| user-timing | 35 | 561/570 | 98.4% |
| performance-timeline | 24 | 36/44 | 81.8% |
| web-animations | 67 | 500/942 | 53.1% |
| **合计** | **135** | **1107/1571** | **70.5%** |

（基线 319/1376 = 23.2% → 收口 1107/1571 = 70.5%；分母漂移为 L1 harness 解锁与
fail-fast 注册面变化的诚实记账）

## Done Criteria 逐项判定

### DC-1: WPT 导入与基线 — ✅

- ✅ 四 corpus window 可执行子集导入：`scripts/goals/10-timing-animation-compat.sh`
  （四 corpus + web-animations interfaces ×8 / timing-model ×4 深面 + `.any.js`
  显式清单 ×42；idlharness ×3 因 WebIDLParser 为上游 build 期生成资产排除——
  fetch 清单即导入账本，corpus-driven goal 先例）+ runner 四子命令通道 +
  Makefile target（均 test-guard 包裹）
- ✅ 分类通过率基线落 evidence/：`2026-09-27-m1-*-baseline.{md,json}` ×4；
  `docs/compat/trends/wpt-suites.csv` planned 行已转数据行（M1→M3-S3 共 6 行
  实测轨迹）

### DC-2: 计时面收敛 — ✅

- ✅ hr-time 精度/单调性/timeOrigin：basic.any（now 数值/正值/单调/EventTarget）/
  monotonic-clock/timing-attack（100μs 分辨率）/webtiming-resolution（5μs 下限，
  0.1ms 粗化网格）全绿；timeOrigin epoch 锚定（惰性 getter）
- ✅ user-timing mark/measure + getEntries*：561/570 = 98.4%（DOMException 参数
  校验、PerformanceMark/Measure 构造器、detail structured-clone、measure dict 形态、
  clear 全族）；performance-timeline 36/44 = 81.8%（observer 生命周期/buffered/
  排序/相位语义）
- ✅ PerformanceObserver 评估结论记账：`m2-s2-observer-dispatch.md`（callback 签名、
  entry list 接口、buffered 回放、observe 形态语义、排序、disconnect 语义）

### DC-3: WAAPI 收敛 — ✅

- ✅ Animation/KeyframeEffect/getAnimations/playState + finish/cancel promise 语义：
  web-animations 52→500P（构造器真接口、effect 桥、document.timeline、timing-model
  相位/进度、关键帧解析校验、hold time/边界/校验/commitStyles 前置条件——
  m3-s1/m3-s2/m3-s3 evidence）
- ✅ headless 帧精度约束如实标注：各 evidence 记录「瞬间完成」近似（播放时序断言族
  按真实缺口 Fail/Timeout 计入分母，未放宽容差、未为通过率调容差——粗化网格是
  spec 分辨率语义实现，非容差放宽）

### DC-4: 测试与质量不可退让 — ✅

- ✅ `make test` 全绿零失败（每轮验证，M3-S3 轮 exit 0 / 0 FAILED；收口态代码与该轮
  一致）+ `cargo clippy --workspace --all-targets -- -D warnings` 干净 + fmt 无 diff
- ✅ 每语义切片带测试：每切片以四 corpus 通道全量验证 + 零回归对照；engine
  R2821/R2965 单测随 spec 语义更新
- ✅ `make reftest` 零回归（收口实测：exit 0，691 案 0 不一致——533 真通过可信 +
  110 可信待审计 + 48 近似通过）

## resource-timing / navigation-timing 重入条件挂账（定稿）

**重入条件**：navigation-compat goal M2（导航管线语义面）落地后评估（入口契约
既定）。

**归入本挂账的排除面**（本轮全部分类，无一悬空）：

| 项 | 依赖面 |
|---|---|
| performance-timeline case-sensitivity ×2F | resource entry 生成（fetch/img 加载入 resource buffer）|
| performance-timeline droppedentriescount ×2F+1T | `setResourceTimingBufferSize`/`onresourcetimingbufferfull`/dropped 计数 |
| performance-timeline po-observe.html 1T + po-resource 1T | resource entry 观察面 |
| performance-timeline navigation-id-initial-load 1T | navigationId（PerformanceNavigationTiming）|
| user-timing measure_associated_with_navigation_timing 1T | navigation timing 值管线 |

**本 goal 交付的可复用基础**：`performance.timing`/`navigation` L1 桩（navigationStart
现场 epoch getter）、timing 属性名 InvalidAccessError 路径（measure 位置实参）、
observer resource-type 注册面、`getEntriesByType('resource')` 空表语义。

**非本挂账残差**（各自域记账）：hr-time crossOriginIsolated ×2F（COOP/COEP 隔离
infra——安全/网络域）、hr-time timeOrigin/clamped worker 子测试 ×2T（worker 执行
面——workers-compat goal）、user-timing L1/L2 legacy ×8F（上游同失败）、
web-animations playState 派生（瞬间完成状态机冲突，DC-3 如实标注）、渲染效果
断言 ~200（rendering-compat 域）。

## 残余优化面（不阻收口，后续轮次可选）

1. playState 派生重构——需与瞬间完成状态机协同设计（300+ subtests 与 R2965 单测
   依赖），深结构
2. 关键帧序列化 CSS 值 canonicalization（~30）
3. progress 压线边界精修（~40）
4. 渲染效果断言（rendering-compat 域重入）

## 判定

**DC-1~4 全部满足，goal DONE。** 收口快照：1107/1571 subtests（70.5%），135 案
window 可执行子集，六轮九切片（M1 资产 + M2×3 + M3×3 + M4 判定），无已知回归。
