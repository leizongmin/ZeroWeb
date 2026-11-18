# 刀 A3：text-shadow 阴影列表插值接入三通道（2026-11-18）

## 靶面

css-transitions 腿 text-shadow-interpolation.html（168 子测，A2 后基线
16P/152F）+ text-shadow-composition.html（60 子测，基线 0P/60F）。插值页
全部为可插值对形（无 no-interp 关键字对形），composition 页 55F 归
composite add/accumulate 累积池（本刀不认领）。上游语义
css-text-decor-3 §text-shadow-property：interpolated as list of shadows，
层数失配零阴影补齐（none ≡ 空表）；颜色预乘空间 lerp（css-color-4 §15.1）。

## 机制（engine 2 文件，css-parser 零编辑）

**宽松臂**（animation.rs interpolate_property_value，mask-border-repeat/
source 臂后）：双端点 `parse_text_shadow_list`（css-parser 现成解析器，
none→Some(vec![])、缺 blur 补 Px(0)）→ 层数 `max(fl, tl)` 零阴影补齐 →
逐层 (Rgba, ox, oy, blur) 四元 lerp：颜色走既有 `lerp_color` 预乘空间
（a≤1e-6 通道归零）、blur 非负域钳 0（offset 不钳）、em/rem ×16 归 px →
串化镜像 computed_style `text_shadow_to_css` 契约：color-first、恒显
blur、`format_num` 千分位、a=1 rgb 形否则 rgba 两位小数。

**严格门**（interpolate_property_value_strict，va 臂后）：双端点可解析
Some 委托宽松臂，否则 None——transitions 通道关键字/非列表对不启动
过渡（css-transitions-1 §starting-of-transitions，同 va 臂形）。

**applier 臂**（apply_single_property）：parse → 逐层
resolve_effect_length 穿线写 `style.text_shadow`。canonicalize 链共因
（js_anim_registry.rs:391 apply→串化往返 ser1≠ser0）第四次验证——缺
臂则 CSSA/WAAPI/transitions 三通道全静态回落。

**登记 +1**：TRANSITIONABLE_PROPERTIES 81→82，"text-shadow" 插
"text-indent" 后（刀 A3 注记）。

**钉测**：test_text_shadow_lerp 8 组断言——同型 lerp（rgb 形双不透明
端点）、none 补齐中值/at0 透明黑、blur 负外推钳 0（offset 不钳）、多层
list 失配补齐、离散翻转、strict Some（rgba 0.50 中值）/None（"10px"→
"red" 非列表对）。首跑 0 passed/4177 filtered 教训：漏 `#[test]` 属性
的 fn 不入测试（恰集门照 #39 核 4178 总数）。

## 门禁

- fmt：animation.rs 17 处漂移中 4 处落本刀 hunks（932 折行/1528 守卫
  分行/3753 applier 三参折行/7189 assert 折行）按 rustfmt 建议形修复；
  余 13 处全既有 HEAD 漂移（690/1369-1491/6991-7058 区，不与本刀相
  交，A2 账面同源不代修）。registry 2 处（873/917）既有测试码。
- clippy：定向 `cargo clippy -p zero-engine --lib` rc=0 零告警（两个
  -A 各带域外法证，非常免）：`too_many_arguments` = script-sandbox
  es_module.rs:2211 transform_import 8/7，对端 fv1 WIP 引入（HEAD 7
  参，WIP 加 `defs: &mut String`），该文件在避让清单，归 fv1 流清偿；
  `unnecessary_cast` = animation.rs:2434/2435 提交态域外（HEAD
  2345/2346，provenance 2d3e71d19 mask 刀 N 收敛轮，今晨 rebase 带入
  晚于 A2 clippy 绿窗），归该流清偿。本刀自有同 lint 两处（px 闭包
  921/922 no-op cast）已当场修复并补跑钉测绿。
- 钉测：1 passed / 0 failed / 4177 filtered（PIN-LEG-EXIT rc=0）。

## A/B 定谳（双建同窗，fv1 WIP 双侧同态）

