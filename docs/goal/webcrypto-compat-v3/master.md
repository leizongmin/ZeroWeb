# Web Crypto 兼容三期运行时控制面板

**状态**: Active——M1 完成（2026-11-01：语料预置 + 分域基线 + deps evidence +
选型征询落档）；M2 无依赖面**全部收口**：M2-1 门禁面 +543P、M2-2 散新案
归因 +12P、M2-3 worker 腿接线 +863P、M2-3b worker 腿排干修复 +12,196P、
M2-4 1T 销案（iframe 子文档动态 script 执行，13 子测试全 Pass，T 5→4）、
M2-5 raw 格式别名 +80P（raw-secret/raw-public ≡ raw，F 扫查误桶更正），
均零回归；余 D1 拍板后域切片 + M 末收口重定基
**最后更新**: 2026-11-02（语料面按 clone 漂移恢复 + main 漂移复验锚 CLEAN；D1 仍征询中）

---

## 当前状态

**语料面恢复 + 漂移复验完成（2026-11-02，[evidence](evidence/2026-11-02-corpus-restore-drift-reval.md)）**：
本 clone（ZeroWeb-3）wpt-data 为按 clone gitignored 本地态——转正子树 77 新文件
只在 ZeroWeb-2（M1–M2-5 执行 clone）取数过，本 clone 全量扫描仅 245 案（非删除，
目录 mtime 停 10-17 可证）。38 号脚本重跑恢复 **275 文件 = M1 记账精确一致**
（旧文件 mtime 零触碰幂等实证 + 跨 clone md5 抽样全同）；恢复后全量 r3
**360 案 / 81,851 keys = P 52,861 + F 28,986 + T 4**，对 m2s5 锚
`webcrypto-anchor-diff` **全 CLEAN（REGRESSED/FIXED/FLIPPED/DRIFT 全 0）**
——rebase 带入的 `dbd4ef9e8`（共享 runner 键位 + part05.js forms 面）与
`59685323f`（editing 面）对 webcrypto 全量账零回归实证，m2s5 锚对照基准不变；
runner 单测 276P/0F。D1 飞书征询线核实无回复（两轮在案不重复催办）。

**M2-5 raw 格式别名切片完成（2026-11-02，[evidence](evidence/2026-11-02-m2s5-raw-format-aliases.md)）**：
`raw-secret`/`raw-public` importKey/exportKey/wrapKey/unwrapKey 格式别名
（WICG webcrypto-modern-algos KeyFormat 扩展；对称面 raw-secret ≡ raw、
EC/OKP raw 面 raw-public ≡ raw、RSA 不映射）——**80 F→P**（raw_format_aliases
新族 + 旧 tentative 路径 × window/worker 双腿 4 案各 20），全量分母 360 案 /
81,851 keys = P 52,861 + F 28,986 + T 4，对 m2s4 锚逐 (case, subtest) 对照
**变化精确落 4 靶案、P→非P = 0、键集零漂移**。本切片同时更正 F 扫查一处
误桶（该案被按目录桶进 D1 域门控，实为只触既有算法的无依赖面；扫查文档已
加更正注记，supports-modern/getPublicKey 余红经同法复核维持域门控）。
零 Rust 触面（part01b.js +17 helper / part02.js +10 三接线）；runner 单测
276P/0F、clippy 干净；r1 V8 fatal 按 M2-1 flake 判据记账（r2 复跑干净）。
对照基准更新为 [m2s5 锚](evidence/2026-11-02-m2s5-full.json.gz)。

**D1 拍板前置证据加固完成（2026-11-02，三轮）**：

