# DC-2 · Encoding worker 变体面收口（2026-11-10）

接[缺口定量](dc2-encoding-gap-2026-11-10.md)：worker 变体面 ~10,900 的通道组装落地，
runner commit `f57d91099`。原始 JSON `/tmp/encoding-worker-variants.json`（71 键）。

## 实现（`CorpusRunOptions.worker_variants` 门，默认关）

- 分道判定：`.any.js` 源**无 `META: global=` 缺省跑**或含 worker/dedicatedworker
  token 时追加 dedicated worker 腿——上游 serve.py `AnyWorkerHtmlHandler` 对
  `.any.js` 无条件双变体服务，manifest global= 缺省 window+dedicatedworker
  （writing-tests/testharness.html#test-globals）。实证 8c04fb8d43 三家 union
  37 个 encoding worker 变体基文件全部为「无 META 或 window,worker token」形态；
  `global=window`-only（single-byte-decoder）分母无 worker 变体不跑——
  初版判定（仅 worker token 跑）会漏掉无 META 的两大头（api-invalid-label
  3,421 + fatal-single-byte 7,168 = 10,589，占分母面 90%）。
- 上报键：worker 腿独立报 `.any.worker.js`（对账 `_VARIANT_RULES` 映射
  `.any.worker.html`），与 window 腿 `.any.html` 分子互不覆盖；
  `run_any_js_dedicated_case_as` 参数化（case_key 与 worker_url 分离），
  workers 通道既有 `.any.js` 键台账不变。
- 仅 encoding 通道开启门；window-only 源、sharedworker/serviceworker 专属变体不跑。

## 结果与对账

| 项 | 值 |
|---|---|
| 本地全量 | 71 键（36 window + 35 worker）23,814 子测试，23,800 过 |
| worker 腿 | 35 键 11,739 子测试（fatal-single-byte worker 腿 7,168 全 Pass，0.3s） |
| 同口径对账（coverage，仅 encoding JSON） | 12,072 → **23,800 过（+11,728）** |
| encoding 组总账（11-09 快照缺口 12,922） | 收口后残余 ~1,194（12,922 − 11,728） |
| worker 变体分母面 11,821 残余 | ~72（见下） |

单案三态验证：worker token / 无 META / window-only 各归其位；workers 通道
键名冒烟不变。`cargo test -p zero-wpt-runner` 318 passed / 0 failed，
clippy `-D warnings` 干净。

## 残余构成（worker 面 ~72）

- **worker 腿新暴露引擎缺口 8**：encodeInto BigInt64Array/BigUint64Array
  `self[type] is not a constructor`（worker 全局面缺 BigInt typed array，4）、
  backpressure `self[streamClass.name] is not a constructor`（worker 全局面缺
  某流构造器，4）。
- 已知引擎缺口跨腿重复：textdecoder-copy SAB 1×2 腿、decode-utf8 detached 2×2 腿。
- 划界/缺语料：idlharness.any.worker（57，net-api 先例 skip）、
  encodeInto-tail-unmodified（1，语料缺失待补拉）。

## 对目标净增的账

test262 ~51,250 + encoding weekly 自动 +10,925 + worker 变体面 **+11,728**
≈ **73,903 / 76,000**，余 ~2,100：wasm 缺口 2,803（未定量，DC-2 收尾项）
+ encoding 残余 ~1,194（iframe 面 ~330 / 划界 / 引擎小项）。

## 待办

- wasm 缺口 2,803 同口径定量（本地 wasm 通道实跑 vs 分母）
- encoding 缺语料 2 源补拉（encodeInto-tail-unmodified、json-document-utf8）
- worker 全局面缺口（BigInt typed array / 流构造器）归簇评估
