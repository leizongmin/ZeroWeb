# 刀 CALC-SIZE-GRAMMAR · calc-size 写面文法门（2026-11-21）

## 缘起与认领

原 MATHFN r1 发射流认领（NOTES-knife-calcsize-claim 00:51 / grammar-claim
05:3x）：css-values calc-size 非动画面 ~234F 中的 parsing 子域——
calc-size-parsing 59F（53 拒形）＋interpolate-size-parsing 3 拒形。
首射 05:57 rc=6（style-system 编译窗撞 RANDOM3b registry.rs 半成品
编辑波 05:56:55–05:58:14）；重排队声明后其 fork 殁于 11-20 夜 429
限额风暴，未再发射。本 session（MOTION-2 v4 收账后）按 NOTES 协议
接手：树态核验（face 完整 marker ×1、face.patch 在档）＋三连静默
实测（unit 无/build 进程无/.rs 编辑窗无）后发射。

## 机制

face = style-system apply.rs 单文件（~+483 行）：

1. `calc_size_decl_gate` 尺寸槽文法门（css-values-5 §8 `calc-size()`，
   tentative）：13 个尺寸槽属性（width/height/min-max 变体/block-size/
   inline-size 族/flex-basis）值含 `calc-size(` 时整值裁决——单一顶层
   `calc-size(<basis>, <sum>)` 形走 basis/sum 文法核；其余形（数学函数
   内嵌 calc-size、多跨度、括号残缺）整拒（§8 calc-size() 不得用于数学
   函数内）。位于白名单检查之前，表外尺寸槽同收口。
2. `interpolate-size` 写面白名单/分派臂（3 拒形翻账）。
3. `#[used]` face marker `CALCSIZE grammar-gate`（代码字面 ASCII，
   apply.rs:223，MATHFN_FACE_MARKER 先例——函数符号 release 内联抹除，
   电池判别只认 rodata 字面）。
4. 此前漏点：calc-size 不在 `value_has_unimplemented_function` 实现函数
   名单 → 放行，math 门 needle 豁免双通道原文透传 store → invalid 面
   store 回显即 Fail（53 形）。替换函数族（var()/env()/attr()）与
   random 族不入本门（computed 期替换重裁/自有文法门）。

## 电池（unit zw-knife-calcsize-battery-0829）

08:26 发射，08:36:01 GATE-DONE rc=0（wall ~10 分钟）。编译门 8.38s
（三 crate check）；b0 = HEAD＋ride 剥 face（strip→build→reapply），
b1 = HEAD＋ride＋face；M2 双 bin md5 异；FM b0=0/b1=1；b1 md5
`90f8bfd15da7e20acd61d642948fa1fa`（与 MOTION-2 v4 b1 同 md5＝确定性
构建同码树，motion2 v4 已在 HEAD、calcsize face 两电池同载）。RIDE 5
文件（types.rs/es_module.rs/SKILL.md/rendering-compat 两 docs，对端
在途），S1/S2/S3 三窗 md5 零扰动；TRAP-NOOP state=done（face 回树=
发射态）。

产物：`.acceptance/wptfyi-css/results-knife-calcsize/`（bin/b0/b1 +
ride-files.txt + s1-md5.txt；日志 knife-calcsize-battery-0829.log）。

## 结果（css-values b0→b1，case 键级全对账）

| 页 | b0 | b1 | Δ |
|---|---|---|---|
| calc-size-parsing | 35/94 | 88/94 | +53 |
| interpolate-size-parsing | 2/5 | 5/5 | +3 |

净 **+56（F2P 56 / P2F 0）**，恰集翻转两页；missing=0 extra=0，
全 9866 断言双臂页级计数对称零幻影；其余 values 页零位移。
b0 P=5085/F=4775 → b1 P=5141/F=4719。

预期兑现核对：认领预期 calc-size-parsing 35→88（53 拒形翻账）＋
interpolate-size-parsing 2→5（3 拒形）——**+56/0 全数兑现，无偏差**。

## 域界与挂账

本刀只收 parsing/写面文法门。挂账池（另刀）：
- **CALC-SIZE-INTERP 候选 ~3305F**（animation 21 页插值面：calc-size-
  height-interpolation 1256F/width-interp 832F/interpolate-size-* 族
  ~800F/box-sizing 30F 等）——engine animation 域，MOTION-2 v4 落地后
  animation.rs/clip_interp.rs 已清空可认领；
- CALC-SIZE-CANON canonical 序列化 6F；
- typed-om calc-size 19F（css-typed-om 域）;
- CALC-SIZE-INTRINSIC DEFINITE 残面 ~209F（`size` 词元 intrinsic 形、
  % 腿 gCS、flex-basis layout Calc basis 消费——需 layout 期内在尺寸
  求值径，previously +41 已收见 master CALC-SIZE-DEFINITE 段）。

## ride 说明

types.rs（css-parser values types +17 行）为独立 lint 债，不入本 face
patch（脚本头注明）；随本刀树在途，另笔提交。
