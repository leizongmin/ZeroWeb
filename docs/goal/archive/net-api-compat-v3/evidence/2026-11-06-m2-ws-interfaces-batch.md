# M2（一）：websockets/interfaces 批次导入——纯客户端 WS API 语义面 74 案

日期：2026-11-06。前置：M0 甄别二（websockets 清单——interfaces 为 DC-3 主
增量）、M1b redirect 批次（零回归基线 12,924P）。

## 一、批次内容

1. **语料导入**：`websockets/interfaces` 74 案入通道（73 `.html` + 1
   `.any.js`，pin `315976933870`； CloseEvent 3 + WebSocket/{bufferedAmount,
   close,constants,events,extensions,protocol,readyState,send,url} 71）。
   `NET_API_CORPUS_SUBDIRS` 增 11 条叶子目录（枚举器 flat 一层，嵌套子目录
   显式列）；fetch 脚本 DIRS 同步（并补 M1b 漏记的 `fetch/api/redirect`）。
2. **ws_fixture 两条新端点**（上游 handlers 逐字语义）：
   - `/echo_raw`（3 案，events/015/017/018）——收到的消息**裸字节**写回
     socket（`request.connection.write(line)`，无帧封装）：客户端把
     `\x81\x04test\x83\x03LOL` 作一条 binary 消息发来，回写后客户端解析器
     逐帧读出 text "test" + 保留 opcode 3 帧（协议错误断连面）；
     binary `b'exit'` → 服务端关闭。
   - `/sleep_10_v13`（1 案，close/close-connecting.html）——101 握手响应
     延迟 10s（`do_extra_handshake` sleep），握手后 text echo、`Goodbye`
     服务端关闭。
3. 连接面其余全部命中既有端点：`/`（35）、`/echo`（19）、`/empty-message`
   （8）、`/handshake_sleep_2`（1）——M0 甄别档第三节盘点兑现。

## 二、通道实跑与批次门

`make testharness-net-api FILTER=websockets/interfaces`：**74 案 133 子测试
88P / 45F / 0T**（批次门基准 88/133 = 66.2%，未达 98.5%——残余全为引擎
语义簇，第四节归 M3；本批按 redirect 批次先例记账导入完成 + 簇归因，
面门随 M3 收敛复验）。

亮点：`close-connecting.html`（10s 延迟握手中止时序）**一次通过**——
close() during CONNECTING 的 CLOSING 状态机与 wasClean=false 面成立；
`/echo_raw` 保留 opcode 注入路径正确触达协议错误分支。

## 三、全量 A/B（零回归门）

M1b 全量 log 对照（Python 逐对解析）：base 12,924P/1,602F/16T →
cur **13,011P**/1,647F/16T。**changed 0 / removed 0**，added 132
（88P/45F 去重后——send/010 两同名子测试被 (case,subtest) 集合键合并，
实跑 133）——零回归门达成，批次净增 +87P。

## 四、残余失败簇（45F，全引擎侧，M3 账）

| # | 簇 | 子测试 | 断言面 |
|---|---|---:|---|
| 1 | 接口常量 ReadOnly 形 | 16 | `WebSocket.CONNECTING` 等 4 常量：赋值须忽略（002×4）、delete 须拒绝（003×4）、defineProperty getter 须 throw（005×4）、setter 须 throw（006×4）——WebIDL 常量 {Writable:false} |
| 2 | EventTarget/dispatchEvent 缺失 | 9 | `ws.dispatchEvent is not a function`（003/007/008/009 + 019×4）+ 004 EventTarget 成员面 |
| 3 | 实例属性描述符 | 6 | `bufferedAmount`/`readyState`/`url` 赋值须忽略（各 1）+ delete 须拒绝（各 1）——readonly {Configurable:false} |
| 4 | 事件处理器回调接口形 | 4 | 013：`ws.onX = {handleEvent: fn}` 赋值后 getter 须回读对象（现回 null）且 dispatchEvent 不调 handleEvent |
| 5 | send 状态机/参数序 | 3 | 001：无参 send 须 TypeError（WebIDL 参数解析先于状态检查，现 InvalidStateError）；007/008：CLOSING/CLOSED 期 send 须静默返回 undefined（现 throw） |
| 6 | 协议错误断连面 | 3 | 015：error 事件时 readyState 须已 CLOSED（现 OPEN）；017 同族；018：close 事件须 CloseEvent 形（现裸 Event） |
| 7 | CONNECTING 期 close() fail 语义 | 2 | close-basic：close 事件 wasClean 须 false（fail 路径）；close-connecting-async：须派发 error 事件 |
| 8 | 事件序（listener+onX 并存） | 1 | 016：同类型 addEventListener 与 onX 按注册序全触发（现 onX 臂缺失） |
| 9 | CloseEvent 构造器默认值 | 1 | `new CloseEvent("foo")` code 默认 0（现 1005）、reason 默认 "" |

## 五、后续

- M3：上表九簇与 xhr/fetch redirect 簇合并收敛（簇 1/3/9 是 WebIDL 绑定
  机器共性面——或一处描述符基建多点收益）。
- M2（二）：xhr auth1-11 HTTP Basic、getResponseHeader 响应头控制面、
  CORS preflight 端点扩表。
