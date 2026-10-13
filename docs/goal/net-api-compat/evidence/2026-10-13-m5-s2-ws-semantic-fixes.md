# M5-S2 — websockets IDL/构造器/状态机修齐（2026-10-13）

**背景**：M5-S1 基线 **334/375 = 89.1%**（case 78/106）失败聚类中 shim/bridge 归属簇
（IDL 校验 19 子测试 + ws.protocol 4 案 + 状态机 3 案 + constructor/006 stringifier）。
本切片修齐该面；cookie jar 4 案（storage 域）、basic-auth（zero-net）、017（runner
工件）维持跨域记账。shim part06/part03 属 zero-web 流共享面，修齐前碰头核对：
`git log -- part06.js` WS 段最后触碰 = zero-web 流落地提交 caf3b3373，其后无并行
编辑，无冲突。

## 数字

| 口径 | M5-S1 基线 | M5-S2 |
|---|---|---|
| 子测试 | 334/375 = 89.1% | **369/375 = 98.4%**（+35） |
| case | 78/106 = 73.6% | **100/106 = 94.3%**（+22） |
| Timeout | 2 | **0** |

残余 Fail 6 案（全部既有记账归属）：cookies/001、002、007、cross-origin-cookie-set
（WS cookie jar——storage goal 挂账族）、basic-auth（zero-net 401 挑战流）、
constructor/017（runner 预处理工件 `__SCHEME` 执行序，非产品缺口）。

## 修齐内容

**前轮（2026-10-13 凌晨 session，429 中断遗留未提交变更，本轮验证 + 补齐）**：
- shim part06 WebSocket 构造器：无参 TypeError、http→ws / https→wss scheme 转换
  （url crate 禁 special↔non-special 换 scheme，经 href 串锚定替换重解析）、options-bag
  `{protocols}` 展开、重复协议 ASCII case-insensitive 拒绝、`binaryType` accessor
  （非法值静默忽略且值不变）、on* accessor（LegacyTreatNonCallableAsNull——非函数落
  null）、send() 非 BufferSource/Blob 落 DOMString 分支（null → "null"）、close(null)
  → 0 → 校验拒绝。
- `crates/net/src/websocket.rs`：`Sec-WebSocket-Protocol` 请求头发送 + 101 响应
  `selected_protocol()` 透出；close() → Closing 态（对端回关后方 Closed，Closing 可读）。
- `crates/engine/src/ws_bridge.rs`：per-host 连接建立串行（constructor/014 serialize
  establish）；close 握手等待段（回关码/原因优先落事件，EOF → 1006 unclean，
  12s 上限按本地码 clean 收尾——Close-delayed）；echo 测试 server close 帧回显同码
  （空回关把 1000 折叠成 1005）。
- `tests/wpt-runner/src/ws_fixture.rs`：`/delayed-passive-close`（sleep 1s 回显码/原因）、
  `/handshake_sleep_2` 服务端关闭、`/echo_exit`/`/echo_close_data`/`/echo`/`/` 分派显式化。

**本轮补齐四项**：
1. **close 码域 → InvalidAccessError**（现行 WHATWG spec step 1——旧版 SyntaxError 已
   改）：close-invalid 6 子测试（0/500/NaN/String/null/2¹⁶+1000）+ Close-1005/2999/
   onlyReason 共 9 子测试翻绿；reason > 123 字节仍 SyntaxError（step 2.2 未变）。
2. **构造器 fragment 门**（spec step 4）：URL fragment 非 null → SyntaxError
   （Create-invalid-urls「http://…/#」「http://…/#test」「#test」3 子测试）。
3. **协议 token 文法**：`0x21–0x7E 无逗号` 收紧为 RFC 7230 token 字符集
   （排除 separators `()<>@,;:\"/[]?={}` SP HT）——Create-asciiSep「/echo」SYNTAX_ERR。
   corpus 在用协议串（"echo"/"ec ho"/"echo"/["echo","eCho"]）复核无交集回归。
4. **A/AREA stringifier→href 三层落点**：前轮在元素 get-trap 锚点分支的加挂经证为
   **死代码**（R34xx toString 分支更早吞 'toString'；而 `String(a)` 实际先命中更早的
   `Symbol.toPrimitive` 分支——探针页实证 `a.toString` 已返正确 href 但 `String(a)`
   仍 '[object HTMLAnchorElement]'，且 `a[Symbol.toPrimitive]` 经 trap 返标签串化函数
   ——ToPrimitive 优先序）。修齐：trap `Symbol.toPrimitive` 分支 + trap `toString`
   分支（R34xx）A/AREA 特例返 href 串化函数 + `HTMLAnchorElement/
   HTMLAreaElement.prototype.toString` 直装（createElement 产物 polyfill Proxy 经
   getPrototypeOf 链可达，own href accessor R130 供值）。constructor/006 翻绿。

