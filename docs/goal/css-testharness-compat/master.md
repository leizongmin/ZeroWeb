# css-testharness-compat master

**状态**: Active——M0 完成（DC-1）、M1 完成（DC-2，2026-10-30）；M2 未启动
**入口**: [css-testharness-compat.md](../css-testharness-compat.md)

## 里程碑状态表

| 里程碑 | 内容 | 判据 | 状态 |
|---|---|---|---|
| M0 | 试点通道（css-values + selectors：语料深拉 + runner 子命令 + 干跑基线 + 形态分簇） | DC-1 | **完成**（2026-10-30，[基线 evidence](evidence/2026-10-30-m0-pilot-baseline.md)） |
| M1 | runner 通道治理（RSS 归因 → 选型 → coverage 分片消费管道） | DC-2 | **完成**（2026-10-30，[归因 evidence](evidence/2026-10-30-m1-rss-attribution.md)——归因否证线性累积，交付分片 JSON 通道 + 恒等复验 + 全量端到端） |
| M2 | css-variables 病态案排除 + 解禁入账 | DC-3 | 未启动 |
| M3 | 按簇扩面（top15 逐目录过 M0 模板；cssom-view / geometry 先协调划界） | — | 未启动 |
| M4 | coverage 复测收口 + wpt-suites.csv 汇总行 | DC-4 | 未启动（csv 试点数据行已回填） |

## 本轮记录（2026-10-30，M1 收口）

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

## 下一步（M2 检查单）

1. css-variables 病态案排除：`variable-exponential-blowup.html`（4.9GB 单案
   实证）与 `*-crash.html` 进 runner 谓词（`css_testharness_case_skipped` 同款
   单点，fetch 脚本头注释两侧同源——M0 先例）。
2. css-variables 分片 JSON 接 coverage misc 分片：corpus 脚本已参数化
   （CORPUS=css-variables 需补 SUBCMD 分支与 35 号 DIRS 核对），584 子测试
   入账（DC-3：快照 css/css-variables 组 covered>0）。
3. DC-3 落成后转 M3 扩面（top15 逐目录过 M0 模板，cssom-view / geometry 先
   按 run-rules §9 协调）或 M4 复测收口。
