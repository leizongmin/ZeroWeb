# 计时与动画兼容 — 运行时控制面板（master.md）

**入口文档**: [../timing-animation-compat.md](../timing-animation-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：四 goal 同批立项中的**轻量快赢切片**——hr-time/performance-timeline/
user-timing/WAAPI 纯 JS API 面，不触布局/渲染计算，预期最快出数字。
headless 帧驱动 opt-in（`__ZW_RAF_FRAME_DRIVEN`）是已知约束，如实标注不放容差。

**与兄弟 goal 的边界**：
- rendering-compat — CSS animation/transition 渲染效果面归其；不碰
  style-system/layout-engine/render-foundation 动画计算
- event-loop-spec（已归档）— rAF/微任务先例消费；IO/RO 已立账处不重复
- keyboard-page-scrolling（已归档）— 帧驱动门控为消费事实，改动须跨流核对
- 同批 net-api/navigation/workers — 无共享面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 四 corpus（hr-time/performance-timeline/user-timing/web-animations）导入 + 基线 | ⏳ M1 纯资产 |
| P2 | hr-time 精度/单调性/timeOrigin 语义 | ⏳ M2 |
| P3 | user-timing mark/measure/getEntries* + PerformanceObserver 评估 | ⏳ M2 |
| P4 | WAAPI Animation/KeyframeEffect/getAnimations/playState + promise 语义 | ⏳ M3 |
| P5 | resource-timing/navigation-timing 重入条件挂账 | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：四 corpus fetch 脚本 + 导入 + 基线（纯资产）+ suites CSV 回填
2. **M2-M4**：按入口文档里程碑逐面收敛

**待用户决策清单**：（空——启动顺序由用户点名）
