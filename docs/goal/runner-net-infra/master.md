# runner-net-infra 控制面（46 号）

## 状态头（2026-11-08）

- **当前状态**：M3 进行中——基线落账 + 组成归因完成（基线 =
  **13,582P/1,337F/8T（14,927 subtest，含 wss 360P/101 键）**，
  [归因档](evidence/2026-11-08-m3-baseline-reattribution.md)）；
  切片一完成（py 路由底座模块迁出，testharness.rs 19,236→15,630 行，
  [切片一档](evidence/2026-11-08-m3-slice1-route-module-extract.md)）；
  **切片二+三落地**：`PyRoute` 保序注册表（Exact/Exacts/DirSuffix/
  DirSuffixes 谓词 + None=落穿语义），链头 23 臂迁入（站点根→xhr
  redirect-cors 段）。切片二 12 臂态 = 54c5f4cea（A/B multiset 空集）；
  切片三 11 臂初版 7 条 DirSuffix 谓词带列表字面量缺陷（永不匹配→
  落穿，机械验证发现已修），**修复后 23 臂态全量 A/B multiset 空集 +
  七域全等已落证**（m3s3-ab-181241 主证 + 181630 冗余印证），谓词
  修复随 6be021b87 落库，证据与事件见
  [切片二+三档](evidence/2026-11-08-m3-slice2-registry-head12.md)。
- **卡点**：无。
- **下一步**：切片四 fetch/api 族 + eventsource 族臂迁入（前缀推进，
  残链头现于 fetch/api/resources/redirect.py），每批全量 A/B 对照
  基线 multiset 空集；其后 inspect-headers/方法豁免（Custom 谓词）→
  尾段收口（84 臂全量入表）；四 fetcher 闭包臂随后同框架分表迁移；
  S12 补丁收编 + update 脚本缺口清偿随批次落。

## 里程碑状态表

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M0 双审计 + 数值门（DC-1） | ✅ 2026-11-08 | [wss 审计档](evidence/2026-11-08-m0-wss-audit.md) + [py 普查档](evidence/2026-11-08-m0-py-endpoint-census.md) |
| M1 TLS 端点 fixture + wss 连通（DC-2 前半） | ✅ 2026-11-08 | [M1 档](evidence/2026-11-08-m1-tls-endpoint.md) |
| M2 scheme/端点寻址接线（DC-2 收口） | ✅ 2026-11-08 | [M2 档](evidence/2026-11-08-m2-wss-wiring.md) |
| M3 py 路由框架 + 先例迁移 + 补丁收编（DC-3） | 进行中（切片一 ✅ 切片二+三 ✅） | 蓝图 py 普查档 §三 + [切片一档](evidence/2026-11-08-m3-slice1-route-module-extract.md) + [切片二+三档](evidence/2026-11-08-m3-slice2-registry-head12.md)（23 臂态 A/B 双跑空集，含谓词修复） |
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
2. net-api 全通道基线 13,582P/1,337F/8T（2026-11-08 落账 +
   [归因档](evidence/2026-11-08-m3-baseline-reattribution.md)，
   supersedes 43 号 M4 终账 13,275P 口径）；
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
- 2026-11-08 18:10 碰撞记录（同 session 多重 spawn 爆发）：43d2b169 同
  session ≥9 个并发 resume 实例同树同 goal——切片二由其中一轮认领提交
  （54c5f4cea，内容与 A/B 验证树逐字节一致，按认领件采纳）；随后一实例
  已起切片三源码编辑（testharness_py_routes.rs M 态）。**仲裁口径**：
  同文件单编辑方（迁移面 .rs 归当前在编实例，他轮勿碰）；同二进制
  A/B 跑窗口互斥（并发双跑失败面不作账）；产物按先完成者采纳、互补
  者按件核对收编；各轮动手前 pull --rebase 重读本节。
- 2026-11-08 18:14 观察注→18:26 对账→18:31 git 定谳：7 处
  `DirSuffix { suffix: "['…']" }` list-repr 缺陷真实存在过，但**只在
  工作树切片三中间态（18:05–18:11 生成），未进 54c5f4cea**——
  git show 证其为纯 12 臂态（零 DirSuffix/Exacts），18:26 注误指该
  提交；在树实例 18:12 sed 修复 + 机械等价验证 11 臂，修复随
  6be021b87 入库；23 臂态全量 A/B 双跑（m3s3-ab-181241 主证 +
  181630 冗余印证，两窗有重叠段但双双 multiset 空集）落证零回归。
  教训两条：观察注要留 mtime 锚点；接手方先核 git 账再下结论——
  「生成器自修」误归因源于只看终态没查修复事件，「缺陷在 54c5f4cea」
  误指源于未跑 git show 即落笔。
