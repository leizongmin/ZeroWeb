# runner-net-infra 控制面（46 号）

## 状态头（2026-11-08）

- **当前状态**：M2 完成（**DC-2 ✅** 2026-11-08）——D3-D5 接线 + websockets
  通道 A/B：既有 110 案 381 行全等零回归，wss 池 +101 案 360 subtest 全绿
  （100% 过 98.5% 批次门，covered > 0）。
- **卡点**：无。
- **下一步**：M3 py 路由框架（py 普查档 §三：注册表 + EndpointCtx stash）。
  迁移前基线已起跑（2026-11-08）：net-api 全通道 13,275P 面经 systemd 单元
  `m3-baseline-164121`（test-guard 4G/8G/14400s + MemoryMax=12G，日志
  `/tmp/m3-baseline-164121.log`，树 = 411e862bb 干净态）。**基线完成前
  runner 源码冻结**（provenance 纪律）；落账后启 153 臂注册表迁移（逐字节
  等价 A/B）+ S12 cors-preflight-redirect 补丁收编。基线二进制出处 =
  411e862bb；推送时 rebase 拾入 3f2d80d27（跨流机械 rustfmt，编译完成后
  才进树，无行为面），A/B 判读按 411e862bb 口径。

## 里程碑状态表

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M0 双审计 + 数值门（DC-1） | ✅ 2026-11-08 | [wss 审计档](evidence/2026-11-08-m0-wss-audit.md) + [py 普查档](evidence/2026-11-08-m0-py-endpoint-census.md) |
| M1 TLS 端点 fixture + wss 连通（DC-2 前半） | ✅ 2026-11-08 | [M1 档](evidence/2026-11-08-m1-tls-endpoint.md) |
| M2 scheme/端点寻址接线（DC-2 收口） | ✅ 2026-11-08 | [M2 档](evidence/2026-11-08-m2-wss-wiring.md) |
| M3 py 路由框架 + 先例迁移 + 补丁收编（DC-3） | 未启动 | 蓝图 py 普查档 §三 |
| M4 下游消费验收 + 零回归收口（DC-4） | 未启动 | 门型见两审计档第四节 |

## 数值门（DC-4，2026-11-08 M0 落定）

- **wss 池**：102 wss 变体案（41 html + 61 any.js）covered > 0，批次门 =
  实跑分母 × 98.5%（M2 实跑池 101 键，普查 102 的组成修正与 cookies/003-005
  预排除归因见 [M2 档](evidence/2026-11-08-m2-wss-wiring.md) §二.1/§二.2）；
  h2 变体 92 实例维持排除；mixed-content.https
  （引擎 mixed-content 缺口并记）与 third-party-cookie-accepted（语料
  helper 缺口并记）不入门、如实落账。
- **py 池**：P-A 三目录池（den 606）+ P-B 判据族框架承接面落地（注册面 +
  单测 ≥1 例/族，下游控制面互指可消费）；迁移案逐字节等价 100% 硬门
  （A/B changed 空集，wss 面显式声明除外）。

## 零回归复验面（DC-2/DC-3/DC-4 共用锚）

1. websockets 通道既有面（M2 A/B 锚：110 案 381 行，[M2 档](evidence/2026-11-08-m2-wss-wiring.md)）；
2. net-api 全通道 13,275P 面（43 号 M4 终账）；
3. S3b `/echo-query_v13`（constructor/016）；
4. S12 cors-preflight-redirect 面；
5. inspect-headers.py POST 豁免面。
迁移/接线前后逐 subtest A/B 对照，changed 集为空（wss 池新增案除外）。

## 碰撞管理（run-rules §9）

- 与 43 号（已归档）：websockets 通道共享——fixture 接线改动同窗复验其非
  wss 面零回归；43 号 wss 面入账以本 goal DC-2 为前置（控制面互指：入口
  文档「依赖约束」节 ↔ 归档入口「划界」节）。
- 与 47 号（http-cache）：py 框架交付前置（其 M0/语料拉取对齐本 goal M3）；
  缓存语义归 47 号，本 goal 只解锁端点管道。
- ws_host_factory 通道级隔离不变（仅 websockets 用例挂接）——CSP/cookies
  通道 WebSocket 用例零耦合。
- 渲染流共享面（engine/Cargo.lock 等）改动即碰头信号：本 goal 预计触碰
  Cargo.toml workspace 依赖（rustls/rcgen）——与并行流撞车时暂停一边记本节。
- 2026-11-08 碰头记录：并行流 3f2d80d27 对 tests/wpt-runner 五文件做机械
  rustfmt——与 M3 迁移面（testharness.rs）重叠。迁移开工前先 pull --rebase
  取齐，迁移中发现的格式冲突以重排为准不改语义。
