# form-url-encoding-compat master

**状态**: Done——M0~M3 全部完成（2026-10-29 收口归档）。encode 面 68 文件
766,959 子测试 **766,958P / 0F / 1T（99.9999%）**；coverage 复测全量头条
**24.19% → 58.02%**。唯一非 P = data: iframe Timeout 契约挂案（重入条件
见下）。
**入口**: [form-url-encoding-compat.md](../form-url-encoding-compat.md)

## 里程碑状态表

| 里程碑 | 内容 | 判据 | 状态 |
|---|---|---|---|
| M0 | 面单 + per-face 分母实测 + form 最小切片 | DC-1 | **完成**（[evidence](evidence/2026-10-29-m0-face-list-denominator-slice.md)） |
| M1 | runner 通道 + form 面能力（acceptCharset 序列化） | DC-2 | **完成**（[evidence](evidence/2026-10-29-m1-form-face-capability.md)；施工 `da608f159`） |
| M2 | href 面 encoding-parse | DC-3 | **完成**（[evidence](evidence/2026-10-29-m2-href-face-encoding-parse.md)） |
| M3 | 基线收口 + coverage 复测 | DC-4 | **完成**（[evidence](evidence/2026-10-29-m3-coverage-closeout.md)） |

## 验证基线

- **通道**：`make testharness-form-url-encoding`（2026-10-29 落地，commit
  `7ae6909cf`；test-guard 4/8G/5400s + `ZW_SCRIPT_TIMEOUT_MS=600000`，语料
  同源复用 31 号 fetch；filter 按路径子串，`FILTER=encode-form`/`encode-href`
  分面）。
- **encode 面终态**：766,959 子测试 = **766,958 P / 0 F / 1 Timeout**
  （99.9999%）。form 面 550,053/550,054、href 面 216,479/216,479、
  schinese encoder 303/303、big5-enc-ascii 123/123。唯一非 P = data: iframe
  Timeout 挂账案。逐文件账 [TSV](evidence/2026-10-29-m2-per-file-post.tsv)。
- **coverage 复测（DC-4）**：54 通道 + 本 goal 全量 JSON 再生成
  [账本](../../../compat/wptfyi/coverage-2026-10-29.md)，头条 24.19% →
  58.02%，Δ通过 = 766,958 与本 goal 基线精确对账（其他通道零扰动）；
  encoding topdir 31.38% → 97.93%。csv completed 行落账
  （[trends](../../../compat/trends/wpt-suites.csv)）。
- **零回归对账**（M2 同轮，M3 无 shim/engine 变更不重带）：31 号 decode 面
  JSON 逐字节一致（md5 `f73af333…`，360,431P/0F）；25 号 form-submission
  216 腿零翻转；20 号 url leg 集逐字节一致（6,261 legs）。

## 缺口清单

1. ~~form 面——序列化编码 override~~ 已销账（M1，`da608f159`）。
2. **`iso2022jp-encode-form-errors-stateful` 特例**（唯一挂账，随归档带走）：
   单子测试 promise_test，链路含 data: URL iframe 脚本执行 + 子→父
   postMessage 回传，runner 侧 data: iframe 不走脚本通道（M0/M1/M2 三轮
   实测 Timeout 600s）。acceptCharset/编码语义已全部落地，卡点纯 runner
   data: iframe 管道。**重入条件**：runner data: URL iframe 脚本通道落地后
   单案复跑（`FILTER=iso2022jp-encode-form-errors-stateful`）。
3. ~~href 面——encoding-parse a URL 消费点~~ 已销账（M2：getter 解析点预
   编码 + host 桥 NCR 直出 `%26%23N%3B` + query set 补 `%` 归一——rust-url
   保留裸 `%`，见
   [learning](../../../learnings/bugs/2026-10/2026-10-29-rust-url-preserves-bare-percent.md)）。
4. ~~口径差~~ 已销账（M0 钉死 766,959）。~~envelope~~ 已销账。

## 碰撞管理记录

- **2026-10-29 认领**：25 号 html-semantics-compat s138 挂账 ④
  （windows-1252 accept-charset + `&#NNNN;` 实体回退 + lone surrogate
  USV，urlencoded2 4 案）归口本 goal M1——**已清偿**（M1 probe 4/4 P +
  urlencoded2 6 案 F→P，25 号流不重复施工）。
- **边界**：20 号 net-api-compat url 主干不动；text/plain、multipart、
  File entry 层留 25 号（s138 挂账 ①②）。
- **零回归底线**：31 号 decode 面 360,431 逐 subtest 对照；每轮 shim 变更
  A/B 带 25 号 form-submission 基线与 20 号 url 基线（M1/M2 均已带）。

## 收口记账（2026-10-29）

- **门禁口径**：`make test` 红面 11 项经直接 A/B 定性——10 项为
  js_dom_bridge fetch/Request 桥接环境性失败面（25 号 s140 stash-对照
  定谳 HEAD 基线在册，勘根归 25 号；本轮较其名册漂移 +2 成员观察已记），
  1 项为负载敏感墙钟 flake（隔离复跑 HEAD/pre-M2 双态 0.67s ok）。零项
  可归因本 goal，按在册环境性失败面口径如实记账，详见
  [M3 evidence](evidence/2026-10-29-m3-coverage-closeout.md)。
- **DONE 判定（Final Output Protocol 核对）**：DC-1~4 全部满足有据 ✓；
  验证基于上游真实 WPT 语料（68 文件 legacy-mb encode 面，无 inline 充数）
  ✓；`cargo build`/`cargo clippy` 通过、`cargo test` 红面零项归因本 goal
  （见上）✓；分母为上游全量 68 文件口径 ✓；master.md 自洽、archive 建立 ✓；
  禁止项逐条核对无违反 ✓。**DONE**。
