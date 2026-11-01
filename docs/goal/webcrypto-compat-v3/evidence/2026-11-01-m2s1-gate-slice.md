# M2-1 门禁面切片 evidence（2026-11-01）

> supports 静态门禁 + getPublicKey 实例方法（D-10，无新依赖首切片）。
> 语料锚：`tests/wpt-runner/wpt-data/WebCryptoAPI/{modern-algos,getPublicKey*,
> supports*}`（M1 转正子树 + 旧 `.tentative.` 新增式保留，同驱动同跑）。

## 切片内容

1. **`SubtleCrypto.supports(operation, algorithm, lengthOrAdditionalAlgorithm)`**
   （wicg/webcrypto-modern-algos 草案）——新增
   `crates/engine/src/js_dom_shim/part02a.js`（自包含 IIFE，挂 part06 尾）：
   - 16 op 白名单（大小写敏感）；encap/decap 四 op 名映射；
   - 第三参分流：number → length、string/object → additionalAlgorithm、
     其余（null/布尔）→ false；
   - 组合语义：deriveKey/unwrapKey/encapsulateKey/decapsulateKey 查附加算法
     importKey 面、wrapKey 查 exportKey 面、其余 op 带第三参 false；
     deriveKey 以附加算法 keygen 长度（HMAC=hash 块大小/AES∈{128,192,256}，
     缺省 256）转 deriveBits 校验；
   - 注册面 = 既有 18 算法 × op 真实矩阵（REG 键一律归一后形态——
     upperAscii 只映射 a-z，`RSASSA-PKCS1-v1_5`→`RSASSA-PKCS1-V1_5`、
     `Ed25519`→`ED25519`）；未注册域（ML-KEM/ML-DSA/X448/Ed448/KMAC/OCB/
     ChaCha/XOF/Argon2/Hybrid）全 op false，supports-modern matrix 期望 true
     属设计内红（待 D1 域切片扩表）；
   - 参数面校验：AES-GCM iv 恒 12B + tagLength 合法集、AES-CBC iv 16B、
     AES-CTR counter 16B + length∈[1,128]、RSA exponent 奇/≥3/位长<
     modulusBits + modulus %8、EC namedCurve 注册面、ECDH/X25519 deriveBits
     域宽（P-256→256/P-384→384/P-521→528/X25519→256）、HKDF 上限
     255×摘要位长（SHA-256→65280）、HMAC/AES length 字节对齐。
   - `getPublicKey` 域检查 getPublicKey=exportKey 注册 + pub 族标记。
   - 接口对象 `SubtleCrypto`（Illegal constructor）+ 静态 supports 挂载；
     **文件自包含**（shim 三段 IIFE 下 crypto helper 内层闭包不覆盖 part06
     尾挂载点，引用面仅限 globalThis 契约 + 本地 upperAscii/bufBytes 镜像）。
2. **`getPublicKey(key, usages)` 实例方法**——`part02.js` `_zwSubtle` 字面量内：
   - 算法域检查（非对称族外 → NotSupportedError）**先于** key type 检查
     （WPT 断言面：AES-GCM/HMAC secret 密钥 → NotSupportedError；
     非对称 public 密钥经域检查后 → InvalidAccessError）；
   - usages ⊆ 公钥 usage 面（`_zw_rsaTypeUsages/_zw_ecTypeUsages/
     _zw_okpTypeUsages(name, true)`，空 usages 合法）否则 SyntaxError；
   - 私→公派生走既有 host DER 面（generateKey 同径，零 Rust 改动）：EC/RSA
     `*_export_der(..., 'spki')`（两族公钥 `_raw` 契约 = SPKI DER，exportKey
     spki 公透传）；**OKP `okp_export_der(..., 'raw')`**（OKP 公钥 `_raw`
     契约 = raw 公钥字节 32——存 SPKI DER 会使后续 exportKey('spki') 以
     44 字节输入落空 okp_export_der 的 32/48 双宽匹配 → InvalidAccessError）；
   - 结果恒 extractable=true、algorithm 成员浅拷贝；派生失败 → OperationError。

## 调试定案（三根因，node 探针复现后修复）

