# Phase A — IFC 三路径统一设计（Spec + RFC）

**版本**：v1.2（+ v1.3/v1.4/v1.5 addendum 见文末）
**日期**：2026-06-19 起；最近 addendum 2026-08-04（R2605）
**状态**：**设计中，整体未实施**。Phase 1/2 部分 LANDED（R207 PHASEA_STORE_EXT + R355 多行放宽 + R817 linebox Phase 2 +45 case）；**Phase 3（line-box metric unification）未解** = reftest 大盘 57% / 37-form-controls label overlap / vertical-mode 的共同结构性阻塞。v1.4 addendum（R1985）裁决：「勿再以 line-box metric / inline-block identity 为独立 lever 狩猎——fix 须随 Phase A 整体 unification」。pre-authorized ruling #4（多 session）。
**关联**：`docs/goal/rendering-compat/master.md` R125 / R198 / R205 / R207 / R208 / R209 / R213 / **R306**；DC-13 产品 smoke 文本保真；DC-14 真实一致率

> **⚠️ v1.2 重大修订（R306 Phase 0 探针）**：原 §0/§6.1/§7.1 推荐的「baseline-resolved 单一权威行盒（baseline_y = 几何基线 frag.y+height）」方案经 env-gated A/B 实证**证伪**——font-051 用 `v_offset=frag.height` 渲染 **16.67% FAIL**，默认 `v_offset=is_ahem?0:font_size` **0.00% PASS**（详见 §6.3B）。geometric baseline ≠ fontdue render baseline。原 Phase 1（加 baseline_y=几何基线）作废；Phase 1 重定向为 **Gate 2 放宽（offset 校准不动）**。offset 语义非 Phase A 阻塞点；真硬阻塞 = 墙② multicol + 换行精度。下文 §0/§6.1/§7.1 的「baseline_y」措辞应据此修订理解。

> **v1.3 勘误（R376 会话，post-R355~R373 实证，2026-06-20）**：本设计 v1.2 描述的 Gate 2（`lines.len()<=1 && is_pure_ahem`）已**过时**。R207 `PHASEA_STORE_EXT` + R355 多行放宽后，实际存储条件（`crates/layout-engine/src/inline_finalization.rs:300-339`）已覆盖：① 有直接文本子节点的 block 容器；② 纯 inline-level 叶文本子容器（无 block 子、inline 子无元素后代），仅显式排除混合 inline+block 内容（line 308）。§2.2/§6.3/§7.1 基于旧 Gate 2 的措辞应据此重读。
>
> **墙 ③（混合内容存储）非当前 lever**：`inline_finalization.rs:308` 显式排除混合 inline+block 内容容器，但**当前 47 个 self-source 失败案无一依赖该排除被移除**——border-bottom-width-006 是匿名块盒*生成*缺失（R361，R109 谱系，非存储），余皆 multicol/baseline/writing-modes/taffy/large-font 簇。移除该排除只会重演 R206 回归（ifc-001/002/003），无 upside。**勿再以「放宽混合内容存储 / Wall ③」为单会话杠杆**（纠正本会话前一轮 CONTINUE 误指）。
>
> **实际失败分布（47 案）→ 真实剩余 lever**（按计数）：multicol 碎片化 13（Phase 2，paint 侧 R157/R198/R203/R317/R122 五轮死路，须 layout 侧 column-aware IFC）/ flex-baseline 3 + flex-abspos 2（baseline-export / abspos §10.3.7 shrink-to-fit）/ writing-modes 5（轴交换）/ taffy-blocked（max/min-content, table-cell-width-0 REF 的 fit-content-on-flex）/ large-font 2（empty-inline-002 25.78% + ifc-011 1.23%）。
>
> **empty-inline-002 诊断（R378 纠正 R376/R377 误判）**：~~R377 据 band 分析误判「span 绿色填充缺失 = paint bug」~~。R378 用 PAINTDBG 插桩（`paint_node` 入口打印 node/display/abs/w/h/border/padding/bg/r109_split/frag_ids）+ 逐像素采样**确证**：空 `<span>`（display:Inline）**被 paint_node 访问且几何正确**（abs=(133,215) w=250 h=350 bt/bb/bl/br=25 pt/pb=100 bg=green，`skip_split_inline_deco=false`），像素 (250,220)/(250,300)/(250,400) 实测全绿 (0,128,0)——**span 渲染正确，非 paint bug**。真 25.78% diff = test 与 ref **结构性形状失配**：test 用 div1 `margin-top:100` + div3 `position:relative;top:-150;z-index:-1` + div3>div `top:-125` 嵌套定位产出绿色形状（内容起 y=135），ref 用更简结构（绿色起 y=35）；relative offset 实测已应用（div3 box.y=250 含 -150）。属 large-font/嵌套定位结构性簇（R125），**非 clean slice，勿再以 paint bug 重查**。
>
> **📍 行号勘误（engine.rs 拆分，R2298）**：本文档正文多处 `engine.rs:NNNN`（如 :1152/:1668/:1681/:1720/:1910）为**设计时**位置（engine.rs 未拆分前）。该文件后续重构拆分到 `crates/layout-engine/`（engine crate 根现为 `lib.rs`；IFC/inline/line-box 代码现主要在 `layout-engine/src/inline/`、`layout-engine/src/engine.rs`、`layout-engine/src/inline_finalization.rs`、`layout-engine/src/engine/postprocess.rs`）。实施 Phase A 前须**按函数名重新定位**（行号已 stale，勿直接引用）。
>
> **v1.4 addendum（R1985 会话，R1982b/R1983 实证，2026-07-24）·R109 mixed-children 匿名块机制精确定位 = Phase A IFC unification 的具体 manifestation·非独立可切片 lever**：承接 visuren subdir 狩猎（anonymous-boxes-001a 6.14%，§9.2.1.1）。R1982b 两路径 A/B：div{height:200px} 含 inline content（text + inline-block span height:50%）+ block `<p>` → **INLINE-ONLY**（无 block 子）span height=100 ✓ 正确；**WITH-BLOCK-CHILD**（`<p>` 触发匿名块生成）span #t 在 box 树**找不到**。R1983 box 树 dump：div#anc(h=200) 子 = [div#anc(h=40, **复用 div node_id**)（=「匿名块」错误归因，应 None）, `<p>`]，span 完全不在树。R1985 code-archaeology 定位**精确机制**：`tree.rs:1112-1179` 消费 `compute_block_container_split` 的 `InlineBlockSegment::Inline` 时，对 `is_block_mixed`（block 容器混合内容）分支建**单个 LEAF taffy 节点**（`new_leaf_with_context`，line 1138）——`display:Block` 的叶子，由片段**首个文本节点**作 measure context（line 1125-1129 + 注释 line 1124「多节点片段仅按首节点近似尺寸，已知限制」），`item_node_ids` 仅登记入 `fragment_registry`（line 1160）**不 build_subtree**。即匿名块是「按首文本节点近似测量的叶子」，**非跑 IFC 的容器** → inline-block/inline-replaced 子无 LayoutBox、%height CB 无承载（故 span 丢失）。对照：INLINE-ONLY 时 div 自身是容器跑 IFC，inline-block 子正常 build → 有 LayoutBox（height=100）。★ **裁决**：R109 mixed-children 匿名块「inline 子丢失 identity」= **本设计 Phase A IFC unification 的精确 manifestation**（匿名块须从「叶子近似」升级为「跑 IFC 的真容器」才能保留 atomic inline-level 子 box identity + CB）——**非独立可切片 lever**（触 core tree-build + IFC ownership = 墙 ③/Path A/B 同根，deadlock 史 R125/R206/R213）。fix 须随 Phase A 整体 unification（消灭 Path B / 匿名块跑 IFC 存权威行盒），非单会话切片。★ **对狩猎的 implication**：勿再以 anonymous-boxes-001a / block-in-inline mixed-children 为独立 lever 狩猎——它们是 Phase A unification 的 success signal（unification 后 WITH-BLOCK-CHILD 应找到 span height=100，R1982b probe 即自动 success signal）。向前 = Phase A 整体实施（pre-authorized ruling #4 多 session）或 font-stack 授权。