## 零回归核查（六 corpus 逐域隔离跑）

| corpus | M4-S29 收口 | 本轮 | 判定 |
|---|---|---|---|
| fetch | 1491/2081 | 1499P/589F/3T（分母 2091） | ✓ 分母漂移内持平 |
| xhr | 557/584 | 562P/30F/3T（分母 595） | ✓（3T = 收口已记「timeout 族 3」） |
| url | 4422/6058 | 4429P/1636F（分母 6065） | ✓ |
| mimesniff | 100%（后经 fileapi 仲裁 865 腿转 Fail） | 1033P/865F | ✓ 与 2026-10-13 漂移注记逐值一致 |
| streams | 905/1019 | 905P/108F/5T | ✓ Pass 数逐值一致 |
| eventsource | 35/35 | 33P/0F/1T | ⚠ Timeout 1（见下） |

**eventsource request-cache-control Timeout 归因（stash 对照实证）**：五文件变更
stash 后于 commit 40fd20c5d 基线复跑 eventsource——33P + 同案 Timeout 逐值复现 →
**先在缺陷，非本轮引入**（收口 2026-10-01 后至 M5-S1 期间漂移，或环境时序敏感），
M5-S2 修齐面（WS + A/AREA）与 EventSource 无交集。记账待查，不计回归。

## 迭代记录

1. **constructor/006 三层落点定位**：前轮死代码（锚点分支 'toString' 加挂，被 R34xx
   分支遮蔽）→ 移植 R34xx toString 分支特例 + 接口原型直装，两轮跑仍标签串 → 临时
   探针页（websockets/zz-probe.html，跑后删除；assert 消息回读内部状态）实证
   `a.toString` 已对但 `String(a)` 仍标签串 → 二轮探针定位 `a[Symbol.toPrimitive]`
   经 trap 返标签串化函数（trap 分支序：@@toPrimitive 先于 toString）→ 三处对齐后
   翻绿。
2. **二进制内容核验方法**：排障期间一度误判「part03.js 改后 engine 未重编」（复验
   ws3 构建日志实已重编——真根因见迭代 1 的 @@toPrimitive 吞点）。核验 shim 改动是否
   进二进制时 `strings` 对**含中文注释串会断行**（UTF-8 非打印字节中断匹配）——须用
   代码内 ASCII 标识符（如 `_eWsaStr`）检索，勿用注释串。
3. **前轮 streams 探针残留清理**：`streams/readable-streams/zz-probe.any.js`、
   `streams/piping/zz-probe-fc.any.js`（前轮调试遗留，2 Fail 污染 corpus 计数）删除。
4. **一次性 url crate panic 未复现**：全 corpus 单进程跑（M5-S2 中间态二进制）曾在
   url-2.5.8 lib.rs:2881 `slice_of` panic（`byte range starts at 6 but ends at 4`），
   终止整个 runner；逐域隔离复跑 7 域（含 url 全量 + fetch 全部子域）均未复现。
   记观察项：url corpus hostile 面 panic 防护（M3 已 catch_unwind `serialize_url`）
   或有未覆盖调用点；单进程跨域长跑时域间状态面亦未排除。复现优先级低于 M5-S3 候选。

## 门禁

- `cargo fmt --all` 应用后 `--check` 干净；`cargo clippy -p zero-engine -p zero-net
  -p zero-wpt-runner --all-targets -- -D warnings` 零告警。
- `make test` 全量（test-guard 包裹）：**20643P / 0F**（71 套件全绿）。首跑 2 失败
  均为存量桥测断言旧语义——随 spec 翻新后定向复绿（ws_host 4P + websocket 6P）再
  全量收口：① `test_t8k_websocket_constructor_validation`——`new WebSocket('http://…')`
  按现行 spec step 2–3 转换放行（断言期望更新 + 增 fragment 门用例）+ close 码域
  InvalidAccessError；② `net_ws_host_echo_roundtrip_end_to_end`——mock 握手回显客户端
  提供的子协议（RFC 6455 §4.2.2——tungstenite 客户端对「client 提供而 server 未选定」
  按规范拒绝连接，open wire 断言携 `chat`）。
- 产品面（engine shim/net/bridge）变更 → 本轮执行 workspace 全量门禁（区别 M5-S1
  纯资产切片）；存量桥测翻新属 spec 语义跟进（照 M4 各轮先例），非产品缺口。

## 下一步（M5-S3 候选）

1. WS cookie jar 集成 4 案（与 storage goal 挂账族并案评估——Set-Cookie 落
   document.cookie + 握手 Cookie 携带）
2. basic-auth 401 挑战流（zero-net，跨域记账）
3. eventsource request-cache-control Timeout 归因（先在漂移，见上）
4. url crate panic 防护调用点审计（低优先，观察项）
