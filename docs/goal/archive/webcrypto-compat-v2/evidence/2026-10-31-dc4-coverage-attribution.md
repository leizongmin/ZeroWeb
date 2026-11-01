# DC-4 coverage 复测与失败归因表（2026-10-31，M2）

## 复测方式与可复现性

- 工具：`tests/wpt-runner/scripts/wptfyi_gap.py coverage`（与官方周报快照同一
  工具、同一 wptfyi-cache：chrome 156.0.8078.80 / firefox 159.0 / safari 27.2
  aligned stable @ ceec05f81f）。
- 输入：M1 基线全量结果（123 案 `--json`，见
  [M1 语料与基线](2026-10-31-m1-corpus-and-baseline.md)）。
- 命令（results 目录内放 runner `--json`）：
  `python3 tests/wpt-runner/scripts/wptfyi_gap.py coverage --results <dir>
  --cache tests/wpt-runner/wptfyi-cache --out <scratch>/coverage.md`。
- **与官方现行快照 coverage-2026-10-31.md（weekly cron 产出）逐位一致**：
  WebCryptoAPI 行 334 文件 / 70,756 子测试，覆盖 69 / 全过 68，通过 13,247 /
  未覆盖 57,509——复测确认分子确定性成立（同工具+同缓存+同结果集 ⇒ 同数字），
  周期 cron 快照无需专门 dispatch 即维持现行账。

## WebCryptoAPI 分子口径总账（tentative / 非 tentative 分列）

coverage 分子（13,247P）≠ ZW 实跑分子（25,788P）。差值 12,541P 与全部缺口
逐桶归因如下（对 M1 基线结果逐案跑 `resolve_to_universe` 实测，非估算）：

| 桶 | 案/路径 | 子测试 | 内容 | 清零路径 |
|---|---|---:|---|---|
| resolved（入分子） | 69 案 | 13,247P（den 13,709） | 稳定案 window 变体 | 已闭合 |
| 未解析·分片变体 | 5 案（pbkdf2、hkdf、successes_RSA-OAEP/PSS/RSASSA-PKCS1-v1_5） | **12,521P 全部隐形** | 上游对大文件按 `?N-M` 分片 variant 跑（如 pbkdf2 `?1-1000`…`?8001-last`，universe 74 路径 / 25,064）；ZW 整文件单案执行，`variant_candidates`（`.any.html`/`.worker.html` 后缀规则）对 `?` 分片路径无映射 → 整案落 zw_only | runner/molecule 基建域（分片 variant 映射或 runner 分片上报），**非本 goal**（通道基建挂账） |
| 未解析·tentative | 49 案 | 7,340（P 20 + F 7,320） | 草案域文件在 universe rev 已转正改名（DC-2），pin 路径无 universe 对应 | 已转正域收敛归 D1 按域立项（转正后 pin 推进 + 收敛） |
| uncovered·转正族 | modern-algos 81 + secure-curves 28 路径 | 18,052 | universe 有、pin 语料无（转正重排后新目录族） | 同上（D1） |
| uncovered·worker 腿 | 75 路径 | 13,812 | universe 记 window+worker 双变体；ZW 通道 window-only（worker 腿 = workers 通道结构性挂账，控制面边界在册） | workers 通道立项 |
| uncovered·新案(pin 无) | 5 路径 | 35 | HEAD 新增 stable 案（export-key-error-order、okp_fixed_length、okp_malformed_asn1、sharedarraybuffer、aes_kw_unwrapKey_failures 的 window 腿） | pin 推进轮（随 D1 或语料滚动） |
| uncovered·记账排除 | 2 路径 | 84 | idlharness.https.any（WebIDLParser build 期资产，runner 记账排除先例）+ secure_context sub.html（include 资产非用例） | 维持排除（跨 goal 一致口径） |

分子内「已覆盖的非 P」= **462**：algorithm-discards Timeout 13（W4 定性，
DOM 域挂账）+ 449 为三家窗口变体子测试数与 ZW wrapper 形态差
（如 failures_AES-CBC 三家 686 vs ZW 618，ZW 自身 618P/0F）——**分子内
ZW 实跑 Fail = 0**。

## 失败归因表（DC-4 要求的 tentative / 非 tentative 分列）

| 归因 | 子测试 | 说明 |
|---|---:|---|
| ZW 实跑 Fail · `.tentative.` 草案域 | 7,320 | 全部 49 个 tentative 案（ML-KEM/ML-DSA/X448/Ed448/AES-OCB/kmac/chacha20_poly1305/argon2/cSHAKE 等）——上游已全部转正（DC-2），修复面归 D1 按域立项 |
| ZW 实跑 Fail · 非 tentative | **0** | 一期 M6 清零状态保持（DC-3 逐 subtest 对照证实） |
| ZW 实跑 Timeout · 非 tentative | 1 | algorithm-discards-context，DOM 域挂账（W4 定性） |
| coverage 分子内失败 | 0 | 13,247P 全部 Pass；462 非 P 为形态差 + DOM 挂账 Timeout，非引擎失败 |

## 未覆盖文件清零路径（成立性判定）

universe 265 个未覆盖文件 = 转正族 109 + 分片变体 74 + worker 腿 75 + 新案 5
+ 记账排除 2，**每一路径都有具名归属与对应清零通道**，不存在「语料已拉而
无账可归」的悬空面：

1. 转正族（18,052）与 7,320 tentative F 同源——D1 按域立项（切片建议已落
   master.md 待用户决策表）即启动清零。
2. 分片变体（25,064，含 ZW 已绿 12,521P）——通道基建域挂账，建议随任一
   D1 立项或 runner 通道轮一并处理（molecule 侧映射为先、runner 分片上报
   为备选）。
3. worker 腿（13,812）——workers 通道结构性挂账（控制面边界在册）。
4. 新案 35 + 记账排除 84——随 pin 滚动/维持排除。

在 D1 立项前，本 goal 的 coverage 口径底线是：**分子确定性**（同输入同数字，
已复测证实）+ **ZW 实跑账零回归**（33,109 逐 subtest，DC-3）。
