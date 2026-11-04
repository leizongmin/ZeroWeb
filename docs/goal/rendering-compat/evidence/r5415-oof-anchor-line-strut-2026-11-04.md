# R5415 OOF 锚空行 strut——abspos-block-level-001 ✗1.44%→✓0.39%

日期：2026-11-04　轮次：R5415　域：layout-engine/inline（break_lines.rs Br 臂）
基线：0651b5bd1（R5414 abspos-007 content-extent 锚）

## 1. 目标案

`css/CSS2/positioning/abspos-block-level-001.html`（R5227 域残量最轻：✗1.44%）。
test 页 = 块级 abspos 在行内内容前/后 4 形（2 LTR + 2 RTL）；ref 页 = inline
abspos span + `<br>` 空行 + 双 div 堆叠 ×2。

## 2. 侦察

- REFTEST_DUMP PNG：双页 4 形带完全同位形，仅 ref 全页整体 +2px（自 div1 起）。
- 布局探针（temp probe_r5415，已删）：REF div1/div1' 行高 20px vs TEST 对应
  文本行 18px；body 总高 REF 112 vs TEST 90（修复前）。
- chromium oracle（`google-chrome --headless=new --screenshot`，/tmp/r5415_oracle/
  一次性截图）：双页均 18px 单行、带位完全同形。定谳两条语义：
  - OOF inline 盒不参与线盒高（css-inline-3 §5.2.2；css-position-3 §4.1 OOF
    不参与行内布局）→ 锚行高 = strut；
  - `<br>` 不在 OOF 锚后产生第二行。

## 3. 归因

Br 臂空行 strut（R1286）对「唯一流内容 = OofAnchor + Br」的空行取硬编码
`default_line_height = 20.0` legacy 近似；规范值 = 容器 strut（CSS2 §10.8.1，
R5298 捕获 used line-height = 18）。2px × 2 锚行 → ref 全页 +2px 错位带。

**★形约束（布局探针实证）**：corpus 真文件 div1 内含源空白文本节点
（`\n    `），div1 成 blk=true 真块盒、IFC 行高计入 extent；删空白的手打最小页
div1 走匿名 0×0 盒（blk=false，行高不进 extent，本修复前后渲染不变、无风险）。
常驻测试与一切复现必须复刻真文件换行缩进。

## 4. 门收窄（R5322 判例）

非锚 br 空行维持 20px legacy：R5322 判例（preserve/全量换 strut →
ruby-align-001/001a 0.77%→1.02% 越阈翻红，已回退另案入账）。故只对
「`current_line_had_oof_anchor` 旗置位」的空行换 strut——ruby 页行内内容非
OOF 锚，不触判例。

## 5. 修复（kill-switch `ZW_OOF_ANCHOR_LINE_STRUT=0`，default-on）

- `crates/layout-engine/src/inline/break_lines.rs`：
  - 行游标 `current_line_had_oof_anchor` 旗（OofAnchor 臂置位；7 处 LineBox
    构造点复位）；
  - Br 臂空行：锚旗置位且 `runtime_flags::oof_anchor_line_strut()` →
    `preserved_empty_line_strut`（R5298 容器 strut，内建 Path B 空 styles
    回退 20px legacy），否则 est_height（原路径不动）。
- `crates/layout-engine/src/inline/runtime_flags.rs`：`oof_anchor_line_strut()`
  （LazyLock/default_on，与 R5227/R5298 族同型）。

## 6. 常驻测试

`crates/engine/tests/r5415_oof_anchor_line_strut.rs` 双针：

- ★load-bearing `r5415_oof_anchor_empty_line_strut`：corpus div1 形（含空白
  节点）+ 后随蓝块 → 蓝 y=26（margin 8 + strut 18）；20px legacy 时 28 判红。
  OFF 极性（kill-switch）实测红 → 双向定谳。
- 守卫 `r5415_plain_br_empty_line_keeps_legacy_strut`：无锚 `<div><br></div>`
  → 蓝 y=28（legacy 20 不变，R5322 判例保护）。
- OFF 精确回退的 corpus 侧由 A/B 双窗覆盖（OnceLock 进程缓存不支撑同进程双
  极性）。

## 7. 验证

- 单案：✓ `abspos-block-level-001.html (0.39%)`（原 ✗1.44%）。
- OFF 极性单案：✗ 复现（kill-switch 回退生效）。
- ruby 哨兵：ruby-align-001 ✓0.77%、ruby-align-001a ✓0.77%（门收窄未触判例）。
- R5414 修复 intact：abspos-007 ✓0.16%、between-float-and-text ✓0.00%。
- 邻域系列：见 §8。

## 8. 门禁

