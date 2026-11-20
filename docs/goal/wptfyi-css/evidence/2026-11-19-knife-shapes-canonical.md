# 刀 shapes-canonical 收账（2026-11-19）

## 面定谳（b0 失败目录 + repr 级 Expected 重取）

face = `crates/css-parser/src/values/parse_basic_shape.rs` 单文件 8 hunks
（patch `.acceptance/wptfyi-css/knife-shapes-face.patch` md5 `cd40b24a`，208 行）：

1. `canonicalize_clip_path_fn` path() 数据双引号强制（cssom-1
   §serialize-a-css-path；单引号输入 canonical 化）。
2. `canonicalize_shape_outside_specified` 全形状族扩臂：circle/ellipse →
   `canonicalize_clip_circle_ellipse`（closest-side 缺省省略；旧
   `canonicalize_circle_ellipse_inner` 孤儿删除）、inset 留原函数内扩展
   （参数/半径裸 `0` 补 px；slash 等值折叠保留——shapes oracle 折 `/` 与
   clip 语料保 slash 口径不同，两通道不共用）、polygon/xywh/rect → clip 族
   臂、path → clip path 臂（data 大小写敏感自原文 v 重取，lower 小写副本
   不可用——SVG H/h 指令语义）、shape() → 物理单位 guard + AST 通道
   （`parse_clip_path` + `clip_path_value_to_css`，`from 0 0` → `from 0px 0px`）。
3. `path_to_css` computed gCS 通道 data token 规范（`canonicalize_path_data`，
   失败回落 raw 兜底）。
4. `parse_xywh_function` 负 w/h 文法门（`is_definitely_negative` 共享；
   shape-outside 与 clip-path 双通道拒收）。

## 电池（zw-shapes-b2-232652）

23:26:52 发射，23:36:12 GATE-DONE rc=0。b0 = HEAD + ride（face 剥除）、
b1 = + face；b0 md5 `3d23c765` / b1 `ec826b2f`。RIDE 30 文件 S1/S3 md5 零扰动。
配方承 knife-atpr2（patch -R 剥除/重放 + trap 归一 + test-guard 包裹；
build per-proc 6/total 14/1800s、腿 per-proc 4/total 8/3000s）。

- **主靶 css-shapes：gained=44 lost=0 regressed=0**，pages_gain=17：
  - path 双引号/数据规范 +15（shape-outside-path-valid 9、path-function-valid 2、
    path-function-computed 3、collision 1）
  - shape() AST 通道 +8（shape-valid 1、shape-arguments-000 5、
    shape-notation-000 2）
  - circle/ellipse closest-side +6（circle-function-valid 2、circle-001 2、
    ellipse-001 2）
  - polygon +6（nonzero 1、round-valid 3、polygon-001 2）
  - xywh +3（valid round 0 补 px 1、invalid 负 w/h 双案 2）
  - inset 0 补 px +2（inset-function-valid）
  - shape-outside-computed +4
- **哨兵 css-masking / css-typed-om：gained=0 lost=0**（逐页零差异）。

发射前置探针（16 观察臂，gitignored zzz 页，发射前已删）：全部按设计命中，
含 xywh 负 w/h 双通道由接受转拒收、无 churn 面（`circle(at 90% 0%)`、
`polygon(round 1em,…)`、`rect(10px auto 20px 30px)` echo 保形）。

## 延后面（不入本刀账）

- inset calc 拒收（`parse_clip_inset` FIXME 残面⑤ calc 代数插值刀域；
  inset-function-valid calc 案 + ellipse calc at-position 拒收 1F 同族）
- circle/ellipse calc 除法折叠 2F（`calc(100% / 4)` → `calc(25%)`，calc 代数）
- valid-position sibling-index 1F（异域函数）
- polygon-function-round-computed 3F（engine computed 通道 round 组解析，
  CONTENTED 面）
- shape-outside-invalid-inset-002/003/004 3F（非法 inset 被接受——parse
  接受面，负 offset 文法门候选）
- shape-outside-valid-position 残面、clip-path 插值族 ~778F（fv1 占用，
  见 residue survey）

## 旁注

- 发射窗协调：knife-g（typed-om）电池 23:23:27 GATE-DONE 后发射；knife-co
  （composite-add）在他 clone ZeroWeb-3 独立 target/树，零干扰。
- 首发即死 rc=126（脚本缺执行位），换新 unit 重发，零受害。
