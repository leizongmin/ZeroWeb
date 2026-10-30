# M0：剩余族对账甄别 + DC-4 数值门落定

日期：2026-10-31。结论先行：**encoding 域 1,152,394 子测试全部对账闭合，无
「未导入案面族」；立项时依据的 coverage-2026-10-30「缺口 573,911」经甄别为
CI 捕获假低 + 已收口账被分片失败掩埋，真实地址池 = 可导入集 25 子测试
（2 文件，36 号同款 href 面；引擎可修簇 0）<< 100k 决策门——按立项规则
第二档降级为收尾 goal（只做 DC-2 根面可导入集 + coverage 复测收口），
legacy-mb 剩余面与文档管道面全数落挂账清单。**

## 数据源

- wpt.fyi aligned stable 三家 summary（`tests/wpt-runner/wptfyi-cache/`，
  chrome 156.0.8078.80 / firefox 159.0 / safari 27.2 @ ce5ca39646，与
  coverage-2026-10-30 快照同 run）——上游全量案面清单与逐文件子测试分母
  （每路径三家 `c[1]` 最大值，query 变体切片求和，口径同 `wptfyi_gap.py`
  `build_universe`）。
- 一期收口账：30 号（12,072/12,075）、31 号（decode 360,431 = 100%P）、
  36 号（encode 766,958/766,959）。
- 本地语料（pin `315976933`，lib.sh `WPT_REV`）与干跑产物（本文「干跑基线」）。

## 分族账本（224 案文件 / 1,152,394 子测试 = 立项分母逐位吻合）

逐文件 disposition 见
[2026-10-31-m0-per-file-disposition.json](2026-10-31-m0-per-file-disposition.json)。

| bucket | 文件 | 子测试 | disposition |
|---|---:|---:|---|
| encode-face（`*-encode-form/-href/-encoder`） | 68 | 766,959 | **已收口**（36 号，99.9999%；1 契约挂案 = data: iframe Timeout） |
| decode-face（`*-decode*.html` 文档族） | 34 | 360,431 | **已收口**（31 号，100%P） |
| api-window（`.any.js` 窗口腿） | 36 | 12,100 | **30 号通道在收**（干跑 12,072P/3F 与收口账逐字吻合，见下） |
| worker-leg（`.any.worker.html`） | 36 | 11,821 | 挂账：dedicated-worker 腿（workers 通道 re-entry 评估，本 goal 不实施） |
| sw-leg / sharedworker-leg | 15+15 | 316+316 | 挂账：runner 无 ServiceWorker/SharedWorker 桥（goal-30 残差同口径） |
| window-form（`.window.js` 形态 ×4） | 4 | 342 | 挂账：全部依赖 `.py` 服务器端点 + iframe `contentDocument`（逐文件核实，见下） |
| document-face（文档级嗅探/编码耦合 ×13） | 13 | 52 | 25 可导入（DC-2，big5/iso-2022-jp-encoder href 面，36 号通道）+ 7 文档管道面挂账 + 19 iframe 面 + 1 SAB/https 挂账（逐文件源码核实，见「可导入集」节） |
| idlharness（4 腿） | 1 | 57 | 挂账：WebIDLParser build 期生成资产（net-api/timing 先例） |
| 非案面（ref 页/crashtest） | 2 | 0 | 不入账 |
| **合计** | **224** | **1,152,394** | |

「未导入族」甄别结论：legacy-mb 三目录 + schinese 上游 1,206 文件 = decode
34 + encode 68 案面 + 夹具/helper（chars/errors fixture、index/decoder/encoder
.js、`.headers` sidecar，无子测试）+ gbk/gb18030-decoder `.any` 双腿（窗口腿
30 号在收、worker 腿挂账）——**无一文件处于「导入即可跑而未跑」状态**。

## coverage-2026-10-30「缺口 573,911」归因（三点分解）

快照 zw 578,483 = decode 360,431 ✓（mb-decode 分片完整）+ encode 部分捕获
~216,905/766,959 + 根面/streams 部分捕获 790/12,072 + schinese 窗口腿 660−357。
对照真实账（一期收口 + 本轮干跑）：

1. **form-encode 分片预算超限**：编码域 CI ≈ 本地 3–4× 负载，7200s 旧预算
   只捕 encode 面 28%。已修：2167d9ca6（六分片重划 + 10800–19800s 提额）、
   12deb8557（/common/blank.html 补拉 + case_timeout 600s）。
2. **根面/streams 分片贡献假低**：`net` 分片（time_limit 3600s）只捕 790，
   实际通道全量 12,072——本轮干跑逐字复现 30 号收口账证伪（见下节）。
3. **分母侧滚动**：wpt.fyi 对账 rev（ce5ca39646）较语料 pin（315976933）上游
   滚动，`textdecoder-mistakes` +21、`iso-2022-jp-decoder` +4（逐案对账，
   干跑 per-case 表 −25 全落在该两案）——记 **pin 漂移带 25 子测试**，非引擎
   缺口；FORCE 续拉（同 pin）不消除，pin 推进属全局语料决策不在本 goal。

93cad26e8（快照 topdir 塌方断回写护栏）已防此类假低再覆盖账本。

## 干跑基线（通道复用，文本 + JSON 双形状）

- 命令：`./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1500
  -- ./target/release/zero-wpt-runner testharness-encoding [--json]`
  （二进制 c38f4d336 态 release 重建）。
