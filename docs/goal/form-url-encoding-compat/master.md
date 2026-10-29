# form-url-encoding-compat master

**状态**: Active——M1+M2 完成（2026-10-29，DC-2/DC-3 判据全落）；M3 收口待启动
**入口**: [form-url-encoding-compat.md](../form-url-encoding-compat.md)

## 里程碑状态表

| 里程碑 | 内容 | 判据 | 状态 |
|---|---|---|---|
| M0 | 面单 + per-face 分母实测 + form 最小切片 | DC-1 | **完成**（[evidence](evidence/2026-10-29-m0-face-list-denominator-slice.md)） |
| M1 | runner 通道 + form 面能力（acceptCharset 序列化） | DC-2 | **完成**（[evidence](evidence/2026-10-29-m1-form-face-capability.md)；施工 `da608f159`） |
| M2 | href 面 encoding-parse | DC-3 | **完成**（[evidence](evidence/2026-10-29-m2-href-face-encoding-parse.md)） |
| M3 | 基线收口 + coverage 复测 | DC-4 | 未启动 |

## 验证基线

- **通道**：`make testharness-form-url-encoding`（2026-10-29 落地，commit
  `7ae6909cf`；test-guard 4/8G/5400s + `ZW_SCRIPT_TIMEOUT_MS=600000`，语料
  同源复用 31 号 fetch；filter 按路径子串，`FILTER=encode-form`/`encode-href`
  分面）。
- **encode 面基线（M2 后）**：766,959 子测试 = **766,958 P / 0 F /
  1 Timeout**（99.9999%）。form 面 550,053/550,054、href 面 216,479/216,479、
  schinese encoder 303/303（M1 挂的 gb18030/gbk-encoder 硬编码 301F 随 NCR
  直出翻转）、big5-enc-ascii 123/123。唯一非 P = data: iframe Timeout 挂账
  案。逐文件账 [TSV](evidence/2026-10-29-m2-per-file-post.tsv)。
- **M2 零回归对账**（同轮）：31 号 decode 面 JSON 逐字节一致
  （md5 `f73af333…`，360,431P/0F）；25 号 form-submission 216 腿零翻转
  （+16 语料新增腿：10P dialog + 6T target 既有挂账类）；20 号 url pre/post
  leg 集逐字节一致（6,261 legs，零 P→F）。
- 零回归参照：31 号 decode 面 360,431P/0F；25 号 form-submission 与 20 号
  url 基线每轮 shim 变更同带。

## 缺口清单

1. ~~form 面——序列化编码 override~~ 已销账（M1，`da608f159`）。
2. **`iso2022jp-encode-form-errors-stateful` 特例**（唯一挂账）：单子测试
   promise_test，链路含 data: URL iframe 脚本执行 + 子→父 postMessage 回传，
   runner 侧 data: iframe 不走脚本通道（M0/M1/M2 三轮实测 Timeout 600s）。
   acceptCharset/编码语义已全部落地，卡点纯 runner data: iframe 管道——
   M3 收口按契约挂账或随 runner 能力顺带解锁。
3. ~~href 面——encoding-parse a URL 消费点~~ 已销账（M2：getter 解析点预
   编码 + host 桥 NCR 直出 `%26%23N%3B` + query set 补 `%` 归一——rust-url
   保留裸 `%`，见
   [learning](../../learnings/bugs/2026-10/2026-10-29-rust-url-preserves-bare-percent.md)）。
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

## 下一步（M3 检查单）

1. coverage 复测：`scripts/check-coverage.sh` 按 goal 影响面（engine
   js_dom_bridge/js_dom_shim 消费点）复测并入账，与 M0 基线对照。
2. 全量头条增量入账：encode 面 550,178P/216,780F/1T → 766,958P/1T（其中
   legacy-mb 归口的 decode+encode 语料 wpt.fyi 口径对照）。
3. `iso2022jp-encode-form-errors-stateful` Timeout 按契约挂账落 Final
   Output Protocol（或 runner data: iframe 能力顺带解锁后复跑）。
4. goal 入口文档状态面收口（DC-4 判据核对 + Final Output Protocol 输出）。