---

## 0. 执行摘要

- **一句话目标**：消除「layout 阶段 IFC」与「paint 阶段 IFC」的二次运行分歧，让 paint 对所有行内容器直接消费 layout 存储的权威行盒结果，从而根除 large-font（100px→16px）、welcome/morning.work 文本度量失真、multicol 多行交互等整簇缺陷。
- **本期范围**：仅产出设计文档 + 分阶段实施计划。**不落地代码**（本轮为 R305 read-only 设计）。
- **明确排除**：multicol-breaking 的 column-aware IFC 碎片化模型（独立 RFC，见 `multicol-fragmentation-design.md`）；writing-mode 轴（R114/R164 4 轮证否）；intrinsic sizing（R97/R301，taffy-blocked per R304）。
- **核心约束**：① 任何阶段**零 count 回归**（项目硬标准）；② 单文件 ≤2000 行（`engine.rs` 现 3969 行，本设计触及处须同步拆分）；③ paint 不得以「改变布局语义」的方式重排 glyph（goal DC-13）。
- **推荐方案**：**baseline-resolved 单一权威行盒**——compute_final 存储每行盒的 `(line_top, baseline_y, line_height, fragments[])`，每个 fragment 存**已解析的绝对基线 y**；paint 永远从该结果渲染，删除 Path B（空 styles 重跑）。用「font_size 一致性不变量」取代 Gate 2 的 single-line+pure-Ahem 启发式。
- **首个落地步骤**：Phase 0（read-only 实测探针）= 用 `LAYOUT_DUMP=1` + 临时 glyph 位置插桩，确证 `frag.y + height == 实际 glyph 基线` 是否普适（R305 执行期发现 GlyphPrimitive.y 即基线、doc 注释亦声明 frag.y+height=基线，但 Path A 的 is_ahem?0:font_size offset 与之耦合须经实证），据此决定 Phase 1 用现有字段还是新增字段。详见 §6.3A / §7.1。

---

## 1. 背景与目标

### 1.1 背景

ZeroWeb 的行内排版（IFC）结果目前在 layout 和 paint 两阶段**各跑一次**，且两者输入不同：

- **layout 阶段**（`compute_final_inline_layouts`，engine.rs:1668）用**真实 ComputedStyle** 跑 IFC，得到正确的 font-size / line-height / line-breaking。
- **paint 阶段**（`paint_text`，text.rs:846）对**未存储** inline_layout 的容器**重新跑一次 IFC**，但用**空 styles + override maps**（R72 为规避 4 个回归而保留的安全路径），font-size 默认 16px。

两趟结果在 font_size、line-breaking、垂直定位上分歧。最典型的可见症状：**large-font bug**——`font-size:100px` 的 Ahem 文本在 paint 阶段被 16px 默认值覆盖（ifc-008/009/011、font-051 多行变体、welcome 标题）。

R125 / R198 / R205 / R209 / R213 共 5 轮尝试单点解锁 font_size 均**净负向回归**：

| 轮次 | 尝试 | 结果 |
|------|------|------|
| R125 | 三路存储（store_font_sizes 覆盖/不覆盖/真实 styles） | 全净 -1/-1/-4，回退 |
| R198 | compute_final IFC 后 store_font_sizes + multicol ancestry 守卫 | 净 -1（CSS2 +1 / css-multicol -1），死锁成立 |
| R205 | paint 注入真实 font_size 单字段（解耦 line-height） | 全净负，font_size 与 line-height 耦合 |
| R207 | narrow 精修：仅「纯 inline 叶文本容器」存行盒 | **+1（font-051）零回归，默认启用** ✅ |
| R209 | 放宽 Gate 2 多行存储（PHASEA_MULTILINE） | ifc-008/009 改善但 multicol-fill-auto-001 0.63→9.15 回归，回退 |
| R213 | 多行存储加 `!in_multicol` 守卫 | 净 0（multicol-fill-auto 由 ref 文件非 multicol 的 float 模拟，守卫无法触及），回退 |

R207 证明**存储架构本身正确**（pure-inline 叶文本容器 +1），但 broad 应用被三处墙阻塞。本 RFC 的任务是把这些墙精确定位并给出**架构性**（非单点）的统一方案。

### 1.2 目标

- **业务目标**：让 ZeroWeb 的行内文本渲染与 Chromium 在 font-size / line-height / 换行上一致，消除 large-font 簇缺陷（DC-13 welcome 文本、DC-2/5 文本类 reftest）。
- **用户目标**：产品静态页（welcome / morning.work）正文不再被压成 16px 默认行；标题、卡片文本保真。
- **可验证成功标准**：① large-font 簇（ifc-008/009/011）chromium-Oracle z_vs_chr 下降；② welcome product-smoke 文本区 diff 下降；③ 全量 reftest loose 438/490 不退、strict 296/490 不退、chromium-Oracle 真实一致率不降。

---

## 2. 现状分析（三条 IFC 路径 + 两个 Gate + 三处墙）

### 2.1 三条 IFC 路径

