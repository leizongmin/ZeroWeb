# 刀 CALC-SIZE-S2A：calc-size definite px basis＋size 词元代换 8 靶（2026-11-21 14:5x 收账）

## 池

S1（bb41f58b4）后 RESOLVED 池残余 128F/6 页的 definite 面切片：sum 含
`size` 词元且 basis definite px（px 字面/嵌套 definite calc-size）形——
写面文法门已放行但 apply 期拒收落 auto（`calc-size(83px, size*3)` 期望
249 got 10px/500px）。per css-values-5 §8：definite basis 下 `size`
解析为 basis、整值 definite，computed 期可求值。intrinsic/%/any basis
（used-value 期求值）由姊妹刀 S2b/CALC-SIZE-INTRINSIC 槽位承载接位
（ZeroWeb-2 apply.rs 协作叠加在飞，doc 明写与本刀互斥切割）。

## face（patch `knife-calcsize-s2a-face.patch`，style-system 单文件）

- `calc_size_definite_sum`：size-token 分支——definite sum 原样包装
  （原通道不变）；sum 含 `size` 且 basis definite px → `size`→basis px
  叶替换后包装走既有 calc() 数学通道。词元扫描提为
  `calc_size_sum_has_size_token`。
- `calc_size_basis_definite_px`（新）：嵌套 calc-size basis 递归 definite
  sum（终止=嵌套深度有界）＋`eval_calc(expr, None)` 无上下文求值（% 叶
  fail-closed None＝布局期依容器挂账）；平实 basis `LengthValue::Px`
  直取，其他单位/关键词/负形 fail-closed（FIXME 绝对单位换算随
  intrinsic 刀扩展）。
- `replace_size_leaves` 加 `repl` 参（写面文法门传 `1px` 型别代裁不变，
  S2a 传 basis px 语义代换）。
- FACE-MARKER `CALCSIZE s2a size-token-basis-subst`（`#[used]` rodata）。
- 钉测 `calc_size_s2a_size_token_basis_substitution`：83×3=249、嵌套
  any,300px×2=600、11×2=22、sign(size)*size 组合形 10、混合 % sum
  Calc 槽、拒面维持（auto/min-content/31%/any/嵌套 indefinite）。
- 双树实证：ZeroWeb-2（217739b80＋多流 ride）与 ZeroWeb-3（baf6b16d7
  净基）钉测 4/4 首执绿。

## 电池（ZeroWeb-3 代发，results-knife-calcsize-s2a，log knife-calcsize-s2a-battery-1439.log）

ZeroWeb-2 apply.rs 为 S2a+S2b 混合区多流拥塞窗（首发电池 knife-
calcsize-s2a-battery-1423 rc=6＝b0 build 撞 calc_size_slots 半成品
E0063，非本刀缺陷），电池改 ZeroWeb-3 baf6b16d7 干净基线代发（b0=
净树、b1=树+S2a face，无 ride）。FACE-MARKER b0=0/b1=1 ✓、M2 md5 异 ✓
（build 14:40:00-14:41:17 双臂，legs 各 3000s 墙 CASE_WALL_S=60，
test-guard 包裹全程）。

判读（judge-calcsize-s2a.py）：

- css-values **P +8**：height 页 +4（83px×3 双容器臂 249px＋嵌套
  calc-size(any,300px),size*2 双臂 600px）、width 页 +1（83px×3）、
  width-box-sizing +1（11px×2 content-box 臂全绿）、flex-basis-on-column
  +1 ＋ flex-basis-on-row +1（83px×3 经 FlexBasisValue::Length 通道
  row/column 双向生效——预注册 +0 推断被实测推翻的 bonus，flex
  resolved-value 通道对 definite 落槽值已可消费）。
- css-sizing **SET-IDENTICAL**、css-variables 哨兵 **SET-IDENTICAL**。
- **GRAND F2P 8 / P2F 0 / ASYM 0**；CALCSIZE-S2A-GATE PASS rc=0。

## box-sizing 四臂 target 断言全兑现注记

height-box-sizing 双臂＋width-box-sizing border-box 臂 b1 后 target
断言全绿（b0 败点 target got 7px/19px → b1 败点后移 container）：case
级 Pass 被 container 断言挡住——height 双臂 expected 37/25 got 35/23
（恒差 +2＝inline-block 容器丢子 margin-top bug 域）；width border-box
臂 expected 25 got 37（容器宽累计缺口）。container 缺口独立刀域挂账。

## 验证面注记

fmt 净（发射窗内一笔钉测长行漂移，cargo fmt 归位）；clippy face 域零
命中——style-system lib 6 warning＝RANDOM3a missing_docs 既有提交态红
＋提取钉测 assert_eq literal bool（HEAD 同在）域外不代修。首发电池
rc=6 的 b0 撞窗判 dispositions：树态瞬态不可编译非本刀缺陷（ZeroWeb-3
基线 b0 check/build 双绿反证）。

## 残面

- **S2b/CALC-SIZE-INTRINSIC**（ZeroWeb-2 姊妹流在飞）：intrinsic/%/
  any basis 槽位承载＋layout 期 basis→px——box-sizing 页 26F＋height/
  width 页残 intrinsic 断言＋flex-basis 页深值面。
- container 断言域：inline-block 容器高丢子 margin-top（+2 恒差）＋
  容器宽 border-box 累计缺口——box-sizing 页 case 级翻绿的把门刀。
- max-* 槽 calc-size（intrinsic basis）与 min/max 前置回退语义。
- flex-basis 页 62F 深值域（flex 消费通道）。

[控制面](../master.md)
