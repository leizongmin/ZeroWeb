# M1 / DC-1 — url / mimesniff / fetch 失败簇归因（2026-10-29）

**日期**: 2026-10-29
**套件**: `testharness-net-api` 通道全量（WPT pin 315976933870b34d6ea30e3f6643403edae678ba，
与一期同 pin；test-guard 包裹，`--json` 捕获）
**原始数据**: [netapi-baseline-20261029-r2.json](../../.acceptance/net-api-compat/netapi-baseline-20261029-r2.json)
（本地 .acceptance/，诊断产物不入库；下方数字即其全量汇总）
**归因基线对照**: 一期收口 [2026-10-01-m4-s28-flow-control.json](../../archive/net-api-compat/evidence/2026-10-01-m4-s28-flow-control.json)
逐 subtest 对照 + 一期 M5 切片账本差值核对。

## 二期基线（七域全量）

| 域 | Pass/分母 | 通过率 | Fail | Timeout | 一期终态对照 |
|---|---|---:|---:|---:|---|
| fetch/api | 1355/1928 | 70.3% | 572 | 1 | 一期终态 ≈1339/1928（M4-S28 1336/1918 + M5 净 +3）——**净 +16P** |
| fetch/other | 155/163 | 95.1% | 7 | 1 | 持平 |
| url | 4919/6065 | 81.1% | 1146 | 0 | M4-S28 4317/6058 + M5 +401 + helper 解锁——**零回归（0 leg 倒退，+488 提升）** |
| mimesniff | 1033/1898 | 54.4% | 865 | 0 | M4-S28 100% → **−856 全部为已仲裁 Blob.type 翻转面**（见「上游漂移」） |
| xhr | 575/594 | 96.8% | 18 | 1 | 一期终态 578P/15F/1T——**+2 真回归**（见「跨流转入回归」） |
| streams | 906/1018 | 89.0% | 107 | 5 | M4-S28 905/1020——零回归（0 leg 倒退；文件 −2 语料漂移） |
| eventsource | 35/35 | 100% | 0 | 0 | 一致 |
| websockets | 376/377 | 99.7% | 1 | 0 | 一期终态 376/376——**+1 真回归**（见 DC-3 档） |
| **合计** | **9354/12078** | **77.4%** | 616 | 7 | |

基建注记：本轮 M1 发现本 clone 语料缺 5 个 `/common` helper（utils.js /
get-host-info.sub.js / subset-tests-by-key.js / gc.js / dispatcher.js——一期
M3 收尾轮拉过，gitignored 不随 clone 迁移）。缺 helper 使 57 案整案塌缩为
1F；补拉后 r2 为真基线。**教训**：fresh clone 跑本通道前先核对 `/common`
helper 面（fetch 脚本只拉 goal DIRS，不拉 /common）。

## 归因表（簇 → 根因 → 归口 → 可收敛性）

归口四类：**引擎**（引擎语义/共享 crate 路径，本 goal 修齐面）、**runner 基建**
（fixture 通道/.py 端点，归 runner 域切片）、**语料边界**（runner 结构性不可
执行，记账）、**上游漂移**（语料间冲突/一期已仲裁，记账）。可收敛性：
**高**（单根因大簇）、**中**（逐案小簇）、**不收敛**（记账维持）。

### url（1146F/6065 = 81.1%）

| 簇 | legs | 根因 | 归口 | 可收敛性 |
|---|---:|---|---|---|
| IdnaTestV2.any.js | 759 | **url/idna crate 共享路径**：`url 2.5.8 → idna 1.1.0` 域转 ASCII 走 `ErrorPolicy::FailFast`，已 punycode（xn--）标签解码含 UTS46 有效性瑕疵即整域拒绝；WHATWG spec「domain to ASCII」非 strict 路径自 2023 起 **IgnoreInvalidPunycode=true**（无效 punycode 标签原样保留、不失败）——chromium/ff/safari 全过，本引擎抛 `Invalid URL`（`urlprobe` 直证 crate 层：`https://xn--ab-j1t/x` → ERR invalid international domain name） | 引擎 | **高**——单根因；收敛点 = `js_dom_bridge.rs` `parse_url_to_json`/`set_url_part` 解析失败回退路径（crate 行为不满足时按 spec 语义预处理 xn-- 标签再重试），正常路径零行为变化 |
| url-origin 11 + a-element-origin 13 | 24 | 同上（IDNA 行，`xn--pokxncvks` 族）+ `<>\u0000` origin 空串单案 | 引擎 | 高（随上簇） |
| url-setters-stripping.any.js | 54 | host/hostname setter 后 C0 控制符保留面——chromium bug-compat（spec 已改回剥除，chromium 保留；一期记账「url crate 不可表达」） | 引擎 | 中——需 shim setter 序列化后处理（非 crate 路径），逐形态甄别 |
| urlencoded-parser.any.js | 28 | **BOM（U+FEFF）丢失**：USP 构造键/值应保留 BOM（`%EF%BB%BFtest` → key `﻿test`），现剥除——USVString 通路 BOM 处理误伤 | 引擎 | 高 |
| urlsearchparams-constructor + misc 单案 | ~11 | USP/URL 边缘（逐案甄别） | 引擎 | 中 |

### mimesniff（865F/1898 = 54.4%，单案 parsing.any.js）

