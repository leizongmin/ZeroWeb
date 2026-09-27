# 计时与动画兼容 — 运行时控制面板（master.md）

**入口文档**: [../timing-animation-compat.md](../timing-animation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-27（M1 落地：四 corpus 导入 + 通道 + 基线 + CSV 回填）

---

## 当前状态

**专项定位**：四 goal 同批立项中的**轻量快赢切片**——hr-time/performance-timeline/
user-timing/WAAPI 纯 JS API 面，不触布局/渲染计算，预期最快出数字。
headless 帧驱动 opt-in（`__ZW_RAF_FRAME_DRIVEN`）是已知约束，如实标注不放容差。

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
| P1 | 四 corpus（hr-time/performance-timeline/user-timing/web-animations）导入 + 基线 | ✅ M1（2026-09-27，见上）|
| P2 | hr-time 精度/单调性/timeOrigin 语义 | ⏳ M2（基线缺口：toJSON/EventTarget 继承/timeOrigin epoch/DocumentTimeline）|
| P3 | user-timing mark/measure/getEntries* + PerformanceObserver 评估 | ⏳ M2（基线缺口：DOMException 校验簇 ~66、PerformanceMark/Measure 构造器、structured-clone detail、observer 派发时序 Timeout ×14）|
| P4 | WAAPI Animation/KeyframeEffect/getAnimations/playState + promise 语义 | ⏳ M3（基线缺口：KeyframeEffect/Animation/DocumentTimeline 构造器三主簇 ~690 subtests——web-animations 52/963 低基线主因）|
| P5 | resource-timing/navigation-timing 重入条件挂账 | ⏳ M4 |

## 已完成切片

- **M1（2026-09-27）**：四 corpus 导入 + runner 通道 + 基线 319/1376（23.2%）+
  suites CSV 回填。无源码语义改动（通道基建除外——wpt-runner 四子命令 +
  goals/10 脚本扩展）。

## 下一步计划

1. **M2-S1（下一轮起点，已勘域）**：user-timing 异常语义簇（~66 subtests，
   part01b.js R2821 performance shim）——
   - `mark()`/`measure()` 无参 → TypeError（现静默）
   - mark 名校验：strip 后空 / 保留 timing 属性名（navigationStart 等）→
     SyntaxError DOMException（code 12）
   - mark options 字典转换：number/NaN/Infinity/string 传 options → TypeError；
     `startTime < 0` → TypeError；`detail` 不可序列化 → DataCloneError
   - `measure(measureName, startMark, endMark)` 联合类型：(DOMString or options
     dict)——**number 实参按 DOMString 转换后查 mark 表**（`51.15`→`"51.15"` 查无 →
     SyntaxError DOMException），删除现 shim「number 当原始时间戳」的非规范路径
   - measure options dict 形态 `{start,end,duration,detail}` 支持（现抛
     `The mark '[object Object]' does not exist`——structured-serialize-detail ×8 依赖）
   - PerformanceMark / PerformanceMeasure 构造器接口（~13 subtests）+
     `entry.detail` 缺省 null 语义
   - 前置确认：shim 环境 DOMException 全局与 structuredClone 可用性（assert_throws_dom
     按 DOMException 原型断言）
2. **M2-S2**：PerformanceObserver 派发时序（performance-timeline Timeout ×9 +
   user-timing ×5——`_defer` microtask 近似 vs 任务队列语义）
3. **M2-S3**：hr-time——`performance.toJSON` + Performance EventTarget 继承 +
   timeOrigin epoch 锚定 + clamped-time-origin Timeout 拆解
4. **M3**：WAAPI——Animation/KeyframeEffect/DocumentTimeline 构造器 + effect 属性桥
   （三主簇 ~690 subtests）→ 关键帧参数解析校验 → ready/finished promise
   （headless 帧精度如实标注）
5. **M4**：DC 逐项判定 + resource-timing/navigation-timing 重入条件挂账定稿

**待用户决策清单**：（空——无门控项）