```
                    compute_final_inline_layouts (engine.rs:1668)
                            │  用真实 styles 跑 IFC
                            ▼
                 ┌──────────────────────────┐
                 │ Gate 2 (engine.rs:1910)  │
                 │ lines.len()<=1 &&        │── 否 ──▶ 不存 inline_layout
                 │ is_pure_ahem             │         （但仍调 store_font_sizes_from_ifc）
                 └─────────────┬────────────┘
                          是   │  存 inline_layout (line boxes)
                               ▼
                 paint_text (text.rs:807)
                 use_stored = !multicol && inline_layout.is_some() && width_matches
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
   Path A: use_stored=true                Path B: use_stored=false
   渲染 stored fragments                  重跑 IFC（空 styles + override maps）
   v_offset = is_ahem?0:font_size         baseline_fs = text_node_font_sizes[node] or 16px
   (text.rs:1208)                         (text.rs:1224-1225)
```

**Path A（stored）**：compute_final 用真实 styles 算出正确的 `frag.y`（fragment 框顶部）+ `frag.font_size`，paint 直接渲染，`v_offset = is_ahem ? 0 : font_size`（Ahem 位图是完美 font_size 方块无 ascent 留白→offset=0；普通字体 font_size≈ascent）。

**Path B（re-run）**：paint 用**空 styles** 重跑 IFC（`frag.y` 基于 16px 默认），再用 `text_node_font_sizes` map 里存的**真实 font_size** 作为 `baseline_fs` 修正垂直定位。R72 刻意用空 styles 而非真实 styles，是为规避 BFC-004 / font-feature-002 / position-absolute-in-inline-005/006 四个回归。

**关键事实**：`store_font_sizes_from_ifc`（engine.rs:1152）在 compute / remeasure 多处（line 1079/1381/3136/3266）被调用，**不受 Gate 2 限制**——即 per-text-node 的 font_size/line_height/is_ahem map 总是广泛建立。Gate 2 只限制 **`inline_layout`（完整行盒）** 的存储。

### 2.2 两个 Gate

| Gate | 位置 | 条件 | 作用 |
|------|------|------|------|
| **Gate 1**（R207 narrow） | engine.rs:1720-1749 | `has_text_children` 扩展 = 有 inline-level 元素子节点 **且** 无 block-level 子节点 **且** inline 子元素无元素后代（叶文本容器） | 决定**哪些容器**进入 IFC 计算 |
| **Gate 2**（R84 安全子集） | engine.rs:1910 | `lines.len() <= 1 && is_pure_ahem`（纯 Ahem 单行） | 决定**哪些容器实际存储** inline_layout |

另有显式跳过（engine.rs:1681-1707）：flex/grid/table 容器、`is_multicol` 容器、非 block-level 元素。

### 2.3 三处墙（broad 应用阻塞点）

**墙 ① — Gate 2 多行限制（~~large-font 簇根因~~，R327 实测纠正）**
> ⚠️ **R327 env-gated 控制实验纠正**：原断言「唯一阻塞 = 多行限制」**错误**。R327 加 env `PHASEA_AHEM_MULTILINE=1`（比 R209 更窄——保留 is_pure_ahem，仅去 `lines.len()<=1`）实测：放宽多行后 ifc-008/009/011 **仍不过**（ifc-008 8.18%→4.17%、ifc-009 6.11%→4.17% 改善但有墙③残余；ifc-011 11.27% 不变未触及 stored 路径）。真阻塞 = 墙③（Path A multi-line 垂直定位）+ 墙②（multicol 一致性），**非** Gate 2 调参。

ifc-008 = `div1 > inner-div(block) > "XX XX" 100px Ahem`，200px 宽换 2 行。inner-div 是 block + 直接文本 → 过 Gate 1（line 1710 直接 `has_text_children=true`，不走 R207 扩展）。但 Gate 2 `lines.len() > 1` → 不存 → paint 走 Path B → 16px。R209 已用干净单趟探针确认 node 被访问、block=true、direct_text=true，原判「唯一阻塞 = 多行限制」。~~但 R327 实测放宽后仍不过（见上方纠正）~~。

**墙 ② — multicol 反向依赖（R198/R209/R213/R327 resolved）**
multicol 容器 paint 永远走 Path B（`use_stored = multicol_info.is_none()`，text.rs:807；multicol_info 在 `!has_in_flow_children && is_balance_mode && height_auto` 时计算，text.rs:713）。放宽 Gate 2 让 multicol 的**内层内容容器**存 inline_layout 后，multicol-fill-auto-001 从 0.63%→9.15% 回归。

> ✅ **R327 resolved 机制**（原「疑点，需 Phase 2 探针实证」）：multicol-fill-auto-001 的 **TEST** = 真 multicol（column-count:3，纯 Ahem），paint 走 `use_stored = multicol_info.is_none()` = **false → Path B 不变**；其 **REF** = float 模拟列（非 multicol），放宽后这些 float 容器（纯 Ahem 多行）切 **Path A** → test(Path B) vs ref(Path A) 分歧 → 破。R327 实测 multicol-fill-auto-001 当前**通过**（不在 52 失败集），放宽后 pass→fail（精确 -1）。**layout 无法区分「ref 上下文」故该墙不可守**（R213 `!in_multicol` 失败同因：守 TEST 侧 multicol 无用，破在 REF 侧 float）。真解 = Phase 2 让 multicol 也走 Path A（column-aware），使 test/ref 一致。原「回归可能不是 font_size map 变化」推测**证实**——是 ref 侧 float 容器切 Path A 的几何变化。

**墙 ③ — v_offset / baseline 语义分歧**
Path A 用 `v_offset = is_ahem ? 0 : font_size`（text.rs:1208），Path B 用 `baseline_fs = stored or 16`（text.rs:1225）。两者对「fragment.y 相对行的垂直锚定」假设不同。对多行非-Ahem 内容，stored 的 `frag.y`（真实 font_size 下的行顶）+ Path A v_offset 与 Path B 的 16px 行顶 + baseline_fs 不一致——这是 R206 broad 应用导致 ifc-001/002/003 翻 FAIL 的直接原因。**只要 Path B 还存在，两套语义就必须手工保持一致，而这已被 5 轮证明不可单点维护。**

### 2.4 结论

墙 ③ 是**架构性**的：只要 paint 同时存在「消费 stored」与「重跑 IFC」两条路径，两套 baseline 语义就无法收敛。R207 的成功恰恰是它把 Path A 限制在「single-line pure-Ahem」这一**两路径天然等价**的子集上。真正的解法是**消灭 Path B**——让所有通过 Gate 1 的容器都存 inline_layout，paint 永远用 Path A，并让 Path A 的位置语义对多行非-Ahem 也正确。

