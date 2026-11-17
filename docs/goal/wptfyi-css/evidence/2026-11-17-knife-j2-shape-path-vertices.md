# 刀 J2：shape()/path() 命令串接几何（border_shape.rs 顶点面第二刀）

日期：2026-11-17
基点：HEAD 3dce9b25f（刀 J 05efa0a62 后 + R5758 更正三 docs）
工法：刀 J 同款对称消去 A′/B（pathspec stash 双文件→release bin→pop；差量=纯本刀；peer WIP 双侧同态抵消）

## 靶点与根因

两 reftest 靶页均为 `border-shape: shape(...)` 顶点面缺失（刀 J 只落 rect/xywh 两函数）：

| 靶页 | vintage 失败率 | 期望 shape 值 |
|---|---|---|
| css-borders/border-shape/border-shape-shape-arc-miter-limit.html | F 0.44% | `shape(from 50% 10%, arc to 90% 80% of 80px ccw, arc to 10% 80% of 80px ccw, arc to 50% 10% of 80px ccw, close) border-box` |
| css-borders/border-shape-absolute-coords-shape.tentative.html | F 4.67% | `shape(from 0% 0%, line to 100% 0%, line to 50% 100%, close)`（无 geometry-box → HalfBorderBox 缺省参考盒） |

实现（css-shapes-2 §shape-function / §path-function，规范文本本体在 css-shapes-1）：
- `flatten_shape_value`：Move/Line/Hline/Vline/Curve/Smooth/Arc/Close 全命令串接状态机（cur/subpath_start/last_ctrl）。
  - 百分比轴相对参考盒宽/高；`to` 绝对 / `by` 相对当前点；控制点参考系 from start|end|origin（缺省 to→Origin、by→Start）。
  - smooth 首控制点=前段末控制点对当前点反射（无前段曲线=当前点）；缺省第二控制点=端点。
  - Arc：SVG 1.1 App. F.6.5 端点→中心参数化全实现（F.6.6 λ>1 半径等比放大、F.6.5.2 圆心符号、F.6.5.4 角域），cw=SVG sweep=1；rx×bw / ry(缺省=rx)×bh；64 段/整圆采样。
  - Close 回子路径起点不复制顶点（环闭合由 add_path_stroke closed 隐式完成）。
- `flatten_path_value`：SVG path data px 直映射参考盒局部坐标；段级多组实参=隐式重复（args.chunks(arity)）；M 后续组=隐式 lineto（SVG 2.3.9）并重置 S/T 反射态；S/T 反射独立跟踪。
- helpers.rs：`resolve_radius_length` 收紧为 `pub(crate)`（shape_len_px 复用）。

css-parser 面（本刀第五文件 `values/serialize_shape_specified.rs` 新模块 + 三处联动）：
- `serialize_shape_specified.rs`：circle/ellipse/inset/polygon 声明串 canonical 化
  （cssom-1 §serializing-css-values 数字最小形剥 `+`/尾零折叠/`-0` 消形；
  css-shapes-1 closest-side 缺省整省 `circle(closest-side)`→`circle()`；
  css-values-4 §position y 缺省补 center——pair 形（kw+lp 全落）不补，
  shape-outside-circle-009 expected_inline `circle(at 50px center)` oracle 实证；
  `at` 后空 position 拒收（`circle(at)` → None，空表落双 center 缺省非合法省略形）；
  polygon evenodd 补逗号）。
- `parse_extended_border.rs`：括号感知切分 ×3（inset 偏移 / ellipse dims /
  position pair）+ 孤立 `split_comma_or_space` 删除——**corpus 判据否决已回滚**
  （五 corpus A/B 实测净 −49：masking P2F=57 + shapes P2F=12，二十一轮
  「期望串含 calc 时本机拒收→expected 落回静态值」回落链在残面⑤三件套在树
  前提下仍未保持，b1v6/b1v7 机制大倍数复现；文件与 clip_path.rs 接受面测试
  一并 git checkout HEAD 还原，拒收语义维持，FIXME(残面⑤) 挂账保持）。

钉测 5 例（border_shape tests）：三角顶点恒等、path M/L/Z 顶点恒等、cubic 采样含 (50,75)、cw 上半弧圆心距离 ±0.1、hline/vline 顶点恒等。

