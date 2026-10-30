# encoding 二期 — legacy-mb 未导入族 + 根面 labels/API 残余 + streams 收口

**版本**: v1.3（M4 复测在跑）
**日期**: 2026-10-31（立项；同日 M0 对账 + 降级落定、M1 可导入集入通道）
**状态**: Active——M0 + M1 完成（DC-2 达成：big5-encoder 13P 新证 +
iso-2022-jp-encoder 2P、10F 流式模型簇挂账，见
[M1 证据](encoding-compat-v2/evidence/2026-10-31-m1-importable-set-wiring.md)）。
M4 coverage 复测进行中：三源分片管道在跑（`/tmp/zw-enc2-m4/`，test-guard
逐分片包裹；enc30 已齐 12,075 = 12,072P/3F 与 30 号在册残差逐一吻合；
form36 八分片 70 文件精确不相交覆盖已模拟验证 + enc31 队尾），
tally 判读脚本 `/tmp/zw-enc2-m4/tally_dc4.py` 就位（分片缺口报 MISSING）。
预期口径：本地三源 1,139,490 子测试、passes 1,139,476 = 天花板（非 P 恰
14 = 3F SAB 残差 + 10F 流式模型簇 + 1T data:iframe 契约挂案）。
**卡点**：无。
**下一步**：分片齐（`/tmp/zw-enc2-m4/ALL-DONE`）→ tally 判读（DC-4 门
≥98.8%）→ M4 evidence + wpt-suites.csv 37 号行 + 收口提交。
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

## M0 对账结论（2026-10-31）

上表「缺口 573,911」经甄别为**覆盖测量假象，非引擎缺口**，三分归因：
form-encode 分片旧预算超限（encode 面只捕 28%，2167d9ca6/12deb8557 已修）、
根面/streams 分片假低（`net` 分片只捕 790，本地全量干跑 12,072P/3F 逐字
复现 30 号收口账）、分母侧 pin 漂移 25 子测试。legacy-mb 三目录 + schinese
1,206 文件全部对账闭合（decode 34 文件 360,431 = 31 号 100%P；encode 68
文件 766,959 = 36 号 99.9999%；gbk/gb18030-decoder 窗口腿 30 号在收）；
encoding 域 224 案文件 / 1,152,394 子测试无一处于「导入即可跑而未跑」状态。

真实地址池 = 可导入集 25 子测试（big5/iso-2022-jp-encoder inline `a.href`
面，36 号同机制），结构挂账池 12,879（worker 11,821 + sw/shared 632 +
window-form 342 + idlharness 57 + doc-iframe 19 + 文档管道面 7——
bom-handling/sniffing/json-document-utf8 的 `document.characterSet`/默认
编码探测耦合，源码实核 + 30 号 M4 划界挂账 html-syntax-compat——+
SAB/https 1）。逐文件 disposition 与干跑双臂基线见
[evidence/2026-10-31-m0-family-adjudication.md](encoding-compat-v2/evidence/2026-10-31-m0-family-adjudication.md)。

**降级决策（按下方立项规则第二档，地址池 25 < 100k）**：收尾 goal——
只做 M1（DC-2 可导入集入通道）与 M4（coverage 复测 + csv 行）；legacy-mb
剩余面不重开，DC-3 撤销、改挂账口径；富余产能转 css-testharness 二期。

## Mission（v1.1 收尾口径）

1. ~~对账甄别~~（M0，已完成）：见上节与 evidence。
2. **根面可导入集收口**（M1）：M0 甄别出的 25 子测试入通道且 covered>0
   （big5/iso-2022-jp-encoder inline `a.href` 面）——36 号通道扫描域加
   `encoding` 根（谓词放行规则既有，仅此两文件匹配）；新面定向干跑，
   big5 若暴露编码器表缺口按「引擎可修」簇归账。
3. ~~legacy-mb 三目录批量入账~~（M2 撤销，降级）：无未导入案面，结构
   挂账池 12,889 子测试（M1 后口径，见 evidence 分族账本），不重开。
