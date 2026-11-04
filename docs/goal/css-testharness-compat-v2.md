# css 二期 — testharness 面全目录扩账 + 一期失败簇承接

**版本**: v1.0
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）
**状态**: Active——M0 收口（2026-11-05：甄别 + 数值门 ≥20,500 + 语料
45 目录 missing=0），下一步 M1 批次接线（B1 首批）
**执行模式**: WPT 驱动（上游 css testharness 语料为验收标尺）+ 按簇批量扩面
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——CSS 面）
**一期**: `docs/goal/archive/css-testharness-compat.md`（35 号，2026-10-30 收口）

## coverage 对账（coverage-2026-11-02 基线，全量头条 67.54%）

css topdir 全量分母 **195,723 子测试 / 35,555 文件**，zw 覆盖 3,293 文件
17,923P = 9.16%，缺口 **177,800——全 WPT 最大可做缺口池**（是第二名
editing 103,397 的 1.7 倍；全关上限把头条抬到 ≈75.4%）。一期收口时只入账
十六目录（M0 试点 css-values/selectors + M3 top15），基线 65,501/17,781
= 27.1%——**约 70+ 个 testharness 目录从未导入**，是缺口主体。

一期挂账（master.md 定稿清单，本 goal 承接素材）：

| 簇 | 体量 | 定性 |
|---|---:|---|
| color Color-4 计算值面 | 7,199 | 最大单簇，归口评估 |
| typed-om API 面 | 96.7% Fail | 整体未实现域 |
| grid 布局探测簇 | 3,280 | top15 之最，布局探测 |
| anchor-size() 序列化参数序 | ~2,500 | 引擎可修单点 |
| text 断行 approx 面 | ~543 | 断行语义 |
| `.xht` XML 路径 | 22 页 | XML 解析缺位挂账 |

关键事实：一期收口（2026-10-30）与今快照（2026-11-02）间 css 通过 17,750
→ 17,923 有自然滚动；上游语料持续滚动，M0 必须先对账再定批次与数值门。

## Mission

把 css 域从一期「十六目录选定面」推进到「testharness 面全目录入账」：

1. **对账甄别**（M0）：剩余 ~70+ 目录逐目录甄别——导入即得（testharness
   管道已覆盖的声明值/序列化面）、引擎缺口（布局/绘制耦合）、渲染面
   （reftest 归 rendering-compat，不入本账）、病态排除（骨架案/手工案）；
   产出目录批次清单与 DC-4 数值门。
2. **按簇批量导入**（M1/M2）：过 35 号 M3 模板（子命令 + CORPUS 映射 +
   Makefile target + DIRS + 干跑基线），大目录（CSS2、cssom-view、
   geometry-1、flexbox 等）分批，每批 parity 复验。
3. **一期失败簇承接**（M3）：anchor-size() 参数序单点、grid 探测簇、
   color 计算值面归口评估——可修单点就地修，布局大簇归账 rendering-compat
   或挂账具名。
4. **复测收口**（M4）：coverage css 顶目录落数值门；wpt-suites.csv 回填。

## Done Criteria

- **DC-1（M0 对账）**：剩余目录构成甄别落
  `docs/goal/css-testharness-compat-v2/evidence/`（逐目录：文件数、形态、
  批次归属）；语料 FORCE 续拉对齐 pin（新文件清单 + missing=0）；DC-4
  数值门按决策规则落定并回填本文件。
- **DC-2（通道扩面）**：M1/M2 批次目录全部入常驻通道（runner 子命令或
  既有 css 通道 DIRS 扩面 + skip 规则 + Makefile target）；weekly 分片
  接线核对 discover-shards 日志。
- **DC-3（失败簇承接）**：anchor-size() 序列化参数序与 grid 探测簇处理
  结论落 evidence（就地修或具名挂账归口）；一期十六目录通道零回归
  （分母逐 subtest 对照）。
- **DC-4（coverage 复测）**：css topdir 从 9.16% 升至 M0 落定的数值门；
  全量头条不回落；`wpt-suites.csv` 回填数据行。

**DC-4 数值门决策规则（立项即定）**：M0 甄别后按地址池（导入即得 + 引擎
可修）分档——≥ 100k 子测试：全量推进，数值门 = 地址池 × 一期同类簇通过
率（M0 实测后填入）；< 100k：降级为按簇收尾，只导入高密度目录，其余挂账
渲染/布局域，富余产能转 rendering-compat 协同面。

