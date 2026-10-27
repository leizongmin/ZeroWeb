# Web Crypto 兼容二期 — 扩面 + tentative 转正甄别

> **二期立项（2026-10-27，coverage 对账驱动）**：一期（2026-10-20 模式 A 归档，
> [归档文档](archive/webcrypto-compat.md)）按原范围收口于 corpus 25,788/33,109 =
> 77.89%，非 tentative Fail 清零，7,320F 全部 `.tentative.` 草案域挂账（重开条件
> = 上游转正后按域立项）。本 goal 承接两件事：未拉语料面扩容与转正甄别。
> **一期零回归底线继续有效**（33,109 分母逐 subtest 对照）。

**版本**: v2.0
**日期**: 2026-10-27（二期立项）
**状态**: Active——M1 未启动
**执行模式**: WPT 驱动（上游 WebCryptoAPI corpus 为验收标尺）+ 实现与语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——加密面）

## coverage 对账（coverage-2026-10-27 基线，47 套件干跑口径，终稿待 `docs/compat/wptfyi/coverage-2026-10-27.md` 复核）

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
   按域立项收敛，未转正域维持挂账。
3. **销小账**：algorithm-discards-context 1F 定性（引擎/runner/语料边界）。

## Done Criteria

- **DC-1（语料补全）**：未拉面补全（照一期 26 号脚本的 deep + any.js 通道，
  FORCE=1 续拉）；`.tentative.` 文件照拉（转正甄别需要基线对照）。
- **DC-2（转正甄别清单）**：挂账域逐域转正状态落
  `docs/goal/webcrypto-compat/evidence/`（上游 rev + 目录事实）；已转正域
  出立项切片建议，未转正域维持挂账。
- **DC-3（扩面基线）**：补全面分类通过率落 evidence；与一期 33,109 分母
  逐 subtest 对照**零回归**。
- **DC-4（coverage 复测）**：WebCryptoAPI 未覆盖文件清零路径成立；
  coverage 失败归因表更新（tentative/非 tentative 分列）。

## 里程碑

- **M1**：语料补全 + 扩面基线 + 转正甄别清单（DC-1/2/3 可一轮完成）。
- **M2**：已转正域按域立项收敛（域多则分轮，每轮零回归）。
- **M3**：coverage 复测收口。

## 依赖约束（run-rules §9）

- 一期收口面（33,109 分母）是回归底线：任何 P→非P 先修再收。
- 加密实现变化走既有 RustCrypto 依赖面（p256/p384/p521/rsa/ed25519-dalek/
  x25519-dalek 3.0 先例），新算法域（ML-KEM/ML-DSA）选型先出 deps evidence。
- `[SecureContext]` 暴露门禁语义（一期 M6 三件套）不得因扩面回退。