第一轮 **MSRV 编译预检**（[evidence](evidence/2026-11-02-dc2-msrv-compile-precheck.md)）：
deps evidence（11-01）为 metadata 级证据，本轮 /tmp 隔离 scratch + rustc
1.85.0 实编译验证，暴露三处记载失真并修正——① PQC 建议值回落
**ml-kem 0.3.3 / ml-dsa 0.1.2→0.1.1**（0.3.4/0.1.2 要求 rustc 1.87），
1.85 编译+冒烟全过；② **X448/Ed448 pre 线 1.85 无解**（elliptic-curve
^0.14 → crypto-bigint ^0.7.5 yanked-only 链），入树即 workspace MSRV
1.85→1.87，子问题②改三选（pre+1.87 / X448 走 stable x448 0.6.0 但
Ed448 无 1.85 签名路径 / 域降 scope）；③ KMAC **option b 可行性实证**
（手写框架 ≡ sha3-kmac oracle 逐位 ×4 用例，encode_string 取 bit 长度
约定定谳）+ **option a 短 key 合规缺口**（MIN_KEY_SIZE=16B 拒 spec 合法
输入）。XOF 五连/ocb3/chacha20poly1305/argon2 维持建议值。真正新增共存
分叉仅 aes 0.9（ocb3）与 rand_core 0.5（x448 stable）；rand_core 0.10/
getrandom 0.4 workspace 树已在本。仓库零代码触面。

第二轮 **先行四域语料面预检**（[evidence](evidence/2026-11-02-d2-domain-slice-corpus-face-precheck.md)）：
编译通过 ≠ 语料参数面可算——argon2/AES-OCB/ChaCha20-Poly1305/XOF 四域
真实 WPT 语料向量在 1.85.0 scratch 探针**逐字节复现全过**（OCB 三档截断
tag + 15B nonce、ChaCha 9 组 AAD 矩阵、XOF 41 断言含 K12 定制串与
TurboSHAKE const 泛型 DS、cSHAKE outputLength=0）。先行四域拍板即可开工。

第三轮 **PQC/KMAC/X448 语料面预检**（[evidence](evidence/2026-11-02-d2-pqc-kmac-x448-corpus-face-precheck.md)）：
D1 拍板子问题①③②的语料真实向量面 1.85.0 逐字节复现——ML-KEM
seed 确定性 + decap KAT 三集全过、ML-DSA verify KAT 三变体全过
（det_sign ≠ corpus 已归因：驱动未开 katFirst，sign 只查 roundtrip）、
KMAC 双案 NIST 6/6 全过且 **option a MIN_KEY_SIZE 缺口语料实锤**
（symmetric_importKey 16B/24B/32B 三档全跑，KMAC256 短 key import 必红）、
X448 option (b) KAT + 5/5 小阶点拒绝成立（host 两条义务：import 用
unchecked 解码 + derive 全零后检查）。1.85 可探域语料面预检全部闭环。

**DC-3（收敛基线）达成判定（2026-11-02）**：四项判据逐项对账——① 新域 +
散新案分母逐案分类账落 evidence（[F 归因扫查](evidence/2026-11-02-m2s4-f-attribution-sweep.md)
81,851 keys 全归因闭环 + [M2-5 更正](evidence/2026-11-02-m2s5-raw-format-aliases.md)
逐案算法面复核）；② 门禁面语义落地（M2-1 supports 门禁 +543P，
getPublicKey 语义面在位、余红为域内新算法未注册的域门控红）；③
`[SecureContext]` 暴露门禁语义零回退（stable-root 157 案零红，m2s5 锚）；④
worker 腿双腿账落档（M2-3 +863P / M2-3b +12,196P）。**DC-3 达成**；
域内余红翻绿随 D1 域切片承接，不构成 DC-3 缺口。DC 进度：DC-1 ✓、
DC-3 ✓、DC-2 差用户拍板、DC-4/DC-5 待 M 末（前置预检均已落档）。

**M2-4 1T 销案完成（2026-11-02，[evidence](evidence/2026-11-02-m2s4-1t-iframe-dyn-script.md)）**：
algorithm-discards-context（一/二/三期连续 T 案）**13 子测试全 Pass，
案级 Timeout 键退役，T 5→4**；全量分母 360 案 / 81,851 keys = P 52,781 +
F 29,066 + T 4，红线 P→非P 0、F 逐位全等、r1/r2/r3 三跑逐项全等。
根因 = iframe 子文档 body（`_zwMakeIframeDoc` 工厂合成体）SCRIPT append
静默不执行，主文档动态面（s95/s97）不覆盖该站；修复 = 工厂 appendChild
出口包装 + `_zwRunIframeDynamicScript` 执行体（run-once 双表记账/s113
分类门/R206 同构域绑定，+68 行纯插入 part05.js）。dom 通道双二进制
2×2 A/B 共享键零回归（差异全落 Range-mutations 报告视界漂移 +
replaceChildren T 案相位漂移两类已记账噪声）。余 T 4 = Hybrid-KEM
generateKey ×4（D1 域门控）。对照基准更新为
[m2s4 锚](evidence/2026-11-02-m2s4-full.json.gz)。**M2 无依赖面四切片
（门禁面/散新案/worker 腿/1T）至此全部收口**。

