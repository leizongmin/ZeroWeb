# DC-1 · 语料修复 + 基线试跑 + 周跑接入（2026-11-09）

本轮五件事：① 发现并修复本地语料 pin 污染；② 按 CI 同款守卫协议重跑全量
基线（其间定位 r3–r5 空产出真因 = derived_bits_length 楔死并修复）；③
coverage 对账器分片路径映射（打通快照少记的 25k 绿桶）；④ 周跑 crypto
分片切 v3 全语料 fetch；⑤ 门禁清单。

## 一、语料污染发现与修复

### 现象

重跑基线（r1，`--json` 全量）得 262 案 / 67,495 keys = P 65,109 + F 2,382 +
T 4，对 11-03 收口锚（69,795 keys = P 67,845 + F 1,950 + T 0，
`archive/webcrypto-compat-v3/evidence/2026-11-03-mend-rebase-final-full.json.gz`）
**REGRESSED 432（全部 modern-algos Hybrid-KEM）+ 4 案 subtest 级 lost 2,304**，
lost 全部为 `generateKey/{failures,successes}_Hybrid-KEM` 双腿枚举中断
（harness 0 tests → Timeout）。

### 根因：本地语料被一期（26 号脚本）pin 内容覆写/回灌

锚对照失败信息全部指向支撑文件内容过期：`importKey` 「Good parameters」
子测试在 `assert_goodCryptoKey` 断
`key.algorithm.name === registeredAlgorithmName`，而本地
`util/helpers.js` 的 `registeredAlgorithmNames` 清单没有 Hybrid-KEM 复合名
（`MLKEM768-P256` 等）——导入成功后断言即炸。逐文件 git blob sha 审计
（对 v3 转正子树 `084495e783…` = commit `cb6cc97a` 的 recursive trees
清单，216 blob）：

| 类 | 数 | 明细 |
|---|---:|---|
| 内容 = pin 旧版 | **14** | `util/helpers.js`（直接 diff 证实逐字节等于 pin 版）+ 全部共享支撑/向量（`derive.js`/`ecdh.js`/`digest.js`/`aes.js`/`rsa.js`/`algorithm_registry.js`/`failures.js`/`successes.js`/`mac.js`/`signature.js` 等） |
| pin 布局残留（v3 树无此路径） | **65** | 50 个已退役 `.tentative.` 案 + 15 个旧路径支撑（`derive_bits_keys/argon2.js`、`util/supports.js` 等，v3 已迁 `modern-algos/`/`secure-curves/`） |
| 与 v3 pin 逐字节一致 | 196 | 测试案 + 向量主体 |

成因：收口（2026-11-03）之后本地对同一语料目录跑过一期 26 号脚本
（`fetch-wpt-webcrypto` 旧接线，WPT_REV pin `3159769338…`）——26 号对已存在
目录的幂等快路径挡不住 FORCE/首拉混合态，pin 内容覆写 14 个共享文件并回灌
65 个旧布局文件。v3 布局测试案（HEAD 内容）+ pin 支撑文件（旧内容）= 前述
全部失败。

### 修复（逐 blob sha 验证）

1. 14 个内容漂移文件从 `raw.githubusercontent.com/…/cb6cc97a/WebCryptoAPI/`
   重拉，写回后逐文件 blob sha 复核一致；
2. 65 个 pin 残留移出语料树至 `.acceptance/wptfyi-webcrypto/retired-tentative-backup/`
   （gitignored 诊断区，不回库）；
3. 终审计：语料树 210/210 文件与 v3 pin 逐字节一致，0 漂移 0 残留
   （上游 6 个非测试资产 META.yml/README/tools 按脚本分流规则不入库）。

### 防再发

`Makefile` 的 `fetch-wpt-webcrypto` 已改接 38 号脚本（见第四节）；Makefile
注释与 master.md 明确「26 号脚本勿再对 v3 语料目录运行」。38 号脚本对已
存在文件幂等跳过、不覆写内容——本地语料内容损坏时按本文第一节方法做
blob sha 审计修复（CI 全新 checkout 不受此影响）。

## 二、基线试跑（r1 污染语料 → r5 修复后语料）