二跑 187/200（13 fail）定位：

1. **REG 键归一不匹配**（10 fail）：algNameOf 大写归一后查表，REG 键与
   validateGenerateKey/validateImportKey 比较仍用原始拼法 → RSASSA×5、
   Ed25519×5、common combos 全 false。修复：键与比较全改归一形态。
2. **HMAC keygen 缺省长度**（2 fail + tentative 2）：keyLengthOfAdditional
   对无 length 的 HMAC 返回摘要输出位长（256），spec get key length of the
   generated key 取 **hash 块大小**（SHA-1/256→512、SHA-384/512→1024）→
   ECDH P-256+HMAC（512>256 域宽）期望 false 得 true、X25519+HMAC 同。修复：
   新增 SHA_BLOCK_BITS 表。探针 15/15 全过后 supports 定向复跑全绿。
3. **getPublicKey 两症**（定向首跑余留）：① OKP 派生结果误存 SPKI DER 进
   `_raw`（见上，Ed25519/X25519 的 export 比对步 InvalidAccessError）；
   ② 错误检查顺序（key type 检查在前 → AES-GCM/HMAC 得 InvalidAccessError，
   期望 NotSupportedError）。修复后 getPublicKey 定向复跑 21/37+21/43，余红全部
   域门控（Ed448/X448/ML-DSA×3/ML-KEM×3/Hybrid×3 未注册域 generateKey 即拒）。

## 结果（定向跑 + 全量跑，test-guard 包裹 `--per-proc-mem 4 --total-mem 8 --time-limit 3600`）

| 案 | 基线（M1） | 切片后 |
|---|---|---|
| modern-algos/supports.https.any.js | 0/200 | **200/200** |
| supports.tentative.https.any.js | 0/171 | **171/171** |
| modern-algos/supports-modern.https.any.js | 0/147 | 77/147（余 70 域门控红） |
| supports-modern.tentative.https.any.js | 0/94 | 53/94（余 41 域门控红） |
| modern-algos/getPublicKey.https.any.js | 0/43 | 21/43（余 22 域门控红） |
| getPublicKey.tentative.https.any.js | 0/37 | 21/37（余 16 域门控红） |
| **合计** | **0/792** | **543/792** |

- **零回归**（红线 P→非P）：对 M1 基线
  [2026-11-01-m1-baseline-full.json.gz](2026-11-01-m1-baseline-full.json.gz)
  逐 (case, subtest) 对照——181 案两侧全等（added=0/removed=0）、543 变化
  **全部 Fail→Pass**、**P→非P 0 例**；543 翻绿精确落 6 触面文件，与定向跑
  逐位一致。全量新分母：**40,925 = P 26,374 + F 14,548 + T 3**。两连跑
  （2026-11-01 22:34/22:40）状态逐项全等（0 变化，对账口径同 M1）。
- 全量 JSON 锚：[2026-11-01-m2s1-full.json.gz](2026-11-01-m2s1-full.json.gz)
  （两连跑第一轮原始输出；后续切片以此为新对照基准，M 末收口重定基
  DC-5② 终账）。
- **环境观察**：首轮全量 14 分钟处 zero-wpt-runner segfault（dmesg signal 11
  V8 堆对象字段读，WSL2）；同二进制同命令复跑 3m53s 全程干净且结果与定向跑
  逐位一致——按单采样不稳定即 flake 判据（learning #35）记环境性一次事件，
  不归因切片；后续轮次全量跑若复现再立案。

## 设计内红（后续切片账）

- supports-modern 矩阵 70+41、getPublicKey 22+16 域门控红：全部等待 D1
  拍板后按域切片（ML-KEM/ML-DSA/X448/Ed448/AES-OCB/ChaCha/KMAC/Argon2/
  XOF/Hybrid）注册面扩表 + 算法实现落地后翻绿；红线仍以 P→非P 为准。
- 旧 `.tentative.` 路径文件（65 个新增式保留）与转正同款文件同驱动——
  supports 两文件全绿（F→P 改善）；supports-modern/getPublicKey tentative
  余红与转正版一致（同款域门控）。