| 簇 | legs | 根因 | 归口 | 可收敛性 |
|---|---:|---|---|---|
| `(Blob/File)` 腿 | 602 | **上游漂移（一期已仲裁）**：本腿钉 MIME parse-serialize（`CHARSET=GBK` 保大小写、参数值按需加引号），FileAPI 语料钉范围判据+小写化——两者互斥；chromium 实测与 FileAPI 一致（fileapi-compat 2026-10-11 M2-P1 仲裁在案）。chromium 自身亦 fail 本腿 | 上游漂移 | **不收敛**——修齐将打翻 FileAPI corpus（上游语料间冲突，维持记账） |
| `(Request/Response)` 腿 | 263 | **引擎语义（翻转附带损伤）**：fetch spec「extract a MIME type」要求 Request/Response body 的 `.blob().type` 返回头部 Content-Type 的 parse-serialize 结果；Blob.type 翻转后内部建 Blob 走构造器判据（范围判据+小写化）→ 参数值大小写丢失（`gbk`）、非常量字节集类型整型置空（`test=ÿ` → ""） | 引擎 | **高**——单根因；内部 Blob 类型直设绕过构造器判据（FileAPI 判据只作用于构造器 `type` 选项，不作用于内部创建），连带 xhr overridemimetype-blob 7F 同根因 |

### fetch/api（572F+1T/1928 = 70.3%）

| 簇 | legs | 根因 | 归口 | 可收敛性 |
|---|---:|---|---|---|
| request-cache 族（default/conditional/force-cache/reload/no-store/no-cache/only-if-cached）+ gc/utils 依赖案 | ~110 | `.py` 端点语义（HTTP 缓存行为）+ `/common/utils.js` 依赖的 token 协同面——一期 M1 起记账「goal 排除 HTTP 栈本体」 | runner 基建 | 中——runner 域 fixture 切片（缓存语义端点） |
| header-values-normalize 36 + request-headers(basic) 24 + header-values | ~62 | 服务端回显端点（inspect-headers 跨源/UA 面）+ 头值 normalize/strip 语义混合 | runner 基建 + 引擎 | 中——先 fixture 解锁再甄别语义残量 |
| cors-filtering 19 | 19 | 跨源 inspect 回显端点缺失（Failed to fetch 整案） | runner 基建 | **高**——fixture 单点解锁 |
| Request 构造/流错误面（request-error 18 + request-structure 14 + request-init-stream 13 + forbidden-method 6 + request-disturbed 6 + request-init-002/003 ~10） | ~67 | 构造器 WebIDL/错误面语义（did not throw 族） | 引擎 | **高**——逐簇修齐 |
| Response 消费/静态面（response-static-json 11 + textstream 11 + headers-record 10 + response-error 10 + text-utf8 10 + response-null-body 10 + response-from-stream/request-upload ~19） | ~71 | 消费/charset/null-body/UTF-8 解码语义 + 部分端点依赖 | 引擎为主 | **高** |
| integrity.sub 16 | 16 | SRI 哈希校验缺位 + payload 端点 | 引擎 + runner 基建 | 中 |
| status.h2 7 + request-upload.h2 | 8 | HTTP/2 origin 面（runner 无 h2） | 语料边界 | **不收敛**——通道 skip 候选（记账） |
| cookies 7 + authentication-basic 4 + credentials 族 | ~15 | cookie jar / Authorization 重定向保留流——一期挂账（storage / security-hardening 邻域） | 跨域记账 | 不收敛（维持一期记账） |
| referrer/preflight-referrer/cors-redirect 端口+Origin 行 | 13 | M5-S5 页锚 http 翻转的端口语义差（期望 `:8000` 端口行）——一期已记账 trade-off 家族 | 上游漂移（一期记账） | 中——随端口语义专项甄别 |
| misc 单案 | ~20 | abort 时序 / formData-on-aborted / scheme-blob invalid type 等 | 引擎 | 中 |

## 一期零回归对账（DC-2 底线，逐 subtest 对照 M4-S28 + M5 账本）

- **零回归域**：url（0 leg 倒退）、streams（0）、eventsource（0）、fetch/other（0）。
- **已记账倒退**：mimesniff −856（Blob.type 翻转，一期仲裁在案，非回归）；
  fetch/api 13 legs（M5-S5 scheme/端口 trade-off，一期记账）；fetch/api 分母 +10
  语料漂移。
- **真回归（跨流转入，收敛轮清偿）**：
  1. xhr/json.any.js「Ensure UTF-16 results in an error」1 leg（预期 json
     responseType 解码错误返 null，现返对象——疑 2026-10-14 后编码链邻域变更）。
  2. xhr/access-control-and-redirects-async.any.js「redirected to a same-origin
     resource」1 leg（CORS 重定向同源跳判定翻转）。
  3. websockets/constructor/016.html 1 leg（non-ASCII query echo，见 DC-3 档）。

## M2+ 收敛排序建议（按可收敛腿量）

1. **mimesniff (Request/Response) blob().type parse-serialize**（263 + xhr 7 连带）
2. **url IDNA IgnoreInvalidPunycode**（~783）
3. **fetch/api Request 构造/Response 消费语义簇**（~138）+ 跨流转入真回归 3 legs
4. runner 基建 fixture 切片（cors-filtering 回显 / request-cache 族 / header 回显）
   ——归 runner 域，本 goal 记账推动
5. USP BOM（28）/ setters-stripping（54）/ integrity（16）独立小片

预期收敛后：mimesniff 865→602（上游漂移记账）、url 1146→~360、fetch/api
572→~300（含 runner 基建归口）。
