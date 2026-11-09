# WPT 95% · 08 URL、Streams 与存储

**状态** Active\
**立项** 2026-11-09，按用户要求扩展到 95% 冲刺，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-data/master.md)\
**共同计分与运行规则** [95% 冲刺计划](../compat/wptfyi-95-plan.md)

## 基线与目标

11 月 7 日 [全量快照](../compat/wptfyi/coverage-2026-11-07.md) 已通过 10,120 / 39,390 项，缺口 29,270 项。`url` 缺 7,596 项，`IndexedDB` 缺 4,781 项，`streams` 缺 4,860 项，`urlpattern` 缺 3,266 项。复用 [IndexedDB](archive/storage-indexeddb/master.md)、[Service Workers](archive/service-workers.md) 和 [Workers](archive/workers-compat.md) 的既有实现。

同口径净增 **25,000** 项，达到至少 **35,120** 项通过。固定 revision 的净增与当前周跑的上游漂移分别报告。

## 范围与协作

计分目录为 `url/`、`urlpattern/`、`streams/`、`IndexedDB/`、`service-workers/`、`FileAPI/`、`fs/`、`workers/`、`compression/`、`storage/`、`web-locks/`、`webstorage/`、`cookiestore/`、`webmessaging/`。

仅计本入口所列目录。`fetch/` 和 `cookies/` 归 07，`encoding/` 和 `wasm/` 归 02。Worker 与存储实现经过真实宿主与隔离边界，保留 origin、持久化和事务保证。Service Worker 需要的多源夹具复用 07 的执行链。 共享文件按 [Rally 运行规则](../rally/run-rules.md) 分配归属。等待共享修复时，继续独立语料导入、对账和失败分类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。先核对 URL、URLPattern 固定向量和现有导入范围，再通过守卫跑 `testharness-indexeddb`、`testharness-workers` 及 Service Worker 代表簇。分别记录 window、worker、服务端依赖、持久化和跨源结果，定位现有实现未进入周跑的路径。 归档固定 revision、当前 commit、预期与实际执行路径，以及按路径和变体的差额。
- **DC-2 · 按簇收敛**。按 URL 与 URLPattern、Streams 背压和关闭顺序、IndexedDB 事务与升级、Service Worker 生命周期及 File API 分批推进。每轮核对不同上下文、持久化重开、失败回滚和隔离断言。 修复随附常驻 WPT 或等价回归。
- **DC-3 · 全量对账**。接入周跑并验收净增至少 25,000 项，邻域无可归因回退。完成守卫下的构建、测试与 clippy，保存有效分片和全量结果，再归档。

每轮更新控制面与证据，最后一行按 [共同计划](../compat/wptfyi-95-plan.md#无人值守运行) 输出 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`。
