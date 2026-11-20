# 刀 CALC-SIZE-DEFINITE · calc-size() definite-sum 尺寸槽求值（2026-11-21）

## 目标与机制

css-values-5 §8 calc-size()：sum 不含 `size` 词元时为 definite，求值结果与
basis 无关——可离线包 `calc(<sum>)` 走既有 `LengthValue::Calc` 求值径
（em/viewport/ch 上下文在 resolve_computed_style 兑现），落七尺寸槽
（width/height/min/max-width/min/max-height/flex-basis）。含 `size` 词元的
intrinsic 形需 layout 期内在尺寸，fail-closed 维持既有拒收。

face `ec2557f13`：apply.rs 单文件 227 行——`calc_size_definite_sum` 提取
（顶层逗号切分＋`size` 词元边界扫描＋括号平衡结构门）、
`apply_calc_size_definite` 七槽臂（parse_length_or_math＋
sizing_length_is_valid 双门，flex-basis 落 `FlexBasisValue::Length`，
落槽清 anchor_size_slots 同既有尺寸臂口径）、match 分派前拦截点、
FACE-MARKER、单元 2 组。

## 电池（.acceptance/wptfyi-css/results-knife-csdef）

- 双臂：b0 `2695282c`（face 剥除）/ b1 `db888632`（face 在树）；双臂同载
  ride 6 文件（MOTION-1b 三文件＋es_module＋MATHFN 探针四文件）→ delta 相消。
- 判别：FACE-MARKER b0=0/b1=1；M2 b0/b1 异 md5；S1/S2/S3 ride md5 三窗恒。
- 8 腿：values/flexbox/motion/typed-om × b0/b1，rc=1（runner 有败测试正常）。

## RECONCILE（gained=41 lost=0）

| 页 | P 变化 |
|---|---|
| values calc-size/calc-size-height.html | 0→14 |
| values calc-size/calc-size-width.html | 0→7 |
| values calc-size/animation/calc-size-height-interpolation.html | 0→10 |
| values calc-size/animation/calc-size-width-interpolation.html | 0→10 |

## 归因与严谨性

- 21 靶（height/width）逐条为 definite 形（any/px/em/嵌套 definite）精确
  命中；`size` 词元形与 box-sizing 页零变化（属 INTRINSIC 域，符合 fail-closed）。
- 插值两页 +20 逐腿为 transition 端点 t=1 腿——端点 application 走 apply
  路径过本臂，非 lerp 面、非幻影。
- 断言对称：gained 四页 b0/b1 断言总数 9866 双侧相等，零 ASYM。
- 哨兵状态级 SET-IDENTICAL：flexbox 4713 / motion 4955 / typed-om 12406
  三窗逐页零翻转；非 calc-size 页零翻转。

## 预注册对照

预测（NOTES-fama-values-probe-census CALC-SIZE-DEFINITE 回执）：F2P 上限
47／高置信 29。实收 +41 在带内。构成偏移如实记录：

- flex-basis 页 18F 预测未兑现：layout 不消费 Calc basis（挂账）。
- 插值端点 +20 超出预测：机制已归因（端点 application 路径）。

## 挂账 CALC-SIZE-INTRINSIC（~209F）

box-sizing 全量 30F＋`size` 词元 intrinsic 形＋% 腿 gCS 语义＋flex-basis
layout Calc basis 消费——需 layout 期内在尺寸求值径，另刀。

## 环境事件（镜鉴）

暂存 face 曾被兄弟流 `git reset --hard` 类操作清空（reflog HEAD@{0}）；
发射时提取的 face.patch 完整恢复（staged blob OID `e8356ea65` 与已扫快照
字节恒等），227 行零丢失——电池 patch 文件双用途（strip 舞蹈媒介＋纯 face
快照源）在本事件中兑现。
