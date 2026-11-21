# 刀 OM-15 · calc-size-typed-om.html computedStyleMap 串化 19F（2026-11-21）

## 认领与边界

- 池＝css-values calc-size/calc-size-typed-om.html **19F/P=0**（census 源
  results-knife-sizing-grammar/b1r css-values.json，基 9da416ba2＋
  SIZING-GRAMMAR face；HEAD 17305e93b 时认领，认领笔
  ebb43f07c/acf7cdac3）。
- 对端 S2 claim（ZeroWeb-2 NOTES-knife-s2-calcsize-basis-claim，14:1x）
  明示 typed-om 19F 独立通道不触；CS-INTERP-S2 claim（17305e93b）为插值
  lane 不相交。本刀域＝crates/engine/src/js_dom_shim/part10-typed-om.js
  单文件，不触 style-system/layout-engine。
- 同轮前置笔：本地 CALC-SIZE-USED-S2B 未验证面板（15:47 设计＋16:07–16:26
  部分四文件 face）与对端 S2 同池同构撞面，按碰头协议剥除让位——存档
  .acceptance/wptfyi-css/knife-calcsize-used-s2b-face-local-yielded-to-zeroweb2-s2-v3.patch
  （1070 行 5 文件），设计档 knife-calcsize-used-s2b-design.md 留档。

## 根因（探针 zz-om15probe 定谳，run2 表）

19 子测同形：`e.style.setProperty(p, <calc-size 形>)` →
`e.computedStyleMap().get(p).toString()`，期望 **declared calc-size 的
computed 形 canonical 串**（css-values-5 §8 计算值＝结构保形），实得
gCS used px（784px/0px/400px…）或 none。computed 面 getPropertyValue
走引擎计算串——calc-size 形被 used-value 解析吞掉。

探针三列（declared / gCS / csm，fsz=16px）：

| # | 输入 | declared（现役 canon） | 期望（computed 形） | 差异 |
|---|---|---|---|---|
| 0 | calc-size(auto, size) | 同左 | 同左 | — |
| 2 | calc-size(min-content, size * 2) | …2 * size | …2 * size | — |
| 3 | max-width: calc-size(max-content, size / 2) | …0.5 * size | …**(0.5 * size)** | 除法源括号 |
| 7 | calc-size(any, 25em) | …25em | …**400px** | em 绝对化 |
| 10 | calc-size(any, min(30px, 2em)) | …min(30px, 2em) | …**30px** | 纯 px math 求值 |
| 15 | calc-size(calc-size(2in, 30px), 25em) | basis 192px ✓ sum 25em | …, **400px** | sum em 绝对化 |
| 其余 13 例 | | | | declared 已即期望 |

max-* "none" 面：declared 通道**两例均存槽成功**（探针 declared 列实
证 `calc-size(max-content, 0.5 * size)` 原文在槽），"none" 仅引擎计算
串缺承载——computed 回显臂从 inline declared 原文取值，绕开计算串。

## face（part10-typed-om.js 单文件 +155）

1. `_zwOmCalcSizeSlots` 槽集（13 尺寸槽，镜像写面文法门
   calc_size_decl_gate 清单）。
2. `_zwOmCalcSizeComputedForm` canonical 化链：basis（关键词集保形＋
   嵌套 calc-size 递归＋length-percentage 绝对化）/ sum（深度 0 项拆＋
   纯 px 叶 min/max/abs 求值＋% 项前置稳定排序＋除法源积项括号启发）。
   长度表：em/rem 按 gCS font-size、in/pc/pt/cm/mm/q 常数（declared
   通道已绝对化者幂等）；ex/ch 字体度量依存 fail-closed 保形。
3. computed 面 getPropertyValue OM-15 臂：declared 顶层 calc-size 形且
   无 var( → 回显 canonical 串；结构不锚/em 无字号基 fail-closed null
   回落引擎串（零漂移）。
4. mapGetAll OM-15 臂：computedFace＋槽集＋calc-size( 前缀 → opaque
   基类实例携 canonical 串（css-typed-om-1 §get 不可分型回落同型；
   declared 面不受影响）。
- FIXME 注记：除法源识别用小数因子启发（declared 串化已失 `size / 2`
  结构源），字面 `0.5 * size` 误括为已知近似；结构源旗标待 declared
  串化补齐后换真判。

## 电池

knife-om15-calcsize-typedom-battery.sh（配方承 knife-sizing-grammar，
patch md5 7673ead6188127ea0b6d47f41dc22083，zw-knife-om15 unit）：
b0r（face 剥除＝HEAD 树）/b1r（face 重放）双臂，test-guard 包裹
（build per-proc 6/total 14/1800s，leg per-proc 4/total 8/3000s +
CASE_WALL_S=60），M2 distinct-md5＋FACE-MARKER token 增量门
（CalcSizeComputedForm b0r=0/b1r=4、CalcSizeSlots 0/3）。
腿：{css-values 主靶, css-typed-om 通道哨, css-sizing, cssom} × 双臂。
判读 knife-om15-calcsize-typedom-judge.py：calc-size-typed-om 19F 恰集
F2P 硬门＋P2F=0 全语料硬门＋F2P=19 恰集（非主靶 F2P 即清单出列）。

## 判读

**OM15-GATE-PASS**（battery-1846，zw-knife-om15-calcsize-typedom-b2 unit，
18:35:20 build 起 18:45:53 收账 10.5 分钟；二进制绑定 b0r=
`e80d719ddcff867adbdf5e9953eed2a0` / b1r=`f41abf87d0ca8f822ec921c8e29840f7`
M2 异 md5）：

- [css-values] P 622→641（**+19**）：F2P=19 全数 calc-size-typed-om.html
  （b0 0P/19F → b1 19P/0F，恰集精确兑现），P2F=0、new=0、gone=0；
- [css-typed-om] 12406 keys SET-IDENTICAL（P 12287→12287）；
- [css-sizing] 5700 keys SET-IDENTICAL（P 2979→2979）；
- [cssom] 1630 keys SET-IDENTICAL（P 888→888）；
- 对端在飞池七页（flex-basis/box-sizing/width/height/
  interpolate-size-computed）零位移 ✓。

判读订正一笔：首判 OM15-GATE-FAIL 系 judge 脚本 targetF2P 归因笔误
（`in pg[0]` 取页路径首字符，恒假）——F2P/P2F 计数本身首判即 19/0 正确；
修正归因后对**同二进制同 JSON 工件**复判 PASS（judge rc=1→rc=0），
电池体与腿产物未重跑。

## 预注册对账

- 主靶 19/19 F2P，expected 串与语料失败消息 ground truth 逐条对表
  （探针 run2 19/19 OK 预验）；
- typed-om/sizing/cssom 三哨 SET-IDENTICAL（P2F=0）；
- flex-basis/box-sizing/width/height/interpolate-size-computed 七页＝
  对端 S2 v3/widthshrink 在飞池，本刀不触（SET-IDENTICAL 预期）。