- 运行协议：`./target/test-guard --per-proc-mem 4 --total-mem 8
  --time-limit <N> -- ./target/release/zero-wpt-runner testharness-webcrypto
  --json`（与 weekly crypto 分片同款守卫阈值；零 runner/引擎代码变化，
  仅语料内容修复）。
- r1（污染语料，1800s）：262 案 / 67,495 keys = P 65,109 + F 2,382 + T 4。
- r3（修复后语料，1800s）与 r4（3600s）：**守卫杀树、JSON 空——非慢，
  是楔死**（定位与修复见第五节）。初判「并行构建争抢放大墙钟」被两组
  证据证伪：① 家族二分探针逐家族跑全部 ≤99s 且键数与 r1 逐位一致
  （digest 16s / hybridkem 6s / generateKey 99s / import_export 22s /
  hkdf 7,322 keys / pbkdf2 17,264 keys 等），单案 `derived_bits_length`
  两腿 exit 124 空 JSON；② 11-03 收口终账同语料段仅 **3m34s** 完成
  （`archive/webcrypto-compat-v3/evidence/2026-11-03-mend-closure.md`
  终账节）。
- r5（修复后语料，**7200s**，2026-11-10 00:50 起跑）：同楔死——01:09:49
  终态 JSON 0 字节（早于 02:50 墙钟顶被 finalize 收走；`.acceptance/
  wptfyi-webcrypto/r5-finalize.log` 在案）。同一根因，与墙钟预算无关。
- r6（修复后语料 + **EnforceRange 修复二进制**，1800s cap）：终账跑，
  锚对照与 coverage 试跑见第五、六节补段。
- 期语料（一期 26 号 pin 布局）不单独重跑：一期 stable face（74 案 /
  25,768 keys）对 v3 布局逐位连续性已由收口 DC-5① 证毕
  （`archive/webcrypto-compat-v3/evidence/2026-11-03-mend-closure.md` 第二节），
  本轮 r6 锚对照即为同口径复验；重跑一期布局只会复现 33,109 锚并把污染源
  重新引入语料目录。

## 三、coverage 对账器：分片路径映射（wptfyi_gap.py）

### 快照少记的定位（按路径与 scope 对照）

对 wpt.fyi aligned stable @ `ce5ca39646`（本地缓存三家 summary，与收口
DC-4 复测同 basis）逐路径核对：WebCryptoAPI universe 334 路径 / 70,878
子测试 = 260 整文件 + **74 分片路径**（10 个整文件 × 双腿 × `?N-M` 分片：
hkdf 4+4、pbkdf2 9+9、successes_RSA-OAEP 16+16、RSA-PSS 4+4、
RSASSA-PKCS1-v1_5 4+4）。三家分片边界逐路径实测一致（单家内分片 = 整文件
划分）。快照 26,779 的少记三桶：

| 桶 | 子测试 | 机制 |
|---|---:|---|
| 分片收录 10 整文件 | ~25,042 绿 | ZW 整文件结果无 `?N-M` 变体可解析，落分子外 |
| modern-algos / secure-curves / encap_decap 等新族 + worker 腿 | ~17,000 绿 | CI fetch 停在一期 pin 布局，新族文件不存在 |
| derive_bits_keys 未执行 26 文件 | — | 同上（CI 只执行 12/38） |

### 对账器修复（resolution 侧，分母零变化）

`wptfyi_gap.py`：`build_sliced_totals()` 建分片组索引（base → 全部分片
路径 + 三家各自分片合计的最大值）；`resolve_to_universe()` 变体/exact 落空
后按分片组兜底；`cmd_coverage()` 对分片组命中记分子一次、其全部分片路径
整体标已覆盖（未覆盖循环不再重复计分母），文件分母维持分片路径数口径。
分母（den_files/den_subtests）与既有快照逐位可比；分子只增不减（收录
ZW 实跑通过的子测试）。

机制验证（r1 结果 + 既有缓存，输出 `.acceptance` 不回库）：WebCryptoAPI
topdir den_files 334 / den_subtests 70,878 与 DC-4 复测逐位相同；
covered 252→326（+74 全部为分片路径）；zw_passes 42,803→65,109；
zw_only 0 / suspicious 0。

