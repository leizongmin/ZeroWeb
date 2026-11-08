# runner-net-infra 控制面（46 号）

## 状态头（2026-11-08）

- **当前状态**：M1 完成（DC-2 前半 ✅ 2026-11-08）——TLS 端点 fixture +
  zero-net 信任旁路钩子 + wss 端到端连通单测 2/2 绿；websockets 通道 A/B
  零回归 changed 空集（381P/0F 双跑全等）。
- **卡点**：无。
- **下一步**：M2 scheme/端点寻址接线（D3-D5：token 映射 + websockets 子树
  变体枚举 + cookies 页锚 https + `.https.` 案重估）；接线后 A/B 基线 =
  [M1 档](evidence/2026-11-08-m1-tls-endpoint.md) 381P 明细。

## 里程碑状态表

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M0 双审计 + 数值门（DC-1） | ✅ 2026-11-08 | [wss 审计档](evidence/2026-11-08-m0-wss-audit.md) + [py 普查档](evidence/2026-11-08-m0-py-endpoint-census.md) |
| M1 TLS 端点 fixture + wss 连通（DC-2 前半） | ✅ 2026-11-08 | [M1 档](evidence/2026-11-08-m1-tls-endpoint.md) |
| M2 scheme/端点寻址接线（DC-2 收口） | 未启动 | 蓝图 wss 审计档 D3-D5 |
| M3 py 路由框架 + 先例迁移 + 补丁收编（DC-3） | 未启动 | 蓝图 py 普查档 §三 |
| M4 下游消费验收 + 零回归收口（DC-4） | 未启动 | 门型见两审计档第四节 |

## 数值门（DC-4，2026-11-08 M0 落定）

- **wss 池**：102 wss 变体案（41 html + 61 any.js）covered > 0，批次门 =
  实跑分母 × 98.5%；h2 变体 92 实例维持排除；mixed-content.https
  （引擎 mixed-content 缺口并记）与 third-party-cookie-accepted（语料
  helper 缺口并记）不入门、如实落账。
- **py 池**：P-A 三目录池（den 606）+ P-B 判据族框架承接面落地（注册面 +
  单测 ≥1 例/族，下游控制面互指可消费）；迁移案逐字节等价 100% 硬门
  （A/B changed 空集，wss 面显式声明除外）。

## 零回归复验面（DC-2/DC-3/DC-4 共用锚）

1. websockets 通道既有 377P 面（43 号 M0 基线）；
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
