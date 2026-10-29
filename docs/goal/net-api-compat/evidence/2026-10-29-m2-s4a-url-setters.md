# M2-S4a — url 组件 setter 语义面（url-setters +123 / setters-stripping +54）（2026-10-29）

**切片**: url 组件 setter 族（`url/url-setters.any.js` + `url/url-setters-stripping.any.js`）。
按 URL spec 组件 setter 语义补齐 shim `URL` 与 bridge `set_url_part`：
no-throw 语义、cannot-have 守卫、host/hostname state override 机、file host
覆盖、opaque path SPACE 归一、hash getter 空串面。

## 结果

| 文件 | M2-S3b 基线 | 本切片后 | Δ |
|---|---:|---:|---:|
| url/url-setters.any.js | 156/279 | **266/279** | +110 |
| url/url-setters-stripping.any.js | 206/260 | **260/260** | +54 |
| url/a-element.html | 823/892 | 834/892 | +11 |
| url/url-constructor.any.js | 829/893 | 840/893 | +11 |
| url/urlsearchparams-delete.any.js | 6/8 | 8/8 | +2 |
| url 域合计 | 2020 | 2208 | **+188** |

setters-stripping（strip-C0/space 后断言）整簇被同一组 setter 语义修复
连带清偿——该簇 54 legs 是 M1 归因档预测的「url 残余 357F」最大单块，
M2-S2 后仍挂账，本切片全部翻绿。

## 根因与实现

### 1. 组件 setter 失败面 no-op（`part02.js` `_setPart`）

URL spec：仅 `href` setter 解析失败抛 `TypeError`；其余组件 setter
（host/hostname/port/username/password 等）解析失败一律 **no-op**，原 URL
不变且不抛。语料 hostile 输入（`https://example.net/@` host 置坏值等）
此前在 shim 抛错或部分应用。`_setPart` 统一按「`raw` 为空 → href 抛、
组件返回」分型。
https://url.spec.whatwg.org/#dom-url-href

### 2. cannot-have-credentials/port 守卫（`js_dom_bridge.rs` `set_url_part`）

username/password/host/hostname/port setter：`this's URL cannot have a
username/password/port`（host 为 null/空串或 scheme `file`）时直接返回。
port setter 空串 → port null；pathname 对 opaque path no-op；search/hash
空串 → 置 null（裸 `?`/`#` 尾语义）。
https://url.spec.whatwg.org/#dom-url-username

### 3. host/hostname state override 机（`set_url_host_override`）

按 spec host state override 实现完整扫描机：`:` 在 bracket 外 →
hostname 失败（no-op）/ host 态提交后进 port 态；分隔符 `/`、`?`、`#`、
special `\` 终止并保留尾段；空 buffer + special（或带 credentials/port）→
host-missing 失败。port 溢出 quirk（语料注记）：`example.com:65536` 端口
越界丢弃但 **hostname 仍生效**（此前整条 no-op 回归 1 leg）。
https://url.spec.whatwg.org/#host-state

### 4. file host 覆盖（`set_url_file_host_override`）

file scheme host setter：buffer 截到首个 `/\?#`，percent-decode，forbidden
host code point 失败，`localhost` 归一空串，compose 后重解析（url crate
`set_host` 对 file 走 EmptyHost/不归一路径，不能直用）。

### 5. opaque path SPACE 归一（`zw_opaque_space_normalize`）

spec opaque path state：结尾 SPACE 后随 `?`/`#` 时写 `%20`（更早 SPACE
保留原样）。对 parse 输入与 base 预归一（`data:space ?query#fragment` →
`data:space%20?query#fragment`），覆盖 parse 与 href setter 两入口。
https://url.spec.whatwg.org/#opaque-path-state

连带增益：parse 入口归一修复 constructor 语料同族 legs——a-element.html
+11、url-constructor.any.js +11、urlsearchparams-delete.any.js +2（USP
delete 后 query 置 null 的序列化面）。

### 6. hash getter 空串面（`url_idna.rs` `UrlComponents`）

