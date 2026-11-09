# M1：TLS 端点 fixture + wss 端到端连通（DC-2 前半）

日期：2026-11-08。实施基准 = M0 wss 审计档 §三 D1/D2；核对源码以实编译/实跑为准
（tungstenite 0.29.0 / rustls 0.23.46 / rustls-pki-types 1.15 / rcgen 0.14.10 /
ring 0.17.14 逐一读 vendored 源核对 API 形态后再落码）。

## 一、交付面

| 项 | 落点 | 内容 |
|---|---|---|
| workspace 依赖 | Cargo.toml | `rustls = { 0.23, default-features=false, features=[std,tls12,ring] }` + `rcgen = { 0.14, [crypto,ring] }`——Cargo.lock 锁 rustls 0.23.46/ring 0.17.14，**全图零 aws-lc-rs**（与既有依赖图一致，无新密码学后端） |
| D2 客户端信任旁路 | crates/net/src/websocket.rs | `WSS_TEST_TLS_CONFIG`（OnceLock）+ `install_wss_test_tls_config()`；connect 分支：**钩子设置且 scheme=wss** 才走 `client_tls_with_config(request, stream, None, Some(Connector::Rustls(cfg)))`（拨号/NoDelay/错误映射复刻 `tungstenite::connect` 同款，`dial_wss_tcp` 全地址逐试 + IPv6 括号剥除同源）；其余分支生产路径逐字节不变 |
| 泛化 IO 层 | tests/wpt-runner/src/ws_fixture.rs | `FixtureStream { Plain(TcpStream), Tls(StreamOwned<ServerConnection, TcpStream>) }`（rustls 流不可 `try_clone` → 读写收敛 `BufReader<FixtureStream>` 单句柄）；握手/帧/消息/数据相全部 handler 改签名 `(conn: &mut BufReader<FixtureStream>)`，行为面零变化 |
| D1 TLS 端点 | 同上 `fixture_wss_port()` | 进程级第二条监听（对齐上游 `{{ports[wss][0]}}` 独立监听形态）；每连接 `ServerConnection::new` + `StreamOwned` 包裹后走**同一** `handle_conn`/`KNOWN_PATHS` 表 |
| 证书 | 同上 `generate_fixture_cert()` | rcgen 进程内生成，**仅驻内存不落盘不入库**（划界）；两级 = 自签 fixture CA（CA 位 + keyCertSign）+ CA 签发叶证书（SAN = DNS localhost/wpt.test/\*.wpt.test + IP 127.0.0.1，EKU ServerAuth） |

## 二、实施发现（记档）

1. **单证方案不可行（实测）**：首版单自签证书（自身兼信任锚 + 端实体）被
   rustls/webpki 链校验拒——`CaUsedAsEndEntity`（信任锚须 CA 位，叶位呈现 CA
   证书违规）。改两级 CA+叶后通过。此形态对齐 wptserve 真实部署（CA/服务器
   证书分离），非 workaround。
2. **tungstenite 0.29 无带 connector 的一步式入口**：`connect*` 族固定
   `connector: None`；带 connector 需 `client_tls_with_config(request, stream,
   config, connector)`（自管拨号）。旁路分支据此复刻 `connect_to_some` 拨号面
   （DNS 全地址逐试 + NoDelay + 错误映射 `Failure(f)→f`），差异仅 connector 注入。
3. **测试驱动面偏离（D2 蓝图措辞）**：蓝图写「`default_net_ws_host` 真栈连」；
   落地单测驱动 `zero_net::websocket::WebSocket` 直连（= `default_net_ws_host`
   的委派底层，ws_bridge.rs:657-658 核实）。原因：ws_bridge 事件泵须经
   `Sandbox` trait（12 个必需方法，V8/rquickjs 绑定面）才能注册 pending-next，
   wpt-runner 测试内实现不成比例；泵面与 TLS 无关且由 engine 既有明文 e2e
   （net_ws_host_echo_roundtrip_end_to_end）覆盖。**wss 全链**
   （FixtureWsHost 改写 → default_net_ws_host → 钩子 → TLS fixture）由 M2
   通道接线后实跑覆盖（通道页即全链）。改写路径本身（`rewrite_wpt_authority`）
   已在单测内消费。
4. **预存在环境失败（非本窗）**：`cargo test -p zero-net --lib` 单败
   `client::tests::test_send_invalid_url_returns_network_error`（断言连
   `http://0.0.0.0:1` 必败——WSL2 环境实际连成功）。干净 HEAD stash 复跑同败
   （0.11s 现场确认），与 websocket 改动无关（client.rs/reqwest 路径未触碰）。

## 三、验证

1. **单测 2/2 绿**（守卫包裹 `cargo test -p zero-wpt-runner ws_fixture`）：
   - `wss_end_to_end_echo_roundtrip`：`fixture_wss_port()` TLS 端点 ↔ zero-net
     真栈经 `wss://wpt.test:<port>/echo`（宿主同款改写）→ TLS 握手 + 101 →
     text echo 往返 → 带码 1000 关闭握手全链。
   - `ws_plain_end_to_end_echo_roundtrip`：泛化 IO 层明文面零分叉钉（同真栈
     连明文 `/echo` 往返 + 关闭）。
2. **websockets 通道 A/B 零回归（changed 空集）**：同命令
   `make testharness-net-api FILTER=websockets/`（test-guard 包裹 + systemd-run
   MemoryMax=12G 双限）对 M1 树与干净 HEAD（stash 本窗 diff）各实跑一次：
   - M1 树：381P / 0F / 110 文件；HEAD 基线：381P / 0F / 110 文件；
   - case 清单 diff = 空，subtest 明细 diff = 空——**逐案全等**。
   - 对照 43 号 M0 档「377P/107 文件」的 +4P/+3 案为 43 号归档前增量（M1a/M4
     枚举扩面等落库修复；HEAD 基线复跑同值），非本 goal 面。
3. **clippy**：`cargo clippy -p zero-net -p zero-wpt-runner` 零告警；
   rustfmt 已过。
4. **明文 e2e 与通道实跑对 fixture 泛化 IO 的覆盖互补**：单测钉 echo 族，通道
   381P 钉全端点表（`/simple_handshake` 同包、`/echo_raw` 裸字节、backpressure
   等 26 path 全部经新 IO 层）。

## 四、DC-2 前半落账与 M2 交底

- DC-2 前半（runner TLS 端点 fixture 落地 + wss:// 端到端连通单测）达成。
- **M2 待接线**（wss 审计档 D3-D5）：token 映射 `{{ports[wss][0]}}` →
  `fixture_wss_port()`（testharness.rs:5471）；websockets 子树变体枚举
  （html `?wss`/`?wss&wpt_flags=https` + `.any.js` `// META: variant=` 解析臂）；
  cookies/001-007 页锚 https；`.https.` 案逐案重估；接线后 A/B
  （本档 381P 明细即 M2 前基线）。
- 钩子安全边界复述：产品代码永不调用 `install_wss_test_tls_config`（OnceLock
  未设 = 生产 webpki-roots 路径逐字节不变）；runner 注入的信任根 = 仅 fixture
  自签 CA（比全局信任收紧，旁路不外溢）。