- **邻域 series ON/OFF**（BASE=同二进制 kill-switch OFF）：
  - abspos 191 案：ON ✗9 vs 基线 ✗10——唯一差异 = 目标案翻绿；OFF ✗10 与
    R5414 基线（/tmp/r5414_abspos_on2.log）逐字节同。
  - ruby 157 案：ON ✗6 = OFF ✗6（同名集：pseudo-first-letter/ruby-align-002/
    002a/ruby-annotation-with-margin/ruby-intra-level-whitespace-001/
    ruby-text-dynamic-style 全预存；哨兵 001/001a 双态近似通过 0.77% 不在列）。
  - line-breaking 134 案：ON ✗0 = OFF ✗0。
  - static-inside-inline 4 案：ON ✗1 = OFF ✗1（002 3.12% 预存）。
  - 目标案 3×3 ON 稳定 ✓0.39%（合计 5 绿）。
- **corpus A/B**（16594 案）：OFF 713F（vs R5414 w2 OFF 714F：唯一差 = 缺
  abspos-007〔R5414 修复保持 ON 所致〕，零其他漂移）；ON 714F——ON−OFF 修复项
  = 目标案；OFF−ON 2 案表观新红（table-anonymous-objects-154〔在册抖动族〕/
  incremental-scroll 1.63% 临界）单案隔离双侧 3×3 **全绿**＝ON 腿跑窗瞬时负载
  阈值抖动（ON 腿跑时叠加 engine 套件超时与 smoke 编译；guidelines #35），
  **真态 FIXED={abspos-block-level-001} REGRESSED=∅，713F→712F**。
- fmt ✓；guarded-clippy 全工作区 EXIT 0；zero-layout-engine 全腿绿。
- engine 逐二进制 **26/26 exit=0**（含 r5415 双针常驻 + r5411-r5414 全部常驻）。
- make test 腿级（`test:` 单 recipe 首腿非零即止，余腿按 Makefile 原命令逐条
  补跑）：workspace 3724P/5F（js_dom_bridge CSSOM 对端账在册 5 案同名单）＋
  renderer-bin 绿＋renderer-lib 370P/1F＝`page_scripts::bounded_wait_tests::
  lingering_bump_dropped_by_navigation_reset_seq_sweep_s41`（**solo 3× 全绿**
  ＝并发采样抖动新名；archive 2026-10-10 有 bounded_wait s41 兄弟流 flake 前科）
  ＋browser-bin/browser-tests 各 742P/1F＝default_actions（在册第 4 轮未自愈；
  paste_and_go 本轮未现）＋gpu×2 腿绿＋quickjs ws 216 案 3F＝script_fetcher
  s249/s250（在册 R5401 stash 同红；本次 panic 实证
  `css-module/resources/basic.css` 文件缺失＝数据子集缺口）＋
  `delivery_marks_page_units_inlined_not_harness_s261`（**新名 solo 3× 全败
  ＋stash 后 HEAD 同红＝位元无关对端既有**，html-semantics-compat M2 s261 域）
  ＋quickjs renderer-bin 绿＋quickjs renderer-lib 368P/1F＝mutation_arm（在册
  对端账，本轮第 2 次观察）。
- product-smoke 主腿 struct PASS（0 issues）＋legacy 42 fixtures
  struct-check failures: 0，双 exit=0。
- **bench-gate 定向 zero-layout-engine**：ON 腿 **GATE PASS 25（NEW=0）**＋
  同窗 OFF 配对腿（`ZW_R5227_CONTENT_EXTENT=0 ZW_OOF_ANCHOR_LINE_STRUT=0`
  双开关全关＝兼 **R5414 挂账 OFF 配对补跑**）**GATE PASS 25（NEW=0）**——
  两腿同条件（对端流同态）双绿。

## 9. 观察腿（R5415 轮）

- page_scripts::mutation_arm_does_not_apply_across_document_reset_window
  （quickjs renderer-lib）：轮初 3/3 FAILED ＋ make test 腿再败＝**第 2 轮
  观察未自愈** → 对端碰撞账持续，静待碰头。
- default_actions_work_without_javascript（browser bin）：轮初腿 filter 落空
  （running 0 tests——tests.rs 为 bin 内模块需 `--bin zero-browser`），修正腿
  形后仍 FAILED（120s 内部超时）＝make test browser 双腿同红，**第 4 轮未自
  愈** → 对端账维持。
- 新入账：`lingering_bump_dropped_by_navigation_reset_seq_sweep_s41`（并发采
  样抖动族，solo 洗清）＋`delivery_marks_page_units_inlined_not_harness_s261`
  （对端既有，stash-HEAD 同红，html-semantics M2 域）。

