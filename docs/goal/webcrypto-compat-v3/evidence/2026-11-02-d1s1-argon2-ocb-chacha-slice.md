# D1 域切片第一轮：Argon2 + AES-OCB + ChaCha20-Poly1305（2026-11-02）

> D1 拍板（2026-11-02 用户会话四子问题拍板）后首批域切片：单 crate 域
> 三连——password-hashes（Argon2d/i/id）+ AEAD 两域（AES-OCB、
> ChaCha20-Poly1305）。deps 引入 → host 实现 → shim 接线 → REG 扩表 →
> 定向翻绿 → 全量锚对照零回归。对照基准
> [m2s5 锚](2026-11-02-m2s5-full.json.gz)；本轮新基准
> [d1s1 锚](2026-11-02-d1s1-argon2-ocb-chacha-full.json.gz)。

## 依赖（D1 建议集批准面，Cargo.lock 落位验证）

workspace 新增 3（`Cargo.toml`）：`argon2 0.6.0`（default-features=false +
alloc）、`ocb3 0.2.0`、`chacha20poly1305 0.11.0`（default-features=false +
alloc）。engine 成员级钉版 2（`crates/engine/Cargo.toml`）：`aes =0.9.2`
（package 重命名 `aes09`——与既有 aes 0.8.4（aes-gcm/cbc/ctr 面）双 major
共存为 D1 拍板明示方案，钉版防 cargo update 漂入 1.87+）、
`ctutils =0.4.2`（ocb3/cipher 0.5 传递依赖，0.4.3+ 要求 rustc 1.87）。
Cargo.lock 终态五项逐一验证：aes 0.9.2 / argon2 0.6.0 /
chacha20poly1305 0.11.0 / ctutils 0.4.2 / ocb3 0.2.0（既有 aes 0.8.4
不动）。MSRV 1.85 编译过（MSRV 预检
[2026-11-02-dc2-msrv-compile-precheck.md](2026-11-02-dc2-msrv-compile-precheck.md)
先行，本轮实装入树复证）。

## 改动（9 文件，+768/-31 + Makefile 一腿校准）

- `crates/engine/src/js_dom_bridge/crypto.rs`（+225）：三个 host 函数
  （`crypto_subtle_aes_ocb` / `crypto_subtle_chacha20_poly1305` /
  `crypto_subtle_argon2`），csv wire 约定与既有面同构（失败 → 空串）。
  OCB：iv 6..=15 校验 + tagLength ∈ {64,96,128} 缺省 128 + ocb3
  NonceSize/TagSize 编译期密封集（6..=15 × 1..=16）三层宏分派
  （by_key! 16/24/32 → by_tag! 64/96/128 → iv.len() 6..=15 十档），
  aead 0.6 `encrypt_inout_detached/decrypt_inout_detached` InOut 面返回
  tag 拼接。ChaCha：key 32B + iv 恒 12B、tag 恒 128。Argon2：ARGON2D/I/ID
  → Algorithm、Version 0x13、ParamsBuilder（m/t/p）+ AssociatedData +
  new_with_secret（secretValue 非空时），`hash_password_into`。
- `crates/engine/src/js_dom_bridge/callbacks.rs`（+62）：三回调注册
  （aes_ocb 6 参 / chacha20_poly1305 5 参 / argon2 9 参）。
- `crates/engine/src/js_dom_shim/part01b.js`（+140）：归一化三新臂
  （AES-OCB length 透传 / CHACHA20-POLY1305 → 规范拼写回写 / ARGON2x →
  规范拼写 + 成员浅拷贝）；`_zw_ocbCall` / `_zw_chachaCall`（spec 步骤
  校验 + host 调用）；`_zw_argon2Derive` helper（length ≥32 且 8 倍数、
  version 缺省 19 ≠19 → OperationError、parallelism ∈ [1,2²⁴-1]、
  memory ≥ 8·parallelism、passes ≥ 1）；`_zw_exportRegistered` 扩两 AEAD。
