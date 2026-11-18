# 刀 K1：关键帧 'inherit' 端点父链接入——cssa/gCS 通道（2026-11-17）

## 面与根因

css-gaps corpus inherit 端点 100F（刀 I 碰头账）：4 页 ×（CSS Animations 50F +
Web Animations 50F）——`column-rule-inset-interpolation.html`（40）、
`row-rule-inset-interpolation.html`（40）、`row-rule-width-interpolation.html`（10）、
`row-rule-color-interpolation.html`（10）。

根因：interpolation-testcommon.js 断言走 getComputedStyle——CSS Animations 半面落
`cssa_registry` 咨询通道（刀 E 建），keyframes start 时 'inherit' 端点无处解析：
cssa `start_animation_with_underlying(..., parent=None)` → 端点保持 `"inherit"`
原样 → 长度/颜色/插值臂 0 化假插值（如 width 从 0px 线性插值，实测 20px 类假值）。
css-cascade-4 §inheritance：'inherit' = 父元素该属性 computed value（继承强制，
含非继承属性）；无父（根元素）行为同 'initial'。

首版认领（CLAIM-knife-k1-inherit.txt 11:40）误标「pipeline 管线通道 50F」，
11:47 对 results-cssa-h1-full css-gaps 断言逐条复核改判：全数 gCS 通道，管线
泵帧改动零 corpus yield——改判后码面含 cssa 通道父链接入（主靶）+ 管线通道
（陪跑加固，真站渲染面正确性）。

## 码面（五文件 + 共文件 hunk）

1. `crates/engine/src/js_dom_bridge/cssa_registry.rs`【本刀独占】：
   `query_override_for_sel/handle/query_core` 加 `parent: Option<&ComputedStyle>`
   → `start_animation_with_underlying` 透传。
2. `crates/engine/src/js_dom_bridge/callbacks.rs`【本刀独占】：gCS per-selector
   miss 路径闭包内 `doc.parent_node(node)` + `styles.get(&pid)` 父 computed 快照，
   闭包返回三元组携出（per-selector map 形状不变）；hit 路径传 None——
   animation-name 写入 bump style_version 清 map，start 锚定恒落 miss 路径，
   hit 臂 ENTRIES 已锚不再消费 parent。
3. `crates/engine/src/js_dom_bridge/computed_style_cache.rs`【本刀独占】：handle
   通道 `doc.parent_node(nid)` + `styles.get(&pid)` 传父（根元素父为 Document
   无样式 → None 回落 initial 同式）。
4. `crates/engine/src/animation.rs`【与 repeater 流共文件，hunk 级拆分提交】：
   - E5 循环 inherit 臂：parent unwrap_or(default)——无父回落 'initial' 同式
     解析（serialize_computed_property_with_sibling 同通道）。
   - interpolate_property_value 长度臂：parse_px_strict 严格解析 + 离散切换
     （css-values-4 §8.5.6 ≥0.5 → to），裸 0 计 0px（css-values-4 §5.2）——
     替换原 parse_px 0 化假插值（关键字端点 auto/min-content 等不再 0 化）。
     共享底座臂（管线/cssa/js_anim tick 三通道共用），A/B 全 corpus 验零扰动。
   - 8 处测试调用点追加 None 实参；2 钉测：
     `test_keyframe_inherit_keyword_resolves_parent`（父 90px → 65px@0.5）、
     `test_keyframe_inherit_without_parent_keeps_fallback`（无父 → width 初始
     auto → 离散 → 40px@0.5）。
5. `crates/engine/src/pipeline/mod.rs` + `pipeline_budget.rs`【本刀独占】：
   `apply_animation_overrides` 加 `doc: Option<&Document>`，泵帧通道父 computed
   解析（tick_animation_styles 传 Some(doc)、render 静态路径 Some(&doc)、
   budget 路径 session.doc.as_ref()）。

