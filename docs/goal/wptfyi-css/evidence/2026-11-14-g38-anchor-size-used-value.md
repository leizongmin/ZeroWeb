# g38：anchor-size() used 值布局期求值刀——六尺寸槽/四 inset 槽双形承载

日期：2026-11-14。承接 g37 挂账「anchor-size used 值布局期求值刀」
（evidence 2026-11-14-g37-anchor-serialize-canonical.md）。目标页族：
anchor-size-001（26F）/ anchor-size-writing-modes-001（24F）/
anchor-size-replaced-001（8F）/ anchor-name-003 混翻面（35F）。

## 1. 语义面（css-anchor-position-1 §4）

- **承载模式**：样式解析面无锚几何 fail-closed——六尺寸槽
  （width/height/min-width/min-height/max-width/max-height）声明原文
  小写入新字段 `ComputedStyle::anchor_size_slots`（`AnchorSizeSlots`
  六 `Option<Box<str>>`），槽位存初始 Auto；布局期 anchor_pos pass3
  （`apply_inset_walk`）按锚几何回填 used 宽高。与既有 inset 槽
  anchor() 承载臂同模式。
- **捕获门**：值含 `anchor-size(` 且括号平衡（未闭括号形 = 语料
  anchor-size-001 上游 typo `anchor-size(--a1 width"`，Chrome 同样
  声明无效，落既有 parse 拒面不修）；大小写归一。
- **轴映射**（§4 + anchor-size-writing-modes-001 钉面）：width/height
  物理直取；inline/block 按**包含块**写作模式；self-inline/self-block
  按**目标元素自身**写作模式；锚自身写作模式无关。
- **隐式缺省关键字**（§4 "defaults to the axis of the property"）：
  width 族槽 → self-inline、height 族槽 → self-block、物理 inset
  left/right → width、top/bottom → height。
- **fallback**：显式名查无 + 默认锚 None → `<length-percentage>`（%
  按槽轴向 CB padding box 基）→ 仍无 → None（槽位保持初始，IACVT 链
  与 parse 失败回落一致）。
- **replaced 均匀缩放**（§4 "Specifying the width or the height should
  scale uniformly"）：有效比 = aspect-ratio 属性优先，否则当前 used
  尺寸固有比；min/max 先 min 后 max（CSS2 §10.4 约束表次序），每步
  等比对侧缩放。
- **stale 槽防回灌**：普通值成功路径清承载槽（六尺寸槽 + 四 inset 槽
  parse 路径清 `anchor_inset_*`——inset 臂既有 latent bug 本刀一并修，
  后声明胜出）。
- **inset 槽双形**：四 inset 臂捕获门扩为 `anchor(` ∨ `anchor-size(`
  （`anchor-size(` 无 `anchor(` 子串——`anchor` 与 `(` 间有 `-`）；
  anchor-size() inset 槽求值产出 `<length>` 直接入槽（无 anchor()
  坐标/对边换算语义）。

## 2. 实现

- `crates/style-system/src/property/computed_style.rs`：新字段
  `anchor_size_slots: Option<Box<AnchorSizeSlots>>` + 新 struct
  （derive Debug/Clone/Default/PartialEq）。
- `crates/style-system/src/property/default_impl.rs`：`None` 初值。
- `crates/style-system/src/property/apply.rs`：`anchor_size_slot_capture`
  helper + 六尺寸臂捕获/清槽 + 四 inset 臂双形门/清槽。
- `crates/layout-engine/src/engine/anchor_pos.rs`：
  `anchor_size_default_keyword` / `anchor_size_axis_px` /
  `resolve_anchor_size_raw`（`AnchorSizeCtx` 收参——clippy
  too_many_arguments）+ pass3 resolve dispatch（`anchor-size(` 前缀
  分流）+ 尺寸槽覆盖块（replaced/非 replaced 两臂）+ collect_anchors
  门扩 `anchor_size_slots.is_some()`。

FIXME 挂账（本刀边界外）：

