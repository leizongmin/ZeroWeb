# 计时与动画兼容 — 运行时控制面板（master.md）

**入口文档**: [../timing-animation-compat.md](../timing-animation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-28（M2-S2 落地：PerformanceObserver 语义修齐，合计 616/1552）

---

## 当前状态

**专项定位**：四 goal 同批立项中的**轻量快赢切片**——hr-time/performance-timeline/
user-timing/WAAPI 纯 JS API 面，不触布局/渲染计算，预期最快出数字。
headless 帧驱动 opt-in（`__ZW_RAF_FRAME_DRIVEN`）是已知约束，如实标注不放容差。

**M2-S2 已落地（2026-09-28）**：PerformanceObserver 语义修齐——
**performance-timeline 20→35P、user-timing observer Timeout 清零（522/528）**，
全四 corpus 合计 **616/1552 = 39.7%**（hr-time 7/15 + performance-timeline 35/44 +
user-timing 522/530 + web-animations 52/963 零回归）。实现（part01b.js）：callback
签名 `(entries, observer, options)` + `this=observer`（11 案 Timeout 簇根因——探针
定位）、PerformanceObserverEntryList 接口、buffered 回放（type 精确匹配）、observe
TypeError/InvalidModificationError、type 叠加/entryTypes 替换、派发批 startTime
stable 排序、disconnect 取消 pending flush。余非 Pass 全为 resource/navigation-timing
排除域 ×8 + webtiming-resolution 1F（M2-S3 粗化簇），见
[evidence/2026-09-28-m2-s2-observer-dispatch.md](evidence/2026-09-28-m2-s2-observer-dispatch.md)。

**M2-S1 已落地（2026-09-28）**：user-timing 异常语义簇修齐——
**user-timing 241→518P（68.1%→98.1%）**。实现：part01b.js mark/measure 语义重写
（PerformanceMark/PerformanceMeasure 接口、保留名 SyntaxError、measure union/dict
形态、InvalidAccessError timing 属性簇、detail structured clone）+ part02
structuredClone wrong-global 修复（R9/R382 先例）+ R2821 单测随 spec 更新。见
[evidence/2026-09-28-m2-s1-user-timing-exceptions.md](evidence/2026-09-28-m2-s1-user-timing-exceptions.md)。

**M1 已落地（2026-09-27）**：四 corpus window 可执行子集导入 + runner 通道 + 基线。
合计 **319/1376 = 23.2% subtests Pass**（135 案执行）：
hr-time 7/15 + performance-timeline 19/44 + user-timing 241/354 + web-animations 52/963。
基线 evidence：`evidence/2026-09-27-m1-{hr-time,performance-timeline,user-timing,web-animations}-baseline.{md,json}`；
suites CSV planned 行已转数据行（active）。

**通道基建（M1 交付物）**：
- fetch：`tests/wpt-runner/scripts/goals/10-timing-animation-compat.sh`（DIRS 深面追加
  web-animations/interfaces ×8 + timing-model ×4；`.any.js` 显式清单补拉 ×42——lib.sh
  未动，排除 idlharness ×3〔WebIDLParser 为上游 build 期生成资产，raw 404〕）
- runner：`testharness-{hr-time,performance-timeline,user-timing,web-animations}` 四
  子命令（testharness.rs `run_timing_subdirs` 共用扫描器：`.html` 直跑 + `.any.js`
  window 变体〔wasm/fs/indexeddb 先例〕+ 绝对路径 helper inline_extras）
- Makefile：`fetch-wpt-timing-animation` + 四 testharness target（test-guard 包裹）

**与兄弟 goal 的边界**：
- rendering-compat — CSS animation/transition 渲染效果面归其；不碰
  style-system/layout-engine/render-foundation 动画计算（M1 已按此记账：
  web-animations responsive/ + reftest-wait 面不扫描/跳过）
- event-loop-spec（已归档）— rAF/微任务先例消费；IO/RO 已立账处不重复
- keyboard-page-scrolling（已归档）— 帧驱动门控为消费事实，改动须跨流核对
- 同批 net-api/navigation/workers — 无共享面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 四 corpus（hr-time/performance-timeline/user-timing/web-animations）导入 + 基线 | ✅ M1（2026-09-27）|
| P2 | hr-time 精度/单调性/timeOrigin 语义 | ⏳ M2-S3（缺口：toJSON/EventTarget 继承/timeOrigin epoch/DocumentTimeline/隔离上下文 ×2〔跨域记账〕）|
| P3 | user-timing mark/measure/getEntries* + PerformanceObserver 评估 | ✅ M2-S1（2026-09-28 异常语义 522/530 = 98.5%）+ ✅ M2-S2（2026-09-28 observer 语义；performance-timeline 35/44，余全为排除域）|
| P4 | WAAPI Animation/KeyframeEffect/getAnimations/playState + promise 语义 | ⏳ M3（KeyframeEffect/Animation/DocumentTimeline 构造器三主簇 ~690 subtests）|
| P5 | resource-timing/navigation-timing 重入条件挂账 | ⏳ M4（M2-S2 后 performance-timeline 余非 Pass ×8 + user-timing ×2 已全部归入此挂账）|

## 已完成切片

- **M2-S2（2026-09-28）**：PerformanceObserver 语义修齐——performance-timeline
  20→35P、user-timing observer Timeout 清零；合计 616/1552（39.7%）。零回归。
- **M2-S1（2026-09-28）**：user-timing 异常语义簇修齐 241→518P（98.1%）。
- **M1（2026-09-27）**：四 corpus 导入 + runner 通道 + 基线 319/1376（23.2%）+
  suites CSV 回填。

## 下一步计划

1. **M2-S3（下一轮起点）**：hr-time——`performance.toJSON` + Performance EventTarget
   继承（basic.any addEventListener）+ **now() 粗化网格**（100μs 非隔离网格使
   webtiming-resolution 5μs 下限与 timing-attack 100μs 簇同闭——改 `_perfNow` 出口，
   注意 rAF/timeline 时序面联动）+ timeOrigin epoch 锚定 + clamped-time-origin
   Timeout 拆解。基线 7/15。
2. **M3**：WAAPI——Animation/KeyframeEffect/DocumentTimeline 构造器 + effect 属性桥
   （三主簇 ~690 subtests）→ 关键帧参数解析校验 → ready/finished promise
   （headless 帧精度如实标注）。基线 52/963。
3. **M4**：DC 逐项判定 + resource-timing/navigation-timing 重入条件挂账定稿
   （performance-timeline 余非 Pass ×8 + user-timing ×2 已归入）。

**待用户决策清单**：（空——无门控项）
