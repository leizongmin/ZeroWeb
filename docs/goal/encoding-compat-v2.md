# encoding 二期 — legacy-mb 未导入族 + 根面 labels/API 残余 + streams 收口

**版本**: v1.0
**日期**: 2026-10-31（立项，coverage-2026-10-30 对账驱动）
**状态**: Active——立项，M0 未启动
**执行模式**: WPT 驱动（上游 encoding 语料为验收标尺）+ 一期通道复用（testharness 编码读取点 + 分片管道）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——编码/文本处理面）

## coverage 对账（coverage-2026-10-30 基线）

encoding topdir 全量分母 **1,152,394 子测试 / 1,345 文件**，zw 578,483 =
50.2%，缺口 **573,911——全 WPT 最大单一缺口池**（全量未覆盖 4,212,627 的
13%）。一期三个 goal 均已收口归档，但各自只收了选定族面：

| 子域 | 分母 | zw passes | 通过率 | 文件覆盖 | 一期账 |
|---|---:|---:|---:|---|---|
| encoding/legacy-mb-japanese | 447,723 | 244,033 | 54.5% | 482/482 | 31 号只收 `*_chars-<label>.html` 解码族（253 文件 100%P） |
| encoding/legacy-mb-korean | 410,448 | 213,752 | 52.1% | 435/435 | 同上 |
| encoding/legacy-mb-tchinese | 268,916 | 119,248 | 44.3% | 283/283 | 同上 |
| encoding（根面） | 23,818 | 678 | 2.8% | 20/91 | 71 文件零覆盖——labels/API/TextDecoder 面从未导入 |
| encoding/streams | 472 | 112 | 23.7% | 11/48 | 残余未收 |
| encoding/legacy-mb-schinese | 1,017 | 660 | 64.9% | 4/6 | 残余未收 |

关键事实：三目录 upstream 文件数（482/435/283）远超一期预置语料（七目录
合计 253 文件），说明一期收口后上游语料已滚动扩面；剩余族的构成（文档解码
变体 / io 表 / API 面 / 异常分支 / 规范不支持面）**未经甄别**，M0 必须先
对账再定数字。

## Mission

把 encoding 域从一期「选定族面收口」推进到「全族面入账」：

1. **对账甄别**（M0）：三目录 + 根面 + streams 剩余族的构成甄别——哪些是
   导入即得（解码管道已覆盖）、哪些是引擎缺口、哪些是规范不支持面
   （TextEncoder 仅 UTF-8，legacy encode 无规范路径）。产出族清单、干跑
   基线和 DC-4 数值门。
2. **根面与 streams 收口**（M1）：71 个零覆盖文件（labels 列表、
   TextDecoder/TextEncoder API、异常分支）+ streams 残余入常驻通道。
3. **legacy-mb 三目录批量入账**（M2）：按族分批过 M0 模板（导入 → 干跑
   基线 → 失败簇归因），复用 goal-35 M3 五批次先例的节奏与产物形状。
4. **缺口归因与复测收口**（M3/M4）：引擎缺口簇逐簇归账；coverage 复测
   落数值门；wpt-suites.csv 回填汇总行。

## Done Criteria

- **DC-1（M0 对账）**：剩余族构成甄别落
  `docs/goal/encoding-compat-v2/evidence/`；语料 FORCE 续拉对齐 pin
  （新文件清单 + missing=0）；通道复用干跑基线（文本 + JSON 双形状）；
  DC-4 数值门按下方决策规则落定并回填本文件。
- **DC-2（根面 + streams）**：encoding 根 71 个零覆盖文件入常驻通道且
  covered>0；streams 残余（37 文件）入账；通道与 Makefile target 落地
  （test-guard 包裹）。
- **DC-3（legacy-mb 入账）**：三目录未导入族全部进通道与 weekly coverage
  分片；快照三目录 zw_passes 达 DC-1 数值门（决策规则见下）。
- **DC-4（coverage 复测）**：encoding topdir 从 50.2% 升至 M0 落定的数值
  门；30/31/36 零回归门全绿（见划界）；wpt-suites.csv 回填数据行。

**DC-4 数值门决策规则（立项即定）**：M0 甄别后按地址池大小分档——

- 地址池（导入即得 + 引擎可修）≥ 100k 子测试：全量推进，数值门 = 地址池
  × 一期同类族通过率（M0 实测后填入）。
- 地址池 < 100k：降级为收尾 goal，只做 DC-2（根面 + streams），legacy-mb
  剩余面甄别后挂账，富余产能转 css-testharness 二期（见
  `docs/goal/archive/css-testharness-compat.md` 挂账清单）。

## 划界

- **规范不支持面不力攻**：WHATWG encoding 标准规定 legacy 编码仅解码方向
  （TextEncoder 仅 UTF-8）；甄别中发现的 encode 向测试若无规范路径，落
  挂账清单不实现。
- **一期已收口账不重开**：31 号 decode 族（385 文件 / 360,431 子测试
  100%P）与 36 号 form/href encode 面（766,958P/0F/1T）只做零回归门——
  本 goal 任何通道/引擎改动后逐字节复验，不重跑扩面。
- **30 号 encoding-compat 一期账**同上，零回归门。
- **net-api-compat-v2 重入条件**不受本 goal 影响：encoding/streams 的
  ReadableStream 面归本 goal，fetch/api streams 归 20 号挂账口径。
- **不触 CSSOM/DOM/layout/editing**：纯解码/编码域，与 html-semantics、
  rendering-compat、editing-compat（在跑 rally）、selection-compat、
  dom-expansion、webcrypto-compat 二期全部无交集。

## 里程碑

- **M0**：对账甄别 + 数值门落定（DC-1）。
- **M1**：根面 + streams 收口（DC-2）。
- **M2**：legacy-mb 三目录按族批量入账（DC-3 数值门达成）。
- **M3**：引擎缺口簇归因收敛（如 M0 甄别出可修簇；无则并入 M2 批次）。
- **M4**：coverage 复测收口 + wpt-suites.csv 汇总行（DC-4）。

## 依赖约束（run-rules §9）

- 编码通道与解码管道改动，同窗复验 30/31/36 三本零回归账（逐字节判据）。
- weekly coverage 分片接线须与 fd2771ea4（form-encode 四分片）、
  12deb8557（coverage 补拉 + case_timeout 提额）刚合入的 encoding 域改动
  同窗核对分片超时预算，防互相挤占。
- 大表驱动测试（单文件数千子测试）沿用 test-guard 内存上限口径（per-proc
  与 total 两级）；病态案（如有）按 fuzz-repro-guard 口径排除，skip 规则
  两侧同源（fetch 脚本与 runner 谓词）。
- 语料上游滚动：pin 对齐 + drift 巡检（31 号先例）；"文件数一致 ≠ 内容
  一致"（goal-35 三次实证），重拉走 blob-hash 对账。
