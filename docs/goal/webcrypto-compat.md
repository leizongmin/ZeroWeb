# Web Crypto 兼容二期 — 扩面 + tentative 转正甄别

> **二期立项（2026-10-27，coverage 对账驱动）**：一期（2026-10-20 模式 A 归档，
> [归档文档](archive/webcrypto-compat.md)）按原范围收口于 corpus 25,788/33,109 =
> 77.89%，非 tentative Fail 清零，7,320F 全部 `.tentative.` 草案域挂账（重开条件
> = 上游转正后按域立项）。本 goal 承接两件事：未拉语料面扩容与转正甄别。
> **一期零回归底线继续有效**（33,109 分母逐 subtest 对照）。

**版本**: v2.1
**日期**: 2026-10-27（二期立项）；2026-10-31（无人值守收口定稿——M2 立项权限落定，
见「待用户决策」与 Support Envelope 排除项）
**状态**: Active——M1/M2 完成、DC-1~4 有据（2026-10-31，控制面
`webcrypto-compat/master.md`，里程碑归档见 `webcrypto-compat/archive/`）；
收口待 `cargo test` 引擎基线（并行流 html-semantics 在清的 fetch/Request
簇 10F，与本 goal 域不相交）清零后 make test 全绿
**执行模式**: WPT 驱动（上游 WebCryptoAPI corpus 为验收标尺）+ 实现与语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——加密面）

## coverage 对账（coverage-2026-10-31 快照现行账，`docs/compat/wptfyi/coverage-2026-10-31.md`；
与 2026-10-27 立项对账数字一致：分母 70,756 / 覆盖 69 文件，未覆盖 57,509）

| WPT 面 | 全量分母（子测试） | 覆盖 | 说明 |
|---|---:|---:|---|
| WebCryptoAPI/derive_bits_keys | 24,918 | 6/38 文件 | 主扩面 |
| WebCryptoAPI（其余族） | 45,838 | 63/296 文件 | 失败 7,320/7,321 全 `.tentative.`（2026-10-27 实测归因，与一期 M6 收口一致），非 tentative 失败仅 1（algorithm-discards-context.https.window.js）|

二期**不是修引擎失败**：失败面已全部挂账。方向是①把未拉语料补全摸底，
②甄别 `.tentative.` 域转正状态。

## Mission

1. **扩面**：derive_bits_keys 等 38 文件中未拉的 32 个补全运行（一期按当时
   corpus 123 案收口，未覆盖 derive_bits_keys 全族），摸清非 tentative 缺口。
2. **转正甄别**：对照一期挂账清单（ML-KEM/ML-DSA/X448/Ed448/AES-OCB/kmac/
   chacha20_poly1305/argon2/cSHAKE 等），逐域核对上游是否已离开 `.tentative.`
   （ML-KEM/ML-DSA 对应 NIST FIPS 203/204 已定稿，转正概率高）；已转正域
   **出立项切片建议**登记 `webcrypto-compat/master.md` 待用户决策表（每域：
   域面 / 分母 / 建议范围 / 依赖需求），未转正域维持挂账。**本 goal 不执行
   收敛**（2026-10-31 用户决策：立项与依赖选型留待后续按域立项——rally 不
   自行立项、不引入新依赖）。
3. **销小账**：algorithm-discards-context 1F 定性（引擎/runner/语料边界）。

## Done Criteria

- **DC-1（语料补全）**：未拉面补全（照一期 26 号脚本的 deep + any.js 通道，
  FORCE=1 续拉）；`.tentative.` 文件照拉（转正甄别需要基线对照）。
- **DC-2（转正甄别清单）**：挂账域逐域转正状态落
  `docs/goal/webcrypto-compat/evidence/`（上游 rev + 目录事实）；已转正域
  出立项切片建议登记 master.md 待用户决策表，未转正域维持挂账。
- **DC-3（扩面基线）**：补全面分类通过率落 evidence；与一期 33,109 分母
  逐 subtest 对照**零回归**。
- **DC-4（coverage 复测）**：WebCryptoAPI 未覆盖文件清零路径成立；
  coverage 失败归因表更新（tentative/非 tentative 分列）。

## 里程碑

- **M1**：语料补全 + 扩面基线 + 转正甄别清单（DC-1/2/3 可一轮完成）。
- **M2**：coverage 复测收口（DC-4）。
- 已转正域收敛不在本 goal（2026-10-31 用户决策移交后续按域立项，切片建议
  见 master.md 待用户决策表）。

## Support Envelope

**范围内**：WebCryptoAPI 未拉语料补全（deep + any.js 通道）、`.tentative.`
文件基线对照与转正甄别、algorithm-discards-context 1F 定性、coverage 复测。
**排除**：已转正域引擎收敛与新增依赖引入（后续按域立项）；一期已收口
33,109 分母面的语义修改（只读对照）；`.tentative.` 域转正前的失败修复。
**资源约束（无人值守）**：批跑限 lane 并发 + case 墙钟预算（照 37 号先例
12 lane / 600s 起步，按面实测调整）；长跑任务走既有 OOM/墙钟守卫
（`docs/engineering/quality-gates.md`），禁止裸跑；CI 发射前查在跑实例
（weekly concurrency cancel-in-progress 会砍旧 run）。

### 依赖约束（run-rules §9）

- 一期收口面（33,109 分母）是回归底线：任何 P→非P 先修再收。零回归对照锚：
  `archive/webcrypto-compat/evidence/2026-10-17-m1-baseline.md`（+ summary.json）。
- 加密实现变化走既有 RustCrypto 依赖面（p256/p384/p521/rsa/ed25519-dalek/
  x25519-dalek 3.0 先例），新算法域（ML-KEM/ML-DSA）选型先出 deps evidence
  ——本 goal 不引入新依赖（见 Mission 2 用户决策）。
- `[SecureContext]` 暴露门禁语义（一期 M6 三件套）不得因扩面回退。

## Final Output Protocol

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方 DONE 允许条件 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **默认输出** |
| 真正外部阻塞（依赖不可用、平台不支持） | `BLOCK: <原因>` | 罕见使用 |

**DONE 允许条件**（同时满足）：DC-1~4 全部满足有据（测试命令、报告路径、
数字证据）；验证基于上游真实 WPT 用例（无内建 inline 充数）；`cargo build`
+ `cargo test` + `cargo clippy` 全过；master.md 内部自洽、里程碑归档完成；
一期 33,109 分母零回归底线保持。

## Document Control

- 每轮进展写 `webcrypto-compat/master.md` 与 `evidence/`；入口文档仅在目标
  实质变化时修改；里程碑完成归档至 `webcrypto-compat/archive/`（只追加）。
- 收口归档：入口 + 控制面整树迁 `docs/goal/archive/`，归档 README 登记行。
- 「待用户决策」表事项不 BLOCK 推进——转零碰撞面自主推进，交付时集中提出。
