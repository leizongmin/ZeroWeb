# form-url-encoding-compat master

**状态**: Active——M0 未启动（2026-10-29 立项，用户决策）
**入口**: [form-url-encoding-compat.md](../form-url-encoding-compat.md)

## 里程碑状态表

| 里程碑 | 内容 | 判据 | 状态 |
|---|---|---|---|
| M0 | 面单 + per-face 分母实测 + form 最小切片 | DC-1 | 未启动 |
| M1 | runner 通道 + form 面能力（acceptCharset 序列化） | DC-2 | 未启动 |
| M2 | href 面 encoding-parse | DC-3 | 未启动 |
| M3 | 基线收口 + coverage 复测 | DC-4 | 未启动 |

## 缺口清单（立项时点，2026-10-29）

1. **form 面——序列化编码 override**：`_encPair` 硬编码 UTF-8（percent-
   encode set 已由 25 号流 s138 对齐 spec），acceptCharset 零消费。核心
   被测语义。归 M1。
2. **form 面——断言链端到端未验证**：GET → target named iframe →
   `iframe.onload` → `contentWindow.location.search` 链路中，导航与 load
   管线已由 s138 落地，search 读回未被 legacy-mb 语料实证。归 M0 最小
   切片。
3. **href 面——encoding-parse a URL 消费点**：全仓 URL 面无文档编码
   fallback；实现路径（shim setter 预编码 query 后交 rust-url）待 M2
   可行性甄别。归 M2。
4. **口径差**：encode 面分母 DC-3（767,673）vs coverage-2026-10-29 余量
   （767,316）差 357，M0 钉死。
5. **envelope**：单文件 ~7,000 async_test 的通道超时面，M1 按 31 号先例
   预评估（`ZW_SCRIPT_TIMEOUT_MS` 提额）。

## 碰撞管理记录

- **2026-10-29 认领**：25 号 html-semantics-compat s138 挂账 ④
  （windows-1252 accept-charset + `&#NNNN;` 实体回退 + lone surrogate
  USV，urlencoded2 4 案）归口本 goal M1——同一能力（acceptCharset
  序列化）的单字节前哨，25 号流不重复施工。
- **边界**：20 号 net-api-compat url 主干不动；text/plain、multipart、
  File entry 层留 25 号（s138 挂账 ①②）。
- **零回归底线**：31 号 decode 面 360,431 逐 subtest 对照；每轮 A/B 带
  25 号 form-submission 基线与 20 号 url 基线。

## 验证基线

待 M0。立项参考数字：encode 面 68 文件，form 46 + href 22；分母
767,673（DC-3 口径）。零回归参照基线：31 号 decode 面 360,431P/0F。

## 下一步（M0 检查单）

1. 68 文件分簇清单（form/href × 四族）+ per-face 子测试分母实测，
   钉死口径差（缺口 4）。
2. form 最小验证集：euc-jp 1 案通道实跑，定位断言链第一断点
   （预期：query 仍 UTF-8 → 断言 Fail 但链路可达），证据落 evidence/。
3. envelope 预评估数据（单案墙钟、chunk 轮转耗时）。