---

## 3. 范围边界

- **在范围内**：
  - `compute_final_inline_layouts`（engine.rs:1668）的存储条件与存储字段语义
  - `paint_text`（text.rs）的 use_stored 决策与 Path A 渲染路径
  - `InlineLayoutFragment` / `InlineLayoutLine`（types/mod.rs）数据结构
  - multicol paint 的 font_size 一致性（仅触及「multicol 内层容器的存储是否触发」，不触及 column 分配算法）
- **不在范围内**：
  - multicol-breaking column-aware IFC 碎片化（→ `multicol-fragmentation-design.md`）
  - writing-mode 轴 / vertical-rl（→ R114/R164，4 轮证否）
  - intrinsic sizing / max-content（→ R97/R301，taffy-blocked）
  - IFC 内部换行算法本身（advance-width plumbing → `advance-width-plumbing-design.md`，已证伪为独立死路 R225）

---

## 4. 设计需求（FR）

### FR-001：单一权威 IFC 结果
- **描述**：paint 必须消费 compute_final 存储的行盒结果渲染行内文本；当 inline_layout 存在且宽度匹配时，paint **禁止**重跑 IFC。
- **优先级**：必须
- **验收场景**：
  - 场景（正常）：给定一个 `font-size:100px` 单行 Ahem 容器，paint 渲染的 glyph 高度 == 100px（非 16px）。验证：`make reftest` ifc-008 class 不再出现 16px 文本（chromium-Oracle z_vs_chr 下降）。
  - 场景（异常）：给定一个 inline_layout 未存储的容器（如 flex/grid/table，被 Gate 1 显式跳过），paint 必须回退到现有 Path B 重跑，行为与现状一致。验证：全量 reftest 这些类目 count 不变。

### FR-002：baseline-resolved 位置语义
- **描述**：stored 行盒必须携带**已解析的绝对基线 y**（`baseline_y = line.y + ascent`），paint 直接用该 y 定位 glyph，不再用 `is_ahem ? 0 : font_size` 启发式 v_offset 推断。
- **优先级**：必须
- **验收场景**：
  - 场景（正常）：多行非-Ahem 文本，每行 glyph 基线 y 与 Chromium 一致（行间距 = line-height）。验证：`make reftest` ifc-001/002/003 保持 PASS（R206 broad 翻 FAIL 的三例须恢复）。
  - 场景（异常）：若某 fragment 的 ascent 无法解析（无 FontLoader 度量），回退到当前 `font_size` 近似并记录 tracing 日志，渲染不崩。验证：单测 `baseline_y_fallback_uses_font_size`。

### FR-003：font_size 一致性不变量
- **描述**：删除 Gate 2 的 `is_pure_ahem && lines.len()<=1` 启发式，改用确定性不变量「stored 行盒的 font_size 与 paint 读取的 font_size 必须同源（都来自真实 styles）」。
- **优先级**：必须
- **验收场景**：
  - 场景（正常）：所有过 Gate 1 的容器（含多行非-Ahem、block-child 直接文本）都存 inline_layout。验证：LAYOUT_DUMP 确认 inner-div 的 inline_layout 非空。
  - 场景（异常）：multicol 容器（Gate 1 显式跳过 `is_multicol`）不存，paint 走现有 column 重跑。验证：multicol 类目 count 不变。

### FR-004：零 count 回归硬门禁
- **描述**：每个 Phase 必须以全量 `make reftest`（loose 438/490）+ strict（296/490）+ chromium-Oracle 抽样 三态不退为合并条件。
- **优先级**：必须
- **验收场景**：见 §10 验证策略。

---

## 5. 约束与假设

### 5.1 必须约束（Must）
- 任何 Phase 落地前 `make test` 全绿、`cargo clippy --workspace --all-targets -D warnings` 干净。
- 触及 `engine.rs`（3969 行）/ `text.rs` 的修改须同步评估 2000 行拆分（§7.2）。
- 修改「禁止修改」路径须停止并说明。

### 5.2 禁止约束（Must Not）
- 不允许以放宽容差掩盖 large-font 回归（DC-14 容差锁定）。
- 不允许 paint 对 glyph 做改变布局语义的整行重排（goal DC-13）。
- 不允许引入新的 `#[ignore]`（除 real_website_compat.rs）。

### 5.3 已定决策
- 复用 `compute_final_inline_layouts` + `paint_text` 现有架构，不重写 IFC 引擎。
- 字体度量优先复用 fontdue 现有 ascent（R188 标记的「font-family→FontId 解析在 paint 懒做」是独立阻塞，本设计在 layout 侧用 fragment 已有的 font_size 近似 ascent，**不引入 FontLoader 全量预解析**——那是更大独立 RFC）。

### 5.4 假设
- **A1（待验证）**：fontdue 能在 layout IFC 阶段为每个 fragment 提供 ascent（或可用 font_size 近似）。— 状态：待验证（Phase 2 探针）。
- **A2（待验证）**：multicol-fill-auto 回归的真正机制是「被存 inline_layout 的内层容器改变了 paint 分支」而非 font_size map 变化。— 状态：待验证（Phase 3 探针）。
- **A3**：R207 narrow 子集（pure-inline 叶文本）在新语义下仍 PASS（baseline-resolved 对单行 Ahem 退化到旧 v_offset=0）。— 状态：需 Phase 1 验证。

### 5.5 代码变更边界
- **允许修改**：`crates/layout-engine/src/engine.rs`、`crates/layout-engine/src/types/mod.rs`、`crates/engine/src/paint/painter/text.rs`、`crates/layout-engine/src/inline/mod.rs`（仅 fragment 字段）。
- **禁止修改**：`crates/taffy-local/**`（vendored，R304 DEFER）、`crates/render-foundation/**`（渲染器，与 IFC 语义无关）、`tests/wpt-runner/**`（reftest harness）。

---

## 6. 详细设计（RFC）

### 6.1 目标状态架构

```
compute_final_inline_layouts (真实 styles IFC)
        │  对所有过 Gate 1 容器存 inline_layout
        │  每行存 (line_top, baseline_y, line_height)
        │  每片段存 (x, y=baseline_y, width, font_size, ...)
        ▼
paint_text
   use_stored = inline_layout.is_some() && width_matches   ← 删除 !multicol 例外
        │  （multicol 见 §6.4 单独处理）
        ▼
   Path A 唯一路径：渲染 stored fragments，glyph y = baseline_y
   Path B（空 styles 重跑）—— 仅 Gate 1 显式跳过的容器（flex/grid/table）保留
```

