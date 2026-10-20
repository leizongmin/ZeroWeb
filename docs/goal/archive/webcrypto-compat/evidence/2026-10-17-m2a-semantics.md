# M2a 语义修齐（2026-10-17）

M1 失败簇 1-3（32 个非 tentative 可修项）全部修齐，另获 44 个错误实例身份修复。

## 改动

1. **随机源（DC-2 守护面）** shim part01b `getRandomValues`：整数 TypedArray
   instanceof 白名单（放行子类）；Float16/32/64 与 DataView 抛 TypeMismatchError；
   \>65536 抛 QuotaExceededError **接口实例**（`globalThis.QuotaExceededError`，
   code 22 + requested/quota null——WPT assert_throws_quotaexceedederror 校验构造器身份）。
2. **原生 DOMException 补缺** dom_bindings/dom_exception.rs：`code_for_name` 补
   `TypeMismatchError => 17`、构造器补 `TYPE_MISMATCH_ERR` 常量（polyfill 表已有、
   原生表漏——WPT assert_throws_dom 校验 code=17）。spec error-names-table。
3. **算法名 ASCII-only 折叠**：新增 `_zw_upperAscii`（仅 a-z→A-Z，U+212A KELVIN
   SIGN 不折叠），替换 subtle 全部算法名 `toUpperCase()`（part01b hash/import/derive/
   keyLength + part02 sign/verify/deriveBits/encrypt/decrypt + import/export 格式名）。
   spec algorithm-normalization：ASCII case-insensitive。
4. **derive fallthrough 修复** `_zw_performDerive`：未知派生算法 reject
   NotSupportedError（原 else 分支把 "PB\u212ADF2" 当 HKDF 执行）。
5. **digest 入参校验**：字符串算法归一 `{name}`；null/非对象/缺 name → TypeError
   reject（WPT「empty algorithm object」组）。
6. **detached buffer** `_zw_bufToBytes`：视图构造包 try/catch，已转移 buffer →
   零字节副本（spec：取字节副本；V8 Construct 抛 TypeError 须转空不 reject）。
7. **错误实例身份** `_zw_subtleDE`：subtle 全部 reject 改经
   `globalThis.DOMException || DOMException` 调用时求值（R384 同族——词法 polyfill
   实例 constructor ≠ 原生构造器，promise_rejects_dom 恒败）。

## 验证

- 定向：`getRandomValues` 27→**39/39 全绿**、`normalize-algorithm-name` 0→**4/4**、
  `digest.https.any.js` 100→**116/116**（tentative XOF 419F 不动）。
- 全量（123 案，两次跑逐位一致）：**32,717 = Pass 6,804（20.8%）+ Fail 25,912 +
  Timeout 1**；M1 6,728 → **+76**，零回归（各域无一下行）。
- 分域变化：(top) 31→47（随机源 12 + normalize 4）、digest 100→116、
  encrypt_decrypt 170→**206**、sign_verify 50→**58**（后两项为第 7 条错误实例
  身份修复的额外收益，AES-GCM/HMAC 负向用例 promise_rejects_dom 翻绿）；
  derive_bits_keys 5,179、generateKey 1,132、import_export 57 持平。
- 引擎测试：`cargo test -p zero-engine crypto` 9/9（R2775 扩展断言：Float32Array/
  DataView → TypeMismatchError，65537 → QuotaExceededError code 22 requested null）。
- 全绿案 5→8（新增 getRandomValues / normalize-algorithm-name / digest）。
- 证据：`-summary.json`（逐案计数 + totals）、`-full.json.gz`（全量逐案消息）。

## 剩余

M1 失败簇 4/5 不变：tentative XOF 419F 挂账；未实现算法域（RSA/EC/AES-CBC/CTR/
KW、jwk/pkcs8/spki、wrapKey）→ M2b AES-CBC/CTR 补面（依赖选型先行：RustCrypto
`cbc`+`ctr`+`aes`，与既有 `aes-gcm` 0.10 同族）。
