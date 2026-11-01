# Web Crypto 兼容三期运行时控制面板

**状态**: Active——M1 完成（2026-11-01：语料预置 + 分域基线 + deps evidence +
选型征询落档）；M2 无依赖面两切片完成（M2-1 门禁面 +543P、M2-2 散新案
归因 +12P，均零回归）；余 worker 腿接线与 1T 销案，D1 拍板后进域切片
**最后更新**: 2026-11-01（M2-2 完成）

---

## 当前状态

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

**下一步（M2 余下）**：拍板前推进无新依赖面——① worker 腿接线与
1T 销案穿插（无依赖，触 runner/workers 通道基建——与 25 号流共享
testharness.rs，落账前对账）。拍板后按 deps evidence 建议定序（依赖就绪度：
aes-ocb/chacha20poly1305/argon2 等 AEADs+password-hashes 单 crate 域先行，
XOF 五连引入次之，ML-KEM/ML-DSA 再之，X448 pre 与 KMAC 两案随拍板；
门禁面注册表 REG 与 getPublicKey 域面随域切片逐行扩表翻绿）。

- 输入：二期 [DC-2 甄别清单](../archive/webcrypto-compat-v2/evidence/2026-10-31-dc2-promotion-screening.md)（逐域分母/建议范围/依赖候选）
- 上游复证（2026-11-01）：HEAD `a0dd301b85…`，WebCryptoAPI 子树
  `084495e783…` 与甄别时（`cb6cc97ada…`）**逐字节相同**——tentative=0、
  未转正域 0、零漂移；pin→HEAD diff 80 增/65 删已逐文件枚举（入口文档
  上游事实节）。
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
| D1 | 10 域新依赖选型：deps evidence 已落档（[2026-11-01-m1-deps-evidence.md](evidence/2026-11-01-m1-deps-evidence.md)），建议值合计 12–13 crate（全 RustCrypto 官方系为主）。**征询中**：① 总体选型建议值是否批准；② X448/Ed448 pre 版（0.14.0-pre.12/pre.15）甄别从严交拍板；③ KMAC 两案（sha3-kmac 0.3.0 引入 vs 手写叠官方 cshake）；④ ml-kem 0.3.4 / ml-dsa 0.1.2 版本升替（候选 0.3.3/0.1.1 过时一日）。拍板前收敛切片仅限门禁面/散新案/worker 腿/1T（无新依赖域） | 已飞书征询在案（GB-20261101，msg `om_x100b61408d1ee4a0cffa63121bb8f1e`）；**待拍板**（evidence 已齐） |

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
