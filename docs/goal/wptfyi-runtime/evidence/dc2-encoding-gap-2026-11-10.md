# DC-2 · Encoding 缺口定量（2026-11-10）

对账口径：wpt.fyi aligned stable 8c04fb8d43（三家缓存 `tests/wpt-runner/wptfyi-cache/`，
2026-11-09 22:47 fetch）vs 本地 `testharness-encoding` 全量实跑（release runner
`/tmp/encoding-baseline.json`，36 源路径）。快照演进：2026-11-07 缺口 23,847 →
**2026-11-09 已收敛至 12,922**（weekly cron 自动入账：encoding 组 693 → 11,618 过、
22/95 → 36/95 文件；本目标未动 encoding，是分片与对账管道自身的周度恢复）。

## 缺口构成（11-09 口径 12,922 子测试）

1. **worker 变体面 ~10,900（84%，DC-2 主攻项）**。wpt.fyi 对 `.any.js` 收录
   `.any.html` 与 `.any.worker.html` 两个变体路径；runner 只跑 window scope、
   上报 `.js` 源名，对账变体规则（wptfyi_gap.py `_VARIANT_RULES`）window 优先——
   一条源结果只投 `.any.html` 变体，`.any.worker.html` 分子恒 0。大头：
   textdecoder-fatal-single-byte（7,168 worker 变体）、api-invalid-label
   （3,421 worker 变体）。执行链素材已备：fetch 通道已服务 `.any.worker.js`
   包装源（workers-compat M2 `synthesize_any_worker_source`），worker 结果
   relay 有 webcrypto worker 腿先例——缺「window 页 spawn worker + testharness
   在 worker 全局跑 + 结果回传 + 上报 `.any.worker.html` 名」的通道组装。
   建议按 `CorpusRunOptions` 门模式做通道级 opt-in（其他通道账面不动）。
2. **window.js 案 ~330（需 iframe/realm 管道）**：unsupported-labels.window
   （158）、single-byte-decoder-iframe.window（168）、streams/realms.window（12）、
   invalid-realm.window（4）。四源均为 `createElement("iframe")` 动态子文档面
   （unsupported-labels 每 label 建 iframe 设 charset；realms 面 iframe realm
   语义——与 test262 createRealm 簇同根缺口）。runner `.window.js` 形态开关
   （net-api M1a `extra_forms` + `run_any_js_window_case`）已存在但执行环境缺
   iframe 管道。
3. **已跑未全过 +3**：textdecoder-copy SAB 腿 1（`Modify buffer after passing
   in (SharedArrayBuffer)` 期望 `@` 得 `""`——SAB 写入可见性）、decode-utf8
   detached chunk 2（`Cannot perform Construct on a detached ArrayBuffer`——
   transferred chunk 容错语义）。×4 变体对账仅计 window 腿，净收益 +3。
4. **划界排除面 ~1,700（暂不动）**：嗅探面 bom-handling/sniffing/utf-32×2/
   remove-only-one-bom（`document.characterSet` 文档编码耦合，encoding-compat
   goal 排除、挂账 html-syntax-compat）；big5/iso-2022-jp-encoder（form-url
   通道划界）；idlharness ×4 变体（57×4，net-api 先例 skip）；eof-* reftest 面
   ×4（P,P,P 单测试）；encodeInto-tail-unmodified + json-document-utf8 语料缺
   encodeInto-tail-unmodified + json-document-utf8 语料缺（2 源——2026-11-10
   补拉复核：pin 31597693 两文件 404、universe revision 792c7512e3 为 200，
   上游 pin 后新增，pin 纪律下不可补拉；DC-3 同 revision 归一时自动出分母）。

## 对目标净增的账

- 11-07 基线 → 11-09：encoding 已自动净增 **+10,925**（weekly cron 入账）。
- 目标净增 76,000：test262 ~51,250 + encoding 自动 +10,925 ≈ 62,175，
  余 ~13,825 由 encoding worker 变体面（~10,900，DC-2 主攻）+ window.js/
  杂项 + wasm（缺口 2,803）回收。
- wasm 缺口 2,803 定量未做（下一轮同口径：本地 wasm 通道实跑 vs 分母）。

## 本地实跑明细（36 源 → 51+44 分母变体）

- 根层 74 分母路径：已映射 51（47 全过）、未跑 23；
- streams 47 分母路径：已映射 44（40 全过）、未跑 3（realms/invalid-realm/
  stringification-crash）；
- legacy-mb-schinese 6：已映射 4（encoder 面）、未跑 2（decoder .any 变体，
  源在本地——通道扫描面问题，worker 变体同理）。
- 墙钟：全量 <1min（textdecoder-fatal-single-byte 7,168 子测试单源 0.27s）——
  CI 分片预算充裕。

## 待办

- worker 变体通道组装（encoding 通道 opt-in 门，主攻 ~10,900）——已完成
  （[worker 收口](dc2-worker-variants-2026-11-10.md)）
- wasm 缺口 2,803 同口径定量——已完成（[wasm 收口](dc2-wasm-gap-2026-11-10.md)）
- encoding 缺语料 2 源补拉——已关闭（pin 404，见上文第 4 条复核结论）
- 三 Fail 引擎缺口（SAB 写入可见性、detached chunk 容错）