**核心变更**：`InlineLayoutFragment.y` 语义从「fragment 框顶部」改为「已解析基线 y」（或新增 `baseline_y` 字段保留 `y` 兼容），paint 直接用，删除 `is_ahem ? 0 : font_size` 推断。

### 6.2 数据模型变更

```rust
// crates/layout-engine/src/types/mod.rs
pub struct InlineLayoutLine {
    pub y: f32,            // 行盒顶部（保留）
    pub height: f32,
    pub baseline_y: f32,   // 【新增】该行基线绝对 y = line.y + ascent
    pub fragments: Vec<InlineLayoutFragment>,
}
pub struct InlineLayoutFragment {
    pub x: f32,
    pub y: f32,            // 保留（fragment 框顶）
    pub baseline_y: f32,   // 【新增】片段基线绝对 y（单行时 = line.baseline_y）
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub is_ahem: bool,
    pub text: String,
    pub node_id: Option<NodeId>,
}
```

**实现来源**：ascent 由 IFC 已计算的 `line.height` + `frag.font_size` 推导（Ahem: ascent=font_size→baseline_y=line.y+font_size；普通字体 ascent≈font_size×0.8 近似，A1 待 Phase 2 用 fontdue 精确 ascent 替换）。

### 6.3 Gate 重构

- **Gate 1**（保留 + 微扩）：维持 R207 narrow 条件 + block-child 直接文本（line 1710 路径）。
- **Gate 2**（删除）：移除 `lines.len()<=1 && is_pure_ahem` 启发式，所有过 Gate 1 容器都存。
- **paint use_stored**：移除 `multicol_info.is_none()` 例外（multicol 改用 §6.4）。

### 6.3A 关键发现（R305 执行期补充）：glyph.y 是基线 + frag.y/offset 耦合

执行 R305 期间精读耦合链发现**原 Phase 1「加 baseline_y 字段」前提不稳**，须先实证：

1. **`GlyphPrimitive.y` = 基线 y**（非 bitmap 顶部）。证据：`glyph_top_left(x, baseline_y, x_offset, y_offset, height) = (x+x_offset, baseline_y - y_offset - height)`（cpu/mod.rs:33-34）——渲染器把 `glyph.y` 当 baseline，bitmap top = baseline − y_offset − height。
2. paint 计算 `glyph_y = content_y + frag.y + offset`（text.rs:1132），故 **`frag.y + offset = 基线`**。
3. types/mod.rs 文档注释声明 **`基线 = frag.y + height`**（InlineLayoutFragment.y 注释）。
4. 但 Path A 用 `offset = is_ahem ? 0 : font_size`（text.rs:1208），Path B 用 `offset = baseline_fs`（=font_size）。

**矛盾**：若基线 = frag.y + height 成立，则 offset 应恒为 `height`；但 Path A Ahem 用 0、Path A/B 普通用 font_size。font-051（单行 100px Ahem）Path A offset=0 仍 PASS，证明 stored Ahem 的 `frag.y` 已被 IFC 定位到「offset=0 即基线」的位置——即 stored frag.y 语义随 is_ahem / 路径 CO-DESIGN，非单一「fragment-box-top」。

**结论**：frag.y / offset / glyph.y 三者构成**经验性自洽耦合**（单行 Ahem 子集成立），无法仅靠读码推导多行非-Ahem 的正确 offset。**原 Phase 1「加 baseline_y 死字段」改为 Phase 0「实测探针」**——在确证「frag.y+height == 实际 glyph 基线」前不引入字段，避免在错误前提上叠加。这把原 P1 推后，P2-P5 顺延，但保证不在 shaky 前提上编码（code-guidelines「先思考再编码，不假设」）。

### 6.3B Phase 0 探针实证裁决（R306，证伪 §6.3A 假设）

R306 执行 Phase 0 探针（env `PHASEA_BL=1` 把 stored Path A 的 `v_offset` 从 `is_ahem?0:font_size` 改为文档化 `frag.height`，对 font-051 A/B 实测）：

| 模式 | font-051 | 裁决 |
|------|----------|------|
| 默认 `v_offset = is_ahem?0:font_size` | **0.00% PASS** | 当前 offset load-bearing 正确 |
| 探针 `v_offset = frag.height` | **16.67% FAIL**（80000/480000 px，max ch 255） | 「frag.y+height」**渲染错误** |

**裁决：§6.3A 假设「geometric baseline（frag.y+height）可作 render baseline」证伪。**

- types/mod.rs:387「基线 = frag.y + height」是 IFC 的**几何基线**（apply_vertical_alignment `run.y = baseline_y - run.height` 推导成立），但 fontdue 光栅化时 `GlyphPrimitive.y`（被 cpu/mod.rs:33 当 baseline）+ fontdue glyph 度量的组合，使 **offset=0（非几何 height）** 产出与 chromium 一致的位图。geometric baseline ≠ fontdue render baseline，差一个 fontdue-metric-dependent 常量。
- **stored Path A 的 `else { frag.font_size }` 分支是死代码**：Gate 2（engine.rs:1910 `is_pure_ahem`）保证 stored 片段 `is_ahem` 恒 true → `v_offset` 恒 0。
- 若 `baseline_y` 字段存几何基线（frag.y+height），paint 直接用会**重演 16.67% 错误**（破坏 R207 子集 font-051 等）。

**对实施计划的影响（§0/§7.1 Phase 1 重定向）**：
- 原 Phase 1「paint Path A 改用 frag.y+height / 加 baseline_y=几何基线」**作废**。
- 真正杠杆 = **Gate 2 放宽覆盖多行/非纯-Ahem**（让更多容器进 stored，offset 校准 is_ahem?0:font_size 不动），即 R209（PHASEA_MULTILINE）已试方向，被墙②（multicol）+ 换行精度阻塞。
- offset 语义**不是** Phase A 阻塞点（Path A offset 对 stored Ahem 已正确）。Phase A 硬阻塞 = 墙② multicol + 换行精度，与本设计原「offset/baseline 统一」重心不同。
- 可行统一方向（替代 baseline_y 字段）：(A) 存「render glyph_y」（compute_final 用同款 offset 校准算出，paint 直接消费，绕过语义分歧）；或 (B) 保留 paint 端 offset 校准，统一靠 Gate 2 放宽。两方向均不引入「几何 baseline_y 作 render y」。

### 6.4 multicol 处理（解墙 ②）

两种方案，Phase 3 探针后定：