**F 面具名归因扫查完成（2026-11-02，[evidence](evidence/2026-11-02-m2s4-f-attribution-sweep.md)）**：
m2s4 账全部非 P 逐案族谱归因——F 29,066 = 旧 `.tentative.` 路径 14,150
（收口重定基退役）+ 新族 14,916（D1 域门控，逐算法分布表落 evidence）
+ stable-root **零红**；T 4 = Hybrid-KEM。签名扫查无散在 shim 缺陷红面、
无无依赖可修簇（唯一潜伏项：失败路径 `algorithm.name` 二次读，无语料
面，记录不修）。**DC-5③ 归因闭环预闭合成立**，M2 收口结论复核维持。

**DC-5② 前置预检完成（2026-11-02，[evidence](evidence/2026-11-02-dc5-continuity-precheck.md)）**：
一期终态锚 → m2s4 单跳对账——123 案全在位，共享 keys 仅 245 个迁移
**全部 F→P**（精确落 M2-1 门禁面三案 supports/supports-modern/
getPublicKey.tentative 171+53+21）+ 1T 案级 Timeout 键退役 + 13 新 Pass，
**P→非P = 0**——五切片两两零回归对账的传递链单跳复验成立，无累积漂移。
退役清单预编账 = 一期分母内 `.tentative.` 路径 49 案 / m2s4 态 7,340 keys
（m2s4 全账 98 案 / 14,150 F），M 末收口对终态锚重算为准。

**DC-4 前置预检完成（2026-11-02，[evidence](evidence/2026-11-02-dc4-coverage-precheck.md)）**：
`wptfyi_gap.py coverage` 对 m2s4 锚机制验证通过（可疑 0、缓存零写入）——
coverage 分子 27,209P（二期 DC-4 时 13,247），**worker 腿结构性闭合**
（125 worker 结果经 `.worker.html` 变体规则全部入分子，二期 uncovered
13,812 桶清账）；unmapped 108 = 98 tentative（DC-5② 退役面吻合）+ 10
分片变体主文件（25,042P 全绿隐形，通道基建挂账维持）；universe 未命中
82 全部具名（sliced 74 + idlharness×6 + secure_context 1 + sharedworker
1 新形态通道域排除）。分子内非 P = F 14,916 + T 4（全 D1 域门控）+
形态差 3,334。顺带更正 F 扫查 prose 新族聚合笔误 14,912 → 14,916
（闭式 29,066−14,150，本预检独立复算吻合）。

**M2-3b worker 腿排干修复完成（2026-11-02，[evidence](evidence/2026-11-02-m2s3b-worker-leg-drain.md)）**：
hkdf/pbkdf2 worker 腿翻绿，**净增 +12,196 子测试位（全 Pass）、T 7→5**，
全量分母 360 案 / 81,839 = P 52,768 + F 29,066 + T 5；红线 P→非P 0、
F 逐位全等、窗口腿 181 案与 m2s3 锚零漂移。**归因修正**：M2-3 挂账
「worker 消息投递空闲饥饿」被 zzprobe13 仪器化探针推翻（6011 条
postMessage 2.6s 内全部入队——投递从未饥饿），真实机制 = runner timer
stub 队列逐轮 partition+sort 二次失速（per-task 模式 1 fire/轮，90s 仅
2070/6017）+ sandbox 逐 execute 固定开销（~105ms/轮 @ 数千条目堆）。修复
四件套落 runner 基建共享面（sorted-on-insert / clearTimeout 真移除 part01
probe-gated 接线 / flood 门控 + flood cap 256 / probe results delta 化），
engine 生产路径零触碰，`cargo test -p zero-wpt-runner` 276P/0F。
对照基准更新为 [m2s3b 锚](evidence/2026-11-02-m2s3b-full.json.gz)。

