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
  **切片四完成**（2026-11-08 晚）：残链清障 + 40 臂合并批迁入
  （fetch/api 头段 + xhr 中段 + eventsource 主块 = 普查档四+五+六a
  前段），谓词面扩至六变体（+Suffix/Custom），A/B multiset 空集 +
  七域全等落证（[切片四档](evidence/2026-11-08-m3-slice4-fetch-es-40arms.md)）；
  同窗 main FF 六提交取齐，s346 module-fetch-tester.py 行合流表尾
  第 64 行（注册表 = 原链 63 行序 + 合流 1 行），trickle/http-cache
  M2 两上游语义面与本批不相交（合流判定与复核链见切片四档 §五），
  残链 19 臂、链头 = xhr/resources/get-set-cookie.py。
  **切片五完成**（2026-11-08 深夜）：残链 21 臂全量迁入（普查概数 19 与
  实提 21 之差 = 405 门 + OPTIONS blanket 两臂口径归属），method 面
  谓词首入表（Custom ×2，405 门/OPTIONS blanket 表内原位承载——移表尾
  会被 method 无关行截答），dispatch 收口为「注册表循环 + 静态回退尾」，
  表 85 行 = 原链 84 臂 + s346 合流行；基线锚对合流后 HEAD 重录
  （13,614P/1,305F/8T，vs 16:41 锚 32 子测试 F→P 漂移全部归因
  c6dcbce0a http-cache M2 wire），A/B 升级计数敏感 multiset 空集落证；
  S12 update 脚本缺口核对销账（pin 树原生含 succeeded 腿，条件成立）。
  [切片五档](evidence/2026-11-08-m3-slice5-residual-clear.md)。
  **文件族拆分随后落库**：testharness_py_routes.rs（4,822 行）→ 目录模块
  底座 mod.rs（809 行）+ site/xhr/fetch/es/misc 五族（89 项全迁出，每
  文件 ≤2000 行），表文本 vs 迁移提交逐字节全等，单测 298P/0F +
  全量 A/B 第三跑 multiset 空集（切片五档 §六）；与上游 7f1c9f717
  lint 合流后 clippy --fix 收敛归零（d19247694，碰撞管理 22:47 条）。
- **卡点**：无。
- **承接挂账**：真实 trickle 分块延迟（`?count=N&ms=M`）+ fetch
  keepalive 配额池未建模（d8d563e14 `FIXME(runner-net-infra)` 挂入，
  现账面 3P/3F 持平）；47 号 M2 族门 102/102 已过、corpus 门挂本
  goal py 框架交付（M4 承接）。
- **下一步**：fetcher 闭包三分表迁移（清单档拓扑：script/sw 两 fetcher →
  独立表实例 py_script_fetch_routes()/py_sw_fetch_routes()，stash Arc
  共享不可断）→ DC-3 收口；M4 验收含 43 号 wss 面入账 + 47 号 corpus
  门承接。

