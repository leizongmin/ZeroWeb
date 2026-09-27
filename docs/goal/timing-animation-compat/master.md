# 计时与动画兼容 — 运行时控制面板（master.md）

**入口文档**: [../timing-animation-compat.md](../timing-animation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-28（M4 收口判定：DC-1~4 全 ✅，goal DONE）

---

## 当前状态

**专项定位**：四 goal 同批立项中的**轻量快赢切片**——hr-time/performance-timeline/
user-timing/WAAPI 纯 JS API 面，不触布局/渲染计算，预期最快出数字。
headless 帧驱动 opt-in（`__ZW_RAF_FRAME_DRIVEN`）是已知约束，如实标注不放容差。

**M4 收口判定（2026-09-28）**：**DC-1~4 全部满足，goal DONE。** 收口快照：
**1107/1571 subtests（70.5%）**（基线 319/1376 = 23.2%），135 案 window 可执行子集；
`make reftest` 收口实测 691 案 0 不一致（零回归）；make test 全绿 + clippy
`-D warnings` + fmt 干净。挂账定稿：resource-timing/navigation-timing 重入 =
navigation-compat M2 落地后评估（归入排除面 ×11 全部分类）；残差各自域记账
（COOP/COEP 隔离 infra、worker 面、L1/L2 legacy、playState 派生〔DC-3 如实标注〕、
渲染效果断言〔rendering-compat〕）。判定 evidence：
[evidence/2026-09-28-m4-dc-verdict.md](evidence/2026-09-28-m4-dc-verdict.md)。

**轨迹**：M1 资产（319/1376 = 23.2%）→ M2-S1 user-timing 异常语义（241→518P）→
M2-S2 PerformanceObserver 语义（performance-timeline 20→35P）→ M2-S3 hr-time 语义
（hr-time 7→10P）→ M3-S1 WAAPI 构造器（web-animations 52→338P）→ M3-S2
timing-model 计算（338→483P）→ M3-S3 WAAPI 收口（483→502P）→ M4 判定。

**通道基建（长期资产）**：
- fetch：`tests/wpt-runner/scripts/goals/10-timing-animation-compat.sh`（四 corpus +
  web-animations interfaces ×8 / timing-model ×4 深面 + `.any.js` 显式清单 ×42）
- runner：`testharness-{hr-time,performance-timeline,user-timing,web-animations}` 四
  子命令（`run_timing_subdirs` 共用扫描器：`.html` 直跑 + `.any.js` window 变体 +
  绝对路径 helper inline_extras）
- Makefile：`fetch-wpt-timing-animation` + 四 testharness target

**与兄弟 goal 的边界**：
- rendering-compat — CSS animation/transition 渲染效果面归其（M3-S1 起渲染效果
  断言 ~200 持续记账于此）
- workers-compat — worker 执行面（hr-time worker 子测试 ×2T 归入）
- security/net — COOP/COEP 隔离 infra（hr-time crossOriginIsolated ×2F 归入）
- navigation-compat — resource/navigation-timing 重入评估触发点（M2 落地后）

**M2 全部落地（2026-09-28，S1+S2+S3 三切片）**：计时面（hr-time + performance-timeline +
user-timing）收敛完成。余非 Pass 全分类：
L1/L2 legacy 上游同失败、COOP/COEP 隔离 infra、worker 面、resource/navigation-timing
排除域（→M4 挂账）。

**M2-S3（2026-09-28）**：hr-time——performance.toJSON + Navigation Timing L1 桩
（unavailable=0、navigationStart 现场 epoch getter）+ Performance EventTarget 继承 +
now() 0.1ms 粗化网格（非隔离 100μs 分辨率，floor 保单调）+ timeOrigin epoch 锚定
惰性 getter。见
[evidence/2026-09-28-m2-s3-hr-time-semantics.md](evidence/2026-09-28-m2-s3-hr-time-semantics.md)。

**M2-S2（2026-09-28）**：PerformanceObserver 语义修齐——callback 签名
`(entries, observer, options)` + `this=observer`（11 案 Timeout 簇根因）、
PerformanceObserverEntryList 接口、buffered 回放、observe TypeError/
InvalidModificationError、type 叠加/entryTypes 替换、派发批 startTime stable 排序、
disconnect 取消 pending flush。见
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
| P2 | hr-time 精度/单调性/timeOrigin 语义 | ✅ M2-S3（2026-09-28，10/15；余 = 隔离 infra ×2F〔跨域记账〕+ DocumentTimeline ×1F〔M3 已闭合〕+ worker ×2T〔排除〕）|
| P3 | user-timing mark/measure/getEntries* + PerformanceObserver 评估 | ✅ M2-S1/S2/S3（561/570 = 98.4%；余 = legacy ×8F + resource 域 ×1T〔挂账〕）|
| P4 | WAAPI Animation/KeyframeEffect/getAnimations/playState + promise 语义 | ✅ M3-S1/S2/S3（2026-09-28，500/942；余 = playState 派生〔瞬间完成冲突，DC-3 如实标注〕+ 渲染效果断言〔rendering-compat 记账〕+ 低密度边界精修）|
| P5 | resource-timing/navigation-timing 重入条件挂账 | ✅ M4（2026-09-28 定稿：重入 = navigation-compat M2 落地后评估；归入排除面 ×11 全分类，见 m4-dc-verdict）|

## 已完成切片

- **M4（2026-09-28）**：收口判定——DC-1~4 全 ✅，goal DONE；reftest 691/691 零
  回归实测；挂账定稿。
- **M3-S3（2026-09-28）**：WAAPI 收口（hold time/边界/校验/commitStyles）——
  web-animations 483→502P。
- **M3-S2（2026-09-28）**：timing-model 相位/进度 + 关键帧校验——web-animations
  338→483P。
- **M3-S1（2026-09-28）**：WAAPI 构造器 + effect 桥——web-animations 52→338P。
- **M2-S3（2026-09-28）**：hr-time 语义修齐（toJSON/EventTarget/粗化网格/timeOrigin
  epoch）hr-time 7→10P。
- **M2-S2（2026-09-28）**：PerformanceObserver 语义修齐——performance-timeline
  20→35P、user-timing observer Timeout 清零。
- **M2-S1（2026-09-28）**：user-timing 异常语义簇修齐 241→518P（98.1%）。
- **M1（2026-09-27）**：四 corpus 导入 + runner 通道 + 基线 319/1376（23.2%）+
  suites CSV 回填。

## 下一步计划

（goal DONE——无后续里程碑。残余优化面供后续可选重入：playState 派生重构
〔深结构，需与瞬间完成状态机协同设计〕、关键帧序列化 canonicalization、
progress 压线边界精修、渲染效果断言〔rendering-compat 域〕。）

**待用户决策清单**：（空——无门控项）