**M2-3 worker 腿接线完成（2026-11-02，[evidence](evidence/2026-11-02-m2s3-worker-leg.md)）**：
全子树 179 个 `.any.js` 案 window/worker 双腿执行（runner 新增
testharness_webcrypto.rs 独立模块 + part05.js worker 全局 crypto/镜像补齐）。
全量新分母 360 案 / 68,645 子测试 = P 40,572 + F 29,066 + T 7；window 腿与
M2-2 锚逐字节全等（零回归）；worker 腿 P 14,186（接线首轮 +863）与同源
window 腿状态族 parity 177/179。配套修复导入源内联扁平化
（`_zwFlattenToplevelDeclarations`——strict direct eval 闭域 + 顶层 let/const
eval 词法闭域双缺口，supports×4/wrapKey/derived_bits_length 等 6 族 worker
腿翻绿；全语料静态核查对其余通道 no-op）。worker 残 T 4 = hkdf/pbkdf2
**worker 消息投递空闲饥饿**（引擎侧缺陷已归因——忙 worker 投递跟 op 节奏
12 条/s、闲 worker 退化 ~0.7s/条，zzprobe10 最小复现 2110 注册 vs 42 完成，
窗口腿同案全绿；修复属 script-sandbox/event-loop 独立切片）+ Hybrid-KEM 2
（域门控随 D1）。对照基准更新为 [m2s3 锚](evidence/2026-11-02-m2s3-full.json.gz)。

**M2-2 散新案归因收敛完成（2026-11-01，[evidence](evidence/2026-11-01-m2s2-scattered-cases.md)）**：
export-key-error-order（2/6→6/6，错误检查顺序——`_zw_exportRegistered`
注册检查前置于 extractable，spec exportKey 步 6/7、wrapKey 步 11/12）+
sharedarraybuffer（0/8→8/8，BufferSource 未拒 SAB——`_zw_isSharedBuffer`
toString 标签探测 + `_zw_bufToBytes` 顶部 throw TypeError，单点守卫覆盖
30 处调用面）。对 M2-1 锚**零回归**（12 变化全 F→P 精确落两靶案，P→非P
0 例）；新分母 P 26,386 + F 14,536 + T 3；**散新案 5 案全绿**，M2 无依赖
面第一项收口。对照基准更新为 [m2s2 锚](evidence/2026-11-01-m2s2-full.json.gz)。

**M2-1 门禁面切片完成（2026-11-01，[evidence](evidence/2026-11-01-m2s1-gate-slice.md)）**：
`SubtleCrypto.supports(...)` 静态门禁（part02a.js 新增，自包含 IIFE）+
`getPublicKey(key, usages)` 实例方法（part02.js，零 Rust 改动）。门禁面 6 案
792 子测试 0P → **543P**（supports 两文件全绿 371/371；supports-modern
130/241、getPublicKey 42/80 余红全部域门控待 D1 按域切片）；对 M1 基线逐
(case, subtest) 对照 **零回归**（543 变化全 F→P，P→非P 0 例，精确落 6 触面
文件）；全量新分母 40,925 = P 26,374 + F 14,548 + T 3。调试定案三根因：
REG 键归一形态（V1_5/ED25519）、HMAC keygen 缺省长度 = hash 块大小
（512/1024）、getPublicKey OKP `_raw` 契约（raw 公钥 32B 非 SPKI DER）+
错误检查顺序（算法域先于 key type）。首轮全量 runner segfault 一次
（WSL2 环境观察，同命令复跑干净 + 结果逐位一致，按 flake 判据记账）。

**M1 完成（2026-11-01，[evidence](evidence/2026-11-01-m1-corpus-and-baseline.md) /
[deps evidence](evidence/2026-11-01-m1-deps-evidence.md)）**：

1. **语料预置（DC-1）**：38 号脚本转正子树（`cb6cc97ada`，= HEAD 子树逐字节）
   全子树新增式拉取——上游 216 blob 应拉 210 全在位、0 缺失、6 元数据正确排除；
   本地 275 = 198 旧（mtime 证零触碰）+ 77 新；65 个上游已删旧 `.tentative.`
   路径文件按新增式保留（收口重定基退役）。散新案 5 案全在位。