- `crates/engine/src/js_dom_shim/part02.js`（+117/-4）：encrypt/decrypt
  /importKey/generateKey/exportKey/wrapKey/unwrapKey/deriveBits 八运算
  分派臂（Argon2 importKey 专用早臂：format 仅 raw-secret →
  NotSupportedError、usages ⊄ {deriveBits,deriveKey} → SyntaxError、
  extractable ≠ false → SyntaxError、空 usages → SyntaxError（spec 公共
  步骤 10，与 PBKDF2/HKDF 同断言）；ChaCha generateKey 固定 32B 随机）。
- `crates/engine/src/js_dom_shim/part02a.js`（+39）：REG 五新行（AES-OCB /
  CHACHA20-POLY1305 全加密面五 ops；ARGON2D/I/ID 派生面两 ops）+
  validateSeal 两新臂（OCB iv 6..=15 + tagLength {64,96,128}；ChaCha iv
  恒 12 + tagLength 须 128）+ validateDerive Argon2 臂（length 域 +
  required 成员存在性）。
- `Cargo.toml` / `crates/engine/Cargo.toml` / `Cargo.lock`：见上节。
- `Makefile`（workspace 测试腿一行）：per-proc-mem 4→6 / total-mem 8→12
  阈值校准 + 实测依据注释——A/B 归因既有结构边际（见门禁补记），非本
  切片代码面；真修归 runner 通道碰头账。
- `crates/engine/src/js_dom_bridge/callbacks.rs` 另有 fmt 归位一处
  （`cargo fmt --all` 调用参数换行）。

## 切片内修复（定向语料暴露，三处）

1. **空明文解密 wire 歧义（ChaCha 3 组 → 0）**：round-trip
   encrypt(空明文) → decrypt(ct=tag-only) 成功返空字节串，与「host 失败
   → 空串」wire 约定撞车 → 单逗号哨兵（host decrypt 空明文返 `","`，
   shim `_zw_ocbCall`/`_zw_chachaCall` 特判还原空 Uint8Array）。AES-OCB
   同构预防性处理；GCM 一期面不动（零回归）。
2. **Argon2 importKey 空 usages（36 组 → 0）**：spec 17.4.2 无空 usages
   检查，但 corpus 共享驱动（symmetric_importKey.js testEmptyUsages）对
   全注册算法断言 SyntaxError——与 PBKDF2/HKDF 既有行为（m2s5 锚全
   Pass）对齐，早臂补 `aU.length === 0` → SyntaxError。
3. **wrapKey/unwrapKey name 匹配（158 组 → 0）**：wrapAlgorithm 归一化
   后与 key.algorithm.name 比较，ChaCha 规范拼写非全大写（
   `CHACHA20-POLY1305` vs `ChaCha20-Poly1305`）恒错配 → 两侧折
   upperAscii 比较（注册表各名折叠互异，等价 spec 归一后 name 相等；
   AES 家族/RSA-OAEP 行为零变化）。

## 验证

1. **定向语料**（`make testharness-webcrypto FILTER=<案名>`，test-guard
   包裹）：argon2 derive_bits 16/16、aes_ocb encrypt_decrypt+generateKey
   1108P/0F、chacha encrypt_decrypt+generateKey 1984P/0F（哨兵修复后）、
   serialization aes-ocb 12P / chacha 8P、importKey AES-OCB 288P /
   Argon2 180P / ChaCha 96P（修复后）、wrapKey_unwrapKey 800P/0F（修复
   后）、symmetric_importKey 1212P/0F（既有算法面零变化）、
   supports-modern 280P/202F（202F 全 ML-DSA/ML-KEM/Hybrid-KEM PQC 域
   门控红，ChaCha 行 52 全 Pass）。
2. **全量语料**：360 案 / **82,065 keys = P 60,395 + F 21,666 + T 4**
   （m2s5 基线 81,851 = P 52,861 + F 28,986 + T 4）。rebase 到
   origin/main `f8925c1e7` 后**复跑复证**：总账逐位一致，锚对照
   REGRESSED/FIXED/FLIPPED/drift 全部与 rebase 前逐位复现（FIXED 同
   48 案同计数、drift 同 10 案同清单）——上游 +8 提交（typing/select
   + CI + docs）对本切片账面零扰动。[d1s1 锚](2026-11-02-d1s1-argon2-ocb-chacha-full.json.gz)
   以 rebase 后复跑落盘。
