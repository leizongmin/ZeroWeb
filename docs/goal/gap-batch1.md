# 覆盖率缺口小面批量补齐 batch1 — 已实现面 runner 通道 + 甄别面入账

**版本**: v1.1
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）
**状态**: Active——DC-1~DC-4 全达成（2026-11-06 M0~M4：甄别 + CONFIRMED
六面通道 + mixed-content 扩面 + LIKELY 五面通道入账 + coverage 复测收口）；
收口归档待后续轮（evidence 路径已入 `wpt-suites.csv` gap-batch1 行，迁移须同步回填）
**执行模式**: WPT 驱动（上游 corpus 为验收标尺）+ 通道补齐 + 按面甄别
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——多小域批量）

**当前卡点**：无。M4 复测快照 headline 68.10%（基线 67.54%，+0.56pp 不回落），
十二面 covered 全 >0（[M4 收口](gap-batch1/evidence/2026-11-06-m4-coverage-retest.md)，
CI run 42096051904）。引擎/runner 缺口按簇挂账（referrer-policy/preload
`.py` 判据端点仿真、wai-aria testdriver computed label/role 查询臂、
input-events 编辑管线簇、storage StorageManager 全域、gen/ wrapper 管道），
见 M3 evidence 失败簇归因，不在 DC 内。
**下一步**：goal 收口归档（docs/goal/archive/，同步 csv 行 evidence 路径）；
挂账簇按归域立项建议走各自 goal。

## coverage 对账（coverage-2026-11-02 基线，全量头条 67.54%）

十二小面合计分母 **28,394 子测试**，zw 通过 **0**——其中五面已有实现仅缺
runner 通道（审计已核实锚点），六面疑似部分实现须甄别，一面通道已有但语料
近乎为空（CI 实证）。逐面：

| 面 | 分母 | 文件覆盖 | 现状 | 审计结论 |
|---|---:|---:|---|---|
| quirks | 6,984 | 0/51 | 0P | **已实现**——`crates/dom/src/parser.rs` quirks 模式判定（R5346），缺 runner 子命令与语料 |
| domparsing | 3,227 | 0/94 | 0P | **已实现**——DOMParser/XMLSerializer shim（js_dom_shim/part02.js，R189/R3019/R2818） |
| domxpath | 1,166 | 0/36 | 0P | **已实现**——XPathEvaluator shim（part06.js，R2981） |
| subresource-integrity | 489 | 0/36 | 0P | **已实现**——`_zwFetchVerifyIntegrity`（part01.js）+ 模块 integrity（s107） |
| cors | 463 | 0/27 | 0P | **已实现**——`crates/security/src/cors.rs`（s153）；topdir 专面从未入账 |
| referrer-policy | 8,815 | 0/1,483 | 0P | 疑似部分（IDL/fetch init 面）——M0 甄别 |
| input-events | 3,378 | 0/35 | 0P | 疑似部分（editing 域邻接）——M0 甄别 |
| preload | 922 | 0/87 | 0P | 疑似部分（link rel=preload 提示面）——M0 甄别 |
| wai-aria | 588 | 0/35 | 0P | 疑似部分（ARIA 反射 IDL 面）——M0 甄别 |
| storage | 482 | 0/56 | 0P | 疑似部分（StorageManager，OPFS 已归档域之外）——M0 甄别 |
| import-maps | 399 | 0/59 | 0P | **已实现**——`crates/script-sandbox/src/import_map.rs`；语料从未导入 |
| mixed-content | 2,281 | 2/388 | 0P | **通道已有、语料近乎空**——run 41401941211 实证仅 2 文件入语料（1T/1F）；上游 388 文件中 380 为 gen/ iframe/worker 包装（runner 无此管道），窗口可执行面 ≈ 顶层 2 + tentative/autoupgrades 4 |