2. **分域基线（DC-3 分母侧）**：181 案 / 40,925 子测试 = P 25,831 + F 15,091
   + T 3（全绿案 76）；新增面 58 案 / 7,816 子测试（新族 53 案窗口腿近全红：
   modern-algos P=2、secure-curves P=18；散新案 5 案 P=23/F=12，其中 3 案
   已全绿）。**零回归**：与二期收口锚逐 (case, subtest) 对照 33,109 对状态
   变化 0 例。两连跑状态逐项全等（T 案消息文本相位漂移，见 evidence）。
   T 3 案 = 谱系挂账 1T（algorithm-discards-context）+ Hybrid-KEM generateKey
   2 案（组合算法 promise 未落定，随域切片销）。
3. **deps evidence（DC-2 前半）**：10 域候选逐域落证。要点：sha3 0.12 实测
   **不含 KMAC**（入口候选前提证伪，KMAC 改两案交拍板）；kangarootwelve
   0.1.3 证伪（个人维护 + MSRV 1.86 > 1.85），官方 XOFs 家族（shake/cshake/
   k12/turboshake 全 MSRV 1.85）替代；ocb3 0.2.0 已入 RustCrypto AEADs
   官方仓（入口预判更正）；ml-kem/ml-dsa 候选各出一日补丁版（建议 0.3.4/
   0.1.2）；X448/Ed448 仅 pre 线活跃（甄别从严交拍板）。新 crate 建议合计
   12–13 个。

**下一步（D1 拍板后）**：M2 无依赖面已全部收口（M2-1/2/3/3b/4/5，见上
各段；M2-5 系逐案算法面复核从 F 扫查误桶中捞回的无依赖案，supports-modern/
getPublicKey 余红经同法复核维持域门控——无新依赖可推进面已用尽的结论现以
逐案算法面复核为据）——余下工作全部依赖 D1 选型拍板。
先行四域（argon2/AES-OCB/ChaCha20-Poly1305/XOF）已完成**语料面预检**
（[2026-11-02-d2-domain-slice-corpus-face-precheck.md](evidence/2026-11-02-d2-domain-slice-corpus-face-precheck.md)
，MSRV 编译预检的硬化补充）：四域真实语料向量在 rustc 1.85.0 scratch 探针
逐字节复现全过，D1 拍板即可开工无预检残留；OCB/ChaCha 暴露 ctutils 传递
MSRV 坑需钉版（aes =0.9.2 + ctutils =0.4.2），K12 crate 名更正为 `k12`，
XOF host 回调需扩参（outputLength/customization/domainSeparation）。

**锚对照工具化（2026-11-02，D1 等待期工程预制）**：`scripts/webcrypto-anchor-diff.py`
——两份 runner 锚逐 (case, subtest) 对照，三查口径（REGRESSED P→非P 阻断
exit 1 / FIXED 按案聚计 / FLIPPED 记账 / DRIFT case+subtest 双级 exit 2 /
未知 status 字面 exit 3 防契约漂移），`.json`/`.gz` 直读。四重验证：
正向 m2s4→m2s5 复现 M2-5 落账逐位一致（80 FIXED 落 raw_format_aliases
4 案）、反向 80 REGRESSED exit 1、自对照 clean、注入未知字面 exit 3。
后续每轮域切片与 M 末重定基（DC-5② 终账）复用，替代逐轮一次性手写对照。
拍板后按 deps evidence 建议定序（依赖就绪度：aes-ocb/chacha20poly1305/
argon2 等 AEADs+password-hashes 单 crate 域先行，XOF 五连引入次之，
ML-KEM/ML-DSA 再之，X448 pre 与 KMAC 两案随拍板；门禁面注册表 REG 与
getPublicKey 域面随域切片逐行扩表翻绿）；域切片完成后进 M 末收口重定基
（DC-4/DC-5，终账切 HEAD 布局全量双腿账，旧 `.tentative.` 路径退役）。

