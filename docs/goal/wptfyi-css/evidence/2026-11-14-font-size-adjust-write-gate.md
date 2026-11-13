# font-size-adjust 写面文法门 57F 收口 + 指定值 ex-height 串化剥离

2026-11-14。承接 [font-* 数值槽 calc 接入批](2026-11-14-font-slot-calc-parent-basis.md)
§5 残余桶第 3/4 项，同日修复（g30）。

## 1. 起点

`font-size-adjust-invalid.html`（57 子测）0P/57F 全败；`font-size-adjust-valid.html`
14P/5F（4F 为 metric 前缀串化非 canonical）；css-fonts 全量基线（g29 后）
**2,411P/2,587F/4T**（163 页）。

## 2. 归因链

invalid 页 57 形（`auto`/负数/`0.5 ex-height` 反序/未知 metric
`em`·`ch`·`ic`/裸 metric/`<metric> none`/双 basis 三 token/`none`·`from-font`
带后缀）在 **apply 链 parse_font_size_adjust 的 splitn 结构下全部已拒**
（metric 后必有 basis、负数域门、未知名拒）——但写面
`VALIDATED_PROPERTIES` 白名单无 font-size-adjust，`validate_decl_value`
白名单外直接放行，shim CSSOM inline store 不经 apply，e.style 回显原文即
Fail。即：语法裁决已存在，缺的是写面接线（g28 text-size-adjust /
g29 font-* 家族同型）。

## 3. 修复

- **apply.rs**：`VALIDATED_PROPERTIES` 第 19 项 + `validate_decl_value`
  font-size-adjust 文法门——整值 math 前置放行（语法由通用 math 门裁，
  eval 留 apply 链带 font ctx）；其余按首个空白剥 metric 前缀后 basis
  复检（from-font | 非负字面数 | math 函数；字面 `inf`/`nan` 经 f64
  parse 后 is_finite 门拒）。空白剥前缀**不进 calc 内部切分**，
  `ex-height calc(1 * tan(atan2(1em, 1px)))` 型多空格 math basis 保真。
- **serialize_math.rs**：`fold_decl_specified` font-size-adjust 臂——
  css-fonts-5 §font-size-adjust-prop 初始 metric ex-height 串化剥离
 （`ex-height 0.5` → `0.5`/`ex-height from-font` → `from-font`/
  `ex-height calc(0.5)` → `calc(0.5)`，裸 <number> 与 ex-height 前缀等价）；
  其余 metric 保持前缀 + basis math canonical 折叠
 （`cap-height calc(0.5 + 1)` → `cap-height calc(1.5)`）；sign 树形不折叠
  保留原文（乘法重排挂账）。
- 单测：`font_size_adjust_write_gate`（style-system，57 形拒面 + valid 形
  + css-wide 放行）；`fold_decl_dispatch_by_property` 扩 font-size-adjust
  断言（css-parser）。

## 4. 验证

- 单测：zero-css-parser **3,188** / zero-style-system **2,532**（+1 门测）
  全绿（test-guard 包裹）；clippy 双 crate 零告警。
- 页级（直跑 runner，重建后二进制）：
  - font-size-adjust-invalid：0P/57F → **57P/0F 全页清零**。
  - font-size-adjust-valid：14P/5F → **18P/1F**（+4：三条 ex-height 剥离
    + cap-height math 折叠；残 1F = `sign(20cqw - 10px)` 乘法重排串化，
    bucket-1 同源挂账）。
  - font-size-adjust-computed：10P/10F 不变（串化刀只作用指定值面）。
- css-fonts 全量（rebase 298f54c14 后重建二进制，final 轮）：
  **2,472P/2,526F/4T**，g29 后 2,411P/2,587F → **净 +61P 零回归**
 （invalid +57 / valid +4；timeout 4 恒定）。
- unit-cycles 单页复测 **20P/75F 零漂移**（fold 门不触碰 G1 裸 math 形）。

## 5. 残余桶（下批候选）

1. **font-size-adjust computed 通道**（10F，本页逐腿拆解）：
   - `from-font` 解析刀：`from-font`/`ex-height from-font` 期望解析为
     字体真实度量比（本页钉 0.5——默认字体 x-height 比）；`cap-height
     from-font` 期望 `cap-height 0.5`（metric 保持 + 比值落数）；
     `ch-width`/`ic-width from-font` 期望 ~1（approx 断言，现 NaN）；
     `ic-height from-font` 期望 `ic-height 1`。注意口径分歧：unit-cycles
     font-size 相对单位钉 ex=0.8×30（合成比），本页 from-font 期望
     0.5（真实字体度量）——两刀需分离：相对单位保持合成比、from-font
     走字体度量。
   - `cap-height calc(-0.5)`：spec 期 valid（calc 不 clamp）、computed
     钳 0（`cap-height 0`）——font-width 同型刀待搬。
   - `sign(20cqw - 10px)` 容器实参（supported-value 2F，bucket-1 同源）。
2. sign 形乘法重排串化（跨页 ~7F：font-size-adjust 1F + font-weight
   1F + font-width 族 4F + computed 支持性 1F）。
3. bolder/lighter 相对权重表 42F / G2/G3 typed compute 75F /
   inline decl alias 腿 1F / 150.25 串化保真 / gCS text-size-adjust
   computed 串化臂（承前批清单）。