- **方案 M1（推荐）**：multicol 内层容器**照常存** inline_layout；paint multicol 路径消费 stored fragments 做列分配（而非重跑 IFC）。需先实证 A2（回归机制）。
- **方案 M2（保守 fallback）**：multicol 内层容器保持现状（不存，paint 重跑），但确保其 font_size 来自真实 styles 而非 16px——即把 Path B 的空 styles 改为「仅 multicol 路径」用真实 font_size（解 large-font 但保留 column 重跑）。

**最终选择**：Phase 3 探针后定，倾向 M1（与全局消灭 Path B 一致）。

### 6.5 影响范围分析

| 影响项 | 程度 | 说明 |
|--------|------|------|
| engine.rs compute_final | 高 | Gate 2 删除 + baseline_y 存储 |
| text.rs paint_text | 高 | Path A 唯一化 + baseline_y 消费 |
| types/mod.rs | 中 | 两结构加字段 |
| inline/mod.rs | 低 | fragment 产 baseline_y |
| engine.rs 文件行数 | 中 | 现 3969 行，本设计净增 ~50 行，须拆分（§7.2） |

---

## 7. 实施交接

### 7.1 推荐修改顺序（6 个 Phase，每 Phase 独立可合并）

0. **Phase 0（实测 glyph 基线耦合探针，read-only）**【R305 执行期新增，前置】：用 `LAYOUT_DUMP=1` + 临时 glyph 位置插桩，对 (a) 单行 Ahem（font-051）、(b) 多行 Ahem（ifc-008）、(c) 多行非-Ahem 三个用例实测 `frag.y`、`frag.height`、`frag.font_size`、实际 `glyph.y`（基线），确证「`frag.y + height == glyph.y`」是否普适成立，以及 Path A 的 `is_ahem?0:font_size` offset 在哪种 frag.y 语义下自洽。→ 验证：产出探针报告，无代码变更，决定 Phase 2 用「现有 frag.y+height」还是需新增字段。
1. **Phase 1（baseline-resolved 渲染，Gate 2 不变）**：据 Phase 0 结论，paint Path A 改用正确基线（`frag.y + height` 或新增 baseline_y），Gate 2 仍 single-line pure-Ahem。→ 验证：font-051 等 R207 子集仍 PASS（A3 验证退化正确）。
2. **Phase 2（探针 multicol 墙 ②）**：read-only 探针实证 A2（multicol-fill-auto 回归机制），定 M1/M2。→ 验证：产出探针报告，无代码变更。
3. **Phase 3（删除 Gate 2 多行限制）**：放宽 Gate 2 到所有过 Gate 1 容器 + multicol 按 Phase 2 方案。→ 验证：ifc-008/009/011 改善，multicol 类目不退，large-font 簇 chromium-Oracle 下降。
4. **Phase 4（收尾清理）**：删除 Path B 中已无消费者的空 styles 重跑代码（仅 flex/grid/table 保留）。→ 验证：全量三态不退 + clippy/fmt 干净。
5. **Phase 5（文件拆分）**：engine.rs 拆分（3969 行 → 抽 inline_finalization.rs）。→ 验证：纯移动不改逻辑，全量三态不退。

### 7.2 文件拆分（2000 行约束）

`engine.rs`（3969 行）须拆分。建议按职责：`compute_final_inline_layouts` + `store_font_sizes_from_ifc` + `remeasure_*` 抽到 `crates/layout-engine/src/inline_finalization.rs`（~400 行）。Phase 5 执行，避免与逻辑改动混在一个 commit。

### 7.3 首批提交建议

| 批次 | 范围 | 预期结果 | 验证 |
|------|------|----------|------|
| Phase 0 | LAYOUT_DUMP + glyph 位置插桩探针 | 探针报告（无代码落地） | 三用例实测 frag.y/height/glyph.y |
| Phase 1 | paint Path A 改用 `frag.y+height` 基线 | R207 子集仍 PASS | `make reftest` loose 438/490 |

---

## 8. 回归风险与缓解

| 风险 | 概率 | 缓解 |
|------|------|------|
| baseline_y 对普通字体 ascent 近似不准 → 文本类大面积漂移 | 高 | Phase 2 先在 Ahem 子集验证退化正确；普通字体分阶段，每类目 set-diff |
| multicol 墙 ② 未解 → 放宽 Gate 2 仍回归 | 中 | Phase 3 探针先行，Phase 4 用 M2 保守 fallback 兜底 |
| Path B 删除后 flex/grid/table 渲染变化 | 低 | Phase 5 保留 flex/grid/table 的重跑分支，仅删 inline 类消费者 |
| engine.rs 拆分引入编译/测试断裂 | 低 | Phase 5 单独 commit，纯移动不改逻辑 |

---

## 9. 验证策略

- **单元测试**：每个 Phase 新增 fragment baseline_y 计算单测（`baseline_y_ahem_equals_line_top_plus_font_size`、`baseline_y_fallback_uses_font_size`）。
- **reftest**：每 Phase 跑全量 `make reftest`（loose 438/490 不退）+ `ZERO_REFTEST_STRICT=1 make reftest`（strict 296/490 不退）。
- **chromium-Oracle**：每 Phase 跑 `scripts/cross-validate.py` 抽样，确认 large-font 簇 z_vs_chr 下降、其他类目污染率不升。
- **回滚**：每 Phase 独立 commit，任一 Phase 三态退步即 `git revert` 该 commit，不污染前序 Phase。

---

## 10. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要 | ✅ Pass | §0 存在，含目标/范围/约束/方案/首步 |
| 场景存在性 | ✅ Pass | FR-001~004 各有 ≥1 验收场景 |
| 异常路径覆盖 | ✅ Pass | 每 FR 含正常+异常场景（回退/不变） |
| 测试绑定 | ✅ Pass | 每场景绑 `make reftest`/单测函数名 |
| TBD 清零 | ⚠️ Warning | A1/A2 待 Phase 2/3 探针验证（非阻塞，已降级为假设） |
| 实施交接 | ✅ Pass | §7 含文件清单、修改顺序、首批提交 |
| 首步可执行性 | ✅ Pass | §7.1 Phase 0（read-only 探针）+ §7.3 首批 |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ Pass | FR 用「必须消费/禁止重跑/必须携带」具体动词 |
| 非确定性措辞 | ✅ Pass | 无「应该/可能/尽量」（§6.4「倾向 M1」标注为待定决策非需求） |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §3 在范围/不在范围无交集 |
| 方案漂移 | ✅ Pass | §6 设计未引入与 §5.2 Must Not 冲突的依赖 |
| 代码边界完备 | ✅ Pass | §5.5 允许/禁止修改均声明 |
| 实现来源闭合 | ✅ Pass | §6.2 ascent 来源（IFC 已计算值 + fontdue 待 A1）已写 |

