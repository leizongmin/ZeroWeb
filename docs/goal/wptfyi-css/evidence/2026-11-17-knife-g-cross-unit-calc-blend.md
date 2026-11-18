# 刀 G·clip-path 跨单位 calc blend 批（2026-11-17）：lerp_len px↔% 混合臂

## 交付面（1 面 + 1 钉）

1. **跨单位混合臂**（`crates/engine/src/animation/clip_interp.rs`）：`lerp_len`
   原仅同单位两臂（`(Px,Px)`/`(Pct,Pct)`），px↔% 跨单位端点 None → 离散
   切换。新增两臂复用刀 E `blend_rule_inset_lp` 模型（animation.rs:1577）：
   逐单位系数 `(1-t)·from + t·to` → `calc(P% ± Xpx)`，px 系数零折叠纯 %，
   同单位对返 None 守卫恰好兼容（同单位仍走既有线性臂）。规范依据
   css-values-4 §8.5.6 interpolation-calc；css-shapes-1
   §basic-shape-interpolation 坐标/半径槽同文法。刀 E GONE 定谳第 4 条
   （clip-path-interpolation-002 ×4 跨单位 blend 缺口）靶面兑现。
2. **钉测 1 个**（`animation.rs::test_interpolate_clip_path_cross_unit_calc`）：
   inset 四边 mix 形 + circle 半径/位置槽 + px 系数零折叠纯 % + 关键词半径
   离散保持 + offset-path 通道行为锁（同文法共享 clip_interp，
   `interpolate_offset_path` 复用 `interpolate_clip_path`，跨单位臂同步生效
   ——offset-path gCS verbatim 直出 emit 形，判定可接受：chrome 同文法
   语义、motion 语料不在五 corpus 账内）。engine lib **4,126/0F**
   （clip 定向 130/0F + clip_path/offset_path 精筛 14/0F 双口径）。

## 通道影响面判定

- **clip-path / shape-outside**（animation.rs:754/1011/1014）：canonicalize
  通道经 `calc_computed_to_css`（js_dom_bridge/computed_style.rs:2068）再串
  化——emit `calc(P% ± Xpx)` 形 canonical 口径 % 项在前、零 px 项丢弃、
  `calc(0% + Xpx)` 原样保留（刀 E TRANS 实证同口径），混合形不破坏。
- **offset-path**（animation.rs:684/1008）：verbatim 直出通道同步进 calc
  形（行为锁钉测断言）。
- 离散面持位：关键词半径（closest-side↔px）、calc 端点、单位失配仍 None
  离散，非本刀靶面。

## 权威账（bin-cssa-g1 `c10cb92c`，五 corpus；A 基线 = 刀 F f1 账）

| corpus | A（f1） | B（g1） | 增量 |
|---|---|---|---|
| css-gaps | 3792 | 3792 | 0 |
| css-multicol | 757 | 757 | 0 |
| css-typed-om | 9873 | 9873 | 0 |
| css-shapes | 5108 | 5108 | 0 |
| css-masking | 2362 | **2367** | **+5** |
| **合计** | 21892 | **21897** | **+5** |

**PAGE-ONLY 零容忍对账：F2P +5 / P2F 0 / NEW-PASS 0 / NEW-FAIL 0 /
GONE-PASS 0 / GONE-FAIL 0 / 五 corpus 页集合全等**——纯增益零回归零丢页。
+5 全部落 `css/css-masking/animations/clip-path-interpolation-002.html` 的
`inset(20px) → inset(40%)` 跨单位混合五档（t = 0 / 0.3 / 0.6 / −0.3 / 1.5，
期望 `inset(calc(...))` 形）——即刀 E GONE 定谳第 4 条靶面本尊，逐条对号。
Timeout 面持位（shapes/masking 各 1，与 A 侧同形 flake）。
typedom SET-IDENTICAL 复证 typed OM 通道不涉 clip 插值；shapes 持位符合
预期（inset 输入无 calc 的纯跨单位案全在 masking 动画页）。

## Provenance 与过程纪要

- B = `results-cssa-g1-full`（bin-cssa-g1 c10cb92cc052c92636e8751466d91521，
  HEAD 构建 794aeefe0，07:57:01 落 bin）。构建窗口 es_module s419 peer WIP
  在树（同刀 E/F 形）。
- 链 `launch-cssa-g1-full.sh`（transient service zw-r5-cssa-g1，cd $ROOT
  内层，日志 chain-cssa-g1-full-075721.log，07:57:21 发射 08:02:44 GATE-DONE
  五腿全 rc=0）。
- 修改范围：clip_interp.rs（lerp_len +2 臂 + 文档注释更新）、animation.rs
  （钉测 +1，post-build 窗 fmt 长行修正），共 2 文件。
- **二进制指纹法证（strings 抽验）**：s419 标记串
  `Assignment to module import binding` f1/g1 双 bin 各 1 处 = 抵消假设成立
  （es_module s419 在两代 bin 同态，A/B 差量不含 peer 面）；刀 H 标记串
  `zw-typedom-listvalued-2026-11-17` 双 bin 0 处 = typedom 腿零污染；刀 F
  标记串 `zw-ruleinset-unitmix-2026-11-17` 双 bin 各 1 处 = 基线延续确认。
  g1 bin 与 f1 bin 的唯一源差 = 本刀 2 文件。
- **树态边界注记**：g1 构建后窗口（08:06–08:08）三处非本刀在途变更进树
  ——css-parser 括号感知切分 WIP（其靶面 inset-function-valid 三案
  A/B 双侧 8P/3F 持位未动 = 未进 g1 bin、未随本刀验证）、js_dom_shim
  part10 typed-om 刀 H WIP（指纹双 bin 0 实证未入账）、examples/demo
  删除集（无 evidence 归属）——三者均**不随本刀提交**，留树待归属流
  自证或认领（碰头信号记入 master）。
- 上轮 session 87debb24 于 08:12 限流中断；本轮续开完成五腿账回填
  （corpus JSON 直接对账复算，flip 口径 Pass/Fail 大写定谳——首轮小写
  `PASS` 口径假全零复现刀 F evidence 已记的同类陷阱）。
