# 页面状态兼容 — sendBeacon / Page Visibility / online-offline

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active（已立项——无用户门控项）
**执行模式**: WPT 驱动（上游三小 corpus 为验收标尺）+ 实现与语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——页面生命周期与上报面）

> **▶ 拆分动机（2026-10-08 html5test 缺口立项，用户「立项」）**：三小套件打包
> （event-loop-spec 多套件先例）——sendBeacon（埋点上报基础设施）+ visibility
> 状态（真实站点的暂停/恢复判据）+ online/offline 事件，均是小面大价值。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：sendBeacon 无命中（net POST 底座
> 现成，薄封装即得）；document.visibilityState/visibilitychange 与
> navigator.onLine/online/offline 事件存在性 M1 盘点（event-loop 遗产消费）。

## Mission

以 WPT 三 corpus 可执行子集为验收标准：① `navigator.sendBeacon(url, data)`
fire-and-forget 语义（POST 投递/数据类型面/返回布尔）；② visibility 状态机
（visibilitychange 序、prerender 值甄别）；③ online/offline 事件语义。

**关键约束**：
1. sendBeacon **不阻塞卸载**（unload 窗口内仍可入队）——上报可靠性语义核心
2. visibility 状态在静态 runner 的口径如实甄别（headless 无真实 minimize，
   按 corpus 可执行子集定分母）
3. 事件消费 event-loop 遗产，不重定义任务源

### 排除（明确不在范围内）

- Service Worker 的 beacon 拦截语义 — service-workers 已归档 goal 余账
- Network Information API（connection 类型）— 未立项域，挂账
- 后台页真实节流 — 多窗口/OS 集成面，挂账

---

## Done Criteria

- **DC-1** 三 corpus 导入 + 基线 + csv planned 行回填
- **DC-2** sendBeacon 语义 + visibility/online 事件面对齐，分级通过率不再下行
  两个连续轮次
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值

## 里程碑

M1 导入与基线 → M2 sendBeacon 实现 + 事件面收敛 → M3 收口归档。

## 依赖约束（run-rules §9）

sendBeacon 实现走 net POST 底座（zero-net 本体不动）；与 event-loop-spec
（已归档）遗产为消费关系；渲染面零交集。
