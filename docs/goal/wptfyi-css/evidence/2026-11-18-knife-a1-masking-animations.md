# 刀 A1：mask 长手族动画 applier 臂 + transitionable 清单补齐（2026-11-18 凌晨）

## 靶面

css-masking animations 域（knifen2 收口后 b0=results-knifen2-b1，bin md5
`8833233e` 在册产物定账）：

| 页 | b0 形态 |
|---|---|
| mask-position-interpolation | 10P/196T（186F） |
| mask-image-interpolation | 240P/408T（168F） |
| mask-border-*-composition ×3 | 93F×3 页全灭 |
| clip-path 各插值页 | shape/001/002/geometry-box/xywh/composition 数百 F |

## 根因（双通道同病）

**① applier 缺臂**（WAAPI/CSS-Animations 通道）：`js_anim_registry`
canonicalize 把插值 raw 施加到基样式 clone 后过 gCS 序列化器
（`AnimationClock::apply_to_computed_style` → `apply_single_property`），
该函数缺 mask 长手族臂 → 写值零生效 → `ser1 ≡ ser0` 恒 None 回落基值
——动画覆盖咨询接通但永不采纳。与 clip-path 臂（2026-11-16）同病同治。

**② transitionable 清单缺项**（CSS-Transitions 通道）：
`transition_registry::TRANSITIONABLE_PROPERTIES`（清单 ≡ strict 臂全集
不变量，刀 MI strict 臂已挂 mask 八长手）缺 8 项 → 显式 transition
声明与 `transition: all` 通配双形均不启动。

**③ 伴生（本轮新增定谳）**：px↔px 插值空串之谜。`mask_pos_fmt` 对
纯 px 对（两端点 length）也发 `calc(0.00% + 15.00px)` 系数形 →
`apply_advanced` 把含 % 项 calc 存不可解析 `Calc` 轴 →
`bg_position_axis_to_css(Calc)` 返 `""`（% calc 无容器尺寸不可解析）
→ 层串化 `" "` → canonicalize `ser1 " " ≠ ser0` 采纳垃圾。语料期望侧
（期望元素 inline 纯 px）是 `"15px 25px"` → 全块 Fail。而
`initial`/`unset` 块期望 inline 也是 calc 形（同通道双垃圾相等）故假
性通过——修复必须保形该通道：仅两端点均纯 length 时发纯 `Xpx`，
任一端点含 % 项维持 calc 系数形（css-values-4：length↔length 插值
产物仍是 length，不引入 calc）。

## 实现

1. `crates/engine/src/animation.rs` `apply_single_property` mask 八长手
   委托臂（`zero_style_system::property::apply_advanced::
   apply_advanced_property_value`，解析失败回落不动样式）；`MaskPosComp`
   扩三元组 `(pct, px, 是否含 % 项)`——`Percent`/关键字恒含 %，纯
   `Length(Px)` 不含，`EdgeOffset` 右/下缘（100%−off 恒含 %）或含 %
   offset 时含；`mask_pos_fmt` 纯 length 对发 `{:.2}px`。
2. `crates/engine/src/js_dom_bridge/transition_registry.rs`
   `TRANSITIONABLE_PROPERTIES` 65→**72** 补 mask 长手族 **7 项**（clip/
   composite/mode/origin/position/repeat/size）。**mask-image 例外不入
   清单**：首轮 A/B 定谳其 transition 面端点快照跨块陈旧（neutral→url
   取前块 stripes-20、inherit 未解析取 none），入清单把 B0 靠「过渡不
   启动→静态跳终值」通过的 url-everywhere 面反转成 P2F；剔除后增益全走
   WAAPI/CSSA applier 臂通道。
3. **显式声明清单门**（同文件 `register_from_style_change` 显式臂）：首轮
   A/B 后探针定谳「剔除不充分」假说不成立（transition:all 面已回还），
   真根因＝显式 `transition-property: <prop>` 登记臂无清单门（只 `all`
   臂过清单展开）——applier 臂激活 mask 动画面后端点快照环境被扰动
   （neutral→url 取前块值/inherit 取 none），mask-image 显式离散启动
   REGRESSED=30（at<0.5 全灭）。补清单 contains 门后显式面＝静态跳终值
   B0 正形。代价：显式 allow-discrete 离散对面 +27 F2P 暂失
   （transition_behavior 字段 style-system 未建模，挂账 A1b 与字段一起
   接入）。清单不变量式改记 ⊆（strict 臂全集含离散面 mask-image）。

## 验证（探针单页）

staged bin `bin-knife-a1`（四阶段演进，md5 谱系 `95969b20` ①② →
`bb21cf2b` +③ → `ae9458e0` +②′剔除 → `fa17c8dc` +显式门＝终态）：