**数值门落定（2026-11-04，M0）**：未导入池 65,969 子测试（74 suites）<
100k → **走按簇收尾分支**。导入批 = B1 值/属性解析序列化 24 目录（23,882）
+ B2 规则/条件/级联 7 目录（9,751）+ B3 混合布局 13 目录（19,286），合计
44 批次目录（另 css/support，sync 脚本 DIRS 共 45 项）池 52,919；挂账具名：WAAPI 依赖 2 目录 4,867、
API 未实现域 6 目录 3,125、协调面（cssom-view/geometry）2 目录 4,213、
尾部零簇 14 目录 845、渲染面 10 目录 0。逐目录甄别表与门公式见
[evidence/2026-11-04-m0-triage-and-gate.md](css-testharness-compat-v2/evidence/2026-11-04-m0-triage-and-gate.md)。
**DC-4 数值门：css topdir 通过子测试 ≥ 20,500**（9.16% → ≥ 10.47%，按
coverage-2026-11-02 分母 195,723 计；M4 复测按绝对子测试数判，公式 =
基线 17,923 + B1 23,882×7% + B2 7,503×7%〔CSS2 折 html 面 300〕+
B3 19,286×2%，一期同类簇率先行取下缘）。M1 首批干跑校验：B1 实测率 <
3% 触发留痕复评，不静默改门。划界 layout goal 触发实测：布局耦合池
22,566（B3 19,286 + 一期 grid 探测挂账 3,280），扣解析份额后 > 15k >
10k → **独立 layout goal 立项条件成立**（测算与建议已通报，立项待拍板）。

**语料 sync 终态（2026-11-05，DC-1 第二件落定）**：45 目录全部 missing=0，
扩展过滤面 18,031 blob 在册，本轮新拉 6,963（B1 837 + B2 203 + B3 5,923），
blob-hash 不符定向重拉 33（一期存量页随上游滚动过期，#31 旧文件替换成功前
保留）。台账见 evidence §6。

## 划界

- **渲染面 reftest 不入账**：css reftest 归 rendering-compat（既有口径，
  coverage 快照不计 reftest 子测试）；本 goal 只收 testharness 面。
- **cssom-view / geometry-1 与 rendering-compat**：一期预留协调面（
  scripts/goals README 预注）——批次落地前按 run-rules §9 与
  rendering-compat 控制面碰头，避免视口/几何语义双改。
- **layout 引擎大改不进本 goal**：grid/flexbox 布局探测簇若定性为布局
  引擎缺口，归口分档（2026-11-04 用户拍板）——默认记 rendering-compat
  控制面；M0 实测布局簇地址池 > 10k 子测试时独立 layout goal 立项。
  本 goal 修到序列化/声明值层为止。
- **typed-om 不力攻**：CSS Typed OM API 面整体未实现（一期已挂账）；M0
  甄别如实标记，不进数值门。

## 里程碑

- **M0**：对账甄别 + 批次清单 + 数值门落定（DC-1）。**已完成**
  （2026-11-05：语料 45 目录 missing=0，新拉 6,963 / 重拉 33，台账见
  evidence §6）。
- **M1**：高密度目录批次 1-2 导入（DC-2 起步）。
- **M2**：余目录按簇批量导入收尾（DC-2 完成）。
- **M3**：一期失败簇承接归口（DC-3）。
- **M4**：coverage 复测收口 + wpt-suites.csv 汇总行（DC-4）。

## 依赖约束（run-rules §9）

- css 通道 DIRS 扩面与 35 号已收口十六目录同通道——每批次后复验十六目录
  基线逐 subtest 零回归（一期 65,501/17,781 台账）。
- 与 rendering-compat（活跃 rally）共享 cssom-view/geometry 语义面——
  批次含这两目录前先碰头，双方控制面留痕。
- weekly 分片超时预算：css 大目录单套件耗时对照分片表，必要时拆套件级
  分片（form-encode 分片先例），防挤占他域预算。
- 病态案（骨架/超长链）按 test-guard 内存口径排除，skip 规则两侧同源
  （fetch 脚本与 runner 谓词）。
