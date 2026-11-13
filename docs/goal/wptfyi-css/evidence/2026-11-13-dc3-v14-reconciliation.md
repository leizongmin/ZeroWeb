# DC-3 v14 周跑对账（run 43240558696）+ cssom serialize-values 回归根因修复

2026-11-13。产物 [coverage-2026-11-12](../../../../compat/wptfyi/coverage-2026-11-12.md)
（generated 2026-11-12T17:03:02Z）。本文件按
[DC-3 runbook §4 验收核对清单](2026-11-11-dc3-prep-reconciliation-runbook.md)
逐项入账，并附 serialize-values.html 回归的完整归因链与修复证据。

## 1. 同 revision 核对（§4-1）✓

- 三家 aligned：chrome 157.0.8103.78 / firefox 160.0 / safari 27.2
  (22625.2.8.11.9)，全部 @ revision **a17acfbc8a**（coverage json `meta.runs`）。
- ZW 语料 pin：`meta.corpus_revs` 含 **315976933**（41-sh DIRS pin）与
  **04067ce9**（runner 语料 pin）✓。
- 上游漂移单列：css 分母 196,597 → **196,806**（+209）；全 WPT 文件分母
  126,750 → 126,937。

## 2. 绝对线（§4-3）未达，结构判断维持

**css 60,545 / 196,806（30.76%）**，对 11-11 快照 56,252 净 **+4,293**；
对验收线 182,392 尚余 **121,847**。缺口结构不变：A 类语义面 ~34k 中未收部分
+ B 类布局面 ~85k（runbook §5）；182,392 需 B 类推到 Chrome 同面 60%+，
属长期交汇面，DC-3 按批逼近。

## 3. 分片截断核对（§4-2）

| 套件 | v14 | 阈值/本地 | 判读 |
|---|---|---|---|
| css-values | 3,539 | 阈值 ~2.2k ✓（本地 g23 3,553） | 达阈值；**−14 待归因**（v14 HEAD 含 g14b–g23 增量，3,539 vs 3,553 差口非 g24——g24 3,735 在 dispatch 后） |
| css-color | 10,962 | 阈值 ≥10,808 ✓（本地 10,972） | 达阈值；**−10 待归因** |
| css-grid | 2,058 | 本地 v13 G2a 后 2,058 | **精确一致** ✓（G2a 面已入周跑） |
| css-anchor-position | 11,859 | 本地 11,863 | 达量级；**−4 待归因** |
| css-typed-om | **9,829** | 本地 v5b 9,872 | **✗ 差 −43 = `.any.js` 变体通道整批**——v14 与 11-11 周跑同值，通道从未在周跑 shard 发过，分片配置缺口信号（本地记录轮 9,872 有值），待修 weekly wrapper/ corpus 脚本 typed-om 臂 |

## 4. css 内负差账（§4-4 邻域回退核对）

| 套件 | 11-11 → v14 | 归因 |
|---|---|---|
| cssom | 1,955 → 1,422（**−533**） | **已归因+已修复**，见 §6 |
| css-properties-values-api | 107 → 102（−5） | 待归因（shard artifacts 逐文件，deferred） |
| css-writing-modes | 118 → 114（−4） | 待归因 |
| css-lists | 126 → 124（−2） | 待归因 |
| css-size-adjust | 6 → 5（−1） | 待归因 |

四个小回归合计 −12；coverage json 无逐文件粒度，深归因需
`zw-results-<shard>` artifact（3 天保留）或本地记录轮复测；已排队，不阻塞
本批入账。

非 css 邻域观察（非本流可归因面）：editing 套件 173→0（套件零 = shard 容错
契约「同未跑态」，editing 流账面）；third_party/test262 −118、svg/types −17、
selection/shadow-dom −8、svg/geometry −6（窗口内多流并行落库的引擎/共享面
漂移，本流批面为 css 解析/CSSOM，无 script-sandbox/svg 几何改动）。

css 正增量主体（窗口内本流落库批 g13–g23 的周跑显影）：anchor +1,970、
values +1,340、grid +637、align +310、color +75。

## 5. zw_env_fails（§4-5）

css 全 suite `zw_env_fails` = **0** ✓。

## 6. cssom serialize-values.html −533：归因链与修复

### 6.1 归因链（三层收敛）

1. **页面级**：v14 runner log——整页 SyntaxError `Identifier 'length' has
   already been declared`，533→0（页级崩，非子测试失败）。
2. **引入提交**：html-semantics 流 b6793f4ad（s400 window.length）在 shim
   part01.js 以 `configurable:false` 定义 own accessor——顶层
   `function length(){}`（serialize-values.html line 37）触发 ECMA-262
   §9.1.1.4.15 CanDeclareGlobalFunction step 4（non-configurable → false →
   GDI 抛 SyntaxError）。11-11 快照未受影响系该提交晚于其起跑。
