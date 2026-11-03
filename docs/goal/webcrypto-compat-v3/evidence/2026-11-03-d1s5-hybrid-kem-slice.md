# D1 域切片第五轮：Hybrid PQ/T KEM 三组合（2026-11-03）

> D1 收官批（[d1s4](2026-11-03-d1s4-x448-kmac-slice.md) 后延续）：MLKEM768-P256
> / MLKEM768-X25519 / MLKEM1024-P384 全链路（draft-irtf-cfrg-concrete-hybrid-kems-03
> §4，CG 框架）。复用 d1s3 落树的 ml-kem 0.3.3 与已有 p256/p384/x25519 依赖，
> 零新增 crate。宿主实现 `crypto_hybrid.rs`（新文件）→ `crypto_asym.rs` 六
> mlkem 入口按名分流 → shim `_zw_mlKemIsFamily` 家族扩名（part02/callbacks
> 零改动）→ REG 三行 → 定向翻绿 → 锚对照零回归。对照基准
> [d1s4 锚](2026-11-03-d1s4-x448-kmac-full.json.gz)；本轮新基准
> [d1s5 锚](2026-11-03-d1s5-hybrid-kem-full.json.gz)。

## 宿主实现（`crypto_hybrid.rs`，新文件 ~600 行）

- **三组合参数表**（draft-03 §4，逐字段）：`HybridParams { ek_pq, ek_t,
  ct_pq, nseed_t, label, alg_id, ml1024 }` —— P256(1184/65/1088/128)、
  X25519(1184/32/1088/32)、P384(1568/97/1568/48)；seed 恒 32B、ss 恒 32B。
  label：`"MLKEM768-P256"` / `"\.//^\"`（X25519）/ `"MLKEM1024-P384"`。
- **ExpandSeed**（§3.2）：SHAKE256(seed, Nseed_T + 64)，拆分序
  seed_PQ(头 64B d‖z) ‖ seed_T(尾 Nseed_T)——keygen KAT 钉死。
- **曲线标量**（§3.1/§3.3）：P 系 rejection sampling（Nseed_T 128/48 池、
  `from_bytes` 拒 0 与 ≥n 后自愈续采样）；X25519 恒等（Nseed_T 32）。
- **Combiner（本轮核心修正，见下节）**：SHA3-256(ss_PQ ‖ ss_T ‖ ct_T ‖
  ek_T ‖ label)。
- 六入口 `hybrid_generate/public_from_seed/import/export_der/encapsulate/
  decapsulate`；encap 侧 ML-KEM 随机性走 ml-kem crate 内建 OS 源、传统侧
  ephemeral 走 getrandom；decap 侧 ct_PQ 篡改走 ML-KEM 隐式拒绝
  （FIPS 203 §7.3）。密钥材料零落 tracing（同前轮纪律）。

## 切片内修复：Combiner 公式（KAT 首轮 P256 不中的根因）

初版误写 X-Wing 形 `SHA3-256(ss_PQ‖ss_T‖ct_PQ‖ct_T‖label)`（含 ct_PQ、
缺 ek_T）——draft-03 §4 只声明「CG framework + SHA3-256」，combiner 精确
公式在 [HYBRID-KEMS]-09 §5.1.3/§5.5：

```
C2PRICombiner(ss_PQ, ss_T, ct_T, ek_T, label) = KDF(concat(...))
```

含**接收方公钥 ek_T**、**不含 ct_PQ**；Encaps/Decaps 两处同式。诊断链：
keygen/roundtrip 绿 + ss_T 经 python cryptography 独立 ECDH 证实 → 嫌疑收
敛到共享定义；15 种 combiner 变体 × 3 哈希离线枚举零命中 → draft 文本版
程序化解析（分页折行感知）逐字节 diff 三常量全 IDENTICAL（排除转录错）→
拉 HYBRID-KEMS-09 §5.1.3 得精确式，离线对 P256/X25519 两向量同时命中。
教训：**「identical to X-Wing」的正文表述与 CG combiner 定义冲突时以框架
定义与向量为准**；X-Wing 原式（含 ct_PQ 缺 ek_T）是 CK/CG 之外的特化。
已有沉淀 [kmac-newx-raw-x](../../../learnings/bugs/2026-11/2026-11-03-kmac-newx-raw-x-vs-tuplehash-encode-string.md)
同类（规范原文仲裁优于记忆）。

## shim 接线（三文件零新回调）

