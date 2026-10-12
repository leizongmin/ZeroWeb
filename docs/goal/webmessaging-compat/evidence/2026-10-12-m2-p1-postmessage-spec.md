# Web Messaging M2-P1 — postMessage spec 化重写 + transfer/ports（2026-10-12）

**64.9% → 93.3%（87/134 → 126/135，净 +39P）**。JSON 全量：
[2026-10-12-m2-p1-postmessage-spec.json](2026-10-12-m2-p1-postmessage-spec.json)。
复现：`make testharness-webmessaging`。

## 变更面（js_dom_shim）

1. **`window.postMessage` spec 化重写**（part06 R2983 → window post message steps，
   HTML #dom-window-postmessage）：
   - message 必参（零参 TypeError，009 钉）；
   - WebIDL 重载甄别：第二参 string → legacy（targetOrigin, transfer）；其余 → options
     dictionary `WindowPostMessageOptions{targetOrigin="/", transfer=[]}`（with-options
     8 案全翻绿——`{targetOrigin,transfer}` 解包 + `/` 缺省 + `{transfer:null}` TypeError）；
   - targetOrigin：`/` → 本 origin；`*` → 放行；其余**无 base 绝对 URL 解析**——失败
     SyntaxError（001/003 钉，legacy code 12）；解析成功 serialized origin 比较，**失配
     静默丢弃非抛**（015 钉；R2983 旧同步 SecurityError 为 spec 误读，本轮更正）；
     尾斜杠/host-specific 形态按 URL 归一放行（002/006 钉）；
   - transfer：sequence<object> 转换（023 null / 025 1 / 026 {length:1} 无 item / 027
     通道对象全 TypeError）→ port 转移克隆 → 非 port detach（serialize-with-transfer 序，
     fileapi P4 先例）；
   - `e.ports` frozen 数组（ports_readonly_array 钉 push 抛）；
   - `e.data === undefined` 直置（010 钉；构造器 dictionary 缺省语义落 null，UA 派发按
     内部槽直置）。
2. **MessagePort.postMessage 升级**（part05）：transfer 双形态（带 @@iterator → legacy
   sequence；其余 → options dictionary——dictionary-transferrable 钉
   `port.postMessage(ab, {transfer:[ab]})` 送达）；closed/source port 入 transfer →
   DataCloneError（close.any / clone_port_error 钉）；port 转移 = **克隆**——新 port
   接管配对（对端改挂 clone）+ port message queue/enabled 状态随迁
   （clone_port 钉 clone ≠ 原对象且可收对端消息；with_transfer_incoming 钉未启用多手
   转移 enable 后保序收齐）；对端已关丢弃（close.any「sent to closed port」）。
3. **投递承载 task 化**（spec「queue a global task on the posted message task source」）：
   window/port/BC 三发送面 queueMicrotask → setTimeout——runner 真 task 下 port
   onmessage 自馈环不再饿死 timer（infinite_loop 翻绿）；沙箱无 host 时 shim setTimeout
   回落 microtask——桥接测试「下 execute 可读」契约不变（R2782/R2783 原样绿）。
4. **UA 派发 message 事件 trusted**（R312 `__zwTrusted` + `_zwUaDispatch` 印记经共用
   工厂 `_zwMakeMessageEvent`）——MessageEvent-trusted 钉 isTrusted=true。
5. **structuredClone 平台对象分支 + 普通对象原型更正**（part02）：Blob/File 克隆为同
   类型实例（Channel_postMessage_Blob 钉 instanceof）；非特殊对象克隆为普通对象
   （Object.prototype）——旧版保留原型致原型链属性随读可见（023 钉 cloned.c ≠ 原型值）。

## 门禁（影响面复跑）

| 通道 | 结果 | 对比基线 |
|---|---|---|
| zero-engine lib（桥接） | 2998P / 0F | +1 绿——R2983 桥接测试「失配抛 SecurityError」断言按 spec 更正为「失配静默丢弃 + 不可解析 SyntaxError」（旧断言固化 spec 误读，WPT 015/001/003 与真实浏览器一致口径），测试体内注记本轮依据 |
| testharness-fileapi | 493/499 = 98.8% | 与收口基线逐字节恒等（6 残余 F 均在案跨域挂账） |
| testharness-dom | 54304P（54492 子测试） | ≥ 记录 54303P（分母漂移来自并行流语料增长；F/T/PreconditionFailed 集为通道既有已知面） |
| fmt / clippy | clean（zero-engine -D warnings） | — |

## 残余 9 non-pass（簇归属）

- BroadcastChannel IDL/语义 6F → M2-P2（constructor no-new/缺名 TypeError、
  postMessage 必参与 close 后 InvalidStateError、投递序 closing-and-creating）。
- user-activation 1F → 根因 `navigator.userActivation.isActive` 缺失（user-activation
  域，messaging 面已通）。
- messageerror 内容属性 1F → M2-P3（onmessageerror on* 编译/派发面）。
- garbage-collected 1 Timeout → 挂账（引擎 GC 联动 close 事件，超 JS shim 面）。
