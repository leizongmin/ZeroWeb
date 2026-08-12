# 归档：R314–R334 详细记录

> 从 `docs/goal/rendering-compat/master.md`「最近轮次详细记录」迁出（R354 归档，保持最近窗口 ≤20 轮：R335–R354 留 master，R314–R334 迁此）。
> 含 R314、R315、R316、R317、R318、R319、R320、R321、R322、R323、R324、R325、R326、R327、R328、R329、R330、R331、R332、R334 共 20 轮（R313/R333 无独立条目）。
> 当前状态以 `master.md` 顶部「综合裁决」与「最近轮次详细记录」为准；本文件为历史记录，只追加。

---

### R314 — 综合 plateau 确认 + 全量基线复验 + latent line-height% defer（read-only 核查，基线 loose 438/490 / strict 295/490 持平；已飞书通知卡点）

**承接**：R313 baseline-overrides lever 证伪后，本轮做「单会话 clean win 是否真枯竭」的最终核查 + 全量基线复验。

**核查 1 — 全量 reftest 基线复验**：`make reftest`（test-guard 包裹）全量 490 → **438/490 (89.4%)**，与 R308 后基线一致，R309-R313 docs-only 提交**零漂移**（DC-7 卫生确认，代码状态 = R308 verified-green）。

**核查 2 — multicol gate 放宽是否可重试**：text.rs:709-711 代码注释**明示**「明确高度 balance 容器涉及 column-breaking，简单均衡分配会回归→回退单块」。R157（净中性）+ R203（净负）+ 本注释三重确认：paint 侧 gate 放宽/协调**已知回归**，重试必重复失败。真修复 = layout 侧 column-aware IFC（R131，major multi-session 架构）。

**核查 3 — DC-9 blend_mode 杠杆**：grep wpt-data **仅 3 文件**用 mix-blend-mode/isolation（R303「近零覆盖」确认）。实现 blend_mode（需 paint-isolation 架构，R278 defer）= **零 reftest 影响 + 高成本**，非 reftest 杠杆。

**核查 4 — latent `line-height: <percentage>` bug（R313 附发现）defer 确认**：computed.rs:195-206 未解析 line-height Percentage（同 R308 font-size% 谱系）。grep **0 个 reftest + 0 个产品 fixture（apps/）** 用 line-height % → 零覆盖零消费者，按 code-guidelines「不实现需求之外的功能」**defer**（与 R308 不同——R308 有 anonymous-inline-inherit 驱动，line-height% 无）。

**综合裁决（R305-R313 九轮收敛）**：
- **三条 clean-win 搜索策略全穷尽**：near-pass 聚类（R307，26 案全结构性/字体噪声）、POLLUTED 逐项（R309，唯一 win=R308 font-size%）、fresh chromium-Oracle cross-validate（R311，4 新候选全 ruled out）。
- **四条结构轨均证非单会话可解**：Phase A IFC 统一（墙②③，R125/R198/R205/R206/R209/R213 六轮死锁 + R306 几何基线证伪）、multicol column-aware IFC（R131，paint 侧 R157/R203 + 本轮注释三重证回归，layout 侧 major 架构）、baseline-export（R310 multicol=None / R312 inline-flex=错值 / R313 baseline_overrides=无效，三轮探针）、DC-9 blend_mode（0 覆盖）。
- **单会话 clean win 真枯竭**——剩余 forward motion 需**多会话架构承诺**一条轨。

**飞书通知**：已按 run-rules 以应用机器人身份向本人发送卡点告知（message_id om_x100b6c7...），说明 plateau 现状 + loose 438/strict 295/chr~36% + 建议多会话攻坚或接受现状。通知仅为告知，不阻塞后续。

**本轮 read-only 核查**：零代码变更。基线 loose 438/490 / strict 295/490 / chromium-Oracle ~48.2% 污染持平。next = 待用户对多会话结构攻坚的决策；若继续 rally，最高杠杆轨 = multicol layout 侧 column-aware IFC（R131，17+ 失败聚类），但需 multi-session spec-rfc + 实施承诺，非单会话。

### R315 — self-fail 集第 4 条搜索路径：plateau 再确认（read-only 实证，基线 loose 438/490 / strict 295/490 持平）

**承接**：R314 综合 plateau 确认后，本轮取**全新角度**——52 个 SELF-FAIL 用例（loose 失败、真实 +1 reftest 计数目标，区别于此前 strict near-pass 与 POLLUTED 候选）。逐个 probe 5 个非已知聚类候选，全部确认为结构性/特性缺口：
- `child-border-box-and-max-content-002`（1.22%）= taffy grid intrinsic-sizing（fit-content 轨道 + box-sizing:border-box，R304 DEFER taffy 升级）。
- `border-padding-bleed-001`（2.40%）= inline line-box 绘制顺序（结构性）。
- `border-bottom-width-006`（2.86%）= height:0+border 的 inline-block 基线（R180/R266 结构域）。
- `multicol-clip-001`（0.56%）= multicol 溢出裁剪 + Ahem（结构性聚类）。
- `float-nowrap-hyphen-rewind-1`（2.92%）= `hyphens:auto` 特性缺口（需语言级连字算法）。

**裁决**：self-fail 集成为第 4 条 clean-win 搜索路径（near-pass R307 / POLLUTED R309 / fresh-xval R311 / self-fail R315）穷尽确认枯竭。零代码变更，基线持平。

### R316 — baseline-export flex-baseline 后处理：实现 + 实验证伪（code attempt + revert，基线 loose 438/490 持平）

**承接**：R310/R312/R313 探针把 baseline-export（baseline-000~008 + flexbox-baseline 聚类）根因定位为「flex 项缺 first baseline」，但仅测了 inline-flex（R313 证 baseline_overrides 无效）与 field-fill（R266 证净 0）。**block-flex + multicol 项的后处理路径此前未实测**——本轮实现并实验裁决。

**前置核查（证 R304 DEFER 正确 + line-height% defer 正确）**：
- taffy 0.7.7 vendored `Style` 结构**无 `baseline_overrides` 字段**（0.8+ 才有）→ 「设 baseline_override + 重 layout」两趟路径不可用，ZeroWeb 后处理是唯一路径。
- `line-height: <percentage>` 在 computed.rs 未解析（同 R308 font-size% 谱系）；grep 实测 **0 reftest + 0 产品 fixture** 用 line-height% → 零覆盖零消费者，R314 defer 正确。

**实现（engine.rs，已 100% 回退）**：新增三函数 + compute() step 10.7 调用——
- `resolve_font_size_px`：ComputedStyle.font_size→px（em/rem 按 16px root）。
- `synthesize_first_baseline(box, styles)`：递归合成盒 first baseline（相对自身 border-box 顶）：优先 taffy 缓存基线；否则递归首个 in-flow 子元素（child.y 已是该盒 border-box 相对，累加）；基情形叶盒用 font-size 近似 ascent（content 顶部 + font-size）。坐标系与 painter 累积（`offset_y + box.y`）一致。
- `adjust_flex_baseline_alignment`：对 `display:flex|inline-flex` + `align-items:baseline` 容器，对 `taffy_baseline=None` 的流内项，按 `desired_y = target - local_b` 重定位（**只改 item.y，子树经 painter 累积自动跟随**）。

**FLEXBL_PROBE 实测 baseline-003（flex > "PA" 文本 + columns:3 multicol > "SS"）**：
- 容器 node 17v1 taffy_baseline=**Some(19.2)**；item[0] "PA"(18v1) 与 item[1] multicol(19v1) **均 y=0 h=19 taffy_baseline=None**；multicol synth=Some(16.0)，"PA" synth=None（匿名文本项无 style/无 LayoutBox 子）。
- 关键：**两 item 已被 taffy 基线对齐**（同 y=0/h=19，内容同字号）。1.1% chromium diff 不在基线对齐，在别处（multicol 列结构/font）。

**两种 target 源均失败（决定性证伪）**：
| target 源 | 结果 |
|-----------|------|
| 兄弟项派生（`max(sibling.y+sibling.taffy_baseline)`） | baseline-003 两 item 均 None→target=None→**no-op**（z_vs_chr 1.118% 不变，证 R310 的 1.058% 即未修状态） |
| 容器 taffy_baseline（19.2） | 触发但**回归**：multicol 子集 40/57→**38/57**（baseline-001 0.52→3.15%、baseline-002 0.00→3.50% 翻 FAIL），因把已对齐项错误下移 3.2px |

**裁决：baseline-export 经 flex-baseline 后处理不可解**——block-flex 项已被 taffy 正确对齐（fallback 对 baseline-001/002 足够），强行重定位破坏已绿用例。这是 baseline-export 杠杆的**第 3 种独立机制证伪**（R266 field-fill 净 0 / R313 baseline_overrides 对 inline-flex 无效 / **R316 flex 后处理对 block-flex 回归**）。三种机制覆盖 field-fill、inline-flex 后处理、block-flex 后处理全谱，baseline-export 从 ZeroWeb 后处理侧穷尽。

**代码状态**：env-gated 探针 + 实现代码**已 100% 回退**（`git checkout engine.rs`，`git diff --stat` 空）；`cargo check -p zero-layout-engine` 干净；`make reftest` 内置 686/686 全绿（DC-7 卫生确认，回退 byte-identical HEAD）。零代码变更落地，基线 loose 438/490 持平。