**汇总**：14 Pass / 1 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → 允许进入实施（本轮为设计，下一轮 R306 起 Phase 1）

---

## 11. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-06-18 | R305 初始 read-only 设计产出 |
| v1.1 | 2026-06-18 | R305 执行期补充 §6.3A：确证 GlyphPrimitive.y=基线，frag.y/offset/glyph.y 经验性耦合，原 Phase 1「加 baseline_y 字段」改为 Phase 0 实测探针前置；Phase 计划由 5 改 6（插入 Phase 0，顺延） |
| v1.2 | 2026-06-19 | R306 Phase 0 探针实证（§6.3B）：font-051 A/B 证伪「geometric baseline 可作 render baseline」（frag.height offset → 16.67% FAIL，默认 offset=0 → 0.00% PASS）。原 Phase 1（baseline_y=几何基线）作废；Phase 1 重定向为 Gate 2 放宽（offset 校准不动）。offset 语义非阻塞点；真硬阻塞 = 墙② multicol + 换行精度 |
| v1.3 | 2026-06-30 | R848-R883 经验性精化（§12 新增）：R848 测绘 4 处 IFC 消费点；R876 三方补偿根因；R877 真路径 = non-stored render_fragment!；4 次 net-negative 先例（R834/R836/R849/R875）；TextFragment 已有 baseline 字段；R639 incremental 已证 + font-bridge 剩余 prerequisites |

---

## 12. R848-R883 经验性精化（v1.3，实施前必读）

> 以下为 R305/R306 设计之后，R848-R883 多轮实证对 Phase A 的关键精化。**实施 Phase A 前必读**——这些发现重定义了「首步可执行 slice」与「墙」的精确位置。

### 12.1 R848：4 处 IFC 消费点测绘
IFC 结果被 4 处消费（painter/text.rs）：
1. **stored path**（text.rs:1308 `if use_stored`，`stored_fragments`）——仅对 **pure-Ahem 块**存储（R84/R829 条件），用 `frag.baseline_y_abs`。
2. **non-stored path**（text.rs:1349 `for fragment in fragments.iter()`）——真实非-Ahem 文本（welcome/morning/linebox 非-Ahem）走此路径，用 `render_fragment!` 宏，`$baseline_offset = baseline_fs(font_size)`。
3. **Path B**（all_fragments 多行 y）——R630/R632 已修（多行 y 堆叠 + line-height override）。
4. **multicol** IFC（text.rs:933 列分配）。

### 12.2 R876：三方补偿根因（welcome 16.11% 平衡机制）
welcome/morning line-metric 残余由 **三项互相补偿** 凑成当前平衡：
- **① strut baseline** 用 0.8（inline/mod.rs:1571/1573/1583 `* 0.8`）；CSS 真实 ascent ≈ 0.928em（fontdue line_metrics ascent/em=0.928，line_gap=0）。ZW baseline 比 chromium 高 0.028fs。
- **② paint v_offset** 用 `font_size`（text.rs:1359 `$baseline_offset = baseline_fs`）；应使 glyph 绝对位 = baseline − ink_ascent。fontdue tight-ink 'H' height=30（40px 时）vs font_size=40，差 10px → glyph 多上移 10px。
- **③ tight-ink vs ascent** 7px 差（fontdue 测字身实高 30 vs CSS ascent 37）。

**关键**：改任一项单独 → 平衡破裂 → welcome 退步（R834 改 ① strut 0.8→0.928 welcome +0.07pp；R836/R849/R875 同）。**安全修复须三方同改**：① strut 用真实 ascent + ② paint v_offset 用 ink-height + ③ 验 stored/Path B/raster glyph_top_left 不双计。

### 12.3 R877：真路径 = non-stored render_fragment!
R877 实证：改 **stored path** 对真实非-Ahem 文本**零效果**（stored path 仅 pure-Ahem 块触发，其非-Ahem else 分支几乎从不执行——R876 实测 welcome/linebox/sans-serif 探针字节同）。**真路径 = non-stored path**（text.rs:1349 循环，`render_fragment!` 宏 $baseline_offset at ~1359）。

`TextFragment`（inline_types.rs:174）**已有 `baseline` 字段**（line 208，「从片段顶部到基线的距离」）+ `height`（line 182）。R877 原判「需加 line_height/baseline_y 字段」——`baseline` 已存在，**须核实 non-stored path 能否消费 `fragment.baseline` 替代 `font_size` 作 $baseline_offset**（可能已是 sufficient plumbing，无须新字段）。

### 12.4 4 次 net-negative 先例（勿单点重试）
R834（strut 0.8→0.928）/ R836 / R849 / R875：单点改 strut 或 v_offset 均 welcome net-negative。**R882 确认单 session clean win 脉络已挖尽**，Phase A 三方协调是多 session 任务。

### 12.5 R639：incremental 已证 + font-bridge 剩余
R639 实证 Phase A **非「全有或全无」**：per-fragment inline-bg（+13）/ per-fragment color（R358）/ 跨 block float 侵入（R362）各独立 LANDED。**剩余 prerequisites = 真实 font 度量 bridge**（IFC↔FontLoader 接口，暴露 fontdue line_metrics 真实 ascent/descent/ink-height）——当前 IFC 用硬编码 0.8 / estimate_char_width 启发式。

### 12.6 首步可执行 slice（v1.3 推荐）
1. **✅ LANDED R885（commit `d5b7e3ae`，零行为变更 enabling infra）**：`crates/layout-engine/src/inline/font_metrics.rs` 新模块 = `LineMetrics` + `FontMetricProvider` trait + `impl FontMetricProvider for FontLoader`（family→font_id→fontdue `line_metrics_full`→ascent/descent/line_gap）+ `InlineFormattingContext.font_metric_provider`（默认 `None`，dormant）+ builder。在 IFC 注入 font-metric 查询，暴露 per-font ascent/descent/line_gap（ink-height 见 step-2 concept ②，须 render-foundation 暴露 glyph ink bounds）。**仅添加接口，不改 0.8 常数 → 零回归**（grep 证 0 production reads；make test 全绿 incl. 4 新 font_metrics 测试断言真实 Ahem 度量；product-smoke welcome 16.11%≈baseline 16.16%）。
2. **三方同改 narrow slice**（消费 bridge）：strut 用 real ascent + non-stored v_offset 用 ink-height（或先试 `fragment.baseline` 替代 font_size）+ 验双路径不双计。**三态门禁 A/B**：welcome <20% + linebox/css-text/normal-flow oracle 零回归 + self-source 通过率不降。净负即回退（5th data point）。
3. 守住 multicol-fill-auto 反向依赖（R198 墙）+ pre-wrap 宽度敏感（R627 -15）。

