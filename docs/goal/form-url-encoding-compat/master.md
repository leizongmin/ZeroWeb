# form-url-encoding-compat master

**状态**: Active——M1 完成（2026-10-29，DC-2 判据全落）；M2 href 面施工中
**入口**: [form-url-encoding-compat.md](../form-url-encoding-compat.md)

## 里程碑状态表

| 里程碑 | 内容 | 判据 | 状态 |
|---|---|---|---|
| M0 | 面单 + per-face 分母实测 + form 最小切片 | DC-1 | **完成**（[evidence](evidence/2026-10-29-m0-face-list-denominator-slice.md)） |
| M1 | runner 通道 + form 面能力（acceptCharset 序列化） | DC-2 | **完成**（[evidence](evidence/2026-10-29-m1-form-face-capability.md)；施工 `da608f159`） |
| M2 | href 面 encoding-parse | DC-3 | **施工中**（host 桥 query mode 已预留；shim A/AREA getter 消费点待落） |
| M3 | 基线收口 + coverage 复测 | DC-4 | 未启动 |

## 验证基线

- **通道**：`make testharness-form-url-encoding`（2026-10-29 落地，commit
  `7ae6909cf`；test-guard 4/8G/5400s + `ZW_SCRIPT_TIMEOUT_MS=600000`，语料
  同源复用 31 号 fetch；filter 按路径子串，`FILTER=encode-form` 分面）。
- **encode 面基线（M1 后）**：766,959 子测试 = **550,178P / 216,780F /
  1 Timeout**（71.7%）。form 面 550,053/550,054（唯一非 P = data: iframe
  Timeout 挂账案）；href 面 216,780 F 待 M2。逐文件账
  [TSV](evidence/2026-10-29-m1-per-file-post.tsv)。
- **M1 零回归对账**（同轮）：31 号 decode 面 360,431P/0F 逐位一致；25 号
  form-submission-0 A/B 翻转 11 legs 全为挂账④等价面 F→P/Timeout→4P、
  P→F 0；20 号 url 域 5,924P/141F（收口 169F − 20 号 M2-S17 USP BOM 28F，
  url-setters 13 与单案面 128 分布同，零 P→F）。
- 零回归参照：31 号 decode 面 360,431P/0F；25 号 form-submission 与 20 号
  url 基线每轮 shim 变更同带。

## 缺口清单

1. ~~form 面——序列化编码 override~~ 已销账（M1，`da608f159`）。
2. **form 面——`iso2022jp-encode-form-errors-stateful` 特例**（挂账）：单
   子测试 promise_test，链路含 data: URL iframe 脚本执行 + 子→父
   postMessage 回传，runner 侧 data: iframe 不走脚本通道（M0/M1 实测
   Timeout 600s）。acceptCharset 已落地，卡点纯 runner data: iframe 管道
   ——非编码语义，M3 收口按契约挂账或随 runner 能力顺带解锁。
3. **href 面——encoding-parse a URL 消费点**（M2，施工中）：`a.href`
   setter 存 raw、A/AREA 组件 getter 经 `__zw_parse_url`（rust-url UTF-8）
   解析（part03.js:25742）。实现路径 = getter 解析点前以
   `document.characterSet`（非 UTF-8 时）预编码 query 交 rust-url：
   首 `?`（须先于首 `#`）切片，query 段走既有 host 桥
   `__zw_text_encode_percent(enc, q, 'query')`，fragment 段原样（URL
   standard fragment 恒 UTF-8 percent-encode）。host 桥两处待补：NCR 错误
   路径按 spec 直接出预编码 `%26%23N%3B`（现出 `&#N;` 字节过 set——form
   mode 输出等价，query mode 会漏编 `&` `;`）；
   `big5-enc-ascii`/`*-encoder` 同链路覆盖。
4. ~~口径差~~ 已销账（M0 钉死 766,959）。~~envelope~~ 已销账。

## 碰撞管理记录

- **2026-10-29 认领**：25 号 html-semantics-compat s138 挂账 ④
  （windows-1252 accept-charset + `&#NNNN;` 实体回退 + lone surrogate
  USV，urlencoded2 4 案）归口本 goal M1——**已清偿**（M1 probe 4/4 P +
  urlencoded2 6 案 F→P，25 号流不重复施工）。
- **边界**：20 号 net-api-compat url 主干不动；text/plain、multipart、
  File entry 层留 25 号（s138 挂账 ①②）。
- **零回归底线**：31 号 decode 面 360,431 逐 subtest 对照；每轮 shim 变更
  A/B 带 25 号 form-submission 基线与 20 号 url 基线（M1 已带，见上）。

## 下一步（M2 检查单）

1. host 桥：`text_encode_percent` query mode NCR 错误路径改 spec 直出
   `%26%23N%3B`（form mode 输出不变，等价路径）；补 unit 案（euc-jp/
   iso-2022-jp query 期望、NCR query 形态）。
2. shim：part03.js A/AREA 组件 getter 解析点（`__zw_parse_url` 调用前）增
   encoding-parse 预编码——`document.characterSet` 归一非 UTF-8 + raw 含
   `?`（先于 `#`）才分派；UTF-8/无桥路径零变化；BASE/LINK 与组件 setter
   不动（最小消费面，20 号 url 零扰动）。
3. 判据：href 面 216,905 全收敛（预期 encoding_rs 编码表反向近 100%；
   `*-encoder` 301F 含 GB18030-2022 面）；31 号 decode + 25 号
   form-submission + 20 号 url 零回归同轮带；form 面 550,053 不动。
4. 通道基线更新落 evidence，M3 转 coverage 复测。
