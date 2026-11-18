# 刀 A1e：bg-position 残面池（WAAPI shim 门 x/y + transitions echo 展开）收口

日期：2026-11-18。CLAIM：`.acceptance/wptfyi-css/CLAIM-knife-a1e-bg-position-waapi.txt`。
基线账：`results-knife-a1d-v2`（刀 A1d 落账，bg/typed-om 腿）+ `results-knife-z`
（刀 Z 落账，transitions/masking 腿最新）。并发注记：knife-z（z-index 整数插值
净 +158，b270f1bca/a52903cf0）同窗落库；本 bin provenance 另含 fv1 在飞 WIP
（css-parser serialize/es_module/shim part01f/part06/apply.rs）——claim 域 =
bg-position 四页 +38 面，其余窗口法证归其系（A1d 电池同款纪律）。

## 靶面与根因（双修一暴露面）

A1d 残面池三面：

1. **WAAPI multi-layer 全 at 无覆盖（7F×2 页，x/y-interp）**：got 恒 underlying
   "40px"。根因在 **shim JS 门，非引擎链**——探针三级定谳：
   - solo 页 stderr（ZW_ANIM_VALIDATION_DEBUG）：21×BUILD-ENTRY = 前 3 blocks
     （neutral/initial/inherit ×7ats），multi-layer block 7 anims **根本未到引擎
     register**（`a1e-x-probe.stderr`）；
   - 引擎全链单元探针 `probe_waapi_multilayer_chain`（knife-z 作者写于
     js_anim_registry tests，本 fork 一账两用复跑定谳）：AST 保形 / lenient
     mix 6 层 / clock tick 正确 / canonicalize(multilayer)=Some 全通；
   - 定谳：`part03.js` 门 ④ 单位序列比较对帧间层数失配误杀——
     `'300px, 400px'`→`'500px, 600px, 700px'`（px,px ≠ px,px,px）→
     `_zwUnsafe=true` → animate 整体不登记。knife A1b 已给 position 简写族
     `_zwPosList` 逐层单位种类比较例外（mask/background/object-position），
     **漏 x/y 长手**（css-backgrounds-4 §2.4 `<position-x>#` 同文法逗号列表）。
2. **transitions at(1) echo 不展开（2F×2 页 x/y + 6F main + 14F origin）**：
   expected LCM 展开 6 层 / calc 系数形（initial→px 对）、echo 级联形恒欠。
   transition_registry query_core eased==1.0 echo 短路门例外清单（A1b R1 已含
   mask-position）缺 bg-position 族 4 名。
3. **（修后暴露）CSSA+WAAPI 轴交换 8F（origin 页）**：`center`→`bottom` 单值形
   ——`mask_pos_layer` 单值臂恒入 X（y 缺省 center），css-backgrounds-3 §3
   单值形垂直关键字属 **Y 轴**（`bottom` ≡ `center bottom`）。

## 修（码面全清单）

- `engine/js_dom_shim/part03.js`：`_zwPosList` += background-position-x/y
  （A1b 逐层单位种类比较同语义；漏长手同病同治）。
- `engine/js_dom_bridge/transition_registry.rs`：echo 例外清单 += background-position
  /-x/-y/object-position 4 名（A1b R1 mask-position 同参数 REG 论证：同形对
  t=1 插值≡echo 经 canonicalize 归一恒等零扰动；WAAPI 同形 at(1) shim 门修后
  全过 = 插值臂 at(1) 正确性实证）。
- `engine/animation.rs`：`mask_pos_layer` 单值形垂直关键字（Top/Bottom/YStart/
  YEnd）臂 → 入 Y、x=center；水平/长百分比维持 X 位（原语义保形）。

solo 终态（`a1e2-*.json`，echo 修后 bin `5c8924b0`；轴修在 r2 补验）：
x/y 页各 103→**112 满贯**、main 190→**196 满贯**、origin 258→272（+14，残余
8F 轴交换面即上第 3 项）。

## battery A/B（bin r1 `5c8924b0`，08:25 发射，echo+shim 双修态）

08:25 发射，08:36:19 GATE-DONE，四腿全收（`knife-a1e-r1-082503.log`），
reconcile（`reconcile-a1e.py` 逐页 Pass-name 集合差，全列表不截断）：

- css-backgrounds vs a1d-v2：**gained=38 lost=0**，恰四靶页——
  x-interp +9、y-interp +9（WAAPI shim 门 x/y 满贯）、main +6、origin +14
  （echo 门展开）；solo 预言逐面精确兑现。
- css-transitions vs knife-z：**gained=0 lost=0 SET-IDENTICAL**（语料无
  position 族页，echo 清单加名零外溢）。
- css-masking vs knife-z：**gained=0 lost=0 SET-IDENTICAL**（echo 门共享
  机械零漂移证明面，mask-position 同门不受加名扰动）。
- css-typed-om vs a1d-v2：**gained=0 lost=0 SET-IDENTICAL**（共享串化器
  零漂移面）。

r1 净 +38 零 P2F。

## r2 补充轮（轴修 bin `6541c6d3`，08:39 他流构建刷新、cargo 0.20s 复用确认最新）

solo 终态（`a1e3-*.json`，08:41）：origin **272→280 满贯**（8F→0F 预言精确
兑现）、x/y 112/112、main 196/196 sanity 全守——四页 **700/700 全清**。

battery（08:42:08–08:44:35 两腿全收，`knife-a1e-r2-084208.log`）：

- css-backgrounds vs r1：**gained=8 lost=0**，恰 origin 页 `center→bottom`
  单值形 8 断言（CSSA+WAAPI × at 0.25/0.5/0.75/1）——轴修 battery 面精确
  兑现。
- css-masking vs r1：**gained=0 lost=0 SET-IDENTICAL**（轴修共享解析器
  零漂移证明面）。

## 净账定谳

**刀 A1e 净 +46 零 P2F**（bg 腿 a1d-v2 3442→3488）：
- r1 +38（shim 门 x/y +18、echo 门 main +6/origin +14）
- r2 +8（轴修 origin `center→bottom` 面）
- transitions/masking/typed-om 四跑（r1+r2）全 SET-IDENTICAL 零外溢。
A1d 池 WAAPI bg-position 通道 14F + transitions echo 展开位 4F 认领清账；
修后暴露轴交换 8F 修清。四靶页 x 112/112、y 112/112、main 196/196、
origin 280/280 全满贯。