**对优先级队列影响**：baseline-export（baseline-000~008 + flexbox-baseline）经三轮探针 + 本轮实现共四轮，**从 ZeroWeb 后处理侧彻底 ruled out**——真修复须 taffy inline-level-box 基线合成或升级 taffy（0.8+ baseline_overrides，R304 DEFER prohibitive）。剩余 forward motion 确认为：① multicol layout 侧 column-aware IFC（R131，major 架构）；② Phase A IFC 统一（墙②③）；③ DC-9 blend_mode backdrop（0 reftest 覆盖）；④ DC-13 残余。均非单会话 clean win。本轮价值 = 以真实实现（非推断）排除 flex 后处理这条未测路径，防止后续轮重试。

### R317 — multicol breaking paint 门控放宽：实现 + 实验证伪（code attempt + revert，基线 loose 438/490 持平）

**承接**：R316 排除 baseline-export 后，转向 multicol column-aware IFC（R131，最大失败聚类）的最具体 paint 侧 wiring 候选——text.rs:713 `height_auto` 门控。设计文档 R201 Round 4' 把 multicol-breaking 的阻塞点 A 定为「paint 门控 `height_auto` 挡住有明确高度 inner 的列分布」，但 R203 称 paint 侧协调 net-negative。两者矛盾**未经单点实验裁决**——本轮实现并实验。

**实现（text.rs:713，已 100% 回退）**：把 `if !has_in_flow_children && is_balance_mode && height_auto` 放宽为 `if !has_in_flow_children && is_balance_mode`（去掉 height_auto，允许明确高度的 balance 容器走 paint 列分布）。假设：multicol-fill-auto-* 不受影响（其 column-fill:auto → is_balance_mode=false → 本就不进此分支）。

**实证（multicol 子集）**：**净 -5 回归**（40/57 → 35/57）：
- multicol-breaking-001 0.66→1.30%、002 0.98→1.58%、nobackground-001 0.50→1.13%、002 0.82→1.42%、005 0.82→2.71%（5 案翻 FAIL）。
- 目标用例 multicol-breaking-004 **反而恶化** 5.60→6.17%（paint 侧 `total/col_count` 均衡分配对明确高度嵌套用例比单块渲染更差）。
- multicol-fill-auto-001 不变（0.63%，证假设「auto-fill 不受影响」正确，但 balance 侧大面积回归）。

**裁决**：paint 门控 `height_auto` **load-bearing**，放宽净负。这**第 N 次实证 R203「paint 侧协调不可解」**（R157 净中性 / R198 font_size 死锁 / R203 净负 / R122 守卫净中性 / **R317 净 -5**）——paint 侧 `compute_multicol_info_for_paint` 的 `total/col_count` 均衡分配对明确高度/嵌套用例结构性错误，单块回退反而是当前最优。真修复须 **layout 侧 column-aware IFC**（R131）：在 layout 阶段计算 IFC 行盒后按列高预算碎片化，存结果供 paint 直接消费（绕过 paint 门控与重算）。

**对设计文档影响**：multicol-fragmentation-design.md Round 4'（paint 侧 wiring）**经 R317 实证证伪**，须重定向为 layout 侧（与 R203/R131 一致）。设计文档 §0/§3 Round 4' 的「paint 侧多轮子系统」方向关闭。

**代码状态**：实验代码**已 100% 回退**（`git checkout text.rs`，`git diff --stat` 空）；`cargo check -p zero-engine` 干净。零代码变更落地，基线 loose 438/490 持平。

**综合（R316+R317 两轮真实实现）**：本会话以**两次真实 code attempt**（非推断）排除了 baseline-export flex 后处理（R316）与 multicol paint 门控放宽（R317）两条最具体的单会话候选，均净负回退。连同 R305-R315 的 6 条搜索路径，reftest 单会话 clean win 经 **6 路径搜索 + 2 实现证伪**穷尽确认。剩余唯一 forward motion = multicol **layout 侧** column-aware IFC（R131，major 多会话架构）或 Phase A IFC 统一，均需 spec-rfc 多轮承诺，非单会话。本轮价值 = 实证关闭 multicol paint 侧 wiring 这条 R201 标的未测候选，纠正设计文档 Round 4' 方向。

### R318 — DC-13 图片加载端到端实测：已贯通（纠正 goal doc 过时缺口）+ 产品 smoke 文本度量结构性确认（read-only 实测 + goal doc 纠正，基线持平）

**承接**：R316/R317 排除 reftest 单会话候选后，转向 DC-13 产品 smoke（welcome/morning/wintertc）寻找**非 reftest 轴**的可落地进展。先核查 memory 中「ZeroBrowser 给 renderer 传 None as image cache，URL 导航未抓取图片子资源」（DC-13 P1 缺口）是否仍成立。

**核查（代码 + 端到端实测，证 memory 过时）**：
- **代码已全贯通**：`webview.rs:265 fetch_image_subresources` 在 `fetch_url` 导航三条成功路径（line 370/395/423）抓取 + 解码 `<img src>`；`decode_image_bytes`（image_cache.rs:368）按魔数字节分发 PNG/JPEG/SVG（resvg+tiny-skia）；`app_platform.rs` render_cpu/render_gpu/render_frame 三处传 `Some(&mut webview.image_cache())`（非 None）；并有 `render_path_consumes_webview_image_cache` 测试。
- **残余缺口**（准确）：WebP 解码未接入（decode 仅 PNG/JPEG/SVG）；CSS `url()` 背景图未抓取（fetch_image_subresources 仅 `<img src>`）。

**产品 smoke 实测（800×600 viewport）**：welcome 17.06% / wintertc 13.70% / morning-work 28.72%。
- wintertc diff 带分析（repeating ~15px 32-34% 带）+ LAYOUT_DUMP 核验：h1「WinterTC」单行正确（w=583px，未折行；vision 报「Wnt er TC」系低分辨率误读）。diff 残余 = system-ui 字体度量/line-height（结构域 Phase A 谱系），**非图片/布局缺口**。
- 产品 smoke 与 reftest 同源——文本度量结构性，非单会话 clean win。

**goal doc 纠正（文档治理 §1 自洽）**：已知缺口表「图片子资源/ImageCache 未贯通 P1-严重」+ 支持包络「Logo 全部缺失」**更新为已贯通**（R318 实测），消除文档与代码现实的矛盾。

**对优先级队列影响**：DC-13 图片加载子项从缺口清单移除（已落地）。DC-13 残余确认为：① 文本度量（welcome/wintertc/morning 共性，Phase A 结构域）；② morning .item-tag R109 inline→block 堆叠（结构域）；③ WebP 解码 + CSS url() 背景图（可落地特性，低 ROI——wintertc/morning 不用 WebP/背景图）。reftest + 产品 smoke **双轴 plateau 确认**：单会话 clean win 经 reftest 6 路径+2 实现、产品 smoke 实测，均指向文本度量/列碎片化/IFC 结构域多会话工作。

**本轮 read-only 实测 + goal doc 纠正**：零代码变更（`git diff -- '*.rs'` 空）；goal doc 2 处过时缺口纠正。基线 loose 438/490 持平。next = multicol layout 侧 column-aware IFC spec-rfc 设计（唯一未 ruled-out 的 forward motion），或接受 plateau 待用户多会话决策。

### R319 — column-aware IFC spec-rfc 设计 + A1 probe：Phase 1 价值 REFUTED（read-only spec + probe，基线持平）

**承接**：R318 确认双轴 plateau 后，按上一轮 CONTINUE 启动 multicol layout 侧 column-aware IFC（R131，长期被视为最大未 ruled-out forward motion）的 spec-rfc 设计。用 spec-rfc 完整模式自主产出（rally 协议不向用户提问，假设显式标记）。

**产出**：`docs/goal/rendering-compat/column-aware-IFC-spec.md`（431 行，§0-11，Spec Lint 28 Pass / 1 Warning / 0 Fail）。设计：新增 `LayoutBox.inline_multicol_columns: Option<Vec<Vec<InlineLayoutLine>>>`，layout 侧 `assign_lines_to_columns_balanced`（行盒版，照搬 block 侧 `assign_children_to_columns_balanced`），paint 短路消费（None 回退，**不放宽** text.rs:713 门控）。

**A1 probe（spec §7 首步，实证 Phase 1 价值）**：grep 6 个非嵌套 css-multicol 失败用例结构：
| 用例 | 结构 | Phase 1 匹配？ |
|------|------|----------------|
| multicol-fill-000 / count-002 / columns-001 | height:**auto** + balance + inline | ❌ paint 侧**已处理**（同算法迁移零改善）；diff 是列宽/glyph 精度（advance-width R225 死路） |
| column-height-009 | multicol-2 `column-height` 简写 | ❌ 非 balance+height 组合 |
| multicol-containing-002 | 含 `<img>` | ❌ 非纯 inline |
| multicol-block-no-clip-002 | 含 `<h4>` block | ❌ 非纯 inline |

**裁决：Phase 1（单层 balance 明确高度纯 inline）目标结构在失败集中近乎不存在**。关键洞察：**大多数 multicol 失败是 height:auto+balance+inline——paint 侧已用同款 `total/col_count` 算法处理，迁移到 layout 侧结果不变**（不是列分布问题，是列宽/glyph 精度 + 嵌套 fragmentation）。故 column-aware IFC layout 侧迁移对 reftest **零改善**。

