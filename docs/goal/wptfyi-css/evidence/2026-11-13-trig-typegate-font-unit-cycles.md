# trig 型别门修复：font-property-unit-cycles 95F 回归根因 + css-properties-values-api −5 闭环

2026-11-13。承接 [DC-3 v14 对账](2026-11-13-dc3-v14-reconciliation.md) §4 四小回归
挂账中的 css-properties-values-api −5 页面归因，同日修复。

## 1. 起点形态

`font-property-unit-cycles.tentative.html`（95 子测）v14 shard 0P/95F，11-11
5P（虚假空过：trig calc 声明 unsupported → font-size 继承父 30px 恰等于期望值）。
即该套件 107→102 的 −5 全额来自此页，且 11-11 的 5P 本身无真实求值。

95 子测三组：G1 直置 font-* 属性（7 属性 × 5 单位 = 35）、G2 registered
custom property 经 var()（35）、G3 registered custom property no-cycle（25）。
失败值形态（v14 后实测）：G1 font-size 恒 10（target 样式表回退）、font-weight
恒 400 / font-stretch 恒 100（默认值回退）、余 NaN；G2/G3 全 NaN。

## 2. 归因链（双探针切面）

本地探针页（zzz-trig-probe{,2}.html，wpt-data 不入库）二轮切面：

1. **失败值全是回退非错算**：em/ex/cap/ch/ic 五单位恒 10 = `#target
   { font-size: 10px }` 原样保留——声明整体被丢，em 基准问题尚不存在。
2. **plain calc 全对**：`font-size: calc(2em)` → 60px（父级基准，css-values-4
   font-size 相对单位按父元素字体解析，计算路径正确）；`margin-bottom:
   calc(2em)` → 20px（自身基准）正确。
3. **`tan(0.25turn)` 求值成功**（字面角度 parse 期弧度 Number 化），凡
   `tan(atan2(1em, 1px))` / `cos(atan2(…))` / `atan(tan(180deg * …))` 复合形
   声明全丢。
4. **定谳**：`calc_expr_math_type`（values/parse_extended.rs）型别表中
   sin/cos/tan 实参仅收 `MTNumber`；`atan2(1em, 1px)` 同族长度实参产出
   `MTAngle`，`tan(MTAngle)` → `MTInvalid` 整式无效 → 槽位解析 None → 声明
   丢弃 → 页面回退值伪装成「错算」。enum 文档注释本就写明「x 为弧度 number
   或 <angle>」，是型别门滞后于设计。

## 3. 修复

`parse_extended.rs` 型别门：sin/cos/tan 实参放行 `MTNumber || MTAngle`
（css-values-4 §10.6 trig functions："A is a <number> or an <angle>"，
<https://drafts.csswg.org/css-values-4/#funcdef-sin>）；sqrt/exp/log 保持
仅 number。eval 臂零改动——字面角度与 atan2/asin 族产出同为弧度 f64。
asin/acos/atan（实参 ratio、产出 angle）保持不变。

## 4. 验证

- 单测：zero-css-parser lib **3,184 passed / 0 failed**；zero-style-system
  lib **2,528 passed / 0 failed**（test-guard 包裹）。
- 探针 2 七腿全绿（font-size trig 父级基准 30/60、margin trig 自身基准
  10/100、plain calc 两基准）。
- 原页：0P/95F → **5P/90F**——G1 font-size 五腿精确回位（em 30 / ex 24 /
  cap 24 / ch 15 / ic 30，与 Chrome 分母期望值逐项一致；CalcContext 0.8/0.5/1.0
  字体度量回退比恰合）。
- `css-values/acos-asin-atan-atan2-computed.html`：24P/28F → **27P/25F**
  （`atan(tan(…))`、`tan(atan2(round(…)))` 复合族解锁）。
- 全量套件回归门：css-values 基线 3,735P（g24 本地）→ 结果见 §6；
  css-properties-values-api 全套件同步实测。
- 无 invalid 断言面依赖旧门：语料扫描 acos-asin-atan-atan2-invalid /
  calc-invalid-parsing 等页无 tan(<angle-expr>) invalid 断言，放宽无回归。

## 5. 残余桶（后续批，非本批）

1. G1 其余 6 槽位（font-weight/font-size-adjust number、font-stretch
   percentage、font-style oblique 角、font-feature/variation-settings 串内
   数值，30 腿）：槽位无 calc 支持，需各槽 calc + eval 接入并按
   css-properties-values-api #dependency-cycles-via-relative-units 以父级
   字体为 font 相对单位基准。
2. G2/G3 registered custom property typed compute（60 腿）：gCS 注册自定义
   属性现回原始 `calc(...)` 串（NaN），需 @property syntax 型别求值 + var()
   通道 font 单位基准。
3. css-values rotate 槽（25F）：裸 math 函数（无 calc 包裹的
   `atan2(1px, -1px)`）不被接受；serialize ctx 未注入 sibling 序 / 字体
   （`atan2(1, sibling-index())` 族）；atan2 `<percentage>` 实参 eval
   （按值作 number）。
4. 四小回归账：css-properties-values-api −5 **本批闭环**（5P 真实回位）；
   writing-modes −4 / css-lists −2 / css-size-adjust −1 仍挂账。

## 6. 全量套件门终账

- css-values 全量 256 页：3,735P（g24 本地基线）→ **3,933P**，逐页对账
  **0 回归页**（253 平 / 3 增 / 0 降）。增量拆账：
  - 本批型别门可归因 **+4**（acos-asin-atan-atan2-computed +3、
    hypot-pow-sqrt-computed +1）。
  - line-break-ch-unit **+194 为基线伪红修正**：页内零 trig 使用，与本门
    无语义关联；A/B 定谳——暂存本批改动重建（NO-GATE 二进制）同页
    **194P**，当前二进制两轮复跑均 194P，语料文件 mtime 10-30 零漂移——
    g24 基线轮（11-13 02:23）该页 194F 为环境/二进制态伪红（伪红不作账）。
    master.md「line-break-ch 194F」独立桶据此**实际已闭**（非本批战果）。
- css-properties-values-api 全套件本地 **117P**（v14 shard 102P 为旧
  HEAD，不可直比；页级：font-property-unit-cycles 0P→5P 为本批净增，
  其余增量归 g24 attr 批）。