FIXME(kf-inherit-waapi)：Web Animations 半面 50F——js_anim 通道父面接入另行开刀
（本刀后该半面端点走 initial 回落，期望值仍 F，页面集合不变）。

## 验证（链后回填 corpus 数字）

- 双 bin：A′ k1aprime（K1 hunks 出树——5 独占文件反向补丁 + animation.rs
  filter-k1-anim-hunks.py 拆分反向补丁，hunk census 21=k1 15+peer 6，peer
  hunks 原位保留）/ B k1（K1 全 hunks 在树）。md5：A′ `0cc733602f786c36be70620c12ff9ec2`
  （bin-j23K1A）/ B `78abca7310474e57a3896bcfbe40bc65`（bin-j23B）。
- 钉测：engine lib 全量 4148/0F（2026-11-17 15:0x 收账，含
  test_keyframe_inherit_keyword_resolves_parent / test_keyframe_inherit_without_parent_keeps_fallback
  两钉 + 8 调用点 None 实参改造）。
- 五 corpus 对账（reconcile-five-corpus.py，results-j23r2-j23{B,K1A}，dirA=B
  → F2P=本刀回归、P2F=本刀 wins；15:3x 收账）：

  | corpus | B | K1A | wins | 回归 | 判据 |
  |---|---|---|---|---|---|
  | css-gaps | 3838 | 3792 | 50 | 4 | CSSA 半面 50 wins ✓ |
  | css-multicol | 943 | 921 | 22 | 0 | 正面外溢 ✓ |
  | css-typed-om | 9947 | 9947 | 0 | 0 | SET-IDENTICAL ✓ |
  | css-shapes | 5123 | 5106 | 17 | 0 | 正面外溢 ✓ |
  | css-masking | 2321 | 2308 | 14 | 1 | 正面外溢 ✓ |

  页集合五 corpus 全等、NEW/GONE 零。合计 wins 103 / 回归 5 / 净 +98。
  主靶 css-gaps 50 wins 全落 CSS Animations 半面（4 页 inherit 端点），
  判据兑现。外溢面（multicol 22/shapes 17/masking 14）共享底座长度臂
  parse_px_strict+离散切换认领 inherit 端点后全 corpus 正面外溢——非违约
  缺陷，逐 corpus 断言抽查均 `from [inherit] to [值]` 形。
- 本刀 5 回归全落 Web Animations 通道端点语义（FIXME(kf-inherit-waapi)
  扩账）：gaps 4× `from [inherit] to [10px] at (1)` got calc-mix 漏出
  （`calc ( 0 % + 10px )` 形）；masking 1× `clip-path from [inherit] to
  [inset(20px)] at (0)` should be inset(10px)——t=0 端点父值未落。均
  js_anim 通道父面接入另行开刀时同族清偿。
- 交叉互证兑现：KA↔B 差量纯 multicol 面（+176，余四 corpus SET-IDENTICAL）
  ——K1 共享底座臂对 K 侧零扰动实证；J2A↔B 差量见刀 J2 evidence 同款判据。
- 提交：531e795e7（六文件）；animation.rs 本刀 15 hunks——工作树反向 apply
  机制（bin 构建同款：K-face 快照出本刀 hunks→提交→复原全量→余量恰 15
  hunks 入本笔），K 侧 filter map-first 过时行号误收本刀文件尾钉测 hunk
  （5353 两 test_keyframe_inherit_*）由收账轮内容定谳归本刀；终局拆分
  K=6/K1=15 并集 21=全量，字节级闭环复验。快照编译证明 cargo check --tests
  绿（K-face×HEAD 态本刀独占五文件）。lei-pre-commit-guard PASS ×2 轮。

## 环境注记

本刀窗内共树在途流：repeater/columns（animation.rs）、shape()/path()
（css-parser serialize_shape_specified.rs，编译红阻塞上游）、typedom 刀 H
收账（part10-typed-om.js，11:27 GATE-DONE）。claim 协调记录见
CLAIM-knife-k1-inherit.txt。