关键事实：CONFIRMED 六面（含 import-maps）合计 **12,728 子测试**是「实现
已在、只差通道」的纯增量；LIKELY 六面合计 **14,384** 子测试必须先甄别
（引擎面/环境受限面/规范外三分类），不得立项即承诺数值门。mixed-content
的 2,281 大数有迷惑性——窗口可执行面上限约百级子测试，如实落账即可。

## Mission

1. **M0 甄别**：LIKELY 六面逐面对账（哪些子测试面属已实现域、哪些需引擎
   工作、哪些环境受限须 zw-static-corpus 标记）+ mixed-content 上游窗口
   可执行面清单；产出 DC-4 数值门与逐面入账顺序。
2. **CONFIRMED 通道落地**：六面各配 fetch 子集脚本（或复用既有通道扩
   DIRS）+ runner 子命令 + per-dir skip 规则 + Makefile target + weekly
   分片接线（discover-shards 自动发现新套件臂）。
3. **mixed-content 语料扩面**：`fetch-security-csp-subset.sh` 的
   mixed-content 清单从 2 文件扩到窗口可执行全量（顶层 + tentative，
   gen/ 不拉照旧记账）；既有 2 文件 0P 是入账起点，不追既往。
4. **基线入账与复测**：逐面基线落 evidence；coverage 复测验证十二面从 0
   到账、全量头条不回落。

## Done Criteria

- **DC-1（M0 甄别）**：LIKELY 六面 + mixed-content 甄别结论落
  `docs/goal/gap-batch1/evidence/`（逐面：in-scope 子测试数、引擎缺口清单、
  环境受限标记清单）；DC-4 数值门按决策规则落定并回填本文件。
- **DC-2（通道落地）**：CONFIRMED 六面全部入常驻通道（fetch/子命令/skip/
  Makefile/weekly 分片五件套）；`make test` 相关套件 test-guard 包裹。
- **DC-3（mixed-content 语料）**：窗口可执行面全量入语料（FORCE 续拉对齐
  pin，missing=0）；`testharness-mixed-content` 既有账零回归。
- **DC-4（coverage 复测）**：十二面 covered 文件数 > 0 且通过子测试达
  DC-1 数值门；全量头条 ≥ 67.54% 不回落；`wpt-suites.csv` 回填数据行。
  ✅ 2026-11-06 全满足——分面门达成为通道干跑账口径（快照分子经 universe
  解析存在机制性差额，注记见 M4 evidence）。

**DC-4 数值门决策规则（立项即定）**：CONFIRMED 六面通道落地后按干跑基线
实数落门（不预设）；LIKELY 六面按 M0 甄别的 in-scope 地址池 × 同类域已收
口通过率落门；环境受限面（TLS/多 frame/worker 依赖）不进门，落
zw-static-corpus 标记记账。

**DC-4 数值门落定（2026-11-06，M1 干跑实数 + M0 池甄别）**：

- CONFIRMED 六面分面门 = 干跑基线 P 数（quirks 35 / domparsing 151 /
  domxpath 21 / subresource-integrity 32 / cors 92 / import-maps 34，合计
  365；证据 [M1 基线](gap-batch1/evidence/2026-11-06-m1-confirmed-channel-baselines.md)）。
- mixed-content 零回归锚：6 文件 11 子测试全入账，既有账 blob 1T +
  imageset 1F 逐位不变（M2 同轮验证）。
- LIKELY 五面：M0 池甄别已落（referrer-policy 69 / input-events 25 /
  preload 51 / wai-aria 3 / storage ≈1 文件，
  [M0 甄别](gap-batch1/evidence/2026-11-06-m0-likely-facet-triage.md)）；
  数值门 M3 干跑实数落定（预登记规则 = 门 ≥ 干跑基线 P 数）：referrer-policy
  1 / input-events 188 / preload 106 / wai-aria 2 / storage 不设门（引擎
  缺口全域，covered>0 对账），合计 P 297
  （[M3 基线](gap-batch1/evidence/2026-11-06-m3-likely-facet-baselines.md)）。
  池实跑对账：wai-aria 池 3→实跑 28 文件（402F 单簇 = testdriver computed
  label/role 缺位，M0 预期证伪）；storage 池 ≈1→实跑 3 文件（manual 变体
  谓词修复后）。环境受限面（gen/ 镜像、编辑管线簇、.any.js 面、
  `.py` 端点簇）不进门。