- `part01b.js`：`_zw_mlKemIsFamily` 三名并入（hybrid 与 ML-KEM 同
  usages/格式面，WICG modern-algos Hybrid KEM 域）。
- `part02a.js`：REG 三行（generateKey/importKey/encapsulate/decapsulate/
  exportKey + pub:1，格式同 ML-KEM 行）。
- `part02.js`/`callbacks.rs`：零改动——全部流量经既有 mlkem 入口按名分流
  （`crypto_asym.rs` 六入口开头 `is_hybrid(name)` 分流到 crypto_hybrid）。

## 语料对齐（wpt-data 本地陈旧副本 → pin rev 复位，非本地补丁）

本地 `WebCryptoAPI/util/helpers.js` 与 `generateKey/algorithm_registry.js`
系早期波次取数（hybrid 落上游前），缺三 hybrid 名——对上游锁定 rev
`cb6cc97a` 逐字节 diff 后整文件复位（上游两文件均已含三名单/三行；
helpers.js 唯一差异 = 三行名单，registry 同）。successes_Hybrid-KEM 靶面
Case 经 `assert_goodCryptoKey` 的 `registeredAlgorithmNames` 查名（helpers.js
:7 独立清单，与 registry 无关）——清单缺名即 expected undefined 全案红。
新 clone 取数走同一 pin，天然含对齐后文件，无本地漂移负担。

## 定向语料（d1s4 同款 test-guard 4/8/1800 直调 --json，filter 路径子串）

| filter | 账面 |
|---|---|
| Hybrid-KEM（generateKey 双族 + importKey） | **2736P / 0F**（exit=0） |
| encap_decap（bits + keys） | **774P / 0F**（exit=0） |
| hybridkem（serialization window） | **6P / 0F**（exit=0） |
| supports-modern | **482P / 0F**（exit=0） |
| getPublicKey | 152P / 8F——8F 全 Ed448 域外（拍板未注册，d1s3/d1s4 同款在案失败集；Hybrid 子测试全绿，d1s4 账面同位 20F 含 Hybrid，本轮消 12） |

host 单测（`cargo test -p zero-engine --lib hybrid_kem_tests`，5/5）：
keygen_matches_draft_vectors（P256/X25519/P384 ek 逐字节）/ decap_vectors_kat
（P256/X25519/P384 三组合 shared_secret 全账）/ encap_decap_roundtrip /
import_export_formats / import_validates_curve_point（P 系非曲线点拒绝、
X25519 任意 32B 收）。runner 单测 278P/0F/76I。

## 全量语料 + 锚对照（d1s4 → d1s5）

1. **全量（r2，前轮 session 落盘）**：树 `fc0fa3837`（WIP 提交态，rebase 前），
   systemd `d1s5-r2-webcrypto` 单发（quiet window 09:48:59 after 1440s——
   ZeroWeb-2 editing 电池在跑，等待窗让过后落盘）→ 360 案 / **84,523 keys =
   P 80,622 + F 3,901 + T 0**，guard-exit=1（在案失败集常态）。
   **T 4→0**：Hybrid-KEM generateKey ×4 案级 Timeout 键退役（d1s3 起
   「余 T 4 = Hybrid-KEM generateKey ×4」挂账销案），全子测试注册
   684/684/468/468 ×双腿全 Pass。r2 全账唯一 REGRESSED = 1：TurboSHAKE
   24MB heisenbug 同案复发（`vector #8 (512 bit output, 24137569 byte
   input)` P→F——d1s2 定谳间歇 ~1/6、d1s3 r1 同形状 vector #9 先例，
   挂账口径不变）。
2. **全量（r3，本轮落盘锚）**：rebase 后现行树 `95d2f3699`（实现面零变化，
   d1s3 先例：锚以复跑落盘），systemd `d1s5-r3-webcrypto` 单发——quiet
   window 未开（ZeroWeb-2 editing 电池连续项 multitest→insertparagraph→
   delete.html），75 分钟 cap 到点 **PROCEED-ON-TIMEOUT 11:26:50** 与电池
   并行落盘（d1s4 先例：跨 clone 并行 inserttext/delete.html 实证无扰），
   语料段 11:26:50→11:31:26（guard 4/8/1800）。→ 360 案 / **84,523 keys =
   P 80,623 + F 3,900 + T 0**。
   **r2↔r3 键级 diff 恰 1 键**：该 flake 键 Fail→Pass，其余 84,522 键逐位
   全等（lost=0/gained=0）——rebase 零位移复现 + heisenbug 复归挂账形态
   一次闭环。