4. **复测收口**（M4）：coverage 复测落 DC-4 数值门；wpt-suites.csv 回填
   汇总行。

## Done Criteria（v1.1 收尾口径）

- ~~**DC-1（M0 对账）**~~ **已完成（2026-10-31）**：分族账本与逐文件
  disposition、干跑双臂基线（12,072P/3F，与 30 号收口账逐字吻合）、
  「缺口 573,911」三分归因落 evidence；DC-4 数值门已落定并回填下文。
- ~~**DC-2（可导入集收口）**~~ **已完成（2026-10-31，M1）**：2 文件入
  36 号常驻通道且 covered>0（big5-encoder 13P 新证 big5 编码器表 URL
  序列化面；iso-2022-jp-encoder 2P/10F，10F 为 per-scalar fresh 模型与
  流式状态机结构性分歧簇，挂账不重开）；legacy 七目录扫描代码与谓词恒等 +
  schinese 恒等干跑 303/303 + weekly 两分片 filter 去尾斜杠并入根面
  （接线与干跑见
  [evidence/2026-10-31-m1-importable-set-wiring.md](encoding-compat-v2/evidence/2026-10-31-m1-importable-set-wiring.md)）。
- ~~**DC-3（legacy-mb 入账）**~~ **撤销（M0 降级）**：改挂账口径——
  worker 11,821 / sw+shared 632 / window-form 342 / idlharness 57 /
  doc-iframe 19 / SAB+https 1 子测试记录在案，dedicated-worker 腿 re-entry
  留待 workers 通道评估。
- **DC-4（coverage 复测）**：encoding topdir 通过率 ≥ **98.8%**（对
  2026-10-30 分母 1,152,394 即 ≥ 1,138,566 通过）；全可收口天花板 =
  分母 − 挂账池 12,889（M0 口径 12,879 + M1 iso-2022-jp 流式模型簇 10）−
  pin 漂移带 25 − API 结构残差 3 − encode 契约挂案 1 = **1,139,476
  （98.86%）**（分母滚动时按 evidence 口径重算）；36 号零回归门全绿
  （legacy 扫描恒等 + 新面干跑 + weekly 分片计数对账）；30/31 号通道零
  改动；wpt-suites.csv 回填数据行。

**DC-4 数值门决策规则（立项即定；M0 判定为第二档）**：M0 甄别后按地址池
大小分档——

- 地址池（导入即得 + 引擎可修）≥ 100k 子测试：全量推进，数值门 = 地址池
  × 一期同类族通过率（M0 实测后填入）。
- 地址池 < 100k：降级为收尾 goal，只做 DC-2（根面 + streams），legacy-mb
  剩余面甄别后挂账，富余产能转 css-testharness 二期（见
  `docs/goal/archive/css-testharness-compat.md` 挂账清单）。

**M0 实测落定**：地址池 = 可导入集 25 + 引擎可修 0（M0 干跑 3F 全为在册
SAB/transferred 残差；M1 干跑 big5-encoder 13P 无编码器表缺口，
iso-2022-jp-encoder 10F 为流式模型簇归挂账）<< 100k → 第二档生效。

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

## 里程碑（v1.1 收尾口径）

- ~~**M0**~~ **已完成（2026-10-31）**：对账甄别 + 数值门落定（DC-1，
  evidence 已落）。
- ~~**M1**~~ **已完成（2026-10-31）**：可导入集入通道（DC-2）——36 号
  通道扫描域加 `encoding` 根；big5-encoder 13P 新证，iso-2022-jp-encoder
  2P/10F（10F 流式模型簇挂账）；weekly 两分片接线。
- ~~**M2/M3**~~ **撤销（降级）**：legacy-mb 无未导入案面；引擎可修簇 0；
  挂账池 12,889 记 evidence 分族账本，不重开。
- **M4**：coverage 复测收口（DC-4 数值门 ≥ 98.8%，天花板 1,139,476）+
  wpt-suites.csv 汇总行。

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