- 输入：二期 [DC-2 甄别清单](../archive/webcrypto-compat-v2/evidence/2026-10-31-dc2-promotion-screening.md)（逐域分母/建议范围/依赖候选）
- 上游复证（2026-11-01、2026-11-02 二次）：11-02 HEAD `93d221d6…`，
  WebCryptoAPI 子树 `084495e783…` 与甄别时（`cb6cc97ada…`）**逐字节相同**
  （两次复证零漂移）——tentative=0、未转正域 0；pin→HEAD diff 80 增/65 删
  已逐文件枚举（入口文档上游事实节）。M 末重定基「HEAD 布局 = 甄别布局」
  前提保持。
- 底线：一期 33,109 分母零回归（25,788P/7,320F/1T）——**本轮基线复证保持**
  （P→非P 0 例）；重定基后由 DC-5② 终账承接。

## 与兄弟 goal 的边界

- **25 号 html-semantics-compat（并行在跑）**：共享触面
  `js_dom_shim/part01.js`、`script_gen.rs`、`testharness.rs`；crypto 实现面
  （part01/01b/02 `__zw_crypto_*`）与其 forms 切片目录级分域但同文件——
  切片落账前对账 main 最新触面，切片号按 s162 撞号先例。
- **worker 腿接线触 runner/workers 通道基建**（testharness runner 侧），
  与 25 号流共享 testharness.rs——接线切片落账前对账。
- **33 号 editing-compat（ZeroWeb-3 工作区并行在跑）**：域不相交；合流经
  main，切片落账前 merge-base 对账。
- 23 号 selection-compat：无共享面。

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | 10 域新依赖选型：deps evidence 已落档（[2026-11-01-m1-deps-evidence.md](evidence/2026-11-01-m1-deps-evidence.md)）并经 **MSRV 编译预检修正**（[2026-11-02-dc2-msrv-compile-precheck.md](evidence/2026-11-02-dc2-msrv-compile-precheck.md)）与 **先行四域语料面预检**（[2026-11-02-d2-domain-slice-corpus-face-precheck.md](evidence/2026-11-02-d2-domain-slice-corpus-face-precheck.md)：argon2/OCB/ChaCha/XOF 真实语料向量 1.85.0 逐字节复现全过；OCB/ChaCha 需 `aes =0.9.2 + ctutils =0.4.2` 钉版，K12 crate 名更正 `k12`）与 **PQC/KMAC/X448 语料面预检**（[2026-11-02-d2-pqc-kmac-x448-corpus-face-precheck.md](evidence/2026-11-02-d2-pqc-kmac-x448-corpus-face-precheck.md)：ML-KEM/ML-DSA 语料 KAT 全过、KMAC 双案 NIST 6/6、X448 option b 语料面成立；1.85 可探域语料面预检全部闭环），修正后建议值 12–14 crate（全 RustCrypto 官方系为主）。**征询中**：① 总体选型建议值（PQC 两域修正为 ml-kem 0.3.3 / ml-dsa 0.1.1——0.3.4/0.1.2 需 rustc 1.87；两域语料 KAT 已全过）是否批准；② X448/Ed448 pre 线**三选**——(a) pre + workspace MSRV 1.85→1.87（全能力，pre.15 含 sign/），(b) X448 走 stable x448 0.6.0（语料面成立：KAT 逐字节 + 5/5 小阶点拒绝，host 两条义务——import 用 unchecked 解码、derive 全零后检查）且 **Ed448 无 1.85 签名路径**（0.9.0 无 signing 面 / ed448 0.5.0 仅类型），(c) 域降 scope；③ KMAC 两案——option a sha3-kmac 0.3.0（**MIN_KEY_SIZE 缺口已语料实锤：symmetric_importKey 16B/24B/32B 三档全跑，KMAC256 短 key import 必红** + sha3 双栈），option b 手写叠官方 cshake（**NIST 语料 6/6 逐字节全过 + ≡ oracle 逐位相等**，无上述缺口，需 NIST 向量单测随切片落仓）；④ ~~ml-kem 0.3.4 / ml-dsa 0.1.2 版本升替~~（**被 1.85 约束否决**，见②修正）。拍板前收敛切片仅限门禁面/散新案/worker 腿/1T（无新依赖域，已全部收口） | 已飞书征询在案（GB-20261101，msg `om_x100b61408d1ee4a0cffa63121bb8f1e`）；11-02 M2 收口进展+催办通知在案（msg `om_x100b614f36f5e8a4cd31575cf2facd6`，附四项拍板子问题）；**待拍板**（evidence 已齐并经编译预检 + 两轮语料面预检三重修正） |

