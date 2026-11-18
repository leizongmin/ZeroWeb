# 刀 A3r：text-shadow 残面——keyframe 端点 unset 级联向 + currentcolor 预解析

**2026-11-18 · 认领 +9（css-transitions CSSA 族 unset×5 + currentcolor×4）· 零 P2F · 全语料位移 +8（窗噪 −1 归 fv1 域既持档案）**

## 根因与码面

A3 收口后残面 30F 三簇（见 knife-a3-text-shadow.md 残面池）：A) `unset`
端点未按级联向解析 10F；B) currentcolor 端点无元素色上下文 10F；C)
Transitions 关键字行时钟量化 12F（归 knife-q2/A3b 域，本刀不触）。本刀收
A/B 两簇中 CSS Animations 通道的 9 行（其余为 WAAPI 通道 underlying=None
不解析，归 FIXME(kf-inherit-waapi) 域不动）。

码面单文件 `crates/engine/src/animation.rs`：

1. **unset 级联向**（css-cascade-4 §default：`unset` = inherited 属性取
   inherit、其余取 initial）：keyframe 端点预处理循环按
   `PropertyRegistry::is_inherited` 分流——修正刀 K1「unset 恒 initial」对
   继承属性解析错基值。非继承属性（background-position 等）行为与 K1
   逐位一致。
2. **currentcolor 预解析**（css-color-4 §5.4：currentcolor = color 属性
   computed value）：新增 `serialize_text_shadow_resolved`——值含
   currentcolor 的 text-shadow 列表端点在 keyframe 装载时以 underlying
   元素色定死（插值臂纯字符串无元素上下文）。**属性门 `name ==
   "text-shadow"`**：`parse_text_shadow` 对裸长度列表默认 CurrentColor
   色（css-backgrounds §6.1 阴影色缺省语义），无门会把 background/
   mask-position、border/mask-image outset/width 的 "Npx Npx" 端点误判
   重写（见下 r2 回归）。
3. 钉测三枚：`test_keyframe_unset_inherited_resolves_parent`（unset→父值
   lerp，orange→green t=0.25 = rgb(191,156,0) 27.5/12.5/27.5px）、
   `test_keyframe_currentcolor_resolves_element_color`（currentcolor→元素
   色 lerp，black→green t=0.5 = rgb(0,64,0)）、
   `test_keyframe_non_text_shadow_untouched_by_currentcolor_resolver`
   （回归钉：background-position "40px 140px"→"80px 80px" 端点不被触碰，
   at(0.5) = 60.00px 110.00px 正常插值）。3 passed/4184 filtered。

## A/B 电池（r3 终账，bin-knife-a3r，八腿四语料）

双建沿 knife-a3 patch 往返法证（face=animation.rs 单文件；fv1 WIP +
knife-q2 WIP 双侧同态相消；M2 往返精确还原 / S2 他流在飞面窗内零扰动，
`knife-a3r-build2-*.log`）。逐语料行级账（`knife-a3r-ab-account.py`）：

| 语料 | b0 | b1 | ΔP | 迁移 |
|---|---|---|---|---|
| css-transitions | 814P/700F/17T (1531) | 823P/2369F/20T (3212) | **+9** | F→P×9 全 [A3R-CLAIM]：unset×5 + currentcolor×4（text-shadow-interpolation.html CSS Animations: 族）；零 P→F |
| css-animations | 516P/779F/7T | 516P/780F/7T | +0 | SET-IDENTICAL（1 行 Timeout↔Fail pop-variance） |
| css-backgrounds | 3491P/2755F | 3490P/2756F | −1 | P→F×1 = background-position-y 'calc(10px - 0.5em)…' 行——A3 先例既持窗噪（同 bin 跨跑双向漂移 5 观测 2P/3F，fv1 serialize 域）；本轮 b1 bin solo 复测 Fail 同签名，非本刀面 |
| css-masking | 4703P/1652F | 4703P/1652F | +0 | SET-IDENTICAL |

- transitions b1 侧行数 1531→3212：properties-value-001 页 pv001 双稳
  b1 侧满展开（1680 全 Fail 行，ΔP=0 免疫；A3 收口段已建档该双稳）。
- WAAPI 通道 unset/currentcolor 9F 不动（underlying=None 不解析，
  FIXME(kf-inherit-waapi)）；Transitions 族关键字行归 knife-q2/A3b。

## r2 回归记录（已修复，不上账）

r2 八腿（属性门缺席版 bin）暴露 **P→F×110**（backgrounds −58 + masking
−52）：裸长度列表端点被 currentcolor 预解析臂误改写为阴影串 → applier
拒收 → 动画失效 → gCS 透出 underlying。根因 = parse_text_shadow 裸长度
默认 CurrentColor；钉测三正例全绿仍漏跨属性污染，八腿普查兜住。修复 =
属性门 + 回归钉第三枚；masking −52→+0、backgrounds −58→−1 全愈。r2 的
+9 不预支，认领以 r3 为准。

## 门禁

- 钉测 3 passed/4184 filtered（`knife-a3r-pintest-5-*.log`）
- clippy `-p zero-engine --lib` rc=0 零告警（`clippy-a3r-2-*.log`；
  `-A too_many_arguments` fv1 流清偿 + `-A unnecessary_cast`
  2d3e71d19 流清偿，法证注记沿 A3 不扩常免）
- rustfmt：本刀 hunks 零漂移（animation.rs 既有漂移经 knife-q2 流
  12:02 全文件 fmt 清偿，非本刀操作）

## 残面池更新

- WAAPI 通道 unset/currentcolor ~10F：FIXME(kf-inherit-waapi)（父链/
  元素上下文接入另行开刀）
- Transitions 关键字行时钟量化 12F：knife-q2/A3b 域（其 claim 注记互认
  行级分域，见 NOTES-knife-a3-text-shadow.txt 12:1x 段）
- composite:add accumulation 池 55F+5F：既持不动