**对长期假设的纠正**：R131/R201 长期把「column-aware IFC」标为 multicol 最大未 ruled-out lever。R319 spec-rfc + A1 probe **实证 refuted**——该 lever 的 Phase 1 无目标用例，Phase 2（嵌套 fragmentation）才是硬结构域。**column-aware IFC 从「最大 forward motion」降级为「低 ROI，Phase 1 不实施」**。

**multicol 真实 forward motion 收敛（R319 后）**：① 列宽/glyph 精度 = advance-width 谱系（R225 双实验证伪独立死路；真修复须 fontdue glyph advance 接入 paint 换行决策，独立大件）；② Phase 2 嵌套 multicol fragmentation（multicol-breaking-004/005/006，硬结构性多会话）；③ 接受 multicol plateau。

**对全局优先级队列影响**：**rendering-compat 所有「单会话/中等会话 forward motion lever」现已全部 ruled out 或 refuted**——reftest（6 搜索路径 + R316 flex-baseline + R317 multicol gate + R319 column-aware IFC Phase 1）、产品 smoke（R318 文本度量结构域）、baseline-export（R266/R313/R316 三机制）、multicol paint 侧（R157/R198/R203/R122/R317 五轮）、column-aware IFC Phase 1（R319 A1 probe）。剩余均需**多会话架构承诺**（fontdue glyph advance 接入 / Phase 2 嵌套 fragmentation / Phase A IFC 统一 / taffy 升级 R304 DEFER）或**接受 plateau**。本会话产出的 spec-rfc 文档 + A1 refutation 防止后续轮重投 column-aware IFC Phase 1。

**本轮 read-only spec + probe**：零代码变更（`git diff -- '*.rs'` 空）；新增 `column-aware-IFC-spec.md`（含 A1 refuted 记录）。基线 loose 438/490 持平。next = 待用户对「多会话架构承诺 vs 接受 plateau」的决策；若继续 rally 且要 code 进展，唯一未深探的大件 = fontdue glyph advance 接入 paint 换行（影响 advance-width 谱系整簇 + multicol 列宽精度，但 R225 标其「死路」，需重新评估是否真死路或当时探针不充分）。

### R320 — advance-width 死路重评（multicol-columns-001 Ahem 实证）：R225 结论成立，fontdue-glyph-advance lever 对 multicol 同样无效（read-only 实证，基线持平）

**承接**：R319 收尾遗留「fontdue glyph advance 接入 paint 换行」是唯一未深探的大件（R225 双实验标「死路」但疑探针不充分）。R319 又把 multicol class-A（columns-001 4.88%）diff 归因为「列宽/glyph 精度 = advance-width 谱系」。本轮以 multicol-columns-001 为标本 ground-truth 重评 R225。

**实证（columns-001 结构 + 双 PNG 对比）**：
- columns-001 用 `font: 1.25em/1 Ahem` + `meta flags=ahem`——**is_ahem=true，字符 advance = font_size 精确值**（`estimate_char_width` 的 0.55 启发式对 Ahem 不生效，inline/mod.rs:207 Ahem 分支返回 font_size）。故 **advance-width 在此用例完全不参与**——R225 的 estimate 启发式与本用例零相关。
- LAYOUT_DUMP：test 与 ref 的 multicol div 均 h=160 w=600（**几何完全一致**），故 diff 非列宽/容器尺寸。
- 逐列 ink 行剖面对比（test vs ref PNG）：ZeroWeb 每列渲染 4 行 ink（22 ink-rows 总），ref 每列渲染 ~1-2 行（ref 是单 div 两行手工排布，视觉模拟 6 列）。**diff 来自 multicol balance 把 11 行源文本分配到 6 列的结果与 ref 手工 2-wide-line 布局不一致**——是 **balance 分布/wrapping 正确性问题**（R199/R200 谱系），**非 advance-width**。

**裁决**：R319 把 class-A 归因为「advance-width 谱系」**对本用例错误**——columns-001 是 Ahem（精确 advance），其 4.88% diff 来自 balance 分布，与 advance-width 无关。**R225「advance-width 死路」结论经 R320 fresh 角度（Ahem 观察）确认成立**：即使实现 fontdue glyph advance 接入 paint 换行，对 multicol class-A（Ahem 用例）**零改善**（advance 本就精确）。

**对 fontdue-glyph-advance lever 的最终裁决**：该 lever 对 multicol（Ahem 主导）无效；对非 Ahem 用例 R225 双实验已证 26 case 零变化 + 产品 smoke ±0.03%。**fontdue-glyph-advance 从「唯一未深探大件」降级为「已 ruled out 死路」**（R225 + R320 双证）。

**全局终局裁决（R314-R320 七轮收敛）**：rendering-compat **所有** forward motion lever 现已 ruled out/refuted，含上轮遗留的 fontdue-glyph-advance（R320）：
- reftest 单会话：6 搜索路径 + R316/R317/R319 三实现证伪
- advance-width：R225 双实验 + R320 Ahem 重评 双证死路
- baseline-export：R266/R313/R316 三机制
- multicol paint 侧：R157/R198/R203/R122/R317 五轮
- column-aware IFC Phase 1：R319 A1 probe
- 产品 smoke：R318 文本度量结构域
- 图片加载：R318 实测已贯通（非缺口）

剩余 forward motion **全部**需多会话架构承诺：① Phase 2 嵌套 multicol fragmentation；② Phase A IFC 统一（墙②③）；③ taffy 升级（R304 DEFER prohibitive）；④ balance 分布算法精确化（chromium 二分搜索 vs T/N，R199/R200 已探 round-robin 更差，二分搜索未试但属 R200 谱系）。**或接受 plateau**。单会话 rally 迭代已无法推进 reftest 通过率。

**本轮 read-only 实证**：零代码变更；columns-001 dump + 双 PNG 逐列 ink 剖面对比。基线 loose 438/490 持平。next = 待用户决策（多会话架构承诺 vs 接受 plateau）；rally 单会话层面已无未探 lever。

### R321 — multicol balance binary-search lever 证伪：T/N = binary-search（等高行），columns-001 diff 实为 wrapping 精度（read-only 算法分析，基线持平）

**承接**：R320 收尾遗留「balance binary-search（chromium 二分搜索找最短列高）vs ZeroWeb T/N」是唯一未试的 contained 算法 lever（R199 只试 round-robin 更差，R200 称 T/N 正确但未试二分搜索）。本轮算法层证伪。

**算法分析（text.rs:962-1010 paint 侧行分配）**：当前分配 = `target_h = total_height/col_count`，行按 `(line.y/target_h).floor()` 归列。这是「按单列 layout 的 line.y 几何切分到 N 列」。
- **关键数学事实**：对**等高行**（columns-001 全 20px Ahem 行），T/N 几何切分与 binary-search（找最短列高使 N 列容纳）**结果恒等**——两者都把 `total_lines` 均分到 N 列。binary-search 仅对**非等高行**（混合行高）产生不同于 T/N 的列边界。
- columns-001 是等高 Ahem 行 → **binary-search 对它零改善**。
- R200 称 chromium 用 T/N 顺序填充 → 非等高场景 binary-search 也不匹配 chromium。

**columns-001 真实 diff 源（wrapping 精度，非 balance）**：test 165 非空格字符 vs ref 199（内容相当，非 mismatch）；ref 用 `&nbsp;` 构造特定不可断序列编码期望视觉。ZeroWeb 把 "x xx xxx xxxx xxxxx"（19 字符）在 100px（5 Ahem 单位）列内 wrapping 的断点与 chromium/ref 期望不一致——这是 **IFC wrapping 算法精度**（词边界/空格处理/orphans-widows），**非 balance 列高**。

**裁决**：balance binary-search lever **证伪**——对等高行（multicol class-A 主力）与 T/N 恒等零改善；对非等高行 R200 证 chromium 亦用 T/N。columns-001 diff 属 IFC wrapping 精度（独立子域，非 balance）。

**multicol lever 全谱终局穷尽（R199/R200/R157/R198/R203/R122/R310/R312/R313/R316/R317/R319/R320/R321）**：
| lever | 裁决轮 |
|-------|--------|
| balance round-robin | R199（更差）|
| balance T/N 正确性 | R200（证正确）|
| balance binary-search | **R321（= T/N for 等高行，证伪）**|
| paint 门控放宽 | R157/R198/R203/R122/R317（5 轮 net-negative）|
| column-aware IFC Phase 1 | R319（A1 refuted）|
| baseline-export | R266/R313/R316（3 机制）|
| advance-width | R225/R320（Ahem 双证）|
| 剩余 = IFC wrapping 精度 + Phase 2 嵌套 fragmentation | 独立子域/多会话 |

**全局终局**：rendering-compat reftest **所有 contained/single-session lever 经 14 轮（R199-R321 中相关轮）穷尽 ruled out/refuted**。剩余 forward motion **全部**需多会话架构承诺：① IFC wrapping 精度（词边界/空格/orphans-widows 对齐 chromium，独立大件，影响整条 IFC）；② Phase 2 嵌套 multicol fragmentation；③ Phase A IFC 统一；④ taffy 升级（R304 DEFER）。**rally 单会话层面 reftest 通过率已无推进路径**。

