# R5316 — va:text-* × 负 leading：行盒围合 + 字形基线父 content area 锚定

- **日期**: 2026-10-29
- **驱动案**: `css/CSS2/linebox/vertical-align-negative-leading-001.html` ✗19.98% → **✓0.12%**
- **附带翻绿**: `vertical-align-117a.xht` ✗2.67% → ✓、`vertical-align-118a.xht` ✗5.87% → ✓
- **corpus**: 827F → **825F**（16592 案，FIXED=3；新红 1 经 kill-switch 同位复现定谳环境负载脆弱非本轮）

## 定性

案形：6 容器 `line-height:10px; font-size:30px; font-family:Ahem`（half-leading = −10），
span 声明 `vertical-align: top/bottom/text-top/text-bottom`。chromium 容器行高
`[10,30,10,10,20,20]`——va:top/bottom 不参与行盒围合（对齐最终行盒边）恒 10；
text-top/text-bottom 按父 content area（基线相对：顶 = baseline − A_parent、底 =
baseline + D_parent）围合生长 20/30。

ZW 旧 R822 half_leading 扩展被 `.max(0.0)` 钳零（负 leading 半 leading 为负）→ 六容器
恒 10。两处缺口：

1. **行盒围合**：text-* 盒的基线锚定需求（负 leading 域参照位越出行盒）未参与扩展。
2. **字形基线**：R822 Phase 3 存储臂（inline_finalization）用「扩展后行度量反推
   half_leading」重建字形偏移，负 leading 域同样被 `.max(0.0)` 钳零——行盒生长修复后
   字形仍滞留 strut 基线（dump 带列走查：c2 text-top/text-bottom span 字形均渲 70..100，
   ref 为 60..90 / 80..110，即 chromium Δ = ∓10px）。

## 实现（kill-switch `ZW_VA_TEXT_BASELINE_PIN`，default-on，`=0` 回退旧行为）

1. **R822 扩展**（inline/mod.rs）：父度量 `A_parent/D_parent = 容器 fs × ratio`
   （`ascent_ratio_lookup` 容器作用域，R990 口径 A+D≈fs 近似）。文本 run（font_size>0）
   臂：TextBottom top_need = `h − D_parent − B_pre`、TextTop top_need = `A_parent − B_pre`，
   与 half_leading 校准值取 **max**（正 leading 域需求 ≤ 0 ≤ half，逐字节同旧行为）；
   bot 合围 `va_bot_need = max(text-top 盒底, text-bottom 盒底钉位)` 按扩展后基线计。
   **原子盒（font_size=0）保持 R822 half_leading 臂**——首轮 corpus 捕获 c44-ln-box-000
   +1 红（4em img 於 10px 容器锚定需求 ≈25 会把已校准行高拉到 45，chromium 判例不开），
   文本 run 限定后 ✓0.69% 收口；原子 text-* 行盒围合域另案挂账。
2. **R5316 Phase 3 字形基线锚定**（mod.rs → 存储臂消费）：新字段
   `TextFragment::anchored_baseline_y`（恒 0 = 未设哨兵），
   Δ(top) = `(lh − fs)/2 + A_span − A_parent`、
   Δ(bot) = `−((lh − fs)/2 + D_span − D_parent)`。
   同字体正 leading 域退化为 ±half_leading = R822 Phase 3 重建式（零扰动）；负 leading 域
   给出精确 ∓10。存储臂（inline_finalization）非零优先消费，未锚定 run（sub/super/
   baseline/旗标关）回退旧重建式。

## 交付

- corpus 同树 A/B（基线 R5315 交付态 827F fail-list）：**825F**，FIXED=3（驱动案 ✓0.12%
  + 117a/118a——R5258 切片 2 结构挂账两案同域翻绿，va 族余 122 ✗6.47% /
  baseline-003 ✗2.93% 维持原值）。逐案 diff：`{−117a, −118a, −negative-leading-001,
  +intrinsic-size-009}`。
- 新红 `css-sizing/aspect-ratio/intrinsic-size-009` ✗2.08%：**kill-switch 关闭（字节旧行
  为）同位同值复现 → 非本轮**；同期 ZeroWeb-3 交叉流 `testharness-encoding-legacy-mb`
  活跃窗（157% CPU，bench 三连 FAIL 同源），归负载脆弱族记账。
- c44-ln-box-000 首轮 +1 红 → 文本 run 限定收口 ✓0.69%（原子域挂账见上）。

## 回归测试（engine/tests/r5316_va_text_neg_leading_tests.rs +3）

- `r5316_negative_leading_line_heights_match_chromium`：六容器行高 [10,30,10,10,20,20]
  ±0.5（chromium 判例；修复前恒 10）。
- `r5316_positive_leading_text_bottom_unchanged`：fs16/lh20 单 text-top/text-bottom 行高
  22（正 leading 守卫，旧行为逐字节）。
- `r5316_va_text_glyph_baseline_anchored_to_parent_content_area`：存面字形基线 5 点
  页绝对钉位（c2 text-top +14 / baseline span +24 不动 / text-bottom +34；c5 +14；
  c6 +24——glyph 顶 = 基线 − A 与 ref up10/down10/up5/down5 位移全等）。
- **mutant**：`ZW_VA_TEXT_BASELINE_PIN=0` → heights + glyph 双钉红、正 leading 守卫仍绿
  （双向可判别）。
- R822 既有单测上界 `130+45+1` → `176.73`：合成混字体 IFC 容器 content-bottom 钉位精确
  化（40px run、D_p≈1.15、B=82.12 → 扩展 46.73）；实域 va-117a 同字体校准域 max 组合
  零漂移。

## 门禁

- fmt 0；clippy `zero-layout-engine --all-targets -- -D warnings` 0。
- **layout-engine lib 2212P/0F**。
- 本地 720 套件唯一红 = `float-shrink-inline-children-zw-001` ✗3.68%（R5311 既有记账）。
- product-smoke struct ×N PASS + welcome **15.68%** 零位移 + form-perf PASS。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-layout-engine`：静窗 **GATE PASS 25 指标
  NEW=0**（medium/first_paint 315.42 / 预算 362.35；flex 3.05M / 预算 3.97M）。活跃窗
  360→794ms 三连 FAIL 为交叉流争用伪影（ZeroWeb-3 job 同期，静窗复跑收口；flag on/off
  A/B 359.7/360.3/364.0 分布重叠，Δ≤1%）。

## 挂账

- va 族余 122 / baseline-003（R5258 切片 2 IFC 切片结构域，本轮零扰动维持）。
- **原子 text-* 行盒围合域**：c44 域 chromium 对原子盒不开基线锚定需求（行高 25 非代数
  45）——机理（原子对齐参照/围合参与口径）另案定性。
- `intrinsic-size-009` 入负载脆弱族记账（kill-switch 同值证据）。
- `resolve_relative_inset` 仅 Px 口径分裂（R5315）维持；R5314/R5315 挂账全维持。
