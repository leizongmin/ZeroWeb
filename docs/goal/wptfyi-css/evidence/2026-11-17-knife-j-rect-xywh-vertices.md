# 刀 J·rect/xywh vertices 批（2026-11-17）：border-shape-shadow 族 3 键确定性回归修复

## 回归定谳与修复范围

rendering-compat 流在册回归 `css-borders/border-shape` 族 3 键
（border-shape-combined-shadow.html 7.69%、border-shape-shadow-semitransparent.html
9.19%、border-shape-shadow.html 10.08%，三跑确定性稳定 F）本刀定谳修复。

**根因**：07fcc9e0e（basic-shape knife）为 `ClipPathValue::Shape|Path|Rect|Xywh`
四兄弟在 `border_shape_vertices` 加了空顶点 FIXME 臂——此前三页的
`border-shape: xywh(0% 0% 100% 100%)` 解析失败 → 声明无效 → 常规盒渲染 →
页过；解析落地后 border-shape 激活 → 空顶点 → `background_vertices()` 空
path_fill（绿底失）+ Stroke plan 空描边（黑边框失），仅 box-shadow 独立
通道留画（紫/蓝影）＝三页 ref（SVG rect 语义：绿填充 + 黑描边 + 外影）
大面积失配。

## 交付面（3 面 + 3 钉）

1. **Xywh 臂**（css-shapes-2 §basic-shape-rect-notation）：x,y 起点与
   w,h 尺寸（% 相对参考盒宽/高）解析成对角点。
2. **Rect 臂**：四值 = 自参考盒左上角起算的 top/right/bottom/left 缘位置
   （非 inset 距离）；top/left auto → 0，right/bottom auto → 参考盒右/下缘。
3. **`rect_shape_vertices` 共享发射**：对角点顺时针四顶点（与 Inset 臂同
   winding）；`round` 槽（`BorderRadiusValue`）水平半径相对内矩形宽、垂直
   相对高解析（css-backgrounds-3 §border-radius 百分比基准），单角钳制
   min(iw,ih)/2 复用 `rounded_corner_polygon_in_box`；全零半径回方角。
   FIXME：css-backgrounds-3 §5.5 相邻半径重叠等比缩放未实现（与 inset()
   round 同款单角钳制近似）。**scope 决策**：Shape/Path 命令串接几何另属
   独立刀面（命令展平 + 弧参数化），保持收窄 FIXME 空顶点——三靶页仅消费
   Xywh；`border-shape-absolute-coords-shape.tentative.html` 等仍 F 属既有
   独立键非本刀账。
4. **钉测 3 个**：xywh 全盒四顶点（三页消费面几何）、xywh 偏移 + round
   弧顶（r<iw/2 时弧顶 = 两角弧最高点非边中点——首版钉测判据写错已修正）、
   rect 缘位置 + auto 语义。engine lib **4,134/0F**（4,131 + 3 钉）。

## 验证：reftest 探针差（vintage 行为锚）

| 页 | A′ bin（刀前） | B bin（刀后） |
|---|---|---|
| border-shape-combined-shadow.html | ✗ F 7.69% | ✓ **P 0.00%** |
| border-shape-shadow-semitransparent.html | ✗ F 9.19% | ✓ **P 0.00%** |
| border-shape-shadow.html | ✗ F 10.08% | ✓ **P 0.00%** |

A′ 探针实测速率与在册回归速率逐页吻合（7.69/9.19/10.08）＝vintage 锚定。
B 侧 border-shape 目录整体 90.6%（44/49），余 5 F 全为独立键
（shape()/path() FIXME 面、miter-limit、stroke-from-border、
replaced-iframe），非本刀账。

## 权威账（B↔A′ SET-IDENTICAL）

树态：HEAD 79aaf6264（刀 I 提交）+ peer WIP 对称在树（es_module s419、
css-parser 括号切分、typed-om 刀H、registry/shorthand rustfmt 折行）。A′ bin
现建（stash 本刀 border_shape.rs → build → pop）而非复用 bin-cssa-i1——五
WIP 文件 mtime 10:23:01 晚于 i1 构建（10:10:23），内容是否漂移不可廉价证；
现建 A′ 保证对称消去无假设。实证：A′ 五 corpus 总账 21,907 与刀 I A′ 逐
corpus 恒等＝10:23 事件语义零差回溯定谳。

- A′ bin `5ce2b39f17e100b6ba88ec8a84a67a01`（10:40 前后落，36.66s 增量）
- B bin `5423a44f6be7b737ef296e7e143ba62d`（52.38s 增量）

**SET-IDENTICAL**（bin-cssa-j1 `5423a44f`，五 corpus；A′ 链 10:42:53 发射
10:52:37 GATE-DONE、B 链 10:52:43 发射 11:02:53 GATE-DONE，十腿全 rc=0）：

| corpus | A′（japrime） | B（j1） | delta |
|---|---|---|---|
| css-gaps | 3792 | 3792 | 0 |
| css-multicol | 757 | 757 | 0 |
| css-typed-om | 9944 | 9944 | 0 |
| css-shapes | 5106 | 5106 | 0 |
| css-masking | 2308 | 2308 | 0 |
| **合计** | 21907 | 21907 | **0** |

**PAGE-ONLY 零容忍对账：F2P 0 / P2F 0 / NEW-PASS 0 / NEW-FAIL 0 /
GONE-PASS 0 / GONE-FAIL 0 / 五 corpus 页集合全等、子测试级零翻转**——
本刀净 0：回归修复（reftest 面 GONE-3 靶）零 corpus 扰动。账外靶面三页
F→P 兑现即本刀交付（css-borders 不在五 corpus 账内）。

## Provenance 与过程纪要

- 链 `launch-cssa-japrime-full.sh` / `launch-cssa-j1-full.sh`（transient
  service，test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1800 包腿；
  CASE_WALL_S=90；results/transcript 按 vintage 分名）。
- 对账脚本 `.acceptance/wptfyi-css/reconcile-five-corpus.py`（刀 I 轮同款
  复用）。
- 修改范围 1 文件：engine/paint/border_shape.rs（两臂 + 共享发射 helper +
  3 钉；`BorderRadiusValue`/`RectCoord` 按 `values::parse_basic_shape` 路径
  导入——该模块非 glob 再导出）。首版钉测 `offset_round` 弧顶判据写错
  （r<iw/2 无边中点顶点，弧顶 = 两角弧最高点）——几何实现正确，钉测修正，
  30/0F 复验。
- peer 会话并发注记：B build 窗口 peer 链在跑 editing 探针
  （insertimage/multitest，runner-only 不持 cargo 锁）——共享 target 的
  in-place runner 换装窗语义零影响（editing 流不消费 border-shape 面）。
  另 clone（ZeroWeb）T79 perf renderer 在跑——RSS 记账与 CPU 负载正交，
  未错峰。