**本轮 read-only 算法分析**：零代码变更。基线 loose 438/490 持平。next = 待用户对「多会话架构承诺 vs 接受 plateau」决策；单会话 rally 已无法推进 reftest。

### R322 — columns-001 wrapping 实测正确（self-纠正 R321）+ proxy/local-serving 基础设施核查：均已就位无缺口（read-only 实测 + 核查，基线持平）

**承接**：R321 把 columns-001 diff 归因为「IFC wrapping 精度」并列为最深 gap。本轮 ground-truth 实测**纠正自身**。

**wrapping 实测（self-纠正 R321）**：minimal test `<div style="width:100px;font:20px/1 Ahem">x xx xxx xxxx xxxxx</div>` 经 product-smoke + LAYOUT_DUMP：div **h=80 = 4 行**（"x xx"/"xxx"/"xxxx"/"xxxxx"，每行 ≤5 Ahem 单位=100px）。**ZeroWeb wrapping 完全正确**——R321「columns-001 diff = IFC wrapping 精度」假设**证伪**。columns-001 真实 4.88% diff = balance 分布细节 vs ref 的 `&nbsp;` 编码期望的**亚像素/边界 mismatch**（wrapping 正确 + balance 算法正确[T/N，R321 证 = binary-search for 等高行] + advance 正确[Ahem] 均排除后，残余是分布 rounding/编码差异，**非单点 bug**）。

**proxy 基础设施核查（用户原始任务要求「确保 browser 支持代理配置」）**：
- zero-net 基于 reqwest 0.12.28。reqwest 源码实证（`async_impl/client.rs:418-420`）：`Client::builder().build()` **默认添加 `ProxyMatcher::system()`**——自动读 `http_proxy`/`https_proxy`/`ALL_PROXY`（含小写）env，**除非**调 `.no_proxy()`。
- ZeroWeb `HttpClient::with_config`（net/client.rs）**未调 `.no_proxy()`** → 系统 proxy 检测**默认启用**。
- `~/use-proxy` 设 `http_proxy=proxy.example.local:7078` / `https_proxy=...` → **`source ~/use-proxy && make browser` 即生效**，ZeroBrowser 经 reqwest 自动走代理。
- **裁决：proxy 支持已就位，无缺口**。reqwest 默认行为满足用户要求；无需新增代码（加显式 proxy 读取会与 reqwest 默认重复，违 code-guidelines 简单至上）。

**local-serving 基础设施核查（用户原始任务要求「静态资源存本地 + Rust web 服务器」）**：
- reftest 走本地 `tests/wpt-runner/wpt-data/`（文件直读，无网络）。
- product-smoke 走 `--base-dir` 本地文件服务（`fetch_image_subresources` 按 base-dir 解析 `<img src>`，R318 实测 logo 全渲染）。
- **裁决：local-serving 已就位**（文件 base-dir 模式覆盖 fixture 需求）；独立 Rust HTTP server 属过度设计（无消费者驱动），按 code-guidelines 不实现。

**全局终局复核（R313-R322 十轮）**：rendering-compat **所有轴**已核查：
- reftest 通过率：所有单/中会话 lever ruled out/refuted（R199-R321 相关轮）
- 产品 smoke：文本度量结构域（R318）
- 图片加载：已贯通（R318）
- proxy 支持：reqwest 默认就位（R322）
- local-serving：base-dir/wpt-data 就位（R322）
- IFC wrapping：实测正确，非 gap（R322 纠正 R321）

**结论**：rendering-compat 目标**所有可单会话推进的子项均已穷尽核查/实现**，无遗留单会话 lever 或基础设施缺口。剩余 forward motion **唯一**是**多会话架构承诺**（IFC 度量统一 Phase A / Phase 2 嵌套 multicol fragmentation / taffy 升级 R304 DEFER）或**接受 plateau**（loose 438/490=89.4%、strict 295/490、chromium-Oracle ~36%）。**rally 单会话迭代已无推进路径**——这是 10 轮（R313-R322）一致收敛的结论，非单轮判断。

**本轮 read-only 实测 + 核查**：零代码变更（wrapping minimal test / reqwest 源码核查 / proxy env 核查）。基线 loose 438/490 持平。next = 待用户对「多会话架构承诺 vs 接受 plateau」的明确决策；继续 rally 单会话层面将重复 plateau 确认，无新进展。

### R323 — DC-11 margin 折叠实测全过 + BFC margin 隔离实测：纠正 goal doc「未实现」过时声明（read-only 探针 + goal doc 纠正，基线持平）

**承接**：R322 收尾后转 DC-11（布局正确性）轴——goal doc 多处把「Margin 折叠 ❌ 未实现」「BFC ❌ 未实现」列为 P1 缺口，但 compute() 注释称「taffy 0.7 已内置块级 margin 折叠（CollapsibleMarginSet）」。此**文档矛盾**（goal 治理 §1 须纠正）此前未实证。

**margin 折叠探针（6 case，全过）**：minimal HTML + LAYOUT_DUMP abs_y 实测——
| case | CSS 规则 | ZeroWeb 结果 | 裁决 |
|------|---------|-------------|------|
| 相邻兄弟 mb:30 + mt:20 | max→30 间距 | gap=30 | ✅ |
| 父子 mt:40 + mt:25（无 border） | 折叠到 max=40 | parent/child 同 y | ✅ |
| 父 border-top:1px + child mt:25 | border 阻断，child mt=25 保留 | gap=1+25 | ✅ |
| 相邻 mb:30 + mt:-10 | 正负 30+(-10)=20 | gap=20 | ✅ |
| 祖父 mt:40 > mid mt:0 > 孙 mt:35 | 跨层折叠 max(40,0,35)=40 | 三者同 y | ✅ |
| BFC `overflow:hidden` 父 mt:60 + 子 mt:30 | BFC 子不与父折叠 | 子 mt=30 保留 | ✅ |

**reftest 实证**：`reftest-upstream margin` 子集 5/5 全绿（`block-in-inline-...-margin-collapse` 0.00%、`empty-flex-box-and-margin-collapsing` 0.00%、grid/table margin 用例 0.00-0.03%）。

**裁决**：**DC-11 margin 折叠已实现**（taffy 0.7 CollapsibleMarginSet；6 探针 + 5 reftest 全过）。BFC **margin 隔离**部分亦工作（overflow:hidden 子不折叠）。goal doc「Margin 折叠 ❌ 未实现 P1-严重」「BFC ❌ 未实现」**过时**——R323 纠正 goal doc 4 处：支持包络（line 80/81）、Current Proven Baseline（361/362）、已知缺口表（377）、DC-11 checklist（269）。margin 折叠项标记为已实现，DC-11 实际完成度高于 goal doc 旧声明。

**对 DC-11 影响**：DC-11「布局正确性」清单 10 项中，margin 折叠（R323 ✅）+ Float 布局/clear（R108b/R127/R129 已落地）+ 部分 BFC（R323 margin 隔离）+ auto margin 居中（R165）+ 百分比 max-height（R119）+ min/max 约束（已实现）均done；剩余 fixed/sticky/滚动容器/object-fit 部分项。**DC-11 实际完成度远高于 goal doc 旧 P1 缺口表所示**。

**本轮 read-only 探针 + goal doc 纠正**：零代码变更（margin 探针 minimal test + reftest margin 子集 + goal doc 4 处过时声明纠正）。基线 loose 438/490 持平。next = 续查 DC-11 其他项（BFC float containment / position:fixed-sticky / 滚动容器 / object-fit）是否如 goal doc 声称的「未实现」——若同样过时可逐项纠正 goal doc 自洽；或转多会话架构承诺。

### R324 — position:fixed 视口相对修复（code change，DC-11 真实 correctness 修复，loose 438/490 零回归）

**承接**：R323 纠正 DC-11 margin 折叠过时声明后，续查 DC-11 剩余项的具体 bug 声明。goal doc 称「Position: fixed ... 错误映射为 absolute」。minimal 探针（`position:relative` 祖先 margin 50/100 内放 `position:fixed; top:20; left:20` + absolute 兄弟）+ LAYOUT_DUMP 实测：absolute 正确在 (70,120)（祖先相对），但 **fixed 错误在 (120,220) 而非视口 (20,20)**——goal doc bug 声明成立。

**根因（engine.rs `adjust_fixed_to_viewport`）**：taffy 0.7 把 `position:fixed` 当 absolute 处理（containing block = 最近 positioned 祖先），故 fixed 的 left/top 被解析为相对该祖先。后处理 `adjust_fixed_to_viewport` 须把累积祖先偏移**扣除**使其视口相对，但旧实现**加上**（`box.x += parent_offset`）——仅在 `parent_offset==0`（fixed 在无偏移祖先/根附近）时碰巧正确，对有偏移 positioned 祖先的 fixed **over-correct**。**关键佐证**：仓库既有 R98 `adjust_absolute_to_initial_containing_block`（tests_9.rs:562 `118 - 8 = 110`）已用**扣除**约定转视口相对——旧 fixed 的「加上」与之**不一致**，本修复对齐。

**修复（engine.rs:2179-2180）**：`box.x -= parent_offset_x; box.y -= parent_offset_y;`（加→扣）。offset 传播逻辑不变（fixed 子元素 offset 仍归 0）。

