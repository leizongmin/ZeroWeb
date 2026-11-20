# 刀 RC · clip-path <position> calc 词法根修 + Calc 同构插值臂（2026-11-18）

## 认领与范围

- 认领：knife R core/R2 union 电池 v1 shapes 腿 P2F=4 定谳后续（仲裁
  NOTES 21:4x 节：`parse_position_pair` 朴素 `split_whitespace` 撕裂
  calc() 内空白 → 4 token 撞 4 值臂 `kw("+")` 拒收）。
- 两段面：①css-parser 词法根修（`split_top_ws_depth` pub(crate) 复用 +
  `parse_position_pair` 换用 + 钉测）；②engine Calc 同构插值臂
  （`clip_interp.rs` `lerp_len` Calc↔Calc arm，css-values-4 §8.5.6）。
- calc −4 归因原由 R2 CLAIM 流移交本流收口（CLAIM-knife-r2 文件 21:57
  让位条款）。

## ① 词法根修（css-parser）

- `parse_basic_shape.rs` `split_top_ws_depth` → `pub(crate)`；
  `parse_extended_border.rs` `parse_position_pair` 改用括号深度感知切分
  （<position> 分量词法同 function-token 域，css-syntax-3）。
- 钉测 `test_clip_path_position_calc_token_boundary`：4 形 calc 缘偏移
  circle/ellipse 2 值/4 值接受 + invalidPositions 契约（`top 50%`/
  `bottom 70%`）拒收保持。
- 验证：css-parser lib 3311 passed 0 failed；钉测首执独立腿 1 passed
  （3310 filtered）。

## ② Calc 同构插值臂（engine）

- 机制链：`animation.rs` `clip_pair_echo_preserve` 门依赖「期望串含 calc
  时写面拒收 → expected 落回静态值」的 B0 伪通过池——词法修复解除写面
  拒收后，12 例伪通过翻真失败（v2 中间账 P2F=12：masking/shapes 各 6，
  全为 `circle(25% at right 5% bottom 15px)→circle(45% at right 25%
  bottom 35px)` 4 值缘偏移对 CSSA/WAAPI 通道）。
- 臂：`lerp_calc_same_shape`——`<percentage> <op> <length-percentage>`
  同 op 同构形逐系数线性（px 系数符号折叠、px 系数零→纯 % 简化、纯 %
  off 对整树折单百分比、异构 None 离散）。同构对走真插值不再进 echo 门；
  echo 门仅余真不可插值对（rect 族跨形/单端不可解析）语义不变。
- `test_clip_pair_echo_preserve_gate` 首断言随伪通过池兑现翻转（原断言
  `""` → 真插值 `circle(35% at 85% calc(100% - 25px))`）。
- 钉测 ×2 新增（语料主形态三 at 断言 + 端点简化/异构拒绝）；engine
  clip 面 132 passed 0 failed。

## 法证与电池账

- **bin md5 法证**：修复回退 bisect bin（target-v2b1 独立重建）md5
  `38d4b216…` ≡ v1 b1 bin——TD1 面在 v1（21:28）/v2（21:52）构建窗间
  零漂移（确定性构建双证），v1→v2 唯一变量＝css-parser 修复。12 P2F
  归因排除 TD1/ride-along/R2。
- v2 中间账（b1=b3083474，calc 修复无插值臂）：masking F2P=74 P2F=6、
  shapes F2P=61 P2F=6、transitions +25 P2F=0 LOST=2（页级 Timeout 截断
  伪影）、animations 0/0 LOST=2、backgrounds F2P=5 P2F=0（bg-position-y
  窗噪第 6 例未复现＝flake 定谳）。
- b2 solo 探针（md5 7a05629f）：masking clip-path-interpolation-001
  358P（v1 352P→+6，transitions 通道 6 例新增真 F2P）、shapes
  shape-outside-interpolation 380P（v1 374P→+6）——12 例伪通过全数
  兑现 + transitions 通道 12 例新增真增益。
- v3 终账（五语料 b1=7a05629f vs v1 b0，23:00:35 V3-GATE-DONE）：
  masking F2P=80 P2F=0、shapes F2P=67 P2F=0、transitions F2P=40 P2F=0
  LOST=1（animate-with-color-mix 截断伪影，子测缺席型）、animations
  0/0、backgrounds F2P=5 P2F=1（bg-position-y calc 窗噪第七例同签名
  不入刀账）。合计 F2P=192 P2F=1＝净 +191；**TP 刀面转嫁 +15**
  （transition-property-invalid，TP 电池窗 apply.rs WIP ride-along）
  归 TP 流账。逐腿程序化核账（r1 JSONs 直比）：本刀 RC 净 **+21**＝
  masking +6（clip-path-interpolation-001 transitions 通道 6 例）
  + shapes +15（circle-ellipse calc 4 形 P2F 修复回绿 + shape-outside
  -interpolation transitions 6 例 + 词法修复新开页 ellipse-011 ×2/
  circle-011/computed/valid-position 各 1）＝ vs union r1 净 +155 →
  +191−15TP−155＝+21。
- backgrounds P2F=1 独立核名定谳 flake（v3 b1 vs b0 程序化直比）：
  `parsing/background-computed.html` 子测 `background-position-y value
  'calc(10px - 0.5em), -20%, bottom, 15%'`，got `13px, 6px, -10px, 8px`
  ＝前兄弟子测残留值（computed 缓存状态泄漏，非当前值错算）。因果排除：
  v1 无修复同子测同签名 1 例、v2 含 calc 修复未复现——若修复因果必在
  v2 复现。判窗噪不入刀账（v1/v3 各 1，对净额零增减）。

## 残面挂账

- 2 值臂 `(x-edge, lp)` 轴指派 FIXME（`right 10px` 存 (100%, 10px) 非
  缘偏移 resolve）——语料 invalidPositions 契约计分域外，后续刀。
- px↔% 跨单位 off Calc 对 / 嵌套 calc 树插值——None 离散，真不可达对
  由 echo 门保位。
