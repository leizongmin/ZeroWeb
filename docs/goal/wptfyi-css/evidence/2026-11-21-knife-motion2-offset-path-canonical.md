# 刀 MOTION-2 · offset-path path() 插值 canonical 化（2026-11-21）

## 目标与机制

css-motion-1 §offset-path 插值：`path()` 对此前仅同段数同命令字母逐实参
lerp（HEAD clip_interp:637 基线臂），相对命令（`m/h/l`）端点、段数失配、
跨变体（path↔ray）形态整臂 None → 调用方 t≥0.5 离散原文，语料
interpolation-001/003/005 path 形断言大量 F。本刀机制：

1. **绝对化 canonical**：相对段（m/h/l 等）端点 absolute 化后参与 lerp——
   `path('m 50 60 h 20')` at(0) canonical 为 `M 50 60 H 70`（css-motion-1
   path interpolation 语料期望 canonical 形）。
2. **同构 canonical lerp**：canonical 后同段数同命令字母逐实参线性；
   单引号发射（`path('M 45 70 H 95')`）。
3. **结构失配臂内离散**：canonical 后段数仍失配 → 返回 None 之外的新
   语义——0.5 前取 from canonical、0.5 起取 to canonical（css-values-4
   §8.5.6 离散 + 语料 005 形钉）。
4. **path↔ray 异构**：调用方离散原文（双端已 canonical，005 形）。

face `MOTION2 offset-path-path-absolute-canonical-discrete-single-quote`
（`#[used] static` 代码字面，clip_interp.rs:845）；实现 clip_interp.rs
+256 行（path_seg_arity/canonicalize_path_segs/lerp_path_segs/fmt_path_
segs 四件）；钉测 animation.rs `test_interpolate_offset_path_m2_path_
canonical`（同构 M/H lerp at(0.125)、相对端点 canonical at(0)、结构失配
at(0.3)/at(0.5)、path↔ray at(0.75)）+ 既有 ray_and_hetero/shape_coord_box
两钉回归保持。调用点接线已在 HEAD（animation.rs:1430 transitions /
:1980 animations）。

## 基线（motion1-r2 b0，2026-11-20 树）

offset-path 页失败池：interpolation-001 134/288、002 6/144、003 8/192、
004 12/192、005 194/502、composition 0/84、coord-box 120/144、006
404/510、007 141/168、008 410/614。本刀主靶 001/003/005 path 形。

## 电池

knife-motion2-battery.sh（05:26:29 发射，unit zw-motion2-battery-052629，
systemd 四件套 + test-guard 全包裹，flock /tmp/knife-motion2.lock，
05:35:20 MOTION2-DONE rc=0，全程 ~9 分钟）：

- 双腿同树构建：b1=face 载（worktree 现态构建，FACE-MARKER b1=1——
  `#[used]` 代码字面 release 存活）/ b0=face 剥（clip_interp.rs+
  animation.rs 两 face 文件 OID 显式 stash（30de4389）后构建，
  FACE-MARKER b0=0，es_module ride 双臂同载相消）。
- 判别：M2 异 md5（b1≠b0 双 bin md5 门过）；FM 门 b1=1/b0=0。
- 3 腿 × 双臂：motion（载荷）/ transforms / typed-om（哨兵），rc 全录。

## RECONCILE（05:35 收账轮；独立 recount 与脚本 TOTAL 一致）

- 页集恒等（无 ASYM，断言数双侧全等）；哨兵 transforms/typed-om 零
  位移＝刀面干净。
- css-motion 四页恰集 F2P：
  - offset-path-interpolation-001：134→258（+124）
  - offset-path-interpolation-002：6→122（+116）
  - offset-path-interpolation-003：8→184（+176）
  - offset-path-interpolation-004：12→180（+168）
- **总净 +584，零回归**。
- 钉面：engine 3P（m2_path_canonical + ray_and_hetero + shape_coord_box
  回归保持），test-guard 定向腿 3 passed 0 failed。

## 残面

- interpolation-001 残 ~30F：段数失配对（`M 0 0 H 1 H 2`×`M 0 0 H 3`）
  其余 t 采样与/animations 变体形——臂内离散语义已立（钉 at(0.3)/
  at(0.5) 过），残留为采样点/变体级差异，挂下一刀考古。
- interpolation-005 整池 ~308F：多子路径（多 M + Z）path canonical 深
  水区——本刀 canonicalize 未覆盖子路径重组语义。
- offset-path-composition 0/84：ray() underlying add-composition 异机制
  （组合算术非插值），MOTION-3 候选。
- coord-box-interpolation 残 24F / 006 106F / 007 27F / 008 204F：
  ray()/basic-shape 变体插值另刀。