## 划界

- **domparsing/domxpath 与 dom-expansion**：DOM 接口面归 34 号；本 goal
  只收 DOMParser/XMLSerializer/XPathEvaluator 专面通道，接口行为不改。
- **referrer-policy 与 net-api**：fetch Referrer-Policy init 语义归
  net-api 通道既有账；本 goal 只收 referrer-policy topdir 专面通道。
- **input-events 与 editing/uievents**：editing 域归 33 号、uievents 已
  归档；本 goal 只收 input-events topdir 通道，getCoalescedEvents 等输入
  管线依赖面如实标环境受限。
- **wai-aria 与 html-aam/accname**：无障碍映射面（html-aam 891 + accname
  500）不在本批（引擎渲染树语义缺位，另行立项）；本 goal 只收 wai-aria
  topdir。
- **mixed-content 与 security-hardening 归档账**：CSP/secure-contexts
  两通道既有账零回归；mixed-content 大头（gen/ iframe/worker 面）挂账
  runner 基建，不进本 goal DC。
- **cors 与 net-api**：`crates/security/src/cors.rs` 引擎语义不重开；
  本 goal 只收 cors topdir 27 文件专面通道。

## 里程碑

- **M0**：逐面甄别 + 数值门落定（DC-1）——✅ 2026-11-06（LIKELY 五面池
  甄别 + mixed-content 窗口清单 + 门落定，evidence/2026-11-06-m0-*.md）。
- **M1**：CONFIRMED 六面通道五件套落地（DC-2）——✅ 2026-11-06（fetch 脚本
  + 六子命令臂 + skip 规则 + Makefile target + weekly catch-all 自动接线；
  干跑基线 P 365，evidence/2026-11-06-m1-*.md；随轮共享路径修复
  `inline_local_scripts` pipe=sub substitution 臂，回归单元测试在册）。
- **M2**：mixed-content 语料扩面 + 基线（DC-3）——✅ 2026-11-06（顶层 2 +
  tentative/autoupgrades 4 = 6 文件 11 子测试入账，missing=0；既有账逐位
  零回归；gen/ 512 不拉照旧挂账）。
- **M3**：LIKELY 面按甄别结果逐面入账（引擎缺口簇挂账他域）——✅
  2026-11-06（五面五件套 + manual 变体谓词修复 + 干跑基线 P 297 门落定，
  evidence/2026-11-06-m3-*.md；weekly catch-all 自动接线本地复核 128 套件）。
- **M4**：coverage 复测收口 + wpt-suites.csv 汇总行（DC-4）——✅
  2026-11-06（weekly dispatch run 42096051904 全分片 success；快照
  headline 68.10% 不回落、十二面 covered 全 >0；csv gap-batch1 行 331
  文件/3,811 子测试/662P 回填；evidence/2026-11-06-m4-coverage-retest.md）。

## 依赖约束（run-rules §9）

- 新 runner 子命令臂按 main.rs 匹配臂序追加，`ci_discover_shards.py` 的
  ≥40 套件断言与 ci-shards.yml 差集自动消费——分片接线后核对 weekly
  discover-shards job 日志无新增 tune 缺口。
- fetch 子集脚本统一 WPT_REV pin（lib.sh）；语料上游滚动走 pin 对齐 +
  drift 巡检（31 号先例）。
- 与 html-semantics-compat（主攻中）、editing-compat（主攻中）、
  rendering-compat 无共享面；referrer-policy 甄别若触 fetch init 语义，
  改动同窗复验 net-api v2 通道分母逐 subtest 零回归。
- 大面（referrer-policy 1,483 文件）导入走分批，单分片 job_timeout 预算
  对照 weekly 分片表，防挤占。