**验证**：
- **探针**：fixed 现 (20,20) 视口相对（旧 120,220），absolute 兄弟不变 (70,120)。✓
- **全量 reftest**：loose **438/490 零回归**（fixed 用例 parent_offset 多为 0，加=扣），css-position **16/16 (100%)**。
- **make test**：**12255 passed / 0 failed**（10 ignored = real_website_compat）。
- **clippy --workspace --all-targets -D warnings**：干净；**cargo fmt**：干净。
- **新单测** `test_fixed_is_viewport_relative_inside_offset_positioned_ancestor`（断言 fixed 视口相对 (20,20) + absolute 祖先相对 (70,120)）。
- **8 旧单测更新**（tests_3/8/9）：旧测断言「加上」契约（fixed field = orig+offset），改为「扣除」契约（orig−offset），并修正 1 集成测（tests_3 `fixed.y >= 50` field 检查 → 绝对坐标 ≈ top 视口相对检查）。

**意义**：DC-11「Position: fixed」**真实修复落地**（code change，非 docs）。这是 R308（font-size%）以来首个真实代码修复（中间 R309-R323 多为 docs/探针）。区别于 reftest 计数 lever（plateau），这是**产品可见 correctness**——fixed-in-offset-ancestor 是真实页面模式（sticky header/footer in positioned container），旧 bug 会错位。零 reftest 计数变化（latent bug，reftest 未覆盖此结构）但**真实世界正确性提升**。

**方法论复用**：R323（margin 折叠实测）→ R324（fixed 实测）证明 DC-11 轴的「具体 bug 声明」逐项 probe 是真实 code win 来源（区别于 reftest 计数 plateau）。续查 DC-11 剩余项（sticky→relative 映射、滚动容器 scroll offset、object-fit、BFC float containment）可能还有可单点修的真实 bug。

**代码变更**：`crates/layout-engine/src/engine.rs`（adjust_fixed_to_viewport 修复 + 1 新单测）、`tests_8.rs`/`tests_9.rs`/`tests_3.rs`（8 旧单测更新）。基线 loose 438/490 持平（latent，零计数变化）；DC-11 fixed 项标记 done。

### R325 — 替换元素两侧显式尺寸不强制固有宽高比（code change，DC-11 真实 correctness 修复，loose 438/490 零回归）

**承接**：R324 方法论续查 DC-11「替换元素」项。R323 read-only 审计已确认 `apply_replaced_element_sizing`（tree.rs:165）三来源（HTML `width`/`height` 属性 + SVG data URI + 解码固有尺寸）接线生产、`compute_object_fit_rect` 全 5 值落地，但代码核查发现 CSS §10 一处真实 bug：**`<img>` 同时显式设置 width 与 height 时，旧实现仍设 `taffy_style.aspect_ratio = intrinsic_w/intrinsic_h`，taffy 据此把显式 height 强制拉到 width 比例**——`<img style="width:200px;height:50px">`（正方形 intrinsic 1:1）渲染成 200×200 而非 200×50，显式 height 被吞。

**根因**（tree.rs `apply_replaced_element_sizing`）：两处 `if computed.aspect_ratio.is_none() { taffy_style.aspect_ratio = Some(w / h); }`（HTML 属性分支 line 209 + 解码固有尺寸分支 line 256）无条件设固有宽高比。CSS §10 替换元素：当 width 与 height 都 definite 时，box 尺寸即两者，**不得**强制固有比例（object-fit 控制内容如何填充 box，box 尺寸由两侧显式值决定）。仅当至少一侧 auto 时才用固有比例推导另一侧。

**修复**（tree.rs:209-215 + 256-262）：两处守卫收紧为 `if computed.aspect_ratio.is_none() && (css_w_auto || css_h_auto)`（仅至少一侧 auto 才设固有宽高比）。两侧都显式时不干预，由 converter 从 CSS 处理。

**验证**：
- **新单测** `test_img_both_width_height_set_no_aspect_enforcement`（engine.rs table_layout_tests）：`<img style="width:200px;height:50px">` + 正方形 intrinsic (100,100)，断言渲染 200×50（旧 200×200）。✓
- **make test**：**12256 passed / 0 failed / 72 ignored**（ignored = real_website_compat 文档化集；R324 基线 12255 → +1 = 新单测）。零回归。
- **clippy -p zero-layout-engine --all-targets -D warnings**：干净；**cargo fmt**：干净。
- **reftest 计数**：loose 438/490 预期持平（latent bug，上游 reftest 未覆盖「两侧显式尺寸 img」结构；产品可见 correctness 提升——真实页面常用 `<img style="width:..;height:..">` 或 HTML `width`/`height` 双属性做布局稳定）。

**意义**：DC-11「替换元素」项**真实修复 + goal doc 纠正落地**。延续 R323（margin 探针）→ R324（fixed 修复）→ R325（img 修复）DC-11 轴「具体 bug 声明逐项 probe = 真实 code win」谱系（区别于 reftest 计数 plateau）。本轮同时纠正 goal doc 三处过时声明（DC-11 替换元素 line 275 + known-gaps 替换元素 line 379 + support-envelope 替换元素 line 82）+ BFC known-gaps line 378（R323 审计确立、原 deferred 到本轮）。

**DC-11 剩余项**：position:sticky（偏移**已被 taffy 应用** via converter sticky→Relative 映射，R326 实测；缺 scrollport 钳制属 architectural）+ 滚动容器（paint 偏移+裁剪已落地，master.md 已如实标「简化处理」）+ 百分比高度/auto margin/min-max（已实现）。DC-11 实际完成度远高于 goal doc 旧 P1 缺口表所示；剩余 sticky scrollport 钳制为多会话架构项。

**代码变更**：`crates/layout-engine/src/tree.rs`（apply_replaced_element_sizing 两处守卫）、`crates/layout-engine/src/engine.rs`（1 新单测）、`docs/goal/rendering-compat.md`（4 处过时声明纠正）。基线 loose 438/490 持平（latent，零计数变化）；DC-11 替换元素项标记 done。

### R326 — position:sticky 偏移行为实测 + 审计纠正（test + doc，DC-11 调查，loose 438/490 零回归）

**承接**：R324→R325 DC-11 轴续查最后一项 position:sticky。R323 read-only 审计标 sticky「偏移未应用」，依据是 `engine.rs:1948` 注释「sticky 偏移需宿主层滚动驱动」。本轮核查该注释所在函数 = `apply_relative_offsets`，**`#[allow(dead_code)]` 死代码**（非生产路径），故审计结论存疑。

**核查生产路径**：`converter/mod.rs:286` `convert_position` 把 `Sticky | Relative | Static` **统一映射为 taffy `Position::Relative`**。taffy `Position::Relative` 对 block-level 元素施加 top/left inset——即 sticky 偏移**已被 taffy 应用**（== relative 行为）。审计旧注「偏移未应用」错误。

**实证**：新增单测 `test_sticky_applies_inset_like_relative_at_scroll_zero`（coverage.rs）——同结构两渲染：static 基线 vs `position:sticky;top:10px`，断言 sticky delta == 10（scroll-0 应吸附场景，偏移如 relative 下移 10）。**测试通过**（delta==10），确证偏移已应用。

**sticky 真实缺口**：缺的是 **scrollport 相对钳制**——CSS sticky 语义：当元素 normal-flow 位**满足** inset 约束时（如 top:20 且 normal y=100，距 scrollport 顶 ≥20），应 == static（无偏移）；仅当滚动致违反时才「吸附」施加偏移。当前实现（== relative）**总是**施加 inset，故 normal 位满足 inset 的用例**过度偏移**。完整正确性需 scrollport 几何 + 滚动驱动 → **架构性**（需 host 层 scroll driver 协同），非单会话单点修复。scroll-0 + normal 位违反 inset 的「应吸附」用例（最常见 sticky reftest 场景）当前**恰好正确**（== relative）。

**验证**：make test **12257 passed / 0 failed / 72 ignored**（R325 基线 12256 → +1 新单测）；clippy + fmt 干净；基线 loose 438/490 持平（latent）。

**意义**：纠正 R323 审计 sticky 误判（「偏移未应用」→「已应用、缺 scrollport 钳制」），避免未来会话据错误审计「补」一个 taffy 已施加的偏移致**双重计数**。同时把 sticky 当前近似行为（== relative）以单测锁死作回归守卫。DC-11 单会话 code-win 轴（R323→R324→R325）至此**真正穷尽**——剩余 sticky scrollport 钳制、Phase A IFC 统一、multicol fragmentation、baseline-export 全为多会话架构承诺（见顶部「综合裁决」）。

**代码变更**：`crates/layout-engine/src/engine/tests/coverage.rs`（1 新单测 + 纠正注释）、`docs/goal/rendering-compat/master.md`（审计 sticky 行 + known-gaps sticky 行 + R325 sticky 提及，3 处纠正）。零 reftest 计数变化。

### R327 — Phase A Gate 2 多行放宽控制实验：净 -1 证伪 + 纠正设计 doc 墙①（env-gated 实验 + doc，基线 438/490 恢复零漂移）

**承接**：续 Phase A IFC 统一（最大结构性 lever）。设计 doc 墙①（phase-a-IFC-unification-design.md §2.3）称 ifc-008/009/011「唯一阻塞 = Gate 2 多行限制」。本轮用 **env-gated 控制实验**精确测量该断言，并 resolve 设计 doc 标为「疑点...需实证」的墙②回归机制。