## 碰撞管理记录（run-rules §9）

- 2026-11-01 立项时点：25 号 html-semantics-compat 并行在跑（s209 刚落
  main `785e8a166`，共享触面 part01.js/script_gen.rs/testharness.rs）；
  33 号 editing-compat 在 ZeroWeb-3 并行在跑。长跑/CI 按质量门禁守卫
  （`docs/engineering/quality-gates.md`），CI 发射前查在跑实例。
- 2026-11-01 M1 轮：runner/shim 零代码改动（语料 fetch + 基线 + docs），
  无碰撞面；`git pull --rebase` 后基于 `434e36f44` 推进。
- 2026-11-01 M2-1 轮：触面 `js_dom_bridge.rs`（part02a include_str 挂载
  +12 行）、`js_dom_shim/part02.js`（getPublicKey +63 行）、
  `js_dom_shim/part02a.js`（新增）；25 号流共享面（part01.js/
  script_gen.rs/testharness.rs）零触碰，main 最新共享面触点仍为 s209
  `785e8a166`（先于本流 M1 基点 `de7c4aea8`），无对账冲突。
- 2026-11-01 M2-2 轮：触面 `js_dom_shim/part01b.js`（+27 行：SAB 判定/
  export 注册面 helper）、`part02.js`（+18 行：错误序前置检查）；共享面
  （part01.js/script_gen.rs/testharness.rs）零触碰，part01b 为 crypto 域
  文件（非共享清单）。M2-1 推送时 rebase 带入 s218（part04/part06，无
  交集）与 7c516159b（webcrypto master.md D1 行，合并共存已核对）；
  本轮开工 `git pull --rebase` 已是最新。
- 2026-11-02 M2-3 轮：触面 `tests/wpt-runner/src/testharness_webcrypto.rs`
  （新增 ~214 行）、`testharness.rs`（webcrypto 块 106 行迁出 + CASE_TIMEOUT/
  any_js_window_wrapper/wpt_js_has_long_timeout 升 pub(crate)）、
  `testharness_workers.rs`（wpt_meta_globals 升 pub(crate)）、`main.rs`
  （mod 声明 + 调用点改道）、`js_dom_shim/part05.js`（+114/-4：worker
  crypto 镜像 + `_zwFlattenToplevelDeclarations`）。共享面 testharness.rs
  落账前对账：fetch 远端 `4f044f3fe..22acb33eb`（insertText typing 面 +
  响应式图片，part04/part06 域），与本切片触面 diff 零交集。
- 2026-11-02 M2-3b 轮：触面 `tests/wpt-runner/src/testharness.rs`（timer
  stub 重写 + take_probe flood 门控/delta + 探测环累计追加，诊断补丁随
  修复一并还原）、`crates/engine/src/js_dom_shim/part01.js`（clearTimeout/
  clearInterval probe-gated stub 队列接线 +14 行）、
  `testharness_webcrypto.rs`（仅 fmt 归位 2 处——75087fa30 自身漂移）。
  共享面为全部 testharness 通道与 25 号 html-semantics 流共用；落账前
  pull --rebase 对账。同 workspace 观测到兄弟流孤儿验证
  （`/tmp/zw-edit-v/gate16.sh` → testharness-editing-run，02:24 起
  systemd 接管）使用本轮重建二进制——与其零文件冲突，不干预。
- 2026-11-02 M2-4 轮：触面 `crates/engine/src/js_dom_shim/part05.js`
  （+68 行纯插入：`_zwMakeIframeDoc` 工厂 appendChild 包装 +
  `_zwRunIframeDynamicScript` 执行体，作用域 iframe 子文档专属）。
  开工 pull --rebase 带入 `4dfdf6b78..e0d6948c0`（25 号流 F2
  insertParagraph/caret 簇 5 提交，触 part01.js/part04/part06/css 面——
  与本切片触面 diff 零交集）；25 号共享清单（part01.js/script_gen.rs/
  testharness.rs）本轮零触碰。engine 共享面回归按 M2-3b 先例以 dom
  通道双二进制 2×2 A/B 承接（见 M2-4 evidence 验证阶梯 5）。