- **文本臂：12,072P / 3F / 0T，exit 1（残差设计内退出码）——与 30 号收口账
  （12,072/12,075 = 99.98%）逐字吻合，零回归**。3F 与在册残差逐一对应：
  `textdecoder-copy` SAB ×1、`streams/decode-utf8` transferred chunk ×2。
- JSON 臂：per-case 计数见
  [2026-10-31-m0-dryrun-per-case.json](2026-10-31-m0-dryrun-per-case.json)
  （36 案；35 案与 universe 分母精确相等，漂移仅上文 −25 两案）。
- **envelope 注记**：本机当前该通道墙钟 ~15–25 分钟浮动，Makefile 缺省
  900s 可能在打印段截断（exit 2；三轮复跑定位为负载浮动非挂起——同命令
  1500s 轻载完整收口）；CI `net` 分片 3600s 预算内。后续轮次本地全跑带
  `TIME_LIMIT=2400`。

## 可导入集（DC-2 实施面，M1）

逐文件源码核实（本地语料 2026-10-31 读取）：

| 文件 | 子测试 | 依据 |
|---|---:|---|
| `encoding/big5-encoder.html` | 13 | inline `a.href` URL 查询序列化面（`<meta charset=big5>` + document 编码）——与 36 号已收的 gbk/gb18030 `*-encoder.html`（49P/254P）同机制；36 号谓词 `form_url_encoding_case_skipped` 的 `-encoder.html` 放行规则既有，仅根面目录不在其扫描域。**注意**：big5 目录无同名案（`big5-enc-ascii.html` 为纯 ASCII 输入，未压 big5 编码器表），`%A1@` 等非 ASCII 指针案是新证面——干跑若暴露 big5 编码器表缺口则改归「引擎可修」簇 |
| `encoding/iso-2022-jp-encoder.html` | 12 | 同机制；iso-2022-jp 编码器表已经 iso2022jp-encode-form/href 族（36 号 100%P）实证 |
| **合计** | **25** | 36 号通道扫描域加 `encoding` 根（谓词只放行该两文件，bom/sniffing/utf-32/eof/ref 页均不匹配放行规则） |

**挂账修正（初版误记 32 可导入，源码核实后改挂账）**：

- `bom-handling.html`（1）：断言 `document.characterSet` 反映 UTF-8 BOM
  优先于 `<meta charset=windows-1252>`（本地文件 `EF BB BF` 起头实核）——
  引擎文档管道 characterSet 面，30 号 M4 划界挂账 html-syntax-compat。
- `sniffing.html`（1）：无编码声明 → 默认 windows-1252；div 字节实核为
  `C2 80`（cp1252 的 Â€，UTF-8 解码为 U+0080 即双断言皆败）——引擎文档
  管道默认编码探测面，同上挂账。
- `json-document-utf8.html`（5）：上游滚动新文件，源码甄别定稿**挂账**——
  依赖 `resources/json-charset.py` 服务器端点 + iframe `contentDocument`
  跨文档读取 + JSON MIME 文档解码语义（RFC 8259 §8.1），与
  unsupported-labels.window.js 的端点+iframe 依赖同族。

## FORCE 续拉结果（DC-1，2026-10-31 01:07）

`FORCE=1 bash tests/wpt-runner/scripts/goals/37-encoding-compat-v2.sh`
exit=0（GitHub core 配额窗口重置后执行）。**新文件清单 = 空**：pin
`315976933` 下四目录全量重拉（文件 mtime 全刷新）无增量——根面本地
16 .html 与 pin 一致，**missing=0 对 pin 口径成立**。对 wpt.fyi 对账 rev
（ce5ca39646）口径缺 1 案面 `json-document-utf8.html`（5 子测试）：该文件
晚于 pin 引入，raw master 源码甄别如上，**属 pin 漂移带同源现象，非拉取
缺口**（漂移带合计 25+5=30 子测试，DC-4 口径不变——挂账池已含其 5）。

streams 窗口腿（114 分母 / 112P）已在收（2T = 在册 SAB 残差）；streams 残余
360 = worker/sw/shared 腿 + `.window.js` 挂账面，无新增可导入。

## DC-4 数值门落定（按立项决策规则第二档：地址池 25 < 100k）

- **门**：coverage 复测（全量账本，weekly cron 或本地聚合）encoding topdir
  **通过率 ≥ 98.8%**（对 2026-10-30 分母 1,152,394 即 ≥ 1,138,566 通过）。
  全可收口天花板 = 1,152,394 − 结构挂账池 **12,879**（worker 11,821 +
  sw/shared 632 + window-form 342 + idlharness 57 + doc-iframe 19 +
  文档管道面 7 + SAB/https 1）− pin 漂移带 25（随 pin 推进重估）− API
  结构残差 3（在册）− encode 契约挂案 1 = **1,139,486（98.87%）**；
  可导入集 25 全过后即达天花板。分母随上游滚动时按同口径重算。
- 判读看 subtest 计数与 topdir 行，不看点位 exit code（30 号通道残差退出
  设计内，31/36 号同口径）。
- 零回归门：36 号通道改动（M1 加 `encoding` 根扫描域）后，legacy 七目录
  扫描代码与谓词恒等（仅追加根目录）+ 新面定向干跑；weekly coverage 的
  mb-decode + 六个 form-encode 分片计数对账为常驻全量回归（预算已修，
  12deb8557/2167d9ca6）。30/31 号通道零改动，本轮干跑双臂即其基线臂。
