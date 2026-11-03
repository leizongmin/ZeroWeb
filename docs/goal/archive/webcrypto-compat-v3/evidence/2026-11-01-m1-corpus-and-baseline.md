# M1 语料预置 + 分域基线（2026-11-01，DC-1 / DC-3 分母侧）

## 语料预置（DC-1）

- 通道：`tests/wpt-runner/scripts/goals/38-webcrypto-compat-v3.sh`（转正子树锁
  `cb6cc97ada2772b2afe668b9fd5cb39a576a5347`，git trees 递归清单 + 逐文件
  fetch_raw 新增式拉取；本地已有文件幂等跳过）。
- 上游对账（rev `cb6cc97ada`，子树 `084495e783…` = 2026-11-01 复证与 HEAD
  子树逐字节相同）：**216 blob**；扩展名分流应拉 **210 全部在位、缺失 0**；
  6 个元数据 blob 正确排除（根+两新族 META.yml ×3、README.md、
  WEB_FEATURES.yml、tools/generate.py）。
- 本地：**275 文件 = 198 旧（pin 语料，未触碰）+ 77 新（今日落盘）**。
  - 幂等零变动证据：198 个旧文件 mtime 全部早于 2026-11-01（fetch_raw
    对已存在文件早退，零写入）；77 个新文件 mtime 均为今日。/common 依赖
    （subset-tests.js / gc.js）已在库，跳过。
  - 65 个上游已删的旧 `.tentative.` 路径文件仍在本地（新增式语义保留，
    与上游 pin→HEAD diff 的 65 删精确吻合；收口重定基时随 DC-5② 退役）。
  - 旧文件内容抽查：pin rev jsdelivr 回源 4/4 逐字节一致（digest.https、
    successes_AES-GCM、util/helpers、aes_ctr）；33,109 分母行为零回归
    （见下节）为全量行为面证据。同路径文件的内容更新按 Support Envelope
    记账不改锚、重定基轮统一处理。
- 散新案 5 案全部在位（export-key-error-order / okp_fixed_length /
  okp_malformed_asn1 / sharedarraybuffer / aes_kw_unwrapKey_failures）。

## 基线运行（DC-3 分母侧）

- 通道：`./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 3600
  -- ./target/release/zero-wpt-runner testharness-webcrypto --json`（Makefile
  `testharness-webcrypto` 同参数；runner/shim 本轮零改动，工作区 diff 仅 docs）。
- 总量：**181 案 / 40,925 子测试 = Pass 25,831 + Fail 15,091 + Timeout 3**，
  全绿案 76。证据：
  [2026-11-01-m1-baseline-full.json.gz](2026-11-01-m1-baseline-full.json.gz)
  （runner `--json` 原始输出 gzip；两连跑复验见文末）。
- 新增面（相对 33,109 锚）：**58 案 / 7,816 子测试** = 新族 53 案 / 7,781
  （modern-algos 40 + secure-curves 13）+ 散新案 5 案 / 35。
  idlharness 记账排除 3 案（根 idlharness.https + modern-algos/secure-curves
  各 1，与一期口径一致）。

### 零回归（一期 33,109 分母底线保持）

- 与二期收口锚 `archive/webcrypto-compat-v2/evidence/2026-10-31-m1-baseline-full.json.gz`
  逐 (case, subtest) 对照：**33,109 对全在，状态变化 0 例（P→非P 0 例）**；
  差集恰为新增 7,816 子测试，无丢失。

### 按 runner 目录桶分类

| 桶 | 案 | 子测试 | P | F | T | 全绿案 |
|---|---:|---:|---:|---:|---:|---:|
| (top) | 11 | 368 | 53 | 314 | 1 | 5 |
| derive_bits_keys | 11 | 12,448 | 12,434 | 14 | 0 | 8 |
| digest | 5 | 535 | 116 | 419 | 0 | 1 |
| encap_decap | 2 | 129 | 0 | 129 | 0 | 0 |
| encrypt_decrypt | 7 | 1,753 | 1,449 | 304 | 0 | 5 |
| generateKey | 39 | 9,047 | 5,449 | 3,598 | 0 | 25 |
| import_export | 23 | 7,686 | 5,337 | 2,349 | 0 | 12 |
| serialization | 19 | 63 | 36 | 27 | 0 | 12 |
| sign_verify | 9 | 813 | 635 | 178 | 0 | 6 |
| wrapKey_unwrapKey | 2 | 302 | 302 | 0 | 0 | 2 |
| modern-algos/ | 40 | 5,931 | 2 | 5,927 | 2 | 0 |
| secure-curves/ | 13 | 1,850 | 18 | 1,832 | 0 | 0 |
| **合计** | **181** | **40,925** | **25,831** | **15,091** | **3** | **76** |

### 新族按 10 域口径 rollup（窗口腿）

