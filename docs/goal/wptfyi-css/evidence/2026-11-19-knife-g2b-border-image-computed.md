# 刀 G2-b：border-image computed-opaque 扩面（2026-11-19）

## 刀域与归因

css-typed-om `the-stylepropertymap/properties/border-image-{outset,repeat,slice,width}.html`
四页 computed 臂（arith2 r2 b1 JSON 法证：`assert_class_string: expected
"[object CSSStyleValue]" but got "[object CSSUnitValue|CSSKeywordValue]"` 形）。

页结构（border-image-outset.html 源读）：`runPropertyTests` 每值形单测覆盖
**set + specified-get + computed-get 三臂**，computed 臂败 → 整测记 Fail——
测试名 "Can set 'X' to a <type>: <value>" 不指明败臂（臂归因法：message 形
分类，见 arith2 b1 JSON 逐页分解）。number 臂 `Invalid unit for property`
形归刀 G2-a 账（unit 分支 famName 直派，兄弟 fork 17:34 落库口径 gained=25
lost=0）；`calc(2 + 3)` number 折叠形归 arith 残池。

## 机制

`_zwOmComputedUnsupportedProps` 清单 += border-image-outset/repeat/slice/width
四键（crates/engine/src/js_dom_shim/part10-typed-om.js 单文件 face）。机制承
刀 P border-*-radius 先例：computed 面（`__zwOmComputedFace` 旗标闭包）命中
清单 → 基类 opaque 回落（`text !== ''` 门由 'got CSSUnitValue' 反证宿主
computed 文本非空——已过空文本门、走到 typed parse）；declared/specified 面
不受影响（typed 解析 equal 照常）。

页面声明（css-backgrounds-3 computed 语义 Typed OM L1 无分型）：
- outset/width：computed 数值缩放多值（`1 1.2` / `30px 2 45px` 形）
- slice：`1 2 fill` 关键字混排多值
- repeat：关键字（stretch/repeat/round/space）——页面对全部值形声明
  `computed: (_, result) => assert_is_unsupported(result)`

## 预口径（arith2 b1 JSON 分解）

四页 computed 臂约 ~16F：outset length ×4、repeat keyword ×4、slice
percent ×2+、width length/percent ×6+（number set 臂除外——G2-a/arith 残池）。
哨兵（animations/pva/masking/transitions）零位移为收账前提。

## 电池

knife-g2b-battery.sh（承 knife-arith2-battery.sh 配方逐字：face patch 保存 +
checkout/恢复 trap + test-guard 包裹 + RIDE census S1/S2/S3）。b0=HEAD 剥
face / b1=HEAD+face。发射记录：17:39:00 zw-g2b-battery（arc 电池
B0-BUILD-FAILED E0308 ×4 出窗后入窗；接领 fork 依 NOTES 认领执行）。
**S1/S3-MISMATCH 在案**：构建窗 ride 文件在飞（fontcycle probe-retry/C
E0308 修复嫌疑）——收账判据 owner 划线见 NOTES：仅收四页预期 delta，
四页外或哨兵 lost>0 整账作废待重跑。

## provenance

b0 md5 `614d919b536123055f50eeffd1a89233` / b1 md5
`b7f642f723fd152288661887fe3b3cee`（face patch
`.acceptance/wptfyi-css/knife-g2b-face.patch`，reverse-check PASS；b1 bin
`strings` 含 `刀 G2-b` 注记为 include_str 全量入 bin 佐证——收账前核）。

## 结果（收账时回填）

- RECONCILE：
- 四哨兵：
- provenance 核验：
