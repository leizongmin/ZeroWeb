# Web Messaging 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webmessaging-compat.md](../webmessaging-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-12（M2-P2+P3 落地）

## 当前状态

**M1 + M2 全部落地（2026-10-12）**：基线 64.9%（87/134）→ M2-P1 93.3%（126/135，
+39P）→ **M2-P2+P3 后 98.5%（133/135，+7P）**。M2-P2：BroadcastChannel spec 化
（构造器 no-new/缺名 TypeError、postMessage 必参 + close 后先于序列化抛
InvalidStateError、投递对象快照 + task 承载）；M2-P3：messageerror 内容属性
（body/frameset Window-forwarding handler 全集补齐 + setAttribute 转发路径消费
R134 新值编译 fn）。证据：
[evidence/2026-10-12-m2-p2-p3-broadcastchannel-messageerror.md](evidence/2026-10-12-m2-p2-p3-broadcastchannel-messageerror.md)。
影响面复跑：zero-engine lib 2998P/0F 恒等、dom 54303P（non-Pass 集为通道既有
已知面，见 evidence 门禁表）。

**下一步（M3 + 收口）**：跨 document 场景挂 navigation-compat M3 frame 树门槛
（goal 契约，不单方面开）；可执行面残余 2 non-pass 全挂账（GC 联动 close、
user-activation 跨域）。收口按 DC-3 走全量门禁：make test 全绿 + make reftest
零回归 + product-smoke 恒值，随后归档。

## 里程碑台账

- **M1（2026-10-12）**：导入 + 通道 + 甄别 + 基线 64.9%。传输注记：raw.githubusercontent.com
  当轮持续超时，语料经 cdn.jsdelivr.net 同 pin 镜像等价落地（脚本仍指 raw 权威源）。
- **M2-P1（2026-10-12）**：postMessage spec 化重写 + transfer/ports + task 承载 +
  trusted 事件 + structuredClone 更正 → 93.3%（+39P）；影响面复跑全绿（见当前状态）。
- **M2-P2+P3（2026-10-12，同轮落地）**：BroadcastChannel IDL/语义 spec 化 +
  messageerror 内容属性（body WindowEventHandlers 全集 + setAttribute 转发消费
  R134 新值 fn）→ 98.5%（+7P）；可执行面失败簇清零，残余全挂账。
- **M3（未开始）**：跨 document 场景挂 navigation-compat M3 门槛；收口归档。

## 残余失败面（2 non-pass，P2+P3 后，全挂账）

| 簇 | non-pass | 归属 |
|---|---|---|
| garbage-collected（GC 联动 close） | 1 Timeout | 挂账（引擎 GC 面，超 JS shim 面） |
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
