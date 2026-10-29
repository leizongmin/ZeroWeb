# css-testharness-compat master

**状态**: Active——M0 未启动（契约 2026-10-27 立项；2026-10-29 基线复核 +
控制面建档）
**入口**: [css-testharness-compat.md](../css-testharness-compat.md)

## 里程碑状态表

| 里程碑 | 内容 | 判据 | 状态 |
|---|---|---|---|
| M0 | 试点通道（css-values + selectors：语料深拉 + runner 子命令 + 干跑基线 + 形态分簇） | DC-1 | 未启动 |
| M1 | runner 套件内存治理（RSS 归因 → 方案选型 → coverage 分片消费管道） | DC-2 | 未启动 |
| M2 | css-variables 病态案排除 + 解禁入账 | DC-3 | 未启动 |
| M3 | 按簇扩面（top15 逐目录过 M0 模板；cssom-view / geometry 先协调划界） | — | 未启动 |
| M4 | coverage 复测收口 + wpt-suites.csv 汇总行 | DC-4 | 未启动 |

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

## 下一步（M0 检查单）

1. 语料：`FORCE=1 bash tests/wpt-runner/scripts/goals/35-css-testharness-compat.sh`
   深拉 css-values + selectors（含 support 资产），盘点落 evidence。
2. runner：`testharness-css-values` / `testharness-selectors` 子命令（或单一
   `testharness-css` 按目录过滤）+ per-dir skip 规则 + Makefile target
   （test-guard 包裹，skip 两侧同源防漂移——html-semantics 先例）。
3. 干跑基线：全量分类通过率（文本 + JSON）落
   `evidence/`，形态分簇表（直跑 / any.js / 资产依赖）入账；
   `docs/compat/trends/wpt-suites.csv` 回填数据行（DC-1）。
4. DC-1 落成后转 M1：RSS 归因小实验先行，再选型落地内存治理。
