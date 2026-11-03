# R5402：letter-spacing 行尾不施——fit 门对词尾 ls 的行宽原谅（2026-11-03）

## 目标案

WPT `css/css-text/letter-spacing/letter-spacing-200.html`（✗5.79% → ✓0.00% 真通过-可信）。
谱系：css-text-3 §7.2 letter-spacing——"Letter-spacing is applied to each typographic
character unit by inserting half of the additional spacing on each side; **except it is
not applied at the beginning or end of a line**... Applying letter spacing to the right
or trailing edge of a line is incorrect"。行尾裁剪使文本恰好 flush 于块边缘。

https://www.w3.org/TR/css-text-3/#letter-spacing

案面：4 个 float 盒（Ahem 20px/1，宽 5em=100px，`letter-spacing: 1em`），盒 1 无空格
文本「12三456七89」期望三行「12三/456/七89」各恰好 100px；ref = 4 份无 ls control 盒
（white-space: pre）。assert = "Letter spacing is not applied at the start/end of a line"。

## 语义争议定谳（三方实证）

R4231 在案注释（line 629-635）主张 count 语义：`word_width = advance + ls×count`
（含词尾字符），与行尾裁剪方向相反。定谳过程：

1. **css-text-3 现行 TR 文本**（w3.org 抓取原文，上引）明确行尾不施 ls；
2. **chromium oracle 探针**（本地 headless chromium + Ahem 同源加载，/tmp/r5402-oracle
   一次性探针，不留档）："1 2" ls=1em → 120px（行尾 ls **保留** = chromium 走
   count 语义，chromium 自身 fail 该案，test vs ref 截图 diff bbox 实证）；
   "12三456七89" → 360px（9 字符全 count）。
3. **corpus 双侧自洽判据**：reftest PASS = ZW 双侧一致。ref = 无 ls control，ZW PASS
   即 ZW 按 css-text-3 语义渲染并恰好匹配 = 真 spec 语义通过，非双侧同错。

结论：chromium（及 R4231 措辞）= count 语义（css-text-4 方向）；letter-spacing-200
判据 = css-text-3 行尾裁剪。ZW 保留 count 语义累计（R4231 不动），在 **fit 门**处
原谅词尾一次 ls（行尾 R4231 后处理裁剪本就不占行宽，两半合拢行尾语义）。两语义
在不同层各司其职，不冲突。

## 归因

单缺陷：fit 门 `current_x + lead_gap + word_width > avail` 把词内 count 语义累计的
词尾 ls 计入行宽需求 →「456」词宽 120 > 100 整词早断/溢出 → 三行断成错误形态
（test 页与无 ls ref 永不重合）。

## 修复

`crates/layout-engine/src/inline/break_lines.rs` fit 门减去词尾一次
`run.letter_spacing`（`runtime_flags::ls_line_end_fit()` kill-switch 包裹，default-on，
`ZW_LS_LINE_END_FIT=0` 回退旧门，LazyLock + selected 模式与既有 flag 一致）。行首
lead_gap、词中相邻 ls 推进（R1450 针）、hang_trailing_space / atomic_gl_first_word /
wrap_allowed 既有分支不动。

常驻测试：`crates/layout-engine/src/inline/tests/r5402_ls_line_end_fit_tests.rs`
2 针——★load-bearing 三行 100px 形态断言（行 2「456」单片段 w=100，旧实现 120 溢出）
+ ls=0 对照锚。OFF 轮由 corpus 覆盖不设单测（进程内 set_var 不可靠，R3858 教训）。

## 验证（A/B 与门禁全绿）

- **家族 A/B**（BASE=HEAD R5401 落账态 vs FIX=WIP）：letter-spacing-200
  ✗5.79%→✓0.00%（FIXED）；201 ✓2.23%→✓0.50%（顺带改善）；202/203/204/206/
  ligatures-005 逐案不变；c542-letter-sp-000/001（R4231 判例案）双态 ✓0.00%
  恒绿；first-letter-space-not-selected ✓0.00% 恒绿。
- **全量 corpus A/B**（16594 案）：BASE 729F → FIX 725F。comm diff FIXED =
  {letter-spacing-200, table-anonymous-objects-150/152/154}；REGRESSED = ∅。
  table-anon 3 案按 guidelines #35 隔离复跑定谳：FIX 3×3 全绿 0.00%，**BASE 3×3
  同样全绿 0.00%** → BASE 全量单跑红 = 并行抖动（R5387 轮该族抖动史同款），案面
  display 动态翻转无 letter-spacing，与 fit 门无因果链，**不计 FIXED**。真态校正
  BASE 726F → FIX 725F，FIXED = {letter-spacing-200}，REGRESSED = ∅。
- **真态分类**（DC-14）：真通过-可信 10363 → **10364**（+1 = letter-spacing-200
  翻真）；不一致 725。
- **kill-switch 敏感性**：`ZW_LS_LINE_END_FIT=0` → letter-spacing-200 精确回
  ✗5.79%（与 BASE 数值一字不差）；默认 → ✓0.00%。因果单一。
- **单测**：inline 域 419 全绿（含新针 2/2）。
- **make test**：65+ 腿；js_dom_bridge 腿 5 failed（part56/p99s/r2809/r2810/r3276，
  CSSOM 域）为**既有红**——BASE 态同腿单跑 5 failed 逐案同名单（控制面
  js_dom_bridge CSSOM 5 案对端账），与 R5402 无关；其余腿全绿。
- **fmt/clippy**：fmt --check 干净；guarded-clippy 全 workspace 清零。
- **product-smoke**：全部 struct-check PASS（welcome 1200/375/320 + morning-work）。
- **bench-gate**（`ZERO_WEB_BENCH_CRATES=zero-layout-engine`）：GATE PASS 25 指标
  全在预算内（fit 门在断行热路径，flex_layout 3.10ms / incremental_layout 3.17ms
  均无回归迹象）。

## 遗留

- letter-spacing-203（✗9.94%）：element boundaries 域——相邻字符间 ls 由最近共同
  祖先决定 + `word-break: break-all`，span 嵌套 ls0/ls1/ls3 十对组合；与行尾 fit
  门不同域，非本轮轻量可修，挂账。
- letter-spacing-206（✗18.57%）：R5368 挂账延续（钉宽公式域），本轮双态数值不变。
- letter-spacing-ligatures-005（✗0.36%）：既有挂账，不变。

## 关联

R4231（count 语义判例）、R1450（ls 推进针）、R5246（ls 断行 paragraph scope）、
R5165/R5240/R5325/R3785（fit 门既有分支谱系）。
