# M2-5 raw 格式别名切片（2026-11-02，无新依赖面）

> `raw-secret` / `raw-public` importKey/exportKey/wrapKey/unwrapKey 格式别名
> 支持——**F 归因扫查误桶更正后确认的无新依赖切片**：该案只触既有算法
> （AES-CTR/CBC/GCM/KW、HMAC、HKDF、PBKDF2、ECDSA/ECDH P-256、Ed25519、
> X25519），零新域算法介入，属拍板前可收敛面（Mission-2 无新依赖面）。
> 对照基准 [m2s4 锚](2026-11-02-m2s4-full.json.gz)；本轮锚
> [m2s5 锚](2026-11-02-m2s5-full.json.gz)。

## 误桶更正（F 归因扫查结论修正）

[2026-11-02 F 扫查](2026-11-02-m2s4-f-attribution-sweep.md) 将
`raw_format_aliases`（新族 2 案 + 旧 tentative 路径 2 案，80 F）按目录桶进
「modern-algos = D1 域门控」（逐算法分布表「其余 684」内），并得出
「无无依赖可修红面」。本轮逐案算法面复核证伪：该案测试文件硬编码算法清单
全部为既有实现面（上文 10 算法），失败原因单一 =
`NotSupportedError: Unsupported importKey/exportKey format: 'RAW-SECRET'
/'RAW-PUBLIC'`（m2s4 锚 4 案 80 子测试同形态）——纯 shim 格式分发缺口。
扫查「无无依赖可修红面」结论相应修正（扫查文档已加更正注记）；
supports-modern / getPublicKey 余红经同法复核维持域门控归因
（测试面算法清单全为新域，无第二例误桶）。

## 规范依据

别名出自 WICG **webcrypto-modern-algos** 规范 KeyFormat 扩展
（`raw-public` / `raw-private` / `raw-seed` / `raw-secret`；w3c/webcrypto
/main 无此值）：

- 既有对称算法（secret 型）：`raw-secret` ≡ `raw`；
- 既有非对称算法（public 型 raw 面）：`raw-public` ≡ `raw`；
- RSA 无 raw 面，两别名均不映射（落既有未知格式拒绝）；
- 类型不匹配语义由既有 raw 分支类型校验承接（EC/OKP export raw 非 public →
  InvalidAccessError，与该规范 ML-KEM export 例同型）。

语料面（`raw_format_aliases.https.any.js` 自述）：对称算法 `raw-secret`
为 `raw` 别名、非对称 `raw-public` 为 `raw` 别名，仅正例断言（import/
export/wrap/unwrap 往返 + `assert_goodCryptoKey` 类型/usage 核对），
无不匹配别名负例——负例行为按上述规范落位，无语料面不扩测。

## 改动（+26/-1，零 Rust 触面）

- `crates/engine/src/js_dom_shim/part01b.js`（+17）：`_zw_normalizeFormatAlias
  (format, algoName)`——family 条件归一（对称面 RAW-SECRET→raw、EC/OKP
  RAW-PUBLIC→raw、RSA 不映射），置 RSA 家族节前；谓词复用
  `_zw_rsaIsFamily/_zw_ecIsFamily/_zw_okpIsFamily`。
- `crates/engine/src/js_dom_shim/part02.js`（+10/-1，三处接线）：
  importKey 在 `_zw_normalizeImportAlgorithm` 后、family 分派前归一（family
  helper 拿有效格式）；exportKey 同位归一（错误检查顺序不变：格式 → 注册 →
  extractable，export-key-error-order 断言面保持）；unwrapKey 格式白名单放行
  两别名（family 适配归一在后续 importKey 入口）。wrapKey 经 exportKey
  直通免改。deriveKey 内部导入不受影响（不走用户格式参数）。

## 验证

1. **构建**：`make zero-wpt-runner-release` 干净（31.88s）。
2. **全量语料**（`./target/test-guard --per-proc-mem 4 --total-mem 8
   --time-limit 3600 -- ./target/release/zero-wpt-runner
   testharness-webcrypto --json`）：360 案 / **81,851 keys = P 52,861 +
   F 28,986 + T 4**；对 m2s4 锚逐 (case, subtest) 对照 **80 变化全 F→P、
   精确落 4 靶案各 20（新族/旧 tentative 路径 × window/worker 双腿）、
   P→非P = 0、键集零漂移（lost 0 / gained 0）**。T 4 不变（Hybrid-KEM
   generateKey，域门控）。
3. **runner 单测**：`cargo test -p zero-wpt-runner` **276P / 0F**（与
   M2-3b 基线一致）；`cargo clippy -p zero-engine --all-features` 干净。
4. **flake 记账**：r1 同命令 V8 fatal（`Fatal error / unreachable code`，
   JSON 零字节，libc 栈帧）——M2-1 先例同型（WSL2 环境观察，同命令复跑
   干净按 flake 判据）；r2 复跑全量干净完成（本节全部数字出自 r2）。

## 账面更新

- **新族 F**：11,252 → **11,172**（−80，`其余` 桶 raw_format_aliases
  窗口+worker 40 × 旧 tentative 同名案 40）。
- **全账 F**：29,066 → **28,986**；P 52,781 → **52,861**（+80）。
- 旧 `.tentative.` 路径退役桶（收口重定基 DC-5②）内
  raw_format_aliases.tentative 双腿 40 keys 本轮 F→P——退役条目终态以
  M 末重定基对终态锚重算为准（[DC-5② 预检](2026-11-02-dc5-continuity-precheck.md)
  既定口径），不回改预检数。
- 对照基准更新为 [m2s5 锚](2026-11-02-m2s5-full.json.gz)。
