# 刀 ATP-R2：registered-property 计算值通道 + animation-range 简写文法门（2026-11-20 收账）

## 刀域与归因

css-properties-values-api registered-property computation 簇（residue survey 批二认领，
17:0x 窗）+ css-animations animation-range 简写文法门。收账 = knife-atpr2 电池
（23:52:51 发射 00:24:21 GATE-DONE）b0/b1 + P2F 根修后 knife-atpr2-fix-verify r2/r3
补腿重跑（b0 vs b1fix 全六语料，02:44:28 GATE-DONE rc=0）。

## 码面（5 文件 570+/82−，2 crate）

- `registered_properties.rs`：RegCtx font/lh 上下文、computed_length_atom 真视口
  折算、calc 共享 computed 通道（calc_computed_to_css）、integer ties-up 舍入、
  transform 参数级串化（canonicalize_transform_value）、var() substitution_view
  （注册属性以计算值参与替换 + 未声明注入 initial 计算形 + guaranteed-invalid
  移除）、computed_value 新签名（HashMap + RegCtx）与 initial 兜底 canonical 化、
  registered_names 双通道合并清单。
- `js_dom_bridge.rs`：decl_supported animation-range 简写文法门
  （parse_animation_range_shorthand 单点，position-area 先例同型）。
- `parse_animation_range.rs`：parse_animation_range_shorthand 简写解析器
  （css-animations-2 §animation-range）。
- `computed_style.rs`：RegCtx 构造 + computed_value 新签名调用侧 +
  substitution_view 消费侧 + line_height_px_of（`10lh`@line-height:20px→200px
  锚定；Normal UA 近似 font×1.2 FIXME(regprop-lh-normal)）。
- `part145_atp_recovery_pins.rs`：computed_value 新签名钉测适配
  （test_atp_note_stylesheets_registry_pins 守卫通过 1 passed）。

## P2F 根修（canonicalize_component 列表切分单源化）

首发电池 b1 出现 2 页 P2F（css-properties-values-api/animation/
custom-property-animation-color-{comma,space}-list.html，各 P 1→0，全部采样
回读空串）。法证：注册门 strict 走 token 感知 `split_value_items`（括号/引号
深度感知），计算门 canonicalize_component 列表形用 naive `split(',')`/
`split_whitespace()`——`<color>#` 值 `rgb(255, 0, 0), rgb(0, 255, 0)` 函数内
逗号被误切 6 段逐段解析全败；b0 旧读路径无声明时直返 initial_computed 原文
不经 canonicalize，故 b0 离散测试虚过（1P 恰为 initial 回显）；b1 新读路径
initial 兜底亦过 canonicalize → 全页空串。修复 = 计算门列表切分改用
`split_value_items`（与注册门单源）；修复后两页回 P=1（≥b0），且
registered-property-computation 再 +2（64→66，声明值列表形串化解锁）。

## 电池账（b0 vs b1fix 全六语料，gained=81 lost=0 regressed=0 pages_gain=14）

- css-properties-values-api **+50**（7 页）：registered-property-computation
  43→66（+23）、registered-property-initial 10→24（+14）、font-property-
  unit-cycles 25→30（+5）、var-reference-registered-properties 9→14（+5）、
  var-reference-002 0→1、registered-properties-inheritance 0→1、
  invalid-at-computed-value-time 0→1。
- css-animations **+18**：parsing/animation-range-shorthand 80→98（简写文法门
  兑现；残面=animation-range 族 entry/exit/calc 形独立面）。
- **哨兵**：css-shapes 6026/6026/6026、css-typed-om 11723 三态恒零差、
  css-transitions reconcile 零非哨兵页——逐页零差异。
- **echo 切换伪账剔除**：fonts 6511→6524（+13 六页）与刀 echo 收账账目逐页
  精确相同——法证 = knife-atpr2-face.patch（22:12 生成）早于 echo 落库
  （7d119d8ef 22:4x），其 computed_style.rs hunks 意外捎带 echo 改动，b0 剥除
  时连带剥除已提交 echo 码 = b0 侧 fonts 假低 13。b1/b1fix 同含 echo 恒等
  （6524==6524）。该 +13 不入 ATP-R2 账；patch 文件作废标记（后续 face 再生成
  须以落库后 HEAD 为基）。

## 会话与仲裁

- 首发电池（knife-atpr2-battery-2352）23:52 发射 00:24:21 GATE-DONE rc=0，
  会话 429 断点未收账。
- P2F 根修验证 r2（本 fork）：pva/animations/fonts 三腿完整落盘后 会话重启截断
  （shapes 0-byte 伪账、transitions 半途）；r3（兄弟 fork 补腿脚本，
  zw-atpr2fix-r3.service 02:33:25 发射）shapes/typed-om/transitions 三腿补齐 +
  全六语料对账收口。r2 顶链 + 孤儿 transitions runner 击落，r3 链零扰动；
  双方共用 b1fix-runner md5 `dc201df4`，无二进制歧义。仲裁注记见
  `.acceptance/wptfyi-css/NOTES-knife-atpr2-r2r3-arbitration.txt`。
- fmt 门：computed_style.rs import 块 rustfmt 补齐（行为零差，收账 bin 先于
  fmt 构建，语义恒等）。

## 残面与让渡

- pva 残（电池预口径）：registered-property-computation 残 ~9F、initial 残
  ~5F、unit-cycles 残 ~18F（calc trig 数学面=css-parser 位语热修流邻接）、
  inheritance 残 ~7F、var-ref-cycles 残 ~5F——后续小刀。
- animation-range 族残 170F（entry/exit/calc 形 selector+shape 新文法）入
  residue survey 候选池。
- 累计账：本轮局部 +6761 → +6829（+68 全额入账）。
