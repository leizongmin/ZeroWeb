# css-testharness-compat master

**状态**: Active——M0/M1/M2/M3 完成（DC-1~DC-3 全落，DC-3 CI 快照已闭环；M3 批次 1–5，top15 收口 15/15）；M4 进行中——css-face 三分片接线已合入（62064f318），DC-4 数据由姊妹流 weekly 运行 41000541561（HEAD 含接线）产出，轮询收口中
**入口**: [css-testharness-compat.md](../css-testharness-compat.md)

## 里程碑状态表

| 里程碑 | 内容 | 判据 | 状态 |
|---|---|---|---|
| M0 | 试点通道（css-values + selectors：语料深拉 + runner 子命令 + 干跑基线 + 形态分簇） | DC-1 | **完成**（2026-10-30，[基线 evidence](evidence/2026-10-30-m0-pilot-baseline.md)） |
| M1 | runner 通道治理（RSS 归因 → 选型 → coverage 分片消费管道） | DC-2 | **完成**（2026-10-30，[归因 evidence](evidence/2026-10-30-m1-rss-attribution.md)——归因否证线性累积，交付分片 JSON 通道 + 恒等复验 + 全量端到端） |
| M2 | css-variables 病态案排除 + 解禁入账 | DC-3 | **完成**（2026-10-30，[解禁 evidence](evidence/2026-10-30-m2-css-variables-unban.md)；CI 快照运行 40935989794 闭环——58 files / 232 passes） |
| M3 | 按簇扩面（top15 逐目录过 M0 模板；cssom-view / geometry 先协调划界） | — | **完成**（批次 1–5 全落 2026-10-30，top15 分母目录 15/15 入账；十六目录统一基线 65,157 子测试 17,747 Pass = 27.2%；evidence/2026-10-30-m3-batch{1..5}-*.md） |
| M4 | coverage 复测收口 + wpt-suites.csv 汇总行 | DC-4 | **进行中**（三分片接线 62064f318 + fetch 白名单补 css/support e425f845a；DC-4 数据由修复后运行 41005658107 产出，轮询收口中） |

## 本轮记录（2026-10-30，M3 批次 2–5 收口 + M4 分片接线）

- **M3 批次 2–5 入账（top15 收口 15/15）**：typed-om / fonts / shapes →
  anchor-position / masking / backgrounds → transforms / motion → grid /
  text 逐批过 M0 模板（子命令 + CORPUS 映射 + Makefile target + DIRS），
  十六目录统一基线 **65,157 子测试 / 17,747 Pass = 27.2%**，每批 parity 全
  IDENTICAL。大簇挂账：typed-om API 面整体未实现（96.7% Fail）、
  anchor-size() 序列化参数序 ~2,500、grid 布局探测簇 3,280（top15 之最）、
  text 断行 approx 543。分批证据：
  [batch2](evidence/2026-10-30-m3-batch2-typed-om-fonts-shapes.md) /
  [batch3](evidence/2026-10-30-m3-batch3-anchor-position-masking-backgrounds.md) /
  [batch4](evidence/2026-10-30-m3-batch4-transforms-motion.md) /
  [batch5](evidence/2026-10-30-m3-batch5-grid-text.md)。
- **M4 接线合入（62064f318）**：weekly coverage matrix 新增 css-face-a/b/c
  三分片（suite 名 = runner 子命令去前缀，aggregate 按案路径归 topdir，
  css 顶目录首次入快照口径）；35 号脚本加 `CSS_FETCH_ONLY` 白名单过滤
  （缺省不设 = 全量，本地 rally 行为不变），三分组 fetch target 按语料体量
  均分；css 分片 job timeout 210min、per-suite time_limit 3600。
- **在途（白名单修复，e425f845a）**：首次带接线运行 41000541561 的
  css-face-b 产物早期对账实证 fetch 白名单漏 `css/support`——案数与本地
  逐目录一致（253/61/121/348）但 helper 引用页子测试坍缩（masking 544→63、
  anchor-position 12,811→3,059、backgrounds 1,085→190；typed-om 零引用
  750=750 反证根因唯一；idl_test 4 页缺 /resources/idlharness.js 与本地
  同界非 CI 洞）。运行已取消（避免聚合落污染快照），三分片白名单补
  css/support 后重派运行 41005658107，轮询收口中——快照 css 面数字核验 +
  wpt-suites.csv 汇总行待其完成后落账。

