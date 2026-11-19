# 刀 ATP-R2：registered-property 计算值通道 + animation-range 简写文法门（2026-11-20 收账）

## 刀域与 face

css-properties-values-api 主靶（registered-property computation 簇 ~84F：computation
`<length>` computed 空值 / initial calc 折叠与 canonical / inheritance invalid→initial
语义 / unit-cycles em-ex-ch cycle fallback / var-ref 族）+ css-animations
animation-range 简写文法门（decl_supported `animation-range` 单点）。

face 5 文件 2 crate（knife-atpr2-face.patch，22:12 重生成 strip-check 过）：

1. `registered_properties.rs`：RegCtx font/lh ctx、`computed_length_atom` 真视口
   折算、calc 共享 computed 通道（`calc_computed_to_css`）、integer ties-up 舍入、
   transform 参数级串化、`substitution_view`（注册属性以计算值参与 var() 替换 +
   未声明注入 initial 计算形）、computed_value 新签名（HashMap+RegCtx）。
2. `js_dom_bridge.rs`：decl_supported animation-range 简写文法门
   （parse_animation_range_shorthand 单点，position-area 先例同型）。
3. `parse_animation_range.rs`：parse_animation_range_shorthand 简写解析器 +
   serialize start 侧 canonical。
4. `computed_style.rs`：RegCtx ctx 构造 + computed_value 新签名调用侧 +
   substitution_view 消费侧（未注册 `--x` 消费 `var(--reg)` 亦读计算值）+
   line_height_px_of（`10lh`@line-height:20px → 200px 锚定）。
5. `part145_atp_recovery_pins.rs`：computed_value 新签名钉测适配。

规范锚：css-properties-values-api-1 §computation-of-computed-values、§animation；
css-variables-1 使用点解析；css-animations-1 animation-range。

## 机制要点

- 替换视图：注册名在 var() 替换期以计算值参与（已声明无 var → 计算值覆盖；
  已声明计算失败 → 移除即 guaranteed-invalid，consumer fallback 生效；未声明 →
  注入 initial 计算形）。注册表空 → 原样克隆（零注册面零额外行为）。
- computed_value 兜底序：无声明值 / 替换失败 → initial 计算形；替换成功不合
  syntax 且声明含 var → initial（invalid at computed-value time）；直 DECL 不合
  syntax → guaranteed-invalid `''`。
- lh 原子：Normal UA 近似 font×1.2 FIXME(regprop-lh-normal)（corpus 目标面仅
  显式长度行高锚定）；Number 按 line_height_str 同口径折 px。

## 首发 A/B 电池（knife-atpr2-battery，11-19 23:52:51–00:24:21 GATE-DONE rc=0）

RIDE 23 文件（S1/S3 md5 census 零扰动，ride-files.txt 存档）；b0 md5
`0101bb8e`，b1 md5 `ec826b2f`（M2 异，face 入二进制双证）；build 49.97s/49.82s。
六语料 {pva, animations, fonts, shapes, typed-om, transitions} × {b0, b1}。

首发 reconcile：gained=79 lost=2 regressed=2 pages_gain=14 pages_loss=2。

## 首发账污染法证（fonts +12 剔除，不入本刀账）

fonts 侧 +12（ffs/fvs computed +6/+3、serialization 三页 +1×3、inheritance +1）
与刀 echo（`7d119d8ef`，已先行落库）逐行同形。定谳：atpr2-face.patch 于 22:12
重生成，早于 echo 落库（22:4x）——patch 的 computed_style.rs hunks 把 echo 未
提交改动一并烤入（patch 内 line 151/184/233 含 `settings_computed_join` 码）。
b0 = `git apply -R` 剥 face 时连 echo 码一并剥除 → b0 丢失 echo 行为 → b0→b1
fonts diff = echo 开关伪影，非 atpr2 效应。S1/S3 census 不覆盖 face 文件故未
拦截。教训：face patch 生成基必须晚于全部在飞落库，发射前重生成复核。
b1 树态（=提交态）echo 码完好（trap TRAP-NOOP state=done）。

## P2F 根修（canonicalize 列表切分 token 感知单源）

首发 2 页 P2F：`animation/custom-property-animation-color-{comma,space}-list.html`
各 P 1→0，败形 = 页内全部取样读值 `''`（b0 为错插值/underlying 值非空）。

根因：lenient 计算面 `canonicalize_component` 列表切分 naive
`split(',')`/`split_whitespace()`——`<color>#` `rgb(255, 0, 0), rgb(0, 255, 0)`
函数内逗号/空白不可切 → 列表项碎裂 → canonicalize None。strict 注册门
`strict_component` 用 token 感知 `split_value_items`（括号深度+引号跳越）故注册
面无此病。b0 读路径无声明值时回 initial_computed 原文不经 lenient 重串化，故
discrete 测试虚过；atpr2 新 computed_value 把 initial 兜底也路由 lenient 串化
→ `''` 全页塌陷。

修复：`canonicalize_component` 两列表臂改走 `split_value_items`（与注册门单源，
css-properties-values-api-1 §animation 列表分项语义）。

## 根修验证（b1fix = b1 树态 + 修复，md5 `dc201df4`）

pva 单语料（r2 首验 + r3 重跑同 md5）：

- b1fix vs b1：2 P2F 页恢复 P=1（=b0 位），registered-property-computation
  再 +2（64→66，括号感知切分白拿），**其余页零差异零回归**。
- pva TOTAL：b0=908 → b1=954 → b1fix=958。

## 收账（b0 vs b1fix 全六语料，r3 GATE-DONE）

<!-- FINAL-RECONCILE -->