- **margin 槽**：anchor-size-001 margin 段 12F——gCS used margin 回写
  需 gCS 静态路径（GENERATION_CACHE 级联）↔ pipeline 布局几何跨通道
  布线，另刀（computed_style.rs 字段注释同记）。
- **gCS width/height resolved-value**：anchor-ident-function
  `width: anchor-size(--a1)` gCS 期望 15px（resolved = used）——本刀后
  内联双形声明被接受（capture 返回 true）→ 内联层叠胜出 → width 槽
  Auto → gCS 静态路径读 'auto'（g37 前该声明 parse 失败，内联不生效，
  样式表 15px 偶然同值伪通过）。gCS resolved 值政策刀域（与
  invalid-fallback 残面同类），挂账见 §5。

## 3. 单测

- style-system 4 测（extended.rs）：承载+Auto+大小写归一+四 min/max
  槽+普通值不置位 / 后声明清槽+none 早退清 / 未闭括号 apply false+
  槽不置位 / left 双形承载+普通值清槽+top anchor() 承载+auto 清槽。
- layout-engine 5 测（g1f_anchor_pos_tests.rs）：物理+implicit 六案
  / VRL CB 轴映射四关键字（锚置 CB 外隔离引擎 VRL 子盒逻辑换向面，
  target 显式 HTB）/ inset 槽 left 定位+top implicit / fallback
  length+percentage+无 fallback 保初始 / replaced 均匀缩放四案
  （min 放大/max 缩小/aspect-ratio 0.5 与 2）。
- unit：css-parser 3208 / style-system 2538 / layout-engine 2515 全过；
  fmt 净；clippy -D warnings 净。

## 4. corpus A/B

基线仲裁：对**本轮 HEAD**（results-g39-head-committed，含 g39 已提交
strut 门、不含 g38——见 §6 归因）与本轮刀后（results-g38-anchor-size）
同链路全量逐页 diff。总账：**+77F→P / −7P→F，净 +70**（77/7 全为 Fail↔Pass，
无 Timeout 迁移）；HEAD+g38 = 265 页 **11,199P/1,654F/4T**（HEAD 态
11,129P/1,724F）。三 corpus（g37-final / HEAD / HEAD+g38）键集逐案
同一（12,857 案、零重名），diff 逐案同基可比。

