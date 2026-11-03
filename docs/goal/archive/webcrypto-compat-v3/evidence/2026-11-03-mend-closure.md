# M 末收口落账（DC-4 / DC-5② / DC-5③ / 归档执行）（2026-11-03）

收口轮唯一目标：把 d1s5 后的全账切到 HEAD 布局终账（旧 `.tentative.` 路径
退役），复测 coverage，出具谱系零移交声明，随后按模式 A 归档。

## 收口判定总表

| 项 | 判定 | 证据 |
|---|---|---|
| DC-1 语料 | ✅ | M1（[2026-11-01-m1-corpus-and-baseline.md](2026-11-01-m1-corpus-and-baseline.md)）；本轮收口前复验：磁盘 0 个 `*.tentative.*`，HEAD 布局语料在位（终账运行 262 案即凭证，本文第一节） |
| DC-2 deps 落账 | ✅ | M1 deps evidence + D1 拍板（2026-11-02）；实际引入与 evidence 一致（12 crate + 钉版 3，见控制面待用户决策表） |
| DC-3 收敛基线 | ✅ | M2 五切片 + D1 五轮域切片逐轮锚对照 REGRESSED=0 |
| DC-4 coverage 复测 | ✅ | 本文第三节（unmapped 108→10，tentative 维度归零） |
| DC-5① 33,109 零回归 | ✅ | 一期 m6-residual 终态锚 stable face 逐位全等（本文第二节） |
| DC-5② 重定基终账 | ✅ | 本文第一节（262 案 / 69,795 keys，对 d1s5 锚切子集逐位 CLEAN） |
| DC-5③ 谱系零移交 | ✅ | 本文第四节（全账非 P 仅 Ed448 降 scope 具名 14 案） |

## 一、收口重定基终账（DC-5②）

### 退役账（旧 `.tentative.` 路径出账）

- 磁盘退役 50 文件（对账定谳：上游 pin→HEAD diff 删除侧 65 = 50 个
  `.tentative.` 改名文件 + 15 个 fixtures 本地从未有；与二期 DC-2 甄别
  「`.tentative.` 50→0」吻合）；corpus 总数 M1 前后一致证语料无漂移。
- 出账 **98 案 / 14,728 keys = P 12,778 + F 1,950 + T 0**，按目录分族：

| 目录族 | keys | 案 |
|---|---:|---:|
| generateKey | 7,196 | 28 |
| import_export | 4,734 | 22 |
| digest | 838 | 8 |
| encrypt_decrypt | 610 | 4 |
| 根面（supports/getPublicKey 四腿） | 604 | 6 |
| sign_verify | 358 | 6 |
| encap_decap | 258 | 4 |
| derive_bits_keys | 76 | 6 |
| serialization | 54 | 14 |

### 终账（HEAD 布局全量双腿）

- **终账 262 案 / 69,795 keys = P 67,845 + F 1,950 + T 0**。
- 运行协议：systemd 单发 unit `mend-r4-webcrypto`（quiet-window 等待 +
  `test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1800`，corpus
  已完成 50 文件退役、树 `b3d86b77c`）；20:10:03 起跑（quiet window 即时
  清——无任何 zero-wpt-runner 在场；ZeroWeb-2 editing 流跨 clone 并行，
  d1s5 先例实证无扰），语料段 20:10:03→20:13:37（3m34s，guard-exit=1
  在案失败集常态，树峰值 926MB）。
- 对 d1s5 锚（360 案 / 84,523 keys = P 80,623 + F 3,900 + T 0）切 HEAD
  布局子集（剔除退役 98 案后恰 262 案）：`scripts/webcrypto-anchor-diff.py`
  对照（[anchor-diff.txt](2026-11-03-mend-rebase-final-anchor-diff.txt)）：
  **REGRESSED（P→非P）= 0、FIXED = 0、FLIPPED = 0、shared 案 subtest 级
  drift = 0（逐位 CLEAN）**，case 级 lost=98（全数退役 tentative 案）/
  gained=0；实跑总账与算术闭合互证（80,623−12,778 = 67,845P、3,900−1,950
  = 1,950F ✓）。
- 终态锚落盘 `evidence/2026-11-03-mend-rebase-final-full.json.gz`
  （md5 `b3ad4905efa4b64f60a9a63834939c3a`）。

## 二、一期 33,109 底线连续性证明（DC-5①，对 m6-residual 终态锚）

对一期终态锚（[archive/webcrypto-compat/evidence/2026-10-17-m6-residual-full.json.gz](../../webcrypto-compat/evidence/2026-10-17-m6-residual-full.json.gz)）逐 subtest 对照
（实跑对照账同 [anchor-diff.txt](2026-11-03-mend-rebase-final-anchor-diff.txt)
第二节：REGRESSED = 0、FLIPPED = 0、FIXED = 0，shared 案键级变化仅下述
三处既记正向）：

- 一期锚内 **74 stable 案 / 25,768 keys 在 HEAD 终账中全部在位且状态逐位
  相同**（同案同果，P→非P = 0，changed = 0）——一期分母零回归底线成立。
- 一期分母算术闭合：33,109 = stable 25,768 + algorithm-discards-context
  1T + 退役 tentative 7,340（49 案 = F 7,320 + P 20）✓。
- 三处既记正向变化（均在一/二期账内已有归属）：
  algorithm-discards-context 1T 销案转 13P（M2-4）；wrapKey 域 +138P
  （上游一期 pin 后新增子测试，全部 Pass）；HEAD 布局较一期锚新增 188 案
  （worker 变体 130 + window 58，全部 M2/D1 收敛面）。

## 三、DC-4 coverage 复测

