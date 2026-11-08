# runner 网络 fixture 基建 — TLS/wss 端点 + wptserve py 端点 shim 框架化

**版本**: v1.3
**日期**: 2026-11-04（立项，用户拍板——websockets wss 面前置 + py 端点基建打包）；
2026-11-08 M0 双审计完成（DC-1 ✅）；2026-11-08 M1 完成（DC-2 前半 ✅，
[M1 档](runner-net-infra/evidence/2026-11-08-m1-tls-endpoint.md)）；
2026-11-08 M2 完成（**DC-2 ✅**——D3-D5 寻址接线 + websockets 通道 A/B
零回归 + wss 池 101 案 360 subtest 全绿过批次门，
[M2 档](runner-net-infra/evidence/2026-11-08-m2-wss-wiring.md)）
**状态**: Active——M0 ✅ M1 ✅ M2 ✅，M3（py 路由框架）**进行中**——
PyRoute 注册表全量落地（切片五，残链 21 臂迁入后 dispatch 残链清空，
表 85 行 = 原链 84 臂 + s346 合流行），余 = fetcher 闭包三分表迁移
（DC-3 收口）
**执行模式**: runner 基建专项（fixture 管道）+ 下游 goal 消费验收
**控制面**: [master.md](runner-net-infra/master.md)
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

**DC-4 数值门落定（2026-11-08 M0 双审计）**：

- **wss 池**：102 wss 变体案（41 `.html` + 61 `.any.js`，`// META: variant=`
  面经普查显影）covered > 0，批次门 = 实跑子测试分母 × 98.5%；h2 变体
  92 实例维持 `.h2.` 排除口径；mixed-content.https.any.js（引擎
  mixed-content 缺口并记）与 cookies/third-party-cookie-accepted.https.html
  （语料 helper 缺口并记）不入门、缺口如实落账。审计见
  [wss 审计档](runner-net-infra/evidence/2026-11-08-m0-wss-audit.md)。
- **py 池**：解锁 = 普查 P-A 三目录池（fetch/http-cache 19 案 +
  stale-while-revalidate 7 案 + fetch/origin 1 案，den 606）+ P-B 判据
  端点族（referrer/preload）框架承接面落地（注册面 + 单测各 ≥1 例，
  47 号/39 号控制面互指可消费）；下游实导入数归各 goal 账。普查见
  [py 普查档](runner-net-infra/evidence/2026-11-08-m0-py-endpoint-census.md)。
- **等价硬门**：迁移案（既有 153 臂）逐字节等价 100%——A/B changed 空集
  （wss 池新增案除外）。

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

- **M0 ✅**（2026-11-08）：双审计 + 数值门（DC-1）——wss 依赖面 102 变体
  案实测（41 html + 61 any.js META 注释面）+ scheme 翻转设计（D1-D6）；
  py 端点 153 臂现状普查 + 逐下游分类（47 号三池/39 号判据族/策略面排除/
  worker 面照旧）。证据落
  [evidence/](runner-net-infra/evidence/2026-11-08-m0-wss-audit.md)。
- **M1 ✅**（2026-11-08）：TLS 端点 fixture + wss 连通（DC-2 前半）——
  workspace 增 rustls 0.23（ring，零 aws-lc-rs）/rcgen 0.14；zero-net 进程级
  wss 信任旁路钩子（未设 = 生产路径逐字节不变）；ws_fixture 泛化 IO 层 +
  `fixture_wss_port()` TLS 监听（自签 CA+叶两级证书，仅驻内存）；wss/明文
  端到端单测 2/2 绿；websockets 通道 A/B 零回归（381P/0F 双跑全等，
  changed 空集）。实施发现与 M2 交底见
  [M1 档](runner-net-infra/evidence/2026-11-08-m1-tls-endpoint.md)。
- **M2 ✅**（2026-11-08）：scheme/端点寻址接线（DC-2 收口）——D3 `{{ports[wss][0]}}`
  → TLS fixture 端点；D4 websockets 子树追加式变体枚举（html + any.js META
  解析臂，net-api 通道独开）；D5 `wpt_flags=https` 变体页锚 https；通道 A/B
  零回归（110 案 381 行全等）+ wss 池 101 案 360 subtest 全绿（100% 过
  98.5% 批次门）。池记账与实施发现见
  [M2 档](runner-net-infra/evidence/2026-11-08-m2-wss-wiring.md)。
- **M3 进行中**（2026-11-08）：切片一 = fetch handler 闭包体整体迁出
  testharness_py_routes 模块（19,236→15,630 行，A/B multiset 空集）；切片二 =
  `PyRoute` 保序注册表落地（PyRouteMatch::Exact + handler None=落穿 + 表序
  = 臂序），链头 12 条全等臂迁入（A/B multiset 空集 +
  [切片二档](runner-net-infra/evidence/2026-11-08-m3-slice2-registry-head12.md)）；
  切片三+四 = 51 臂按族批迁（谓词面扩至六变体，S12 redirect.py 收编随
  切片四落，A/B multiset 空集 +
  [切片四档](runner-net-infra/evidence/2026-11-08-m3-slice4-fetch-es-40arms.md)）；
  **切片五 = 残链清空**（末 21 臂迁入，405 门/OPTIONS blanket 以
  Custom 行表内原位承载，dispatch 收口为「注册表循环 + 静态回退尾」；
  基线锚对合流后 HEAD 重录，全量 A/B 计数敏感 multiset 空集 +
  [切片五档](runner-net-infra/evidence/2026-11-08-m3-slice5-residual-clear.md)）；
  余 = 四 fetcher 闭包三分表迁移（DC-3 收口）。
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
