# Web Crypto 兼容三期 — 转正域收敛 + 谱系零遗留终局（modern-algos + secure-curves）

> **三期立项（2026-11-01，二期 D1 用户拍板）**：二期（2026-11-01 收口归档，
> [归档文档](archive/webcrypto-compat-v2.md)）DC-2 甄别定谳：一期 7,320F 挂账的
> `.tentative.` 域在上游已全部转正（`.tentative.` 50→0，迁入 `modern-algos/` +
> `secure-curves/` 两新目录族），另新增 pin 时不存在的 Hybrid-KEM 域。按域立项
> 建议（10 域 109 案/18,052 子测试，universe 口径）经用户拍板（2026-11-01，
> git 账面即本立项）落地本 goal。
> **同日二次拍板（零遗留终局）**：用户明确要求本 goal 执行完即 webcrypto 谱系
> 全部完成、无遗留——散新案漂移、worker 腿、algorithm-discards-context 1T、
> 收口重定基一并收编（见范围补全节）。**一期 33,109 分母零回归底线全程有效**
> （25,788P/7,320F/1T），收口重定基后由 HEAD 布局终账承接（过渡判据见 DC-5）。

**版本**: v1.1（同日二次拍板扩围）
**日期**: 2026-11-01（立项；同日零遗留扩围）
**状态**: Active——M1 完成（语料预置 + 分域基线 + deps evidence + 选型征询，
控制面 `webcrypto-compat-v3/master.md`）；M2 按域收敛待启动（拍板前仅无
依赖面切片）
**执行模式**: WPT 驱动（上游 WebCryptoAPI corpus 为验收标尺）+ 按域切片收敛
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——加密面）

## 上游事实（2026-11-01 复证）

- 语料 pin（`3159769338…`，子树 `ad14e36c81…`，202 blob / 126 test 入口）；
  上游 HEAD `a0dd301b85…`，WebCryptoAPI 子树 `084495e783…`（216 blob）——
  **与二期 DC-2 甄别时（`cb6cc97ada…`）逐字节相同**，甄别结论零漂移，
  `.tentative.` = **0**，未转正域不存在。
- pin→HEAD 路径 diff：80 新增 / 65 删除。删除侧 = 全部旧 `.tentative.` 路径
  及其 fixtures；新增侧 = 两新族全部文件 + **5 个散新案**（旧目录下的 stable
  新文件）：`export-key-error-order.https.any.js`、
  `import_export/okp_fixed_length.https.any.js`、
  `import_export/okp_malformed_asn1.https.any.js`、
  `sharedarraybuffer.https.any.js`、
  `wrapKey_unwrapKey/aes_kw_unwrapKey_failures.https.any.js`。

## 立项范围（二期 [DC-2 甄别清单](archive/webcrypto-compat-v2/evidence/2026-10-31-dc2-promotion-screening.md) D1 表）

| 域 | 分母（universe） | 建议范围 | 依赖候选 |
|---|---:|---|---|
| ML-KEM | 12 案 / 3,480 | encap/decap + generateKey + import/export raw 全链 | ml-kem 0.3.3（RustCrypto） |
| ML-DSA | 10 / 1,206 | sign/verify + generateKey + import/export | ml-dsa 0.1.1（RustCrypto） |
| X448/Ed448 | 26 / 3,700 | deriveBits/deriveKey + generateKey + import/export + sign/verify + serialization | x448 0.14.0-pre.12 / ed448-goldilocks 0.14.0-pre.15（pre 版，甄别从严） |
| AES-OCB | 10 / 2,652 | encrypt/decrypt + generateKey + import/export + serialization | ocb3 0.2.0（非 RustCrypto AEADs 主族） |
| ChaCha20-Poly1305 | 10 / 1,044 | 全链 | chacha20poly1305 0.11.0（RustCrypto AEADs） |
| KMAC | 10 / 1,418 | sign/verify + import/export | sha3 0.12（KMAC feature 面核实） |
| Argon2 | 4 / 92 | deriveBits + importKey | argon2 0.6.0（RustCrypto password-hashes） |
| XOF digest（cSHAKE/K12/SHA3/TurboSHAKE） | 8 / 838 | digest 扩展算法注册面 | sha3 0.12 + kangarootwelve 0.1.3 |
| Hybrid-KEM | 7 / 2,742 | encap/decap 组合域（ML-KEM+curve 系） | 随 ML-KEM/curve448 选型 |
| modern-algos 门禁面 | 12 / 880 | supports 门禁语义 + getPublicKey | 无新依赖 |