## 上一轮记录（2026-10-30，M3 批次 1 + DC-3 CI 闭环）

- **DC-3 正式闭环**：workflow_dispatch coverage 运行 40935989794 全绿
  （17 shard success / misc 1,795s）；聚合快照 `css/css-variables`
  covered_files=58（判据 ≈58 ✓）/ zw_passes=232（本地 239，−7 = CI runner
  动画子测时序差异，案数 58=58）/ zw_env_fails=0；css 顶目录自 0 入账
  （61 files / 232 passes / 195,338 分母 / 0.12%）。
- **M3 批次 1 入账**：css-color / css-align / css-sizing 三目录过 M0
  模板（子命令 + 泛化 CORPUS 映射 + Makefile target + DIRS）；语料
  git blob hash 对账至上游 pin 全量一致（460/342/865——精选仓内容漂移
  21+4 页实证"文件数一致 ≠ 内容一致"）；单进程基线 color 62 案
  11,009 子测试 11.3%、align 234 案 4,540 子测试 14.5%、sizing 168 案
  1,894 子测试 37.0%。六目录统一基线 **1,052 案 / 27,824 子测试 /
  5,593 Pass（20.1%）**，csv 聚合行同步。
- **根 support 文件修复（批次 1 最大发现）**：`resources/check-layout-th.js`
  缺失（css 面 201 页引用）使 align/sizing 的 checkLayout 族页脚本报错或
  挂起——153 个 Timeout 子测试、2,921 个子测试被掩蔽；补拉 + goal 脚本
  定向接线后 **Timeout 全部清零**、两目录墙钟 1m54s→41s / 2m10s→22s。
  testdriver\*.js 不补（runner 装配层置空注入，无需落盘）。
- **失败分簇（六目录）**：color 一簇独大——Color-4 计算值面 7,199
  （73.7%，color-mix/oklch/light-dark computed style 不支持）；align
  指定值展开/序列化 ~1,345 + 布局几何 ~848；sizing aspect-ratio ~145 +
  布局几何 ~300+。布局几何两簇与 rendering-compat 同域，修复须两账
  同窗回归。
- **parity 全语料首跑**：color 62/62、align 234/234 IDENTICAL；sizing
  6 子测 status-diff（stretch 族）——三步归因实验（standalone ≡ 分片臂 /
  全量重跑 0 差异）定性为**单进程跨案状态残留**（engine 页面加载顺序
  依赖，0.3%），按 M2 Timeout 先例记通道语义差异，归失败分簇账本。
- **跨流修复**：兄弟流（editing-compat）dc719ecf0 在 csv 数据行引入裸
  逗号（16 列，rebase 后 trend_csv_check 红）——按生成器契约改分号，
  随本轮提交。属并行双流共享面碰撞的常规互修，无工作面重叠。

## 上一轮记录（2026-10-30，M2 收口）

## 本轮记录（2026-10-30，M2 收口）

- **通道迁移**：`testharness-css-variables` 子命令自 R5114 专用通道迁入
  css 共享底座（`run_css_corpus_dir` + 共享谓词单点）；R5114 谓词、
  `CSS_VARIABLES_SUBDIRS` 与 `run-css-variables-corpus.sh`（头注释含被 M1
  否证的 35MB/案前提）删除；分片脚本补 css-variables 分支，Makefile
  target 指向共享脚本，M1 parity 脚本同步支持。
- **病态案谓词排除（M2 核心）**：共享谓词新增 `PATHO_CASES`
  （`variable-exponential-blowup.html` 4.9GB + css-values
  `if-conditionals.html` 双案名单）；探针复验即时 `[]` 无内存事件。
- **运行面语义修正**：R5114 通道对 202 页非 testharness reftest 形态页照跑
  产合成单 Pass——coverage 管道按案 `max(universe, zw_total)` 会虚增分母；
  共享谓词"无 testharness 引用"规则把这些页出运行面，入账面 = 真实
  harness 断言面。运行面 **58 案 532 子测试：239P/282F/11TO**（44.9%，
  Timeout 全部为 animation/transition 异步子测试——M3 分簇素材）。
