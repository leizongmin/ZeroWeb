# M2-S3 — fetch/api Request 构造/Response 消费语义簇收敛（2026-10-29）

**切片**: M1 归因排序 fetch/api 簇——Request 构造校验/结构、Headers record
init、Response 消费（textStream/static json）、wire 头合并、流式上传分型。
含跨流转入的 M2-S3b 真回归清偿准备面（xhr json 解析路径定位，修复落 M2-S3b）。

## 根因与实现

### 1. Request 构造校验链（`part01.js` `_zwRequestValidate`）

旧构造器近乎不校验（request-error 4/22、request-init-stream 10/23 起步）。
对齐 spec #dom-request 全链抽为单函数：无 `new` 抛 TypeError；`init.window`
非 null；URL 解析失败/含 credentials；mode/credentials/cache/redirect/
referrerPolicy 枚举（referrerPolicy 缺省 `''`——pinned corpus 面）；referrer
解析（no-referrer/client/空串分型 + **忽略端口的同源判定**——s116「跨源判定
按 host 段」runner 惯例）；方法 token 校验 + normalize 表 + forbidden 大小写
不敏感（TRACK 不在表内不大写化）；navigate 拒绝；no-cors 非 GET/HEAD/POST；
only-if-cached 组合；duplex half 语义；input 体 disturbed/locked；GET/HEAD
带体。校验通过才标 input bodyUsed（构造失败不标 disturbed 面）。

属性改只读访问器（hidden `_zw*` 字段承载；`this.headers` 改 `_zwHeaders`——
own-property 会破坏 sloppy 只读 no-op 面）；`clone()` 复制 guard + tee 流。

### 2. fetch consumer 构造优先（abort/general 16 legs）

`fetch(input, init)` 复用 `_zwRequestValidate`，置于 signal 读取之前；同步抛
转 `Promise.reject`（fetch 恒返 promise）。修 r1 A/B 暴露的 abort/general
回归——该族在基线经旧部分校验通过、构造器重写后失守。preflight 重入
（`_zwPreDone`）跳过校验与流体处理（首轮已校验、流已泵尽，重跑误判 disturbed）。

### 3. Headers record init 逐 trap 对齐（`part02.js` `_fillHeaders`）

`new Headers(record)`：`Reflect.ownKeys` → 逐键 gOPD（跳 undefined/非枚举）→
`String(key)`（Symbol → TypeError）→ 名合法校验**先于** `[[Get]]` → append。
Headers 实例走通用序列路径（去掉 `init instanceof Headers` 分支——instanceof
的 getPrototypeOf trap 先于预期首个 `[[Get]](Symbol.iterator)` 破坏 trap 计数，
headers-record 13/13 面）。

### 4. Body 消费面

- `textStream()`（textstream 14/14）：标记 used 后**直读**字节/泵 reader
  （原经 `_zwConsumeBodyBytes` 的 unusable 守卫自拒）。
- `Response.json`（response-static-json 16/16）：`init.headers` 是否含
  Content-Type **构造前**探测（构造器会给 string 体派生 text/plain）；
  `JSON.stringify === undefined`（Symbol/function）→ TypeError。

### 5. wire 头合并（r1 A/B 11 legs）

`_makeResponseFromWire` 的 `{name: value|[values]}` duck 形直接作 Headers
init → record 路径 `String(array)` ','-join 破坏多值合并（期望 `0, 0` 得
`0,\t 0` 等）。改调用侧转 `[k, v]` 序列（多值拆分对）走 combine 语义。
header-value-combining 6/6、xhr getresponseheader 6/6、
getallresponseheaders-after-reuse 1/1 收口。

### 6. 流式上传分型（request-upload 0→18/19 + h2 4→10/12）

fetch 流体按 runner 端点能力分流（wrapper 层，先校验后分流）：

- `.h2.py` 端点（上游 h2 面）：`_zwReadAllStreamChunks` 泵尽为字节
  （非 BufferSource 块 → TypeError——String/null/number 面余 3 legs 待
  端点语义区分，见记账 2）；`fetch(request)` 标 input bodyUsed。