- 2026-11-02 DC-5 预检轮：纯 docs 改动（连续性预检 evidence + 本文件），
  零代码触面，无碰撞面。
- 2026-11-02 上游复证轮：纯 docs 改动（入口文档上游事实节 + 本文件上游
  复证行），零代码触面，无碰撞面。
- 2026-11-02 DC-4 预检轮：纯 docs + /tmp 离线对账（coverage 工具只读
  m2s4 锚与 wptfyi 缓存，仓库与缓存零写入），无碰撞面。
- 2026-11-02 deps 编译预检轮：纯 docs + /tmp 隔离 scratch（rustc 1.85.0
  minimal 工具链 + test-guard 包裹；仓库 Cargo.toml/源码零触碰，workspace
  target 零写入），无碰撞面；在跑兄弟流 testharness-editing-run（33 号
  editing）不受干扰。
- 2026-11-02 锚路径修正轮：纯 docs 改动（入口文档依赖约束节 33,109 锚
  目录笔误 `webcrypto-compat-v2` → `webcrypto-compat` 一期归档目录，
  终态锚补精确文件名 2026-10-17-m6-residual-full.json.gz），零代码触面，
  无碰撞面。锚本体本轮复验：m6-residual 锚 123 案 25,788P/7,320F/1T
  = 33,109 逐位成立；m2s4 锚 360 案在位。D1 维持征询等待（git 账面零
  变化，两轮征询在案不重复通知）。
- 2026-11-02 M2-5 轮：触面 `crates/engine/src/js_dom_shim/part01b.js`
  （+17 行：`_zw_normalizeFormatAlias` family 条件归一 helper）、
  `part02.js`（+10/-1：importKey/exportKey 入口归一 + unwrapKey 白名单
  放行）。共享清单（part01.js/script_gen.rs/testharness.rs）零触碰；
  25 号/33 号流共享面零交集。开工 pull --rebase 已是最新（fa5444f60）；
  落账 rebase 带入 `7ddf00c39`（33 号流 F6 delete/forwardDelete，触
  part01f 新分片 + part01d/part06 editing 面，与本切片触面 diff 零交集，
  按 M2-3 先例不复跑）。
- 2026-11-02 语料恢复 + 复验轮：零代码触面（38 号语料脚本重跑 + docs）。
  开工 rebase 带入 `dbd4ef9e8`（25 号流 radio ArrowRight——触共享
  testharness.rs 键位四向 + part05.js forms 面，与本流 part05.js worker
  crypto 面同文件目录级分域）与 `59685323f`（33 号流 D4 typing——触
  part01/01f/06 editing 面）：两触面按 M2-4 先例以 m2s5 锚全量复验承接，
  r3 逐位 CLEAN 零回归（见当前状态段）。开工时工作面有 33 号流在途草稿
  （part01f.js delKeepChainShells），已 stash 记账
  （`typing-style-round-1102`）不硬解。
- 2026-11-02 复验续轮：r3 基点（`dbd4ef9e8`）后 main 又入 `5248566a3`
  （css_vars +90）与 `aac5e481f`（part03/part05.js switch 反射——与本流
  part05.js worker crypto 面同文件目录级分域，变更臂 scoped
  `input.switch` forms 面）两笔 engine 修复；r4 全量对 m2s5 锚再证
  **逐位 CLEAN**（360 案/81,851 keys = P 52,861 + F 28,986 + T 4，
  REGRESSED/FIXED/FLIPPED/DRIFT 全 0），runner 单测 276P/0F，m2s5 锚
  对照基准维持；零代码触面无碰撞。D1 飞书线再核无回复（pos 851 后
  无用户消息，两轮征询在案不重复催办）。
- 2026-11-02 复验续轮 2：`70fc23d7b`（25 号流 radio-focus 键盘链 miss
  回落——触**共享 testharness.rs** +19，本流通道执行底座同文件）；
  r5 全量对 m2s5 锚再证 **逐位 CLEAN**（总账同 r4），runner 单测
  276P/0F，对照基准维持；零代码触面无碰撞。D1 飞书线三核无回复
  （chat 顶仍 pos 851 全 bot 通知，维持不重复催办）。