- **语料对账**：本地精选仓对照上游 pin（git trees 清单）缺 2 用例页 +
  5 helper 资产，补齐至 268/268；`fetch-css-variables-subset.sh` 扩展为
  用例树深挖补拉（goals/lib.sh 复用），weekly misc 分片 fetch 入口 =
  `fetch-wpt-css-variables`；35 号 DIRS 加入 css/css-variables。
- **CI 接线**：weekly.yml misc 分片 suites 加入 css-variables（单进程
  `--json` 形态，本地实测 1m58s / 峰值 RSS 983MB——排除病态案后守卫余量
  充足，DC-2"weekly 免特调纳入新套件"判据实践）；矩阵注释同步更新。
- **验证**：三通道计数一致（单进程 JSON ≡ 分片文本 ≡ parity 子集 10/10
  IDENTICAL）；coverage 管道消费 0 suspicious；M0 试点面探针回归一致；
  runner bin 265/265；wpt-suites.csv 聚合行更新（10,375/2,978，生成器
  契约 8/8）。
- **DC-3 边界**：本地判据全落；CI 快照（css/css-variables 组 covered>0）
  待下一次 weekly cron 落账——misc 接线、fetch 入口、管道消费三环节已
  本地验证。

## 上一轮记录（2026-10-30，M1 收口）

- **归因否证**：逐案 RSS 插桩（`ZW_TESTHARNESS_MEM_LOG=1`，`HarnessCaseTimer`
  RAII + `/proc` 采样，镜像 `REFTEST_MEM_LOG` 先例）实测四序列 197 案斜率
  0.06–0.82MB/案——"~35MB/案线性累积"前提不成立；历史 css-variables >8.8GB
  修正为病态案 4.9GB 抬升笼水位的平摊误读。内存维度不治理，每案独立进程通道
  的真实价值 = 病态内存案与原生同步死循环案的外部兜底隔离。
- **DC-2 交付**：分片 corpus 脚本增 JSON 通道（`CSS_CORPUS_JSON_DIR` runner
  形状合并数组 + `CSS_CORPUS_ONLY_SUBSTR` 子集过滤 + 墙钟杀案合成 Timeout
  条目）；`verify-css-corpus-json-parity.sh` + `make testharness-css-parity`
  恒等复验（selectors 43/43、css-values 59/59 IDENTICAL）；`wptfyi_gap.py`
  results 目录通配 + 按案取最完整——分片文件零特调投喂。
- **全量端到端**：selectors 274 案 2,001P/2,249F/6TO（4m11s）、css-values
  256 案 738P/4,837F/12TO（2m47s），与 M0 文本基线逐项一致，合并 JSON 落
  coverage 消费形状。
- **回归修复**：`trend_csv_check` 抓到 M0 回填的 csv 行 note 字段含裸逗号
  （17 列违反生成器契约，单测先红）——去逗号修复，runner bin 265/265 过。
- **经验沉淀**：[守卫杀树总增量按案平摊会伪造线性内存累积结论](../../../learnings/bugs/2026-10/2026-10-30-guard-kill-total-averaging-fakes-linear-memory-growth.md)。

## 上一轮记录（2026-10-30，M0 收口）

- **通道落地**：`testharness-css-values` / `testharness-selectors` 子命令
  （tests/wpt-runner/src/testharness_css_corpus.rs，svg 模块先例）+ 内容
  筛减规则（runner 谓词与 fetch 脚本头注释两侧同源）+ Makefile 全量分片
  target 与 -one 定向 target（test-guard 包裹）。
- **基线（DC-1）**：run-face 530 案 9,843 子测试——css-values 738/5,587 =
  13.2%、selectors 2,001/4,256 = 47.0%（合计 27.8%）；逐案 JSON + 失败簇
  归类落 evidence；wpt-suites.csv 试点数据行回填（分母口径 = 本地 pin
  run-face，与 coverage 快照并行）。
- **形态分簇**：两目录均无 `.any.js`；html 总数 43%/42% 为 testharness
  run-face，其余为 reftest 渲染面（rendering-compat 流域，内容规则自动
  分界）；`.xht` 22 页 XML 路径缺位挂账。
