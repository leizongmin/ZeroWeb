# 刀 CALC-SIZE-RESOLVED-S1：gCS width/height used-value echo 45 靶（2026-11-21 13:3x 收账）

## 池

RESOLVED 池 128F/6 页（css-values 36 页 corpus b0 census，设计
[ knife-calcsize-resolved-s1-design.md](../../../.acceptance/wptfyi-css/knife-calcsize-resolved-s1-design.md)）：
六页测试源 `getComputedStyle(target)` 读形统一，失败全数 gCS width/height
读通道——静态级联串化（flex 项恒 "125px"、% 原文 "31%"、intrinsic
basis "auto"、box-sizing 页 auto 断言期望 used px）。按 CSSOM-1
§resolved value（width/height 非可解析 percent/需依布局属性 resolved
value = used value），engine 级缺口，calc-size 只是触发面之一。
MINMAX 侦察笔（b05db12d0）让位定谳的 engine 车道：CHMETRIC-2
（a5380b4bb）收账后本刀接位。

## face（patch `e3857162041dd6065f2bcf7b21ce521e`，engine 双文件 +216/−1）

- **callbacks.rs 两臂接线**：gCS selector 通道 cache-hit 臂与未命中臂，
  动画三咨询（M2-S2a/WAAPI/Transition/CSSA）与 CSSA override 之后、
  静态级联串化之前插入 echo 咨询（spec 序：动画覆盖整体替代先于
  used 回填）；cache-hit 臂先算后返（entry 被 map.insert move 前消费）。
- **computed_style_cache.rs**：`GEN_RECT_CACHE` 代际 rect 表（键 =
  html/drain_gen/style_version/视口 + 级联源分量，同 GENERATION_CACHE
  键理由 + rect 随视口走；换代经 clear_generation_cache 清空）+
  `gen_rect_for_selector`（miss 期 LayoutEngine + HitTestCache 布局
  回填，gBCR selector 桥同语义）+ `used_size_for_selector` 门集 +
  `used_size_from_border_box` 扣减。
- **门集**（任一不满足回落静态串化 None 零漂移）：prop ∈ {width,
  height}（min/max 不扩——S3）；display 非 Inline/None；元素在 rect
  快照有盒；box-sizing: content-box 时四向 padding/border-width 全 Px
  且 border-style 非 none/hidden（% 混合面不扣即错值，回落静态，
  FIXME % padding 经布局 CB 解析后扩）；border-box → rect.w/h 直接；
  border-style none/hidden → border 0 计（csswg #2768 used 面对齐
  Chromium）；负值钳 0；整数 px 串 round。
- **刻意不做（本切片）**：handle 通道不接 echo（s284 clone-img 期望
  面雷区，留 S3 带独立 battery）；transition_registry 登记面不接
  （登记先于 echo 位，shipped 刀行为零漂移）。

## 电池（knife-calcsize-resolved-s1-battery.sh，log knife-calcsize-resolved-s1-battery.log）

基线 1ec03355a（rebase 后；拉入 CHMETRIC-2 style-system + testharness
+118——双臂同树吸收上游漂移，rebase 前 smoke（基 4a0051382）css-values
41 F2P/0 P2F 一致性旁证）。b0r＝face 剥除、b1r＝树面＋face，M2 异
md5 ✓，FACE-MARKER used_size_for_selector 0→1 ＋ GEN_RECT_CACHE 0→1
（符号入二进制实证）。13:19:43 build 起、13:37:56 收账（18 分钟，
全程 test-guard 包裹：build per-proc 6/total 14/1800s＋leg per-proc
4/total 8/3000s＋CASE_WALL_S=60）。corpus＝css-values 主靶＋css-sizing
＋cssom 重域开放面（新拉 319/225 页）＋css-variables 哨兵。

判读（knife-calcsize-resolved-judge.py）：

- css-values **P 567→608（+41）**，全部 calc-size 族恰集：
  flex-basis-on-column 15＋flex-basis-on-row 12＋height 7＋
  width-box-sizing 4＋width 3（census 池 128F 中 S1 门集覆盖面；
  intrinsic/% basis 深值面残 S2）。
- css-sizing **P 2902→2905（+3）**：inheritance-002 ×2（width/height
  initial 0px）＋min-width-max-width-precedence ×1（min 胜 max）。
- cssom **P 880→881（+1）**：computed-style-001 relative_property_values。
- css-variables 哨兵 **SET-IDENTICAL**（56 页，gCS width/height 非热域
  行为恒）。
- **GRAND F2P 45 / P2F 0**；页级 P 计数门零降；靶族 P2F 门零触；
  **CALCSIZE-S1-GATE PASS rc=0**。TRAP-NOOP state=done rc=0（b1 face
  态即提交态）。

## 验证面注记

fmt 净（face 域零 diff，engine 既有无关漂移未触碰）；clippy face 域
零告警；engine lib unit **4246P/1F**——clip_interp
test_offset_path_m2_canonical_lerp `None ≠ Some("path('M 0 0 H 1 H
2')")`，face 剥树复跑同红＝基线 1ec03355a 既有红非本刀（MOTION-2
lane 挂账，引入链 443144ab0→47378b843）。

## 残面

- **S2**：calc-size 非 definite 值进 style-system 槽＋layout 期
  basis→px 解析（size token 代换、intrinsic basis、% basis）——
  flex-basis/height 页残 F 的深值面。
- **S3**：min/max-width/height echo 面＋handle 通道 echo（独立
  battery）＋flex-basis 页 S1 后残余。
- % padding/border 经布局 CB 解析后扩（门集 FIXME）。

[控制面](../master.md)