3. **锚对照**（`scripts/webcrypto-anchor-diff.py` m2s5 → head）：
   **REGRESSED（P→非P）= 0**（零回归红线）、FLIPPED = 0、case 级
   lost/gained = 0。FIXED（非P→P）**6,216 落 48 案**（tentative 路径
   3,108/24 案 + modern 新族 3,108/24 案，逐案对称；按域 Argon2 184 /
   AES-OCB 3,924 / ChaCha20-Poly1305 2,088 + supports-modern ChaCha 行
   20——全部三域靶面，无一案域外）。subtest 级 drift 10 案逐项归因：
   - argon2 derive_bits ×4：gained 12（3×4）全 Pass——驱动 importKey
     成功后条件注册逐算法案（base 1 聚合 Fail 键 → head 4 Pass 键）；
   - aes_ocb encrypt_decrypt ×4：lost 276 / gained 276 同数翻转——共享
     驱动 importKey 失败分支的「importKey step…」命名（base 276 全
     Fail）翻转为成功分支规范名（head 276 全 Pass），同向量集 276↔276，
     无覆盖损失；
   - wrapKey_unwrapKey ×2：gained 202（101×2）全 Pass——ChaCha 封装
     面条件注册（wrap 对象含 AES-CBC/CTR/GCM/KW/HMAC/ECDSA/ECDH/
     X25519/RSA 系 × ChaCha wrapper）。
   - 数字闭合：P +7,534 = 6,216（共享名 F→P）+ 1,318（新注册全 P）；
     F −7,320 = 6,216 + 1,104（aes_ocb 失败路径旧名退役，等量翻转）；
     keys +214 = 1,318 − 1,104；T 4 不变（Hybrid-KEM，域门控）。
4. **runner 单测**：`cargo test -p zero-wpt-runner` **276P / 0F**（基线
   一致；rebase 后复跑同账）。单测裸调（4/8 旧阈值）同撞 4G 结构边际
   被杀一次——与 Makefile workspace 腿校准同因同证，按校准后 6/12
   阈值过；`cargo clippy -p zero-engine --all-targets -- -D warnings`
   干净（rebase 后复跑过）；`cargo fmt --all -- --check` 干净。