入口文档 10 域 universe（含 worker 变体）对照窗口腿实测——各域窗口腿计数
恰为 universe 的对半（worker 变体缺失），印证 M-worker 接线前终账天生缺口：

| 域 | 窗口腿案 | universe 案 | 窗口腿子测试 | P | F | T |
|---|---:|---:|---:|---:|---:|---:|
| ML-KEM | 6 | 12 | 1,740 | 0 | 1,740 | 0 |
| ML-DSA | 5 | 10 | 603 | 0 | 603 | 0 |
| X448/Ed448 | 13 | 26 | 1,850 | 18 | 1,832 | 0 |
| AES-OCB | 5 | 10 | 1,258 | 1 | 1,257 | 0 |
| ChaCha20-Poly1305 | 5 | 10 | 522 | 0 | 522 | 0 |
| KMAC | 5 | 10 | 709 | 1 | 708 | 0 |
| Argon2 | 2 | 4 | 46 | 0 | 46 | 0 |
| XOF digest | 4 | 8 | 419 | 0 | 419 | 0 |
| Hybrid-KEM | 4 | 7 | 224 | 0 | 222 | 2 |
| 门禁面 | 3 | 12 | 390 | 0 | 390 | 0 |
| （横切）raw_format_aliases | 1 | — | 20 | 0 | 20 | 0 |
| **新族合计（窗口腿）** | **53** | — | **7,781** | **20** | **7,759** | **2** |

- 门禁面 3 案（getPublicKey 0/43、supports-modern 0/147、supports 0/200）：
  supports 门禁语义未实现（未注册算法一律 reject），属 M2 无依赖面首切片。
- 基线期少量 P：X448/Ed448 importKey 失败边界 18P、AES-OCB/KMAC 各 1P
  （错误路径/参数校验先绿，实现路径未红转）。

### 散新案逐案（5 案 / 35 子测试，P 23 / F 12）

| 案 | 子测试 | P | F | 说明 |
|---|---:|---:|---:|---|
| export-key-error-order.https.any.js | 6 | 2 | 4 | 导出错误顺序语义 |
| import_export/okp_fixed_length.https.any.js | 6 | 6 | 0 | 全绿（既有 OKP 面） |
| import_export/okp_malformed_asn1.https.any.js | 12 | 12 | 0 | 全绿（既有 OKP 面） |
| sharedarraybuffer.https.any.js | 8 | 0 | 8 | SAB 域（BufferSource 语义/非 crypto 算法面） |
| wrapKey_unwrapKey/aes_kw_unwrapKey_failures.https.any.js | 3 | 3 | 0 | 全绿（既有 AES-KW 面） |

### Timeout 3 案归因

| 案 | runner 消息 | 归因 |
|---|---|---|
| algorithm-discards-context.https.window.js | `pending:13, phase:4`（与一期/二期每轮恒定同消息） | 谱系挂账 1T——iframe 跨帧注入 + `top` 回调链，本 goal Mission-5 销案 |
| modern-algos/generateKey/failures_Hybrid-KEM.https.any.js | `pending:0, tests:0` 未注册即超时 | Hybrid-KEM 组合算法面未实现；run_test 对组合名（MLKEM768-P256 等）注册路径在未实现分支未落 promise_test（M2 随 Hybrid-KEM 域切片核） |
| modern-algos/generateKey/successes_Hybrid-KEM.https.any.js | completion callback 未调用（`pending>0`） | 同上：`crypto.subtle.generateKey` 对 Hybrid-KEM 名返回未落定 promise（M2 随域切片修） |

注：两族多案带 `// META: timeout=long`，runner webcrypto 通道 CASE_TIMEOUT
10s（`ZW_CORPUS_CASE_TIMEOUT_SECS` 机制在册，二期 M1 曾接线后按机制归因
证伪回退）；本轮归因未受 10s 预算影响（T 案均为挂起非超工作量），M2 若
出现工作量型超时再启用。

## 复跑验证

第二轮 `test-guard` 同参数连跑（2026-11-01 同日）：**40,925 子测试状态逐项
全等（status diff = 0）、案序一致**；逐字节 cmp 有 3 处差异，全部落在 3 个
T 案的 runner 超时消息文本（消息含 JS 堆地址 `scripts=[…]` 与截断相位
`phase:2/4`，天然逐轮漂移；算法-discards-context 本轮 A 相 phase:4 与二期
锚消息一致，B 相 phase:2）。基线以状态为准，确定性成立（区别于二期
33,109 锚「逐字节一致」口径——本轮 T 案数 3 > 1，消息漂移面扩大）。

## 门禁与状态

- 本轮 runner 引擎/shim **零改动**（基线为纯记录）；`make test` / `make
  reftest` 非本轮触发面（零代码变更），不重复空跑。
- deps evidence 与选型征询另账：
  [2026-11-01-m1-deps-evidence.md](2026-11-01-m1-deps-evidence.md)。
