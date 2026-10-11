# Web Messaging 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webmessaging-compat.md](../webmessaging-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-12（M2-P1 落地）

## 当前状态

**M1 + M2-P1 完成（2026-10-12）**：基线 64.9%（87/134）→ **M2-P1 后 93.3%
（126/135，净 +39P）**——window.postMessage spec 化重写（options dictionary +
targetOrigin URL 解析/静默丢弃 + transfer 序列校验）、port 转移克隆-改挂、task 承载
投递、trusted message 事件、structuredClone 平台对象/普通对象更正。证据：
[evidence/2026-10-12-m2-p1-postmessage-spec.md](evidence/2026-10-12-m2-p1-postmessage-spec.md)。
影响面复跑：zero-engine lib 2998P/0F（R2983 桥接断言按 spec 更正）、fileapi
493/499 恒等、dom 54304P ≥ 基线。

**下一步（M2-P2）**：BroadcastChannel IDL/语义簇（6F：constructor no-new/缺名
TypeError、postMessage 必参/close 后 InvalidStateError、投递序
closing-and-creating、e.origin 已随 P1 修）+ messageerror 内容属性面（P3）。
残余挂账：garbage-collected（GC 联动）、user-activation（缺
`navigator.userActivation`，跨域 user-activation 域）。

## 里程碑台账

- **M1（2026-10-12）**：导入 + 通道 + 甄别 + 基线 64.9%。传输注记：raw.githubusercontent.com
  当轮持续超时，语料经 cdn.jsdelivr.net 同 pin 镜像等价落地（脚本仍指 raw 权威源）。
- **M2-P1（2026-10-12）**：postMessage spec 化重写 + transfer/ports + task 承载 +
  trusted 事件 + structuredClone 更正 → 93.3%（+39P）；影响面复跑全绿（见当前状态）。
- **M2-P2（未开始）**：BroadcastChannel IDL/语义簇。
- **M2-P3（未开始）**：零散面（messageerror 内容属性等）。
- **M3（未开始）**：跨 document 场景挂 navigation-compat M3 门槛；收口归档。

## 残余失败面（9 non-pass，P1 后）

| 簇 | non-pass | 归属 |
|---|---|---|
| BroadcastChannel IDL/语义 | 6 | M2-P2 |
| messageerror 内容属性 | 1 | M2-P3 |
| garbage-collected（GC 联动 close） | 1 Timeout | 挂账（引擎 GC 面） |
| user-activation（缺 navigator.userActivation） | 1 | 挂账（user-activation 域） |

## 排除面（挂账，非待办）

- 跨 document/iframe 全族（without-ports+with-ports 016-021、broadcastchannel
  多数、multi-globals、xsite）→ navigation-compat M3 frame 树门槛（goal 契约）。
- worker 面 5 案（worker.any.js、worker-post-after-close、025、workers、blobs、
  user_activation）→ workers-compat 管道域（goal Mission 排除）。
- HTTPS/分区/bfcache 3 案 → security-hardening 分区面。
- close-event dispatcher 族 3 案 → navigation-compat 同门槛 + dispatcher 资产导入评估。
- `.htm` 形态 7 案（.sub. 子域/iframe 面）→ 排除记账，可执行面无损。

## 后续重入条件（记账）

- 常驻断言集：testharness-webmessaging 通道（`make testharness-webmessaging`）+
  evidence/2026-10-12-m1-baseline.json——后续涉及 js_dom_shim postMessage/
  MessagePort/BroadcastChannel 的变更跑该通道防回归。
- R2983 spec 误读修正是 M2 第一簇的一部分：现 shim 失配抛 SecurityError、缺省按
  '*' 放行两处与 spec 不符（spec：失配静默丢弃；缺省 targetOrigin 为必参——
  单参形式走 options dictionary 重载）。
- fileapi-compat 残余挂账「FrozenArray<MessagePort> e.ports transfer 1（webmessaging）」
  待本 goal M2 簇 ② 落地后复验回流。
