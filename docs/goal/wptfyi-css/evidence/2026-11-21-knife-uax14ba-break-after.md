# 刀 UAX14-BA：BA 类后断机会词内子分词＋LB13 页驱动最小禁断（2026-11-21）

## 池与根因

- 恢复靶＝baspglwj 016..118 族 **0/4×74 页**（GLBREAK 收账笔挂账的
  UAX14 BA/B2/WJ/SY 大刀域之 BA 员；74×4=296F 上界）。
- 页形态：50ch monospace 25px 盒内 `39a+BA+30b`，四臂
  white-space（normal/pre-line/pre-wrap/break-spaces）断言
  `offsetHeight > 35`（必须断行）。当前断行器断点仅在可折叠空白
  （collapse_split_words split）、CJK/SEA per-char（R645）、ZWSP
  （R1449）——BA 字符嵌词无机会 → 四臂单行 30px 断言全败。
- 字符域（74 页实集，程序化核成员表零缺员）：连字符/破折号系
  （058A/05BE/007C/2010/2012/2013/2027/2E17）、danda/词分隔符系
  （0964-0965/0F0B/1361/1735-1736/A8CE-A8CF/AA5D-AA5F/10A50-10A57/
  1039F/103D0/1091F/12470/10100-10102/16EB-16ED/2056-205E/2E19/
  2E2A-2E2D/2E30/0F85/0F34/0F7F/0FBE/0FBF/0FD2/1804-1805/1C3B-1C3F/
  1C7E-1C7F/2CFA-2CFC/2CFF/A60D/A60F/A92E-A92F）。

## face

| 文件 | 内容 |
|---|---|
| `inline_types.rs` | `is_uax14_break_after` BA 成员表 73 码点＋`UAX14BA_FACE_MARKER`（`#[used]` ASCII rodata）＋`is_uax14_lb13_ex`（LB13 页驱动最小集 {U+0021}）＋钉测两枚（BA 员真/GL 2011 假/2014 B2 假/字母假；EX 真/后继字母假） |
| `inline/mod.rs` | preserve 臂 BA flush-keep（逐字 tokenizer，BA 附着词尾、零注空格；空白分支**后**的 else-if——U+2002 类 BA∩White_Space 不误触发）＋normal 臂 BA 前缀词元 flush（has_ba gate＝`cjk_contiguous`＝`is_ahem\|\|ZW_CJK_CONTIGUOUS` default-on，Latin 词含 BA 形全开；word-spacing 尾空格仅在 step-1 原空格处）；两 flush 处 peek 后继 LB13 禁断（后继 EX「!」无机会——line-breaking-028 `X⟨BA⟩!` 1ch 盒 `offsetHeight <= 35` 不断行） |
| `runtime_flags.rs` | `uax14_ba_break()` default-on，`ZW_UAX14_BA_BREAK=0` 回退（R5165/R4345 房风） |

- FIXME(R646-slice2)：完整 UAX#14 pair table（LB25 数值上下文 BA×NU、
  LB13 CL/CP/EX/SY 全类＋可折叠空白路径断点过滤、B2/WJ/SY 类域）另刀。

## 预注册预期（发射前）

- baspglwj 74 页 0/4→4/4（+296 上界；缺字形不溢出页 P2F 逐案归因）；
  其余 39 页逐键 SET-IDENTICAL（002–015 2/4 保持、4/4 26 页保持、
  no-break 13 页保持、121/124 4/4 保持）。
- text 语料含 U+2010/2013 hyphen 散文页可能位移——gained/lost 逐案
  归因后才可入账；white-space-collapse-001 8/8 保持。
- fonts/sizing/values 三哨 SET-IDENTICAL 为佳；reftest 725 全量兜面。

## 电池 r1（unit zw-knife-uax14ba-battery-1431，14:26–14:45）

