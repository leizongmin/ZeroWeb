# M1 / DC-3 — WebSocket 挂账重估结论（2026-10-29）

**重估输入**：
- 一期归档控制面 [master.md](../../archive/net-api-compat/master.md) P6 行 +
  M5-S1~S4 四切片证据（2026-10-13~14）
- 收口判定 [2026-10-14-m5-s4-ws-complete.md](../../archive/net-api-compat/evidence/2026-10-14-m5-s4-ws-complete.md)
- 二期 coverage 对账基线（coverage-2026-10-27）：websockets 8/799 文件覆盖、
  102/2,389 子测试（该口径为全量并集分母 + 当时工作区实跑分子，与本 goal
  通道口径不同——见下「口径对齐」）

## 一期收口事实（重估基点）

一期 **P6 WebSocket 二期切片已在 2026-10-14 收口**，与二期立项文档所述
「WebSocket 二期挂账」不同——挂账在归档前已被 M5-S1~S4 清偿：

| 切片 | 内容 | 结果 |
|---|---|---|
| M5-S1 | runner 内嵌 RFC6455 fixture 端点（21 端点，语义基准 = 上游 `handlers/*.py`）+ FixtureWsHost 改写 + `.sub.js` 三通道 | 基线 334/375 = 89.1% |
| M5-S2 | close 码域/构造器 fragment 门/协议文法/`ws.protocol`/连接串行 | 98.4% |
| M5-S3 | WS cookie jar 双 jar 断缝修齐 + eventsource preflight fixture | 99.5% |
| M5-S4 | classic 顶层 `let x;` 声明导出扫描（产品缺口翻案）+ basic-auth userinfo 预置 | **376/376 = 100% / case 106/106** |

二期基线复测（见同日 M1 归因 evidence「websockets」节）：376/377，与一期收口
376/376 逐案核对，**1 条跨流转入真回归**——`websockets/constructor/016.html`
（non-ASCII query windows-1252 echo，onclose assert_unreached）；该案不改变
挂账结论，归二期收敛轮清偿（归因档「真回归」节）。

## 重估结论

**维持挂账（不可行切片维持原状），无新增可实施切片**——一期收口后剩余面全部为
结构性/环境性不测面，逐项如下：

| 不测面 | 性质 | 重入条件 |
|---|---|---|
| `wss://` / h2 变体 | runner 无 TLS 终结通道（真栈 TLS 面） | runner TLS fixture 通道立项 |
| `.https.*` 5 案 | 同上（secure context 前提 + TLS） | 同上 |
| multi-globals / unload-a-document / bfcache 结构面 | runner 无多文档/页生命周期编排管道 | runner 多文档管道落地（与 worker/iframe 筛减同因） |
| `websockets/handshake`、`create-*` 服务端行为面 | 上游 wptserve websocket handler 服务端语义，fixture 只承接语义端点 | 按上游 handler 逐文件甄别后可扩端点（一期已按甄别结果记账 21 端点） |
| userinfo 百分号编码凭据解码 | net 层 conservative 路径（语料无此形态） | 语料刷新出现该形态 |

**口径对齐注记**：coverage-2026-10-27 的 websockets 行（102/2,389）分母为
wpt.fyi 全量并集（799 文件），分子来自对账跑当时工作区的 websockets 通道
实跑——该跑非本 goal 通道全量（本通道 case 分母 106/376 子测试），不可与一期
收口数字直接对读。二期 M 收口 coverage 复测以本通道全量重跑为准。

## 对二期 Mission 的影响

Mission 3「WebSocket 二期重估：可行则本轮内立项推进」——**重估结论为不可行**
（残面全为 runner 结构面，非引擎语义），不立项；DC-3 以本档落定。收口后唯一
增量是 constructor/016.html 1 条跨流回归（清偿归 DC-2）。二期收敛
火力集中在 url / mimesniff / fetch/api 三族失败簇（DC-1/DC-2）。