5. **make test**：见[门禁补记](#门禁补记)。

<a name="门禁补记"></a>

## 门禁补记（make test 四跑 + workspace 腿阈值校准 A/B 归因）

- r1：workspace 腿 test-guard 单进程内存超限杀（4,202,068 KB >
  4,194,304 KB）——首判一次性波动（css-variables/html-semantics 先例），
  同轮并行跑了个 clippy（调度失误，非成因但并跑不规范）。
- r2：无超限，image_batch_e2e 1F——`zero-browser not found next to the
  test executable`（既有前置实证形态：r1 被杀 build 留下 debug 零浏览器
  二进制缺失），非代码失败；预建 `cargo build -p zero-browser` 补齐。
- r3：同点再超限且涨幅增大（4,313,012 KB，4.11G）——两跑同二进制
  同点破线，升级为结构怀疑，做 **A/B 归因**（禁止凭先例放行）：
  - 方法：`git stash` 后主树重建 HEAD 代码（无本切片改动）的
    zero-wpt-runner lib 测试二进制（fixture 走编译期路径，worktree 缺
    gitignored 字体/资源不可用——首次 worktree A/B 6 失败即为该污染，
    弃用），`/usr/bin/time -v` 同守卫环境实测峰值。
  - **HEAD 代码 276P/0F、峰值 4,585,564 KB（4.37G）已超 4G 门限**；
    本切片二进制同口径 276P/0F、峰值 4,103,236 KB（3.91G，make test
    态同二进制 4.2–4.3G 波动）。**边际性为既有结构问题**（runner 测试
    并发引擎实例聚合工作集随 shim 全流增长累积跨过 4G；css-variables
    4.22–4.32G、html-semantics 4.31G 超限先例同因），本切片未推高。
  - 处置：Makefile workspace 腿 per-proc-mem 4→6 / total-mem 8→12
    （仅此腿，renderer/browser 各腿维持），注释附实测依据（R5389）；
    真修（runner 测试并发工作集瘦身）归 runner 通道碰头账
    （master.md 碰撞管理记录）。对照 R5000 片 c 栈 32MiB 加固先例
    （同一腿既有基建校准先例）。
- r4（校准后，终跑）：workspace 腿过阈值不再杀（本切片触面全绿）+
  renderer 腿过 + quickjs clippy 腿过；**browser bin 腿 737P/1F**——
  `default_actions_work_without_javascript` 失败（make 整体 Error 101，
  MAKE-EXIT=2）。该 1F 为**兄弟流预存红，非本切片回归**——归因见下节。
  7 条 rustc warning 为 script-sandbox 既有死代码（feature unification
  触发，非本切片触面，clippy -D warnings 腿全过）。

### 预存红归因（browser bin 腿 default_actions_work_without_javascript）

失败签名（r4 日志与单跑一致）：
`assert_url("https://zero.test/submitted?name=after&...")` got
`"error://https://zero.test/submitted?name=&subscribe=yes&plan=pro&go=1"`
（tests.rs:2945）。归因证据四件：

1. **非本切片**：`git stash` 后 HEAD 代码（无本切片改动）重建二进制
   单跑同签名失败（2026-11-02）；rebase 到 origin/main f8925c1e7 后
   同失败。
2. **窗口 ≥ 064dbbeca**（今日早时点，origin/main head 前 35 笔）：
   worktree /tmp/zw-bracket 于 064dbbeca 自洽构建（含 zero-renderer
   二进制）单跑**同签名同点失败**（tests.rs:2945）——aac5e481f（switch
   反射）/ 5248566a3（css_vars）等近期 engine 提交洗清嫌疑。
3. **环境无关（本机）**：带代理（shell HTTP(S)_PROXY 全开）与
   ZERO_NOPROXY=1 清代理两态同红——非代理假红先例同款
   （memory: webview-lib-proxy-env-false-red）。
4. **签名分解**（兄弟流接手线索）：红 = 两独立成分叠加。(i) **键入值
   丢失**——query 中 `subscribe=yes&plan=pro&go=1` 全在（click/勾选/
   序列化管线完好），唯 `name=` 空（type_text 键入值未落到 input
   value）；(ii) **error:// 前缀**——快照 URL 为
   `error://<请求URL>`（构造点 apps/renderer/src/runtime.rs:3324
   show_error_page），而 `assert_url` 要求提交快照 URL 精确相等
   （TabManager::page_url 只读快照，无乐观置位）——zero.test 不可解析
   的环境下导航永不提交真页，(ii) 单独即可判红。同签名 webview 层
   集成测试（tests/integration/src/html_compat.rs:363，SetFormValue +
   intent 断言）全绿——表单序列化底座完好，红在 browser 多进程
   typing/导航集成面。error:// 构造仅 f1382985f（2026-08-19，早于本
   窗口）一笔触过——供兄弟流排查参考。

处置：本流不越界代执行（run-rules #9 先例），记 master.md 碰撞管理；
本切片 make test 结论修正为「workspace/renderer/clippy 腿全绿 +
browser 腿预存红 1F（归因在案）」。

## 账面更新

- **全账**：81,851 → **82,065 keys**；P 52,861 → **60,395**（+7,534）；
  F 28,986 → **21,666**（−7,320）；T 4 不变。
- **新族 F**（D1 域门控桶）：三域翻绿后余红收敛至 PQC 两域 +
  XOF 五连 + X448/KMAC + Hybrid-KEM 待切片面（逐域分布以下轮切片账
  更新，本轮不回改 m2s4 扫查档）。
- 旧 `.tentative.` 路径退役桶内三域案 3,108 keys F→P——退役条目终态
  以 M 末收口对终态锚重算为准（[DC-5② 预检](2026-11-02-dc5-continuity-precheck.md)
  既定口径），不回改预检数。
- 对照基准更新为 [d1s1 锚](2026-11-02-d1s1-argon2-ocb-chacha-full.json.gz)。
- 下一批域切片：XOF 五连（sha3/shake/cshake/k12/turboshake）→ PQC 两域
  → X448/KMAC → M 末收口重定基。