## 里程碑状态表

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M0 双审计 + 数值门（DC-1） | ✅ 2026-11-08 | [wss 审计档](evidence/2026-11-08-m0-wss-audit.md) + [py 普查档](evidence/2026-11-08-m0-py-endpoint-census.md) |
| M1 TLS 端点 fixture + wss 连通（DC-2 前半） | ✅ 2026-11-08 | [M1 档](evidence/2026-11-08-m1-tls-endpoint.md) |
| M2 scheme/端点寻址接线（DC-2 收口） | ✅ 2026-11-08 | [M2 档](evidence/2026-11-08-m2-wss-wiring.md) |
| M3 py 路由框架 + 先例迁移 + 补丁收编（DC-3） | 进行中（切片一~五 ✅ 残链清空；余 fetcher 闭包三分表） | 蓝图 py 普查档 §三 + [切片一档](evidence/2026-11-08-m3-slice1-route-module-extract.md) + [切片二+三档](evidence/2026-11-08-m3-slice2-registry-head12.md)（23 臂态 A/B 双跑空集，含谓词修复） + [切片四档](evidence/2026-11-08-m3-slice4-fetch-es-40arms.md)（40 臂批 A/B 空集 + FF 合流记录） + [切片五档](evidence/2026-11-08-m3-slice5-residual-clear.md)（残链 21 臂清空 + 基线锚重录归因） |
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
   supersedes 43 号 M4 终账 13,275P 口径）——M3 切片五起对合流后
   HEAD 重录为 **13,614P/1,305F/8T**（32 子测试 F→P 归因 c6dcbce0a
   http-cache M2 wire，[切片五档](evidence/2026-11-08-m3-slice5-
   residual-clear.md) §三；后续批次以新锚对照）；
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
- 2026-11-08 20:35 合流记录：切片四 40 臂在途态与 main FF 六提交
  （524b3b548→a47d00cf9）冲突一处（注册表表尾，autostash）。解法 =
  原链序连续优先（40 行 = 行 24–63），s346 module-fetch-tester.py 行
  合流表尾第 64 行；trickle POST 包裹器（d8d563e14，拦截路径
  fetch/api/resources/trickle.py）与 http-cache M2 wire（c6dcbce0a）
  与本批语义不相交，判定与复核链见切片四档 §五。顺带处置 origin
  既有红：runner 单测编译 E0063（c6dcbce0a 漏改 test 初始化器）以
  `cache: None` 一行最小补齐随本批落库；py_routes 29 处 clippy 既有
  债维持碰头挂账不硬解。过程教训：本批执行实例 18:58 A/B 跑完后
  中断，对照分析 20:17 由后续 session 以孤儿日志补齐——证据档
  「待填」栏不得跨实例存续，长跑等待窗是实例死亡高发段。
- 2026-11-08 22:33 同 session 多实例中断爆发：A/B 等待窗内本 goal 执行
  session 五次中断/重启，各自对同一已完成跑（m3s5-split-ab-222458）落
  §六草稿并暂存（数值一致、两份含 fmt 前旧行数）——续轮按互补链仲裁
  合并为切片五档单节 §六落证；systemd 单元跨 session 存活使长跑未被
  废弃。教训：等待窗内不落「待填」占位的纪律有效，但多实例并发写同
  一档需续轮先 grep 重复节标题再合并。
- 2026-11-08 22:42 补验与在飞声明（续轮实例）：c7207e1e0 收编件独立
  补验通过——注册表表文本 vs c0cab1830 逐字节全等、五族 pub(crate) fn
  计数 15/37/17/9/11（合计 89 项）与 A/B 验证树一致、§六与状态头拆分
  段落在位；该树即单测 298P/0F + 全量 A/B multiset 空集
  （m3s5-split-ab-222458 vs 基线锚）验证态。另：六文件 170 行
  needless_borrow 清理在飞（未暂存，他实例工作面），他轮勿碰；其
  收尾后切片六 fetcher 闭包三分表迁移再认领开工。
- 2026-11-08 22:47 合流收口：push 前 pull --rebase 与上游 7f1c9f717
  （切片四 40 臂 69 处机械 lint + android fetch 臂 cache_mode）冲突
  一处（py_routes 单文件 UU）——取本流迁移版后，rebase 零后续冲突；
  上游 lint 类（needless_return/needless_borrow）在本树余 90 处
  （= 上游修过的 40 臂态 + 本流 21 新臂同形态），以 clippy --fix 同
  手法收敛（170 行，两类机械变换），clippy 零告警 + fmt 净 + 单测
  298P/0F 持平，随收敛提交落库。上游 s347（08f55becb，testharness.rs
  内联 module）与本流唯一交叉行 :5934 无冲突。
- 2026-11-08 22:49 切片六认领（本实例）：fetcher 闭包三分表迁移开工——
  按[清单档](evidence/2026-11-08-fetcher-closure-inventory.md)拓扑迁
  script fetcher（`wpt_data_script_fetcher_with_stash`）与 service-worker
  fetcher（`wpt_data_service_worker_script_fetcher`）为独立表实例
  （`py_script_fetch_routes()`/`py_sw_fetch_routes()`），清单档 §二
  stash 恒等约束守门（script fetcher 的 serve_custom_stash/cred_cookies
  与 PyEndpointCtx 同 Arc，不得另建）；他轮勿碰两 fetcher 闭包段与
  testharness_py_routes 表面，逐臂普查证据随切片六档落。
