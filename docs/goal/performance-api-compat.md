# Performance API 兼容 — resource / navigation / event-timing / longtask 扩面

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active（已立项——缺面甄别在 M1）
**执行模式**: WPT 驱动（上游 performance-api corpus 为验收标尺）+ 实现/语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——可观测性面）

> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：timing
> goal（编号 10，已收口）只圈了 hr-time/performance-timeline/user-timing/
> web-animations；performance-api 四件套（resource-timing/navigation-timing/
> event-timing/longtask-timing）从未度量——它们是真实站点性能监控的基础面。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：performance.now/getEntries 部分面
> 已有（timing goal 遗产）；resource-timing 条目生成（资源装载钩子）、
> navigation-timing（导航生命周期数据）、event-timing（事件循环集成）、longtask
> （长任务判定）——host 数据管道可用性全部待 M1 甄别，缺面部分如实挂账。

## Mission

以 WPT performance-api corpus 可执行子集为验收标准，逐步补齐四个 timing 面
的条目生成与属性语义。排除：PerformanceObserver 基础机制（timing goal 遗产，
消费之不重定义）、user-timing/WAAPI（编号 10 域）、服务端/传输层计时。

## Done Criteria

- **DC-1** 导入 + 基线 + 缺面甄别报告（哪些子域因 host 管道缺失不可执行，逐项
  理由落 evidence/）+ csv planned 行回填
- **DC-2** 可执行子域条目语义对齐，分级通过率不再下行两个连续轮次
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值

## 里程碑

M1 导入与缺面甄别 → M2 可执行子域收敛（event-timing/longtask 优先——纯
事件循环面）→ M3 resource/navigation-timing host 管道（缺面大则挂账带数字收口）
→ 收口归档。

## 依赖约束（run-rules §9）

与 timing-animation-compat（编号 10，已收口）遗产边界清晰；resource-timing 需
net 装载钩子时与 net 域既有测试资产协调；渲染面零交集。
