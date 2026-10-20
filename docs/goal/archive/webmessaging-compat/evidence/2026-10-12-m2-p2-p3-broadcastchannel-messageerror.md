# Web Messaging M2-P2+P3 — BroadcastChannel IDL/语义 + messageerror 内容属性（2026-10-12）

**93.3% → 98.5%（126/135 → 133/135，净 +7P）**。JSON 全量：
[2026-10-12-m2-p2-p3-broadcastchannel-messageerror.json](2026-10-12-m2-p2-p3-broadcastchannel-messageerror.json)。
复现：`make testharness-webmessaging`（本轮 JSON 经 test-guard 同参数 + `--json` 捕获，
与该通道 Makefile 入口同一 runner 面复核逐字节一致）。

## 变更面（js_dom_shim）

1. **BroadcastChannel spec 化**（part05，HTML #broadcastchannel）：
   - 构造器 IDL——无 `new` TypeError（basics 钉，旧 lenient `return new` 移除）；name
     必参 TypeError（缺参才抛；null/undefined/数字按 USVString 转 `'null'`/`'undefined'`/`'123'`）；
   - postMessage IDL——message 必参 TypeError（interface 钉）；**close 后必抛
     InvalidStateError 且先于序列化**（interface 钉 `postMessage(Symbol())` close 后为
     InvalidStateError 非 DataCloneError；open 时 uncloneable 仍 DataCloneError）；
   - 投递对象快照（post 时注册表成员）+ task 承载（P1 ③ 同款 setTimeout）——
     create-in-onmessage 钉「投递期新建 channel 不收早前已 post 消息」；派发时已
     close 的成员跳过（closed-after-postMessage 钉）。
2. **body/frameset Window-forwarding handler 全集**（part01 `_ZW_BODY_FORWARD_ON`）：
   补齐 spec HTML WindowEventHandlers 全集（onafterprint/onbeforeprint/onbeforeunload/
   onhashchange/onlanguagechange/**onmessage/onmessageerror**/onoffline/ononline/
   onpagehide/onpageshow/onpopstate/onrejectionhandled/onstorage/onunhandledrejection/
   onunload）——messageerror.html 钉 body onmessageerror 内容属性经 window 派发触发。
3. **body 转发路径消费 R134 已编译 fn**（part04）：setAttribute on* 的 R134 块已用
   **新值 v** 编译 fn 入 `_onHandlers` 缓存；旧转发路径此处又清缓存并按 host attr 回读
   重编译——host set_attr 异步批处理未应用时回读为空 → fn 丢失 → window getter 落
   null（messageerror.html「compiled into property」钉 typeof function）。改为直接消费
   R134 编译结果，删除二次重编译。

## 门禁（影响面复跑）

| 通道 | 结果 | 对比基线 |
|---|---|---|
| testharness-webmessaging | 133/135 = 98.5% | +7P（BC 6F + messageerror 3P 全翻绿；residual 见下） |
| zero-engine lib（桥接） | 2998P / 0F | 与 M2-P1 记录恒等。注：`test_pipeline_deeply_nested_html` 须 `RUST_MIN_STACK≥4MiB`（测试文档注释在案，`make test` 设 32MiB），裸 `cargo test` 下该案必栈溢出——首轮直跑踩坑，带 env 复跑 2998P/0F |
| testharness-dom | 双跑 54303P / 54304P（54492 子测试） | 54304P 跑与 M2-P1 记录逐项一致；双跑 non-Pass 集 diff 恰 1 行 = `Event-timestamp-high-resolution.https.html :: GamepadEvent timestamp`（M2-P5 在册 flake 族成员，同二进制同树 Pass↔Fail 漂移）——**本轮改动零回归**归因闭环 |

纯 .js shim 变更无 .rs——按门禁表仅验证受影响构建与行为（fileapi M2-P5 先例）。

## 残余 2 non-pass（全挂账，簇清零）

- garbage-collected 1 Timeout → 挂账（引擎 GC 联动 close 事件，超 JS shim 面）。
- user-activation 1F → 根因 `navigator.userActivation.isActive` 缺失（user-activation
  域，messaging 面已通）。