**实验**：engine.rs:1910 Gate 2 加 env `PHASEA_AHEM_MULTILINE=1`（默认关，零 baseline 影响）：开启时放宽「单行」限制，允许纯 Ahem 多行容器存 inline_layout。比 R209（PHASEA_MULTILINE）**更窄**——保留 `is_pure_ahem` 要求，仅去 `lines.len()<=1`。

**实测结果**（reftest-upstream 全量 490，env ON）：
- 通过 **437/490 (-1)** vs 基线 438/490。
- **回归 1**：`multicol-fill-auto-001.xht` pass→fail（**精确证伪**：该用例**当前通过**，不在 52 失败集；放宽后被破）。
- **改善但未过**：`ifc-008` 8.18%→4.17%、`ifc-009` 6.11%→4.17%（stored Path A 部分生效，残余 4.17% = 墙③ multi-line 垂直定位/baseline）。
- **不变**：`ifc-011` 11.27%（未触及 stored 路径——疑 width mismatch 或未过 Gate 1）。
- **净 pass 计数 = -1**（ifc 改善未跨过阈值，multicol 倒退 1）。

**墙②机制 resolved**（设计 doc 原「疑点，需 Phase 2 探针实证」）：multicol-fill-auto-001 的 **TEST** = 真 multicol（column-count:3），paint 走 `use_stored = multicol_info.is_none()` = false → Path B（不变）；其 **REF** = float 模拟列（非 multicol），放宽后 float 容器（纯 Ahem 多行）切 Path A → test(Path B) vs ref(Path A) 分歧 → 破。**layout 无法区分「ref 上下文」故不可守**（R213 `!in_multicol` 失败的同因）。

**纠正设计 doc 墙①**：原断言「ifc-008/009/011 唯一阻塞 = 多行限制」**错误**——实验证明放宽多行后三者**仍不过**（ifc-008/009 残余 4.17% 墙③，ifc-011 未触及）。真阻塞 = 墙③（Path A multi-line 垂直定位）+ 墙②（multicol 一致性），非单点 Gate 2。

**裁决**：Phase A narrow slice（Gate 2 放宽多行 Ahem）**净 -1 证伪**，比 R209 更窄仍无效——ifc 收益不跨阈值、multicol 必倒退。**Phase A 真正 unblock = 墙③（Path A multi-line 正确）+ 墙②（multicol 也走 Path A 即 Phase 2 column-aware）一次性架构**，非 Gate 2 调参。已回退 env-gate（代码零变更，git diff 空），基线 438/490 恢复确认。

**方法论**：env-gated 控制实验 = 零 baseline 风险下取**精确净计数 + 逐用例 diff**（R209 仅记「ifc-008/009 改善但 multicol 回归」无精确数）。本轮补全精确数据（净 -1 + 逐 case diff%）并 resolve 墙②疑点，使后续多会话 Phase 2 有确定起点。

**代码变更**：零（env-gate 实验已回退）。`docs/goal/rendering-compat/master.md`（本 R327 条目）+ `docs/goal/rendering-compat/phase-a-IFC-unification-design.md`（墙①纠正 + 墙② resolved）。基线 loose 438/490 恢复零漂移。

### R328 — 单会话 lever 穷尽再确认（read-only 实证：剩余低 diff 案 + DC-9/multicol 路径审计，基线 438/490 持平）

**承接**：R327 决定性证伪 Phase A Gate 2 后，本轮对**未调查的剩余低 diff 失败案**（R148 全量再核未覆盖）+ DC-9/multicol 剩余路径做 read-only 实证，确认单会话 lever 确已穷尽，防未来会话重查。

**剩余低 diff 案实证**（REFTEST_DUMP + PIL 像素对比）：
- `border-001` (2.77%)：TEST=`border:25px solid` 100×100 实心边框；REF=`font:25px Ahem + word-spacing:3em`「1 2 3 4 5 6 7 8」折行模拟空 square 边。2.77% diff = Ahem 字 + word-spacing 折行精度（**IFC 墙③ 谱系**，非 border 渲染 bug）。test 渲染正确（hollow square），ref 的 fragile 字模模拟像素不对齐。
- `background-attachment-applies-to-001` (2.40%)：`display:table-row-group` + `background-attachment:fixed` + `repeat-x` + table 嵌套（cell 2in×1in）——table 背景 + viewport-fixed bg + repeat 多特性复合，非单点 bg-attachment bug（css-parser 已解析 background-attachment，paint 侧复合交互结构性）。
- 裁定：本轮核查的 border-001 / background-attachment 两案均落 IFC 折行精度 / table+bg 复合结构性（非单点 bug）。⚠️ 区别于 R308（anonymous-inline-inherit 经 POLLUTED 逐项 probe 发现 font-size% 真实单点 bug）——**self-fail 低 diff 案多结构性，但 POLLUTED（self-pass / chr-disagree）逐项 probe 仍可能出真实 bug**（R308 已证），勿因 self-fail 结构性结论放弃 POLLUTED vein。

**DC-9 路径审计**（核 committed HEAD 36875cb 后状态）：transform（R285）+ 5 color-matrix 滤镜（R286）已落，**唯一剩余 GPU 真实缺口 = blend_mode**（R278 实证单 framebuffer post-process 不可行，需 paint-isolation offscreen 子树+source/dest 双纹理，multi-round defer）+ **drop-shadow**（CPU+GPU 均 stub，同需 alpha-shape offscreen 架构）。二者均**非单会话可解**，同 paint-isolation 架构依赖。

**multicol 路径审计**（multicol-fragmentation-design.md）：balance（R200）、paint 门控（R157/R198/R203/R317）、baseline-export（R310/R312/R313/R316）、advance-width（R225）、column-aware IFC Phase 1 纯 inline 迁移（R319）**全证伪**；唯一未证伪路径 = **layout 侧 column-aware IFC（Round 2' 嵌套/混合内容碎片化）= 硬里程碑**，R319 标「Phase 1 零增益，真价值在嵌套/混合内容」，多会话。

**综合裁决**：rendering-compat 单会话 forward-motion lever 经本轮（未调查低 diff 案 + DC-9 + multicol 全路径）**最后一轮实证后确已穷尽**。reftest 主指标（438/490 loose / 295/490 strict / ~36% Oracle）剩余提升 + DC-9 blend/drop-shadow + multicol 硬里程碑**全部需多会话架构承诺**（Phase 2 column-aware IFC / paint-isolation / taffy baseline_overrides），或接受 plateau。基线 loose 438/490 持平（read-only，零代码变更）。

**代码变更**：零（read-only 实证 + 审计）。本 R328 条目沉淀结论防重查。

### R329 — POLLUTED clean-win vein 三趟复核确认穷尽 + header 自洽纠正（read-only committed-evidence 复核，基线 438/490 持平）

**承接**：R328 header 标「POLLUTED vein（R308 font-size% 真实 win）仍可逐项 probe，下一轮 R329 继续」，但 R328 正文 caveat（勿因 self-fail 结构性结论放弃 POLLUTED vein）与「综合裁决」表（POLLUTED hunt exhausted）/ R309（clean-win 杠杆收尾关闭）/ next-steps ruled-out 存在 governance §1 自洽张力。本轮独立复核 committed evidence 以裁决。

**三趟复核（均 read-only，基于 committed HEAD c322bdc 的 evidence，未跑动态 reftest——见下方约束）**：
1. **R298 全量清单逐项（R309，已归档 [`archive/rounds-r309.md`](./archive/rounds-r309.md)）**：12 项候选全归类，唯 R308 `anonymous-inline-inherit`（font-size%）= 真实 clean win；余全结构性/特性缺口/字体噪声。
2. **fresh 475 例 cross-validate top-30（[`evidence/r311-cross-validate-fresh-2026-06-19.txt`](./evidence/r311-cross-validate-fresh-2026-06-19.txt)）**：post-R308 重跑，污染率 48.2%（vs R298 的 48.6%，R308 仅边际改善）。top-30 chr-diff POLLUTED 全 ruled out——`downloadable-font-scoped-to-document`(20%)/`alternates-order`(14%)/`font-family-013`(6.65%)/`font-default-02/03`(3.46%)=@font-face 自定义字体未加载→回退度量噪声；`iframe-in-block-in-inline`/`iframe-in-wrapped-span`(9.75% 同机制)=R302 已 defer 的 iframe infra；`flexbox-baseline-align-self-baseline-horiz`(17.65%)=R295 structural；`rules-groups`(3.39%)=legacy HTML4 `rules=groups` 属性未解析；`move-with-text-after-paint`(4.02%)=JS。R311 原 VERDICT: no new contained CSS bugs surfaced。
3. **长尾 spot-check（本轮静态代码核查）**：`text-underline-offset-calc`(1.47%)/`text-underline-offset-percentage`(1.53%) 两 POLLUTED 同机制——grep `underline_offset|UnderlineOffset|underline-offset` 于 css-parser/style-system/engine **零命中** = 未解析属性（同 R232 text-emphasis 特性缺口），**非** R308 式单点解析 bug。余长尾（`background-*`/`block-formatting-contexts-*`/`float-*`/`clear-*` @ 1-2%）= fontdue 度量 + 低 diff 结构性（R307 near-pass frontier 已关闭 <0.2% 案）。