合计 109 案 / 18,052 子测试（universe 口径，含 worker 变体）。

## 范围补全（2026-11-01 零遗留二次拍板收编）

| 项 | 账面 | 说明 |
|---|---|---|
| 散新案 5 案 | 分母 M1 基线实测定 | 上表之外的 stable 新文件（见上游事实节）；M1 一并入桶基线 |
| worker 腿 | coverage worker 桶 13,812（一期口径）+ 新族 worker 变体 | runner 现为 window 单腿；workers 通道 runner 能力在册（70 号先例），crypto 面接线 + worker isolate 侧 crypto 绑定核实/补齐；新族 universe 分母含 worker 变体，不接线则终账天生缺口 |
| 1T algorithm-discards-context | 33,109 分母内 1T | 二期定性 runner/shim DOM 域边界（iframe 跨帧注入 + `top` 回调链）；本 goal 内修复销案，不再外挂 |
| 收口重定基 | 33,109 分母中 7,320F 位于上游已删除的旧 `.tentative.` 路径 | 收口时终账切 HEAD 布局全量（老 stable 案 + 两新族 + 散新案），旧路径条目随重定基退役；过渡判据 DC-5 |

## Mission

1. **语料预置**：38 号通道对 WebCryptoAPI 子树在锁定转正 rev（`084495e783…`
   子树，今日复证零漂移）做**全子树新增式**拉取——两新族 + 散新案 + 既有
   pin 语料幂等跳过零变动（33,109 锚保持）。
2. **依赖选型 deps evidence 先行**：逐域候选 crate 落 evidence（版本/维护面/
   feature/license），**引入前用户拍板**（二期 2026-10-31 决策纪律延续：
   rally 不自行引入新依赖）；拍板前收敛切片仅限无新依赖面（门禁面/散新案）。
3. **按域收敛**：每域一切片——基线 → deps（拍板后）→ shim/引擎实现 → 定向
   翻绿 → join 零回归。引擎切片号沿用全局 s 流水记账，撞号按 s162 先例
   （先落账保号，后收者顺延重编）。
4. **worker 腿接线**：crypto 面接入既有 workers 通道，全子树 window/worker
   双腿执行；worker isolate 侧 crypto 绑定缺失则按域切片补齐；逐案归因
   不外挂。
5. **1T 销案**：algorithm-discards-context iframe 跨帧注入 + `top` 回调链
   边界修复，1T → P。
6. **收口重定基**：终账切 HEAD 布局全量双腿账，旧 `.tentative.` 路径条目
   退役；重定基判据见 DC-5。

## Done Criteria

- **DC-1（语料）**：HEAD 布局子树全量测试文件入库（rev 记账 + 逐文件对账，
  含两新族与散新案），既有 33,109 分母语料零变动（幂等跳过证据）。
- **DC-2（deps evidence 与选型落账）**：10 域依赖候选逐域 evidence 齐备；
  用户拍板结论落控制面待用户决策表；实际引入与 evidence 一致。
- **DC-3（收敛基线）**：新域 + 散新案分母逐案分类账（通过率 + 失败归因）
  落 evidence；门禁面语义落地（supports 门禁 + getPublicKey）；`[SecureContext]`
  暴露门禁语义（一期 M6 三件套）不回退；worker 腿双腿账落档。
- **DC-4（coverage 复测）**：HEAD 布局全量 coverage 清零路径成立；tentative
  维度消失（上游 0），worker 变体分列归因。
- **DC-5（零回归 + 重定基 + 零移交）**：① 33,109 分母锚复证零差异；② 重定基
  过渡判据——重定基轮与旧锚对老 stable 案逐 subtest 连续性证明（同案同果），
  终账 = HEAD 布局全量双腿账，旧 `.tentative.` 路径条目退役有账；③ **谱系
  零移交**——webcrypto 一/二/三期全部账面闭合，所有非 P 子测试在本 goal
  evidence 内具名归因闭环，不向其他 goal / 待决清单移交任何挂账。

## 里程碑

- **M1**：语料预置（全子树新增式）+ 分域基线（含散新案）+ deps evidence +
  选型征询（DC-1 + DC-2 前半；征询在案即推进无新依赖面切片，不等拍板）。