## 探针差表（reftest-upstream，15:41 实测回填）

双 bin（bin-j23J2A `d502b182` / B-face bin `78abca73`，reftest-up-j23-*.txt）
border-shape lens 页级：

| 页 | J2A（无本刀） | B-face（含本刀） |
|---|---|---|
| css-borders 汇总 | 44/49 (89.8%) | **45/49 (91.8%)** |
| border-shape-absolute-coords-shape.tentative.html | ✗ (4.67%) | **✓ (0.39%)** |
| border-shape/border-shape-shape-arc-miter-limit.html | ✗ (0.44%) | ✗ (0.31% 改善仍 ✗) |

本刀第二靶页（absolute-coords，无 geometry-box → HalfBorderBox 缺省参考盒）
翻绿；第一靶页（arc-miter-limit）失败率收窄未过线——弧顶点采样与 miter 极限
残面留后续刀。`reftest` 本地 725 fixture 全套（15:36）border-shape lens 零差
（本地 fixture 无 shape()/path() 消费页，预期内）。

## 五 corpus 对账（reconcile-five-corpus.py，results-j23r2-j23{B,J2A}，15:4x 收账）

J2A（五文件全反向）↔ B 实测：

| corpus | J2A | B | delta | F2P | P2F |
|---|---|---|---|---|---|
| css-gaps | 3838 | 3838 | +0 | 0 | 0 |
| css-multicol | 943 | 943 | +0 | 0 | 0 |
| css-typed-om | 9947 | 9947 | +0 | 0 | 0 |
| css-shapes | 5125 | 5123 | **−2** | 10 | 12 |
| css-masking | 2368 | 2321 | **−47** | 10 | 57 |
| TOTAL | 22221 | 22172 | **−49** | 20 | 69 |

判据否决：SET-IDENTICAL 预期被 masking/shapes 双面打破——全部位移归
parse_extended_border 括号感知切分（唯一 CSSOM 可达面；serialize 模块零
消费者 grep 证明、border_shape/helpers 纯 paint 面不入 testharness）。
gaps/multicol/typedom 三 corpus SET-IDENTICAL ✓（paint 面零入账预期兑现）。
处置（15:4x 收账轮终局）：css-parser 面**整体出树**——parse_extended_border
翻转 + serialize 模块与 mod.rs 声明一并回退（WIP 全文存
.acceptance/wptfyi-css/parser-face-j2-wip.patch，刀 L 候选），提交面仅
border_shape + helpers 残面（7071c0b30）；corpus 中立性改由提交后五 corpus
复验链实证（bin-j23J2R 脚本作废未发射）。

## pin 测试

全量收账（2026-11-17 15:0x，test-guard 包裹，--test-threads=4）：
`cargo test -p zero-css-parser -p zero-style-system -p zero-engine --lib` 三腿全绿——
css-parser 3299/0F（含 serialize_shape_specified corpus 钉测 + inset calc 括号感知
接受面测试翻新）、style-system 2555/0F、engine 4148/0F（刀 J 基线 4,134 + 本刀
5 钉 + 刀 K columns 8 钉 + 刀 K1 inherit 2 钉同 lib 计账）。
注：css-parser 腿系 parse 面在树时点实测；判据否决回滚后 css-parser=HEAD 态
（提交前 cargo check -p 三 crate 绿证明回滚编译中性，HEAD 态此前已验证绿），
engine/style-system 两腿与本提交面逐字节同态、绿账直接继承。

## Provenance

- B bin: 78abca7310474e57a3896bcfbe40bc65（bin-j23B，HEAD 63b0d6b73 + K/K1/J2 全码面 + peer WIP）
- A′ bin: d502b18280b64d1a08a15afcd9de72c4（bin-j23J2A，J2 五文件反向出树）
- 构建窗：build-j23-bins.sh 三刀联合 4-bin 原子窗（快照指纹 pre/post 全等校验
  通过）；同窗在飞 peer 面（es_module/repeater/examples 删集）双侧对称消去。
- 构建后树态：git diff HEAD 指纹 pre/post 逐字节全等（j23-snap-150329）。
