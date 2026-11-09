# DC-2 · generateKey failures 面 800 F 修复：数值成员 [EnforceRange] 入口归一（2026-11-10）

本轮四件事：① r6 遗留 F 2,750 中 800 F 的参数面定谳；② 三处 shim 入口
[EnforceRange] 归一（与 deriveBits length 同根因类，复用同款判据）；
③ part53.rs 契约块同步；④ 全量 r7 终账 + 锚对照 + coverage 两 basis。

## 一、800 F 参数面定谳

r6 F 2,750 = Ed448 面 1,950（降 scope 账，见
[DC-1 evidence 第五节](2026-11-09-dc1-corpus-repair-and-baseline.md)）+
可修面 800。800 逐 key 提取（r6 JSON 按 failures 文件 × subtest 名）全部
落 `generateKey/failures_*` 9 算法 × 2 腿 18 文件，失败值单一：

| 算法 | 文件 F/腿 | 失败参数（每算法两组） |
|---|---:|---|
| AES-CBC/CTR/GCM | 68 | `{length: 4294967424}`、`{length: -4294967168}` |
| AES-OCB（modern-algos） | 68 | 同上 |
| RSA-OAEP | 56 | `{modulusLength: 4294968320}`、`{modulusLength: -4294966272}` |
| AES-KW、HMAC | 20 | `{length: ±(128 + 2**32)}` |
| RSA-PSS、RSASSA-PKCS1-v1_5 | 16 | 同 RSA-OAEP |

语料侧（`generateKey/failures.js`
`badAlgorithmPropertySpecifiersFor`）：AES 组补测
`[64,127,129,255,257,512,128+2**32,128-2**32]`，HMAC/RSA 组补测
`±2**32` 邻域；期望（failures.js:186-190）`length < 0 || length > 2**32`
→ **TypeError**，in-range 非法值（64..512 等）→ OperationError。r6 账面
in-range 组全过、±2^32 组全 F——失败模式单一收敛。

## 二、根因

generateKey 算法定典成员缺 WebIDL `[EnforceRange]` 入口转换。spec IDL：

- `AesKeyGenParams.length` = `required [EnforceRange] unsigned short`
  （合法域 [0, 65535]）；
- `HmacKeyGenParams.length` = `[EnforceRange] unsigned long`（缺省 = hash
  块大小）；
- `RsaKeyGenParams.modulusLength` = `required [EnforceRange] unsigned long`。

`[EnforceRange]` 转换发生在**任何算法步序之前**（WebIDL 字典转换层）：
NaN/±Infinity/<0/>上界/非整数 → TypeError。修复前 shim 直接以 number 语义
放行 ±2^32 邻域值，穿透到算法步序后的 length/modulus 校验落
OperationError（AES）或 host 拒绝（RSA）——与 deriveBits `length` 楔死同
根因类（同 [EnforceRange] 家族；前批修的是 deriveBits 方法参数，本批是
generateKey 字典成员），指导条目同为 zeroweb-guidelines #21。

## 三、修复（三处入口归一，判据与 deriveBits `lenArg` 同款）

`isFinite(n) && n >= 0 && n <= max && n === Math.floor(n)`，否则
`new TypeError`；`length != null` 才校验（缺省语义不变）：

1. `part02.js` generateKey **AES 分支**（含 AES-OCB）：max 65535
   （unsigned short）；缺省 256 不变（spec required 语义 corpus 未覆盖，
   行为保持）。
2. `part02.js` generateKey **HMAC 分支**：max 2^32−1；缺省 = hash 块大小
   不变。
3. `part01b.js` `_zw_rsaGenerateKey`：max 2^32−1，置于 usage SyntaxError
   **之前**（WebIDL 转换先于算法步序；缺成员缺省 0 → host OperationError
   路径不变）。

