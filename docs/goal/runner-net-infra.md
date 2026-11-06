# runner 网络 fixture 基建 — TLS/wss 端点 + wptserve py 端点 shim 框架化

**版本**: v1.0
**日期**: 2026-11-04（立项，用户拍板——websockets wss 面前置 + py 端点基建打包）
**状态**: Active——立项，M0 未启动
**执行模式**: runner 基建专项（fixture 管道）+ 下游 goal 消费验收
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——测试基建面）

## 基线事实（2026-11-04 核实）

- **WS fixture 已有雏形**（net-api 一期 M5-S1）：`ws_host_factory` 接线只挂
  websockets 用例（通道级隔离），内嵌 fixture 端点 + 宿主改写委派
  `default_net_ws_host` 真栈（ws_fixture.rs）；但**无 TLS**——`?wss` 变体
  两臂均不可测（testharness.rs M5-S1 注记「P3 挂账另案（两前置审计）」，
  v2 重估维持挂账）。
- **py 端点 shim 已有先例、未框架化**：v2 逐案补 fixture（echo-content、
  inspect-headers.py POST 豁免、`/echo-query_v13` 路由表）；S12 的
  cors-preflight-redirect 对齐是**本地补丁** + 语料仓 update 脚本缺口
  记账——缺统一路由框架。
- **下游被堵的面**：43 号 websockets wss 面（fetch 通道 scheme 双态 +
  TLS 端点）；43/39 号 fetch/http-cache/redirect/reflector 等 `.py` 密集
  目录；47 号 http-cache corpus（stash/cache 端点族）。
- 上游 wptserve 的 py 端点是静态语料导入最大单点阻塞（语料里 `.py` 文件
  runner 只能按案 shim，无通用机制）。

## Mission

1. **M0 双审计**（v2 挂账的两前置审计在此落地）：wss 端点形态盘点
   （上游 websockets 语料的 wss 依赖面分类）+ 通道 scheme 翻转设计
   （页锚 http/https 双态之外的 ws/wss 端点寻址）；py 端点依赖普查
   （下游四个 goal 语料中 `.py` 引用分类：可静态 shim / 需动态端点 /
   worker 面排除）。
2. **TLS/wss 端点 fixture**（M1/M2）：runner 内嵌 TLS 端点（自签证书 +
   runner 侧信任旁路，不出仓不涉真域名）；wss:// 寻址接入
   ws_host_factory 真栈。
3. **py 端点 shim 框架化**（M2/M3）：路由表机制从 v2 逐案先例升级为
   框架（端点注册 + 请求状态 stash 语义 + reflector 族模板）；
   cors-preflight-redirect 本地补丁收编进框架，语料仓 update 脚本缺口
   清偿。
4. **下游消费验收**（M3/M4）：43 号 websockets wss 面解锁、39/47 号
   py 目录解锁的接线验证；既有 fixture 面（S3b `/echo-query_v13` 等）
   零回归。

## Done Criteria

- **DC-1（M0 双审计）**：wss 依赖面分类 + scheme 翻转设计档 + py 端点
  普查（逐下游 goal 的可 shim/需动态清单）落
  `docs/goal/runner-net-infra/evidence/`；DC-4 数值门落定并回填。
- **DC-2（TLS/wss）**：runner TLS 端点 fixture 落地（wss:// 端到端连通
  单测）；通道 scheme/端点寻址按 M0 设计接线，既有 http 通道零回归。
- **DC-3（py 框架）**：路由表框架落地且 v2 先例案全部迁入框架（行为
  逐字节等价，A/B 复验）；S12 本地补丁收编 + update 脚本缺口清偿。
- **DC-4（消费验收）**：43 号 websockets wss 面入账（covered > 0）；
  下游 py 目录解锁数达 M0 数值门；四既有 fixture 面 A/B 零回归；
  头条不回落。

**DC-4 数值门决策规则（立项即定）**：解锁子测试数按 M0 普查清单实数
落门（wss 面 + py 目录两池分列）；框架迁移案逐字节等价为硬门，不设
通过率门。

## 划界

- **不碰引擎网络栈语义**：default_net_ws_host/fetch_bridge 真栈行为
  不改；本 goal 只做 fixture 管道与寻址，TLS 语义按「runner 测试环境
  旁路」处理，不涉产品 TLS 策略。
- **自签证书不出仓**：证书生成进 runner 测试流程（构建期或首次运行期
  生成），不入库不涉真 CA；安全边界审查随提交门禁。
- **不代下游做语义收敛**：43 号 websockets 语义簇、47 号缓存语义仍在
  各自 goal；本 goal 只解锁通道。
- **worker 面排除照旧**：普查中 worker 执行面依赖的 py 端点如实标记，
  不解锁。

## 里程碑

- **M0**：双审计 + 数值门（DC-1）。
- **M1**：TLS 端点 fixture + wss 连通（DC-2 前半）。
- **M2**：scheme/端点寻址接线（DC-2 收口）。
- **M3**：py 路由框架 + v2 先例迁移 + 补丁收编（DC-3）。
- **M4**：下游消费验收 + 零回归收口（DC-4）。

## 依赖约束（run-rules §9）

- 与 43 号（net-api-compat-v3，已收口归档
  [archive/net-api-compat-v3.md](archive/net-api-compat-v3.md)）共享
  websockets 通道——fixture 接线改动同窗复验其非 wss 面基线零回归；
  43 号 wss 面入账以本 goal DC-2 为前置（两 goal 控制面互指）。
- ws_host_factory 通道级隔离不变（仅 websockets 用例挂接，CSP/cookies
  通道零耦合）。
- 无人值守 fixture 构建/运行走 test-guard 与 systemd-run 内存墙钟双限
  （质量门禁）；自签证书生成脚本过提交门禁审查。