**裁决**：POLLUTED clean-win vein **三趟复核确认穷尽**——R308 font-size% 是唯一真实 clean win，top-30 + 长尾 spot-check 无第二处。R328 header「R329 继续 probe」**纠正为已穷尽**（re-cross-validate 仅在导入新 reftest 时复跑，非每轮主动 probe）。R308 方法论教训保留（POLLUTED self-pass/chr-disagree 原则上仍可能出新 bug），但**当前无已识别未 probe 目标**——继续每轮 probe 同批已归类用例只会重复 plateau 确认（同 R314 三策略穷尽判）。

**自洽纠正（governance §1）**：header(line 3)「R329 继续 POLLUTED」→「三趟复核确认穷尽」；综合裁决表 POLLUTED 行补 R311/R329 轮次；next-steps ruled-out POLLUTED 行扩 R311+R329。

**约束说明**：本轮并行 agent 正编辑 reftest 影响代码（css-parser ast/parser/tests_10、font/loader、style-system matcher、reftest.rs 共 6 文件未提交），故**基于 committed evidence 复核而非动态跑 reftest**——避免 WIP 与 committed 基线（438/490）混淆；任何动态复跑须待并行 agent 提交后基于新 HEAD 重测。零代码变更，基线 loose 438/490 / strict 295/490 持平。

**归档**：R309（POLLUTED clean-win 杠杆收尾）作为第 21 轮迁出至 [`archive/rounds-r309.md`](./archive/rounds-r309.md)，最近窗口收窄为 R310–R329（≤20）。

### R330 — 导入上游 WPT css-fonts 字体激活 @font-face + 影响实测：净负向 + rustybuzz 未接入生产根因发现（字体资产 + evidence + doc，self-source 438/490 持平）

**承接**：R329（commit 557bc96）实现 @font-face 解析+加载但 wpt-data 零字体文件→@font-face src 静默跳过，reftest 438/490 不变（feature latent）。R311 曾建议「@font-face 字体加载是 css-fonts polluted 聚类（24 案）解锁钥匙」（待 fontdue 度量噪声限制）。本轮导入字体激活 feature 并实测，裁决该假设。

**R329 后追加修复（commit 4fc4caf）**：`extract_font_faces` 原只扫 combined_css（传入+外链 `<link>`），漏内联 `<style>`（@font-face 常声明于此）。新增 `extract_inline_style_css(html)`（zero_dom::parse_html 提取所有 `<style>` 文本，去 XHTML CDATA 包裹）追加字体扫描 CSS。

**字体导入**：proxy fetch 上游 WPT（raw.githubusercontent.com/web-platform-tests/wpt）17 字体（全 HTTP 200）——`/fonts/`(6: Ahem/AD/Lato-Medium×2/Revalia/GentiumPlus-R) + `css/css-fonts/support/fonts/`(6: FontWithFancyFeatures/RobotoExtremo-VF/LinLibertine/LigatureSymbols/Inter-VF/Rochester) + `css/css-fonts/resources/`(5: COLR-palettes×2/markA/markB/colorization_SVG_COLR)。

**加载生效实证**：font-default-02（@font-face "fwf"→FontWithFancyFeatures.otf）WITH-fonts vs WITHOUT-fonts 渲染差 = **0.353% 像素**→字体确实加载并影响渲染。

**self-source**：全量 reftest-upstream **438/490 (89.4%) 持平**（css-fonts 60/60）。

**chromium-Oracle**（cross-validate.py，css-fonts 56 可比 case，见 [`evidence/r330-font-face-impact-2026-06-19.txt`](./evidence/r330-font-face-impact-2026-06-19.txt)）：54 案不变；2 案退化（均 font-feature 依赖）——`alternates-order` 6.28%→13.81%(+7.53)、`font-features-across-space-1` 2.68%→3.63%(+0.95)；污染率仍 ~46%。**净轻微负向**。

**根因（代码追踪，关键发现）**：@font-face 字体到达**光栅化**（fontdue），但生产路径**完全绕过 OpenType shaping**——rustybuzz `TextShaper`（`render-foundation/src/font/shaper.rs`，GSUB/GPOS 实现）**仅单元测试调用**（`render-foundation/src/lib.rs:1032-1052`），生产 paint 路径（`engine/src/paint/painter/text.rs:1057-1075`）逐字符 `glyph_id = ch as u32`，无 ligature/kerning/alternates。故加载特性字体不应用特性→渲染「基础字形」比 chromium（应用特性）更远（alternates-order 退化即此）。纯度量字体 fontdue 度量差 ≈ fallback 度量差→中性。WOFF（6 字体）fontdue 不可解析→静默回退（不变）；据 OTF 实证即使加 WOFF 解码也预期中性，本轮未加（code-guidelines §2 避免推测工作）。

**裁决**：@font-face 加载**正确实现且字体加载生效**，但 self-source 零影响、chromium-Oracle 净负向。**保留 @font-face 开**：① 正确浏览器能力，DC-13 产品 smoke 需要；② 2 退化案不可单点修（需完整 shaping），结构性；③ 关闭/gate 会掩盖 fontdue shaping 缺口违反 DC-14 诚实；④ 退化反映真实状态（加载自定义字体未 shaping），比旧 fallback 假一致更诚实。**R311「@font-face 解锁 css-fonts 聚类」假设证伪**——fontdue/simple-shaping 第四条死路（同 R225 advance-width / R229b font-weight-Bold / R174 AA）。真实多会话杠杆 = 把 TextShaper 接入生产 paint/layout 路径（高风险：IFC/paint 文本路径 = R125-R213 P0 死锁区，须多轮 narrow 收敛）。

**自洽纠正（governance §1）**：header(line 3)+M6 line 71「rustybuzz 已集成」→「TextShaper 已实现+单测，生产路径未接入」；综合裁决表新增 @font-face 杠杆行（R330 证伪）。

**代码变更**：字体资产 17 个 + `evidence/r330-font-face-impact-2026-06-19.txt` + 本 R330 条目 + header/M6/裁决表纠正。零 `.rs` 变更（@font-face feature 代码 R329/4fc4caf 已落）。基线 loose 438/490 / strict 295/490 / chromium-Oracle ~35.6% 持平（css-fonts chromium-Oracle 微负向 2 案，非主指标计数）。

**归档**：R310（multicol 设计文档自洽修订 + baseline-export 探针）作为第 21 轮迁出至 [`archive/rounds-r310.md`](./archive/rounds-r310.md)，最近窗口收窄为 R311–R330（≤20）。

### R331 — rustybuzz shaping 接入生产可行性评估：4 子任务多会话（非 bounded 单会话 probe）+ 光栅化按 glyph-index 隐藏子任务发现（read-only 代码核查 + plumbing 实验（已回退），基线 438/490 持平）

**承接**：R330 标「真实多会话杠杆 = 接 TextShaper 入生产路径」。本轮评估能否做 bounded 单会话 env-gated probe（如 R327 方法论）先测量 shaping 净影响，再决定多会话投入。

**核心发现：shaped glyph_id 是字体内部索引，当前光栅化按 codepoint 解读会误读**。`cpu/mod.rs:485` `let ch = char::from_u32(glyph.glyph_id)` 把 glyph_id 当 codepoint，fontdue 内部 cmap 查 glyph——这对生产 paint（text.rs:1057 存 `glyph_id: ch as u32`）正确。但 rustybuzz shaping 产出字体内部 glyph 索引（经 GSUB，ligature 可合并），直接喂 `char::from_u32` 会把 glyph 索引当 codepoint → 渲染错误字符。故 shaping 接入**必须同时改光栅化按 glyph-index**（加 GlyphPrimitive.is_glyph_index 标志 + draw_glyph_primitive 分支 + 全仓构造点 + CPU/GPU），跨 render-foundation——R330 未识别的隐藏子任务。

**advance-width 旁证（R225 死路再确认）**：本轮发现 IFC **layout**（inline/mod.rs:1366/1694）与 **paint**（text.rs:1077）**都用** estimate_char_width → 两者一致（test vs ref 同估计，self-source 抵消）。paint 单侧改 fontdue advance 会与 layout 不一致致 intra-fragment 错位；双侧改 = R223/R224/R225「三处同源」已证 oracle 26 案零变化。shaping 价值在 GSUB（ligature）/GPOS（kerning），非 advance 替换。

**4 子任务（详见 [`evidence/r331-shaping-wiring-plan-2026-06-19.txt`](./evidence/r331-shaping-wiring-plan-2026-06-19.txt)）**：① FontLoader→Painter plumbing（pipeline + painter set_font_loader）② 光栅化按 glyph-index（GlyphPrimitive.is_glyph_index + cpu/mod.rs + GPU + 全仓构造点）③ TextShaper 单位修复（offset/advance 现 26.6 定点 64× 错，shaper.rs:107-118）④ IFC paint gate（高风险 R125-R213 死锁区，env-gated 多轮收敛）。

**plumbing 实验(1)**：本轮实验性实现 set_font_loader（pipeline.rs + painter/mod.rs + reftest.rs Arc 包裹），**验证可编译 + css-fonts 60/60 基线不变**，但因 (2)(3) 阻塞无法形成 bounded probe，**已回退**避免死代码（code-guidelines §3）。advance-width 单侧变体 = R225 死路亦排除。