3. **锚对照**（`scripts/webcrypto-anchor-diff.py` d1s4 → r3；
   [anchor-diff.txt](2026-11-03-d1s5-hybrid-kem-anchor-diff.txt)）：
   **REGRESSED（P→非P）= 0**（零回归红线；r2 态 flake 于 r3 复绿后归零）、
   FLIPPED = 0、case 级 lost/gained = 0，退出码 2 仅为 subtest 级 drift
   4 案（Hybrid-KEM generateKey T 键退役 ×4，全 Pass 增益侧，见 5）。
   FIXED（非P→P）**750 落 11 案全部 Hybrid-KEM 靶面**（无一案域外）：
   encap_decap_bits 9×2、encap_decap_keys 120×2、getPublicKey 6×2、
   Hybrid-KEM_importKey 216×2、serialization hybridkem 6（window 单腿）、
   supports-modern 21×2。
4. **总账算术闭合**：keys 82,223 + 2,304（Hybrid generateKey 条件注册
   新键 684×2 + 468×2 全 Pass）− 4（T 键退役）= 84,523 ✓；P 77,569 +
   750（FIXED）+ 2,304 − 4 = 80,623 ✓；F 4,650 − 750 = 3,900 ✓；T 4 →
   0 ✓。
5. **subtest 级 drift 4 案归因**（M2-4 1T「案级 Timeout 键退役」同构）：
   failures_Hybrid-KEM / successes_Hybrid-KEM ×window/worker 双腿，各
   lost=1（Timeout 键）gained=684/468（真实子测试全 Pass）——组合算法
   generateKey promise 从未落定（T 形态）→ 三组合注册后全链落定。
6. **F 面具名归因（3,900 全量闭环，DC-5③ 同口径）**：全部 Ed448 降
   scope 域外红（D1 拍板②具名归因在案）——24 案 secure-curves
   eddsa_curve448 / derive cfrg curve448 面 + getPublicKey 4 案（8F 子
   测试逐条 Ed448 具名核验）。**未注册清单仅余 Ed448**（d1s5 起真值，
   此前为 Ed448/Hybrid）。r2 态瞬态 +1 = TurboSHAKE flake（r3 复绿）。

锚双 md5：gz 文件 `957ec62384623a62715e91a07bbb8699` / 内容（zcat）
`98fec48ca9b5576ee17d86f27038bc2c`
（[d1s5 锚](2026-11-03-d1s5-hybrid-kem-full.json.gz)）。对照基准更新为
d1s5 锚。

## 验证与门禁

1. **定向语料**（前轮 session 于 `fc0fa3837` 诊断账，make 同款 test-guard
   4/8/1800 直调 --json，filter 路径子串；全量 r3 于现行树逐位复证）：

   | filter | 账面 |
   |---|---|
   | Hybrid-KEM（generateKey 双族 + importKey） | **2736P / 0F**（exit=0） |
   | encap_decap（bits + keys） | **774P / 0F**（exit=0） |
   | hybridkem（serialization window） | **6P / 0F**（exit=0） |
   | supports-modern | **482P / 0F**（exit=0） |
   | getPublicKey | 152P / 8F——8F 全 Ed448 域外（拍板未注册，d1s3/d1s4 同款在案失败集；Hybrid 子测试全绿，d1s4 账面同位 20F 含 Hybrid，本轮消 12） |

2. **host 单测**：`cargo test -p zero-engine --lib hybrid_kem_tests`
   **5/5**（现行树本轮复跑 exit=0，guard --compile-first 6/16/600 包裹）：
   keygen_matches_draft_vectors（P256/X25519/P384 ek 逐字节）/
   decap_vectors_kat（三组合 shared_secret 全账）/ encap_decap_roundtrip /
   import_export_formats / import_validates_curve_point（P 系非曲线点
   拒绝、X25519 任意 32B 收）。
3. **runner 单测**：`cargo test -p zero-wpt-runner` **281P/0F/76I**
   （现行树本轮复跑，guard --compile-first 6/16/900，exit=0）。计数
   d1s4 账面 276 → 281（+5）：本切片零 wpt-runner 触面，增量来自 rebase
   入树的兄弟流 runner 侧测试（s249–s253 / gate28 窗口）。
4. **fmt/clippy**：`cargo fmt --all -- --check` 干净；zero-engine clippy
   （`--no-default-features --features quickjs`，`-D warnings`）零告警；
   make 全量 quickjs clippy 腿（14 crate --all-targets -D warnings）rc=0。