---

## 13. v1.5 addendum（R2605 会话，2026-08-04，design-vs-code 同步核验）

> 承接 v1.3/v1.4。R2605 sanctioned design 调研（「Phase A 只做设计后实施」，不自主开工实现）核验 IFC line-box-metric 当前代码态 vs 本设计文档，发现 **v1.3 §12.6 step-1 关于 R885 bridge 的「dormant」描述已 stale**——bridge 经多轮演进已激活并消费。本 addendum 同步代码现实 + 锐化剩余切片 readiness，供用户授权实施时无误导。**不改变 master.md 控制面的 user-gate**（IFC line-box-metric 仍 user-gated，repeats R834/R836/R849/R875 net-negative 史）。

### 13.1 R885 font_metrics.rs bridge 状态订正：dormant → 已激活

v1.3 §12.6 step-1 称 R885（`font_metrics.rs`）「默认 None，dormant，零回归」。**R2605 核验此描述 stale**：
- `crates/engine/src/pipeline/mod.rs:270-280` `set_font_metric_map`（U1b-wiring）= 生产激活路径：从 `FontLoader::build_line_metric_map()` 构建 per-family 行度量 `FontMetricMap` provider，注入 LayoutEngine，经 compute_final_inline_layouts + measure_text_content 双路径触达 IFC。
- bridge 经 `inline_finalization.rs` ~12 处调用点（614/666/902-903/1125/1151/1325-1326/1376-1377/1442/1516-1517/1559）+ `engine.rs`（173/195/267-268/351/459）触达。
- `crates/layout-engine/src/inline/text_metrics.rs:409 resolve_normal_line_height` **真消费 provider**：`Some(p) && Some(m) = p.line_metrics(...) → return m.ascent - m.descent + m.line_gap`（fontdue/chromium 真实 hhea），仅 provider 缺省/无法解析时 fallback 常数比率（Ahem 1.0 / 非-Ahem 1.164）。
- 即 **line-height:normal 已走 per-family 真实度量**（非 dormant）。

### 13.2 三方补偿三因素当前态（R876 §12.2 重核）

| 因素 | v1.3 描述 | R2605 当前态 | 证据 |
|------|-----------|--------------|------|
| ① strut baseline | 0.8 启发式（应真实 ascent） | **half-leading + dominant_fs × ascent_ratio**；ascent_ratio = R1004 `ascent_ratio_overrides[node]`（**dormant 空 map，从未 populate**）→ 回退 R990 is_ahem-gated（Ahem 0.8 / 非-Ahem 0.928）。**R1004 map = step-2 精确注入点**（从 live provider populate per-node real ratio） | `inline/mod.rs:215 ascent_ratio_overrides: HashMap<NodeId,f32>`（default empty `:271`）/ `:1228-1244 ascent_ratio_lookup`（dormant override → R990 fallback）/ R990 常数在 `ascent_ratio_lookup`（Ahem 0.8/非-Ahem 0.928）；注 `:1201`「line_height*0.8」**注释 stale**（旧 flat 描述，实际代码已 R990+R1004） |
| ② paint v_offset | font_size（应 ink-height） | 未变更（`render_fragment!` `$baseline_offset = baseline_fs(font_size)`，`text.rs:1100-1332`） | `text.rs` |
| ③ tight-ink vs ascent | ~7px 差 | 未变更 | R876 §12.2 |

**结论：三方协调（v1.3 §12.6 step-2）仍未实施**——strut ascent 走 R990 is_ahem-gated 常数（R1004 `ascent_ratio_overrides` dormant 空 map 未 populate），`font_metric_provider` 经 line-height:normal 已 live 但未注入 strut。bridge 已 live + R1004 注入点 purpose-built，故 step-2 的 provider-plumbing 风险较 v1.3 假设更低（provider 已 proven zero-regression at line-height:normal level）。

### 13.3 v1.3 后附加既 land 工作

- **R1192 font-size-adjust apply**（`text_metrics.rs:362-378`，is_ahem-gated narrow slice，CSS Fonts 3 §3.6）：`adjusted_size = font_size × adjust / aspect`（aspect = Ahem 0.8 常数），adjusted font_size 经 line-height + advance + paint 全链路传播。非 Ahem defer（须 OS/2 sxHeight 派生 + font 接入 layout = 同 Phase A 字体度量架构 gap，Slice 3+）。
- **R990 is_ahem-gated 常数**（Ahem 0.8 / 非-Ahem 0.928，`inline/mod.rs:212/424`）：strut/normal fallback 比率。
- `TextFragment.baseline`（`inline_types.rs:132`）+ `baseline_y`（:173/:216）字段已存在（R877 §12.3 既证），non-stored path 消费 `fragment.baseline` 替 font_size 作 `$baseline_offset` 的 plumbing 已就绪。

### 13.4 剩余切片 readiness（锐化）

v1.3 §12.6 step-2「三方同改 narrow slice」**仍是精确下一步**，且 readiness 提升：
- provider 已激活 + proven（line-height:normal 零回归）→ step-2 须做 = 「populate R1004 `ascent_ratio_overrides` map（从 live provider per-node real ascent ratio，使 `apply_vertical_alignment` strut ascent 走真实度量替 R990 常数）+ non-stored `v_offset` 消费 ink-height 或 `fragment.baseline`（替 font_size）+ 验 stored/Path B/raster glyph_top_left 不双计」，三方同改 + kill-switch（`ZW_` env，default-off）+ 三态 A/B（welcome <20% + linebox/css-text/normal-flow oracle 零回归 + self-source 不降），净负即回退。**R1004 `ascent_ratio_overrides` 是 purpose-built dormant 注入点**（mod.rs:215/405 `with_ascent_ratio_overrides` builder 已就绪），无需新基建。
- 与 v1.4 R109 匿名块 manifestation 的关系：v1.4「匿名块须从叶子近似升级为跑 IFC 的真容器」是 tree-build + IFC ownership 深构造（墙 ③/Path A/B 同根，deadlock 史 R125/R206/R213）；本 §13 step-2 是更窄的「strut/v_offset 度量三方协调」slice，**不触及 tree-build / 匿名块**，是 v1.4 整体 unification 的一个可独立先行的度量层子切片。
- 仍受 master.md 控制面 user-gate——**本 addendum 仅同步 design-vs-code 使授权实施时无误导，不自主开工**。



