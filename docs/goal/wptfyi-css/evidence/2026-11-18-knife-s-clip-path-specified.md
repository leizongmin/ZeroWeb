# 刀 S：clip-path specified 声明值 canonical 串化（css-parser 域）

日期：2026-11-18 · 提交 `2692ac1f0`（rebase 前号 `124289605`）· 域：css-parser（fold_decl_specified clip-path 臂改文本层优先 + canonicalizer + 4 钉测，+503 行）

## 定谳

css/css-masking/parsing/clip-path-valid.html 14F/54 根因 = clip-path 声明值 canonical
依赖既有 AST 往返串化（`parse_clip_path` → `clip_path_value_to_css`），而 14 形在该通道
恒等回落或剥除语义错向（geometry-box 被剥除但 oracle 页源期望亦为剥除——页源
`test_valid_value("clip-path","circle(7% at 8% 9%) border-box","circle(7% at 8% 9%)")`
两序均全剥，非保形；其余 13 形 AST 串化恒等原文 ≠ oracle canonical）。

修复 = 文本级 `canonicalize_clip_path_specified`（css-masking-1 §the-clip-path +
css-shapes-2 §serializing-basic-shape），规则逐行钉自页源：

- `<geometry-box>` 前缀/后缀全剥除（`border-box circle(...)`、`circle(...) border-box`
  → `circle(...)`；裸 box 关键字 identity passthrough）；
- 裸 `0` 长度补 px：inset 参数/round 半径（含 slash 组）、xywh/rect 参数与 round 半径、
  polygon 坐标；`auto`/`%`/calc 形保形；
- 缺省半径省略：circle 单 `closest-side` → `circle()`；ellipse 半径全 `closest-side`
  → 省略；混合半径（`1px closest-side`）保形；
- polygon：`nonzero`（缺省 fill-rule）省略、`evenodd` 保形；全零 round 组省略、
  非全零组保形（组内 0 补 px）；段仅 fill-rule 形（`nonzero, x y, ...`）段处置；
- path()：`nonzero` 省略（`evenodd` 保形）+ 路径数据 token 规范——逗号→空白、
  命令字母单空格连接、小写 `z`→`Z`（其余字母大小写保形）、数值 token 原文保形；
  数据串内含逗号故不可逗号切分，以引号前缀判定单/双段；
- at 位置段重排复用刀 Q `canonicalize_at_position`（`circle(4% at top right)` →
  `circle(4% at right top)`）；
- identity 形与 url()/ray()/shape()/不可识别形 None passthrough——回落既有 AST 串化
  通道（物理单位 guard 保留）零漂移。

## 探针账（testharness-css-masking / css-shapes 单页，test-guard 包裹）

探针 bin `bin-knifes-p3` md5 `37c812e0`（提交态重建）：

| 页 | 刀 S 前 | 刀 S 后 |
|----|---------|---------|
| clip-path-valid | 14F/54 | **0F**/54 |
| clip-path-shape-parsing | 22F/44 | 22F（全 shape() 命令面 + invalid 门，未触及） |
| clip-path-invalid | 48F/48 | 48F（validate_decl_value 白名单门，阻塞面不变） |
| clip-path-computed | 17F/21 | 17F（engine computed 面，阻塞面不变） |
| shape-outside-circle-002（刀 Q 回归） | 0F/136 | **0F** 零漂移 |
| shape-outside-circle-004（刀 Q 回归） | 63F/780 | **63F** 零漂移 |
| shape-outside-inset-003（刀 Q 回归） | 240F/520 | **240F** 零漂移 |
| shape-function-valid（P2F 对照） | 3F/51 | 3F 不变零 P2F |
| shape-function-invalid（P2F 对照） | 40F/40 | 40F 不变 |

**净清 14F**（clip-path-valid 满贯），九页探针零回归零 P2F。

## 门禁

zero-css-parser **3306 passed / 0 failed**（+4 钉测：geometry-box 剥除、0 补 px +
缺省半径、位置重排、polygon/path + passthrough）；clippy 零告警；fmt clean。
推放入 rebase 窗：远端来件 `281f787d3`（rendering-compat 纯 docs）与本刀码面零
重叠，`rebase --autostash` 干净回落（共享树他人 WIP 未受扰动），提交号
124289605 → `2692ac1f0`。探针工件 `.acceptance/wptfyi-css/{bin,results}-knife*s*`/
（诊断产物不入库）。

## 余面挂账（阻塞面，待树清）

- clip-path-invalid 48F + shape-function-invalid 40F = 同一 `validate_decl_value`
  VALIDATED_PROPERTIES 白名单门（style-system/apply.rs——对端 fv1 在飞文件）；
  apply 链 parse 臂已在（parse_clip_path/parse_shape_function 均拒畸形形），白名单
  接入即启用（shape-outside 先例同款）；clip-path 需补 `none` 例外（`clip-path: auto`
  /`ray()` 拒、`none`/`url()`/裸 box 放行）。
- clip-path-shape-parsing 22F：shape() 命令 canonical（arc `small rotate 0deg` 省
  略、关键字序）+ invalid 门形，css-parser 后续刀。
- clip-path-computed 17F：engine computed 面（calc 负钳 0px、polygon round token、
  xywh→inset 折算）。
