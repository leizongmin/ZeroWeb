# Web Messaging 兼容 — postMessage / MessageChannel / BroadcastChannel

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active（已立项——无用户门控项）
**执行模式**: WPT 驱动（上游 webmessaging corpus 为验收标尺）+ 实现/语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——页面间通信面）

> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：多进程
> IPC（zero-protocol）有底子，但 window.postMessage/BroadcastChannel 的 Web 面
> 从未度量；真实站点的 iframe 通信/模块解耦依赖此面。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：postMessage shim 存在性、origin/
> targetOrigin 校验、MessageChannel/Port 传输、BroadcastChannel 同源组播——全部
> M1 盘点；跨 document 场景（iframe）与 navigation-compat M3 frame 树同门槛。

## Mission

以 WPT webmessaging corpus 可执行子集为验收标准，收敛消息构造/校验（origin/
targetOrigin/transfer）/投递序语义。排除：worker 内 messaging（workers-compat
域）、WebSocket/SSE（net 域）、跨进程 fission（与 navigation-compat 同门槛）。

## Done Criteria

- **DC-1** 导入 + 基线 + csv planned 行回填
- **DC-2** postMessage 校验/投递语义 + MessageChannel/Port + BroadcastChannel
  对齐（单 document/同源场景优先），分级通过率不再下行两个连续轮次
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值

## 里程碑

M1 导入与基线 → M2 同源/单 document 面收敛 → M3 跨 document 场景（frame 树
依赖部分挂 navigation-compat M3 门槛）+ 收口归档。

## 依赖约束（run-rules §9）

跨 frame 依赖统一挂 navigation-compat M3 门控，不单方面开；与 workers-compat
的 worker 侧 messaging 面协调归属。
