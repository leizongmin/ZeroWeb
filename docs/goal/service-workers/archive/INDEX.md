# Service Worker 真实化 — 归档索引

**创建**: 2026-09-06（goal 收口；本目录存已完成里程碑的决策记录与历史证据索引）
**入口**: [../../service-workers.md](../../service-workers.md) · 控制面: [../master.md](../master.md)

## 归档策略说明

`evidence/*.tsv` 资产清单是 `Makefile` fetch/audit 目标的默认输入
（`WPT_ASSET_MANIFEST` 路径硬耦合），`evidence/*.md` 被 master.md 大量相对链接
引用——两者均**原地保留**，不迁入本目录。本目录收录：

1. 已完成里程碑的**决策记录快照**（本文档）
2. 上游 WPT 可执行面的历史裁决账本指针（裁决已固化为 runner disposition）

## 里程碑归档快照

### M0 — 选型 RFC（✅ 2026-08-19 方案 C 获批）

- 决策：抽取 `zero-script-sandbox::WorkerRuntime` 线程核 → typed
  `ServiceWorkerRuntime`；browser process `ServiceWorkerManager` 单一 owner；
  WebView 同算法 in-process adapter。全文见
  [../m0-execution-environment-rfc.md](../m0-execution-environment-rfc.md)
- WPT 可执行面裁决：294 testharness 源 / 331 URL → 48 core / 35 defer /
  169 gated / 42 skip，机器 contract
  [../evidence/2026-08-19-wpt-disposition.tsv](../evidence/2026-08-19-wpt-disposition.tsv)
  （`make audit-wpt-service-workers-disposition` 可确定性重建）
- 候选资源闭包 / Tier A 合约 / 逐文件清单：evidence/ 下
  `2026-08-19-m0-*`、`2026-08-19-m1-tier-a-*`

### M1 — 脚本真实执行 + 生命周期真事件（✅ core WPT 65/249，与 M3 共同收敛）

- 过程切片 M1-1..M1-5c：threaded runtime → manager slots → lifecycle runtime →
  WebView host bridge → page bridge → IPC contract → browser owner → renderer
  bridge → registration discovery → lifecycle task projection → registration URL
  contract。逐切片证据：evidence/ 下 `2026-08-19-m1-*.md`
- core WPT 从 12 case / 36 subtest 起步至 65/249 全绿

### M2 — fetch 拦截 + Cache 集成（✅ fetch/message 31/85 + CacheStorage 25/318 全绿）

- runtime foundation → production 路由 → caches.match/open/put/matchAll/keys →
  worker fetch → Cache.delete/listing → 31 个 fetch/message WPT 案（含 M2-44
  streaming/cancel 全案）→ 25 个 CacheStorage 案。逐案证据：evidence/ 下
  `2026-08-2x-m2-*.md`
- 生产页面 fetch respondWith/pass-through 与 registration-local CacheStorage
  持久化均已接入生产链路

### M3 — 控制语义 + 消息 + 收尾（✅ 与 M1 core 249/249 收敛）

- skipWaiting / controller / clients.claim / postMessage 双向 / update /
  persistence / updateViaCache / importScripts / module graph / client registry
  / iframe client lifecycle / 生命周期 error 边界 / waitUntil settle 语义 /
  controller-on-load/reload/disconnect。逐切片证据：evidence/ 下
  `2026-08-2x-m3-*.md`、`2026-09-0x-m3-*.md`

### 2026-09-06 质量收口

- M3-68 cache-put getReader disturb（潜伏红项根因：body getter 加入晚于基线）
- M3-69 extendable-message-event wave manifest 闭包（漏登记 2 worker fixture）
- M2-44 streaming/cancel 专项（cancel/abort 跨 runtime 反传 + settle 语义对齐
  spec + pump watchdog）——evidence/
  [2026-09-06-m2-streaming-cancel-slice.md](../evidence/2026-09-06-m2-streaming-cancel-slice.md)
- skip-waiting flake 根因修复（controller-on-load 初始赋值非 change，82fcb379f）
