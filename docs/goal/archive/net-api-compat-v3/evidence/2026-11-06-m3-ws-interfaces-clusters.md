# M3（一）：ws interfaces 九簇收敛——WebIDL 接口面基建 + fail-connection 语义

日期：2026-11-06。前置：M2（三）响应头控制面批次（基线 13,064P/1,612F/16T，
基线日志同树 `/tmp/netapi-m2-full-cur.log` 快照对照）。本批只改
`crates/engine/src/js_dom_shim/part06.js`（WebSocket/CloseEvent shim 单文件），
零 fixture 改动、零 Rust 改动。

## 一、改动内容（八刀，均引上游案面）

1. **CloseEvent ctor 内部槽 + prototype accessor**：`wasClean`/`code` 落
   `_zwWasClean`/`_zwCeCode` 内部槽，prototype getter-only accessor（set:
   undefined，configurable: true——close-basic 断言赋值忽略 + `delete
   e.wasClean` 无效 + `delete CloseEvent.prototype.wasClean` 后读 undefined）。
   code 缺省 **0**（CloseEvent/constructor.html 断言面；1005 是 close 事件
   派发面缺省——泵 close 臂维持派发面语义，两缺省分立）。
2. **CloseEvent.prototype Symbol.toStringTag = 'CloseEvent'**（events/018
   `close e.toString() === '[object CloseEvent]'`；继承 Event 的 'Event' tag
   须覆写，MessageEvent 同形先例）。
3. **`_zwWsInvokeListeners` 共享遍历**：函数与 `{handleEvent: fn}` 双形
   （EventListener 接口形）、`currentTarget`/`eventPhase = AT_TARGET(2)`
   派发中标记后复位（events/017 断言面）、snapshot 快照遍历（派发中增删
   listener 不影响本趟）。
4. **`_zwWsSetHandler` on* 双语义**：(a) 值存取——函数或 `{handleEvent}`
   对象**原样存**（events/013 读回 === 原对象），其余非 callable 落 null
   （`[LegacyTreatNonCallableAsNull]`：`ws.onopen = 123` → null）；(b) 派发
   接线——旧哨兵 removeEventListener 后按新值注册 wrapper（注册序即派发
   序，events/016），wrapper 只调函数形（013 断言对象形 on* 不触发派发）。
5. **url/readyState/bufferedAmount 内部槽 + getter-only accessor**（实例
   赋值静默忽略——url/002、readyState/002、bufferedAmount-readonly；继承面
   可读——bufferedAmount-deleting；`delete WebSocket.prototype.url` 后
   undefined——url/003）。`extensions`/`protocol` 无 accessor 断言面，维持
   实例 own。
6. **WebIDL 常量双挂 ReadOnly**：`{writable: false, enumerable: true,
   configurable: false}` 挂构造器与 prototype（constants/002 赋值忽略、003
   delete 忽略、005/006 redefine 抛 TypeError；`ws.OPEN`/`this.CLOSED` 双
   读取面）。
7. **prototype.dispatchEvent**：typeof 检查 + 共享遍历手动派发
   （events/003/004/007/008/009/019——on*/listener 按注册序、移除后不触发）。
8. **send 三态 + fail the WebSocket connection**：
   - send：无参 → TypeError（WebIDL 入口，send/001）；CONNECTING →
     InvalidStateError（send/002 面）；CLOSING/CLOSED → 静默 return
     undefined（send/007/008）。
   - err wire 臂只标记 `this._zwErrSeen = true` 不派发；close wire 臂
     readyState 先 CLOSED，再按 `!dispatched && (!clean || errSeen)` 补派
     error 恰一次，后派 close。
     https://html.spec.whatwg.org/#fail-the-websocket-connection

## 二、fail 面派发缺口修复（018 二轮根因）

首轮 `!clean && !dispatched` 门在协议错误序（宿主
`ws_bridge` 泵 Err 臂 = err wire → close(1006, clean=false)）下被 err 臂
置位的同一 flag 短路——err 标记即 `_errorDispatched = true`，close 臂
`!dispatched` 恒 false，error 永不派发。events/018 的 errorFired 断言挂
而 events/015 过的原因：015 的 onerror 断言全部位于 listener 内部，
listener 不触发也无断言失败（onclose `t.done()` 照常收案）——**通过不证明
派发**。修复拆双标记：`_zwErrSeen`（err wire 证据）与 `_errorDispatched`
（恰一次门）分立，close 臂 `errSeen` 支路补派。CONNECTING close 宿主仲裁
序（单 close wire clean=false 无 err wire）走 `!clean` 支路不变，
close-connecting-async「error 先于 close 且恰一次」复验通过。

## 三、定向 A/B（ws interfaces 面，同树基线对照）

88P/45F → **133P/0F**（+45P）：九簇全数收口——接口常量 ReadOnly 16F /
EventTarget dispatchEvent 9F / 属性描述符 6F / 事件处理器 handleEvent 形
4F / send 状态机 3F / 协议错误断连 3F / CONNECTING fail 语义 2F / 事件序
1F / CloseEvent ctor 1F。27 文件清单（= 全量 A/B changed 面）：
CloseEvent/constructor、bufferedAmount ×2、close ×2（basic/
connecting-async）、constants ×4、events ×12（003/004/007/008/009/013/
015/016/017/018/019）、readyState ×2、send ×3（001/007/008）、url ×2。

## 四、全量 A/B（零回归门）

同树基线 13,064P/1,612F/16T → cur **13,109P**/1,567F/16T（+45P 精确对应
定向面）：分母 14,692 逐 subtest 不变，changed 45 = 45 F→P + **零 P→F**
+ 零 T 变化，changed 行全部落在 websockets/interfaces（close-invalid 族、
basic-auth/cookies 通道 WS 用例、CloseEvent 派发面零外溢）。websockets
topdir 通道内 Fail 清零。

**批次门**：ws interfaces 批次 133/133 = **100%**（M2 落账 66.2%，九簇
挂账随本批收口）；DC-3 interfaces 批次面门（pass ≥ 实跑分母 × 98.5%）
达成。