| 阶段 | mask-position-interpolation | mask-image-interpolation |
|---|---|---|
| b0 | 10P/196T | 240P/408T |
| ①②（calc 形） | 60P/196T | 341P/408T |
| +③ 纯 px 形 | **176P/196T** | 341P/408T（不受③影响） |
| +②′ mask-image 清单剔除 | 176P/196T | 344P/408T（实测 `ae9458e0`；显式臂无门 REGRESSED=30 在册） |
| +显式清单门（终态） | 176P/196T | **347P/408T**（实测 `fa17c8dc`，REGRESSED=0） |

探针页实证：动画覆盖 gCS `" "` → `"15px 25px"`；基样式串化不受影响。

## 残留挂账（本页 20F 分解，A1b 池）

| 族 | 形 | 根因 | 数 |
|---|---|---|---|
| R1 | `initial`/`unset` at(1) ×2 方法、mismatched-layers at(1) ×2 方法 | Transitions 端点钳制采纳 raw 端点原文——% 系数对与 LCM 展开对需 mix 形（t=1 插值产物）采纳 | 6 |
| R2 | `from [inherit]` 全 7 at ×1 方法（WAAPI） | js_anim keyframe 端点 `inherit` 未解析父 computed（刀 K1 只挂 cssa 通道） | 7 |
| R3 | mismatched-layers 全 7 at（WAAPI） | WAAPI keyframe 合成/校验门拒逗号多层值（CSS Animations 同块过 → apply 路径无恙） | 7 |
| R4 | mask-image 显式 `transition-behavior:allow-discrete` 离散对面（27F，B0 同形） | `transition_behavior` 字段 style-system 未建模——显式清单门关闭离散启动（零回归方向），A1b 与字段建模一起接（同面还有 transition-property:all+allow-discrete 27F） | 27+27 |

## A/B 六腿账（v5 + masking gate-leg 合成账）

bin 谱系（resume-swarm 双 fork 收敛， CLAIM 全程仲裁）：

| md5 | 码面 | 产物 |
|---|---|---|
| `8833233e` | b0 在册（HEAD knifen2 收口态） | results-knifen2-b1 |
| `95969b20` | ①applier 臂 + ②73 清单 | 首轮探针 |
| `bb21cf2b` | ①②+③纯 px 形（73 清单） | battery-1（masking REGRESSED=60 定谳②过度） |
| `ae9458e0` | ①②′剔除（72 清单）+③ | run4 全账（mask-image REGRESSED=30 → 显式臂无门假说） |
| `fa17c8dc` | +③′显式声明清单门 | **v5 五腿 + masking gate-leg（终账）** |

终账（b0 vs results-knife-a1-b5 全目录复算 02:58，verify 角色独立复核 +
本节 reconcile 双通道一致）：run5 六腿实际全部落 gate bin `fa17c8dc`
——masking 腿 exec `bin-knife-a1-v5`（02:50:03），余五腿 exec
`bin-knife-a1`（02:52:06 脚本改路径 + 02:51:27 该路径已 cp 终态 bin，
fork verify 补记在册）；b5 masking.json 为 02:57:41 gate-bin 单腿重跑
（原腿 02:50 exec 早于删页带 1 页 stub → 重跑页集 62=62 全等）。

```
TOTAL: gained=1117 lost=0 regressed=0 pages_gain=11 pages_loss=0
masking:  mask-position +166、mask-image +107 ＝ 靶面净 +273
fonts:    +844（system-fonts +192 / font-valid +306 / font-computed +232
          / serialization-prevention +104 等——窗口法证归 knife-n3/n3b
          在飞 WIP，不入本刀账）
gaps/multicol/typed-om/shapes: SET-IDENTICAL（reconcile 零行）
```

零容忍判据：P2F=0 ✓ / lost=0 ✓ / pages_loss=0 ✓ / regressed=0 ✓
（fonts 腿本账内亦零回归——run4 期 font-family_parsing −102 形为 n3
更早 WIP 态，02:46 树态已自愈）。masking corpus 全量逐子测独立复算：
gained=273 lost=0 regressed=0 pages_gain=0 pages_loss=0。

补位复核（02:58）：b5 原位 masking.json 已由 gate-bin（`fa17c8dc`，
bin-knife-a1-v5）独立 leg 重写复原，全六腿 b0-vs-b5 reconcile 重跑
复现 `gained=1117 lost=0 regressed=0 pages_gain=11 pages_loss=0`
（fonts 腿合计 +844：gate 面 +738 归 knife-O 注册门、serialization
面 +106 归 n3/n3b 在飞 WIP——均窗口法证、不入本刀账）。页集含
`zz-probe-mask-longhands.html` 系 b0 基线同在遗留（零位移，非本刀引入）。

## 收尾挂账

- R1–R3（mask-position 页 20F）+ R4（allow-discrete 离散对面 27+27F，
  `transition_behavior` 字段未建模）→ A1b 池。
- clip-path 文法站（geometry-box 144F 全灭）/ basic-shape 插值 /
  mask-border composition additive（93×3F）/ mask 简写 no-interp 42F
  → 原池不变。