3. **语义定谳**（修复依据，双源）：
   - WHATWG HTML spec Window 接口：`[Replaceable] readonly attribute
     unsigned long length`——**非** [LegacyUnforgeable]；四真 unforgeable 为
     window/document/location/top（spec source line 97865–97889 逐一核对）。
   - Chromium `window.idl` line 63：`[Replaceable, CrossOrigin, Measure]
     readonly attribute unsigned long length`——与 spec 一致。
   - WPT `windowproxy-define-own-property-unforgeable-same-origin.html` 钉测
     面恰为 `["window","document","location","top"]` 四属性，不含 length。
   - chrome-a17-summary（run 5129789854449664）：该页 Chrome **686/697 通过**
     ——[Replaceable] = own accessor configurable:true，顶层函数声明按 V8
     CanDeclareGlobalFunction step 5 成功替换访问器，与 ECMA-262/WebIDL/
     plain-V8 三方语义全和解。
   - 结论：s400 原 L4 pin（configurable:false）是对 spec 的误读（把
     [Replaceable] 当 [LegacyUnforgeable]）。

### 6.2 修复（本批）

- shim `part01.js`：`configurable: false → true` + 注释纠正（[Replaceable]
  语义 + 四真 unforgeable 面 + FIXME setter 未落地）。
- 桥测 `part102_s400_object_schemes_window_length.rs`：L4 pin
  `true/true/false/true` → `true/true/true/true` + 注释同步（跨流面修改，
  对方流 pin 修正依据 = §6.1 双源）。
- **双面验证**：
  - 桥测：`cargo test -p zero-engine --lib s400`（test-guard 包裹）2 passed
    （object_scheme_fetch_faces_s400 + window_length_count_faces_s400，L4
    新 pin 过，L 计数/bare-origin/error/移除递减面全保）。
  - 页面：release runner 定向跑（results-v14-fixprobe/cssom.json）——
    `cases=1 pass=533 fail=164`，对 11-11 基线 533 精确回位；697 总子测试
    = Chrome 分母；164F 为该页既有引擎能力缺口（回与 v14 前一致）。
- 语料扫描：css 全语料仅 serialize-values.html 一页声明顶层
  `function length()`，无同病面。
- **跨流碰头信号**（master.md 同步记）：engine/shim 共享面 +
  对方流桥测 pin 修正——b6793f4ad 的 L4 pin 依据 spec 双源修正为
  configurable:true；对方流其余 pin（计数相位 0,1,3 / bare-origin / error /
  L5 `'length' in window` / L6 移除递减）全部不受影响（桥测实证）。

## 7. v14 窗口五分账（§4-6）

| 类 | 账 |
|---|---|
| 新增执行 | 上游漂移 +209 子测试 + 新接入面（selection 等他流扩面 +19,405 等非本流域） |
| 失败转通过 | 本流 g13–g23 增量周跑显影：anchor +1,970 / values +1,340 / grid +637 / align +310 / color +75 |
| 已有通过回退 | cssom −533（serialize-values 页级崩，本批已修复回位）+ 四小回归 −12（待归因） |
| 超时 | 0（css 域） |
| 未执行 | typed-om `.any.js` 通道 −43（周跑 shard 从未发过，配置缺口）；editing 套件 0（未跑态，他流域） |

## 8. 下一步

1. ~~typed-om `.any.js` 通道~~ → **已修（§9）**，下轮周跑验证 9,872 显影。
2. 四小回归（−5/−4/−2/−1）shard artifact 逐文件归因。
3. 本批修复落库后，cssom 533 于下轮周跑回显（v14 快照仍含 −533，对账以
   本 evidence §6 修复证据为凭）。
4. DC-3 继续按 DC-2 批次路径推进（master.md 残余桶）。

## 9. typed-om `.any.js` 周跑通道修复（同日追加）

**机制定谳**：`goals/lib.sh` `fetch_dir_html` 默认排除 `.any.js`（line
106–120，goal 脚本设 `GOAL_PULL_ANY_JS=1` 才拉，fs/wasm/timing 先例）；
35 号脚本（css-testharness-compat）未设 → CI 全新 checkout 端
css/css-typed-om 语料永远缺 numeric-objects 11 个 `.any.js` 文件。本地 v5b
批为 ad-hoc 拉取（文件在本地语料树、不入 git），故本地 runner 变体通道
（testharness_css_corpus.rs `.any.js` window wrapper，c9f66634c）能跑而
CI 周跑恒缺——9,872 − 9,829 = 43 精确对账，11-11 与 v14 两轮周跑同值
（该通道从未在周跑发过）。

**修复**：Makefile `fetch-wpt-css-testharness-b`（css-face-b 分片 fetch
目标，weekly.yml `matrix.shard.fetch` 直达）设 `GOAL_PULL_ANY_JS=1`。
范围论证：全 css 语料 `.any.js` 仅三处——typed-om 11（真账面 +43）、
geometry 1 + filter-effects 1（均 `idlharness.any.js`，runner 谓词
`css_testharness_case_skipped` 排除 idlharness 专有面，fetch 亦空产出，
不在本批范围）；fetch-b 其余目录（fonts/shapes/anchor/masking/backgrounds）
上游无 `.any.js` 形态，环境变量零副作用。

**验证**：本地探针（run-css-testharness-corpus.sh css-typed-om，含
`.any.js` 名册）对账本地面值；端到端 CI 显影待下轮周跑（核对清单：typed-om
= 9,872）。
