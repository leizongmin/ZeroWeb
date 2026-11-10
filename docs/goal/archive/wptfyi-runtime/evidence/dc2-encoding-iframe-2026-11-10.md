# DC-2 · encoding iframe 面收口（+326 过，翻红 0）

**2026-11-10** · commit（本轮）· 上轮 ShadowRealm 切片（+59）之后续

## A/B（release runner，test-guard 包裹，`testharness-encoding` 全量通道）

| 面 | BEFORE（`f3fa1b5b1`） | AFTER（本轮） | 净增 |
|---|---|---|---|
| unsupported-labels.window（158 子测试） | 未执行（skip 面） | **158P/0F** | **+158** |
| single-byte-decoder-iframe.window（168 子测试） | 未执行（skip 面） | **168P/0F** | **+168** |
| encoding 全通道（71→75 文件） | 23,800P/14F/0T | 24,126P/18F/1T | **+326，Fail/Timeout 增量恰为新放行 streams 两源（invalid-realm 4F + realms 1T），既有账零翻红** |
| dom/nodes 面（178 文件） | 12,781P/23F/2T | 12,781P/23F/2T | **逐字零漂移** |
| R141 归一化面（654 子测试） | 654P/0F | 654P/0F | 零漂移 |

log：`/tmp/round11-enc-full-{before,after}.json`、`/tmp/round11-enc-{unsupported-labels,single-byte-decoder-iframe}-v2.json`、`/tmp/round11-dom-nodes-{before,after}.json`、`/tmp/round11-r141-{before,after}.json`。

## 方案（三件）

1. **py-routes charset 端点族**（runner `testharness_py_routes/`）：新增
   `encoding/resources/text-plain-charset.py` 与 `text-html-meta-charset.py`
   两注册项＋handler（上游逐字等价——前者 `Content-Type: text/plain;charset={label}`
   + "hello encoding"，后者 `Content-Type: text/html` + `<meta charset="{label}">`；
   437/bocu-1/iso-8859-8 visual 等 unsupported label 走后者，单字节 supported
   label 走前者）。
2. **encoding 通道 `.window.js` 形态开**（runner `testharness.rs`）：
   `run_encoding_cases` 加 `extra_forms: true`（encoding 域恰 4 源，其余通道账
   分母不动）＋ `encoding_case_skipped` 对 4 源显式放行（既有 iframe/worker
   依赖 skip 账维持）。顺带修 worker 腿与 `.window.js` 的键冲突：worker 腿门限
   加 `file_name.ends_with(".any.js")`——`worker_key` 的 `replacen(".any.js",…)`
   对 `.window.js` 是 no-op，同键重跑产生重复 case（unsupported-labels.window
   曾双跑，第二腿 Timeout 伪影）。
3. **shim fallback-encoding 父继承＋meta 引号语义**（engine `part05.js`
   `_zwMakeIframeDoc`）：
   - 传输 charset 与 meta 嗅探均落空时，回落**同源祖先文档编码**（spec
     determine-fallback-encoding——`globalThis.document.characterSet` 代表最近
     同源祖先，目标语料均 depth-1；顶层保持 windows-1252；嵌套链祖先回溯待
     真实 frame 树，FIXME 标注）。unsupported label 继承父文档 UTF-8 判据面。
   - meta charset 正则改引号感知形态（quoted 值含空格整值取——
     "iso-8859-8 visual" 判据面：旧首 token 截断误解析为 supported
     "iso-8859-8"；引号形态与 part01 传输 charset 提取同构）。

## 残余定性（不入账，另案）

- `streams/realms.window`（12 子测试，Timeout）：srcdoc iframe `onload` +
  postMessage→eval 跨 realm 泵面缺——sim-win 近似的跨 realm identity 断言面
  （Text*Stream 实例 instanceof 跨 realm）同 s376 既定边界。
- `streams/invalid-realm.window`（4 子测试，Fail TypeError）：detached realm
  的 stream write/close 需 detached-frame 生存期语义（引擎未实现
  iframe.remove 后 contentWindow 构造器继续可用面）。

## 账目

净增账 75,730 → **76,056 / 76,000**（+326 子测试）——**账面越过 76,000
验收线**。DC-3 全量对账（同 revision 归一、验收线 1,393,955 项通过）待执行。