**账基线更正**：g37-final 在账线为 11,993P/864F/4T，但 results-g37-final
JSON 工件实存 **11,989P**/864F/4T（Δ+4P；该跑原始 stdout 日志未存，
无法复源；本轮三 corpus 全按工件 JSON 逐案计数）。对工件基：
g37-final 11,989P → HEAD 11,129P → HEAD+g38 11,199P，真态差 **−790**
（= g39 揭穿 −860 + g38 +70）。master.md 旧线 11,993P 与 12,061P/796F
（g39 组合态）两处 ±4 与工件/留存日志不合（后者留存日志实存
12,061P/**792**F），记账时一并更正。

逐页：

| 页 | Δ |
|----|---|
| anchor-size-001 | +17 |
| anchor-size-writing-modes-001 | +24 |
| anchor-size-replaced-001 | +8 |
| anchor-size-minmax-001 | +3 |
| anchor-name-001 | +2 |
| anchor-name-002 | +4/−2 |
| anchor-name-003 | +8/−4 |
| anchor-name-004 | +3 |
| anchor-name-multicol-002 | +1 |
| anchor-position-dynamic-002 | +2 |
| anchor-position-multicol-006 | +1 |
| anchor-position-multicol-colspan-001 | +1 |
| anchor-query-fallback | +2 |
| anchor-fallback-invalidation | +1 |
| anchor-ident-function | −1 |

（anchor-size 四页合计 +52，与开工面估计一致；探针快验同数：
001 19P/9F、wm-001 24P/0F、replaced 12P/0F。）

## 5. 页内残面归因

- **anchor-name 家族 −6 混翻负半**（002 ×2、003 ×4）：锚资格判定
  文法缺口——本刀 lookup 沿用 g37 平面 AnchorMap（inline 流门 + 滚
  上下文门），未实现 §3.2 资格规则（含块链祖先条件、tree-order
  「positioned after」排除、abspos 锚限定）；anchor-size 开始求值后
  「不该命中的声明」从伪失效变真解析（正半 +18 同门红利）。资格刀
  另立（与 anchor()-only 页 anchor-position-004 等 32F 同域）。
- **anchor-ident-function −1**：gCS resolved-value（见 §2 FIXME）。
- **invalid-fallback 6F** 持平（gCS computed-time fallback 政策，
  g37 挂账不变；其中 4F 经 g39 轮 comb3-x7 skew 定谳伪红复正）。

## 6. 本轮定谳：g39 strut 门暴露 writing-modes-001 伪通过桶（−864）

本轮 A/B 期间发现 HEAD（g39 已提交态）对
anchor-position-writing-modes-001 全页 1296P→432P/864F（solo 复现，
特征 `expected [200, 200, 500, 300] got [100, 500, 600, 600]`，vrl
系组合全翻；g37-final 态 1296P）。**归因与 g38 无关**（g38 stash 后
HEAD 二进制 solo 同翻 432P/864F 双证；g38 对该 anchor()-only 页
零改动语义面）。

机制：该页 target 为 VRL CB 内 abspos 空盒（inset 全 anchor() 捕获→
style top/right/bottom/left 全 Auto）——sizing pass 内 `width_fix`
被 D2 门关（`cb_horizontal=false`）、`height_fix` 被 g39
`has_inline_content` 门关（空盒无行内内容）→ 整盒跳过 →
`style != style_before` 不触发 set_style/dirty → **空转重布局不再
发生**；而该重布局（g39 前 strut 恒写强制 dirty 触发）对 VRL 容器
in-flow 锚几何是 load-bearing 的——writing-modes-001 1296P 系该副作用
撑出的伪通过（g36 挂账桶「writing-modes-001 伪通过」正是此疑）。

定谳：g39 门揭了一个**先在 VRL in-flow 布局缺口**（垂直容器内流内盒
几何依赖 sizing pass 的二次布局；本刀单测 VRL CB 内锚盒 style
width→物理盒逻辑换向同域实证）。非规范回归——伪通过摘除后页面回到
真态红；修复面 = VRL in-flow 布局正确性刀（大域，另立）。

g39 流程账目修正：g39 A/B 二进制（18:27:55 起编）先于终版 gate
精化（18:39 提交），「本刀 +3 零回归」对**已提交态**不成立
（provenance 法证：A/B 态 vs 提交态 sizing.rs 分歧，本轮 HEAD
solo bisect 实证）。g37-final → HEAD 逐案分解（工件基）：直接效应
**+5**（position-area-basic ×2、multicol-001、dynamic-001、
dynamic-004 各 +1——比 A/B 旧账 +3 多后两项）、伪通过摘除 **−865**
（writing-modes-001 864 + zz-g36probe-wm001-drain 探针页 1），净
**−860**；g38 刀独立计（+70）。

## 7. 验证链

- systemd scope release 重建 zero-wpt-runner（本树 g38 全改动 +
  HEAD g39）；test-guard 包裹探针 + 全量 corpus
  （results-g38-anchor-size，transcript 唯一名）。
- HEAD 基线 corpus（results-g39-head-committed）同链路重录（g38
  stash 态；solo bisect 双证见 §6）。
- 三 corpus 键集逐案同一性 + 逐案迁移矩阵已程序化核对（77/7 全
  Fail↔Pass；g37→HEAD 5/865 见 §6）。
- 单测三 crate 全过 + fmt/clippy 净（见 §3）。
- 结果目录与日志均在 `.acceptance/wptfyi-css/`（results-g37-final /
  results-g39-head-committed / results-g38-anchor-size，logs/
  zw-g39-head-committed.log 等），不入库。