新 basis 预览（同 r1 结果 + 最新 aligned stable @ `792c7512e3` 缓存，
`cache-latest/`）：WebCryptoAPI den 336 / 70,920（上游漂移 +2 路径
+48 子测试），covered 326 不变，zw_passes 65,109 → 91.81%。分片映射
在新 basis 上行为一致；下轮周跑对账将以此 basis 为准。

## 四、周跑接入（Makefile + ci-shards）

- `Makefile` `fetch-wpt-webcrypto`：26 号脚本 → **38 号脚本**
  （`tests/wpt-runner/scripts/goals/38-webcrypto-compat-v3.sh`，v3 转正子树
  全子树新增式）。周跑 crypto 分片（ci-shards.yml `fetch:
  fetch-wpt-webcrypto`）零编辑即得 HEAD 布局全量语料。
- `ci-shards.yml` crypto 分片 `time_limit` 1800→3600（全量 69,795 keys 为
  一期 33,109 的 2.1 倍；1800 按一期体量定）。**后续勘误（2026-11-10）**：
  r3/r4 空产出实为 derived_bits_length 楔死（第五节），3600/7200 提额是
  对伪慢的误判；修复后实测 3m58s，分片与本地 7200 线按保守余量维持，
  待 CI 实测后如需收敛另行落账。
- runner 侧注释同步：`main.rs` / `testharness_webcrypto.rs` 语料来源注释
  26 号 → 38 号（仅注释，无逻辑变化）。

## 五、derived_bits_length 楔死定位与 [EnforceRange] 修复（2026-11-10）

### 取证链（r3/r4/r5 空产出 → 单案根因）

1. **stall 非慢**：/proc/<pid>/io 双采样 read_bytes/wchar 冻结 + 99% CPU
   = 纯内存同步计算不推进（慢跑 io 应持续增长）；11-03 锚同语料 3m34s
   反证墙钟预算无关。
2. **家族二分**（FILTER 子串逐家族探针，600–900s cap，新二进制 + 修复后
   语料）：argon2 / cfrg / digest / ecdh / hkdf / pbkdf2 / hybridkem /
   generateKey / import_export 全部 exit 0 且键数与 r1 逐位一致；
   `derive_bits_keys` 卡死；细分到 `derived_bits_length` 单案两腿 exit 124
   空 JSON（`sub-probe-derivedlength*.json/log` 在案）。
3. **参数面审计**：案文件与其 `_vectors.js` 未变（mtime 10-31）；修复重写
   的 `_testcases.js`（11-09 22:57）把上游 enforceRange 组补齐——r1 旧内容
   每腿 29 keys = 4 算法 × 7 参数 + setup，HEAD 补 4 参数
   （NaN / Infinity / −8 / 2\*\*32+8）→ 每腿 45。楔死案必然在新增参数。

### 根因

`crates/engine/src/js_dom_shim/part02.js` deriveBits 的 PBKDF2/HKDF 分支
length 校验 `typeof length !== 'number' || length < 0 || length % 8 !== 0`
对 **2\*\*32+8 = 4,294,967,304**（number、≥0、8 倍数）放行 → host KDF 以
dkLen = 4,294,967,304/8 ≈ 537MB 执行：PBKDF2 100,000 迭代 × 1,680 万块
≈ 10^12 次 SHA-256，纯 CPU 不可终止——RSS < 4GB 不触守卫、io 冻结、
单核满载，即 r3/r4/r5 的楔死形态（HKDF 同参数虽有限也在 512MB 级输出 +
CSV 字符串/数组爆炸面）。spec：deriveBits `length` 为
`[EnforceRange] unsigned long?`，NaN/±Infinity/<0/>2^32−1/非整数须在
**任何算法步序前** TypeError；WPT `enforceRangeTestCases` 期望 4 参数
全部 TypeError。

### 修复

