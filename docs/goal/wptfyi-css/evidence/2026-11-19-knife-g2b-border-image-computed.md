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
face / b1=HEAD+face。

**17:39 首发中止（存档）**：zw-g2b-battery 17:39 发射即中止——S1-MISMATCH
定谳＝anchor 流三文件在发射→b0 build 窗（35s）内被编辑，b0 扫编辑前态、
b1 将扫编辑后态＝非对称 ride，computed_style.rs 波及全部 computed 页，账
不可收（NOTES 17:4x 仲裁段）。重发条件＝ride 面 md5 稳定 + patch 复核。

**22:12:30 重发收账（zw-g2b-battery-2212）**：RIDE 10 文件（atpr2 四面 +
ffix 后残面 + part115/es_module 等在途件，census 存
results-knife-g2b/ride-files.txt）；b0 build 52.5s / b1 build 37.8s；
S1/S3 ride 窗内零扰动（无 MISMATCH）；22:39:24 GATE-DONE rc=0，trap
TRAP-RESTORE applied（face 态即提交态）。发射窗互斥论证见 NOTES 22:13
段（并行 fontcycle-r2 solo 电池 b0/b1 已过 build 窗，part10 strip/restore
不入其已建二进制与已过 census 点）。

## 结果（22:39 收账）

- RECONCILE（ab-compare 逐页 Pass 集，gained=33 lost=0 regressed=0
  pages_gain=4 pages_loss=0）：
  - border-image-outset.html P 26->34（+8）
  - border-image-repeat.html P 33->37（+4）
  - border-image-slice.html P 25->33（+8）
  - border-image-width.html P 22->35（+13）
- 四哨兵（animations/pva/masking/transitions）：零差异（对账输出零行）。
- 预口径 ~16F vs 实收 33：arith2 b1 JSON 分解只计了当时可归因的 computed
  臂 Fail 行；runPropertyTests 每值形单测三臂结构下 computed 臂翻转即整测
  过，实际翻转面大于残池抽样估计。账以本电池自身 b0/b1 为准。
- provenance：b0 md5 `708ff0330c3e97b7193ecfb7b47a0ffe` / b1 md5
  `c4b2d4c860a049c684278fca20bab3f2`（face patch
  `.acceptance/wptfyi-css/knife-g2b-face.patch`，发射前 apply-check PASS；
  17:39 中止批旧 bin md5 `614d919b…`/`b7f642f7…` 已废不收账）。