刀 A3 双建 knife-a3-build-bins.sh：patch 往返换面（不碰 stash 栈——栈
上他流 4 条在途），trap EXIT 崩溃自恢复，M2/S1=S2 法证（A3 两文件
md5 往返精确还原、fv1 WIP 11 文件窗内零扰动）。

- b0 = fv1 WIP + HEAD（无 A3）＝ md5 80fe1329f2a17f276d99cdd617c36a49
- b1 = 同窗 + A3 四件套 ＝ md5 50d0142f25e1e2301a898c2b8523bde7

**solo 探针**（双 bin × 双页，页集合双侧恰等、恰集计数非空）：

| 页 | b0 | b1 | Δ |
|---|---|---|---|
| text-shadow-interpolation | 16P/152F | 138P/30F | +122 F→P 零 P→F |
| text-shadow-composition | 0P/60F | 5P/55F | +5（replace→add 塌缩形） |

**population unlock（css-transitions 语料，归因本刀、ΔP=0 不计 P）**：
properties-value-001/-inherit-002 两页 b0 只报页级 **Fail** 1 条（异步链
挂死套件早夭）；b1 页级 Timeout（套件跑满墙钟，子测照常流式落账）——
properties.js:214 表含 `'text-shadow': ['shadow']`，generalParallelTest
挂 transitionend 监听（:59），b0 缺 text-shadow 过渡 ⇒ 事件永不触发 ⇒
异步链整页挂死；b1 过渡真启动 ⇒ 全枚举展开 +560×2=1120 子测（0P 全 F，
properties-value 池旧估 260F 被实测取代）。净 ΔP 不变（+127 全落两
text-shadow 页），新人口全 F 只增分母。

**八腿电池**（results-knife-a3，r1 包装被杀仅 css-transitions b0 完整
收档含 rc=1 DONE 标记行、r2 续跑七腿 11:10→11:25 GATE-DONE 零
LEG-FAIL）：

| 语料 | b0 | b1 | ΔP | F→P | P→F |
|---|---|---|---|---|---|
| css-transitions | 673P/1393F/16T | 800P/2385F/18T | +127 | +127 | 0 |
| css-animations | 516P/780F/7T | 同 | 0 | 0 | 0 |
| css-backgrounds | 3488P/2758F | 3489P/2757F | +1 | +1 | 0 |
| css-masking | 4703P/1652F | 同 | 0 | 0 | 0 |
| **TOTAL** | 9380P | 9508P | **+128** | +128 | **0** |

**认领口径 +127**（插值页 +122 + composition 页 +5，页级小计双行见
knife-a3-account-1125.txt）。css-backgrounds +1 ＝ background-computed.html
"Property background-position-y value 'calc(10px - 0.5em), -20%, bottom,
15%'"——与 A2 的 +1（同页 bg-position-x calc 多层串化）同页族，fv1
serialize.rs 域固有非确定性漂移，两 bin 共含其 WIP 代码，归其系不认领
（A2 先例同判，第二次出现注记）。窗噪定性实证：b0 solo 复测该子测
Pass（solo.bgcomputed.b0.json）而 battery b0 Fail/b1 Pass——跨跑双向
漂移非刀面效应。animations/masking 双腿 SET-IDENTICAL
（页集合+子测键全等）。

## 残面池（不认领，挂观测）

1. **关键字端点未解析计算值**（插值页 ~24F）：neutral/inherit/unset
   端点以字面串进臂 → parse 失败离散翻转；期望端点解析为元素当前
   computed 值再 lerp。canonicalize 往返家族 gap，后续刀点。
2. **currentColor 缺 Animations/WAAPI 元素色上下文**（~6F）：transitions
   通道因 registry gCS 预串化已过；CSSA/WAAPI 通道 keyframe 串进臂，
   `resolve_color_current` 落黑底。同 rule-color 臂契约边界。
3. **composite add/accumulate 累积引擎**（composition 页 55F）：
   `underlying [...] from add to add` 期望多层列表累积——composite 池
   既有挂账，本刀不扩。WAAPI replace→add 塌缩形 5F 同池注记
   （CSS Animations 同形已塌缩过 → +5）。
