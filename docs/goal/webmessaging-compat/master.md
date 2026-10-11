# Web Messaging 兼容 — 运行时控制面板（master.md）

**入口文档**: [../webmessaging-compat.md](../webmessaging-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-12（M1 基线落地）

## 当前状态

**M1 完成（2026-10-12）**：corpus 导入（131 文件，WPT pin
`315976933870b34d6ea30e3f6643403edae678ba`）+ runner 通道
（`testharness-webmessaging`，Makefile target 同名）+ 可执行子集甄别
**72 案 134 子测试 87 Pass = 64.9%**（46 F + 1 T，零静默误伤对账）。基线证据：
[evidence/2026-10-12-m1-baseline.md](evidence/2026-10-12-m1-baseline.md)。

**下一步（M2）**：按失败簇收敛——① targetOrigin URL 解析 + options dictionary
签名簇（~19 F，最大簇，含 R2983 的 spec 误读：失配应静默丢弃而非抛 SecurityError、
不可解析应 SYNTAX_ERR）；② transfer/ports 传递簇（~11 F，window.postMessage 第三
参数 WebIDL sequence 校验 + ports 随消息转移——fileapi M2-P4 已修
serialize-with-transfer 序，此处补 window 面）；③ BroadcastChannel IDL/语义簇
（7 F）；④ 零散面（isTrusted、port 投递 task 承载）。DC-2 门 = 分级通过率不再
下行两个连续轮次。

## 里程碑台账

- **M1（2026-10-12）**：导入 + 通道 + 甄别 + 基线 64.9%。传输注记：raw.githubusercontent.com
  当轮持续超时，语料经 cdn.jsdelivr.net 同 pin 镜像等价落地（脚本仍指 raw 权威源）。
- **M2（未开始）**：失败簇收敛（见下一步）。
- **M3（未开始）**：跨 document 场景挂 navigation-compat M3 门槛；收口归档。

## 失败簇 → M2 计划（47 non-pass）

| 簇 | non-pass | 要点 |
|---|---|---|
| targetOrigin 解析 + options dictionary | ~19 | SYNTAX_ERR/origin 归一/失配静默丢弃/`{targetOrigin,transfer}` 解包 |
| transfer/ports 传递 | ~11 | window.postMessage transfer 序列校验、ports 转移、e.ports frozen |
| BroadcastChannel IDL/语义 | 7 | constructor/postMessage/close 的 TypeError/InvalidStateError、e.origin、投递序 |
| 零散面 | 4 | isTrusted、port 投递 task 承载（fileapi M2-P3 先例）、onmessageerror 内容属性、structured clone own-properties |
| 挂账（跨域/超 shim 面） | 2 | garbage-collected GC 驱动 close（引擎 GC 联动）；user-activation 缺 `navigator.userActivation` |

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
