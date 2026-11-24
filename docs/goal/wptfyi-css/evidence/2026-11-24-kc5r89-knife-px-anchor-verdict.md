# kc5r89 knife-2′ verdict——gCS used width/height px 锚回＋x.5 half-down 量化（2026-11-24）

## 结论

- **预注册 knife-2（echo 串化改精度保留）证伪于实施前**：replica 六通道代数证明四个
  width/height gCS 读输出恒整数（单串化器），±1 来自 rect 表值逐元素分歧（布局边缘整数
  取整，方向随跑翻 r1 vs r2/r7）——仅改串化使两元素各打印各自互异 rect 值，`w1==h2`
  探针门构造性不可过。margin 通道 r7 `95429.1px`（f32 95429.125 的 6 位有效截断）另证
  computed 通道亦非精度保留形，「对齐 computed 口径」不成立。
- **实施刀面 knife-2′＝rect 伪影带内 px 锚回**（`used_size_from_border_box`，selector
  :1302 / handle :1329 双通道唯二汇点）：computed width/height 为有限 Px 且
  `|rect − spec| < 1px`（整数取整误差恒 <1px＝伪影带）时锚回 specified f64 源；≥1px
  真实钳制不触，rect 语义全保。同 specified 值跨元素断言对（gCS(t1).width vs
  gCS(t2).height）同源 f64 同串＝'Shared globally' SAMPLING-FLAKE 家族根因确定性消除。
- **run-1（genK1，half-away）P2F=1 定谳 → 同轮修订 half-down → run-2（genK2）全绿，
  judge/verify 双算 PASS，NET +1**（唯一 F2P＝flaky 键消flake）。

## run-1 P2F 定谳（'Fixed: random(fixed <number>, a, b)'）

四证链：

1. 页 `random-computed.tentative.html` :893 硬编码
   `test_random_equals(t1ComputedWidth, "150000px", "Random value with fixed should be 150000px")`；
2. randomFixed 类 `random(fixed 0.5, 1px, 300000px)` → X = 1+(300000−1)×0.5 =
   **150000.5**（f64/f32 皆精确 .5 边界）；
3. genK1 锚臂 half-away `round(150000.5)` = 150001 ≠ 页钉 '150000px' 翻红；
4. 三 bin 实值探针（恒失败消息带原始串）：**b0=150001px / genK1=150001px /
   genK2=150000px**。b0 电池 Pass 依赖页内元素 rect 恰落 floor 侧（150000），探针元素
   rect 落 ceil 侧（150001）＝±1 伪影逐元素方向性直证；b0 该键 Pass 属**碰对形**
   （bistable），genK1 固定化打破碰对，genK2 固定 150000=页钉形且确定性化。

规约判定：CSSOM-1 未规定 used 值整数串化形；corpus 唯一 .5 边界钉形（页期望
'150000px'）＋ b0 该键实际产出一致 ⇒ 整数串化 .5 边界规约＝half-down（x.5 下取整）。
边界外（frac≠0.5）与 round 逐位同形 ⇒ corpus 唯一受影响键＝该键（电池 v2 P2F total=0
实证）。judge as-run FAIL 不回填（judge-r89-asrun.out 存档），条款匹配器不另修。

## run-2 门账（genK2 md5 e36e61f5，指纹 ZW-KC5R89-PXSRC-FACE=1，FACE md5 b9d67939）

| 门 | 结果 |
|---|---|
| 探针 A 臂 replica ×10 | 10/10 六通道 per-run 统一（w1==h2==w2==h1 gate OK） |
| 探针 B 臂 flaky ×5 | 5/5 Pass（消flake确定性） |
| 电池双腿 RC=0 | 23:17:47–23:24:18，results-knife2-b0/b1（genK1 原档不覆写） |
| G0 flaky 键 | b0 Fail → b1 Pass ✓ |
| G5 presence | 9881/9881，GONE=0 NEW=0 ✓ |
| G2 P2F | **total=0（hard=0）**＋哨兵 8 持 F ✓ |
| G3 集外 F2P | 0 ✓ |
| G4 SET-IDENTICAL | 非 random 页 diff=0 ✓ |
| judge 恒等式 | adjusted NET=+1；verify 独立双算 9882/9882 common，F2P=1（恰 flaky）P2F=0，P1−P0=1==F2P−P2F=1 ✓ |

## 落库与代际

- face 单文件提交 **a5c66047b**（fix(engine)），rebase 后活 hash；face md5 b9d67939
  与 genK2 产线恒等。
- bins/GENERATIONS.txt：genD1 df0efef5（定谳仪表面）／genK1 606b83ce（half-away，
  as-run FAIL 在案）／genK2 e36e61f5（half-down，落库形）；b0=kc5r88/bins/r88-b1-runner
  b8e53ab5。
- 全链账面 `.acceptance/wptfyi-css/kc5r89/`（本地）：NOTES §零-§九、probe-k2.log、
  judge-r89-asrun.out（FAIL 原样）、judge-r89-v2.out、verify-r89-v2.out、
  fixedhalf-three-bin.log、battery-r89-v2.log、probe2-results{-genK1}、
  results-knife-b0/b1、results-knife2-b0/b1。

## 残面移交

1. margin computed 通道 6 位有效截断形（'95429.1px' ← f32 95429.125）＝精度损失面新
   候选刀（与 echo 整数面分立）；
2. unresolvable-argument `Selecting a resolvable item…`（fixed 0.999）持 F（r88 移交）；
3. doc 指针换代窗（R386）独立挂账（定谳轮降级维持）。
