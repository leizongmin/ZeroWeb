# WIDTHSHRINK-PB-DOUBLE-COUNT 刀证据（2026-11-21）

## 机制

- **face**：`crates/layout-engine/src/intrinsic_sizing.rs` 单文件。
- **缺陷**：`box_content_max_width_inner` 收尾 `inner + frame` 无条件加算——
  R5368 定宽替换臂（`own_definite`）在 `box-sizing: border-box` 语境取的定宽
  已是完整 border-box 宽，再外加水石 frame（padding+border）＝双计。
- **实证页**：`css/css-values/calc-size/calc-size-width-box-sizing.html` 臂 15
  `{"width":"calc-size(11px, size * 2)","box-sizing":"border-box"}`——target
  定宽 22px（border-box）＋frame 12（pl5+pr3+bl3+br1）＋margins 3（ml1+mr2）：
  container 期望 25＝22+3，ZW 旧径 37＝22+12（双计）+3。
- **分岔证据**：同页臂 14 content-box 同定宽 22＋frame 12＝34 正确（期望 37
  一致）——缺陷仅在 border-box 分支；两臂同走 own_definite 臂（S2b ride 下
  calc-size 均解形 Px(22)），唯一分岔变量是 box-sizing。
- **同族先例在库**：`clamp_used_width_cross_constraints`（R5517 注）明示
  「box-sizing:border-box 时 min/max 本身即 border-box 值（不加 frame）」；
  本刀把同一口径补到 intrinsic 收尾。

## face 设计（三 hunk）

1. `WIDTHSHRINK_PB_FACE_MARKER`：`#[used]` rodata，ASCII
   `WIDTHSHRINK-PB border-box definite width frame skip`（电池 FM 门探针）。
2. 收尾条件化：`own_definite` Some＋own_style `box_sizing == BorderBox`＋
   `inner <= w`（own_ar 未抬过定宽；inner=max(w, own_ar) 精确比较）→ 直接落
   inner（定宽即 border-box 全宽）；ar 胜出／无定宽／其余分支保持 + frame 旧径。
3. 两钉（`mod tests`，无 calc-size 依赖、peer face 免疫）：
   - `intrinsic_border_box_definite_width_contribution_excludes_frame`：
     22px 定宽＋frame 12 → 贡献 22（旧径 34）；
   - `intrinsic_content_box_definite_width_still_includes_frame`：对照臂
     22 内容基＋12＝34（旧径不变）。

## 预验（16:1x）

- 钉测 filter `definite_width` 11P/0F（含既有 r5368 钉无回退）。
- fmt 零漂移（漂移面＝peer calc_size.rs 9 处＋font_resolution.rs:48，域外）。
- clippy RC=0，intrinsic_sizing.rs 零告警。
- patch reverse-check OK；三 hunk 零 peer 混入（文件级独占）。

## 电池（zw-knife-wsp-battery-1819，GATE-DONE rc=0 18:36:20）

- 发射簿记：首发 unit 1620（16:20）队满 SERIALIZE-WAIT 2h51m（四 sibling
  电池串行），18:20:57 被外部 `systemctl stop`（journal 记录；CPU 1.5s 全在
  门轮询、零树突变、无 state 文件）；9aae62cb 谱系 sibling fork 以同一脚本
  代发 1819（ps 945720），first-writer-wins 簿记见
  `.acceptance/wptfyi-css/NOTES-knife-widthshrink-pb-double-count-claim.md`。
- 判别门：FM b0=0 / b1=1；M2 双臂 md5 异；S1/S3 ride 35 文件零扰动；
  无 S1/S3-MISMATCH；TREE-NOT-COMPILABLE 前置门过。
- RECONCILE：**values gained=1 lost=0**——
  `calc-size-width-box-sizing.html P 5->6 (+1)`；box/sizing/text/flexbox
  四哨兵全 0/0；零 ASYM。
- 子测试级：翻转恰为臂 15（container 断言，b0 msg
  `getComputedStyle(container).width expected "25px"`），其余 14 臂双臂
  同态；height 页不在 RECONCILE（SET-IDENTICAL，BFCIM 已收账 4 臂无回退）。

## 恰集外挂账（本刀不修）

- `clamp_box_max_contribution` 的 `cap + frame` border-box 口径；
- `block_intrinsic_width_ex` CIS 臂 `own_definite + frame`；
- leaf 分支 own_explicit 路径同型双计。
  语料现无翻车实证，出现时逐案开刀。
- 同页余 9 臂失败挡在 target.gCS（calc-size auto/min/max 基解算＝S2 线域），
  S2 落库后 container 断言侧共享本 face 重达。