hash getter：fragment 为 null **或空串**返空串（与 search getter 同型）。
裸尾 `#` 被 url crate 解析为 `Some("")`——href 序列化保留 `#` 而 hash
getter 归 `""`，两侧不对称此前撕裂 `hash = '#'` 腿。
https://url.spec.whatwg.org/#dom-url-hash

### 7. WebSocket 构造器 fragment presence 判定回归修复（`part06.js`）

item 6 改动连带暴露真回归：ws 构造器 spec step 4（fragment 非 null →
SyntaxError）此前以 `resolved.hash !== ''` 判 presence——依赖旧 getter
「裸 `#` 返 `'#'`」的非对称行为；getter 归空串后 `http://…/#` 漏抛
（Create-invalid-urls 该腿 r4 过程轮 Pass→Fail）。修复：presence 以
`resolved.href.indexOf('#') >= 0` 判定（序列化 href 中首个 `#` 恒起
fragment——query 面 `#` 必转 %23，裸尾 `#` 保留）。shim 全域扫描确认
ws 构造器是唯一「hash getter 判 presence」消费点。
https://html.spec.whatwg.org/#the-websocket-constructor

## 验证

- probe（`testharness-net-api url-setters`，r4）：两文件 266/279 +
  260/260，上表。
- 全通道 A/B（`netapi-m2s4a-r4.json` vs `netapi-m2s3b-r3.json`，逐
  subtest，2026-10-29 06:05 完成）：+259 增益 / **3 回归**（10841 pass）
  ——回归归因：①「CORS invalid integrity」与 ②「cors-preflight … user
  agent」为同树 M2-S3c 切片的显影/交互（归该切片证据档，非本切片）；
  ③ ws Create-invalid-urls「`http://…/#`」为本切片 item 6 连带（item 7
  修复）。本档初稿曾写「0 回归 +164」为运行完成前的推算值，与实际
  运行结果不符，以本节实测为准（过程教训：全通道 A/B 数字必须出自
  已完成的运行产物）。
- 联合终态验证（本切片 item 7 修复 + M2-S3c 全套，同一 working
  tree）：`netapi-m2s3c-r4.json` vs `netapi-m2s3b-r3.json`——见 M2-S3c
  证据档「联合终态验证」节。
- 构建无 warning（`build-m2s4a-r4.log`）。

## 记账（未清偿，13 legs 全部落语料边界/上游 quirk）

1. **xn-- setter ×2**：`host = 'xn--'` 期望 IDNA 失败 no-op，语料期望
   「原 URL 不变」；url crate `set_host("xn--")` 报 IdnaError（no-op 一致）
   但 `example.com` 基座走 pc_lenient_parse 回退路径需 setter 级接线——
   2 legs，IDNA 回退 setter 面挂账。
2. **non-spec `/.` marker 序列化 ×2**：`<non-spec:/.//p>.hostname` 期望
   `non-spec://h//p`（host 非空时 marker 撤除）；url crate 对非 special
   路径 `//` 前缀**无条件**插 `/.` marker（`"//p"` set_path 重序列化回
   `/.//p`），`non-spec:////p` 形态在 url crate 序列化不可达——需脱离
   url crate 序列化层，2 legs 挂账。
3. **pathname 族 ×9**：spec path state 机（含 `/.`/`/..` marker 序列化、
   file 反斜杠分型、path-only URL（`foo:/some/path` host 为空串非
   opaque）pathname 可置空、`a:/` percent-encode 集）未实现——M2-S4b
   候选，9 legs 挂账。

## 记账（承前）

- url crate panic（lib.rs:2881 inverted range，hostile username/password
  触发）仍由 M2-S2 `catch_unwind` 守卫兜住，stderr 噪声非失败面，根因
  挂 runner 域（承 M2-S3b 记账 4）。**观测更新（M2-S3c 轮）**：本切片
  落地后联合运行 stderr panic **0 次**（M2-S3b 基线每轮 2 次）——组件
  setter no-op 分型不再触发 hostile username/password 的 crate panic
  路径；根因排查随之降级（无观测即无入口）。
