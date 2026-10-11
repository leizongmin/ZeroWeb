# Web Messaging 兼容 — postMessage / MessageChannel / BroadcastChannel

**版本**: v1.2
**日期**: 2026-10-12
**状态**: Active（M1 + M2 完成，M3 跨 document 挂门槛 + 收口待执行）
**执行模式**: WPT 驱动（上游 webmessaging corpus 为验收标尺）+ 实现/语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——页面间通信面）
**运行时控制面板**: [webmessaging-compat/master.md](webmessaging-compat/master.md)

> **▶ 当前状态（2026-10-12，M2-P2+P3 轮）**：M1（基线 64.9%）、M2-P1（postMessage
> spec 化，93.3%）、M2-P2+P3（BroadcastChannel IDL/语义 + messageerror 内容属性，
> **98.5% = 133/135**）均已落地，同 document 可执行面失败簇清零，残余 2 non-pass
> 全挂账（GC 联动 close、user-activation 跨域）。证据：
> [evidence/2026-10-12-m2-p2-p3-broadcastchannel-messageerror.md](webmessaging-compat/evidence/2026-10-12-m2-p2-p3-broadcastchannel-messageerror.md)。
> **下一步（M3 + 收口）**：跨 document 场景挂 navigation-compat M3 frame 树门槛
> 不单方面开；可执行面收口走 DC-3 全量门禁（make test / make reftest /
> product-smoke）后归档。
>
> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：多进程
> IPC（zero-protocol）有底子，但 window.postMessage/BroadcastChannel 的 Web 面
> 从未度量；真实站点的 iframe 通信/模块解耦依赖此面。
>
> **▶ 基线事实（M1 盘点定稿，2026-10-12）**：window.postMessage shim（R2983，
> 同 document 面）、MessageChannel/Port（part05）、BroadcastChannel（part05）均
> 存在；缺口在 targetOrigin URL 解析/options dictionary、transfer/ports 经
> window 面、BroadcastChannel IDL 校验、事件 task 承载。跨 document 场景
> （iframe）与 navigation-compat M3 frame 树同门槛。

## Mission

以 WPT webmessaging corpus 可执行子集为验收标准，收敛消息构造/校验（origin/
targetOrigin/transfer）/投递序语义。排除：worker 内 messaging（workers-compat
域）、WebSocket/SSE（net 域）、跨进程 fission（与 navigation-compat 同门槛）。

## Done Criteria

- **DC-1** 导入 + 基线 + csv planned 行回填
  - [x] corpus 导入（131 文件）+ 可执行子集甄别（72 案；排除面逐类挂重入门槛，
        evidence/2026-10-12-m1-baseline.md）
  - [x] 分类通过率基线落 evidence/，csv planned 行回填（2026-10-12 数据行
        87/134 = 64.9%，status=active）
- **DC-2** postMessage 校验/投递语义 + MessageChannel/Port + BroadcastChannel
  对齐（单 document/同源场景优先），分级通过率不再下行两个连续轮次
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值

## 里程碑

M1 导入与基线（✅ 2026-10-12，64.9%）→ M2 同源/单 document 面收敛（✅ 2026-10-12，
98.5%，失败簇清零见 master.md）→ M3 跨 document 场景（frame 树依赖部分挂
navigation-compat M3 门槛）+ 收口归档。

## 依赖约束（run-rules §9）

跨 frame 依赖统一挂 navigation-compat M3 门控，不单方面开；与 workers-compat
的 worker 侧 messaging 面协调归属。