- **基建补丁**：goals/lib.sh 深挖分支补 `*.css` 扩展；35 号 DIRS 补共享
  helper 目录 `css/support`（`parsing-testcommon.js` 等 22 helper 到位）；
  分片运行器补每案 90s 外部墙钟（runner 内部 30s 超时拦不住同步死循环）
  + 断点续跑/并行分片参数。
- **病态案**：css-values `if-conditionals.html` 单案 Oilpan CagedHeap OOM
  （fuzz-repro-guard 复现），分片脚本名级跳过（css-variables 先例）；
  重入条件 = if() 求值内存语义修复。
- **教训**：跨脚本版本的并行分片会 todo 错位一档（病态案跳过增删使奇偶
  互补变完全重叠）——分片必须同版本脚本 + 同 seeding；本轮 152 案漏跑由
  第三次续跑收口。

## 缺口清单（立项时点，2026-10-29）

1. **试点两目录无 runner 通道**：runner 无 css/* testharness 子命令；35 号
   fetch 脚本已预置 css-values + selectors 深拉（`GOAL_FETCH_DEEP=1`），
   通道 / per-dir skip 规则 / Makefile target 待 M0 落地。
2. **形态分簇未实测**：`.html` 直跑 / `.any.js` window 变体 / support 资产
   依赖的构成只能实测（css-values 519 文件 / selectors 833 文件）。fetch 前
   本地语料以渲染清单为主，testharness 语料不全。
3. **长套件内存治理**：单进程 RSS ~35MB/案线性累积（css-variables 全语料
   单进程 >8.8GB 被 test-guard 杀，三跑复现实证）是扩面套件进 CI coverage
   的前置；M1 先做 RSS 归因小实验（分配器驻留 vs 引擎态），再在进程内分批
   与子进程分片间选型。
4. **css-variables 病态案**：`variable-exponential-blowup.html` 单案 4.9GB
   杀树实证；`*-crash.html` 按 fuzz-repro-guard 口径本就不入常驻运行面。
   skip 规则单点排除后，既有分片运行器
   （tests/wpt-runner/scripts/run-css-variables-corpus.sh，每案独立进程）
   经 M1 管道接回 coverage misc 分片（M2）。
5. **CSSOM 通过率带未知**：testharness 面测 `getComputedStyle` /
   `CSSStyleDeclaration` / 支持性判断，与渲染面（rendering-compat 95%+）
   不是同一能力域。shim 现有 getComputedStyle 命中（part01/04/06 共 19
   处）只证明实现存在，非覆盖度；试点干跑才能定性。

## 碰撞管理记录

- **边界**：reftest 渲染账（16,594 案）归 rendering-compat，两账按文件形态
  天然分界；cssom-view / geometry 与 rendering-compat 工作面重叠（DOMRect /
  scroll 断言双域），排 M3+，落地前按 run-rules §9 协调。
- **css-transitions / css-animations**：断言依赖 WAAPI 底座（timing-animation-
  compat 已收口），导入后失败簇优先归口该底座挂账，不在本 goal 修引擎。
- **零回归底线**：css 值语义修复若触 `zero-engine`/`style-system`，与
  rendering-compat 的 reftest 回归门（716/716）同窗验证，防两账互相打红。

## 下一步（M4 收口检查单）

1. ~~DC-3 CI 快照落账~~（闭环：40935989794，58 files / 232 passes）。
2. ~~M3 批次 2–5~~（top15 收口 15/15，见本轮记录）。
3. **M4（DC-4）收口**：weekly 运行 41000541561 完成后——快照
   coverage-2026-10-30 核验（css 顶目录 covered>0 且 16 目录全入组、
   试点目录未覆盖子测试清零路径成立）→ evidence 落账 →
   wpt-suites.csv 汇总行（十六目录 65,157/17,747 run-face 口径 + DC-4
   快照注记）→ 控制面状态收口。
4. **失败簇归口评估**（后续 goal 素材，不阻 M4）：color 的 Color-4 计算
   值面（7,199，最大单簇）评估引擎工作项归属；align/sizing 指定值展开族
   归 CSSOM 序列化、布局几何族与 rendering-compat 同窗；grid 布局探测
   3,280 体量之最；typed-om API 面（96.7% Fail）整体挂账。