- **M2+**：按域收敛切片（依赖拍板后按域推进；顺序按依赖就绪度，M1 证据后
  定序落 master.md）；worker 腿接线与 1T 销案穿插在无依赖间隙。
- **M 末**：收口重定基 + coverage 复测（DC-4/DC-5）+ 收口归档（模式 A）；
  归档时同步删除 38 号 rally 启动器。

## Support Envelope

**范围内**：HEAD 布局子树语料预置与基线、deps evidence、`__zw_crypto_*`
shim/引擎实现、worker isolate 侧 crypto 绑定、门禁语义、1T DOM 边界修复、
coverage 复测、收口重定基。
**排除**：依赖引入先于拍板；pin 语料既有文件的内容更新（上游若改内容，
记账不改锚，重定基轮统一处理）。
**资源约束（无人值守）**：批跑限 lane 并发 + case 墙钟预算（照 37 号先例
12 lane / 600s 起步，按面实测调整）；长跑任务走既有 OOM/墙钟守卫
（`docs/engineering/quality-gates.md`），禁止裸跑；CI 发射前查在跑实例
（weekly concurrency cancel-in-progress 会砍旧 run）。

### 依赖约束（run-rules §9）

- 一期 33,109 分母（25,788P/7,320F/1T）零回归底线：任何 P→非P 先修再收。
  锚：`archive/webcrypto-compat-v2/evidence/2026-10-17-m1-baseline.md`
  （+ summary.json，终态锚 m6-residual-full.json.gz 同目录）；二期收口
  三跑复验在 `f97190c41`。重定基后零回归底线由 DC-5② 终账承接。
- 新 crate 引入全部走 deps evidence + 用户拍板；X448/Ed448 pre 版候选
  甄别从严（0.14.0-pre.\* 非 stable）。
- 既有 RustCrypto 依赖面（rsa 0.9 / p256/p384/p521 0.13 / ed25519-dalek /
  x25519-dalek 3.0 / aes-gcm/cbc/ctr / sha2）为复用优先基准。

## 碰撞管理（run-rules §9）

- **25 号 html-semantics-compat（并行在跑）**：共享触面 `js_dom_shim/part01.js`
  （其 s209 切片触过）、`script_gen.rs`、`testharness.rs`；crypto 实现面
  （part01/01b/02 `__zw_crypto_*`）与其 forms 切片目录级分域但同文件——
  切片落账前先对账 main 最新触面，切片号按 s162 撞号先例处理。
- **worker 腿接线触 runner/workers 通道基建**（testharness runner 侧），
  与 25 号流共享 testharness.rs——接线切片落账前对账。
- **33 号 editing-compat（ZeroWeb-3 工作区并行在跑）**：editing shim 面，
  与 crypto 域不相交；合流经 main，双边切片落账前 merge-base 对账。
- 23 号 selection-compat：无共享面。

## Final Output Protocol

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方 DONE 允许条件 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **默认输出** |
| 真正外部阻塞（依赖不可用、平台不支持） | `BLOCK: <原因>` | 罕见使用 |

**DONE 允许条件**（同时满足）：DC-1~5 全部满足有据（测试命令、报告路径、
数字证据）；验证基于上游真实 WPT 用例（无内建 inline 充数）；`cargo build`
+ `cargo test` + `cargo clippy` 全过；master.md 内部自洽、里程碑归档完成；
33,109 分母零回归底线保持 + 重定基终账成立 + **谱系零移交**（DC-5③）。

## Document Control

- 每轮进展写 `webcrypto-compat-v3/master.md` 与 `evidence/`；入口文档仅在目标
  实质变化时修改；里程碑完成归档至 `webcrypto-compat-v3/archive/`（只追加）。
- 收口归档：入口 + 控制面整树迁 `docs/goal/archive/`，归档 README 登记行，
  trends csv 收行；归档时同步删除 `scripts/rally-goal-38-webcrypto-compat-v3.sh`
  启动器（2026-11-01 用户决策：归档 goal 启动器随归档删除）。
- **谱系终局声明**：本 goal 收口即 webcrypto 兼容谱系（一期 26 号 / 二期 26 号
  v2 / 三期 38 号）全部闭环，无挂账移交、无后继 goal。
- 「待用户决策」表事项不 BLOCK 推进——转零碰撞面自主推进，交付时集中提出。