5. **make test**（现行树 `95d2f3699`，2026-11-03 10:45 systemd detached；
   workspace 腿止于**兄弟流预存红**，余腿按 Makefile 同款 test-guard 参数
   直调补跑，逐腿账）：
   - workspace 腿（guard 6/12/900）：63 测试套件全绿，**zero-engine lib
     3719P/5F**——5F 全部 `js_dom_bridge::tests` CSSOM 序列化守卫
     （p56_adopted_snapshot_restore_hook / p99s custom_property_case /
     stylesheet add_remove_rule r3276 / rule_style r2810 / write r2809），
     隔离单线程复跑同败（确定性红）；归因 **s249 `652a78e00`**（05:17
     落 main，「CSSOM 颜色归一 + _zwSerializeDecls 尾分号 Blink 形」）——
     序列化形变致旧期望串过期（断言 `rgb(9, 105, 218)` vs 实际
     `rgb(9, 105, 218);` 尾分号形态吻合）；d1s4 make test（01:00 基线
     3719P/0F）先于该提交，与本次零交集的 crypto 触面无关；按 run-rules
     #9 本流不代修（渲染流域碰头账，见 master.md 碰撞管理记录）。
   - quickjs clippy 腿：rc=0（14 crate -D warnings 零告警）。
   - renderer bin 0 测试（rc=0，main.rs 两树逐字节同为零测试——d1s4
     「renderer bin 54P」记账注误不溯源）/ renderer lib **368P/0F**；
     quickjs renderer bin 0 测试（rc=0）/ quickjs renderer lib
     **365P/1F**——`page_scripts::tests::mutation_arm_does_not_apply_
     across_document_reset_window`（「未观测滞留窗，时序假设失效」），
     隔离 --test-threads=1 复跑同败（确定性红，非负载 flake）：该腿在
     make test 序列中居 browser bin 预存红之后，d1s4 起从未达腿（第一次
     被 make test 级执行即翻出）；rebase 窗口 s249–s253（动态 src module
     执行通道 rev3 / 内联 module 延迟求值 / module 链接错误语义 / CSP
     eval 编译通道）全部 script 执行时序面，测试（10-17 落）时序假设
     与之失配归因同窗；V8 renderer lib 368P 全绿（同 shim 文本）排除
     本流 crypto 触面；同归渲染/script-sandbox 流域碰头账。
   - browser bin 腿 **741P/1F**：`default_actions_work_without_javascript`
     （d1s4 同数同签名预存红，forms/typing 面在途，归因在案不赘）；
     browser --tests 腿同 1F + 集成套件（cdp_* 三件等）全绿。
   - GPU adapter-only 腿：adapter available 本轮实跑 **96P/0F**
     （635 filtered）。
   - quickjs 测试腿（script-sandbox/webview/webview-demo/integration-
     tests/wpt-runner）：rc=0 全绿（776P/59I + 774P + 3P）。

## 行数治理与文件边界

crypto_hybrid.rs 648 行（新文件，< 2000 红线）/ crypto_asym.rs 1629 行 /
crypto.rs 1472 行。part01b.js / part02.js / part02a.js 为既有聚合文件存量
面（d1s4 记账口径同），本切片按既有结构追加未再拆分。

## 过程记录（落账轮接手）

前轮 session（09:08 rally）完成 WIP 提交（`fc0fa3837`）+ release runner
重建 + Hybrid canary + r2 全量落盘（09:53）后于证据占位节处中断
（rally events `exit_nonzero`，上下文耗尽）。本轮接手：开工 `git pull
--rebase`（带入 2 提交：`7fd64d148` s253 CSP、`e6ba14d7d` 渲染 DC-4
docs，与本切片触面零交集，干净 rebase）；现行树复验 host/runner 单测 +
fmt/clippy 全过；r3 复跑定锚（r2 flake 复归 + rebase 零位移一次证毕）；
make test workspace 腿翻出两族兄弟流预存红（engine lib 5 CSSOM + quickjs
renderer lib 1 时序——均为 s249–s253 窗口序列化/时序形变，make test 腿序
居前故既往轮未达，本流不代修、碰头账记录）；余腿按 Makefile 同款参数
直调补跑全绿（除 browser bin 同一枚在案预存红）。全程与 ZeroWeb-2
editing 电池（multitest→insertparagraph→delete.html）并行窗口，跨 clone
既有先例无扰。
