# 刀 I·important-tier 批（2026-11-17）：css-cascade important-prop 确定性回归修复

## 回归定谳与修复范围

rendering-compat 流在册回归候选 `css/css-cascade/important-prop.html`
（三跑 3/3 确定性稳定 F，9962/480000 px 2.08% 失配，归因刀 F
a68b07336）本刀定谳修复。根因四层：

- **（A）多选择器关键帧折叠**（animation.rs `block_to_point`）：`from,
  to {...}` 块折叠取首个选择器 → 丢 100% 帧 → `register_keyframes` 合成
  空边界帧 → 刀 F 通用 underlying 臂填入计算值，恒定输出动画漂移为渐变面
  （刀 F 激活了此前置缺陷）。
- **（C）关键帧 `!important` 未丢弃**（css-animations-1 §keyframes：无效
  按未声明）——`border-color: red !important` 经 HashMap 末写胜出覆盖同块
  前条 `green`。
- **（B2）动画施值无级联重要度分层**（css-cascade-4 §importance：animation
  declarations 排在 important 声明之下）——渲染 stamping 与 gCS 咨询通道
  动画值整体替代属性，author `!important` 胜出面失守。
- **（D）applier border-color 无臂**（先于本刀存在）：动画 border-color
  输出静默丢弃，keyframe green 落不了盘、元素 `border:10px solid red` 持位。

## 交付面（4 面 + 6 钉）

1. **A 面**（`block_to_point` → `block_to_points`）：逐选择器展开为独立
   KeyframePoint（From→0.0 / To→1.0 / Pct→p/100），同块声明集与段 timing
   复用；`register_keyframes` 改 `flat_map`。
2. **C 面**（`block_to_points` 内过滤）：`d.important` 声明按未声明丢弃
   （keyframes 非级联上下文，重要度标志零排序效果）。
3. **B2 面**：`ComputedStyle` 新字段 `important_won_props: Arc<Vec<String>>`
   ——级联胜者 order 带 `!important` 的属性集（升序去重），由
   `CascadeOutcome` 组合缓存从 `cascade_with_orders` 输出的 `orders` 旁表
   派生一次（`CascadeOrder.important` 既有字段，零级联签名改动），compute
   期挂表。施值门置于 `AnimationClock::apply_to_computed_style` 单点：
   渲染 stamping（pipeline `apply_animation_overrides`）与 gCS 咨询
   （js_anim/cssa 共享 `canonicalize`——跳过即串化恒等返 None 回落级联）
   同通道覆盖。简写按**写达 longhand** 对位（`background` →
   `background-color` R3882 同源；`border-color` → 四边 color 槽）。
4. **D 面**（`apply_single_property` 补臂）：`border-color` 写达四边
   color 槽，与静态声明 apply 同语义。
5. **钉测 6 个**（animation.rs）：strips-timing（R3883 改新签名）、
   multi-selector-expansion（from,to + 50%,75% 两形）、drops-important、
   skips-important-won（含 background 简写洞）、
   border-color-four-slots、register-no-synth-boundaries（全链）。
   engine lib **4,131/0F**（+6 钉）；style-system **2,550/0F** +
   cascade_cache 5/0F（新字段过 oracle 双路径）。

## 验证：reftest 探针差（vintage 行为锚）

- A′ bin（刀前）：`important-prop` ✗ F（2.08%，9962/480000 px）
- B bin（刀后）：✓ **P（0.00% 像素差）**——整面绿方无红，FAIL 字绿色
  隐入绿底，与 ref 一致（REFTEST_DUMP 转储对照）。
- 刀 I 无自有字符串字面量（字段名不入 release bin）——vintage 判别以
  行为探针差替代 strings 指纹；peer 面指纹双侧同态（下节）。

## 权威账（对称消去设计）

**树态**：HEAD 9cb993d61（刀 G 提交）+ peer WIP 对称在树（es_module s419、
css-parser 括号切分、js_dom_shim part10 typed-om 刀H、registry/shorthand
rustfmt 折行[语义零差：arg-list 尾逗号语法糖]）。g1 bin 不含 08:06 后进树
的 css-parser/typed-om WIP——直接对 g1 会毒化账目，故先建 **A′ bin**（同
WIP 态、无本刀）跑五 corpus，再落刀建 **B bin**（同 WIP + 本刀 4 文件），
**差量 = 纯本刀**（peer 面双侧同态抵消）。