- 其余 http(s)（上游 h1 面）：TypeError「Streaming upload is not supported
  on HTTP/1.1」（替代旧行为 String(stream) 化垃圾体；已 aborted signal
  优先 AbortError——abort/general 面零回退）。
- data:/blob: 短路体不传输：不泵不拒（h2 feature-detect data: 面零回退）。

runner 新增 `fetch/api/resources/echo-content(.h2).py` fixture（原样回显
请求体；`req.body_bytes`/`body` 双回落）。

### 7. runner html 分支 token 替换（`testharness.rs`）

net-api `.html` 语料此前不做 `{{host}}` 等替换（`.any.js` 分支已做）→
request-init-003 全文件 TypeError「Invalid referrer: http://{{host}}:...」。
html 分支补 `apply_wpt_substitutions`（行数/CSP 位置预计算仍按原文）。

## 验证

- 通道全量 A/B（`netapi-m2s3-r2.json` vs `netapi-m2s3-r2` 前基线
  `netapi-m2s2-r2.json`，逐 subtest）：**0 回归，+160**
  （`abdiff_m2s3.py` REGRESSIONS=0、MISSING=0、EXIT=0）。
- 13 文件语义族聚合：**91/220 → 219/220（+128）**。

| 文件 | base | r2 |
|---|---:|---:|
| request-error.any.js | 4/22 | 22/22 |
| request-init-001.sub.html | 34/53 | 52/53 |
| request-init-002.any.js | 3/8 | 8/8 |
| request-init-003.sub.html | 0/4 | 4/4 |
| request-init-stream.any.js | 10/23 | 23/23 |
| request-disturbed.any.js | 3/9 | 9/9 |
| forbidden-method.any.js | 0/6 | 6/6 |
| request-structure.any.js | 10/24 | 24/24 |
| response-error.any.js | 0/10 | 10/10 |
| response-static-json.any.js | 5/16 | 16/16 |
| textstream.any.js | 3/14 | 14/14 |
| headers-record.any.js | 3/13 | 13/13 |
| request-init-contenttype.any.js | 16/18 | 18/18 |

族外联动收益：request-upload 0→18/19、request-upload.h2 4→10/12、
request-headers 59→60、mime-type 19→20、request-clone/keepalive/request-head/
request-referrer/url-parsing 各 +1（合计通道 +160）。

- 预存注记：通道运行 stderr 的 url crate panic（lib.rs:2881 byte range，
  每轮 2 次）在基线运行日志同现（`netapi-baseline-20261029-r2.run.log`），
  非本切片引入；panic 子进程由 test-guard 按文件重拉，A/B 可比性不受影响。
  根因排查挂 runner 域（与 M2-S2 证据档「serialize_url panic 守卫」同族线索）。

## 记账（未清偿）

1. **request-init-001 余 1 leg**：相对 referrer `/relative/ressource` 对
   端口无消费锚（`http://wpt.test`）解析 → `http://wpt.test/...`，期望
   `http://wpt.test:8000/...`——runner-infra 结构面（anchor 无端口 vs 语料
   substituted 端口），引擎语义正确。
2. **request-upload 余 4 legs**：h1 421 连接复用重试 + h2 421 流式面（需
   connection-pool `network-partition-key.py` fixture 语义）；h2 流块
   String/null/number TypeError 3 legs 现走 h1 端点分型拒绝（待 `.h2.py`
   端点流块校验接通后翻转）。
3. **abort/general 流-中止族余 4 legs**：fetch 入口 signal 已 aborted 时
   流体**同步 cancel(reason)**（`cancel` 回调同步调用 + AbortError 实例
   一致性）——需引擎入口同步取消机制，非本切片。
4. **M2-S3b 真回归 3 legs**（跨流转入，本切片定位未修）：xhr json UTF-16
   （`part03.js:544` json 分支复用 BOM 嗅探解码；spec 为 UTF-8 decode——
   修复点已定位）、xhr access-control-and-redirects-async 同源重定向
   （onerror 提前，redirect follow 链待查）、ws constructor/016
   （windows-1252 页查询串百分号编码）。
