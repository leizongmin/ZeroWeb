# 刀 SIZING-GRAMMAR：css-sizing min/max 四属性声明写门＋fit-content 参量链（2026-11-21 15:25 收账）

## 池

恰集 54F/12 页（css-sizing parsing 族，S3 b1r results 全量 census）：
computed 页 4×5F（`calc(10px - 0.5em)` → "0px" 负钳、`fit-content(10px)`/
`fit-content(20%)` 参量丢弃、`fit-content(calc(...))` 两形 CSS.supports
拒）；invalid 页 min-\* 2×11F＋max-\* 2×4F（none/auto 专属键、负值、
双 token、border-box/content-box 尾缀、available、60 unitless 全部落库
成功应拒）；valid 页 4×1F（`"0"` → canonical "0px"）。根因五缺口
A（gCS FitContent 丢参量）/B（fit-content 内参 calc 整值拒）/C（gCS
min/max 非负域无钳）/D（shim 无 sizing 写门）/E（`0` canonical 缺）。
设计 [.acceptance/wptfyi-css/knife-sizing-grammar-design.md]
（patch `d8e32a6cdbbcc4bfa6127cccc4e6ca69`，四文件 +183/−11）。

## face

- **css-parser values/types.rs**（活 `parse_length`；死文件
  parse_basic.rs 同名臂为 R544 死代码陷阱，本刀先误改后回滚——活身
  注释自证）：fit-content 内参 `parse_length.or(parse_math_function→Calc)`
  ＋内参括号平衡门（column-width MATHFN 片段守卫先例）——CSS.supports
  （matcher supports_length_or_math 同源）与落槽双通道一个修复点。
- **engine computed_style.rs**：FitContent 臂保参量串化（cascade
  resolve_length_field R3925 已保 % 内参/纯绝对内参解析——槽侧全备）；
  `sizing_length_to_css` 四臂换装：Px(INFINITY)→"none"（承接
  max_size_to_css 删除）、Px<0→"0px"、Calc 纯静态负结果→"0px"（含 %
  canonical 保留形不钳——钳制需 layout 基准挂账 used 期）、其余
  length_to_css。width/height 非非负域不动。
- **engine callbacks.rs**：`__zw_sizing_normalize_decl` 回调＋
  `sizing_decl_normalize` 纯函数：CSS-wide echo 小写；var()/env()/
  anchor()/anchor-size()/calc-size()/random() 透传（替换期/anchor-func 臂
  /apply_calc_size_definite/替换＋apply 各自裁决）；auto=min-\* 专属、
  none=max-\* 专属；min-content/max-content/裸 fit-content echo；
  thin/medium/thick 借道拒；fit-content(<lp>) 内参独立非负；裸 `-` 前缀
  拒；`"0"`→`"0px"`。非 v8 腿回落 "null"（grid/APC R5334 同口径）。
- **shim part06.js**：`_zwNormalizeDeclValue` sizing 臂（anchor-func 臂
  之后——值域前置校验 fall-through 序保持；!important 先剥；'null'→拒）。
  rule 声明/rule setter/setProperty/cssText 四写路经此单点。

## 电池（knife-sizing-grammar-battery.sh，log knife-sizing-grammar-battery.log）

基线 9da416ba2（S2a 入库后净树）。b0r＝face 剥除、b1r＝树面＋face，
M2 异 md5 ✓，FACE-MARKER sizing_decl_normalize/sizing_length_to_css/
\_\_zw_sizing_normalize_decl 三 token 0→2/1→3 增量 ✓。15:08:29 build
起、15:25:28 收账（16m59s，全程 test-guard 包裹：build per-proc 6/total
14/1800s＋leg per-proc 4/total 8/3000s＋CASE_WALL_S=60）。corpus 四腿
（css-values/css-sizing/cssom/css-variables 哨兵）。

判读（knife-sizing-grammar-judge.py）：

- css-sizing **P 2905→2979（+74）**：12 主靶页恰集 54 F2P 恰合 census
  （computed 4×5＋invalid 11/11/4/4＋valid 4×1）＋开放域 bonus **+20**
  ＝animation/{min,max}-{width,height}-interpolation 四页（插值负侧
  gCS 负值钳连带翻面——S3 census 曾将该域列为插值域排除，本刀负钳
  面顺带收编）。
- css-values **P 616→616（+0）**、cssom **P 888→888（+0）**（b0r 616
  含 S2a +8 基线漂移，本刀零位移）、css-variables 哨兵
  **SET-IDENTICAL**（56 页）。
- **GRAND F2P 74 / P2F 0**；**SIZING-GRAMMAR-GATE PASS rc=0**。TRAP-NOOP
  state=done rc=0（b1 face 态即提交态）。

## 验证面注记

face 域 cargo check 净；clippy 零新增（既有 animation.rs/style-system
computed_style.rs 告警原样）；fmt face 净——types.rs 本刀 hunk
rustfmt-stable，文件既有 9 漂移点未触碰（death-file 回滚后手贴
canonical 形）。单测：css-parser 3342P/0F；style-system 2596P/0F；
engine lib 4246P/1F——clip_interp test_offset_path_m2_canonical_lerp
既有红（MOTION-2 lane 挂账，同 S1/S2a/S3 基线）。

## 残面

- quirks 文档 inline `60`（unitless quirk）被标准文法门拒——quirks
  单位省略面挂账（census 页均 standards）。
- @supports `(max-width: auto)` apply/matcher 层仍放行（写门已拒——
  CSSOM 写通道与 oracle 面分歧，Chromium oracle 为 false，挂账）。
- fit-content(负内参) 槽串化（"fit-content(-10px)" 形）不在 census，
  gCS FitContent 臂不内钳。
- S2b/CALC-SIZE-INTRINSIC（ZeroWeb-2 姊妹流）：calc-size intrinsic
  basis 槽位＋layout 期 basis→px。
- handle 通道 echo（s284 clone-img 雷区，独立 battery）。

[控制面](../master.md)