错误序说明：corpus「Bad usages」组全部用合法算法参数、本批入检查只在
数值越界时触发，两者不叠加探测；spec 序（转换 → NotSupported → 算法步序
usages/length/空 usages）不受影响，part53.rs 既有契约
「非法 usage SyntaxError 先于 length 校验」（length:128 合法值路径）保持
通过。

## 四、验证

- 引擎契约（`test-guard --compile-first` 包裹 `cargo test -p zero-engine
  --lib crypto_subtle`）：**12/12 ok**；part53.rs `kw_jwk_m3b` 新增契约块：
  AES/HMAC/RSA ±2^32 六例全 **TypeError**，AES-OCB 512（in-range 非法）
  仍 **OperationError**。
- 定向 `generateKey/failures`（release 修复二进制，provenance：二进制内
  新增错误消息串 grep 命中）：42 文件 = **14,332 P + 168 F**，余 F 恰为
  Ed448 降 scope 面（`secure-curves/generateKey/failures_Ed448` 双腿）。
- 定向 `generateKey/successes`：6,626 P + 72 F——F 全部
  `successes_Ed448` 双腿（r6 已有，同属 Ed448 面），非 Ed448 全过。
- 全量 r7（2026-11-10，3600s cap）：exit 链见附注，262 案 /
  **70,627 keys = P 68,677 + F 1,950 + T 0**
  （`.acceptance/wptfyi-webcrypto/2026-11-10-r7-full.json`）。
- 锚对照 r7 vs r6（`webcrypto-anchor-diff.py`）：**REGRESSED 0 /
  FLIPPED 0 / case 级漂移 0 / subtest 级漂移 0**；FIXED 恰 **800**、全部
  落 18 个 failures 文件（9 算法 × 2 腿，逐文件计数与 r6 失败清单位位
  对齐）。
- coverage 两 basis（`wptfyi_gap.py coverage`，均 exit 0；`--out` 需置于
  results 目录外——同目录时脚本会扫到自身输出的 cov.json 报非数组错）：
  ce5ca39646 **68,677 / 70,878 = 96.89%**（r6 口径 95.77%）；最新
  792c7512e3 **68,677 / 70,920 = 96.84%**（95.71%）。对发布快照净增
  **68,677 − 26,779 = 41,898 ≥ 31,000 达标线**。全过文件
  294→312（+18 = 本批修复文件）。
- clippy/fmt：`cargo clippy -p zero-engine --all-targets -D warnings` 无
  告警；`cargo fmt -p zero-engine --check` 无 diff（shim JS 不经
  rustfmt；part53.rs 已覆盖）。
- 附注：quota 重置触发同目标 re-dispatch 爆发——r7 三链并发（02:21/
  02:24/02:25）同写一账，产出逐位一致，按互补链仲裁采认先完成链、清理
  冗余链；首跑曾随 session 退出终止（JSON 0 字节）。同 clone editing 流
  `formatblock.html` 探针并发在跑（只读同一 release 二进制，无构建/写面
  冲突，墙钟计时不作锚证据）。

## 五、挂账观察（不在本批修）

- HMAC generateKey 显式 `length: 0`：现状经 `_zw_keyLengthBits` 落
  「0 → hash 块大小」分支**生成 512/1024 位密钥**，spec 要求 0 →
  OperationError；corpus 未覆盖该组合（failures.js HMAC 只有 ±2^32 组），
  part02.js:773 附近旧注释（「显式 length 0（或负）→ OperationError」）
  与实际行为不符。留后续批：行为对齐 + 注释修正 + 契约补测，一并落账。

## 六、CI run 42667268541 crypto 分片核验（DC-1 遗留项收口）

产物 `zw-results-crypto`（30.6KB tar / 13.7MB JSON）非空；解包逐 key 对
本地 r6：**70,627 keys 完全一致，0 漂移 0 状态错位**（CI 分片跑在
4482feb56 = deriveBits 修复后、本批 generateKey 修复前的二进制，账面即
r6 口径）。其余分片与全量对账归 DC-3。