- 干跑机制验证（[2026-11-02-dc4-coverage-precheck.md](2026-11-02-dc4-coverage-precheck.md)）：
  subset 262 条 → mapped 252 / unmapped 10 / suspicious 0 / multi-suite 0，
  unmapped 10 全数分片变体双腿（预检预测精确吻合）。
- 正式复测（对 r4 锚，`source ~/use-proxy.sh` 前置 + wptfyi cache，
  2026-11-03 20:15）：**unmapped 108 → 10 成立**——WebCryptoAPI 行
  universe 334 文件 / 70,878 子测试，ZW 覆盖 252 / 全过 238 / 通过
  42,803 = 60.39%；可疑条目 0、跨套件重复 0、unmapped 10（分片变体
  双腿，预检名单精确吻合）；universe 未命中 82（sliced 74 +
  idlharness×6 + secure_context 1 + sharedworker 1）维持具名；tentative
  维度随 98 案退役归零（诊断产物落 /tmp 不入库，数字即本节凭证）。
- 对账口径：WebCryptoAPI 面 universe 334 文件 / 70,878 子测试（ZW 修正
  分母 max(universe, zw.total)）；与周报快照
  [coverage-2026-11-02.md](../../../../compat/wptfyi/coverage-2026-11-02.md)
  相比 **unmapped 108 → 10**：98 个 tentative 未映射随退役消失（tentative
  维度归零），余 10 = 分片变体通道基建挂账；universe 未命中 82（sliced 74
  + idlharness×6 + secure_context 1 + sharedworker 1）维持具名。

## 四、DC-5③ 谱系零移交声明

终账 **F 1,950 全数落在 14 案，路径与子测试名均为 Ed448 具名**（D1 拍板
降 scope：MSRV 1.85 无可用签名实现，收口终账承接、不外挂）：

| 案（window/worker 双腿） | keys |
|---|---:|
| `modern-algos/getPublicKey` ×2 | 4 |
| `secure-curves/generateKey/failures_Ed448` ×2 | 168 |
| `secure-curves/generateKey/successes_Ed448` ×2 | 72 |
| `secure-curves/import_export/okp_importKey_Ed448` ×2 | 144 |
| `secure-curves/import_export/okp_importKey_failures_Ed448` ×2 | 1,520 |
| `secure-curves/serialization/ed448` ×2 | 4 |
| `secure-curves/sign_verify/eddsa_curve448` ×2 | 38 |
| **合计 14 案** | **1,950** |

（getPublicKey 两腿 4 子测试名含 Ed448 / NotSupportedError 具名核验；
`eddsa_curve448` 为上游对 Ed448 签名面的命名。）

**谱系终局**：webcrypto 一期（26 号，2026-10-20 归档）→ 二期（26 号 v2，
2026-11-01 归档）→ 三期（38 号，2026-11-03 归档）全部闭环；全账非 P 仅
Ed448 降 scope 具名，无挂账移入任何其他 goal / 待决清单，无后继 goal。

## 五、make test 门禁腿账与兄弟红归因

- 主腿（`make_test.sh`，tree `b3d86b77c`）：workspace 腿
  **3719P/5F**（Error 101，与 d1s5 轮逐位同）——5 红全数 s249 `652a78e00`
  CSSOM 序列化尾分号 Blink 形过期期望（`p56_adopted_snapshot_restore_hook_keeps_page_value`
  / `p99s_rule_style_preserves_custom_property_case` /
  `stylesheet_add_rule_remove_rule_r3276` / `stylesheets_rule_style_r2810`
  / `stylesheets_write_r2809`），渲染流域在册红（[d1s5 evidence 归因](2026-11-03-d1s5-hybrid-kem-slice.md)），
  按 run-rules #9 不代修；goal 域自身全绿。quickjs clippy 并行腿编译完成
  零告警。
- 余腿（`mend-legs` unit，Makefile 参数逐腿复刻，2026-11-03 12:07~12:21）：
  renderer-bin **rc=0**、renderer-lib **rc=0**、browser-bin **rc=101（741P/1F**，
  `tests::default_actions_work_without_javascript`——browser forms/typing
  兄弟流在案红）、browser-tests **rc=101（741P/1F**，同一测试）、GPU 探测
  wgpu adapter 在位后 adapter-only 两腿 **rc=0**、quickjs 五 crate 组合腿
  **rc=0**、quickjs-renderer-bin **rc=0**、quickjs-renderer-lib **rc=101
  （365P/1F**，`mutation_arm_does_not_apply_across_document_reset_window`
  ——quickjs renderer lib 时序在案红）。除两枚兄弟流在案红外全部绿，
  零新增红。
- 门禁红全部归因兄弟流域在册（渲染流 CSSOM ×5 + browser forms/typing
  default_actions ×1 + quickjs renderer lib 时序 mutation_arm ×1，合并树
  碰撞账在册），无一落在本流触面——v2 收口「4 键归因红收口」先例同口径。

## 六、归档执行清单（模式 A）

- [x] 入口文档戳收口状态（v1.2 / 2026-11-03）
- [x] `archive/README.md` 登记行（模式 A：master/evidence/archive + 全要点）
- [x] `tests/wpt-runner/scripts/goals/README.md` 38 行转收口、链接切归档路径
- [x] `docs/compat/trends/wpt-suites.csv` 收行（231 行，15 列对齐 v2 行结构）
- [x] 入口 + 控制面整树 `git mv` 至 `docs/goal/archive/`（含本文与锚）
- [x] 删除 `scripts/rally-goal-38-webcrypto-compat-v3.sh`（2026-11-01 用户
      决策；引用扫描确认仅自引用，38 号语料脚本
      `tests/wpt-runner/scripts/goals/38-webcrypto-compat-v3.sh` 按 v1/v2
      先例保留）