- 门：check 绿；B0 build 83s / B1 build；FM `UAX14 BA break-after
  slice` b0=0/b1=1 ✓；M2 md5 异 ✓；RIDE 20 文件 S1/S2/S3 三窗零扰动
  ✓；EXIT rc=0。
- RECONCILE：**text gained=296 lost=1**——74 页全 `P 0->4`（上界全清）
  ＋`line-breaking-028` P 1→0（−1）；fonts/sizing/values 三哨
  gained=0 lost=0；全语料无 ASYM。
- **−1 归因定谳**：line-breaking-028＝UAX14 LB13 禁则页（`X+BA+!`
  1ch 盒断言不断行，BA×EX 对 LB13 `× EX` 禁断）。b0 P1＝U+1361 无
  机会 pass、U+1680/2009/205F 空白 split 既有缺口 fail；r1 BA flush
  无禁则面 → U+1361 臂 pass→fail 翻面＝本刀引入 P2F。
- 处置：face 内最小修——两 flush 处 peek 后继 LB13 禁断（后继
  EX∈{U+0021} 页驱动集不 flush）；1680/2009/205F 三臂**既有**缺口
  （b0 已 fail 域）挂账 LB13-空白面 slice2，非本刀引入。修复后
  钉测 2/2 绿（新增 `uax14_lb13_ex_forbidden_after_ba`），本面
  fmt 干净（同 crate 他流 calc_size.rs 在飞面 11 处 Diff 非本刀域）。

## 电池 r2–r3（LB13 修复后两次撞他流写码窗，作废）

- r2（r2-1500，14:50）：B0 build 过、B1 build 撞 s2-calcsize-basis
  fork 在飞半成品（calc_size.rs E0061/E0425，B0/B1 窗内被编辑）rc=6
  作废——共享树构建扫入他流在途码形态。
- r3（r3-1507，14:52–14:58）：B0 build 撞第三条流（font/paint 域）
  跨 crate 半成品（text_shaping.rs E0425）rc=6 作废；定谳修正＝
  face_match/font-mod/text_shaping 三文件 14:58:15 同刻同步落盘，
  编译读文件时刻早于落盘的**瞬时竞态红**，15:1x check 绿——非持久
  半成品。

## 电池 r4（unit zw-knife-uax14ba-battery-r4-1512，15:04–15:21，LB13 修复态）

- 门：check 绿；FM `UAX14 BA break-after slice` b0=0/b1=1 ✓；
  M2 md5 异（b0 `9ca318d3…` vs b1 `f61bdae1…`）✓；RIDE 42 文件
  S1/S2/S3 三窗零扰动 ✓；EXIT rc=0。
- RECONCILE：**text gained=296 lost=0**——74 页全 `P 0->4`（r1 同账
  重现）、line-breaking-028 零位移（LB13 修复生效，保持 b0 态 P1）；
  fonts/sizing/values 三哨 gained=0 lost=0；全语料无 ASYM——预注册
  全兑现。

## 收账结论

- 判定：+296/−0 净增，P2F=0，无 ASYM，预注册全兑现——入库。
- GLBREAK 收账笔挂账「UAX14 BA/B2/WJ/SY 类断行大刀域」的 **BA 员域
  至此清偿**；B2/WJ/SY 类域（002–015 空白 BA 页 preserve 臂、em dash
  对偶页等）＋LB13 完整类（CL/CP/EX/SY＋可折叠空白路径断点过滤）
  挂账 slice2 另刀；line-breaking-028 的 U+1680/2009/205F 三臂为
  b0 既有缺口（空白 split＋EX 前禁断面），非本刀引入。
- 执行注记：r1 即 rc=126（脚本无执行位，chmod 后重发）；发射脚本
  （flock 防双发＋test-guard 内存墙钟＋trap face 剥除/恢复＋ride
  三窗）逐轮换唯一 results 目录，r2/r3 撞车半账目录保留作证据。
- reftest 兜面：`make reftest` 全量 725/725（Layout 521/521、Text
  204/204）零位移（face 在树态 15:2x 跑，rc=0）。

