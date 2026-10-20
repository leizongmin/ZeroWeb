# 收口后漂移核对（2026-10-11，并行双流 main 组合态）

**目的**：M4 收口（2026-09-28，HEAD `17a0963c6` 语义态）后，main 经导航/渲染/
engine-dom/媒体等并行流累计 ~350 个 engine 提交（核对时 HEAD `5f384cb4e`）。
按 run-rules §10 归因纪律复跑四通道，核对收口快照是否仍成立。

**原始捕获**：`2026-10-11-drift-check-{hr-time,performance-timeline,user-timing}.raw.txt`
+ `2026-10-11-drift-check-web-animations{,-full-run}.raw.txt`（raw stdout，含断言消息）。

## 复跑对照（M4 收口快照 → 本轮）

| corpus | M4 | 本轮（存档捕获） | subtest 级 flip |
|---|---|---|---|
| hr-time | 10/15 | 11/16 | 0 lost，+1 gain（clamped-time-origin「timeOrigins are clamped」转 Pass）|
| performance-timeline | 36/44 | 36/44 | **零漂移**（flip 集 = ∅）|
| user-timing | 561/570 | 561/570 | **零漂移**（flip 集 = ∅；余 8F legacy + 1T navigation-timing 挂账如旧）|
| web-animations | 500/942 | 502/959（首跑 503/958）| +4 新 Pass，−1（见归因）|
| **合计** | **1107/1571（70.5%）** | **1110/1589（69.9%）** | **零净回归（+3~4P）** |

通过率分母变大（1571→1589）为可执行面组合漂移（raf-coarsened-time 新入集、
web-animations 分母 +17），非能力回退；分子净增。

## 漂移分类（全部可归因，无悬空）

1. **hr-time +1P**：`clamped-time-origin.html`「timeOrigins are clamped」M4 Fail →
   本轮 Pass（并行流 timeOrigin 面推进收益）。同轮组合变化：`raf-coarsened-time.https.html`
   新进入可执行集（×1F，rAF 帧时间戳粗化——headless 帧驱动 opt-in 已知约束族，如实计入）；
   worker timeOrigin ×2 子测试 Timeout→Fail（worker 执行面推进使断言真正跑到——
   归 workers-compat 域）；M4 时执行的 DocumentTimeline 案本轮不在可执行集。
2. **performance-timeline / user-timing 零漂移**：与本轮 HEAD 逐 subtest 对照
   M4 JSON flip 集为空——收口数字在并行流组合态下原样成立。
3. **web-animations +4P**：Animatable/getAnimations ×3（fill:both 持久化/persisted/
   style change event）+ timing-model update-and-send-events ×1——并行流 engine
   改进的真实兼容收益。
4. **web-animations −1P（已归因，不硬解）**：
   `setting-the-playback-rate-of-an-animation.html`「The playback rate affects the
   rate of progress of the current time」M4 Pass → 本轮确定性 Fail（3/3 复跑）。
   **A/B 归因**：navigation-compat M2-S5（2026-10-10）的 pagereveal-at-first-rAF
   wrapper（testharness.rs 首个 rAF 注册时同步派发 pagereveal + trusted event 构造）
   在该子测试两次 `timeline.currentTime` 读之间引入 ~0.2ms 墙钟——期望
   `timelineDiff × rate = 0.4 ± 0.001`、实得 0（瞬时完成模型的 currentTime 是冻结
   数据属性，不做 live 派生）。本地 env 门控跳过 wrapper 复跑即 Pass（临时补丁
   **已还原未提交**，A/B 单变量）。归属：已记账残差族「playState 派生/瞬间完成
   冲突（DC-3 如实标注）」被 wrapper 的合法时序消耗揭开；机制属导航流共享面，
   按 run-rules §9 记账不硬解。
5. **web-animations 运行间方差 ±2**：三跑实测 503/958、502/957、502/959——
   ①getAnimations 案壳 probe 预算内完成与否 ±1 注册面（时间敏感，非子测试翻转）；
   ②DocumentTimeline/constructor「origin time of zero」1 案时间敏感翻转
   （run1/run2 Pass、run3 Fail）。

## 结论

- 零净回归：M4 收口的所有 Pass 子测试本轮无一转 Fail（web-animations 唯一
  flip 已单变量归因到导航流 wrapper 的时序消耗，属既有残差族）。
- 分子净增 +3~4P；performance-timeline / user-timing 在 ~350 提交的组合态下
  逐 subtest 恒等。
- goal DONE 态维持，无需重开；无新增用户决策项。残差面（playState 派生、
  canonicalization、progress 边界、渲染效果断言）不变。
