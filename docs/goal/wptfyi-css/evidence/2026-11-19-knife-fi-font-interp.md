# 刀 FI：font 车道三面（keyframe 简写展开 + stretch/style/lh 插值臂 + registry 挂表）

2026-11-19。FI fork（CLAIM-knife-font-interp 03:55 认领，侦察定谳 07:5x）。
池 = results-knife-a1c-b2/css-fonts.json 11-18 账三页 ~658F：system-fonts 396F +
font-stretch-interpolation 153F + font-style-interpolation 109F；邻页红利
font-stretch-interpolation-math-functions / discrete-no-interpolation 同通道。

## 机制（三面）

1. **keyframe `font` 简写零展开**（system-fonts 396F 根因）：`block_to_points`
   帧声明原样入 properties 表，apply 端无 `font` 简写臂 → `font: italic 100
   small-caps ultra-expanded 100px/100px Ahem` 整条 no-op，长手全落 UA 默认。
   修复：ingest 侧经 style-system `expand_shorthands` 单源展开（`expand_font`
   16 槽，css-fonts-4 §font-prop 先复位后覆盖）；**只扩 `font`**（apply 端已有
   臂的简写 background/border-color/columns 在 ingest 展开会改插值语义，存量
   P 面 P2F 风险；font 无臂 = 零扰动）。
2. **font-stretch / font-style / line-height 插值臂全缺**：strict
   （`interpolate_property_value_strict`）+ lenient（`interpolate_property_value`）
   双臂补挂——stretch 关键字↔% 数值 lerp（`parse_font_stretch` 映射单源，负 %
   文法拒、上界不钳）；style normal≡oblique 0deg、italic 异族离散、oblique 角
   对 lerp、t∈{0,1} 端点 verbatim、**角域 [-90deg,90deg] 钳 + 恰 0deg 串化
   `normal`**（css-fonts-4 #font-style-prop 值域门）；line-height px 域 lerp
   （used-px computed 形，`normal` 端点对 = 非 animatable 对）。
3. **registry 三名挂表**：`TRANSITIONABLE_PROPERTIES` 82→85（transition 车道
   显式声明与 all 通配同口径；清单 ⊆ strict 臂全集不变量保持）。
4. **apply 写回臂**：`apply_single_property` 增 font-family/style/stretch/
   line-height 委托 style-system `apply_property_value`（单声明单源）+
   font-variant-caps 值域直写（style-system 无单声明臂）。

## 电池账（results-knife-fi-font，11:15 发射 zw-knife-fi-battery）

b0=HEAD 剥面 / b1=FI 树，双 release runner md5 异（M2 过）；S1 ride-along
零扰动、S3 面还原恒等；钉腿 knife_fi 5 passed + registry 17 passed。

| 语料 | b0 P/F | b1 P/F | F2P | P2F | net P |
|---|---|---|---|---|---|
| css-fonts | 5744/2497 | 6274/1969 | 532 | 4→0（二轮，见下） | +530 |
| css-transitions | 992/521 | 994/2196 | 2 | 0 | +2 |

css-fonts F2P 分页：system-fonts +276、font-stretch-interpolation +130、
font-style-interpolation +86、font-stretch-interpolation-math-functions +23、
discrete-no-interpolation +17。

**一轮 P2F=4（font-style-interpolation，角域门缺位）**：`oblique -90deg↔90deg`
对 at(0.5) 期望 [normal]（0deg 规范等价 normal）、at(±外推) 期望钳 ±90deg——
一轮臂缺钳制与 0deg 串化。二轮修复：lerp 后 `clamp(-90.0, 90.0)` + v==0 串化
`normal`；钉测补三形（-90↔90 at 0.5/1.5/-2.0）。二轮复验见文末。

css-transitions NEW-keys 伪形注记：properties-value-{,inherit-001,inherit-002}
三页 b0 各仅 1 key（页内枚举中途夭折形）、b1 各 560 key 全 NEW-Fail——b0 页
夭折伪形非回归（GONE=0 零 Pass 损失），net 记 +2（all-interpolates 页 F2P）。

## 常驻钉

`crates/engine/src/animation.rs` `test_knife_fi_*` 五只：keyframe 展开槽位、
stretch 关键字↔% 插值、style 端点/离散/角域门、line-height px 域 + normal 离散、
apply 写回 gCS round-trip。

## 二轮复验（11:5x，clamp 修复后 b1 重腿 css-fonts）

钉测 `knife_fi` 5 passed（含角域门三新形）。b1r2 release runner 重构建
（md5 `879e7ae3…`≠一轮 `d0d532df…`）——provenance 注记：r2 构建扫入 break 流
在飞 WIP（parse_layout.rs/parse.rs/apply_advanced.rs，break-inside avoid-region
文法，已编译绿），故 r2 账为 **P2F 清除旁证形**；一轮双 bin 账（面洁净）为
canonical ledger。

- css-fonts b1r2：P=6288（vs b0 5744 = **+544**），F2P=542、**P2F=0**——
  一轮 4 P2F（角域门）全清偿并转 F2P（font-style-interpolation 分页
  +86→+96）。红线解除。
- css-transitions 一轮账维持 canonical（+2，F2P=2、P2F=0）。

两轮合计对账：css-fonts +530（一轮 canonical）/ +544（二轮清除形），
css-transitions +2，零 P2F。