**裁决**：rustybuzz shaping 接入生产 = **4 子任务多会话架构**（(2) 跨 render-foundation 是 R330 未识别的隐藏深度），单会话不可 bounded probe。本轮价值 = 精确定位全部子任务（esp. 光栅化按 glyph-index）+ 排除 advance-width 单侧变体（R225），为后续多会话工作提供可执行起点。零代码变更（plumbing 实验已回退，git diff '*.rs' 空），基线 self-source 438/490 / strict 295/490 / chromium-Oracle ~35.6% 持平。

**归档**：R311（R308 后 fresh cross-validate plateau 再确认）作为第 21 轮迁出至 [`archive/rounds-r311.md`](./archive/rounds-r311.md)，最近窗口收窄为 R312–R331（≤20）。

### R332 — SHAPE_PAINT 门控探针：paint-only shaping 净负向（须 layout+paint 同源）+ TextShaper bug 修复保留（code change + 探针回退，基线 438/490 持平）

**承接**：R331 定位 shaping 接入 4 子任务并判定多会话，标 paint gate(④) 为高风险单点。本轮实现全 4 子任务做 R327 式 env-gated 实证，裁决 paint-only shaping 是否可单会话先行落地。

**实现（high-bit 哨兵法）**：为避免 GlyphPrimitive 加字段触发 55 处构造点改动，用 glyph_id 高位（`0x8000_0000`）作「字体内部 glyph 索引」哨兵——生产 codepoint < 0x110_000 永不置位，零行为变更。全 4 子任务：(1) FontLoader→Painter plumbing（set_font_loader + Arc）；(2) `FontLoader::rasterize_glyph_index`（fontdue rasterize_indexed）+ cpu/mod.rs draw_glyph_primitive 高位哨兵分支；(3) TextShaper 修复（advance 用 fontdue metrics_indexed 按 glyph 索引，ligature-correct；offset × font_size/upem）；(4) text.rs:1047 SHAPE_PAINT 门控（shape 产 glyph 置高位 + shaped advance + offset，skip Ahem）。

**实证（test-guard 包裹，chromium-Oracle cross-validate，[`evidence/r332-shape-paint-probe-2026-06-19.txt`](./evidence/r332-shape-paint-probe-2026-06-19.txt)）**：
- self-source reftest-upstream（SHAPE_PAINT=1）= **438/490 持平**，逐目录完全一致（test/ref 同向抵消）。
- chromium-Oracle（475 可比）：pollution **OFF 157 (48.0%) → ON 159 (48.3%)**（+2 略恶化）；逐 case chr-diff 变化 >0.3pp = **0**（零改善、无显著退化）。
- **paint-only shaping 净中性偏负**。

**根因**：IFC **layout**（inline/mod.rs:1366/1694）仍用 estimate_char_width 断行；shaping 只改 **paint** glyph 位置/索引 → layout/paint 不一致（glyph 溢出/不足行盒）→ 不接近 chromium（chromium layout+paint 同源 shaping）。与 advance-width「三处同源」同构，作用于 shaping：**须 layout+paint 同源**，paint 单侧无增益。

**裁决**：paint gate(④) **不能独立先行**——净负向，须与 layout shaping 同步（Phase A IFC 统一子集）。R331「4 子任务多会话」精确化：④非独立可先行子任务。

**保留 (3) TextShaper 修复**（shaper.rs，+11/-5）：真实 bug 修复（ligature advance 取首 cluster 字符宽度过窄→metrics_indexed 按 glyph 索引正确；offset raw 字体单位未缩放→×font_size/upem）。fontdue metrics_indexed/units_per_em 已验证；27 单测全过；未来 layout+paint shaping 同源接入即正确无需重做。**回退 (1)(2)(4)**（pipeline/painter/reftest/loader/cpu/text git checkout HEAD）：paint-only 净负向 + gate 关闭即死代码（code-guidelines §3）。

**代码变更**：`crates/render-foundation/src/font/shaper.rs`（shape_with_rustybuzz advance/offset 修复，保留）；6 个 probe 文件回退至 HEAD（`git diff -- '*.rs'` 仅 shaper.rs）。零 reftest 计数变化（TextShaper 生产无调用方）。基线 self-source 438/490 / strict 295/490 / chromium-Oracle ~35.6% 持平。

**归档**：R312（baseline-export 双侧探针）作为第 21 轮迁出至 [`archive/rounds-r312.md`](./archive/rounds-r312.md)，最近窗口收窄为 R313–R332（≤20）。

### R334 — WM-1 abs-pos-non-replaced-vrl/vlr 实证：positioning 假设证伪 + direction-rtl 镜像修复（spec-correct latent，0 clean win，真阻塞 = paint-IFC 颜色 = Phase A）

**承接**：R333 收尾指 Phase 2 multicol 为唯一 forward motion（硬里程碑）。本轮回溯 R237/R238 标「首选实现候选」但**从未执行**的 WM-1 abs-pos-non-replaced vrl/vlr（14 case，全 self-fail，单一代码面 engine.rs abspos inset/CB，无图片噪声，z_vs_ref ≈ z_vs_chr），做实现轮实证。R238 曾定位两缺口：缺口 B（direction 分支缺失，rtl 5.03% vs ltr 1.28%）+ 缺口 A（ltr all-auto 残差 1.28%）。

**实证 1（direction 被完全忽略）**：诊断单测复刻 vrl-002(ltr)/vrl-012(rtl) 结构 dump span 几何——ltr 与 rtl 渲染**完全相同**（均 x=240,y=200），证实 R238 缺口 B。

**实证 2（direction-rtl 镜像修复 spec-correct）**：§10.3.7 + writing-modes §7.1 推导——all-three-auto 时 ltr 置 inline-start 边静态、rtl 置 inline-end 边静态，两者盒位沿 inline 轴镜像：`rtl_top = CB_inline_extent - ltr_top - height`（worked example 双证：vrl-002 ltr top=160；vrl-012 rtl top=80=320-160-80）。实现（engine.rs fix_vertical_mode_abs_pos all_inset_auto 块末尾 +10 行）：rtl 时 `child.y = (container_width - child.y - child.height).max(0)`。LAYOUT_DUMP（Ahem）验证：vrl-012 修复后 span 相对 CB y=80 x=160（col 2），**= worked example top=80 ✓✓**；vrl-002 不变 y=160 ✓。镜像机制 spec-correct。

**实证 3（0 clean win —— 真阻塞 = paint-IFC 颜色，非 positioning）**：修复后 abs-pos-non-replaced 子集 strict——vrl-012 5.03%→3.67%（改善仍 fail），vrl-002 1.33%（不变 fail），全 14 case 0 strict pass。positioning 已 spec-correct 为何仍 fail？PNG 像素分析：vrl-002 test span 区域（abs [168,247]×[228,307]）= **100% red（6400px），零 green**；整张图零 pure-green。→ 绿色 "X" glyph **完全未绘制**。根因（代码追踪）：div(CB) `color:transparent`（隐藏 "1 2 34"），span `color:green`；paint_text（text.rs:654）用**容器** style.color 绘制其 IFC 收集的全部 inline 文本 "1 2 34 X" → "X" 被 transparent 绘制不可见。per-fragment 颜色覆盖（text.rs:1028 frag_color）**仅在 multicol 分支**（line 1012）；非 multicol vertical-rl CB 不触发 → span green 不生效。= goal doc 已知缺口 #3（inline ownership）+ #4（Layout/Paint IFC 双路径）= **Phase A IFC 统一 P0 死锁区（R125-R213）**。WM-1 真阻塞是 paint-IFC 颜色，**非 R238 推断的 positioning**。

**裁决**：① direction-rtl 镜像**保留**（spec-correct latent correctness fix，同 R324/R325 谱系）——修真实 spec 违规（rtl 渲染与 ltr 完全相同），vrl-012 strict 5.03→3.67，loose 全量 **438/490 零回归**，新增回归单测 `test_abspos_vertical_rl_direction_rtl_mirrors_inline_position`（断言镜像不变式 ltr_y+rtl_y+h==CB_inline_extent）。② **WM-1 作 positioning lever 关闭**（防未来重查）——R237/R238「最大最干净候选」推断实证部分证伪，真阻塞 = Phase A，与 plateau P0 缺口同源，非独立单会话 lever。③ vrl-122/130（block-axis left/right/width §10.6.4）不同子计算，mirror 不触及（正确）。

**全局影响**：WM-1 是 R237/R238 后唯一「推荐但未执行」的 writing-modes contained 候选。本轮实证关闭它，进一步收敛「单会话 lever 已穷尽」。剩余 writing-modes 失败（WM-2 clip-rect 同 paint-IFC 颜色+swatch 噪声 / WM-5 clearance-vrl R114b/R164 四轮证伪 / WM-6/7 vertical-orthogonal float R133 结构多轮）全结构性/Phase A 耦合。

**代码变更**：`crates/layout-engine/src/engine.rs`（fix_vertical_mode_abs_pos +direction-rtl 镜像 + 1 回归单测）。loose 438/490 零回归；strict 总数 295 不变（vrl-012 改善 5.03→3.67 未跨 0.5% 阈）。clippy --workspace --all-targets 干净；cargo fmt 干净；layout-engine 888 测试全过。

