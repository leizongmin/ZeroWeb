# M2-S1 — Request/Response.blob() 与 XHR blob type 直设（2026-10-29）

**切片**: M1 归因档收敛排序第 1 位——mimesniff `(Request/Response)` 腿 263F +
xhr overridemimetype-blob 7F，同根因。

## 根因

Blob.type 翻转（一期 fileapi-compat 2026-10-11 仲裁：构造器 type 选项走 FileAPI
范围判据+ASCII 小写化）后，三个**内部创建 Blob**的调用点把 fetch/xhr 规范要求
的 parse-serialize MIME type 传进了构造器 `type` 选项，被构造器判据二次加工：

- 参数值大小写丢失（`charset=GBK` → `gbk`）；
- 含非常量字节集字符的类型整型置空（`test=ÿ` → `""`）。

规范语义（判据只作用于构造器 `type` 选项，不作用于内部创建）：

- fetch spec [#dom-body-blob](https://fetch.spec.whatwg.org/#dom-body-blob)：
  blob() 的 Blob type = `get the MIME type`（extract a MIME type：逐值 parse、
  essence 小写、参数值原样、failure → 空串）。
- xhr spec [#the-response-attribute](https://xhr.spec.whatwg.org/#the-response-attribute)：
  blob responseType 的 type = final MIME type（override 逐字使用）。

## 修改

| 文件 | 调用点 | 改法 |
|---|---|---|
| `crates/engine/src/js_dom_shim/part01.js` | `Request.prototype.blob` | `new Blob([b])` 无 type 构造后 `.type` 直设 `_zwBodyMimeType(headers)`（null → 保持空串） |
| 同上 | `Response.prototype.blob` | 同上 |
| `crates/engine/src/js_dom_shim/part03.js` | XHR `responseType === 'blob'` | 同上（`finalMime` 直设；override 面保大小写） |

构造器判据（`_zw_blob_type`）与 `Blob`/`File` 构造器面零变化——FileAPI 语料面
不受影响。formData() 两个消费点的 File/类型路径不在本簇（语料无钉），未动。

## 验证

- 全通道 A/B：`netapi-m2s1-r3.json` vs r2 基线逐 subtest 对照（脚本
  `.acceptance/net-api-compat/diff_subtests.py`，本地诊断产物不入库）。
- 结果：见下方「r3 对照」节。

## r3 对照（netapi-m2s1-r3.json vs r2 基线）

| 域 | r2 P/F | r3 P/F | ΔP |
|---|---|---|---:|
| mimesniff | 1033/865 | **1296/602** | **+263** |
| xhr | 575/19 | **582/12** | **+7** |
| fetch/api | 1355/573 | 1354/574 | −1 |
| url / streams / eventsource / fetch-other / websockets | — | — | ±0 |

- **+270 修复**：mimesniff `(Request/Response)` 腿 263 全数翻红（残余 602 =
  M1 归因预测的上游语料冲突记账面，精确命中）；xhr overridemimetype-blob
  7（该案 77/77 收口）。
- **−1 记账**：`response-consume.html`「from URLSearchParams to blob」——见
  下节。
- 其余域逐 subtest 零变化；websockets 376/377、streams 906/1018 持平。

## 已知差异记账

- `fetch/api/response/response-consume.html`「Consume response's body: from
  URLSearchParams to blob」1 腿：语料期望派生 CT 面 blob type 为**小写**
  `charset=utf-8`，而 `fetch/api/body/mime-type.any.js` 同浏览器绿面钉派生 CT
  **头面**为大写 `charset=UTF-8`（extract-from-headers 语义下两者互斥——
  parse-serialize 保参数大小写）。
  **记账证据升级（M2-S3c 轮复核，2026-10-29）**：原「浏览器靠非规范双字面量
  实现调和」的推测不成立——张力是**双向实证**的，任何单一实现三面不可兼得：
  - C 面（隐式 body init CT 经 blob() 小写，response-consume:230）：
    Chrome latest（wpt.fyi run 5077221233131520，rev 5ccef26dd9）
    response-consume.html **40/40** 全绿（分母与本引擎逐 subtest 同构 40）；
  - B 面（显式头参数值大小写经 blob() 保留，mimesniff parsing.any
    (Request/Response) `CHARSET=GBK` → `text/html;charset=GBK`）：
    Chrome latest parsing.any.html 仅 **712/1898**——语料腿算术（B/F 955 +
    R/R 942 + parent 1 = 1898，harness 与 pin 逐字同源排除漂移）推得 R/R 腿
    至少挂 231，GBK 保留面不在 Chrome 绿面；本引擎按 spec parse-serialize
    实现在该面 +263 全翻。
  - Chrome 以「丢 B 得 C」、本引擎以「丢 C 得 B」各执一端，spec 文本
    （parse-serialize 保参数原样）支持 B 面故本引擎选择不变；该腿保持 1F
    记账，归**语料内部张力**（与 M5-S5 trade-off 家族同性质）。