- A′ bin `3ddc68391cbfd47fc4b998d4de5d8ff9`（09:53:29 落，42.04s）
- B bin `fa4355e25169658c6efa2541f5df1fb0`（10:10:23 落，30.22s）
- 指纹法证（strings）：s419 双 bin 各 1、刀H typedom 标记双 bin 各 1
  （刀 G 时代 0/0 → 本轮 1/1 = typed-om WIP 对称入账实证）、刀F 标记
  双 bin 各 1（基线延续）。

### A′ ↔ g1（peer WIP 面定量化，非本刀账）

| corpus | g1 | A′ | delta |
|---|---|---|---|
| css-gaps | 3792 | 3792 | 0 |
| css-multicol | 757 | 757 | 0 |
| css-typed-om | 9873 | 9944 | **+71**（F2P 74/P2F 3） |
| css-shapes | 5108 | 5106 | −2（F2P 10/P2F 12） |
| css-masking | 2367 | 2308 | **−59**（F2P 10/P2F 69） |
| **合计** | 21897 | 21907 | +10 |

typed-om +71 = typedom 刀H WIP 靶面在账（其归属流落地时认领）；
masking −59 / shapes −2 = css-parser 括号 WIP 面现状（**碰头信号**：
在途 WIP 对 masking 净负 59，归属流落地前需自行 A/B 定谳）。页集合
五 corpus 全等，子测试级无 NEW-GONE。

### B ↔ A′（本刀权威账）

**SET-IDENTICAL**（bin-cssa-i1 `fa4355e2`，五 corpus；B 链 10:10:38 发射
10:16:52 GATE-DONE 五腿全 rc=0）：

| corpus | A′ | B（i1） | delta |
|---|---|---|---|
| css-gaps | 3792 | 3792 | 0 |
| css-multicol | 757 | 757 | 0 |
| css-typed-om | 9944 | 9944 | 0 |
| css-shapes | 5106 | 5106 | 0 |
| css-masking | 2308 | 2308 | 0 |
| **合计** | 21907 | 21907 | **0** |

**PAGE-ONLY 零容忍对账：F2P 0 / P2F 0 / NEW-PASS 0 / NEW-FAIL 0 /
GONE-PASS 0 / GONE-FAIL 0 / 五 corpus 页集合全等、子测试级零翻转**——
本刀净 0：回归修复（GONE-1 靶面）零 corpus 扰动。账外靶面 important-prop
（css-cascade 不在五 corpus 账内）F→P 兑现即本刀交付；五 corpus 账不增
不减为修复的零副作用实证。

Timeout 面持位（shapes/masking 各 1，与 A′ 侧同形 flake）。

## Provenance 与过程纪要

- 链 `launch-cssa-aprime-full.sh` / `launch-cssa-i1-full.sh`（transient
  service，test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1800
  包腿；CASE_WALL_S=90；results/transcript 按 vintage 分名）。
  i1 链 START/GATE-DONE 标记串沿用 APRIME 大写形（脚本 sed 派生残留，
  日志文件名与 provenance 行均为 i1 权威，纯注释面不影响账目）。
- 对账脚本 `.acceptance/wptfyi-css/reconcile-five-corpus.py`（本次新增，
  子测试级 F2P/P2F/NEW/GONE + 页集合等值 + 翻转清单；status 大写口径——
  刀 F/G evidence 同款陷阱防复发）。
- 修改范围 4 文件：engine/animation.rs（block_to_points + 过滤 + 施值门 +
  D 臂 + 6 钉）、style-system lib.rs（CascadeOutcome 派生 + compute 挂表）、
  property/computed_style.rs（字段）、property/default_impl.rs（Default）。
- peer 会话并发注记：本刀窗口内 peer 会话对 registry.rs/shorthand/mod.rs
  落 rustfmt 折行（10:00:52，arg-list 尾逗号语法糖级、`git diff -w` +
  归一化比对定谳语义零差）；B 构建扫入不毒化账目（非源语义面）。
  提交以显式 pathspec 限本刀 4 文件 + docs，peer WIP 一概留树。
