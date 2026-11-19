# 刀 ANIMCOMP S2：WAAPI/CSSA 车道引擎合成模式（均匀 add 影族）+ animation-composition apply 面

**状态：电池已收账（S2 账兑现在 transitions 侧；css-animations 池账零位移
——池-兑现面错位，见电池账节）**
台账：`.acceptance/wptfyi-css/NOTES-knife-animcomp-s2.txt`（03:1x 发射 → 多 fork
收账风暴 → 收账道移交分解电池）。本刀面（5 文件 patch
`knife-animcomp-s2-face.patch`）＝crates/engine/src/animation.rs +
animation/compositor_export.rs + js_dom_bridge/cssa_registry.rs +
js_anim_registry.rs + js_dom_shim/part03.js。

## 池与定谳

S1 收口残面（evidence 2026-11-18-knife-animcomp-s1）＝css-animations
composition 族 14F（composition 6 + keyframes 6 + compositeOrder 2）。
S1 写面门只开了 CSS 属性通道；WAAPI `el.animate(keyframes, {composite})`
车道的 add/accumulate 合成面由本刀承接。

## 机制

- **引擎合成（本刀主体）**：均匀 add 影族门——逐属性 op 分类后，全部
  add/accumulate 属性 ∈ {text-shadow, box-shadow} 且各自全帧同 op（同一
  op 串）→ keyframes 原样登记（spec.comp 动画级 op + spec.und shim gCS
  预读 underlying），合成在引擎 start/tick 期做。影列表追加＝字符串级
  （web-animations-1 §5.4.4 add 语义对 shadow 列表＝拼接，无多影端点插值
  依赖）；accumulate 同形去重拼接。vertical-align 不入门（WAAPI px 加法
  预合成既有通道已覆盖；px↔% 端点对维持拒登）。
- **回落路径**：任一 add 属性出影族或 op 不均匀 → 原
  `_zwComposeAddKeyframes` shim 预合成路径（混合 op 影族拼接臂），行为
  不回退。
- **Rust 面**：`compose_animation_value`（css-animations-2
  §animation-composition / web-animations-1 §5.4.4——属性有效 op ＝ 帧内
  animation-composition ?? 动画级 config.composition ?? replace；恒等回落
  动画值）+ `compose_tick_props`；帧级 `composition:
  Option<AnimationCompositionValue>`（关键帧内声明对该帧值生效，None＝
  回落动画级，再回落 replace——css-animations-2 §animation-composition）
  + 动画级 `config.composition`（默认 Replace）；序列化形态锚定 WPT
  composition 期望串（逗号 + 空格分隔）。
- **账面归因协议**：分解电池 knife-decomp（b0/b0s/b1 三二进制）——
  css-animations b0s−b0 ＝ S2 账；css-transitions b0s−b0 ＝ S2 哨兵
  （预期零位移）。S2 独立电池（zw-knife-animcomp-s2-battery-0312）窗内
  被并发流污染（transition_registry/part04 窗内编辑＝后来 TB face；
  截断重试、多 fork collector 风暴，见台账 INCIDENT 节）——其 verdict
  **不作账**，仅存 provenance。

## 门禁与钉测

本刀无专项单测钉（引擎合成语义由分解电池单变量腿实证——b0s−b0 F2P=54
零 P2F 为行为权威）；engine lib 全绿（4222 passed，含 animation.rs 内联
multicol columns 钉与 js_dom_bridge_tests 全量）。S2 face hunks 随
`knife-decomp-full.patch` 进电池 b1 构建（bin md5 `4ca80d11`，M2 异同门过）。

## 电池账（results-knife-decomp，2026-11-19 09:40 GATE-DONE）

分解电池单变量腿 b0s−b0（唯一差异 = S2 face 5 文件；bin b0s
`a473acbd` 介于 b0/b1 之间，S2 单变量入二进制 M2 过）：

- **css-transitions b0s−b0：F2P=54 P2F=0 net=+54**——页级
  text-shadow-composition.html **+40** + vertical-align-composition.html
  **+14**，零回归。均匀 add 影族门属性集（text-shadow/box-shadow）与
  transitions composition 页型完全匹配：引擎合成接管后逐采样点断言全绿。
- **css-animations b0s−b0：F2P=0 P2F=0 net=0**——目标池（composition 族
  14F：composition 6 + keyframes 6 + compositeOrder 2）零位移。池-兑现面
  错位定谳：css-animations composition 页的 add 影族属性集出影族门
  （非 text/box-shadow 单属性族或 op 不均匀）→ 回落 shim 预合成路径
  行为不变。目标池不因本刀清偿，挂残面池（见下）。
- **css-animations b1−b0s / css-transitions b1−b0s 哨兵**：S2 账不被后续
  face 扰动（b1−b0s 差异归 TB+TS-RB，账在 TB evidence）。

VERDICT：引擎合成模式兑现（transitions composition 面净 +54 零回归），
池账错位如实挂账；P2F 红线全零。

## 残面

- css-animations composition 族 14F（池原样）：影族门属性集扩容
  （transform/color/margin 等均匀 add 形）或逐属性影族分区合成为下一刀
  候选；回落 shim 预合成路径对混合 op 影族的既有红不因本刀变化。
- vertical-align-composition.html +14 的兑现通道为 part03.js hunk 面
  （shim 侧 compose 路径），机制注记与引擎影族门并行；如需 px↔% 端点对
  WAAPI 登记另立刀面。