- part02.js deriveBits 入口统一 [EnforceRange] 归一（`lenArg`）：非法值
  TypeError；null/undefined/省略 → null（ECDH/X25519 全宽、HKDF/PBKDF2/
  Argon2 各自 OperationError 分支，语义不变）。ECDH/X25519 分支删除被
  归一覆盖的 `!(lenBits >= 0)` TypeError 分支；PBKDF2/HKDF 校验与
  `_zw_performDerive` 改用 `lenArg`；Argon2 传 `lenArg`（其 `length ==
  null` OperationError 分支已有，null 行为不变）。
- `crates/engine/src/js_dom_bridge_tests/part53.rs` M6 契约同步：HKDF
  −8 期望 OperationError → **TypeError**（`neg:TypeError`），注释注明
  [EnforceRange] 与 WPT derived_bits_length enforceRange 组同语义。
- 指导条目：zeroweb-guidelines #21（外部可控尺寸封顶）。

### 验证

- 物证（#28 产物本体核验）：新二进制 `grep -a -c "must be an unsigned
  long" target/release/zero-wpt-runner` = 3。
- 单案探针（修复二进制，600s cap）：`derived_bits_length` 两腿 exit 0，
  各 45 subtest **全 Pass**（含 enforceRange 16/腿；与 r1 的 29 keys 级
  对照归锚对照判定）（`sub-probe-derivedlength-fix.json` 在案）。
- 全量终账 r6（修复二进制，1800s cap）：exit 0，耗时 **3m58s**
  （01:29:23→01:33:21），262 案 / **70,627 keys = P 67,877 + F 2,750 +
  T 0**（`.acceptance/wptfyi-webcrypto/2026-11-10-r6-full.json` 在案）
  ——3600/7200 墙钟线均为楔死误判产物，锚速量级恢复（比 11-03 锚
  3m34s 多出的 ~1 分钟即 832 个新增 subtest 的枚举）。
- 锚对照（r6 vs r1 同语料，`webcrypto-anchor-diff.py`）：**REGRESSED 0 /
  FLIPPED 0 / case 漂移 0**；FIXED 432 全部落 Hybrid-KEM importKey
  双腿（语料修复效果，非本修复）；subtest 漂移 24 案 =
  `derived_bits_length` 双腿各 +16（enforceRange 组，32 全 P）+
  generateKey failures 面 22 案（`failures.js` v3 内容枚举恢复，新显形
  800 F 归 DC-2 余量分类）。
- 对 11-03 收口锚（污染前语料）：**REGRESSED 0 / FIXED 0 / FLIPPED 0 /
  case 漂移 0**——69,795 共享 keys 逐位一致；漂移恰为 **832 = 800**
  （failures 面，全 F）**+ 32**（enforceRange，全 P），分母对平
  70,627−69,795。F 2,750 分解：Ed448 面 1,950（secure-curves Ed448
  1,946 + getPublicKey Ed448 4，均 NotSupportedError，2026-11-02 拍板
  降 scope 账）+ 新显形 800（DC-2 余量）；T 0——r1 的 4 T（turboSHAKE
  flake）在 r6 清零。

## 六、门禁与验证清单（本轮）

- 修复后全量基线（r6）对 r1 / 11-03 收口锚逐 subtest 对照：零回归，
  见第五节。
- engine 定向测试（`cargo test -p zero-engine --lib crypto_subtle`）：
  **12/12 ok**（含 M6 新契约 `neg:TypeError`；2.81s）。
- coverage 试跑（r6 结果 + 两缓存 basis，guard 均过）：
  ce5ca39646（发布快照同 basis）WebCryptoAPI **67,877 / 70,878 =
  95.77%**（r1 同 basis 65,109 / 91.86%）；最新 792c7512e3
  **67,877 / 70,920 = 95.71%**。**净增 67,877 − 26,779 = 41,098 ≥
  31,000 达标线**（r1 口径为 38,330）。
- `python3 -m py_compile wptfyi_gap.py` 通过；scope 对账机制经 r1（污染
  语料）实跑 + 新 basis 预览两次验证。
- clippy/fmt：`cargo clippy -p zero-engine --all-targets -- -D warnings`
  无告警；`cargo fmt -p zero-engine -- --check` 无 diff（触面 shim JS
  不经 rustfmt；part53.rs 已覆盖）。
