# 语料面恢复 + 漂移复验（2026-11-02）

## 背景

D1 依赖选型仍待拍板（飞书征询线 two 轮在案，本日 `+messages-mget` 核实无回复，
不重复催办），域切片不可开工。本轮开工 `git pull --rebase`（`fa5444f60` 后无
远端漂移 → 随后拉入 `dbd4ef9e8` radio ArrowRight——触共享面
`testharness.rs` + `part05.js` forms 面——与 `59685323f` D4 typing——触
part01/part01f/part06 editing 面），按碰撞管理规则对 main 最新触面做
m2s5 锚全量复验。复验前发现语料面缺角，先恢复后复验。

## 语料面发现：按 clone 漂移（非删除）

- 本 clone（ZeroWeb-3）`tests/wpt-runner/wpt-data/WebCryptoAPI/` 只有旧 pin
  一期语料：全量扫描仅 **245 案**（m2s5 锚为 360 案），`modern-algos/`、
  `secure-curves/`、散新案 5 案全部缺位；目录与旧文件 mtime 停在
  **2026-10-17**（一期取数时点）。
- 根因：wpt-data 为 gitignored 按 clone 本地态，转正子树（M1 的 77 新文件）
  只在 ZeroWeb-2（本流 M1–M2-5 实际执行 clone）取数过，ZeroWeb-3 从未取数
  ——**非删除事故**（目录 mtime 早于 M1 可证无增删）。
- 检出手段：全量账 tally 与锚计数不符（245/66,225 ≠ 360/81,851）；首跑 r2
  完整 JSON 缺 115 案且缺位集 = 两新族 + 散新案全体，与目录缺位精确吻合。

## 恢复（DC-1 机制复用）

`bash tests/wpt-runner/scripts/goals/38-webcrypto-compat-v3.sh`（转正子树
`cb6cc97ada` 全子树新增式，幂等跳过既有文件）重跑：

- 全子树 **275 文件** = M1 记账精确一致（198 旧 + 77 新）；
  modern-algos/ 58、secure-curves/ 14、散新案 5 案全在位（脚本盘点输出）。
- **幂等零变动实证**：旧文件 mtime 保持 2026-10-17（`getPublicKey.tentative
  .https.any.js` 00:14:07、`digest/` 目录 00:09:56）；77 新文件 mtime 本日
  10:25–10:26。
- 跨 clone 逐字节核对：modern-algos digest/aes_ocb、secure-curves x448
  derive、散新案 sharedarraybuffer 4 样本 md5 与 ZeroWeb-2 全同。

## 复验（main 当前态 vs m2s5 锚）

- r1/r2（恢复前，245 案残语料）记账**无效不作对照基准**：r1 零字节 JSON +
  exit 1（V8 fatal flake 按 M2-1/M2-5 同型判据记账，同命令 r2 完整完成）；
  r2 完整 245 案账（缺位为语料非行为）。
- **r3（恢复后全量）**：`./target/test-guard --per-proc-mem 4 --total-mem 8
  --time-limit 3600 -- ./target/release/zero-wpt-runner testharness-webcrypto
  --json` → **360 案 / 81,851 keys = P 52,861 + F 28,986 + T 4**，与 m2s5 锚
  总账逐位一致（exit 1 = 存在域门控 F 的常态退出码）。
- `scripts/webcrypto-anchor-diff.py m2s5锚 r3`：**REGRESSED 0 / FIXED 0 /
  FLIPPED 0 / DRIFT cases 0 / DRIFT subtests 0，ANCHOR DIFF CLEAN**（exit 0）
  ——T 4 同键（Hybrid-KEM generateKey，域门控），无计时翻转（wall 并行有
  本机他项目 systemd 守卫负载，未致漂移）。
- runner 单测 `cargo test -p zero-wpt-runner` **276P / 0F**（radio 流触
  testharness.rs 后基线保持）；本轮零 Rust 触面，clippy 按影响面不跑。

## 结论

- `dbd4ef9e8`（共享 runner 键位 + part05.js forms 面）与 `59685323f`
  （editing 面）对 webcrypto 全量账**零回归实证**（r3 逐位 CLEAN）；
  m2s5 锚在 main 当前态复证成立，对照基准不变。
- DC-1 语料面在本 clone 恢复至 HEAD 布局全量（275 文件），幂等零变动证据
  本轮再证一次（第三种初始态：裸旧 pin → 补 77 新，旧 mtime 零触碰）。
- D1 维持征询等待；余下工作面不变（拍板后按 deps evidence 定序域切片）。
