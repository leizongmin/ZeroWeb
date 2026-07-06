# 历史轮次归档 — R991–R1093（multicol spanner/Phase 2 · logical props · vertical-mode · FreeType C-dep · ::first-letter Phase A · nbsp/word-spacing · plateau era）

> 内容 100% 保留自主控文档 `master.md`：`## 下一步` 节中 **R1093→R991 降序逐轮详记块**逐字迁出（仅作归档，未去重、未重排；物理顺序为 newest-first，含 R990 余波/site-3 结论尾段）。下文为该 era 完整逐轮详记。回主控文档：[`../master.md`](../master.md)。

---

### R1093 ::before/::after color 修复（R554-separable）= 0 yield（无 WPT 案用 colored ::before text）·autonomous plateau exhaustive 完成·已回退·零 net 源码

承 R1092（autonomous plateau definitive，C-dep=root unlock）。本轮试最后一个非 C-dep/Phase-A 的 tractable candidate：R554 ::before/::after 伪元素 color bug（inject_pseudo_text_nodes 存伪样式于文本节点 NodeId，但 collect_inline_items 读父 + paint owner_id 取父 → 伪 color 丢失）。R554 net-negative 是 content-list/counter，基础 color 修复疑可分离。

**实现（3 文件，已回退）**：painter/text.rs render_fragment! macro（line 1355）+ multicol 分支（line 1174）owner_id 对文本节点优先取自身 NodeId 样式 + inline/mod.rs collect_inline_items（line 581）`styles.get(&child_id).or_else(parent)` 使伪 font-metrics/color 生效。

**A/B（generated-content 全量 227）**：92/227 → 92/227 **NET 0**。逐案 top-15 全 **+0.00**（before-after-table-parts 93.36 / content-177 7.09 / before-after-display-types 6.89 / content-173 6.38 等全不变）。

**根因（0 yield）**：grep 确证 generated-content **无任何 case 用 `content: "非空字符串"` + color**（全用空 content / visual styling bg+border，无 colored ::before text）；welcome.html 亦无 ::before。故 color 修复虽 CSS-correct（零回归）但**无 case 触发**，reftest + product-smoke 双 0 yield。

**裁决**：按协议（0 yield + unexercised → revert）git checkout 3 文件回退，零源码，cargo check 绿。修复存档于此（未来 WPT/产品页加 colored ::before text 时可重应用，e.g. FontAwesome ::before 图标）。

**★ autonomous plateau exhaustive 完成（8 session R1085-R1093 穷尽）**：
| 维度 | 状态 |
|------|------|
| fresh scan 全 10 dir | 全 cluster font-wall 或深结构性（R1087-R1092） |
| ::first-letter（436 案）| Phase-A 门控（full/color-only/+门控 三变体 net-neg/0，R1088/R1089）|
| Phase A line-box Phase 3 | C-dep 门控（fontdue tight-ink，combined -29，R1090）|
| abspos §10.3.7（10 案）| 探针确证几何正确，是 instruction font-wall（R1092）|
| float §10.3.5 / R109 / multicol | 深结构性 / Phase-A deadlock |
| ::before color（R554-separable）| 0 yield（无 case 触发，本轮）|
| C-dep（FreeType）| user-blocked（CI billing 6-target 全 failure + policy）|

**所有 autonomous reftest-yield lever 穷尽**。C-dep（价值上修 +200~400，R1092）= font-wall plateau 唯一 batch unlock，仍 user-blocked。forward = **C-dep 用户决策**（CI 计费恢复 + policy 批准 → 翻 freetype-raster default）。

**门禁**：纯调查（修复加已撤），零 net 源码，make test 未跑。tree clean，cargo check 绿。

**▶ 下会话**：C-dep 用户决策 = 唯一 lever；autonomous 侧无 yield（plateau exhaustive）。若用户解 CI 计费 + 批 policy，翻 default → +200~400 font-wall batch unlock + 解锁 Phase A/::first-letter 重试。

### R1092 abspos §10.3.7 cluster 探针确证 = 几何正确·2.89% 是 instruction 文本 font-wall·最后「非 font-wall candidate」亦 font-wall·plateau definitive 完成·零 net 源码

承 R1091（positioning abspos §10.3.7 cluster 2.89%×10 是 EV 最高残余 candidate，Ahem 用例疑非 font-wall）。本轮加临时探针（env ABSPOS_PROBE，递归打印 abspos 子元素 geom + 计算 inset/margin）确证根因。

**探针结果（3 案代表簇）**：
| case | 几何 | 计算 inset/margin | 裁决 |
|------|------|-------------------|------|
| width-001（全 auto）| x=3 y=3 w=100 h=100 | left/right/top/bottom/width/height 全 auto | ✓ 静态位 + shrink-to-fit 100px Ahem X 正确 |
| width-004（auto-margin，全 inset fixed）| x=103 w=100 | left=100 right=100 width=100 mleft/mright auto | ✓ over-constrained 忽略 right + margin=0，x=100+3border 正确 |
| height-002（全 auto）| x=3 y=3 w=100 h=100 | 全 auto | ✓ 同 width-001 |

**三案 abspos 几何全部正确**——ZW（taffy 0.7 + ZW postprocess）正确处理 §10.3.7 static-position + shrink-to-fit + auto-margin solve + over-constrained（之前疑「无 horizontal abspos solver」= 误判，taffy 已覆盖）。**2.89% diff = instruction 文本 font-wall**（"Test passes if a filled blue square is in the upper-left corner..." 长 instruction @ 默认非-Ahem 字体），非 abspos bug。

**★ 裁决：abspos §10.3.7 cluster 非 lever，是 font-wall**。R1091 标「最高 EV 残余 candidate」推翻。**fresh scan 全 dir 所有 cluster 现均确证 font-wall 或 user-blocked**——R1085/R1086 是仅有的两 clean hit（linebox/text），其余 linebox-applies-to / margin-padding-clear ×126 / bottom ×62 / lang / attr / class / abspos instruction 全 font-wall（near-pass band），结构性簇（§9.7 布局 / §10.3.5 float 宽 / R109 vertical / multicol）多 session。**autonomous plateau definitive 完成**。

**★ C-dep 价值上修**：探针证 abspos 几何正确→C-dep（font-wall root unlock）除 R1084 测的 +32 外，还批量 flip 所有 font-wall instruction cluster（margin-padding-clear ×126 + abspos instruction + bottom ×62 + linebox-applies-to 残余 + lang/attr/class green-text + word-spacing 簇残余 等），**实际 yield 远超 +32**（粗估 +200~400 案 near-pass band 批量翻 strict/loose）。C-dep 是 font-wall plateau 的唯一 batch unlock，仍 user-blocked（CI 计费 6-target 全 failure + policy）。

**门禁**：纯调查（探针加已撤），零 net 源码，make test 未跑。tree clean，cargo check 绿。

**▶ 下会话**：① **C-dep 用户决策**（CI 计费 + policy → 翻 freetype-raster default → font-wall batch unlock +200~400 案 + 解锁 Phase A/::first-letter）；② C-dep 仍 blocked 时，残余 autonomous 仅深结构性多 session（R109 FR-002/003 / float §10.3.5 / multicol nested）低 EV；③ 勿再 fresh scan / 探针 abspos（plateau definitive，全 cluster font-wall/结构性）。

### R1091 positioning + floats/floats-clear fresh scan = cluster 全 font-wall / 深结构性·plateau 扩展确证·零 net 源码·纯调查

承 R1090（autonomous plateau 四重确证 C-dep root unlock）。本轮扫未深扫 dir（CSS2/positioning 578 PNG + CSS2/floats/floats-clear 合 314 PNG）找非 font-wall clean lever（R1085/R1086 类）。

**CSS2/positioning 300/578 (57.3%)**：identical-delta cluster 全 font-wall 或深结构性——
- 1.33% × 62（bottom-004 簇）= `bottom:-0px; position:relative`（无实际 offset），diff = instruction 文本 font-wall（同 R1087 margin-padding-clear 1.33% × 126 谱系）。
- 0.85% × 66（position-relative-014）/ 0.89% × 38（right-offset）/ 0.69% × 36（abspos-overflow）/ 0.41% × 24（left-019）= 同 font-wall。
- 2.89% × 10（absolute-non-replaced-width/height）= **Ahem**（非 font-wall）测 abspos 全 auto offset/margin + auto width/height（§10.3.7 static-position + shrink-to-fit）；grep 无 ZW abspos auto-margin 代码 = 疑 abspos auto-sizing 真缺口，但 §10.3.7 深（R500/R324 谱系），10 案 EV 中。

**CSS2/floats + floats-clear 117/314 (37.3%)**：低通过率，小 cluster（4-5 案）高 delta——
- 4.74% × 5（float-applies-to-001a 等）= `display:table-row-group; float:right`。**§9.7 调整已存在**（style-system/lib.rs:573-589，table-internal→block when floated，曾修 float-applies-to-012）→ 4.74% 非 §9.7 缺口而是**布局侧**（floated ex-table-row-group 定位），深结构性。
- 5.22% × 5（float-non-replaced-width-008）/ 1.77% × 4（-002）= float 非替换宽度（§10.3.5），R1019/R180 shrink-to-fit 谱系，结构性。
- 4.17% × 4（floats-placement-vertical）/ 2.91% × 4（adjacent-floats）/ 2.77% × 4（margin-collapse-033）= 深结构性。

**裁决**：fresh scan 第五轮（linebox/text/box/mpc/decor/visuren/selectors/positioning/floats/floats-clear 全扫尽）确证 plateau 扩展——除 R1085/R1086 两 clean hit 外，所有 dir 的 identical-delta cluster 全 font-wall（near-pass band）或深结构性（§9.7 布局 / §10.3.5 float 宽 / §10.3.7 abspos auto / R109 vertical）。**clean single-session CSS-语义 lever 跨全 dir 穷尽**。positioning abspos §10.3.7（10 案 Ahem）是最高 EV 残余 candidate，但 §10.3.7 深（多 session）。

**★ 战略**：autonomous plateau 五重确证（fresh scan 全 dir + ::first-letter 三变体 + Phase 3 + C-dep blocked + 本轮 positioning/floats）。**C-dep（FreeType）= 唯一 root unlock**（font-wall + Phase A + ::first-letter 共同根），仍 user-blocked（CI 计费 6-target 全 failure + policy）。残余 autonomous lever 仅深结构性多 session（abspos §10.3.7 10 案 / float §10.3.5 / R109 FR-002/003）。

**门禁**：纯调查，零 net 源码，make test 未跑。tree clean。

**▶ 下会话**：① **C-dep 用户决策**（CI 计费 + policy → 翻 freetype-raster default → +32 font-wall + 解锁 Phase A/::first-letter 重试）；② 若 C-dep 仍 blocked，abspos §10.3.7（positioning 10 案 Ahem）是 EV 最高残余——per-pixel 探针确证 abspos auto-sizing 缺口后尝试（多 session，§10.3.7 static-position + shrink-to-fit）；③ 勿再 fresh scan（全 dir 扫尽，cluster 全 font-wall/结构性）。

### R1090 Phase A line-box 度量统一 Phase 3（store-gate 移除 + paint 公式扩展）definitive net-negative（linebox -1 / css-text -14 / css-text-decor -14）·确证 Phase A 非-Ahem = fontdue tight-ink = font-wall = C-dep 根 unlock·已回退·零源码

承 R1089（::first-letter 三变体穷尽 Phase A 门控）。本轮直接攻 Phase A line-box 度量统一 Phase 3（linebox-metric-unification-rfc 的 blocked phase），definitive 实验确证其 net-negative + 根因 = fontdue tight-ink（font-wall 谱系），C-dep 是真正 root unlock。

**Phase 3 阻塞点定位**：compute_final_inline_layouts（inline_finalization.rs:776）`if !is_pure_ahem || ...return` 把 valign-aware baseline_y 存储（R822）门控到 pure-Ahem 容器。R817/R822 只对 Ahem 工作 = 此门控。移除门控让非-Ahem 走 Path A（stored）替代 Path B（重跑）= Phase 3 核心。

**实验 1（store-gate 移除 only，env STORE_NON_AHEM）**：linebox **131→84 = -47**。非-Ahem Path A 存了 fragments 但 paint R817 公式仍只对 is_ahem_font（text.rs:1570）→ 非-Ahem fragment 用旧 `v_offset=font_size` 启发式定位 stored 位置 → mismatch 大回归。

**实验 2（combined：store-gate 移除 + paint 公式扩展 `baseline_y_abs - 0.928·fs - frag.y` for non-Ahem，env 同）**：
| dir | baseline | combined | net |
|-----|----------|----------|-----|
| linebox | 131 | 130 | **-1**（paint 公式恢复 Ahem-heavy，-47→-1）|
| css-text | 359 | 345 | **-14** |
| css-text-decor | 108 | 94 | **-14** |
非-Ahem 0.928·fs 定位对 fontdue tight-ink 字形错（bitmap 是 tight-ink H=30 vs metric ascent 37，R876 谱系）→ text dir 回归。**合计 -29，definitive net-negative**。

**★ 核心结论：Phase A 非-Ahem = font-wall = C-dep root unlock**。Phase 3（非-Ahem Path A + paint 公式）net-negative 根因 = fontdue tight-ink 光栅化（bitmap 不带 metric ascent），与 font-wall（R388 fontdue≈chromium 光栅但 line-metric 偏差 + R876 tight-ink）同根。FreeType（C-dep）提供 proper font ascent metric（非 tight-ink），是 font-wall（+32 已测）**和** Phase A 非-Ahem line-box 度量的共同 root unlock。R1088/R1089（::first-letter）+ R1090（Phase 3）三独立角度收敛：**C-dep 是唯一能 batch unlock 的杠杆，当前 user-blocked（CI 计费 + policy）**。

**★ CI 计费仍 blocked**：本轮 dispatch freetype-raster-cross-platform workflow（run 28760051180），6-target 全 failure（payments/spending limit 未解）。C-dep 决策双重门（policy + CI cross-platform 验证）均非自主可解。

**裁决**：按协议（count net-negative→revert）git checkout 2 文件回退（inline_finalization.rs + painter/text.rs），零源码，cargo check 绿。

**意义**：Phase A line-box 度量统一 Phase 3 definitive 穷尽（store-only -47 / combined -29 均负，根因 fontdue tight-ink）。autonomous plateau 四重确证（fresh scan + ::first-letter 三变体 + Phase 3 + C-dep blocked）均收敛 C-dep root unlock。**forward = C-dep 用户决策（解 CI 计费 + 批准 policy）= 唯一自主不可解的 lever**；Phase A 非-Ahem 须 C-dep 后才有意义（FreeType metric 替 fontdue tight-ink）。勿再试 Phase 3 非-Ahem（fontdue tight-ink 墙，三变体穷尽）。

**▶ 下会话**：① **C-dep 用户决策**（CI 计费恢复 + policy 批准 → 翻 freetype-raster default → +32 font-wall + 解锁 Phase A 非-Ahem 重试）；② C-dep 落地后重试 Phase 3 combined（FreeType metric 应使非-Ahem 0.928·fs 定位正确，linebox vertical-align 簇 + 非-Ahem 文本批量 flip）；③ C-dep 落地后重应用 ::first-letter（R1088 evidence）；④ 勿再试 Phase 3 非-Ahem / ::first-letter（fontdue tight-ink 墙，须 C-dep 先行）。

### R1089 ::first-letter color-only 变体 = net count-neutral（299→299）·oracle 确定性验证（A/B 可信）·::first-letter 三变体穷尽确证 Phase A 门控·已回退·零源码

承 R1088（::first-letter 全量应用 net-negative -7，line-box 度量级联）。本轮试 color-only 变体——合成元素 color 取伪样式，font_size/line_height/font_family/font_weight/font_style/letter_spacing/word_spacing **全部重置为元素自身值** → 零布局级联（R1088 -7 根因消除），仅 color 等 paint-only 生效。

**A/B（selectors 全量 542）**：299/55% → 299/55% **NET count-neutral**（FAIL→PASS=0 / PASS→FAIL=0）。49 案 >0.01pp：11 improved（全 first-letter，color match 工作）/ 12 worsened（含 first-line-pseudo-013/014 + selectors-parsing-001 + class-selector-009/010/011 **均无 ::first-letter 规则**，grep 确认）。

**★ oracle 确定性验证（A/B 方法论可信度）**：reverted 后重跑 baseline selectors oracle，与上次 session baseline 逐案对比 = **0 案 ≥0.01pp 差异** → oracle **完全确定性**（HashMap RandomState 未致 layout/paint 顺序漂移）。故 R1089 的 12 worsened 是**真实**副作用（非噪声）= 我的 `first_letter.color != elem_style.color` 门控对**无 ::first-letter 规则但 color 经选择器设置的元素**误触发（伪 compute color 与 elem color 微差，疑 quirks 调整 / currentColor 解析差异；compute_element_style_internal line 558 已证继承 parent_style，但 apply_quirks_mode_adjustments line 579 对 pseudo 可能分歧）。即使用正信号门控（collect_pseudo_declarations 非空）修复副作用，11 improved 仍**零 count flip**（color match 太小，size/line-height 不匹配主导 diff）。

**结论（::first-letter 三变体穷尽）**：full = net -7（36px/line-height:2 cascade）/ color-only = count-neutral（color match 不足 flip）/ 加正信号门控 = 仍 ~0 flip。**::first-letter（项目最大 autonomous lever，436 案）确证完全 Phase A line-box 度量门控**——color 单维不足以跨阈值，须 Phase A 度量统一后 font-size 全量应用（color+size+line-height 三维同匹配）才 flip。R1088 evidence 存档的合成 inline 元素实现 + 本轮 color-only 度量重置 = Phase A 后重应用的完整方案。

**意义**：autonomous plateau 三重确证——clean lever（R1085/R1086）+ 最大 lever（::first-letter 三变体）+ C-dep（user-blocked）均收敛到 Phase A line-box 度量统一。**Phase A 是唯一未试通的 universal gate**（R817 Ahem-gated baseline_y +45 是 Phase A 唯一成功 narrow slice 先例）。oracle 确定性 = A/B 方法论可信，历史 yield 结论（R1085 +14 / R1086 +2 / R1088 -7）可靠。

**裁决**：按协议（无 count yield → revert）git checkout 4 文件回退，零源码，cargo check 绿。

**▶ 下会话**：① **Phase A line-box 度量统一 narrow slice**（照 R817 Ahem-gated 先例，找下一个安全子集存 baseline_y + paint Path A 消费；linebox-metric-unification-rfc Phase 3 的 per-fragment valign-aware baseline_y 是 universal gate）；② Phase A 度量统一后重应用 ::first-letter（R1088 evidence + R1089 color-only 度量重置方案），批量 flip 436 案；③ font-wall C-dep 用户决策（CI 计费恢复）；④ 勿再试 ::first-letter 单变体（三变体穷尽，Phase A 前无 yield）。

### R1088 ★::first-letter 端到端实现正确但 Phase A 度量门控 net-negative（-7 selectors）·fresh scan 最大 lever（436 案）定位·已回退·零源码

承 R1087（fresh scan 收益递减）。本轮扫未扫 dir（visuren + selectors）+ 深挖发现并实现 ::first-letter（fresh scan 第三轮唯一定位到的 clean CSS-语义 lever，亦项目最大 autonomous lever），A/B net-negative 已回退。

**fresh scan visuren + selectors**：visuren 25/34（73.5%）top-worst 全结构定位（position-absolute-%-inherit 11% / anonymous-boxes / fixed+static-CB），无 R1085 类语义 cluster。selectors 299/542（55%）ORACLE_DUMP_ALL 全量 identical-delta cluster：lang-pseudoclass 11.33%×2 / attribute-value 1.35%×3 / class-selector-012 14.73% **全 font-wall 或已正确实现**（:lang matches_lang + class_list split_whitespace + [attr~=] split_whitespace 均已实现；diff = green-text 渲染）。唯一定位 **::first-letter-punctuation-* 簇 ~300 案 @ ~0.95-1.13%**。

**::first-letter lever 量化**：全 wpt-data **959** first-letter/first-line 测试文件（933 在 selectors）；oracle-covered **518**（436 first-letter，仅 11 PASS）。**远超 C-dep（+32）**——若批量 flip 是项目最大 autonomous yield。

**根因**：::first-letter / :first-letter 仅 css-parser 解析（parser.rs:333），下游完全未实现——matcher PseudoElement(_)=>false（正确），style-system lib.rs:180-210 仅计算 ::before/::after（"first-letter" 从未传入），::first-letter 规则级联后被丢弃 → 块首字母从未按伪样式渲染（diff = 未样式化首字母 ~1%）。

**架构（合成 inline 元素，复用 infra 零 paint 改动）**：R554 教训（::before inject 文本节点 + collect_inline_items 查 parent → 伪 color 不生效，painter/text.rs:1174 owner=parent）。改用合成 inline ELEMENT：inject 把首字母包成 `<zw-first-letter>`（display:inline，伪样式）+ 文本子节点，插为块首子节点。collect_inline_items 对 inline 元素读自身样式（mod.rs:860）→ 伪 font 度量；paint owner=合成元素（text.rs:1174）→ 伪 color。**零 collect_inline_items / paint 改动**。

**实现（4 文件，已回退，存档 evidence/r1088）**：computed_style.rs 加 first_letter_pseudo 字段 + default_impl.rs None + lib.rs compute（pseudo_name="first-letter"，color/font-size 差异门控）+ pipeline.rs inject_first_letter_nodes（is_first_letter_punct ASCII+Unicode P* 近似 + split_first_letter_unit CSS §5.12.1 前导标点+首字母+尾部标点 + 块级容器首直接文本子节点提取 + 合成元素 + set_text_content 去 unit）。端到端正确（提取 `/T/` ✓ 验证）。

**A/B（selectors 全量 542）**：299/55%→292/54% **NET -7**（FAIL→PASS=0 / PASS→FAIL=7）。49 案 >0.01pp 变化：14 improved / 35 worsened。7 PASS→FAIL 全 first-letter-punctuation-*（0.98-0.99%→1.00-1.03%，+0.01~0.04pp 跨 1%）。**实现正确但应用 36px/line-height:2 首字母 run 改变首行行盒高，ZW line-box 度量与 chromium 微差→级联 +0.02~0.04pp，近-pass 推过 1%**。

**★ Phase A 度量门控（核心发现）**：::first-letter 与 linebox-metric-unification-rfc（R813）Phase 3 同类——inline run 的 font-size/line-height 改变首行行盒高时，ZW strut/half-leading/ascent 与 chromium 不一致（R814 A2 已证 vertical-align 簇连单行 line-box 高都算错）。::first-letter 暴露同一度量缺口。**Phase A line-box 度量统一解前，::first-letter net-negative（度量级联 > color 匹配收益）**。

**裁决**：按协议（count net-negative→revert）`git checkout` 4 文件回退，零源码，cargo check 绿。实现存档 evidence（Phase A 度量统一后 5 分钟重应用 + 重 A/B，line-box 度量精确后 color 匹配收益应压过级联，批量 flip 436 案）。

**意义**：fresh scan 第三轮确证 plateau 深度——除 R1085/R1086 两 clean hit 外，visuren（结构）/ selectors（font-wall + ::first-letter）均无 clean single-session win。**★::first-letter（最大 lever 436 案）端到端正确但 Phase A 度量门控 → autonomous plateau 再证：clean lever 与最大 lever 均收敛到 Phase A line-box 度量统一。Phase A 是 universal gate。** fresh scan 已穷尽（visuren/selectors/box/mpc/decor/text/linebox 全扫尽），勿再扫。

详见 [`evidence/r1088-first-letter-phaseA-gate-2026-07-06.txt`](./evidence/r1088-first-letter-phaseA-gate-2026-07-06.txt)。

**门禁**：纯调查 + 实验（已回退），零 net 源码，make test 未跑（无源码变更）。tree clean，cargo check 绿。

**▶ 下会话**：① **Phase A line-box 度量统一**（linebox-metric-unification-rfc Phase 3 = 解 per-fragment valign-aware baseline_y + strut/half-leading 冲突，多 session，是 universal gate）；② 度量统一后重应用 ::first-letter（evidence/r1088 存档），批量 flip 436 案；③ font-wall C-dep 用户决策（CI 计费恢复）；④ 勿再 fresh scan（全扫尽）。

### R1087 fresh R740 scan 三 dir（CSS2/box + margin-padding-clear + css-text-decor）= 无 clean cluster win·cluster 全 font-wall/JS/rendering-precision·零 net 源码·纯调查

承 R1086（fresh scan 路线）。本轮 scan 3 dir 找 R1085/R1086 类 CSS 语义 cluster bug，**均无 clean win**——R1085/R1086 是该方法的仅有两 hit（line-height-applies-to + word-spacing），余 dir cluster 性质不同。

**CSS2/box（128 案，45 pass）**：top-worst 全 insert/delete-inline-in-blocks-* / insert-block-in-blocks-*（23-43%）= **JS DOM mutation 测试**（ZW harness 跑 JS 但不反映 DOM 变更到 layout，R888 谱系）+ 匿名块 R109。cluster（3.78% × 4, 3.74% × 3）同 JS-driven。非 clean lever。

**CSS2/margin-padding-clear（682 案，309 pass）**：巨 cluster **1.33% × 126**（margin-bottom-004/005/.../028...）+ 1.15% × 63（margin-right-*）+ 1.21% × 27（*-applies-to-*）。per-pixel margin-bottom-004：diff = instruction 文本 font-wall（y=18-33 gray AA）+ 1px border 下移（border 本身 blue/orange 触碰正确）→ **cluster font-wall 主导**（非 margin bug），C-dep 解后批量 flip。top-worst margin-collapse-106/112/155/038（18-24%）= R702 collapse-through 结构性；margin-em-inherit-001（11.25% oracle / 21.12% product-smoke 差异待查）= em+inherit+collapse 复杂个案，ZW green bbox 完全错位，非单点。

**css-text-decor（242 案，104 pass）**：top-worst text-decoration-thickness-length-rounding / dotted / inset-025（13-15%）= **rendering precision**（厚度取整/点线光栅，font-wall 谱系）。cluster（text-emphasis-position × 5, skip-spaces × 4, 1-3%）= vertical-mode / feature gap，非 clean handling bug。

**裁决**：R1085/R1086 fresh-scan + identical-delta cluster 方法的 clean hit 已尽（line-height-applies-to + word-spacing 是仅有的两个 CSS-属性 handling cluster）。余 dir cluster 性质：font-wall（margin-padding-clear 1.33% × 126，C-dep）+ JS DOM mutation（box）+ rendering precision（decor）+ structural（margin-collapse R702）。fresh scan 收益递减。

**▶ 下会话**：① font-wall C-dep 用户决策（CI 计费恢复后，margin-padding-clear 1.33% × 126 + line-height-applies-to 残余 + word-spacing 簇批量 flip——C-dep 是这些 font-wall cluster 的真正解锁）；② Phase A step-2（多 session，empty-styles 重跑度量统一）；③ 若重启 fresh scan，转 CSS2/visuren + normal-flow + selectors（未扫，但 cluster 性质可能同 font-wall）；④ 勿再扫已 ruled-out dir（box JS / mpc font-wall / decor precision）。

### R1086 word-spacing 前导间隙修复 LANDED = CSS2/text +1 + css-text +1·28 案改善·零 PASS→FAIL·CSS correctness（cluster font-wall 主导故 flip 少）

承 R1085（fresh R740 scan 路线继续）。本轮 scan CSS2/text 找到 word-spacing 簇（1.13% × 17）+ white-space-processing 簇，深挖 word-spacing 定位真 bug。

**bug 发现**：CSS2/text word-spacing-007/008/.../080 簇 1.13% × 17。minimal repro（`<div style="font:16px/1em Ahem; word-spacing:96px">x x</div>` + Ahem）：ZW 第二 x @x=40（应 @136），black width 48（应 144）→ word-spacing 完全不作用于 glyph 位。

**根因（mod.rs break_into_lines 词循环）**：旧实现 `word_width += run.word_spacing`（word_idx>0）把 word_spacing 计入 word_width。fragment.x = current_x（置位**前**值），current_x 在置位后才 += word_width。故 word_spacing 仅推进 current_x 给**下一**词，**本词** glyph 位（fragment.x）不含 gap → 第二词 glyph 落在无 gap 处。

**修复（lead_gap 模型）**：word_spacing 改作非首词的**前导间隙**——置位前 `current_x += lead_gap`（word_idx>0 且非行首词），word_width 回归纯内容宽。fit-check 含 lead_gap，换行后 lead_gap=0（行首词无 gap）。+1 单测 `test_r1086_word_spacing_applied_to_position`（gap >= 110px，含 96px word_spacing）。

**A/B（stash 对照）**：minimal repro 修复后 black width 144（第二 x @136 ✓）。CSS2/text **212→213 net +1**（word-spacing-justify-001 1.03→0.75 PASS）/ css-text 249→250 net +1 / linebox 131→131 net 0 / product-smoke welcome **16.57% 不变**。**零 PASS→FAIL**。28 案改善 >0.3pp（word-spacing 簇 1.13→1.03，未 flip——cluster 余 1.03% 为 instruction 文本 font-wall 主导，C-dep 解后当批量 flip）。

**门禁全绿**：make test 45 bin 0 failed（含新 r1086 单测）/ clippy --workspace --all-targets -D warnings 干净 / cargo fmt 干净 / welcome 16.57%（DC-13 <20% PASS）。inline/mod.rs 2024 行（pre-existing >2000，本轮 +9 行，未重构）。

**意义**：CSS correctness 修复（word_spacing 位 bug 真，minimal repro 实证）。yield 低（+2）因 cluster 余量 font-wall 主导（近-pass band），与 R1084 C-dep 互补——C-dep 解后 word-spacing 簇 + line-height-applies-to 残余当批量 flip。R1085/R1086 两连 clean code win 证 fresh R740 scan（under-mined dir）+ identical-delta cluster 方法有效。详见 [`evidence/r1086-word-spacing-lead-gap-landed-2026-07-06.txt`](./evidence/r1086-word-spacing-lead-gap-landed-2026-07-06.txt)。

**▶ 下会话**：① 继续 fresh R740 scan CSS2/box + CSS2/visuren + CSS2/normal-flow（box-model/visual-formatting 基础，可能有 R689/R716/R1085 类单位/属性 handling bug）；② C-dep 用户决策（CI 计费恢复后，word-spacing + line-height-applies-to 簇批量 flip）；③ letter-spacing-applies-to / white-space-processing 簇（CSS2/text，1-2%，可能 font-wall 或边界 handling，低 EV）。

### R1085 ★nbsp (U+00A0) 保留修复 LANDED = linebox Oracle +10（line-height-applies-to 簇 10 翻 PASS）+ css-text +1 + writing-modes +3·零 PASS→FAIL·clean win

承 R1084（plateau）。本轮 fresh R740 scan linebox dir（未近期深扫）找到 CSS 语义 bug 并 LANDED——R1082-R1084 三轮调查后首 landed code win。

**bug 发现（per-pixel）**：line-height-applies-to-001..015 簇 7+ 案 4.75% identical。line-height-applies-to-009（`<span style="display:block; background:blue; line-height:2in; width:1in">&nbsp;</span>`）：CHR blue bbox x[8..103] y[50..241]（192px=2in），**ZW blue bbox=None（完全不渲蓝）**。minimal repro 矩阵：line-height:2in + nbsp → 0 blue；line-height:2in + X → 18381 blue @192px；height:2in + nbsp → 18432 blue @192px → bug 仅当「高度来自 line-height」+「内容仅 &nbsp;」时元素塌缩 0。

**根因（CSS Text §4.1.1）**：U+00A0 (NO-BREAK SPACE) 是 **preserved** + **non-breaking**——不可折叠、不可作断行点。ZW 旧实现 `collapse_whitespace`（inline_types.rs:225）用 `ch.is_whitespace()` + `split_into_words`（mod.rs:1920）用 `text.split_whitespace()`，二者含 U+00A0 → nbsp 折成普通空格再被行首尾 trim → 仅含 nbsp 的元素 0 词 0 行盒 0 高 → 无 bg。

**修复（surgical，仅排除 U+00A0）**：`inline_types.rs` 新 `is_collapsible_ws(ch) = ch.is_whitespace() && ch != '\u{00A0}'`（首版 5-char CSS 窄集合回归 css-text 7 案——U+3000 IDEOGRAPHIC SPACE 等浏览器仍按可折叠/可断行；surgical 仅排 U+00A0 是 WPT 实证最优）。`collapse_whitespace` + `split_into_words` default 模式均改用之。+2 单测（nbsp_is_not_collapsible_ws + collapse_whitespace_preserves_nbsp）。

**A/B（stash 对照）**：linebox **121/190→131/190 net +10**（line-height-applies-to-001/002/003/004/007/009/012/013/014/015 全 4.75→0.95 PASS）/ css-text-decor 104→104 net 0 / css-text(excl decor) 248→249 net +1 / css-writing-modes 56→59 net +3。**合计 +14，零 PASS→FAIL**。

**已知 tradeoff（非 flip，仍 FAIL）**：css-text shaping-arabic-diacritics-002 10.81→16.50（+5.69pp）。该 test 明用 &nbsp;（注释「within the width of the NBSP」）+ Arabic 变音。nbsp 保留（CSS 正确）改变其与变音的 non-breaking 连接，但 ZW Arabic shaping 是 per-char（rustybuzz 未接生产，font-feature gap 谱系）→ spec-correct tradeoff，shaping gap 解后该案受益。

**门禁全绿**：make test 45 bin 0 failed（含 2 新 nbsp 单测）/ clippy --workspace --all-targets -D warnings 干净 / cargo fmt 干净 / product-smoke welcome **16.57%（== baseline，DC-13 <20% gate PASS）**。inline/mod.rs 2015 行（pre-existing >2000，本轮 +2 行，未重构——非本修复范围）。

**意义**：★ 纠正 plateau 悲观——R740 fresh scan（linebox，未近期深扫 dir）找到 clean CSS-correctness bug（nbsp 语义）。R1082-R1084 三轮调查后首 landed code win。forward = 继续 fresh scan 其它 under-mined dir（css-writing-modes 近-pass / CSS2 subdir）找同类 CSS 语义 bug。详见 [`evidence/r1085-nbsp-preserve-landed-2026-07-06.txt`](./evidence/r1085-nbsp-preserve-landed-2026-07-06.txt)。

**▶ 下会话**：① 继续 fresh R740 scan（writing-modes 近-pass / CSS2 subdir / css-tables）找 CSS 语义 bug（单位/属性 handling，R689/R716/R1085 类）；② font-wall C-dep 用户决策（CI 计费恢复后）；③ Phase A step-2（多 session）。

### R1084 font-wall C-dep (freetype-raster) yield 全表测绘完成 = +32（集中 text dir）+ CI 计费仍阻塞·零 net 源码·纯调查

承 R1083（multicol Phase A 死锁确证，pivot）。本轮完成 font-wall C-dep（最高 EV lever）的 yield 全表测绘，为用户决策补全证据。

**A/B 测绘（4 dir，fontdue default vs --features freetype-raster，z_vs_chr<1%）**：
| dir | fontdue→freetype | net | 分母 |
|-----|------------------|-----|------|
| css-text | 248→263 | **+15** | 1408 |
| css-text-decor | 104→117 | **+13** | 242 |
| css-fonts | 98→99 | **+1** | 282 |
| css-multicol | 155→158 | **+3** | 452（R1082） |
| **合计** | | **+32** | 2384 |

★ css-text 现测 +15（R1068 曾报 +24，不同 tree/oracle 状态）。yield **集中 text dir**（css-text+decor = +28 / +32 = 88%）。css-fonts +1（font-features/variant 89 例 = feature gap rustybuzz 未接生产，非光栅化）；css-multicol +3（R1082 证结构性 Phase A，非光栅化）。layout dir（position/tables/CSS2）预期 ~0（layout diff 非光栅化）。

**裁决**：C-dep yield = **moderate（~+32），非 transformative 批量 unlock**。aggregate ~10k 案 +32 ≈ +0.3pp。仍值得翻（css-text/decor +28 实在，strict 真通过率提升更高），但**勿以「batch unlock 大量 dir」为论据**。决策阻塞仍是：① policy（FreeType C 依赖，rusty_v8 先例）；② 跨平台编译验证（**CI billing 仍阻塞**——gh run 28755337078 6-target 全未启动「payments failed / spending limit」，R1082 报告的 billing 阻塞未解）。

**★ 战略**：rendering-compat 自主 plateau 再证——clean lever 尽（pending-clean-levers R740 实证）+ font-wall C-dep moderate yield blocked + 结构性 Phase A 多 session。R1082-R1084 三轮调查闭环：multicol Phase A 死锁（R1082/R1083）+ C-dep yield 全表 moderate（R1084）。

详见 [`evidence/r1084-freetype-cdep-yield-table-2026-07-06.txt`](./evidence/r1084-freetype-cdep-yield-table-2026-07-06.txt)。

**门禁**：纯调查（feature default-off，零 net 源码），make test 未跑。tree clean。

**▶ 下会话**：① **font-wall C-dep 用户决策**（CI billing 恢复后 dispatch workflow 取 6-target，全绿翻 default 落地 ~+32）；② **Phase A step-2 narrow slice**（R1083 确证 multicol-basic + welcome/morning 共同根 = paint Path B empty-styles 重跑度量偏差；照 R890 store_font_sizes override 模式，fragile-balance 风险须 narrow A/B）；③ fresh R740 找新 simple handling bug（R739-R840 已穷尽，低 EV）。

### R1083 multicol balance-inline 深挖：探针推翻「block children 绕过」首判（children 实 inline-level，paint-side fire）+ option A 复活 store 路径负面结果（11.24% 反退）+ 确证 Phase A 死锁·零 net 源码·纯调查

承 R1082 CONTINUE（Phase A balance-aware 列宽 IFC，先 per-pixel 重试 R902 balance 扩展）。本轮深挖 multicol-basic-001 残余机制，**option A 复活 store 路径实测负面（11.24% 反退），确证残余 = Phase A 死锁**。

**实验 1（放宽 store gate，env-gated MULTICOL_BALANCE_INLINE）**：store_inline_multicol_columns 放宽（balance + DOM-based has_block_child + distribute_lines_balanced）。MC_DEBUG 实测 store 成功存 21 行 @120px。但 diff 10.91%→10.91% 不变 → store 是死代码。

**实验 2（paint_text use_stored 探针，决定性）**：text.rs:938 探针实测 `multicol_info=true ifc_w=120 width_matches=false use_stored=false`。★ **推翻首轮「block children 绕过 paint」假设**——children 是 **inline-level**（非 block-mapped），has_in_flow_children=false（text.rs:807）→ multicol_info=Some（paint-side **正常 fire**），ifc_width=120。store 死代码双因：(a) multicol_info.is_some()（use_stored 要求 is_none()）；(b) width 不匹配（store 设 360，paint 用 120）。渲染由 paint-side 主导（重跑 IFC @120px + line.y/target_h balanced 分布 text.rs:1132-1181）。

**实验 3（option A 复活 store，env-gated，已 revert）**：① store 放宽 gate；② store inline_layout_width=col_width（balance）/content_width（auto）；③ text.rs:938 use_stored 去 multicol_info.is_none()。A/B multicol-basic-001：**10.91%→11.24%（+0.33pp 反退）**。→ 即使 store 正确算 21 行 @120px 且被 paint 消费，输出仍 11% 错。**复活 store 路径非 lever，rule out**。

**结论（honest）**：multicol-basic-001 残余 = Phase A inline ownership / 行盒分布 accuracy（distribute_lines_balanced ceil-split vs chromium balancing + line-height/font-metric + inline ownership 双绘协调）。非单 session 可解。R1082/R1083 两轮深挖确证 multicol balance-inline 残余属 Phase A 死锁 territory（同 R125/R198/R890 empty-styles 谱系）。

**★ 战略**：multicol 单 session 杠杆确尽（R1074-R1080 已收 +9 net；R1082 证残余结构性非 font-wall；R1083 option A 复活 store rule out + 确证 Phase A 死锁）。下会话应 **pivot**——非 multicol 角度，或 accept plateau 等 font-wall C-dep（CI 计费恢复后用户决策）。勿再投 multicol-basic balance-inline 单 session（option A/B 均 Phase A 谱系，empty-styles 重跑度量偏差是根）。

详见 [`evidence/r1083-multicol-paint-bypass-2026-07-06.txt`](./evidence/r1083-multicol-paint-bypass-2026-07-06.txt)。

**门禁**：纯调查（实验+探针 default-off revert），零 net 源码，make test 未跑。tree clean。

**▶ 下会话**：① **pivot 出 multicol**——R1081 已证 css-tables/css-flexbox clean lever 穷尽，转其它面（如 box-display R109 FR-002/003 font-wall gated / writing-modes / 产品 smoke fixture 健康）；② font-wall C-dep 用户决策（CI 计费恢复后取 6-target evidence；当前 blocked）；③ 若重启 multicol，须 Phase A empty-styles 重跑度量统一（R890 store_font_sizes override 模式，多 session），非单点 gate 改。

### R1082 ★css-multicol 残余 = 结构性（非 font-wall）三证 + multicol-basic 内容丢失精确诊断 + CI 计费阻塞 freetype 6-target 验证·零 net 功能源码·纯调查（fmt cleanup 已提交 2defd817）

承 R1081（C-dep 用户决策点）。本轮自主验证「multicol 近-pass 是否真 font-wall 主导」——**裁决：推翻长期假设，multicol 残余主导 = 结构性（Phase A inline 列宽测量），font-wall C-dep 对 multicol 仅 +3（非批量 unlock）**。

**证 1：freetype-raster feature A/B（css-multicol 全量 452 案）**：fontdue baseline 155/452 (34%) → `--features zero-render-foundation/freetype-raster` 158/452 (35%)，**NET +3**（css-text R1068 是 +24）。若 multicol 真为 font-wall，freetype 应批量 unlock。实际 +3 = 噪声级。逐案 worst-case（fontdue→freetype）：multicol-basic-001..004 10.91→10.86（−0.05）/ multicol-columns-001..006 簇 9.95→9.95（0.00）/ nested-balancing-004 17.17→17.17（0.00）/ span-all-rule-002 26.25→26.25（0.00）→ 全 worst-case 几乎不动 = 非 font-wall。

**证 2：per-pixel multicol-basic-001 几何重建**：3 inline span（紫/橙/蓝各 28 Ahem-X）在 columns:3 w:360 gap:0。ZW 每色 ≈18 字形（丢 36%），chromium ≈28（精确）；ZW 文本 4 行 y=40–120，chromium 6+ 行 y=72–184；ZW 3 列紧挤 x=8–230，chromium 3 列展开 x=8–320 带 yellow 间隙。

**证 3：MC_DEBUG layout_multicol 结构 dump**：`container_w=360 content_h=140 col_count=3 child_info=[(0,60),(1,60),(2,60)]`，各列一匿名块子元素 visual_height=60（3 行@20px）。但列宽 120px 下每 span 需 6 行=120px → 后 3 行从未计算/存储（非裁剪丢，paint 非 breaking 不裁高 painter/mod.rs:862-863）。

**根因（精确）**：multicol 容器 inline 内容被包成匿名块子元素，各在自身宽（=容器宽 360px）下测 IFC → 3 行。layout_multicol 用此 60px 分配列。paint 在列宽 120px 渲染存储 IFC（360px 算的 3 行），120px 应有的另 3 行不存在。架构事实：`inline_finalization.rs:530-536` multicol 容器永远 early-return（仅试 R900 store_inline_multicol_columns，gate 排除 balance 模式），依赖匿名块子元素各自存（@容器宽非列宽）。R900 gate（line 191-194）：仅 column-fill:auto + available_height>0 + inline-only。multicol-basic-001 是 balance 默认 → 不命中 → 无列宽存储。

**为何 R902「balance 扩展零 yield」（inline_finalization.rs:191 注释）**：R902 试扩 balance，零 oracle-pass yield 回退。推测真因：store_inline_multicol_columns 用 fragment_lines_into_columns + ColumnFillMode::Auto（顺序填满一列再下一列），balance 应均布，顺序填语义错 → 不 flip。R902 量 oracle-count（非 per-pixel delta），可能 per-pixel 改善但未跨 <1% 阈值 → 「零 yield」掩盖机制改善。真修须 balance-aware fragmentation（行均布），非简单去 sequential_fill gate。

**意义（战略纠偏）**：① css-multicol 残余 ≠ font-wall，C-dep 不会 batch unlock 138；② 真 multicol lever = Phase A inline 列宽测量统一（匿名块子元素须列宽测 IFC），多 session（phase-a-IFC-unification-design.md 37KB）；③ font-wall C-dep 价值重估：css-text +24（R1068）+ multicol +3（本证）= 主要收益在文字 dir，布局 dir 受益极小，C-dep 仍值得翻但勿以「batch unlock multicol」为论据；④ CI 计费阻塞（gh run 28754164214：freetype-raster-cross-platform 6-target 全未启动「account payments failed / spending limit」）→ 6-target 验证路径当前不可用，C-dep 决策双重门（policy + CI 验证）均非自主可解。

**门禁**：本轮纯调查（探针加已撤），make test 未重跑（无源码变更）；R1082 fmt cleanup commit 2defd817（loader.rs/apply_advanced.rs/tests/core.rs cargo fmt 对齐，cargo fmt --check 过，pre-commit-guard PASS）。

详见 [`evidence/r1082-multicol-structural-not-fontwall-2026-07-06.txt`](./evidence/r1082-multicol-structural-not-fontwall-2026-07-06.txt)。

**▶ 下会话**：① **Phase A balance-aware 列宽 IFC**——先 per-pixel 重试 R902 balance 扩展（量 delta 非仅 count）验证机制改善，再投资 balance 行均布 fragmentation（潜在 unlock multicol-basic-001..004 + columns 簇）；② font-wall C-dep 用户决策（CI 计费恢复后取 6-target evidence；当前 blocked）；③ positioned-in-multicol 残余（abspos CB 在 multicol 外，R1080 下会话 ①）。

### R1081 FreeType C-dep CI job 抽独立 workflow（一键 dispatch，不连带全量 CI）+ css-tables/css-flexbox 扫描确认无 clean lever（font-wall/complex）·零 net 功能源码·CI 配置 + 纯调查

承 R1080（multicol positioned-flush LANDED，multicol 可处理机制 +9 net 收官）。本轮 pivot R740 扫其它 dir + 改善 C-dep CI 就绪度。

**css-tables / css-flexbox 扫描（R740 策略）**：css-tables 仅 3 案 >5%（table-cell-width-0 20% R109 territory / baseline-vertical 12% font-wall / row-group-order 5%）。css-flexbox 295/497（59.4%）top-worst：flex-flow-001/002 23%（12+ flex-flow 变体矩阵）/ flex-abspos-inset-nested 19%（R850 territory）/ writing-mode 11% 簇（R114）/ 等。**minimal 实测确认 ZW flex 机制全工作**：flex-direction:row-reverse ✓（B,G,R 视觉序）、column ✓（R,G,B 垂直序）、flex-wrap ✓（B 换行 row2）、flex-flow shorthand 已解析（shorthand/mod.rs:215）。故 flexbox top-worst 全 font-wall（Ahem/sans-serif 文本）或 complex（abspos-nested），无 clean 机制 lever。

**FreeType C-dep CI job 抽独立 workflow（R1081）**：R1073 把 freetype-raster-cross-platform job 嵌在 ci.yml（dispatch ci.yml 连带跑全量 build-and-test+reftest+benchmarks 三 job，非「一键」轻量）。本轮抽成独立 `.github/workflows/freetype-raster-cross-platform.yml`（仅 workflow_dispatch，6-target matrix，continue-on-error，仅 check zero-render-foundation --features freetype-raster），ci.yml 恢复三主 job（build-and-test/reftest/benchmarks）。两 YAML 合法（python yaml 解析 OK）。→ C-dep 决策的 6-target 编译验证现可 `gh workflow run freetype-raster-cross-platform` 独立轻量 dispatch（不浪费全量 CI 配额）。

**裁决**：C-dep（最高 EV lever）evidence 完备（R1068 +24 css-text / R1071 CI de-risk bundled / R1073 job），CI 就绪（R1081 独立 dispatchable），用户决策点。自主侧 rendering-compat 全景 plateau 再证：css-multicol（R1074-R1076+R1080 +9 net 收官）+ css-tables（R109/font-wall）+ css-flexbox（font-wall/complex）+ box-display（R109 FR-002 font-wall）。**clean single-session 机制 lever 跨 dir 穷尽**——残余 = font-wall C-dep（用户决策）+ 多 session 结构性（Phase A IFC inline 跨列 deadlock / nested fragmentation / R109 FR-002/003）。

**▶ 下会话**：① **font-wall C-dep 用户决策**（dispatch freetype-raster-cross-platform workflow → 6-target 编译结果 → 全绿则翻 crates/render-foundation/Cargo.toml default=["freetype-raster"]，batch unlock css-multicol 近-pass 138 + box-display + css-text + linebox font-wall band）；② Phase A IFC inline 跨列（已知 deadlock，多 session）；③ nested multicol fragmentation（nested-balancing-004 17%）；④ R109 FR-002/003（font-wall gated）。clean single-session lever 跨 dir 穷尽，forward = C-dep 用户决策 或 多 session 结构性。

### R1080 ★multicol 列子元素 positioned 后代本地 flush+clip LANDED = css-multicol Oracle 151→154（+3 net）·multicol-overflow-clip-positioned 16→0 PASS·paint-containment-001/oof-nested 各 PASS·scroll-content −3.6pp·零回归（消 R1079 over-render 12 worsened）

承 R1079（multicol 列子元素 positioned 后代 drop bug 定位 + 简单 collection 修复 over-render 已回退，须 clip-context flush）。本轮实现**本地 flush + 列子元素 overflow box clip**，clean win。

**驱动案 multicol-overflow-clip-positioned（16%）**：`columns:2 > div(h:200,overflow:hidden) > div(bg:blue,h:800,position:relative)`。修复前 0 blue（position:relative 后代 drop）；R1079 简单 collection → over-render 800px（16→31%更差）；R1080 blue 渲 + 裁到列子元素 overflow box → **0.00% PASS**。

**修复**（paint/painter/mod.rs multicol 列循环）：每个列子元素（column_span_offsets 非空）每个片段，paint_node(child) 后：① collect_positioned_descendants(child) 收集列子元素的 positioned 后代；② 本地 flush（paint_node 每个后代）；③ 若列子元素 overflow≠visible，**裁到列子元素 padding box**（Rect(frag_abs+border, padding+content+padding)，同 paint_node needs_clip 公式）；④ 既有的列 clip（counts_before_frag..列宽+gap/2）。★ 外层 collect_positioned_descendants 仍 skip 列子元素（line 184）→ 无 double-paint + 无外层 over-render。clip-context：本地 flush 带 (a) 列子元素 overflow 裁 + (b) 列裁。

**A/B（stash 对照 R1076，ORACLE_DUMP_ALL per-case 452 案）**：baseline(R1076) 151/452 → **154/452（net +3；headline 155 34.3%）**。**3 flip PASS**：multicol-overflow-clip-positioned **16.00→0.00** / paint-containment-001 **2.79→0.73** / oof-nested-in-single-column **1.73→0.73**。**0 flip FAIL**。**1 improved**：multicol-scroll-content **7.81→4.17(-3.64pp)**。**0 worsened ≥0.5pp**。★ 零回归——R1079 简单 collection 的 12 worsened（overflow-clip-positioned +15 / scroll-content +6 / 9 abspos/fixed-in-multicol）经 clip-context flush 全消（abspos/fixed 案本地 flush+列 clip 后 unchanged/improved）。

**门禁全绿**：1 新 R1080 单测（r1080_multicol_column_positioned_descendant_not_dropped，pipeline 渲 minimal case 断言 blue fill 存在）/ multicol 26 + paint 630 测全过 / **make test 全 workspace 45 binary 0 failed** / clippy --workspace --all-targets -D warnings 干净 / cargo fmt 干净 / **product-smoke welcome 16.57%（<20% gate 不变）**。

**意义**：解 R1079 定位的 multicol 列子元素 positioned 后代 drop 真 bug（clip-context flush 方案，纠正 R1079 简单 collection over-render）。multicol positioned-in-column 簇（overflow-clip-positioned/paint-containment/oof-nested/scroll-content）解锁。paint-deferral + multicol + overflow 三方交互的 clip-context 模型建立。累计 css-multicol R1074(002)+R1075(balance)+R1076(sequential)+R1080(positioned-flush)：Oracle **145→154（+9 net）**。详见 [`evidence/r1080-multicol-positioned-flush-clip-landed-2026-07-06.txt`](./evidence/r1080-multicol-positioned-flush-clip-landed-2026-07-06.txt)。

**▶ 下会话**：① positioned-in-multicol 残余（abspos/fixed CB 在 multicol 外的案，如 abspos-containing-block-outside-spanner，本地 flush+列 clip 或须 CB-aware）；② font-wall C-dep（最高 EV，用户决策点）；③ nested multicol fragmentation（nested-balancing-004 17%）。

### R1079 multicol 列子元素 positioned 后代 drop bug 定位 + collection 修复 over-render 已回退·须 clip-context flush（多 session）·零 net 源码·纯调查

承 R1078（plateau，寻 fresh 结构 lever）。本轮 PIL 定位 multicol-overflow-clip-positioned（16%）= **position:relative 内容在 multicol 列子元素（overflow:hidden）内完全 drop**，根因 + 修复方向明确但须 clip-context（多 session）。

**bug 隔离（minimal repro）**：`columns:2 > div(overflow:hidden,h:200) > div(bg:blue,h:800,position:relative)` → ZW 渲 **0 blue**（全丢）；position:static 同结构 → 38400 blue（正确）；无 multicol 同结构 → 160000 blue（渲染，over-clip 另一 pre-existing bug）。即 **multicol + 列子元素 + positioned 后代 = drop**。

**根因（paint/painter/mod.rs collect_positioned_descendants line 184-186）**：multicol 列子元素（column_span_offsets 非空）被 `continue` 跳过（交 multicol 循环绘制）。但其 positioned 后代（blue, position:relative）须由**外层 scope** 收集 flush——列子元素自身非 scope（overflow:hidden 不建 SC, creates_stacking_context 仅 positioned+z-index/opacity<1, engine.rs:1206），paint_node(列子元素) 的 steps 3/4/5 排除 positioned 子（line 777 `!is_positioned_child`），steps 2/6/7 仅 scope 跑（line 762/784/794 `if is_scope`）→ positioned 后代**无人收集 flush = drop**。无 multicol 时外层 root scope 经 collect_positioned_descendants 递归进列子元素收集 blue flush（line 209），故工作。

**修复实验（已回退）**：collect_positioned_descendants line 184-186 改 `continue` 为 `collect_positioned_descendants(child, ...) + continue`（递归收集列子元素的 positioned 后代）。**A/B（stash 对照 R1076）net +1（151→152）但质量净负**：1 flip PASS（oof-nested-in-single-column 1.73→0.73）/ 0 flip FAIL / **12 worsened**：multicol-overflow-clip-positioned **+15.36**（blue 现 over-render 未裁剪到 overflow:hidden 200px）/ multicol-scroll-content +6.09 / multicol-nested-032/033 / abspos-containing-block-outside-spanner / abspos-autopos-contained-by-viewport / abspos-multicol-in-second-outer-clipped / fixed-in-nested-multicol-with-transform / fixedpos-static-pos-with-viewport-001/002/003。收集的 positioned 后代经外层 scope flush **未带列子元素的 overflow/column clip** → over-render。

**裁决回退**：按 code-guidelines 回退（1 PASS 不抵 12 worsened 含 +15pp）。★ 正确修复须 positioned 后代 flush 时**继承列子元素的 clip context**（overflow:hidden + column clip），即 flush 不能简单交外层 scope，须在列子元素的 clip 栈内 flush——深 paint 架构改（positioned flush 携带 clip context），多 session 结构性。

**意义**：定位 multicol 列子元素 positioned 后代 drop 真 bug（paint deferral + multicol 交互），修复方向明确（collect + clip-context flush）但非单 session。+ oof-nested-in-single-column 证实机制（positioned 后代收集后正确渲染当无 overflow 裁剪需求）。multicol-overflow-clip-positioned（16%）+ multicol-scroll-content + 9 abspos/fixed-in-multicol 案 = positioned-in-multicol-column 簇，待 clip-context flush 多 session 解。

**▶ 下会话**：① **positioned-in-multicol-column clip-context flush**（多 session 结构性——flush 携带列子元素 overflow+column clip，解 overflow-clip-positioned 16% + scroll-content + abspos/fixed-in-multicol 簇）；② font-wall C-dep（最高 EV，用户决策点）；③ nested multicol fragmentation（nested-balancing-004 17%）。

### R1078 R109 §9.2.1.1 FR-002 评估 = 驱动案 onload JS 实际工作（R1078 首判「JS-gated」纠正）+ 残余 font-wall Ahem 主导（K->F 21940px）+ FR-002 bg/margin 仅 ~800px·FR-002 不 flip·零 net 源码·纯调查

承 R1077（css-multicol plateau，pivot R109）。本轮评估 R109 FR-002（容器 bg 涂满匿名块/margin 区）。

**★ 纠正首判「JS-gated」**：insert-block-in-inlines-beginning-001 的 `<body onload="insertABlockAtBeginning()">` **实际工作**——reftest render path 的 `apply_scripted_dom_mutations`（reftest_scripts.rs，V8Sandbox + DOM shim + DomMutation 记录 + apply_mutations_to_html）正确执行 onload，记录 6 个 mutation（CreateElement div → AppendChild CreateTextNode → SetAttr class=inserted → **InsertBefore 到 div.container 前 #insertion-point** → SetAttr html class="" 移除 reftest-wait），apply 后 container 1 HTML = `<div class=inserted>Inserted new block</div><span id=insertion-point>Several</span>...`（与 container 2 静态结构一致）。**onload JS 路径（R988 基建）功能完备**，案非 JS-gated。

**残余 6.32% 组成（PIL）**：主导 **K->F 21940px**（ZW Ahem 文本 vs CHR fuchsia bg，即匿名块盒内 "Several inline elements..." 文本 y 定位差，font-wall）+ F->K 2060 + margin/bg 区 ~800px（inserted block margin:1em 0 区 ZW 白 / CHR fuchsia，FR-002 + margin-collapse 语义）。

**裁决**：FR-002（容器 bg 涂满）+ margin-collapse 精度修仅消 ~800px（minor），**不 flip**（font-wall 21000px 主导）。R109 FR-002 非当前 flip lever——① 驱动案 onload JS 工作（纠正 R1078 首判），但残余 font-wall Ahem 主导；② box-display 近-flip（anonymous-box-generation-001 1.11% 等）全 font-wall（font-metric line-height 差）；③ FR-002 bg/margin 是 minor 分量。

**★ rendering-compat 全景 plateau 确认（再证）**：css-multicol（R1074-R1076 +6 net 收官）+ box-display（R109 FR-001 done，FR-002/003 驱动案 onload JS 工作但残余 font-wall）+ 各 dir 近-pass ~1% band 全 font-wall。**最高 EV 下一 lever = font-wall C-dep（FreeType default flip）**：batch unlock css-multicol 138 案 + box-display 近-pass（含 insert-block 等 onload 案，JS 已工作，font-wall 解即 flip）+ css-text/linebox 等。用户决策点（R1072-R1073 evidence 完备 + CI smoke job `freetype-raster-cross-platform` 就绪待 dispatch）。

**▶ 下会话**：① **font-wall C-dep**（最高 EV，用户决策点）——用户 dispatch CI job 验证 6-target 后翻 default，或 rally 据 evidence 建议；② R109 FR-002 bg/margin 精度（font-wall gated，低 EV，可作 spec-correctness 随手修但非 flip lever）；③ 结构性 nested multicol fragmentation（nested-balancing-004 17%，多 session）。

### R1077 column-wrap 解析评估 = LOW-EV（所有驱动案 nested/complex，0 flip 可期）+ R1075 2 worsened 非 column-wrap 亦非 tractable·column-wrap:wrap 垂直换行 chromium-confirmed·零 net 源码·纯调查

承 R1077（css-multicol plateau，pivot R109）。本轮评估 R109 FR-002（容器 bg 涂满匿名块/margin 区），**裁决：驱动案 JS-gated，无 clean 静态 driving case，FR-002 非 flip lever**。

**主驱动案 insert-block-in-inlines-beginning-001（6.32%）= JS-gated**：`<body onload="insertABlockAtBeginning()">` + `class="reftest-wait"` —— chromium onload 把 `<div class=inserted>` 插入第一 container 的 inline 前（触发匿名块盒生成）。ZW 不执行 onload JS（reftest-wait/onload 是已知 feature gap）→ 第一 container 渲为纯 inline（无 inserted block），与 CHR（有 block）结构性错位。PIL 实测 6.32% 残差主导 = **K->F 21940px**（ZW 文本 vs CHR fuchsia bg，即 JS-mismatch 致首 container 文本在错 y）；FR-002 bg-gap（inserted block margin 区 ZW 白 / CHR fuchsia）仅 ~800px（minor）。FR-002 完修亦不 flip（JS-mismatch 主导）。

**box-display 近-flip 案全 font-wall**：anonymous-box-generation-001（1.11%，静态 case-b 但容器**无 bg**）残差 = 文本定位 + 2px blue stripe 偏移（ZW blue y=70 vs CHR y=68，font-metric line-height 差，font-wall），非 bg-gap。block-in-inline-001/002（10.55%，case-a split inline border FR-003）、box-generation-002（7.99%）同 font-wall 主导。

**裁决**：R109 FR-002（容器 bg 涂满）非当前 flip lever——① 主驱动案 JS-gated（须先 onload/reftest-wait JS 执行，独立 feature 多 session）；② 静态 case-b 近-flip 全 font-wall（Ahem metric，FreeType C-dep）；③ FR-002 bg-gap 本身是 minor 分量。FR-003（split inline border）同受 font-wall 主导（block-in-inline-001/002 10.55% 非 border 单点）。

**★ rendering-compat 全景 plateau 确认**：css-multicol（R1074-R1076 +6 net 收官）+ box-display（R109 FR-001 done，FR-002/003 驱动案 JS-gated/font-wall）+ 各 dir 近-pass ~1% band 全 font-wall。**最高 EV 下一 lever = font-wall C-dep（FreeType default flip）**：batch unlock css-multicol 138 案 + box-display 近-pass + css-text/linebox 等（R1068 +24 css-text 实证），用户决策点（R1072-R1073 evidence 完备 + CI smoke job `freetype-raster-cross-platform` 就绪待 dispatch）。

**▶ 下会话**：① **font-wall C-dep**（最高 EV，用户决策点）——用户 dispatch CI job 验证 6-target 后翻 default，或 rally 自主据 evidence 建议；② **JS 执行 feature**（onload/reftest-wait，独立多 session，unlock insert-block-in-inlines 等多案）；③ R109 FR-003 block-in-inline border（font-wall gated，低 EV）；④ 结构性 nested multicol fragmentation（nested-balancing-004 17%，多 session）。勿单 session 投 R109 FR-002（驱动案 JS-gated）。

### R1077 column-wrap 解析评估 = LOW-EV（所有驱动案 nested/complex，0 flip 可期）+ R1075 2 worsened 非 column-wrap 亦非 tractable·column-wrap:wrap 垂直换行 chromium-confirmed·零 net 源码·纯调查

承 R1076（▶ 下会话列 column-wrap 解析为候选）。本轮评估 column-wrap 解析的 yield，**裁决 LOW-EV，勿作下 lever**。

**column-wrap:wrap 垂直换行 chromium-confirmed**：minimal `<article columns:2 column-fill:auto column-wrap:wrap height:60><div height:200>` → chromium green **下溢**（article 下 4000px，0 右溢出），ZW（R1076 inline-overflow）green **右溢**（article 右 4000px，0 下溢）。证实 column-wrap:wrap 改 overflow 方向（inline→block）。

**★ 所有 column-wrap:wrap 驱动案均 nested/complex（0 flip 可期）**：
- column-height-004/025/026/027（R1076 worsened 1.96→3.21 等）：**全 nested multicol**（inner multicol column-height + column-wrap:wrap + absolute overlays）+ column-height（ZW 亦未解析）。仅 column-wrap 解析不 flip（须 nested fragmentation + column-height + overlay 三者同修）。
- column-height-003（R1075 worsened 1.54→2.38）：column-wrap:wrap + absolute overlay，同病。
即 column-wrap 解析本身（4-5 文件：parser + ComputedStyle + apply + registry + layout）对 WPT 通过率 **0 flip**（驱动案全卡在更深的 nested/column-height/overlay）。

**R1075 2 worsened 复核（非 column-wrap，亦非 tractable）**：
- column-height-003：column-wrap:wrap（上）。
- multicol-span-all-children-height-010（1.75→2.36）：columns:10 + nested span + column-span:all negative-margin + orphans/widows + 10 inline-block —— 10 列复杂布局，R1075 inline-overflow 与 10 列协同偏差，非单点可修。

**裁决**：column-wrap 解析 LOW-EV（0 flip），**勿作下 lever**。可选 cleanup：解析 column-wrap 仅用于 R1075/R1076 gate detect-and-skip（column-wrap:wrap 案退回 drop-overflow 即 pre-R1075 值，消 ~5pp 小回归，但案仍 FAIL，0 flip，borderline 不做）。css-multicol 经 R1074-R1076（+6 net）三路径 inline-overflow 全 land，残余 = nested fragmentation（nested-balancing-004 17% 多 session）/ font-wall（138 案 ~1% band，FreeType C-dep 用户决策点）/ draft feature（column-wrap/column-height css-multicol-2，全 nested 卡深）。

**▶ 下会话**：css-multicol 暂 plateau（R1074-R1076 +6 net 收官），**pivot R109 §9.2.1.1 匿名块**（central 结构性 lever，FR-001 done，剩 FR-002 容器 bg 涂满 / FR-003 split inline border 归属，有 spec）或 font-wall C-dep（用户决策点）。勿投 column-wrap 单 session（LOW-EV）。

### R1076 ★column-fill:auto sequential 路径 inline 列溢出 LANDED（+!has_nested_multicol 守卫）= css-multicol Oracle 150→151（+1 net）·column-height-011 2.28→0.73 PASS（column-wrap:auto 默认）·0 翻 FAIL·1 worsened（column-wrap:wrap unsupported feature 仍 FAIL）·承 R1076v1 net-negative 回退后 gate 扩展

承 R1076v1（sequential inline-overflow net-negative 回退，gate 须扩展）。本轮加 `!has_nested_multicol` 守卫（R1035 先例）消 6 nested 回归 + nested-028 翻 FAIL，**v2 LANDED**。

**gap 真实（chromium-confirmed）**：minimal `<article columns:1 column-fill:auto height:100><div height:200>` → chromium green x=13..207（右外侧溢出列），ZW 仅 x=13..107（drop overflow）。`assign_children_to_columns_with_breaking` 在 col_count 处 break 丢弃 overflow（同 R1075 balance 前病）。

**v1 回退根因精确定位**：① **nested multicol**（multicol-nested-019/014/021/028 + nested-column-rule-002 + nested-past-fragmentation-line 6 案 + nested-028 翻 FAIL）——子元素自身 column-count/width，nested fragmentation 须独立模型；② **column-wrap:wrap**（column-height-004/025/026/027 4 案）——ZW 未解析 column-wrap（css-multicol-2 draft），垂直换行语义不覆盖；③ column-height-004/025/026/027 实为 column-wrap:wrap（grep 确认 2 mentions 各），非 nested 但属 unsupported feature（ZW 不可解，gate 跳过无意义）。fill-balance-003 经 `!has_nested_multicol` 守卫后亦消（其含 nested sequential 子）。

**实现（v2 LANDED）**：layout_multicol `sequential_fill && height_limit>0` 分支前置 gate `total > col_count×height_limit && !monolithic && !forced_break && !has_nested_multicol` → fire 时用 `assign_children_to_columns_multirow`（以 height_limit 顺序填 + 超出 push 新列）。has_nested_multicol 查子元素 column_count=Number/column_width=Length（同 R1035 spanner 路径）。minimal 测试 A/B 0.00% 完美匹配 chromium。

**A/B（stash 对照 R1075，ORACLE_DUMP_ALL per-case 452 案）**：baseline(R1075) 150/452 → **151/452（net +1；headline 152 33.6%）**。**1 flip PASS**：column-height-011 **2.28→0.73**（column-wrap:auto 默认 inline 溢出）。**0 flip FAIL**（nested-028 守卫消）。**0 improved 非 flip**。**1 worsened 非 flip**：column-height-004 1.96→3.21（+1.25，column-wrap:wrap unsupported feature，仍 FAIL——ZW 不解析 column-wrap，垂直换行不可实现，本路径不可解，记录待 column-wrap 解析支持）。

**门禁全绿**：2 新 R1076 单测（r1076_sequential_fill_auto_inline_overflow + r1076_nested_multicol_child_guarded_sequential，后者断言 nested 子不产生 col_x>0 inline 溢出）+ R1074/R1075 单测 + 25 multicol 测全过 / **make test 全 workspace 45 binary 0 failed** / clippy --workspace --all-targets -D warnings 干净 / cargo fmt 干净 / **product-smoke welcome 16.57%（<20% gate 不变）** / multicol.rs 1370 行（<2000）。

**意义**：R1074（spanner）+ R1075（非 spanner balance）+ R1076（非 spanner sequential）三路径 inline 列溢出全 land，累计 css-multicol **145→151（+6 net）**。sequential 路径守卫演进示范：nested multicol 须独立 fragmentation 模型（同 R1035 spanner 路径先例），inline-overflow 不越界。column-wrap:wrap（unsupported）残余为 column-wrap 解析多 session lever。

**▶ 下会话**：① column-wrap 解析支持（css-multicol-2 draft）→ column-height-004/025/026/027 等垂直换行案可修（独立 feature lever）；② nested-balancing-004（17%，nested fragmentation，多 session）；③ css-multicol 近-pass 138 案 ~1% band = font-wall（FreeType C-dep 用户决策点，解锁批量 flip）；④ 2 小 R1075 worsened（column-height-003 / span-all-children-height-010）逐案精度。

### R1075 ★非 spanner balance 路径 inline 列溢出 LANDED = css-multicol Oracle 146→150（+4 net）·4 flip PASS（column-height-008/fill-balance-029/nested-030/restyle-002）·nested-balancing-004 −20pp/002 −11pp 大改善·monolithic 守卫消 overflow-unsplittable 回归·零翻 FAIL

### R1075 ★非 spanner balance 路径 inline 列溢出 LANDED = css-multicol Oracle 146→150（+4 net）·4 flip PASS（column-height-008/fill-balance-029/nested-030/restyle-002）·nested-balancing-004 −20pp/002 −11pp 大改善·monolithic 守卫消 overflow-unsplittable 回归·零翻 FAIL

承 R1074（spanner 路径 inline 列溢出 LANDED）。本轮把同一 inline-overflow 语义扩展到 **非 spanner balance 路径**（minimal multicol 同病：height:50 col-count:2 child:200 → ZW 仅渲 article 内 2 列 drop overflow）。

**chromium 确认**（product-oracle-shot.mjs + /usr/bin/chromium）：minimal `<article columns:2 width:400 height:50><div height:200>` → chromium yellow x=13..799（article 右外侧 2 溢出列）→ 非 spanner balance 亦走 inline 溢出（列高 cap 容器高度，超出内容生成额外 column box 向右）。

**修复**（`crates/layout-engine/src/multicol.rs` layout_multicol balance 分支）：旧 balanced target=total/col_count 在 col_count 处 break 丢弃 overflow。R1075 前置 gate `total > col_count×container_height && content_height>0 && !has_monolithic_child` → fire 时改用 `assign_children_to_columns_multirow(child_info, col_count, container_height)`（以 container_height 作 max_col_height 拆片段，超 col_count 自动 push 新列），定位走既有 row_height=0（inline 向右）。gate 不 fire 时字节同。

**★ monolithic 守卫（关键）**：overflow≠visible 子元素不可分（CSS Fragmentation）。multirow 拆分超高子元素对 monolithic（overflow-unsplittable 的 overflow:scroll+200px 孙）错。`has_monolithic_child` 查 `children[idx].overflow_x/y != OverflowClip::Visible`，有 monolithic 子退回 balanced（R1037 gate 不拆 auto-height/monolithic）。

**A/B（stash 严格对照，ORACLE_DUMP_ALL per-case 452 案）**：baseline（R1074）146/452 → **150/452（net +4；headline 151/452 33.4%）**。**4 flip PASS**：column-height-008/fill-balance-029(2.80→0.73)/nested-030(2.79→0.73)/span-all-restyle-002(1.11→0.93)。**0 flip FAIL**（monolithic guard 消 overflow-unsplittable-001/002 回归）。**4 improved 仍 FAIL**：rule-nested-balancing-004 **37.67→17.17(-20.5pp)**/002 15.10→4.10(-11pp)/span-all-children-height-007 6.41→3.25/with-custom-layout 3.57→2.39。**2 worsened 非 flip 小**：column-height-003 +0.84/span-all-children-height-010 +0.61（仍 FAIL）。

**门禁全绿**：2 新 R1075 单测（r1075_non_spanner_balance_inline_overflow + r1075_monolithic_child_not_split）+ R1074 单测 + 25 multicol 测全过 / **make test 全 workspace 45 binary 0 failed** / clippy --workspace --all-targets -D warnings 干净 / cargo fmt 干净 / **product-smoke welcome 16.57%（<20% gate 不变）** / multicol.rs 1308 行 / r717 测 490 行（均 <2000）。

**意义**：R1074（spanner）+ R1075（非 spanner balance）合覆盖 multicol inline 列溢出全路径，纠正旧模型（balanced break 丢弃 / multirow 向下堆叠）→ chromium 对齐（列恒 inline，overflow 向右）。累计 R1074+R1075：css-multicol **145→150（+5 net）**。monolithic 守卫示范 inline-overflow 拆分须尊重 CSS Fragmentation 不可分。详见 [`evidence/r1075-non-spanner-inline-overflow-landed-2026-07-06.txt`](./evidence/r1075-non-spanner-inline-overflow-landed-2026-07-06.txt)。

**▶ 下会话**：① 2 小 worsened（column-height-003 / span-all-children-height-010）逐案查精度；② nested-balancing-004 残余 17%（inline-overflow 已 -20pp，残余 column-rule 交互/nested 结构）深挖；③ css-multicol 近-pass 138 案 ~1% band = font-wall（FreeType C-dep 解锁后批量 flip，用户决策点）；④ inline-overflow 扩展到 column-fill:auto sequential 路径（如有同类 drop）。

### R1074 ★multicol inline（水平向右）列溢出 LANDED = css-multicol Oracle 145→146（+1 net）·span-all-children-height-002 3.99→0.29 PASS·003 17.26→1.03（-16pp 近 flip）·零回归·纠正 R1035「垂直 multi-row」误模型

承 R1039 CONTINUE（span-all-children-height 簇残余，记录为「multi-row+breaking 协同精度，多 session」）。本轮 per-pixel 定位证伪 R1039 归因，**真根因 = overflow 方向**：chromium 把超出 col_count×列高的内容放到 **inline 方向（容器右外侧）**，ZW 旧 multi-row 模型放到下方再被 R1039 slice-clip 隐藏 → 内容丢失。

**驱动案 multicol-span-all-children-height-002**（article column-count:2 height:200 + spanner + block2 height:100%=200px）：region1（spanner 下剩 50px）block2 应以 50px 列高拆 4 列（2 in-article + **2 右溢出**，同 y 单行）。ZW 修复前 R1035 multi-row 把 col2/col3 放下方行 y=213-263，R1039 slice-clip 隐藏 → block2[100:200] 丢失（pixel diff 主导 `.->Y` 17750 px @ x=429-620/637-799 article 右外侧，z_vs_chr 3.99%）。

**per-pixel 定位**（product-smoke + PIL）：dense x-scan 示列结构同（双黄），precise transition `. -> Y` 17750 px 全集中 article 右外侧 → CHR 渲 4 列右溢，ZW 不渲。minimal multicol（无 spanner，height:50 col-count:2 child:200）同病（ZW yellow 仅 x=13-412，右侧 0 列）→ 通用 inline-overflow 缺口非 spanner 特有。

**关键发现**：`assign_children_to_columns_multirow` **已**支持列数增长超 col_count（advance_col! push 新列），4 片段几何正确。bug 纯在定位：`position_multicol_children` 用 row_height>0 时 col_in_row=col_idx%col_count（wrap，垂直堆叠）；row_height=0 时 col_in_row=col_idx（单调递增，inline 向右）。

**修复**（`crates/layout-engine/src/multicol.rs`，1 行净 + 注释 + 1 单测）：`layout_multicol_with_spanners` multirow 分支 assign 仍以 region_available 作 max_col_height（列高，block2→4×50px 片段不变），**定位传 row_height=0.0**（旧 region_available）→ 溢出列落 col_idx×(col_w+gap) 的 x（容器右外侧），同 y_base 单行；region_height=max 片段高=50px（block 方向不增高）。CSS Multicol：definite 高度容器内容超 col_count×列高时额外 column box 在 inline 方向溢出——本修复对齐此语义。

**A/B（stash 严格对照，ORACLE_DUMP_ALL per-case 452 案）**：baseline（当前 tree）= **145/452**（注：R1039 记 140，tree 已漂移 +5）→ with-change **146/452（net +1）**。**1 flip to PASS**：002 **3.99→0.29**；**0 flip to FAIL**；**1 improved**：003 **17.26→1.03（-16.23pp 近 flip）**；**0 worsened ≥0.5pp**。★ 零回归。

**门禁全绿**：multicol 单测 25 pass（含新 `test_position_multicol_inline_overflow_row_height_zero`）/ make test 全 workspace 45 binary 0 failed / clippy --workspace --all-targets -D warnings 干净 / cargo fmt 干净 / **product-smoke welcome 16.57%（< 20% gate，== baseline 不变）** / multicol.rs 1277 行（<2000）。

**意义**：纠正 R1035「multi-row 垂直溢出」误模型（标准 multicol 列恒 inline，多行只由 column-span:all 区域分割产生，overflow 列走 inline 方向非下方）。关闭 002（PASS）+ 003 大改善。累计 css-multicol R1027-R1039 + R1074 持续产出。详见 [`evidence/r1074-multicol-inline-overflow-landed-2026-07-06.txt`](./evidence/r1074-multicol-inline-overflow-landed-2026-07-06.txt)。

**▶ 下会话**：① 003 残余 1.03% 近 flip（逐案 LAYOUT_DUMP 查结构 vs font-wall）；② 004a/004b/006/007 簇（仍 FAIL，R1039 标 nested 结构稍紧）是否同类 inline-overflow 可改善；③ inline-overflow 模型扩展到非 spanner balance 路径（minimal multicol 同病，潜在多 case）；④ font-wall C-dep（用户决策，002 残余 0.29% 文本字形亦待其解锁）。

### R1073 FreeType C-dep 跨平台 CI 冒烟门禁（freetype-raster-cross-platform job，非阻塞）= 一键 6-target 验证 bundled FreeType 编译 → C-dep 决策从「需 macos/windows 本地验证」（无法自主做）降为「dispatch CI 看结果」·reversible/非 outward-facing

承 R1072 CONTINUE（C-dep 在用户决策点，实际阻塞 = 6-target CI 编译可行性无法从 Linux 验证）。本轮加 **非阻塞 CI 冒烟 job**，把「无法自主验证的跨平台编译」降为「用户 dispatch CI 即得结果」。

**实现**（`.github/workflows/ci.yml` 新 job `freetype-raster-cross-platform`）：① 仅 `workflow_dispatch` 触发（同主 CI，非每 push）；② 6-target matrix 全覆盖（ubuntu x86/arm + macos intel/arm + windows x86/arm，含 ARM 变体）；③ `continue-on-error: true`（非阻塞，informational，失败不影响主 CI 绿）；④ `cargo check -p zero-render-foundation --features freetype-raster --target <t>`（仅 check render-foundation，不依赖 rusty_v8，验证 freetype-sys bundled 在各 target 的 C 编译——cc crate 从源码编译 FreeType2+libpng）。YAML 合法 + 本地 command 验证通过。

**裁决**：C-dep 决策阻塞**从「需 macos/windows 本地验证」（自主无法做）降为「dispatch CI freetype-raster-cross-platform 看结果」**。job 非阻塞 + 仅 dispatch 触发 → **reversible / 非 outward-facing 风险**（不会让 push CI 红）。用户翻 default 前 dispatch 一次即知 6-target 是否全绿；若某 target 失败（如 windows-11-arm MSVC quirks），精确定位后再决策。→ C-dep 决策的最后技术不确定（跨平台编译）被消除，决策降为纯政策（是否接受 FreeType C 依赖，ZW 已有 rusty_v8 先例）。

**▶ 下会话**：① **用户 dispatch `freetype-raster-cross-platform` workflow** → 6-target 编译结果出 → 据此决策翻 default（或我下轮据结果自主翻）；② 全绿则翻 default = `crates/render-foundation/Cargo.toml` `[features] default = ["freetype-raster"]` + 全量 `make reftest-oracle`（feature 默认 on）确认 +24 泛化无回归；③ font-wall 收官（R1072），rendering-compat 非 font-wall = 结构性 lever（multicol Phase 2 → Phase A 依赖 / R109 taffy-blocked / clean 四证穷尽）。

### R1072 font-wall Phase 2 特性化收官（CSS2/text +5 泛化证毕·6 数据点）+ CI 6-target 矩阵复核（不自主翻 default）+ pivot 结构性 lever·零 net 源码·纯调查

承 R1071 CONTINUE（font-wall 在 C-dep 决策点；自主续泛化量化 + 结构残余）。本轮 CSS2/text 泛化 A/B + CI 矩阵复核，**font-wall Phase 2 特性化收官，pivot 结构性 lever**。

**CSS2/text 泛化 A/B**（408 案，css21 text dir，区别 css-text）：feature-off credible 205 / oracle-pass 212 / strict 18 → feature-on credible **210（+5）** / oracle-pass 217 / strict 18。★ yield 小因 dir 由结构性 white-space/bidi 主导（worst：white-space-collapsing-bidi-002 40.7% / white-space-mixed-001 37% / word-spacing-characters-001 33% / white-space-normal-001/002 28.5%——全 white-space 处理结构性，FreeType on barely move 28.56→28.50 证非字体）。FreeType 修该 dir 小文本光栅化分量 +5。

**★ font-wall Phase 2 特性化收官（6 数据点 yield map 完备）**：
| dir/page | feature-off | feature-on | Δ | 主导分量 |
|---|---|---|---|---|
| css-text | credible 344 | 368 | **+24** | 光栅化（文本 dir） |
| css-text-decor（css-text 内） | 108/242 | 117/242 | +9 | 光栅化 |
| CSS2/text | credible 205 | 210 | +5 | white-space 结构 + 小光栅化 |
| CSS2/backgrounds | credible 228 | 230 | +2 | background-root 结构 |
| welcome | 16.57% | 16.29% | −0.28pp | 文本 + 部分 layout |
| morning | 58.15% | 58.06% | −0.09pp | 58% 结构主导 |

**CI 6-target 矩阵复核**：`.github/workflows/ci.yml` matrix = ubuntu-latest / ubuntu-24.04-arm / macos-15-intel / macos-latest / windows-latest / windows-11-arm（6 target，含 ARM Linux/Windows）。翻 default 须 freetype-sys bundled 在 6 target 全编译——**无法从 Linux 验证 macos/windows 特别是 windows-11-arm**，自主翻 default 风险（CI red 影响全项目）超可接受阈值。**裁决：不自主翻 default，C-dep 留用户决策**（6-target CI 风险是合理 user-gate，非 rally 应绕过的多会话执行阻塞）。

**裁决 + pivot**：font-wall Phase 2 **特性化收官**（yield 已证 + DEFAULT 最优 + 零回归 + CI de-risk bundled + 6 数据点 yield map）。C-dep 在用户决策点（evidence 完备 + 诚实）。rendering-compat 非 font-wall 路径 = **结构性 lever**（clean single-session 四证穷尽 + 产品页结构深查 plateau）：multicol Phase 2（有 spec，R1027/R1028 续）/ R109 case-a（taffy-blocked）/ Phase A IFC（多 session）/ white-space 结构（CSS2/text worst 簇）。

**▶ 下会话**：① C-dep 用户决策（翻 default，6-target CI；可先在 CI 加 freetype-sys bundled 冒烟 build 验证 6 target 再翻）；② **pivot 结构性 lever**——首推 multicol Phase 2 第一切片（读 multicol-phase2-unified-column-flow-spec.md 定可独立首切，R1027/R1028 已续 column-span/break-after）或 white-space 结构（CSS2/text white-space-normal-001/002 28.5% 定位是否可独立修）；③ font-wall 已收官勿再投 A/B（6 数据点 pattern 确凿）。

### R1071 FreeType C 依赖 cross-platform CI de-risk = freetype-raster feature 经 freetype-rs/bundled 从 C 源码编译 FreeType2+libpng（freetype-sys bundled，cc crate）→ 无须系统 FreeType，CI 三平台一致可用 + bundled 输出 == 系统 FreeType（welcome 16.29% 像素 byte-identical 78176）→ C 依赖决策从「需评估三平台系统 FreeType 可用性」降为「cc crate 编译 FreeType2 C 源（高置信跨平台）」·决策阻塞基本消除

承 R1070 CONTINUE（font-wall Phase 2 在 C 依赖决策点，C-dep 实际阻塞 = 三平台 CI 构建可行性未 de-risk）。本轮 **de-risk C 依赖的跨平台 CI 构建路径**，C-dep 决策从「推测」降为「近零风险」。

**问题**：R1068 freetype-raster feature 默认链系统 libfreetype.so（Linux 装包，macOS/Windows CI runner 无）→ 翻 default 会 break macOS/Windows CI。这是用户 C-dep 决策的实际阻塞。

**de-risk**：`crates/render-foundation/Cargo.toml` freetype-raster feature 加 `"freetype-rs/bundled"`（freetype-rs 0.38 `bundled = ["freetype-sys/bundled"]`，freetype-sys 0.23 `bundled` feature 经 `cc` crate 从 C 源码编译 FreeType2 + libpng，build.rs:15/93/125 `!cfg!(feature="bundled")` 分流系统 vs 编译）。→ **无须系统 FreeType，C 源自包含编译**。

**验证**：① `cargo build --release -p zero-render-foundation --features freetype-raster`（bundled）成功（freetype-sys/freetype-rs 从 C 编译，14.35s 略慢于系统链接，自包含）；② **bundled 输出 == 系统 FreeType**：welcome product-smoke **16.29%（78176 px）byte-identical 于 R1068 系统 FreeType 结果** → bundled 是系统 FreeType 的完美 drop-in，CI 用 bundled 三平台一致结果；③ 默认 feature-off 路径不变（clippy 0.14s cached = byte-identical）+ feature clippy 干净。

**裁决**：C-dep 决策**实际阻塞基本消除**。原担忧「macOS/Windows CI runner 须装/找系统 FreeType 2」→ 现解为「cc crate 编译 FreeType2 可移植 C 源」（cc crate 设计即为此，FreeType2 C 源跨平台可移植，freetype-sys bundled 是社区标准跨平台方案，高置信）。**残余风险 = macOS/Windows CI runner 的 C 工具链（clang/MSVC）** —— 这是所有含 C 依赖 Rust crate 的通用前提（ZW 已有 rusty_v8 等 C 依赖），非 FreeType 特有。→ **用户 C-dep 决策可基于：yield 已证（+24 css-text / DEFAULT 最优 / 零回归）+ CI 风险近零（bundled 自包含 + cc 跨平台）**。

**▶ 下会话**：① **待用户 C 依赖决策**（evidence 完备 + CI de-risk，可翻 default）；② 翻 default 步骤 = `crates/render-foundation/Cargo.toml` `[features]` 加 `default = ["freetype-raster"]` + CI workflow 确认 C 工具链（ubuntu/macos 自带 clang，windows MSVC via rust-toolchain）+ 全量 `make reftest-oracle`（feature 现在默认 on）确认 +24 跨 dir 泛化无回归；③ 翻 default 前/后可扩非-Ahem 文本 dir A/B（CSS2/text 等）量化泛化收益；④ font-wall 结构分量（morning 58% / welcome 16% layout）须各自修，FreeType 非其 lever。

### R1070 morning-work CJK oracle 工具 + FreeType CJK yield A/B = morning 58.15→58.06%（−0.09pp 边际，结构主导 58% diff）+ product-oracle-shot.mjs 可复用工具·yield 天花板证毕（FreeType 收益 ∝ 光栅化分量占比）·零 net 功能源码·纯调查+工具

承 R1069 CONTINUE（morning-work CJK oracle 量化，CJK 光栅化差 11-16% 最大）。本轮新建产品页 oracle 工具 + 生成 morning chromium oracle + A/B，**yield 天花板证毕**。

**工具**（`tests/wpt-runner/scripts/product-oracle-shot.mjs`，复用 chromium-oracle-shot.mjs 的 HTTP-server 模式，面向任意产品 fixture）：`--root <dir> --html <rel> --out <png> [--width 800 --height 600 --selector <css> --wait 300]`。内嵌静态 server（R388 http:// 取代 file://，@font-face 本地字体 + 相对资源正确解析）+ headless chromium（`--font-render-hinting=none` 与 product-smoke ZW 侧一致）+ img.decode 等待（R745 race 修复）+ 外部 CDN（ads/disqus/googletag）任其超时失败（仅本地内容入 oracle）。生成 `evidence/product-static/morning-chromium.png`（800×600，gitignored）。

**morning A/B**（CJK 产品页，leizongmin blog，body NotoSansCJK）：feature-off **58.15%** → feature-on **58.06%（−0.09pp 边际）**。★ morning 58% diff **结构主导**（layout/栏目/images/sidebar/@font-face FiraCode webfont/外部脚本失败），CJK 文本光栅化仅小分量——FreeType 改该分量 −0.09pp 但结构性不动（同 backgrounds +2 模式）。

**★ yield 天花板证毕**（FreeType 收益 ∝ 各 dir/page 的**光栅化分量**占总 diff 的占比）：
| dir/page | feature-off | feature-on | Δ | 解释 |
|---|---|---|---|---|
| css-text（文本 dir） | credible 344 | 368 | **+24** | 光栅化 = 主 diff 分量（layout 正确）→ 大 yield |
| welcome（Latin 产品） | 16.57% | 16.29% | −0.28pp | 文本占可观 + 部分 layout |
| backgrounds（布局 dir） | credible 228 | 230 | +2 | 结构性 background-root-* 主导 |
| morning（CJK 产品） | 58.15% | 58.06% | −0.09pp | 58% 结构主导，CJK 光栅化小分量 |

**裁决**：FreeType yield **不是银弹**——文本 dir（layout 正确、光栅化主导）大 yield（css-text +24），结构复杂 dir/page 边际（backgrounds/morning 结构主导）。C 依赖决策 evidence 完备且**诚实**：① 文本 dir 类收益真实（css-text +24 零回归，预期 css-text-decor/linebox/text-transform 等文本 dir 同类）；② 产品页收益小（welcome −0.28pp / morning −0.09pp，受限于结构 diff 主导）；③ DEFAULT hinting 最优（R1069）。翻 default 收益 = 文本 dir oracle 一致率显著提升 + 产品页小幅改善；非银弹，font-wall 的结构分量须各自修。

**▶ 下会话**：① **待用户 C 依赖决策**（evidence 完备 + 诚实，yield ∝ 光栅化分量）；② 翻 default 前/后可扩文本 dir A/B（linebox/text-decor/css21-text 验证 +24 类收益泛化）；③ 产品页残余（welcome 16% / morning 58%）须结构性修（layout/栏目/img/@font-face webfont 加载），FreeType 非其 lever；④ product-oracle-shot.mjs 可复用（任一产品 fixture oracle，DC-13 扩 fixture 基础设施）。

### R1069 FreeType hinting A/B（DEFAULT 最优）+ 多 dir yield 测绘 = DEFAULT 381 > LIGHT 371 > NOHINT 357≈fontdue（证 fontdue=unhinted，FreeType full-hint 向 chromium 收敛）+ yield 集中文本 dir（css-text +24 / backgrounds +2 结构性封顶）·DEFAULT 无需再调·零 net 功能源码（+4 行注释）·纯调查

承 R1068 CONTINUE（Phase 2 LANDED feature-gated，续 feature-on 调优 + 多 dir 量化）。本轮 A/B FreeType hinting 模式 + 量化 yield 跨 dir 分布，**DEFAULT 验证为最优，yield 集中文本 dir**。

**Hinting A/B**（css-text Oracle 1650 with-oracle，env `ZW_FT_HINTING` 运行时切，单 build 多 run）：DEFAULT(full hinting TARGET_NORMAL) **oracle-pass 381 / credible 368 / strict 88** > LIGHT(slight TARGET_LIGHT) 371 / 358 / 89 > NOHINT 357 / 344 / 84 ≈ **fontdue 基线 357/344/84**。★ DEFAULT 最优（R1068 选择正确，勿改 LIGHT/NOHINT）。★ NOHINT==fontude 证 **fontdue tight-ink 即 unhinted 光栅化**，FreeType DEFAULT(full hinting) 向 chromium（hinted）收敛——这正是 R1068 +24 的机制。原假设「chromium 用 slight/LIGHT」**refuted**（oracle chromium 实为 full hinting）。

**多 dir yield 测绘**（feature on vs off）：
- **css-text**（文本 dir）：oracle-pass 357→**381（+24）**，credible 344→368（+24），全 per-dir 零回归（css-text-decor 108→117 / line-breaking 60→67 / white-space 45→48）。welcome −0.28pp（16.57→16.29%）。
- **CSS2/backgrounds**（布局 dir）：oracle-pass 228→**230（+2）**，strict 0→0（全 228 卡 0.1-1.0% font-wall 带）。★ yield 小因 dir 由结构性 `background-root-*`（41%/40%/35%... background-image/color on root，非指令文本）主导；指令文本 font-wall（R1064 测 1.15%）小，FreeType 修该分量 +2 但结构性 background-root 不动。
- **模式结论**：FreeType yield **集中在文本 dir**（css-text +24 / text-decor / line-breaking / white-space），**布局 dir 边际**（backgrounds +2，结构性封顶）。font-wall 各 dir 收益按文本占比分配。

**裁决**：① DEFAULT hinting 验证最优（A/B 数据写入 loader.rs 注释，+4 行，零功能改动）——勿再投 hinting 调优；② C 依赖决策 evidence 完备：css-text +24（文本 dir 主收益）+ welcome −0.28pp + 零回归 + DEFAULT 最优；③ yield 天花板 = 文本 dir（css-text 类）+ 产品页 Latin/CJK（welcome/morning，待 morning oracle），布局 dir font-wall 须各自结构性修。**env knob 已移除**（code-guidelines 不留未要求可配置性 + 避免每字形 env::var 热点）。

**▶ 下会话**：① **待用户 C 依赖决策**（翻 default，evidence 完备：+24 css-text / DEFAULT 最优 / 零回归）；② 翻 default 前最高 EV = morning-work CJK 产品页 oracle（CJK 光栅化差 11-16% 最大，须生成 chromium oracle shot via R388 工具链）量化 CJK yield；③ advance/kerning 精度（FreeType FT_Load_Glyph advance vs fontdue，潜在再降文本 dir diff）；④ 布局 dir（backgrounds/borders）font-wall 须各自结构性修（background-root-*），FreeType 非其 lever。

### R1068 ★Phase 2 FreeType 光栅化路径 LANDED（feature-gated default-off）= css-text Oracle +24 credible pass 零目录回归 + welcome −0.28pp·首 font-wall 正 yield·C 依赖决策升级 evidence-backed

承 R1067 CONTINUE（font-wall 收敛 rasterization-only，Phase 2 = 唯一 lever，待 C 依赖决策）。本轮 **feature-gate FreeType 光栅化路径绕过 C 依赖阻塞**——把 Phase 2 hypothesis 用 A/B 数据验证，C 依赖决策从「推测」升级为「evidence-backed」。

**实现**（`freetype-raster` feature，default-off）：① `crates/render-foundation/Cargo.toml` 加 `freetype-rs = { version="0.38", optional=true }` + `[features] freetype-raster=["dep:freetype-rs"]`；② `font/loader.rs` 加 `freetype_raster` 模块（thread_local FreeType Library + `rasterize(font_bytes, ch, size)→GlyphBitmap`），坐标约定 `x_offset=bitmap_left` / `y_offset=bitmap_top−height`（推导自 `glyph_top_left`），灰度位图按 pitch→紧凑 width×height；③ `rasterize_glyph` 非-Ahem 路径 `#[cfg(feature)]` 优先 FreeType，失败回退 fontdue；④ Ahem 路径不变（A4：Ahem fontdue≈FreeType，保留 rasterize_ahem_glyph 方块特判）。feature-off 整模块不编译 → CI / 默认构建纯 Rust 不变。

**A/B（feature on vs off，同一 tree）**：welcome product-smoke **16.57%→16.29%（−0.28pp）**；css-text Oracle（1650 with-oracle 案）**oracle-pass 357→381（+24）/ credible 344→368（+24）/ strict 84→88（+4）/ 近似 272→293（+21）**，**全 per-dir 零回归**（css-text-decor 108→117 +9 / line-breaking 60→67 +7 / white-space 45→48 +3 / word-break 8→10 +2 / text-transform 7→8 / hyphens 9→10 / word-spacing 2→3 / 余持恒）。★ 与 R1067 Phase 1 metric swap（net-neutral）正反对——**font-wall 光栅化分量真实可解**，FreeType FT_Render_Glyph（chromium Linux 同栈）向 chromium 收敛。

**门禁全绿**：make test 45 bin ok 0 failed（feature-off 默认路径零变化）；clippy `--features freetype-raster --all-targets -D warnings` 干净（let-chain 折叠 + f32 非否定比较 + 去 i32 冗余 cast 三修）；默认 `clippy --workspace --all-targets -D warnings` 干净；新 `freetype_rasterize_ahem_glyph_end_to_end` cfg-gated 测试 PASS（Ahem 'X'@20px 20×20 + y_offset∈[−h,0] + advance≈20，坐标约定守卫）。loader.rs 1305→1442 行（<2000）。

**裁决**：Phase 2 hypothesis **VALIDATED**（+24 css-text 零回归，首 font-wall 正 yield）。feature-gate 使 C 依赖决策**解耦**——default-off 落地经验证的基础设施（不阻塞 CI，纯 Rust 默认路径不变），用户决策 = 「翻 default 启用 CI 三平台 FreeType」。C 依赖决策从 R1064「推测需 accept」升级为「**evidence-backed：+24 css-text 零回归证明 FreeType 光栅化收益真实**」。

**▶ 下会话**：① **待用户 C 依赖决策**（翻 default：CI ubuntu/macos/windows 须 FreeType 2；Linux 系统装 / macOS/Windows vendored via freetype-sys bundled feature）——收益已证；② 翻 default 前可继续 feature-on 调优（FreeType hinting/subpixel 对齐 chromium 具体设置，潜在再降 diff；advance/kerning 精度；paint 侧 v_offset 与 FreeType 度量 coherence）；③ R1067 Phase 1（metric）已 closed，font-wall 唯一 lever = Phase 2（本轮 LANDED feature-gated）。

### R1067 Phase 1（fontdue 度量 → FreeType 度量 swap）A/B 实测 NET-NEUTRAL = 第 7 证（metric-source swap 无 yield）+ R1066「Ahem font-wall=度量」refuted（16px 度量差为 FreeType 舍入伪影，真值 fontdue=FreeType=0.8）+ font-wall 收敛 rasterization-only（Phase 2 唯一 lever）·已回退·零 net 源码·纯调查

承 R1066 CONTINUE（Phase 1 度量 coherence，R848 三方同改用 FreeType 真实度量）。本轮扩展 fontcmp prototype（dump fontdue hlm + freetype size_metrics 跨 16/20/40px）精确测绘度量源 + A/B 实测 non-Ahem ascent 0.928→0.95，**Phase 1 refuted（第 7 证），font-wall 收敛 rasterization-only**。

**度量源精确测绘**（关键纠正 R1066）：多 size 测绘证 FreeType `size_metrics` @16px 为 fixed-point 舍入偏低——DejaVu 真值 0.95（20/40px）/ Ahem 真值 **0.80**（20/40px，与 fontdue 一致；16px 0.8125 是舍入伪影）/ CJK 1.175-1.20。**R1066「Ahem 16px 12.80 vs 13.00」暗含度量差被证伪**——Ahem fontdue 与 FreeType 度量实际一致（0.8）。

**A/B 实验**（`ascent_ratio_lookup` 非-Ahem 0.928→0.95，单行改动，仅 R848 三方之第一方 layout strut）：welcome 16.57→16.48%（−0.09pp 噪声）；css-text oracle-pass 357==357 / credible 344==344 / strict 84→85（+1 噪声）/ 全 per-dir（css-text-decor 108/242、white-space 45/395、line-breaking 60/127、text-align 12/73、letter-spacing 5/32、text-indent 14/25、word-break 8/87）**identical**。**裁决 NET-NEUTRAL，已回退**。

**★ 7th proof + Phase 1 关闭**：metric source（fontdue 0.928 vs FreeType 0.95）不影响 oracle——ZW pipeline compensating offsets（paint v_offset=fs + height−0.8·fs + half-leading）吸收 2.4% ascent nudge，阈值未翻。承 R834/R836/R849/R875/R1052/R1056 六证，本轮第七证：metric **源** swap（非数值调）亦 net-neutral。font-metric 任何单维度改动（数值/源/单点）在 ZW 当前 pipeline 均无正 yield。**Phase 1 关闭，勿再投入 metric swap**。

**★ font-wall 收敛 rasterization-only**：① Ahem-font WPT 残余几何全证毕——line-height:normal Ahem=1.0 LANDED（R759 `AHEM_LINE_HEIGHT_RATIO`），non-Ahem=1.2 = chromium DejaVu（FreeType asc−desc @20px = 24.0 = 1.2em exact），ascent swap net-neutral；② css21 指令文本 / welcome-morning Latin+CJK font-wall = **DejaVu/CJK rasterization 差**（A1：'a'/'g' per-glyph 27-30%），非度量（本证）非光栅化无关（A4 仅限 Ahem）。**残余 font-wall 100% = rasterization（Phase 2，须 C 依赖）**。

**裁决**：scoping doc 升 v0.4（Phase 1 移除/refuted，Phase 2 = 唯一 font-wall lever）。C 依赖决策从「Phase 1 可绕过」升级为「font-wall 唯一解锁器」——决策紧迫度上升。详见 [`evidence/r1067-phase1-metric-swap-net-neutral-2026-07-06.txt`](./evidence/r1067-phase1-metric-swap-net-neutral-2026-07-06.txt)。

**▶ 下会话**：① **待用户 C 依赖决策**启动 Phase 2（rasterize 替换 freetype-rs，唯一 font-wall yield lever）；② 决策前 rendering-compat 转**非 font-wall 结构性 lever**（multicol Phase 2 column-fragmentation 有 spec / R109 case-a taffy-blocked / Phase A IFC 多会话）；③ 勿再投 font-wall metric/line-height 几何（Ahem/non-Ahem/ascent/line-height-ratio 全证毕，rasterization-only）。

### R1066 fontdue vs freetype-rs 光栅化 prototype = A1+A4 验证（freetype-rs per-glyph ≠ fontdue：Latin 3-30% / CJK 11-16% mean|Δ|，向 chromium 收敛；Ahem ≈ identical 20px 0.0000 无回归）+ Ahem WPT font-wall = 度量非光栅化（细分）+ Phase 1/2 可分步独立·零 net 源码·纯调查

承 R1065 CONTINUE（Phase 0 empirical prototype）。本轮 standalone cargo 项目（/tmp/fontcmp，fontdue 0.9 + freetype-rs 0.38，链系统 libfreetype.so.6）实测 fontdue vs FreeType per-glyph + 度量 diff，**验证假设 A1+A4，refine Phase 1/2 计划**。

**A1 验证（freetype-rs ≠ fontdue）**：DejaVuSans 'a'@20px mean|Δ|/255=**0.2992 (30%)** / 'g'@20px 0.2701 (27%) / 'T'@20px 0.0292 (3%)；NotoSansCJK '試'@20px 0.1642 (16%)。fontdue tight-ink vs FreeType FT_Render_Glyph 渲染模型差大（曲线字形 'a'/'g' 差最大）。→ **替换 fontdue→freetype-rs 必显著改 ZW 渲染向 chromium（FreeType）收敛**，font-wall 光栅化分量可解。

**A4 验证（Ahem 不回归）**：Ahem 'X'@20px mean|Δ|=**0.0000 精确一致** / @16px 0.0235 covΔ=0.0000。方块字形 fontdue≈FreeType。→ **Ahem WPT 测试不受光栅化器替换影响**。

**★ 重要细分：Ahem WPT font-wall = 度量非光栅化**。Ahem 光栅化 fontdue≈FreeType identical，但 WPT Ahem 测试仍 1-3% diff（R1064）。故该 diff 来自**度量差**（fontdue asc 14.85 vs FreeType 15.00 @16px DejaVu；Ahem 12.80 vs 13.00；CJK 23.20 vs 24.00）+ IFC 行盒几何，**非光栅化**。→ **WPT（Ahem 主导）修须 Phase 1 度量管线 coherence**（R848 三方同改用 FreeType 真实度量），光栅化替换对 WPT 收益小；**产品页（welcome/morning Latin/CJK）修须 Phase 2 光栅化替换**（光栅化差 11-30% 主导产品 diff）。

**★ Phase 1/2 可分步独立**：Phase 1（度量 coherence，用 FreeType line_metrics 替 fontdue，**fontdue 仍光栅化**，最小 slice 低风险）→ Phase 2（rasterize_glyph 替 freetype-rs FT_Render_Glyph）。降低单步复杂度 + A/B 风险。详见 [`evidence/r1066-fontdue-vs-freetype-prototype-2026-07-06.txt`](./evidence/r1066-fontdue-vs-freetype-prototype-2026-07-06.txt) + scoping doc（待升 v0.3）。

**裁决**：prototype 证 freetype-rs 可用（系统 FreeType 链接成功 + API 工作 + per-glyph diff 实证）。Phase 0 完成。**待用户 C 依赖决策**（R1064 飞书通知 3 开放问题）启动 Phase 1（度量 coherence 最小 slice，不须替换光栅化器，fontdue 仍光栅化，A/B 守 Ahem WPT + welcome/morning/linebox oracle）。

**▶ 下会话**：① **待用户决策**启动 Phase 1（度量 coherence，R848 三方同改：layout strut_ascent + paint v_offset −real_ascent + half-leading (lh−(asc−desc))/2，用 FreeType line_metrics 替 fontdue，fontdue 仍光栅化）；② 或自主起 Phase 1（决策独立——度量用 FreeType 真值是 correctness，C 依赖仅在 Phase 2 光栅化替换时引入，Phase 1 仅读 FreeType 度量须 freetype-rs 依赖但可不进 CI）；③ rendering-compat clean lever 四证穷尽，fontdue 替换（Phase 1 度量 + Phase 2 光栅化）是唯一 unblocker。

### R1065 fontdue→chromium 替换 Phase 0 web research = freetype-rs 定为唯一 chromium 像素匹配候选（chromium Linux 链 Chrome→Skia→FreeType 实证）+ swash 误归类纠正（shaping 非光栅化）+ 纯 Rust 候选不像素匹配 chromium·scoping doc v0.2·零 net 源码·纯调查

承 R1064 CONTINUE（fontdue 替换 Phase 0 research，rally 自主推进不阻塞用户决策）。本轮 web research（WebSearch）对比 fontdue 替换候选，**freetype-rs 定为唯一理论 chromium 像素匹配候选，scoping doc 升 v0.2**。

**chromium Linux 字体管线实证**（Skia 官方 + SO + chromium blog）：**Chrome → Skia → FreeType**。Skia 在 Linux 把字体解析/光栅化（FT_Render_Glyph + hinting + AA）委托 FreeType，Skia 维护 glyph cache + 合成。新兴 Fontations（Rust）仅做**解析**，光栅化仍 FreeType。→ **chromium 最终光栅化 = FreeType（Linux）**。

**★ 候选裁决（§3.2）**：① **`freetype-rs`**（C 绑定 FreeType 2 + FT_Render_Glyph）= **唯一理论像素级匹配 chromium 候选**（同栈 FreeType，验证假设 A1）；② ~~`swash`~~ **误归类纠正**——swash 是 **shaping 库**（HarfBuzz 风格复杂脚本整形）非光栅化器，不能替 fontdue rasterize（可作 ZW shaping 缺口独立 lever，当前非阻塞）；③ **`ab_glyph`**（纯 Rust）无 hinting/LCD subpixel，光栅化模型 ≠ chromium，**不像素匹配**（同 fontdue tight-ink 谱系，换不解决 font-wall）；④ Pathfinder/font-rs（GPU）模型不同 + 须 GPU 集成。

**★ 核心权衡（须用户决策，已在 R1064 飞书通知）**：fontdue 替换 = **accept FreeType C 依赖**（chromium Linux 像素级匹配，unblock font-wall）vs **保持纯 Rust**（ab_glyph/fontdue 同谱系，font-wall 不可解）。**无中间方案**——纯 Rust 无法匹配 chromium FreeType 光栅化（tight-ink vs FT_Render_Glyph 模型差）。

**Phase 0 web 部分完成**（§4 表）：剩 empirical prototype（接 freetype-rs 到 ZW fontdue 调用点，对 Ahem + DejaVuSans + NotoSansCJK 同字形渲染 chromium vs freetype-rs，像素 diff 验证 A1 像素级匹配 + hinting/subpixel/gamma 配置对齐），**待用户 C 依赖决策后**。详见 [`fontdue-replacement-scoping.md`](./fontdue-replacement-scoping.md) v0.2。

**▶ 下会话**：① **待用户 C 依赖决策**（R1064 飞书通知 3 开放问题）—— accept freetype-rs C 依赖启动 empirical prototype / Phase 1 RFC，或调整优先级；② 决策前可自主做 empirical prototype（验证 A1，C 依赖 prototype 在 dev 环境不影响 CI 决策）；③ rendering-compat clean single-session lever 四证穷尽，font-wall 结构性平台期，fontdue 替换是唯一 unblocker。

### R1064 css-backgrounds/borders fresh dir 扫描 = 簇全 font-wall（1.15% background-001-022 + 1.46% 026-053 + 3.10% border-top-width-012-078 identical diff = "Filler Text" 指令文本 fontdue≠chromium 渲染）+ negative border-width 处理已正确（apply.rs:225 reject）·clean lever 穷尽第四证（R740/R1053/R1057/R1063/R1064）·零 net 源码·纯调查

承 R1063 CONTINUE（pivot css-backgrounds/borders fresh dir）。本轮扫 CSS2/backgrounds (228/339=67.3%) + CSS2/borders (399/506=78.9%) + css-fonts (98/287=34.8%) borderline 簇，**全部簇 = font-wall（指令文本 fontdue≠chromium），clean lever 穷尽第四证**。

**簇全 font-wall（identical diff = 同指令文本）**：① **CSS2/backgrounds**：background-001/002/006/007/008/009/010/014/018/022 + background-color-175 全 **1.15%** exact（~10 案同 `<p>Test passes if...</p>` + 全宽 green div，layout diff=0，1.15% 纯 = 指令文本 fontdue 渲染）；background-026/029/038/041/050/053 全 **1.46%** exact（~6 案同结构）；② **CSS2/borders**：border-top-width-{012,023,034,045,056,067,078} 全 **3.10%** exact（7 案 `border-top-width: -X` 变体，同 "Filler Text" 文本）；border-bottom-width 同簇 **3.26%**；border-*-width-applies-to-001-004 **3.21%**。

**★ negative border-width 处理已正确**（非 bug）：border-top-width-012 测 `#span1 { border-top-width: -1pt }`（应 §8.5.1 reject → initial medium 3px）vs span2 medium。ZW `apply.rs:225` 已 `if let Px(px) = v && px < 0 { return false }`（reject → 保 medium initial）。LAYOUT_DUMP span1 h=22 ≈ span2 h=23（border 都 3px）。**3.10% = "Filler Text" font-wall 非 border bug**。

**★ font-wall baseline 量化**：WPT css21 测试标准化 `<p>Test passes if...</p>` 指令文本，fontdue vs chromium 字形渲染差贡献 ~1-3% diff baseline，**笼罩全 CSS2 测试**。case 须 layout diff > ~0.5% 才有 fix 后越过 font-wall <1% 阈值的可能。backgrounds/borders 簇 layout diff≈0（纯 background/border 渲染正确），故全卡 font-wall。R1058（inline vmargin）yield 因 layout diff=1.12% 足够大（fix 后 0.70% < 1% 越过 font-wall 0.7% baseline）。

**裁决**：css-backgrounds/borders clean single-session lever 穷尽（簇全 font-wall），**clean lever 穷尽第四证**（R740 doc-scan + R1053 5-dir scan + R1057 list-item + R1063 box-display + R1064 backgrounds/borders）。rendering-compat 目标**结构性平台期确认**：clean lever 耗尽，残余失败 100% fontdue≠chromium 字体墙（backgrounds/borders/box-display/text/text-decor/writing-modes 簇）+ R109 结构性（box-display case-a/case-b）+ vertical（R1043/R1052）+ multicol（Phase 2）。R1058（inline 垂直 margin）是 R1057-R1064 七轮唯一 code yield。

**▶ 下会话战略**：clean single-session lever 已四证穷尽，**进一步推进须转 multi-session 架构 lever**：① **fontdue → chromium-matching rasterizer 替换**（最高 EV，unblock backgrounds/borders/box-display/text/text-decor/font-wall 簇；fontdue API surface = render-foundation/src/font/ FontLoader，`from_bytes`/`rasterize`/`horizontal_line_metrics`；候选 FreeType (freetype-rs, chromium Linux 同栈) / swash / ab_glyph；须 lei-spec-rfc 起 RFC 定迁移路径 + 切片）；② R109 case-a 3px offset（taffy measured-leaf，须 anon-box 重构 new_with_children）；③ Phase-A IFC 单次源统一（非 font-metric 子任务，六证 ruled out font-metric）。★ 勿再盲扫 fresh dir 找 clean single-session lever（四证耗尽，font-wall baseline 笼罩）。

### R1063 paint 双渲染 probe + case-b 容器抑制修复 = 确证双渲染但抑制致 normal-flow -29 回归 + box-display case-b 簇终裁 font-blocked（1.04% 残余 = 字体/精度 非 anon-box）·已回退·box-display 簇穷尽·零 net 源码·纯调查

承 R1062 CONTINUE（probe paint_text anon fragment）。本轮复加 width fix + R1063DBG probe paint_text，**确证 case-b 双渲染**（#div1 容器 + anon fragment 共 node_id 都渲染文本），实施容器抑制修复，**A/B 揭示 box-display font-blocked + 抑制致 normal-flow -29 回归，已回退，box-display 簇穷尽**。

**R1063DBG probe 实证双渲染**（anonymous-box-generation-001）：paint_text 对 node_id=30v1(#div1) 调 2 次：① 容器（fragment=false, has_anon_child=true, content_w=192）；② anon fragment（fragment=true, content_w=192）。**两者都渲染 "Filler Text"**（inline_layout=false 走 Path B）。双渲染 = case-b 容器自身 IFC + anon fragment IFC 共渲染同文本。

**★ 容器抑制修复**（text.rs:785 后加）：`fragment_node_ids.is_none() && children.iter().any(|c| c.fragment_node_ids.is_some())` → return（case-b 容器文本在 anon fragment，不自身渲染）。逻辑 spec-correct（§9.2.1.1 block-mixed inline 文本全在 anon 盒）。

**A/B（width fix + 抑制）**：① **CSS2/box-display 39→39（net-0）**——anonymous-box-generation-001 仍 1.04%（width fix 已让双渲染都 @x=50 居中重叠，抑制其一无可视变化；1.04% 残余 = `<p>` 文本 + "Filler Text" fontdue vs chromium + blue stripe AA = **font/precision 非 anon-box**）；② **CSS2/normal-flow 604→575（-29 回归 ❌）**——抑制过宽，normal-flow 多案合法容器（含 anon fragment 子但须自身渲染文本）被误抑。

**裁决：box-display case-b 簇终裁 font-blocked，box-display 簇穷尽**。① 双渲染抑制 spec-correct 但 **normal-flow -29 回归**（过宽，须窄化到真 case-b 容器，但 normal-flow 合法 case 难区分）→ 回退；② width fix + 抑制对 box-display **net-0**（target 1.11→1.04 后续 font 残余不可越 <1%）→ 无 yield；③ **1.04% = font/precision**（双渲染对齐 x=50 后残余纯字体），box-display case-b 非 anon-box 可产 lever。git checkout 回退 width fix + 抑制（零 net 源码）。

**box-display R109 簇 5 轮（R1059-R1063）收官**：R1058（inline 垂直 margin）= 唯一 clean yield（+1）；R1059 定位 2 bug；R1060 Bug 2 measure refuted（taffy）；R1061 width fix net-0；R1062 paint-side；R1063 双渲染抑制回归 + font-blocked 终裁。**剩余 box-display 1-1.6% 簇 = font-wall（fontdue 字体度量/AA）+ R109 case-a 3px offset（taffy measured-leaf）+ case-b 1.04%（font）**，皆非单 session clean lever。

**▶ 下会话**：box-display 簇穷尽，**pivot 必要**：① css-backgrounds / css-borders fresh dir 扫 borderline 簇（R740 strategy ②，未扫过）；② css-tables top-worst 复审（R177 territory，可能漂移出新 lever）；③ 或转 Phase-A IFC 单次源统一续（非 font-metric 子任务）；④ R109 case-a 3px offset（span w=6 taffy measured-leaf，与 case-b 同 taffy quirk，taffy-blocked）。★ box-display case-b 勿再投（font-blocked + 抑制回归，R1063 终裁）。

### R1062 R1061 text-align bug 定位 = compute_final 确为 anon 盒建 Center IFC（probe 实测 text_align=Center container_w=192 is_block_level=true，存储无条件）·bug 在 paint 侧（use_stored 不为 anon 触发 或 容器 #div1 自身 IFC 渲染文本 而非 anon 盒）·零 net 源码·纯调查

承 R1061 CONTINUE（probe compute_final 内 anon 盒 text-align 应用）。本轮复加 width fix + R1061DBG probe compute_final，**确证 compute_final 端正确，bug 在 paint 侧**。

**R1061DBG probe 实测**（anonymous-box-generation-001，width fix 复加使 anon 盒 w=192）：compute_final 内 anon 盒 `node_id=Some(NodeId(30v1)) container_w=192.0 text_align=Center style.text_align=Center is_block_level=true`。**compute_final 确为 anon 盒以 Center text-align + 192 宽建 IFC**（resolve_text_align(styles[#div1]) = Center 正确）。存储无条件（inline_finalization.rs:831 `if !lines.is_empty()`，"Filler Text" 非空 → 必存）。

**★ bug 缩窄到 paint 侧**：compute_final 建 + 存 Center IFC，但渲染未居中（diff 仅降 0.07pp 非居中应降 ~0.38pp）。paint use_stored 条件（text.rs:937-938）：`multicol_info.is_none() && inline_layout.is_some() && (inline_layout_width - ifc_width).abs() < 1.0`。anon 盒满足（inline_layout Some + 192=192），use_stored 应 true。**疑点**：① paint 是否对 anon 盒（fragment_node_ids.is_some）跑 paint_text？或跳过？② 容器 #div1 自身也有 IFC（node_id 同 30v1，has_text_children via DOM），paint #div1 时若也渲染其 IFC 文本（left?），可能与 anon 盒 Center IFC 冲突/覆盖（双渲染或 #div1 赢）。text.rs:785 `is_r109_split && fragment_node_ids.is_none` 仅处理 split parent（非 fragment），anon fragment 的 paint 路径未单独审计。

**裁决**：width fix + probe 已 git checkout 回退（零 net 源码）。**bug 精确定位 paint 侧**：compute_final 端 Center IFC 正确存储，paint 端未用（或被容器 IFC 覆盖）。下会话 probe paint_text 对 anon fragment 的调用（box_node.fragment_node_ids.is_some 时 use_stored 是否真触发 + 是否双渲染）。

**▶ 下会话**：① probe paint_text 对 anon fragment：在 text.rs:937-938 print use_stored/inline_layout_width/ifc_width for fragment_node_ids.is_some 盒；若 use_stored=false 找原因，若 true 但仍左对齐则查双渲染（#div1 vs anon）；② 找到 paint bug 后，复加 R1061 width fix + paint fix 同落 → yield anonymous-box-generation-001（+1 box-display，可能 unlock case-b 簇）；③ 若 paint 侧复杂，pivot css-backgrounds/borders fresh dir。★ compute_final 端勿再查（R1062 已证正确）。

### R1061 anon 盒宽度 postprocess 修复（pre-compute_final）= 盒级生效（100→192）但 oracle net-0（4 dir 1348 案零翻转）+ 目标 1.11→1.04 未 flip（text-align:center 另 bug 阻断）·已回退·待 text-align 修后同落·零 net 源码·纯调查

承 R1060 CONTINUE（Bug 2 = block-mixed anon 盒满宽，R1060 推翻 measure 级修复，留下 postprocess width fix 或 anon-box 重构两条路）。本轮实施 **postprocess width fix（pre-compute_final）**：盒级确生效（anon 盒 100→192），但 **A/B 4 dir 1348 案 net-0 零翻转**，目标案仅 1.11→1.04 未 flip，**根因 = text-align:center 另一独立 bug 阻断**。已回退（net-0 不 land）。

**改动（已 git checkout 回退，零 net 源码）**：① `postprocess.rs::fix_r109_anon_block_widths`（新函数，先序遍历，anon 盒（fragment_node_ids.is_some）width/content_width = 父 content_width，CSS §9.3.1 block 满父宽）；② `engine.rs:424` 在 `compute_final_inline_layouts` **之前**调用（关键：compute_final 以 root.content_width 重建 IFC，inline_finalization.rs:619，故 pre-compute_final 修正宽度使 IFC 在正确宽下重建）。

**A/B（stash baseline，4 dir）**：CSS2/box-display 39→39（net-0）/ css-flexbox 295→295（net-0）/ css-grid 20→20（net-0）/ CSS2/margin-padding-clear 310→310（net-0）。**1348 案零翻转（PASS->FAIL=0，FAIL->PASS=0）**。welcome product-smoke 16.57% 不变。

**★ 盒级生效但未 yield**：anonymous-box-generation-001 LAYOUT_DUMP `div（anon）w=100 → w=192`（block-mixed anon 盒确满 container 192 宽，CSS §9.3.1 spec-correct）。目标案 1.11%→1.04%（仅 -0.07pp，未 flip <1.0%）。**text-align:center 未生效**——"Filler Text" 应居中（192 宽内 (192-92)/2=50 居中位），实际仍左对齐（diff 仅降 0.07pp，若居中应降 ~0.2pp+）。

**根因定位**：compute_final_inline_layouts **确为 anon 盒跑**（`is_block_level || is_anon_fragment`=true，engine.rs:1193 实证；node_id=#div1 有 text-align:center style；has_text_children=true），IFC 应在 192 宽以 center 重建。但实测文本未居中 → **text-align 在 IFC 重建时未正确应用**（独立 bug，非 width）。可能：① anon 盒 node_id 映射（taffy_to_dom）非 #div1 而是别的（text 节点？）致 style 查询拿错 text-align；② resolve_text_align 在 anon 路径读错 style；③ IFC 重建用了 measure-time 缓存非新 build。**须下会话 probe 定位**（compute_final 内 print text_align + node_id for anon 盒）。

**裁决**：postprocess width fix net-0（零翻转）+ text-align blocker → **git checkout 回退**。width fix 本身 spec-correct（§9.3.1 block 满父宽）且盒级确生效，但 **text-align 不修则 anon 盒满宽无可视 yield**（文本仍左对齐于满宽盒，diff 不变）。两 fix 须同落（width + text-align）才 yield 目标案。★R1060「须 anon-box 重构或 taffy 升级」**部分纠正**：postprocess width fix 可绕 taffy（pre-compute_final），不必重构；真阻断 = text-align 应用 bug。

**▶ 下会话**：① R1061 width fix 复加 + probe compute_final 内 anon 盒 text-align 应用（node_id 映射？resolve_text_align 路径？）→ 修 text-align → 两 fix 同落 yield 目标案（anonymous-box-generation-001 +1，可能 unlock case-b 簇）；② 或 pivot 非 R109 角度（box-display containing-block 簇已证 ZW 正确 1.1-1.6% precision 噪声，非 lever；css-backgrounds/borders fresh dir 待扫）。

### R1060 Bug 2（case-b block-mixed anon 盒满宽）measure 级修复 REFUTED = taffy 0.7 忽略 measured block leaf 的 measure 返回宽（probe 实测 measure 返 192 但 box 仍 w=100）·box width 源未定位·需 anon-box 重构或 taffy 升级·零 net 源码·纯调查

承 R1059 CONTINUE（Bug 2 攻坚——block-mixed anon 盒 w=文本宽非满 container 宽，entry tree.rs:827 + measure_text_content:868）。本轮实施 measure 级修复 + probe 实证，**REFUTED：taffy 0.7 不用 measure 返回宽给 measured block leaf 赋宽，box w=100 源未定位，需 anon-box 重构**。

**measure 级修复尝试**：`measure_text_content` text-node 分支（inline_finalization.rs:868）`width: known_dimensions.width.unwrap_or(measured_width)` → 加 `available_space.width` Definite 分支（block 上下文用 container 宽，MinContent/MaxContent 保留 measured_width）。逻辑正确（block 应满父宽）。

**R1060DBG probe 实证（anonymous-box-generation-001）**：measure 调用 1 次 `text="Filler Text" known_w=None avail_w=Definite(192.0) measured_w=92.0 -> ret_w=192.0`。**measure 返回 192（修复生效）**。但 LAYOUT_DUMP anon 盒 `w=100` 不变（ neither 192 Definite nor 92 measured）。

**★ taffy 0.7 行为**：`new_leaf_with_context(anon_style, ctx_node)`（tree.rs:832）创建的 measured leaf，taffy **不用 measure 返回的 width** 给 box 赋宽（measure 仅用于 height / intrinsic hint）。extract_layout `width = layout.size.width`（engine.rs:1213）直读 taffy resolved，= 100。**100 源未定位**（非 measured_w=92、非 Definite=192、非 container=192；疑 taffy 0.7 measured-leaf block sizing 内部值或 cache）。

**裁决**：measure 级修复 REFUTED（已 git checkout 回退）。Bug 2 真 fix 须：① anon 盒**重构为非 measured leaf**（new_with_children + 子 IFC 节点，让 taffy block 流满父宽），多 session 架构；② 或 extract_layout 后处理把 anon 盒（fragment_node_ids.is_some）width 设为父 content_width——但 IFC 已在 wrong width(100) 跑，仅改 box width 不 fix text-align:center（须 rerun IFC）；③ 或 taffy R304 升级（已 ruled out vertical 收益，但 measured-leaf block sizing 或修）。**皆多 session / out-of-single-session**。

**意义**：排除「measure 返文本宽是 Bug 2 根因」假设（measure 返 192 但 taffy 忽略）——R1059 的 Bug 2 entry 须 correction（root cause 非 measure_text_content:868，是 taffy 0.7 measured-leaf block sizing）。锁定为 taffy-side，ZW-side measure 修无效。

**▶ 下会话**：① Bug 1（case-a split-span 内块子 3px x-offset，R1059 定位）——span w=6 residual inline 盒来源 + 3px offset，可能在 ZW-side（非 taffy），比 Bug 2 tractable；② 或 pivot 非 R109 角度（box-display borderline 含 containing-block-007/008/010/019/028/030 多案 1.1-1.3%，非 anon-box 簇）；③ Bug 2 留待 taffy 升级或 anon-box 架构重构。★ Bug 2 勿再以 measure_text_content 修（R1060 refuted）。

### R1059 block-in-inline margin-collapse 簇调查 = margin 已正确折叠（R1058 后）+ 2 R109 anon-box 构造 bug 精确定位（case-a split-span 内块子 3px x-offset / case-b block-mixed anon 盒 w=文本宽非满宽）·多 session structural·零 net 源码·纯调查

承 R1058 CONTINUE（block-in-inline margin-collapse 簇，box-display 10 案 1.0-1.66% flip 候选）。本轮 LAYOUT_DUMP 簇逐案对比 test vs ref，**margin 折叠已正确（R1058 后），残余 diff = 2 个 R109 anon-box 构造 bug，多 session structural，定位精确供后续 session**。

**margin 已正确**：multiple-block-in-inlines-margins-collapse（1.05%）LAYOUT_DUMP 实测 a/b/c（mb=40/mt=30/mt=50）gap = **40px / 50px 精确**（max 折叠对），与 ref flat-block 一致。R1058（inline 垂直 margin 归零）已清 margin 输入，折叠逻辑本身正确。

**★ Bug 1（case-a split-span 内块子 3px x-offset）**：block-in-inline-margins-collapse-with-trailing-block（1.11%）LAYOUT_DUMP：`span abs_y=44 x=8 w=6`（R109 split parent，w=6 bogus 空inline 盒）→ 子 `div.first x=11 w=100`，而 span 外兄弟 `div.second x=8 w=100`。**split-span 内块子 x 偏移 +3px**（= span w=6 / 2，疑似 anon 盒居中于 span 的 residual inline 盒 w=6，或 half-leading）。case-a 簇多案同 pattern（multiple-block-in-inlines / nested-spans-with-block / block-in-inline-followed-by-*）。spec 预期 anon 盒是 block-level 满 container 宽 @ content edge（x=8），非偏移。

**★ Bug 2（case-b block-mixed anon 盒 w=文本宽非满宽）**：anonymous-box-generation-001（1.11%）LAYOUT_DUMP：`#div1 w=192` 子 anon 盒（包裹 "Filler Text"）`w=100`（应满 container 192）→ text-align:center 无法居中（文本左对齐于 100px 盒）。block-mixed anon 盒经 `Style::default() + display:Block + new_leaf_with_context(measure)`（tree.rs:827），taffy 应 block 满宽但 measure callback 似覆盖为文本宽。case-b 簇 anon 盒应 block-level 满 container 宽。

**裁决**：2 bug 都在 R109 anon-box 构造/定位（tree.rs:824-855 区）+ taffy measure 交互，**多 session structural**（R109 spec FR-002/003 territory）。margin collapse 本身（§8.3.1）已正确，残余是 anon-box 几何（width/position），非 margin 逻辑。本 session 不强修（A/B 回归风险高，需 coordinated anon-box width/position 重构）。

**精确 handoff**：① Bug 1 fix 入口 = tree.rs inline-split 分支（:834-855），anon 盒 x-position 须 = 父 container content edge（非 span inline 盒偏移）；span residual inline 盒 w=6 须查源（IFC 空 inline 盒？）；② Bug 2 fix 入口 = tree.rs:827 `is_block_mixed` anon_style，确保 taffy block 满宽（可能须禁 measure 改用 content_size 或显式 width=auto 满 container）；③ A/B 守 margin-padding-clear（R743/R744 回归风险 dir）。

**意义**：R1058 后 margin 输入正确，本 round 排除「margin 折叠逻辑错」假设（已正确），锁定残余 = anon-box 几何（width/position），为后续 R109 FR-002（bg 涂布依赖 anon 盒满宽）+ FR-003（border 归属）提供精确靶点。box-display 簇 10 案 1.0-1.66% flip 须 Bug 1/2 任一修才 unlock。

**▶ 下会话**：① Bug 2（case-b anon 盒满宽）攻坚——entry 已锁定 tree.rs:827，A/B 守 margin-padding-clear；② 或 Bug 1（case-a 3px offset）——须先查 span w=6 来源；③ 或转 R109 FR-002（bg 涂布）/ FR-003（border 归属）多 session slice。

### R1058 ★CSS §8.3 display:inline 垂直 margin 归零 LANDED = box-display +1（block-in-inline-vertical-margins-on-span-ignored 1.82→0.70 PASS）零回归（margin-padding-clear/normal-flow/linebox net-0 + welcome 不变）·converter 上游修复·R109 Phase-0 附带 yield·有 net 源码

承 R1057 CONTINUE（R109 §9.2.1.1 anonymous block Phase-0 攻坚）。本轮 R109 Phase-0 map（FR-001 已 landed postprocess.rs:610，剩 FR-002/003 + margin-collapse 多 session），map 中定位 box-display borderline 簇 `block-in-inline-*-margin*` 真根因（converter §8.3 缺失），**A/B 确证 clean +1 零回归 LANDED，R1051-R1057 七轮 docs-only 后首个 code LANDED**。

**定位（LAYOUT_DUMP + R1058DBG 探针）**：`block-in-inline-vertical-margins-on-span-ignored`（1.82%）= `<span mt/bt:50><div></div></span><div>` 两绿块应相邻（span 垂直 margin §8.3 ignored）。LAYOUT_DUMP `span.span mt=50 dmt=50` + `div.sibling abs_y=155`（应 ~105，50px gap）。R1058DBG 探针确认 `is_inline_r109=true`（span R109-split）。

**假设 1（tree.rs anon_style margin 泄漏）REFUTED**：tree.rs:839 inline-split 的 anon_style 经 `computed_style_to_taffy(&computed)` 继承 split inline 全 computed（含 margin），仅清零 inset。改 `anon_style.margin = Rect::zero()`——**A/B ZERO-EFFECT**（span.span mt=50 不变，box-display 38→38），证明 anon 盒 margin 非 span mt=50 来源。

**真根因 = converter 上游**：`computed_style_to_taffy`（converter/mod.rs:117）把 `style.margin_top/bottom` 原样转 taffy Style margin，不区分 display。span computed display=Inline（CSS 初始值），margin-top:50 喂给 taffy → span taffy 节点 + 继承 computed 的 anon 盒都 margin-top=50 → layout.margin.top=50。CSS §8.3：**非替换 inline 元素垂直 margin 无布局效果**（chromium 同行为）。

**改动（converter/mod.rs:117 margin 分支）**：`display:Inline`（非替换 inline；替换 inline 如 img UA 默认 InlineBlock 不在列）→ `margin.top = margin.bottom = Length(0.0)`；`margin.left/right` 保留（inline 水平 margin 有效，IFC 内 inline 片段承担）；其他 display 不变。9 单测（2 新 R1058 + 7 既有 margin 测加 `display:Block` 上下文——`ComputedStyle::default().display=Inline`，旧测隐式假设 block）。

**A/B（stash 重建 baseline，release）**：① **CSS2/box-display 38 (31.7%)→39 (32.5%) = +1**（121 案 join：FAIL->PASS=1 / PASS->FAIL=0；唯一翻转 = 目标案 1.82→0.70 PASS，div.sibling abs_y 155→71 gap 消除）；② **CSS2/margin-padding-clear（R743/R744 风险 dir）310→310 net-0**；③ CSS2/normal-flow 604→604 net-0；④ CSS2/linebox 121→121 net-0；⑤ welcome product-smoke 16.57%→16.57% 不变（无 inline 垂直 margin）。**零回归**。

**为何零回归**：§8.3 是 chromium 同行为（非替换 inline 垂直 margin 无效果），旧 ZW 错误应用 → 向 chromium 收敛 = 净正或中性。converter 上游修覆盖 span 节点 + anon 盒（都经 computed_style_to_taffy），故假设 1 的 anon 单点修冗余。详见 [`evidence/r1058-inline-vertical-margin-zeroed-boxdisplay-plus1-2026-07-05.txt`](./evidence/r1058-inline-vertical-margin-zeroed-boxdisplay-plus1-2026-07-05.txt)。

**意义**：R109 Phase-0 附带 yield——map 中定位的 box-display borderline 簇真根因（converter §8.3 缺失）= 独立 clean lever，不依赖 R109 FR-002/003 多 session。converter §8.3 inline 垂直 margin 归零是 foundational correctness，为后续 R109 FR-002（bg）+ block-in-inline margin-collapse 簇（multiple-block-in-inlines-margins-collapse 1.05% / block-in-inline-margins-collapse-with-trailing-block 1.11% 等 10 案 1.0-1.15%）提供正确基底。★R1058 证明 R109 Phase-0 map + 精准定位仍能在多 session 结构性任务中产 single-session clean yield（区别 R1056/R1057 net-negative/no-op）。

**▶ 下会话**：① R109 FR-002（匿名块盒区容器 bg 涂布，paint 侧，R1058 已清 inline margin 基底）；② block-in-inline margin-collapse 簇（10 案 1.0-1.15% flip 候选，R1058 后可能 unlock）；③ R109 FR-003（split inline border 归属，case-a inline 级）。

### R1057 display:list-item marker 门控修正（tag→display）REFUTED = CSS2/lists -1（li→ListItem 致 list-item-dynamic-color +0.60 PASS→FAIL）+ 目标簇 list-style-position-applies-to-008/009/010/016/017 delta=0.00（likely R109 anonymous-box mask）·R740 strategy ② clean lever 耗尽第三证·零 net 源码·纯调查

承 R1056 CONTINUE（fresh chr-vs-ZeroWeb per-case scan，R740 strategy ② 找 R689/R716 类 clean correctness lever）。本轮扫 CSS2/lists/colors/values + generated-content（fresh dir），定位 list-style-position-applies-to 簇（5 案 @ 3.37-3.73% identical diff = 单一系统性 cause 候选），**假设「missing marker on `<span display:list-item>`」A/B 证伪，net-negative 已回退**。

**假设**：`paint_list_marker`（text.rs:403）门控 `local_name()=="li"`（tag-name）是 `<li>` UA 默认误设 Block（lib.rs:54，应 ListItem）的 compensating hack → WPT 簇用 `<span display:list-item>` 拿不到 marker = 3.37% diff。

**改动（3 部分已 git stash drop 回退，零 net 源码）**：① lib.rs UA default `"li"` Block→ListItem（CSS §12.5 spec）+ 测试列表移除 li + 新 `test_li_defaults_to_list_item`；② text.rs:403 gate `local_name()=="li"`→`style.display != ListItem`（按 computed display 判定）+ DisplayValue import。converter ListItem→taffy Block（layout 不变）。style-system 测 + clippy 全绿。

**A/B（CSS2/lists 157 案 join）**：CSS2/lists baseline **144 (92.3%)→with 143 (91.7%) = -1 net**。唯一翻转 = `list-item-dynamic-color.html` **PASS→FAIL (0.94→1.54, +0.60)**（li→ListItem 改变 `<li>` marker 渲染）。**目标簇 delta=0.00（精确零）**：list-style-position-applies-to-008/009/010/016/017 (3.37-3.73%) + 015 (1.09%) 全 0.00；list-style-image-004/007 +0.02。

**★ 两路证据证伪「missing marker」假设**：① 目标簇 delta=**0.00 精确零**——marker 若新渲染必有像素变化（6px dot ~41px²），delta=0.00 → `<span display:list-item>` 在 box-tree 中**没带着 ListItem computed display 到达 paint_list_marker**；② list-item-dynamic-color +0.60 证明 fix 生效但**只影响 `<li>`，不影响 `<span>`**。

**★ likely 真因 = R109 §9.2.1.1 anonymous-block mask**：测试结构 `<div display:inline><span display:list-item>>`——inline 父含 block 级子触发 CSS §9.2.1.1 匿名块拆分（R109 territory，同 memory R255 morning-work 4× 高度机制）。ZW 匿名块包装（tree.rs R109）很可能用**父 div 的 node_id/style**（display:inline）包装 span，而非保留 span 自己的 element 身份（display:list-item）→ paint_list_marker 见 div 的 inline → gate 失败 → 无 marker。3.37% 主导 = orange box 几何（margin-left:1in / span vs div sizing）非 marker。与 R1047/R1052 vertical 耦合证 = **同一 R109 架构缺口**。

**裁决**：marker 门控修正对目标簇零效果（R109-blocked），致 `<li>` 副作用 -1。**REFUTED git stash drop 回退**。CSS spec-correct（li→ListItem + display gate）但在 ZW 当前 R109 匿名块架构下不产 yield，须先解 R109。

**R740 strategy ② clean lever 耗尽第三证**（R740+R1053+R1057）：本轮 fresh dir 扫描全部落 multi-session structural——generated-content = R554 pseudo-element gap / CSS2/values units = font-wall（fontdue Ahem advance）/ CSS2/lists marker = **R109 anonymous-block mask（本轮新锁定）** / CSS2/colors = 1 deep fail。**无新 clean single-session lever**。详见 [`evidence/r1057-list-item-marker-gate-refuted-2026-07-05.txt`](./evidence/r1057-list-item-marker-gate-refuted-2026-07-05.txt)。

**▶ 下会话**：① **R109 §9.2.1.1 anonymous block 攻坚**（最高频 unblocker：本轮 list-item + R1047/R1052 vertical + R255 morning-work 4× 全指向它；deadlock 历史但 EV 最高，multi-session）；② Phase-A IFC 单次源统一续（非 font-metric 子任务）；③ fresh dir 续扫 css-fonts/backgrounds borderline（EV 低，三证耗尽）。★勿再：list-item marker gate / `<li>` UA default 单 session lever（R1057 REFUTED R109-blocked）/ 盲扫 top-worst（三证）/ font-wall single-knob（六证）/ vertical 单点 bundle（四证）。

### R1056 CJK ascent 1.160 wiring 实测 net-negative 已回退 = css-text segment-break -13（13 案 PASS→FAIL 全 CJK borderline）压过 welcome -0.19pp 改善·font-metric single-knob 第六证·零 net 源码·纯调查

承 R1055 CONTINUE ①（水平 CJK font-wall 小切片：wire NotoSansCJK 1.160 metric）。本轮实施 R1055 forecast 的 CJK ascent wiring（`ascent_ratio_lookup` 加 `is_cjk` 分支，CJK 文本 0.928→**1.160** = NotoSansCJK 真实 ascent；新 `text_has_cjk` 检测片段文本），**A/B 硬数据复证 R1055 forecast「CJK 非 WPT high-yield 轨道」，net-negative 已回退**。

**改动（已 git stash drop 回退，零 net 源码）**：`inline/mod.rs` `ascent_ratio_lookup(overrides, node_id, is_ahem, is_cjk)` 加第 4 参 + CJK 分支（Ahem→0.8 优先 / 非-Ahem+CJK→1.160 / 非-Ahem+Latin→0.928）；`apply_vertical_alignment` 两处调 `text_has_cjk(&r.text)`（strut dominant mod.rs:1718 + per-run mod.rs:1749）；`ascent_ratio_for` dormant 传 false。新单测 `test_r1056_cjk_ascent_ratio_branch`（CJK 1.160 / Latin 0.928 / Ahem 优先 / text_has_cjk 6 区块）。layout-engine lib 70 测全绿 + clippy 干净。

**A/B（stash 重建 baseline，release 构建）**：① **product-smoke welcome（84 CJK UI 标签）**：baseline **16.57%**（79545px，与 R1055 记录精确匹配）→ with R1056 **16.38%**（78628px）= **-0.19pp 改善**（确定性像素位移，CJK 1.160 更近 chromium）；② **css-text-decor oracle**：108/242→108/242 **net-0**；③ **css-text oracle**：**357 (21.6%)→344 (20.8%) = -13 净回归** ❌。

**★ ORACLE_DUMP_ALL 逐案 A/B（1651 案 join）定位 -13 全在 `css/css-text/line-breaking/segment-break-transformation-rules-*`**（CJK 段分隔符变换规则测试），FAIL→PASS 0 案。**13 案全 borderline**（base 0.89-0.98% → with 1.09-1.22%，delta +0.18~0.24pp）：OLD 0.928 让这些 CJK 用例勉强 <1% oracle-pass，NEW 1.160 推过 1% 阈值 → PASS→FAIL。**非阈值噪声散布**（13 案全在同一 CJK 子簇 = 真实 CJK 几何信号）。

**裁决：net-negative 回退**。trade-off = WPT css-text **-13**（CJK segment-break）+ css-text-decor net-0 / welcome **-0.19pp** 改善。WPT pass-count 是主验收口径（DC-14），product-smoke 是回归门禁非 yield 指标；-13 WPT 换 -0.19pp product-smoke = 净负。**R1056 git stash drop 回退**。

**★ font-metric single-knob net-negative 第六证**（R834 strut 0.8→0.928 welcome +0.07pp / R836 Path B +1.12pp / R849 全链 / R875 / R1052 vertical -26 / **R1056 CJK 1.160 css-text -13**）。机制一致：**fontdue≠Skia 行级累积**——「spec-correct real metric」（1.160 NotoSansCJK 真值）反把 borderline CJK 用例推离 oracle，因 surrounding pipeline（fontdue raster y_offset / half-leading）未 coherence 对齐 chromium。OLD 0.928（实为 DejaVuSans Latin 度量，非 CJK 真值）碰巧在 CJK segment-break 用例更近 oracle（compensating 近似）。

**R1055 forecast 硬复证**：welcome 改善（-0.19pp）= R1055 预测的 product-smoke yield；css-text -13 = R1055 预测的「WPT pass-count 非 high yield」；segment-break 簇 = R1056 新锁定的「CJK-ascent 敏感 WPT 子簇」。**真修复须全 pipeline coherence**（R848 路线图：layout strut 用真实 per-font metric + paint v_offset −real_ascent + half-leading (lh−(asc−desc))/2 + **fontdue→chromium-matching rasterizer**）；无 rasterizer 替换，任一 layout-side CJK metric 改动对 WPT CJK 簇 net-negative。详见 [`evidence/r1056-cjk-ascent-1160-net-negative-revert-2026-07-05.txt`](./evidence/r1056-cjk-ascent-1160-net-negative-revert-2026-07-05.txt)。

**▶ 下会话**：① R109 §9.2.1.1 anonymous block 攻坚（unblock block-in-inline 簇）；② Phase-A IFC 单次源统一续（layout IFC==paint IFC，非 font-metric 子任务）；③ taffy maintainability 升级（R304 vertical 收益已 ruled out）；④ product-smoke morning/wintertc 作 CJK 验收口径（须先 fontdue 替换）。★勿再：盲扫 top-worst / vertical 单点或 bundle / taffy R304 vertical / font-wall Latin / **CJK ascent single-knob（R1056 第六证）**。

### R1055 font-wall 实测 = Latin ruled out（DejaVuSans=0.928 匹配 R990）+ CJK 潜在 yield（NotoSansCJK=1.160 vs 0.928，受限）+ welcome 16.57% 非字体度量·零 net 源码·纯调查

承 R1054 CONTINUE（font-wall R887 攻坚起步）。本轮 fontdue line_metrics_full 实测字体度量，**font-wall Latin ruled out，CJK yield 受限，font-wall 非 WPT pass-count 高 yield 轨道**。

**★ font-wall Latin ruled out**：DejaVuSans.ttf（welcome/morning 的 sans-serif 实际字体）ascent=14.852@16 → ratio=0.9282，descent=-0.2358，line_gap=0。R990 的 0.928 常数（非-Ahem ascent ratio）= DejaVuSans 真实度量，**精确到 4 位小数匹配**。→ wiring FontMetricProvider（Phase A §12.6 step-2）对 Latin **零收益**（0.928 已真值）。R631 字体选择 0% + 本轮度量匹配 = font-wall Latin 双重 ruled out。

**★ CJK 潜在 yield（受限）**：NotoSansCJK-Regular.ttc ascent=18.560@16 → ratio=1.1600，descent=-0.2880，line_gap=0。vs R990 常数 0.928 **偏差 25%**（CJK 字体典型 ascent>1.0em 含 ruby 空间）。ZW 对 CJK 用 0.928（应 ~1.160）→ strut baseline 偏低 0.232·fs/行。**但 yield 受限**：CJK-重 WPT 用例多 vertical-blocked（R1054）；morning.work 是 product-smoke 非 WPT-count；R631 未测 metric 变化（只测 matching），1.160 vs 0.928 是未测独立变量。font-wall CJK = 限于水平 CJK + product-smoke，**非 WPT pass-count 高 yield 轨道**。

**welcome 16.57% diff 非字体度量**：UI diff vision tool 报告「严重字符间距破坏」（ZeroBrowser→"Zer oBr owser"）——**纠偏**：welcome.html 故意用 letter-spacing 0.1em/0.05em/0.08em（line 48/57/90），vision tool 误读 letter-spacing 为「断字」，非真 bug。16.57% 残余 = letter-spacing 像素精度 + fontdue vs chromium AA + 行盒亚像素（非字体度量，DejaVuSans 匹配）。

**战略裁决**：font-wall 双重 ruled out for Latin；CJK 25% 度量偏差但 yield 受限（vertical-blocked + product-smoke 非 WPT-count）。**font-wall 非 WPT pass-count 高 yield 轨道，暂缓**。整体 plateau 复盘（R1051-R1055 五轮 docs-only）：clean lever 耗尽（R740+R1053）/ vertical blocked（R1052+R1054 四证）/ taffy ruled out（R1054）/ font-wall ruled out 或受限（R1055）。剩余推进面：R109 anonymous block（deadlock 历史）/ Phase-A IFC 统一续（baseline 像素精确）/ 水平 CJK font-wall 小切片（1.160 wiring，yield 受限）。★勿再：盲扫 top-worst / vertical 单点或 bundle / taffy R304 / font-wall Latin。详见 [`evidence/r1055-font-wall-dejavu-ruled-out-cjk-potential-2026-07-05.txt`](./evidence/r1055-font-wall-dejavu-ruled-out-cjk-potential-2026-07-05.txt)。

**▶ 下会话**：① 水平 CJK font-wall 小切片（wire NotoSansCJK 1.160 metric，A/B 水平 CJK 用例 + morning product-smoke，受限 yield 但可尝试）；② 或 R109 anonymous block 攻坚；③ 或 Phase-A IFC 统一续；④ 或重新评估 product-smoke morning/wintertc fixture 作验收口径（CJK font-wall 在产品验收面有意义）。

### R1054 vertical 完整 bundle（spec-correct）net -28 已 revert + taffy R304 ruled out（不支持 writing-mode）·零 net 源码·纯调查

承 R1053 CONTINUE（taffy R304 评估起步）。本轮 web research 推翻 taffy R304 前提，转 vertical 完整 bundle 实验，**spec-correct 仍 net -28（4 证）已 revert**。

**★ taffy R304 ruled out**：R1053 CONTINUE 推荐 taffy 升级作 vertical unblocker（基于 memory R304/R849 旧评估）。web research 实证 taffy（所有版本含 0.11）**不支持 writing-mode / vertical text**（maintainer Issue #308 视作「ergonomic 改进」未实现；taffy 把 text layout 全委托宿主 MeasureFunc Issue #216）。→ taffy 升级**不解锁 R1043/R1052 vertical block-flow**，memory R304/R849 评估过时。升级仅余 maintainability / minor grid-flex fix 收益，vs 590 refs + 4 major breaking + 全布局回归风险，**ROI 不成立 indefinitely defer**。

**vertical 完整 bundle 实验**（R1052 Slice α 四层：A container_width WM-aware + B trailing-space 裁剪 + C line-height vertical col-width [store_font_sizes_from_ifc vertical frag.line-height 在 frag.width 非 frag.height] + D vertical emphasis re-enable [painter/text.rs char_advance_is_y 分支 mark 左/右侧垂直居中]）。EMPHDBG 实证 **bundle 字符几何 + mark 位置 spec-correct**（006d 試 mark_x=0 char@base_x=8 mark_y=55 char_pos=67 left=true，mark 左置垂直居中 ✓；col_width 经 C = 80 修前 16）。

**A/B = net -28**：css-text-decor 108→82（-26 PASS→FAIL）；css-writing-modes 56→54（-2，比 A+B alone net-0 还差 2）。即加 C+D（line-height+emphasis，全 spec-correct）仍 net -28，EMPHDBG 确证 D 渲染且位置规范正确。

**★ 裁决：vertical 即使 spec-correct 仍 net-negative（R1047/R1050/R1052/R1054 四证）**。根因 = vertical 须 near-perfect 才 net-positive：① vrl block-flow mirror（R1043）仍缺（vrl 用例 86-87% 未修，bundle 只解 vlr）；② 小用例（emphasis 簇）oracle diff 由残余不完美（CJK fontdue advance、mark 像素精确位、subpixel）主导——OLD 错误水平布局「紧凑」偶然近 oracle <1% pass，NEW spec-correct 垂直因残余不完美推过 1% 阈值 → PASS→FAIL；③ OLD「错但近」反比 NEW「对但不完美」更近 oracle。vertical 子系统**单点、多点、全 spec-correct bundle 均 net-negative**，须 (a) vrl mirror + (b) CJK 字体度量精确 font-wall + (c) mark 像素精确位**全满足**才可能 net-0/正。当前均缺，vertical track **genuinely blocked**。

**forward**：① font-wall 攻坚（R887 per-font 度量，独立于 vertical，解 product-smoke + 多 font-wall 用例）；② R109 anonymous block 攻坚；③ Phase-A IFC 统一续；④ vertical 暂搁置（blocked 4 证，除非 vrl mirror + font-wall 同解）。★勿再：盲扫 top-worst（双证耗尽）/ vertical 单点或 bundle（4 证 net-negative）/ taffy R304（ruled out 不支持 writing-mode）。详见 [`evidence/r1054-vertical-bundle-taffy-ruled-out-2026-07-05.txt`](./evidence/r1054-vertical-bundle-taffy-ruled-out-2026-07-05.txt)。

**▶ 下会话**：① font-wall per-font 度量（R887 provider wiring，多 session，独立于 vertical）；② 或 R109 anonymous block 攻坚；③ 或 Phase-A IFC 统一续。

### R1053 5 目录 top-worst 扫描（clean lever 耗尽 R740 复证）+ root-abspos sizing 实验 net-flat-to-negative 已 revert·零 net 源码·纯调查

承 R1052 CONTINUE（vertical Slice α / 转非 vertical 轨道）。本轮按 R740 战略转向 ②（直接 chr-vs-ZeroWeb 对比找新 clean lever）扫 css-grid / css-text-decor / css-tables / css-position 4 目录 top-worst，**全部 entangled，clean single-session lever 耗尽复证**；root-abspos 实验 html 几何修对但 body 未 re-layout 故 net-flat-to-negative 已 revert。

**5 目录扫描结论**（top-worst 全 entangled）：css-grid（baseline-synthesized vrl/vlr/srl/slr vertical / replaced %height grid-in-flex complex / table-grid-item JS-dynamic）；css-text-decor（text-decoration thickness/dotted/line 全 font-wall 谱系 R1045 证伪 / text-decoration-inset CSS4 draft）；css-tables（table-cell-width-0 实测 ZW 已正确 w=8 intrinsic，20% diff = 默认字体+border-collapse font-wall/structural / collapsed-border-vertical vertical）；css-position（backdrop structural / dynamic-relayout JS / in-inline R109 / root-element abspos §2）。**R740 复证：clean lever 在已工作目录耗尽**。

**root-abspos sizing 实验**（position-absolute/fixed-root-element-{flex,grid} 4 案 4.46%）：`<html>` 自身 abspos/fixed + 全 inset 应按 §10.3.7/§10.6.4 stretch（border-box = 视口 − insets）。ZW 实测 html h=64（shrinkwrap）w=715（应 530/770）。Fix = `size_root_abspos_to_viewport`（abspos.rs post-processing，仅 root abspos/fixed 时补 width/height，gate 严零 static 影响）。**html 几何修到规范正确（h=530 w=770）**但 **A/B 4.46%→4.52%（+0.06pp net-flat-to-negative）已 revert**：body 未 re-layout（taffy 用旧 html 715 layout body w=689），post-process 改 html 后 body 文本换行用旧 content_width 不重排，残余 diff 抵消 border 修对。**裁决**：post-processing 不足（文本换行依赖 taffy 期 body content_width），须 pre-layout（converter 设 root taffy size）或 two-pass（R695/R1018 基建 set+mark_dirty+recompute），属 taffy 0.7 root quirk 谱系（R123/R500），yield 小（4 案）effort 高 → defer。

**裁决与 forward（战略路径，多 session）**：clean single-session lever 耗尽（R740+R1053 双证）。后续须转结构性多 session：① **taffy 升级 R304**（R304/R849 列 #1 viable lever，解锁 native vertical block-flow 减 R1043/R1052 耦合 + 541 ref + 108 alignment + native float，一次性 unblock 多 manual workaround，EV 最高）；② R109 §9.2.1.1 anonymous block（unlock block-in-inline 簇）；③ Phase-A IFC 统一（行盒度量）。★ 勿再盲扫 top-worst 找 clean single-session lever（双证耗尽）。详见 [`evidence/r1053-scan-root-abspos-investigation-2026-07-05.txt`](./evidence/r1053-scan-root-abspos-investigation-2026-07-05.txt)。

**▶ 下会话**：① taffy 升级 R304 起步（评估 541 ref + 108 alignment + native float 冲突面，列迁移计划，最高 EV 多 session）；② 或 R109 anonymous block 攻坚；③ 或 Phase-A IFC 统一续（baseline 定位 / 字体度量）。

### R1052 ★vertical IFC container_width=0 根因 = 纠正 R1051 handoff 诊断（axis-swap 已存在）+ vertical 耦合系统三证（单修 inline-flow net -26 已 revert）·零 net 源码·纯调查

承 R1051 CONTINUE（R109 vertical inline Slice 1 实施）。本轮按 handoff §3 试实施 Slice 1，**VIFCDUMP 实证推翻 R1050/R1051 诊断，单点修复 net-negative 已 revert，handoff doc 升级 v1.1**。

**★ 纠正 R1050/R1051 诊断**：R1050 EMPHDBG 测 006d chars x=8,24,40,56,72 递增 → R1051 handoff 断言「IFC 缺轴交换（current_x/current_y 未互换），须新建双模式 char 推进」。**本轮 VIFCDUMP 实证推翻**：轴交换代码**早已存在且正确**（commit 942a2948，2026-06-09，`break_items_into_columns` mod.rs:1450 字符沿 y 推进 / 列沿 x 推进；paint `char_advance_is_y` text.rs:1392-1450 也对）。**真根因 = vertical IFC 的 `container_width=0.0` → max_depth=0**：`max_depth = self.container_width`（mod.rs:1452），而 container_width 取 `root.content_width`（inline_finalization.rs:619）/ `box_node.content_height`（painter/text.rs:797）= 元素水平 block 尺寸，vertical-lr auto 时=0 → `current_depth + word_height > 0` 恒真 → **每字符触发列断 → 每字符各占一列沿 x 排列 → chars 横向排列**。vertical 应取 `content_height`（竖直 inline 尺寸 = 字符向下推进可用深度）。R1050 EMPHDBG 的 x 递增是 max_depth=0 副产物，非缺轴交换。

**修复实验（已 revert）**：Fix A = container_width WM-aware（2 处：compute_final + paint Path B，vertical 取 content_height / horizontal 取 content_width，gate 隔离 horizontal 字节一致）；Fix B = trailing-space 裁剪（split_into_words mod.rs:1983-1987 为非末词加 trailing space 与注释「CJK 不带尾部空格」矛盾，vertical 下 word_height 虚高 fs+space_w 致列断提前；break_items_into_columns 词循环头 trim）。Fix A+B 实测 006d chars 几何**完全规范正确**（col0 run0..4 x=0 常量，y=0,16,32,48,64 连续 fs 间隔，单列）。

**★ A/B = net-negative 已 revert**：css-text-decor **108→86（Fix A，-22）→ 82（Fix A+B，-26）**；css-writing-modes 56→56（net-0，block-flow R1043 主导，line-box-direction-vlr-014 修后仍 86.86%）；006d 单案 1.00%→1.01%（持平）。即便 006d 字符几何规范正确，oracle 仍 net -26。

**★ 裁决：vertical 渲染 = 耦合系统（R1047/R1050/R1052 三证）**。单修 inline-flow 致输出**既不同于旧错误布局、又不同于 chromium**（block-flow R1043 容器定位错 + line-height:5 vertical 列宽未传 col_width=16 应 80 + emphasis `!char_advance_is_y` 门控跳过 vertical 装饰全缺 + paint Path B 空-styles），故净负。三证（R1047 sibling-push / R1050 vertical emphasis / R1052 vertical inline-flow）一致：**vertical 子系统单点修复 net-negative，须多层同步修**。

**意义**：R1052 真价值 = ① 纠正 handoff 诊断（轴交换已存在，真根因 container_width=0，后续勿再「新建双模式 char 推进」）；② 锁定精确靶点（§2 Fix A+B + line-height vertical 列宽 + block-flow R1043 + emphasis re-enable，四层同改）；③ 耦合系统三证裁决；④ VIFCDUMP 探针代码（evidence §6）作 vertical IFC 调决定性工具。handoff doc 升级 v1.1（§0/§2/§3/§4/§6 全面纠正）。详见 [`evidence/r1052-vertical-ifc-container-width-zero-2026-07-05.txt`](./evidence/r1052-vertical-ifc-container-width-zero-2026-07-05.txt)。

**▶ 下会话**：① vertical 多层同步（Slice α：container_width fix + block-flow R1043 converter 层镜像 + line-height vertical 列宽 + emphasis re-enable，四层同改 A/B 守 net-0/正）—— 最高 yield 轨道但多 session；② 或 taffy 升级（R304）减 block-flow 耦合；③ R702 margin-collapse-through（em-inherit 11%，yield 小）；④ font-wall per-font 度量（R887）。**勿再单点修 vertical 任一子层**（三证 net-negative）。

### R1051 R702 margin-collapse-through 调查 = taffy collapse-through 深 bug·ruled out 单 session·+ R109 vertical inline handoff doc（最高 yield 多 session 轨道蓝图）·零 net 源码·纯调查

承 R1050 CONTINUE（R109 vertical inline / R702 / font-wall）。本轮 LAYOUT_DUMP 调查 R702 + 写 R109 vertical inline 实施 handoff，**结论：R702 单 session ruled out，R109 handoff doc 落地供后续 session 实施**。

**R702 margin-collapse-through 调查**（`margin-em-inherit-001` 11.25%）：LAYOUT_DUMP 实测 ZW 渲染：
```
html   abs_y=0   h=280
body   abs_y=56  h=196  mt=56  dmt=8    ← body effective mt=56（应 16）
  p    abs_y=56  h=40   mt=16           ← p 与 body 同 y（重叠）
  div(gp) abs_y=56 mt=56                ← grand-parent 同 y
```
ref 渲染：body abs_y=16 mt=16（正常）。**根因**：`#parent` margin-top 56 经无 border/padding 的 `#grand-parent` **collapse-through** 上提，ZW/taffy 把 max(body 8, p 16, #parent-through-gp 56) = 56 **全应用到 body**，忽略 `p` 元素 content 应作为 separator 阻断 collapse 链（CSS2 §8.3.1：adjoining margins collapse，但 p 的 in-flow content 使 p 顶/底 margin 不再 adjoining，#parent mt 应与 p_mb 折叠 max(56,16)=56 落在 p 与 gp 之间，非上提到 body 顶）。**裁决**：taffy 0.7 CollapsibleMarginSet「intervening content blocks collapse-through」逻辑不完整，深 margin 算法，ZW postprocess 重分布 collapse margin 风险高（R1047 sibling-push 同族 net-negative 先例），**单 session ruled out**。yield 小（em-inherit 簇 ~3 案）。

**R109 vertical inline 实施 handoff doc LANDED**（[`vertical-inline-layout-handoff.md`](./vertical-inline-layout-handoff.md)）：承接 R1050 根因（IFC `current_x` 水平推进 vertical 文本），产多 session 实施蓝图：
- **问题**：IFC（mod.rs:973+）`current_x += char_width` 水平推进每字符，vertical-rl/lr 应 `current_y += char_height` 垂直推进（chars 同 x 列、y 递增），line-break = column-break。
- **影响**：vertical 子域全 R109-blocked（emphasis/ruby/text-decor/bidi-vertical + css-writing-modes ~250 vertical 案 86-87%），解锁 yield **当前 corpus 最高**（潜在 flip ~30-80 案）。
- **实施路径**：IFC 双模式（horizontal `current_x`/`current_y` ↔ vertical 轴交换），paint `char_advance_is_y` 已存在协调。Slice 1 = 纯 CJK 单列无 float/ib 紧 gate（net-0 守 horizontal-tb 字节一致），Slice 2+ = word-wrap/float/ib/Latin advance/text-orientation 逐项扩展。
- **风险**：taffy 0.7 vertical BLOCK flow 方向（R1043 rl packing）仍 taffy-blocked，本 handoff 只解 inline flow（char 推进），两层独立可分步；paint Path B 空-styles（R890）须协调；line-height/baseline 三方同改（R834 单点 net-negative 先例）。

**意义**：R1050 根因 → R1051 实施蓝图，R109 vertical inline 轨道从「诊断」进入「可实施」阶段。后续 session 可按 handoff §3 Slice 序起步（首 slice 纯 CJK 单列 gate，net-0 守回归）。**勿再以 vertical 子域为独立 lever**（须先 R109 vertical inline 解锁）。

**▶ 下会话**：① **R109 vertical inline Slice 1**（按 handoff，IFC `if self.is_vertical` 纯 CJK 单列分支，net-0 守 horizontal-tb，1 个 vertical 用例 frag 几何对）——最高 yield 轨道首切片；② 或 R702 多 session 起步（须 taffy collapse-through 重设计，yield 小不优先）；③ font-wall per-font 度量（R887 provider wiring 多 session）。

### R1050 text-emphasis 简写 LANDED（net-0 correctness）+ 垂直 emphasis net -8 回退已 ruled out·★R109 vertical inline 布局根因（IFC 水平布局 vertical 文本）·vertical-mode emphasis 簇 ruled out

承 R1049 CONTINUE（logical-props WM-aware 主体已尽，转更高 yield 轨道或 sideways-lr）。本轮扫描近失簇定位 text-emphasis-position-property-{003,005,006}-{d,e,f,g}（~12 案 1.00-1.02%，vertical-lr），**调查发现 R109 vertical inline 布局深层根因，LANDED 简写 correctness fix，垂直 emphasis ruled out**。

**近失簇扫描**（css-text/fonts/text-decor [1.0%,1.5%]）：text-emphasis-position d/e/f/g 簇（vertical-lr，1.00-1.02%）最大；bidi-007 簇全变体同%（bidi-normal=isolate=embed=1.31%，证共享 baseline 非 bidi 算法 bug）；vertical-align-baseline（font-wall R990 territory）；inline-size/block-size 已隐式 WM-aware（converter swap，apply.rs:122 注释，slice ③ ruled out double-swap）。

**缺口 1 — `text-emphasis` 简写完全未展开**（R1021 只实现 longhand text-emphasis-style/position）：corpus `text-emphasis: circle` 被静默忽略→text-emphasis-style=None→无 mark。本轮 LANDED `expand_text_emphasis`（shorthand/mod.rs）：`<style> || <color>`，color token 剥离（ZW 暂未存储 text-emphasis-color），剩余拼回 style。3 单测（style-only / filled circle red 剥 color / string "*"）。

**缺口 2 — 垂直 emphasis 位置（R1021 line 1494 `!char_advance_is_y` 跳过垂直）**：实现 R1050 垂直 emphasis 块（mark 绘字符左/右侧按 position 含 Left/Right）。**A/B 决定性 net -8**：css-text-decor 108→100（8 案 PASS→FAIL）。EMPHDBG 揭示根因——**已回退**。

**★ R109 vertical inline 布局深层根因**（比 R1043 block-flow 方向更深）：EMPHDBG 实测 006d（vertical-lr）chars x=8,24,40,56,72（**x 每字符递增 Δ16=fs**），y=67,83,99,115,131（**y 也递增**）。即 ZW 的 IFC 对 vertical-lr 文本**水平布局**（chars 左→右排列，仅旋转 glyph 90°），而非规范要求的**垂直布局**（chars 上→下，同 x 列，y 递增）。故：
- 垂直 emphasis mark 即使按垂直语义定位，底层文本位置错（水平），mark 也错位 → 净负。
- text-emphasis-position vertical 簇（d/e/f/g 1.00-1.02%）非 emphasis-position bug，是 R109 vertical inline 布局缺口主导（残余 CJK font 噪声 + vertical 布局错）。
- 与 R1043「taffy Block 无方向 packing」互补：**block-flow 方向（R1043）+ inline-flow 方向（R1050）双层 vertical 缺口**，均 taffy 0.7 / IFC 架构限制，须 layout 重构或 taffy 升级（R304 多 session）。

**A/B**：仅简写展开 = **net 0 oracle**（css-text-decor 108/242、css-writing-modes 56/784 持平），**0 PASS→FAIL 回归**（dump diff 确证）。简写展开是 correctness fix（水平案经 R1021 正确渲染 mark；vertical 案因 R109 布局错 mark 位置也错，但 site 2 `!char_advance_is_y` 门控跳过故 vertical 不渲染 mark→无 vertical 回归）。垂直 emphasis 块 = net -8 已回退。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（style-system +3 新简写测，全树零失败）/ **product-smoke welcome 16.57% < 20%** ✓。

**意义**：text-emphasis 简写 LANDED 是 correctness 修复（CSS Text Decoration 3 §3.1 简写原完全未处理，net-0 因 corpus emphasis 测试多 CJK 主导 diff）。R1050 真价值 = **R109 vertical inline 布局根因定位**（IFC 水平布局 vertical 文本，chars x 递增），为未来 R109 vertical 解锁轨道提供精确靶点（须 IFC 支持垂直字符推进，非仅旋转）。vertical-mode emphasis/ruby/text-decor 簇全部 R109-blocked，勿再以这些为 lever。详见 [`evidence/r1050-text-emphasis-shorthand-vertical-ruled-out-2026-07-05.txt`](./evidence/r1050-text-emphasis-shorthand-vertical-ruled-out-2026-07-05.txt)。

**▶ 下会话**：① R109 vertical inline 布局是新定位的高 yield 多 session 轨道（IFC 支持垂直字符推进，chars 同 x 列 y 递增；解锁后 vertical-mode 全簇 emphasis/ruby/text-decor/writing-modes flip）；② 或 R702 margin-collapse-through（em-inherit 11% 真布局 bug）；③ 或 font-wall per-font 度量（R887 provider wiring）。vertical-mode 子域（emphasis/ruby/bidi-vertical）已 ruled out 非独立 lever，须 R109 vertical inline 解锁。

### R1049 margin/padding/inset 逻辑属性 writing-mode-aware LANDED·logical-props 轨道 slice ②·零回归（horizontal-tb 字节一致）·净 0 oracle·unified PhysicalSide helper

承 R1048 CONTINUE（logical-props 轨道 slice ②：margin/padding/inset 转 WM-aware / sideways-lr / table col-border）。本轮把 margin/padding/inset 逻辑属性从 R143 静态 horizontal-tb 映射升级为 **writing-mode-aware**，**零回归 LANDED**。

**实现**（CSS Logical Properties §1 + Writing Modes §6）：
- `apply_advanced.rs`：把 12 个 margin/padding/inset logical longhand arm（原 R143 静态 `style.margin_top = v` 等）改为调用新 helper `apply_logical_{margin,padding,inset}(style, axis_inline, start, value)`，内部按 `logical_physical_side(axis_inline, start, &style.writing_mode)` 映射物理边。
- **重构 R1048 helper**：`BorderSide` → `PhysicalSide`（通用），抽出 `logical_physical_side(axis_inline, start, wm)` 公共映射器；`logical_border_physical_side` 改为薄封装（解析属性名 → 调 `logical_physical_side`）。border margin padding inset 4 组共用一套映射。
- 4 新单测（tests/core.rs）：horizontal-tb 字节一致 + vertical-rl margin（block-start=right/inline-end=bottom）+ vertical-lr padding（block-start=left/inline-start=top）+ vertical-rl inset（block-end=left），全过。

**A/B（chromium Oracle，stash 对照 + ORACLE_DUMP_ALL 全 case diff）**：
- **css-writing-modes**：56/784 → 56/784（**净 0 oracle，0 PASS→FAIL 回归** dump diff 确证）。logical-props-002 1.050→0.990（borderline 1% 阈值噪声内，非稳定 flip）。
- **css-tables**：74/115 → 74/115（持平）。
- **css/CSS2/normal-flow**：604/746 → 604/746（**horizontal-tb 字节一致零回归确认**）。
- **0 PASS→FAIL flip 回归**（writing-modes 全 182 案 with-oracle dump diff 确证）；vertical-mode 不 flip 因 R109 taffy-blocked（margin 映射现在正确，但整体 vertical 渲染缺口更大）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（style-system 1933→1937 +4 新测，全树零失败）/ **product-smoke welcome 16.57% < 20%** ✓（一字不差）。

**意义**：logical-props 轨道 slice ②——margin/padding/inset 逻辑属性 WM-aware 化，**比 R1048 更干净**（R1048 有 3 案轻微恶化暴露下游 table col/colgroup 缺口；R1049 零恶化，纯 correctness 改进）。`PhysicalSide` + `logical_physical_side` 统一 4 组逻辑属性映射，为后续 slice（sideways-lr / table-internal border / R109 vertical 解锁后的 vertical-mode flip）奠基。horizontal-tb 下 WM-aware 输出与 R143 静态字节一致（block-start→top 等），故零回归；vertical-mode 映射现在正确但 R109-blocked 暂不 flip。详见 [`evidence/r1049-margin-padding-inset-wm-aware-landed-2026-07-05.txt`](./evidence/r1049-margin-padding-inset-wm-aware-landed-2026-07-05.txt)。

**▶ 下会话**：logical-props 轨道 slice ①（border）②（margin/padding/inset）均 LANDED WM-aware，轨道 foundational 已近完整。续推：① **inline-size/block-size 转 WM-aware**（R143 静态，同样可 WM-aware 化，horizontal-tb 字节一致）；② **sideways-lr writing-mode 支持**（enum + converter `apply_vertical_writing_mode` 加 SidewaysLr 变体）；③ **table-internal col/colgroup border 渲染**（R177 territory，解 R1048 的 003/004 恶化）；④ 或转 R109 vertical block-flow（taffy 多 session）/ R702 margin-collapse-through。logical-props WM-aware 化主体已尽，下 slice 转 sideways-lr 或换轨。

### R1048 ★border 逻辑属性（border-inline/block-start/end-{width,style,color}）writing-mode-aware LANDED·CSS Logical Properties §3 feature gap 修复·净 0 oracle·foundational enabling slice·零硬门禁回归

承 R1047 CONTINUE（R109 vertical-rl block-flow = taffy-blocked 多 session，转 logical-props feature gap）。本轮实现 border 逻辑属性（原完全未注册：parse/store/apply 全缺），**spec-correctness + foundational enabling slice，LANDED，净 0 oracle**。

**缺口确认**：margin/padding/inset logical 属性 R143 已有（horizontal-tb 静态映射，apply_advanced.rs）；inline-size/block-size R143 LANDED。**唯独 border-inline/border-block logical 属性完全未实现**（corpus 12+5 文件用，logical-props-001~004 / rules-groups.html 等）。CSS `border-inline-start: 5px green solid` 被静默忽略→无 border 渲染。

**实现**（CSS Logical Properties §3 + Writing Modes §6）：
- `shorthand/mod.rs`：4 简写（border-{inline,block}-{start,end}）经 `expand_border_side` 展开为 12 logical longhand（border-{axis}-{side}-{width,style,color}）。
- `apply_advanced.rs`：12 logical longhand 处理 + `logical_border_physical_side(property, &style.writing_mode)` 按 computed writing-mode 映射物理边：
  - horizontal-tb（ltr）：inline-start=left, inline-end=right, block-start=top, block-end=bottom
  - vertical-rl：inline-start=top, inline-end=bottom, block-start=right, block-end=left
  - vertical-lr：inline-start=top, inline-end=bottom, block-start=left, block-end=right
  - inline 轴 direction 暂按 ltr（vertical 模式 inline-start=top，logical-props-001 预期）。
  - 与 R143 静态映射不同，border 用 writing-mode-aware（因 border 是新属性零回归风险；margin/padding/inset 维持 R143 静态不动）。
- 6 新单测（tests/core.rs）：horizontal-tb inline-start/block-end + vertical-rl inline-start/block-start + vertical-lr block-start + 简写 color 路径，全过。

**A/B（chromium Oracle，stash 对照）**：
- **css-writing-modes**：56/784 → 56/784（**净 0 oracle**）。logical-props-002 1.05%→0.99%（borderline 噪声内，per-dir 持平确证非稳定 flip）。
- **css-tables**：74/115 → 74/115（**净 0 oracle**；rules-groups.html border-block-start horizontal-tb 未 flip）。
- **3 已 FAIL 案轻微恶化**（暴露下游渲染缺口，非 mapping bug）：logical-props-003/004（col/colgroup + vertical-rl，1.05→1.30，ZW table-internal col/colgroup border 渲染缺口 R177 territory）+ logical-physical-mapping-001（10.73→10.98，综合 8 writing-mode 含 sideways-lr 未支持）。
- **0 PASS→FAIL flip 回归**（dump diff 确证）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（style-system 1927→1933 含 6 新测，layout-engine 1024，全树零失败）/ **product-smoke welcome 16.57% < 20%** ✓（welcome 不用 logical border，一字不差）。

**意义**：CSS Logical Properties §3 feature gap 修复——border 逻辑属性首次实现，writing-mode-aware 物理边映射（horizontal-tb/vertical-rl/vertical-lr）。属 R885（font-bridge dormant）/R897（multicol Phase 2a enabling）式 **foundational enabling slice**：净 0 oracle 但为 logical-props 多 session 轨道奠基（margin/padding/inset 可后续转 writing-mode-aware；table-internal border 渲染改进后 003/004 可 flip）。3 已 FAIL 案恶化是暴露下游缺口（table col/colgroup border + sideways-lr），非本 mapping bug——mapping 单测全过、spec-correct。详见 [`evidence/r1048-border-logical-props-landed-2026-07-05.txt`](./evidence/r1048-border-logical-props-landed-2026-07-05.txt)。

**▶ 下会话**：① **margin/padding/inset logical 转 writing-mode-aware**（现 R143 静态 horizontal-tb，转用 `logical_border_physical_side` 同款映射，可能 flip vertical-mode margin/padding 簇）；② **table-internal col/colgroup border 渲染**（R177 territory，解 003/004）；③ sideways-lr writing-mode 支持（enum + converter，解 mapping-001 部分）；④ 或转 R109 vertical block-flow（taffy 多 session）/ R702 margin-collapse-through。logical-props 轨道 foundational 已立，可按 slice 续推。

### R1047 sibling-overlap grow-push postprocess 实验 = net -1 回退已回退·block-in-inline（R109）破坏拓扑·postprocess 层 ruled out·零 net 源码·纯调查

承 R1046 CONTINUE（③ R1018 两趟 + sibling 重定位：p 文本测量延迟致 padding-em-inherit-001 sibling 重叠）。本轮实现 postprocess「增高下移兄弟」对称分支并 A/B，**结论：net -1 oracle，postprocess 层 ruled out，已回退**。

**实现（已回退）**：`inline_finalization.rs::remeasure_inline_only_containers` 递归循环已有「收缩分支」（inline-only 容器收缩后把后续兄弟上移），本轮加对称「增高分支」——子块因 DOM 文本被 IFC 重测增高（taffy 原 content_height≈0）后，按 `grow_delta` 把后续普通流兄弟下移。门控演进：① 宽松 gate `old_content_height<1.0`；② DOM-text gate（加 `node_id` 有 Text 子）；③ 紧 gate（镜像 `needs_dom_text_remeasure` 全条件）。

**A/B（chromium Oracle，stash 对照）**：
- **驱动簇改善（未 flip）**：padding-em-inherit-001 **11.17→4.80%**、margin-em-inherit-001 11.25→跌出 top-15（<4.8%）、padding-percentage-inherit-001 7.47→跌出 top-15。3 案均 -6pp+，sibling 重叠 bug 确被修（单测 `test_r1047_text_block_grow_pushes_sibling_down` 验证 sibling.y ≥ p.y+p.height 不再重叠）。
- **零 oracle flip**：3 案均未跨 1%（残余 = R702 margin-collapse-through，多 session）。
- **回归（决定性）**：**css/CSS2/normal-flow 604→603（-1 oracle）**。ORACLE_DUMP_ALL 全 case diff 精确定位**唯一 PASS→FAIL flip = `block-in-inline-margins-003.html` 0.050%→2.870%（+2.82pp）**，另有 block-in-inline-first-line-001 2.58→3.14、block-in-inline-remove-006 0.60→0.67 等 block-in-inline 簇小幅恶化。
- 三种 gate（宽松 / DOM-text / 紧）**均 -1**——回归案有 DOM text，gate 收紧到 `needs_dom_text_remeasure` 全条件则同时破坏驱动案（`<p>` 含 `<br>`，br 是 inline LayoutBox 子违「仅 abs/fixed 子」）。**无 clean gate 可区分驱动案与回归案**。

**根因（postprocess fundamentally flawed for R109）**：block-in-inline-margins-003 属 R109 §9.2.1.1 insert-block-in-inlines 簇（R928-R935 已证 Phase A 结构性）——inline 容器被 block 子分裂为匿名块盒，sibling 拓扑与普通 block 容器不同。postprocess sibling-push 假设的「后续普通流兄弟」在 R109 匿名块盒上下文下语义错误，强行下移破坏布局。**与 R1043「postprocess mirror 无法更新 float-exclusion/margin-collapse 状态，fundamentally flawed」同族裁决**：postprocess 层对涉及 R109/float/margin-collapse 的 sibling 重定位系统性不可解。

**结论**：R1046 ③「sibling overlap = p 文本测量延迟」在 postprocess 层 **ruled out**（与 R1046① R109 匿名块文本高度 / ② R702 margin-collapse-through 同列多 session 结构性）。clean single-session win 谱系再确尽。forward = ① R109 匿名块文本高度须 layout 期 cell_content_height 计入文本（多 session）；② R702 margin-collapse-through 链（intervening sibling 分布）；③ 任何 sibling 重定位须在 layout 期内（taffy/converter 层，非 postprocess）知晓 R109/float/margin-collapse 状态。详见 evidence [`evidence/r1047-sibling-grow-push-reverted-2026-07-05.txt`](./evidence/r1047-sibling-grow-push-reverted-2026-07-05.txt)。

**★ 跨目录近失扫描（为下轮定位，read-only）**：[1.0%,2.5%] 带 flip 候选——css/CSS2/normal-flow：min/max-height-047/058 簇（1.04-1.05%，4 案，mm 单位亚像素疑似 font-wall 谱系，非 clean layout）；block-formatting-contexts-011（1.13%）；block-replaced-height-001（1.14%）。css-flexbox：flexbox-writing-mode-slr/srl-rtl（1.00%，writing-mode 谱系）；flex-inline.html（1.04%）。css-position：position-relative-table-{tbody,tfoot,thead}-{left,top}-absolute-child 簇（1.32%，5+ 案，R1042 table.rs 独立 abspos 机制 entangled 多 session）；position-absolute-center-001（1.14%）。多为 font-wall 亚像素或已知结构性，无 obvious clean 单点。

**▶ 下会话**：clean single-session lever 谱系跨 10+ dir 穷尽（R1042/R1046/R1047 三连再证），forward motion 全 multi-session 结构性——按 R1042「跨会话架构任务」序选 1 dedicated 推进：① **R109 vertical block-flow**（css-writing-modes 87% 失败 dir，~30+ potential，converter `apply_vertical_writing_mode` 接 rl 信号 + taffy 反向布局，R1043 postprocess-mirror 已 ruled out）；② **R702 margin-collapse-through 链**（intervening sibling margin 分布，padding/margin-em-inherit 簇受益）；③ **逻辑属性完整实现**（border/margin/padding/inset inline/block→物理，writing-mode 映射，registry 全未注册 feature gap）；④ font-wall 解除（per-font 真实度量，R887 provider wiring）。每条均多 session，rally 可续跑承接，非人工阻塞。

### R1046 css-tables / CSS2 margin-padding-clear 候选调查 = 全结构性（table-text-height R109 / margin-collapse R702 / sibling-overlap）·零 net 源码·纯调查

承 R1045 CONTINUE（转 CSS2/margin-padding-clear / table-cell-overflow 清洁 worst）。本轮调查 3 个候选，**结论：全结构性，无单点 clean win**。

**① table-cell-overflow-explicit-height-001/002（3.87%，2 case）= R109 非 cap**：测试 `td{height:20px;overflow:hidden}` 含 300px 子。**纠正「cell 应 cap 到 20px」假设**——chromium oracle 实测 td cyan border y=11-338（**grew to fit 300px div**），即 chromium 走 CSS 2.1「cell height=min，cell 增长含内容，overflow:hidden 无 overflow 可裁」路径，**不 cap**。ZW 原 grow 行为正确。实现 cell-height cap 实验（`position_cells` line 1219 overflow!=visible + Px height → cap）实测 css-tables **+0 case flip + divergence 升**（cap 后 ZW td=24 vs CHR td=327，差更大）已回退。真 3.87% 差 = td 高 304（ZW）vs 327（CHR）= **div 后 inline 文本 "Can you see this text?" 高度未计入**（cell_content_height sum children，文本匿名块高度 ~16px 缺，R109 §9.2.1.1 territory）。engine.rs:1150 已有 Mozilla bug 1880550 注释（overflow 映射对 cell 工作），但 cell grow 优先（CSS 2.1）。

**② margin-em-inherit-001（11.25%）= margin-collapse R702（非 em-inherit）**：`#parent{font-size:28px;margin:2em}` → 56px，`#child{font-size:40px;margin:inherit}` → 应继承 computed 56px。LAYOUT_DUMP 实测 **child margin-top=56 ✓（em-inherit 工作正确）**。真 11.25% 差 = body abs_y=56（应 16）= parent margin-top 56 **全折叠上提到 body**（绕过 intervening p 的 margin 分布），margin-collapse-through 链 bug，**R702 territory**（collapse-through + intervening sibling 的 margin 分布，doc 标「类 R680 R109 同族」）。

**③ padding-em-inherit-001（11.17%）= sibling overlap（非 em-inherit）**：`#parent{padding:2em}#child{padding:inherit}`。LAYOUT_DUMP child padding-top=48 ✓（em-inherit 正确）。但 **grand-parent abs_y=16 与 p abs_y=16 重叠**（siblings 同 y）+ body h=244=grand-parent h（**p 的 40px 高度未计入 body content height**）→ render 实测 green box y=16-159（ZW）vs y=72-325（CHR），**ZW green 与 p 文本重叠**。最小复现（body>p+div）**不重叠**→ 非通用 sibling bug，特定于嵌套 img+padding 结构，根因疑似 **p 的文本高度 taffy 测量为 0、post-process 补 h=40 但未重定位 sibling/grand-parent**（R1018/R695 两趟 + sibling 重定位谱系）。

**结论**：css-tables / CSS2 margin-padding-clear worst 全结构性（R109 匿名块文本高度 / R702 margin-collapse / taffy 文本测量延迟致 sibling 错位）。em-inherit + em 单位解析本身**全对**（child 继承 computed px，非 re-resolve）。clean single-session win 谱系确尽（与 R1042/R1045 一致）。forward = ① R109 匿名块文本高度（cell_content_height 缺文本，多 session）；② R702 margin-collapse-through 链（intervening sibling 分布）；③ R1018 两趟 + sibling 重定位（p 文本测量延迟）。

### R1045 text-decoration-thickness 实现实验 = net -2 回退已回退·装饰管线厚度耦合（font-wall 谱系）·零 net 源码·纯调查

承 R1044 CONTINUE（转 css-text-decor / css-floats-clear 清洁 worst）。本轮实现 `text-decoration-thickness` CSS 属性（原完全不支持：parse/store/apply 全缺，paint 用 hardcoded `font_size * 0.06`），**实测 net -2 回退已回退**。

**实现**（已回退）：types.rs `TextDecorationThicknessValue` enum（Auto/FromFont/Length）+ computed_style 字段 + default_impl + apply.rs parse（auto/from-font/length/percentage）+ registry（parse list + property list）+ effects.rs paint_text_decoration_from_style 用显式厚度（length/percentage 相对 font_size 解析，floor + max(1.0)）。

**A/B（css-text-decor Oracle，10GB test-guard per-proc-mem 避 OOM）**：**108→106（net -2）**。
- thickness-length-rounding-001/002/min-val 簇（~13%）**未 flip**（残余 = 装饰 y-offset 管线 `font_size*0.15` underline / `*0.35` line-through 启发式 vs chromium 字体度量，font-wall 谱系，非厚度主导）。
- **2 case 回归**：text-decoration-thickness-overline-001（→7.92%）+ thickness-underline-001（→6.08%）原 PASS 现 FAIL——ZW 装饰渲染（offset/style）**为默认厚度（~1px）校准**，改厚度（显式值）致偏移/视觉与 chr 新不匹配。
- 厚度逻辑正确（2.3px→floor 2，0.3px→floor 0→max 1），但装饰管线未就绪。

**结论**：`text-decoration-thickness` 单点修复**净负**——装饰线渲染管线（y-offset 启发式 + decoration-style 渲染）当前为默认厚度耦合校准，改厚度触发回归。真修须先统一**装饰 y-offset 从字体度量推导**（同 R990 font-metric 谱系，多 session）。**勿再单点补 text-decoration-thickness**。

**副产物记**：① `make reftest-oracle` 默认 test-guard per-proc-mem=6GB，全 corpus 加载（~10k case）边际超限 OOM；用 `./target/test-guard --per-proc-mem 10 -- cargo run --release --bin zero-wpt-runner -- reftest-oracle <dir>` 提至 10GB（合规，仍经 test-guard 包裹）。② css-text-decor worst 全 font-wall 耦合（thickness/offset/style/dilation），单点 lever 净负，转其它 dir。③ 之前会话「min-val flip」A/B 是 stale binary 假象（cargo stash 后未 rebuild）——教训：stash A/B 须确认 cargo 触发 rebuild（`Finished` 行）。

### R1044 ★R850 inline CB-height 链 + inline relative % 修复 LANDED = css-position Oracle 55→57（+2 PASS）·零回归·有 net 源码

承 R1043 CONTINUE（多向调查后转 css-writing-modes near-pass / css-position 清洁 lever）。**R1043 vertical-rl converter-reverse 实测推翻**（taffy Block 不支持 bottom-up packing，反转 children order 仍从 x=0 起，无法实现 rl 方向）→ vertical-rl 方向确证 taffy/architecture-gated（R304 / 重实现）。转 css-position worst 扫描定位 **position-relative-001/002（3.64/4.88%）= R850 percent-inset × R109 inline-split 交互**，**+2 net**。

**根因 A — inline 截断 CB-height 链（position-relative-002）**：`div(red,h:100px) > span(relative,top:100/left:100) > div(green,relative,top:-100%/left:-100%)`。green 是 block-level relative，`top:-100%` 应解析到 CB 高度。CSS §9.2.1.1/§10.1：inline span **不**为 block 后代建立 CB → green 的 CB 跳过 span 继承 red div（100px）。但 R850 walk 用 `style.height==Px` 判 my_content_h，**inline span（auto h）的 None 截断了 CB-height 链** → green 收到 cb_h=None → top:-100% 不解析（green abs_y=151 未覆盖 red@51）。

**根因 B — inline relative top/bottom % 未应用（position-relative-001）**：`div(red) > span(relative,top:100%/left:100%) > div(green,top:-100px)`。span 的 `top:100%` 应解析到 red 100px。taffy 0.7 丢弃 top/bottom %（R715），R850 补 block-level 但**门控 `is_block_level`** → inline span 跳过 → top:100% 不应用（span abs_y 不变，green 落 red 上方 -49）。`left:100%` 工作（taffy 应用水平 %）→ 仅垂直轴坏。

**修复（postprocess.rs::apply_block_relative_percent_insets）**：
- 修复 A：`my_content_h` 分支——block-level 按 `style.height==Px → Some(content_height)`；**inline（is_block_level=false）透传继承到的 cb_h**（inline 不建立 CB for block 后代）。
- 修复 B：应用门控**移除 `is_block_level`**——inline relative 的 top/bottom % 同样补（taffy 对 inline/block 都丢垂直 %）。仅 top/bottom %（垂直轴）；水平 % 仍由 taffy（不 double-count）。

**验证（chromium Oracle + stash A/B 多 dir）**：
- **css-position**：55→57（**+2 PASS**）；position-relative-002 **4.88%→0.73%**，position-relative-001 **3.64%→0.73%**。position-relative-005（4.88% 持平）= JS-driven（`<script>` 设 height），独立。
- **CSS2/positioning**：296→296（R850 原 R711 +10 cluster 全保，零回归）。
- **CSS2/normal-flow**：604→604 / **CSS2/visuren**：25→25 / **CSS2/floats**：117→117（零回归）。
- **product-smoke welcome (DC-13)**：16.57% 不变（< 20% gate PASS）。
- engine 1022/0 + layout-engine 全绿；clippy -D warnings 干净；fmt 干净。

**2 新单测**（intrinsic_two_pass_tests.rs）：`test_r1044_inline_passes_through_cb_height_for_relative_percent`（修复 A：green top:-100% 解析到祖父）+ `test_r1044b_inline_relative_percent_inset_applied`（修复 B：inline span top:100% 应用）。

**意义**：补全 R850 percent-inset 在 inline-split（R109）上下文的两层缺口（CB-height 链 + inline 应用门控）。CSS §9.2.1.1/§10.1「inline 不为 block 后代建立 CB」+「inline relative 也应解析 top/bottom %」原则在 ZW 首次显式接线（R850 原仅处理纯 block 链）。block-level 路径字节同 R850（不可回归）。**css-writing-modes vertical-rl converter-reverse 实测推翻**（taffy Block 不支持 reverse packing）记此，vertical-rl 方向 lever 仍 taffy/architecture-gated（R304）。**position-absolute/fixed-root-element-{flex,grid}（4 case @ 4.46%）= `border:5px dashed` + root abspos inset-sizing**：root inset-sizing 修对（html 770×530）但 dashed border 模式是 CSS implementation-defined（各浏览器 dash length 不同），oracle 永远 ~4.5% 不可 flip，记此避免重试。详见 [`evidence/r1044-r850-inline-pass-through-landed-2026-07-05.txt`](./evidence/r1044-r850-inline-pass-through-landed-2026-07-05.txt)。

**▶ 下会话**：① 继续扫 css-position 残余 worst（position-absolute-semi-replaced-stretch-input/other 21/13% replaced-stretch 簇；position-absolute-in-inline-006 5.1% R109；hypothetical-dynamic-change 4.17% JS）；② 或转 css-writing-modes 近-pass（bidi 簇 ~1.3% / sizing-orthog 1.08% / horizontal-rule-vrl 1.04%）逐案 per-pixel（非 rl 方向细节）；③ R109 vertical-rl 方向仍 multi-session（converter-reverse ruled out，须 taffy 升级 R304 或 layout 期镜像）。

### R1043 R109 vertical block-flow 调查 = 纠正「children 垂直堆叠」误判（block flow 实横向）+ rl/lr 方向 bug + postprocess mirror net-negative 已回退·零 net 源码·转 converter 层

承 R1042 CONTINUE（R109 vertical block-flow dedicated）。本轮纠正上会话误判 + 实验 mirror，**结论：block flow 已横向（正确），真 bug = rl/lr 方向不区分，postprocess 不可解须 converter**。

**★ 纠正上会话「vertical 下 children 仍垂直堆叠」假设（错误）**：最小测试 vertical-rl 容器 + 2 block 子（50×50）LAYOUT_DUMP：a@x=8, b@x=58（**同 y=8，不同 x = 横向并排**）。ZW converter `apply_vertical_writing_mode`（tree.rs:666）轴交换 + engine.rs:1232 un-swap 使 vertical block 流**已横向**（正确）。上会话假设错误。

**★ 真因 = rl/lr 方向不区分**：vertical-lr 测试同 a@x=8 b@x=58（identical to vertical-rl）。vertical-rl 应 a 在右（block 流右→左），vertical-lr 应 a 在左（左→右）。ZW 对 rl/lr **同样处理**（都左→右）。`apply_vertical_writing_mode`（converter/mod.rs:232）对 rl/lr 同样 swap（Column→Row），不镜像。inline_finalization 有 is_vertical_rtl（line 671/940/1099/1221）处理 inline 方向，但 **block 流方向未镜像**。

**postprocess mirror 实验（net-negative 已回退）**：实现 `mirror_vertical_rl_block_children`（postprocess.rs，VerticalRl 容器 in-flow block 子 `x = content_w - x - width`）。**A/B v1 net -1**（2 flip: caption-side-vrl-002/float-vrl-006；3 regress: float-clear-vrl-006/008 + margin-collapse-vrl-034）。v2（排除 float）**net -2**（丢 float-vrl flip，float-clear 仍回归）。**根因**：postprocess 镜像无法更新 float exclusion / margin-collapse 状态——clear/collapse 须在 layout 期内知晓方向。postprocess fundamentally flawed。

**结论**：rl/lr 方向修复须 **converter/taffy 层**（让 taffy 知道 rl 反向，float-clear/margin-collapse 自然正确），非 postprocess 镜像。**R109 vertical block-flow 确证 multi-session**（converter `apply_vertical_writing_mode` 须传 rl 信号 + taffy 反向布局，或 layout 期镜像）。

**未解**：vertical-rl 多 block 容器方向错（首子在左应右）。单 block 容器无影响（inline 方向 rl/lr 同）→ 近-pass 多 1-3% 残余非此 bug 主导（其它 writing-mode 细节：text baseline / glyph rotation / logical props）。

**▶ 下会话**：① **R109 vertical-rl converter 层**（多 session——apply_vertical_writing_mode 接收 rl 信号，taffy 反向布局或 layout 期内镜像使 float-clear/margin-collapse 正确，首 slice gate 紧到纯 block 无 float/margin）；② **css-writing-modes 近-pass 残余**多非 rl 方向（text baseline / glyph / logical props），逐案 per-pixel；③ 或转 position:relative converter（+12 entangled）/ font-wall。R109 postprocess 已 ruled out。

### R1042 position:relative + css-tables + css-writing-modes 三 dir 调查 = 全 multi-session 结构性·系统 quick-win 已尽·零源码·纯调查

承 R1041 CONTINUE（转 position:relative converter）。本轮调查三方向，**结论：系统 quick-win 已尽，残余全 multi-session 结构性**。

**① position:relative converter-layer（+12 potential）**：找到 table.rs 独立机制——table.rs:1108/1128-1129（row）、1178（cell）、1553（row_group）对 table-internal relative 元素**直接**应用 relative inset 到 x/y（`row_box.x = row_rel_dx`），独立于 taffy。R1020-cont postprocess `propagate_relative_cb_offset_to_abspos` 对此 double-apply。**但 R1020-cont A/B 0 improved**（即使 div 测 spec-correct y=100 也未更好匹配 chr）→ 不止 table double-apply，div 路径 position-relative-004 亦回归。**真统一须 converter/taffy 层**（div 用 taffy abspos 解 pre-inset CB；table 用 table.rs）——多 session entangled（R98/R123/R500 谱系）。**defer**。

**② css-tables 近-pass（74/115，25 近-pass）**：subpixel-collapsed-borders（subpixel 精度）、colspan-004（R177 结构）、border-collapse-empty-cell（R1026 font-wall）、th-text-align（ZW 有 th→center UA hint line 471/732，非缺失）、row-margin-border-padding（R1026 双层归零已做）。**CSS 默认值全对**（border_collapse:Separate / empty_cells:Show / table_layout:Auto / caption_side:Top / vertical_align:Baseline）——无 R1040 同款默认值 bug。无 obvious 系统布局修，近-pass 多为精度/结构/font-wall。

**③ css-writing-modes 近-pass（56/784 = 7%，376 近-pass）**：250 vertical（R109 blocked）+ 126 非垂直但 writing-mode-dependent（logical-props border/margin/padding/inset 逻辑属性 + sizing-orthog + baseline-orthogonal）。**逻辑属性（border-inline-start 等）registry 完全未注册**（feature gap），但映射依赖 writing-mode（horizontal-tb 简单 1:1，vertical-rl/lr 复杂依赖 text-orientation），WPT corpus 多用 vertical 测映射 → 非 single-session slice。converter 仅有 `apply_vertical_writing_mode`（swap width/height for logical SIZE，line 232-277）。

**战略结论**：跨 multicol（done +16）/ position:relative / css-tables / css-writing-modes，**系统 quick-win（per-pixel → 默认值/spec 修）已尽**。残余 forward motion 全 multi-session 结构性：
1. **R109 vertical block-flow**（css-writing-modes 87% dir，~30+ potential，大重构）
2. **逻辑属性完整实现**（border/margin/padding/inset inline/block → 物理，依赖 writing-mode 映射，css-writing-modes logical-props 簇 + 多 dir 受益）
3. **position:relative converter 统一**（+12，div+table 双 abspos 路径，entangled）
4. **font-wall 解除**（pervasive text glyph，Ahem subpixel + 真实字体光栅，R1034 结论）

**▶ 下会话**：选 1 个 dedicated multi-session 推进：① **R109 vertical block-flow**（最高 yield ~30+，css-writing-modes 7%→大幅提升；首做 vertical block-flow 方向实现，children 在 vertical mode 横向堆叠）；② **逻辑属性完整实现**（logical-props 簇 + 跨 dir，writing-mode 映射表）；③ position:relative converter（+12 entangled）；④ font-wall。**单 session 系统快赢已尽，须 committed 多 session**。

### R1041 multicol 近-pass 残余调查 = 渲染精度（subpixel AA）非 layout·arc 已尽·零源码·转 position:relative

承 R1040 CONTINUE（multicol layout 已尽，扫近-pass 或转方向）。本轮调查 css-multicol 180 个近-pass 案（1.0-3.0%）残余成因，**结论：渲染精度非 layout，arc 已尽**。

**180 近-pass 案分类**：采样结构分类——① 基础 multicol（multicol-rule-003/height-001/gap-large-002 等）；② 不支持特性（column-height-012 用 column-height、column-height-020 用 column-wrap:wrap——multicol-2 草案 ZW 未支持）；③ font-wall text。

**Per-pixel 验证（2 案确认渲染精度）**：
- **multicol-rule-003**（1.13%，column-count:4 + column-gap:1em + column-rule:1em + `font:1.25em/1 Ahem`）：rule_x 公式验算正确（col0[0,60] gap[60,80] rule 居中 ✓），残余在 border/rule 边缘 AA + Ahem 方块 subpixel 定位（颜色匹配 ZW=CHR blue/gray，diff 在边缘）。
- **multicol-height-001**（1.13%，column-fill:auto + height:8em + `font:1.25em/1 Ahem`）：y 带残余 ~80px/带 baseline（全白匹配），y[16-32] 顶缘 1594px——Ahem 方块 subpixel AA（ZW 整数方块 vs CHR Ahem.ttf AA 边）。

**结论**：multicol 近-pass 残余 = subpixel AA（Ahem 边 + rule/border 边），非 layout bug。R1035-R1040 五轮已尽 big systematic layout wins（+16）。进一步 multicol 需渲染精度/font-wall（多 session，R1034 结论）。

**★ position:relative converter-layer 评估（下会话 lever）**：R1020-cont 证 postprocess 路径失败（table path 有独立 abspos 机制，postprocess double-apply；div path baseline abs@50 错应 100）。converter-layer 是未试路径，须统一两条 abspos 路径（div + table），entangled（R98/R123/R500 谱系）。+12 potential（position:relative-table 簇 6 + text-decoration-thickness 簇 6）。多 session。

**▶ 下会话**：① **position:relative converter-layer**（+12 potential，下会话 dedicated——先 LAYOUT_DUMP 固化 div vs table 双路径 abspos 机制差异，设计 converter 统一，首 slice gate 紧到非 table）；② 或 **R109 vertical block-flow**（css-writing-modes 87% dir，大 yield 大重构多 session）；③ font-wall 解除（pervasive，Ahem subpixel + 真实字体光栅，多 session）。multicol arc +16 done，转方向。

### R1040 ★column-gap:normal 默认值修复 LANDED = css-multicol Oracle 140→146（+6 PASS）·per-pixel x 带定位·累计 css-multicol +16

承 R1039 CONTINUE（002 残余 4.49%）。本轮 **per-pixel x 带对比**定位 column-gap 默认值 bug，**+6 net**。

**per-pixel x 带定位**：002 post-R1039 y 带 diff 颜色匹配（ZW=CHR），转 x 带分析 y=55-70（block1 bg）。x[200-216]：**ZW=(255,255,0) yellow（block1）vs CHR=(144,238,144) lightgreen（article bg）**。真因：002 未指定 column-gap，CSS Multicol §4.1 初始值 = `normal`（=1em≈16px）。CHR col宽=(400-16)/2=192 + gap[205:221] 显 article bg。**ZW `default_impl.rs:129 column_gap: LengthValue::Px(0.0)` 错误**（应 normal）→ col宽=200 gap=0，block1 覆盖 gap 区。

**探针确认 yield**：强制 multicol gap=1em（所有案）net -25（6 flip/31 regress）——证实 6 案未设 column-gap 想要 1em，proper fix（仅未设案 → 1em）预期 +6。

**实现（LengthValue::Auto 作 normal sentinel）**：gap 不接受 auto，故 Auto 作 normal 专用 sentinel（无冲突）。① default_impl column_gap: Auto（原 Px(0.0)）；② multicol compute_column_info `if Auto → font_size_px(1em) else length_to_px`（显式 column-gap:0 的 Px(0.0) 与 Auto 区分清晰）；③ converter convert_length_to_lp(Auto) 已返 0——flex/grid normal=0 无需改。**显式 column-gap:0 与 normal(Auto) 不冲突**。

**A/B（stash 对照 R1039 140/452）**：**146/452（+6 net）**。**5 flip**（multicol-clip-scrolled-content-001 1.08→0.88 ★修 R1039 回归 + multicol-fill-auto-block-children-003 + multicol-rule-nested-balancing-001/003 + nested-floated-multicol-with-monolithic-child）。**0 回归**。002 4.49→3.99（残余 = text glyph font-wall，layout 全对）。flex/grid 无回归（css-flexbox 295、css-grid 20 同 baseline）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（style-system column_gap 测全过 + layout-engine 1021）/ **product-smoke welcome 16.57% < 20%** ✓。21 multicol 测全过。

**意义**：CSS Multicol §4.1 column-gap 初始值 normal（=1em）正确实现（default_impl Auto sentinel + multicol 解析 normal=1em + flex/grid 经 convert_length_to_lp(Auto) 保 normal=0）。per-pixel x 带对比定位。**累计 R1035(+1)+R1037(+4)+R1038(+1)+R1039(+4)+R1040(+6) = css-multicol 130→146（+16）五连 landed win**。002 layout 全对（残余 3.99% font-wall）。详见 [`evidence/r1040-column-gap-normal-default-landed-2026-07-05.txt`](./evidence/r1040-column-gap-normal-default-landed-2026-07-05.txt)。

**▶ 下会话**：① **002 残余 3.99% = font-wall**（layout 全对，R1034 font-wall 结论）——multicol 簇 002/004a/004b/006/003 layout 已尽，残余全 font-wall text glyph，须 font-wall 解除（多 session，per R1034）；② multicol 其它 dir fresh worst 扫（column-gap 修复后可能新近-pass 案）；③ **转 R109 vertical / position:relative / font-wall 换方向**（multicol 五连 +16 后 layout 侧近尽，font-wall 是下个 pervasive blocker）。column-gap:normal 对 row-gap（flex/grid）的 normal 默认 row_gap 仍 Px(0.0) 但 flex/grid normal=0 已正确，无 yield 缺口。

### R1039 ★multicol column fragment slice clip LANDED = css-multicol Oracle 136→140（+4 PASS）·per-pixel 定位 paint clip 根因·修 002 block1 覆盖 spanner·累计 css-multicol +10

承 R1038 CONTINUE（目标簇 002 残余 8.56%）。本轮 **per-pixel y 带对比定位** paint clip 系统根因，**+4 net**。

**per-pixel 定位**：product-smoke 渲 002 → PIL per-10px y 带对比 chr oracle。y[110-160]（spanner 区）**ZW=(255,255,0) 黄 vs CHR=(173,216,230) lightblue**（3900 px/band 大 diff）。真因：block1（R1037 balance-breaking 跨 2 列各 100px）paint `clip_h = box_node.content_height`（容器 200）未裁到 slice 100px，block1 全 200px 渲染覆盖 spanner 区。前轮 paint clip 探针（clip_h+1000）证伪是因为方向错（应**收紧**到 slice 非放宽）。

**实现（column_span_offsets 4-tuple → 6-tuple + paint slice ∩ container clip）**：① types/mod.rs 扩存 `(child_x, child_y, col_x, col_w, col_top, col_h)`，col_top=y_offset（片段列顶），col_h=visual_height（slice 高）；② multicol.rs push 扩展；③ paint mod.rs:840 breaking 片段 clip 从 `(content_y, container_h)` 改 `(content_y+col_top, col_h ∩ container)`——`col_top>=container_h` 跳过（multi-row overflow row，chromium 裁剪 multicol 列溢出），`effective_h=col_h.min(container_h-col_top)`。

**★ clip 演进**：v1（slice 仅）139/452(+3) 但 003 17.88→**31.12(+13pp)**——block2 overflow row 被显示（chromium 裁剪）；v2（slice ∩ container，overflow row 跳过）**140/452(+4)** + 002 8.56→**4.49** + 003 恢复 17.92 ✓；v3（+1px tolerance）同 v2，tolerance 无效（回归结构性）→ 回 v2。

**A/B（stash 对照 R1038 136/452）**：**140/452（+4 net）**。**7 flip**（multicol-contained-absolute 8.50→0.33 大改善 + change-fragmentainer-size/column-wrap-no-constraints/fill-balance-018/spanner-fragmentation-012/span-all-020/column-height-017）。**3 borderline 回归**（column-height-020 0.98→1.13、nested-023 0.99→1.24、spanner-fragmentation-004 0.73→1.33，结构性）。**002 驱动案 8.56→4.49（接近 flip）**，007 6.85→6.39。004a/004b/006 略恶化（仍 FAIL，slice clip 对 nested 结构稍紧）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0** / **product-smoke welcome 16.57% < 20%** ✓。21 multicol 测全过。

**意义**：per-pixel y 带对比定位 paint clip 系统根因（breaking 片段未裁到 slice），column_span_offsets 扩存 + slice ∩ container clip 修复。002 驱动案 8.56→4.49 接近 flip。**累计 R1035(+1)+R1037(+4)+R1038(+1)+R1039(+4) = css-multicol 130→140（+10）四连 landed win**，彻底打破 R1030-R1034 五轮 plateau。详见 [`evidence/r1039-fragment-slice-clip-landed-2026-07-05.txt`](./evidence/r1039-fragment-slice-clip-landed-2026-07-05.txt)。

**▶ 下会话**：① **002 残余 4.49% 接近 flip** + 004a/004b/006/003 簇残余——slice clip 对 nested 结构稍紧，逐案 LAYOUT_DUMP 查 col_top/col_h 精度 + 对比 ref 像素带；② 3 borderline 回归（column-height-020/nested-023/spanner-fragmentation-004）查是否可微调；③ 或转 R109 vertical / position:relative 换方向（multicol 四连 +10 后可换）。

### R1038 ★spanner balance-breaking region_idx gate LANDED = css-multicol Oracle 135→136（+1 PASS）·修 R1037 no-balancing 回归·累计 css-multicol +6

承 R1037 CONTINUE（目标簇残余深挖 + 2 小回归）。本轮先探目标簇 002 残余（paint clip 放大探针证伪——clip 正确，002 8.56→18.98 更差），转修 R1037 确定回归 no-balancing。

**002 残余调查（探针证伪）**：LAYOUT_DUMP 确认 block1 现 split（spanner abs_y 213→113，region0=100px ✓）。假设 multi-row overflow row 被 paint clip 裁剪——`mod.rs:849` breaking fragment `clip_h=box_node.content_height`。**探针放大 clip_h+1000 → 002 8.56→18.98（更差！）证伪**：clip 正确（隐藏溢出内容），残余**不在** paint clip，在 multi-row+breaking 协同精度（region_available/fragment_y_offset/row_height 交互，多 session）。

**★ 修 R1037 no-balancing-after-column-span 回归**：结构 = column-fill:**auto** + column-span:all + 内容在 spanner 后。assert：auto 模式 spanner **之后**不应 balance。R1037 spanner 路径 region_explicit 对 auto 模式也触发 balance-breaking（误）。**修**：region_explicit gate 加 `|| region_idx == 0`——region 0（spanner 前）保留 breaking（always-balancing-before-column-span 案），region>0（spanner 后）+ auto 禁 breaking。

**Gate 演进**：v1（`!sequential_fill`）136/452(+1) 但 always-balancing-before 2.81→3.84(+1.03 仍 FAIL，过宽禁了 region 0 breaking) → v2（`!sequential_fill || region_idx==0`）136/452(+1) + always-balancing 不再恶化。chromium auto+spanner 语义：spanner 前 balance（region 0）/ 后 sequential（region>0）。

**A/B（stash 对照 R1037 135/452）**：**136/452（+1 net）**。**1 flip**：no-balancing-after-column-span 1.77→0.73（修 R1037 回归）。噪声 column-height-029 +0.05（仍 FAIL）。6 flip + 目标簇 + always-balancing 全保持。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0** / **product-smoke welcome 16.57% < 20%** ✓。21 multicol 测全过。

**意义**：修 R1037 no-balancing 回归，css-multicol 135→136。gate 精细化（region_idx 区分 spanner 前后）匹配 chromium auto+spanner 语义。**累计 R1035(+1)+R1037(+4)+R1038(+1) = css-multicol 130→136（+6）**，三连 landed win。详见 [`evidence/r1038-no-balancing-after-span-fix-2026-07-05.txt`](./evidence/r1038-no-balancing-after-span-fix-2026-07-05.txt)。

**▶ 下会话**：① **目标簇 002/007/013 残余**——paint clip 探针证伪，残余在 multi-row+breaking 协同精度，须 LAYOUT_DUMP 逐 fragment 查位置 + ref PNG 像素带对比定位（多 session，但 007 6.85%/013 1.23% 接近 flip，EV 高）；② clip-scrolled-content-001（R1037 残余 +0.21 小回归）；③ 或转 R109 vertical / position:relative 其它结构性（multicol 三连 +6 后可换方向）。

### R1037 ★balance-mode column-breaking + explicit-height gate LANDED = css-multicol Oracle 131→135（+4 PASS）·目标簇 span-all-children-height 大改善·2 小回归·纠正 R1036「avoid 前置」误判

承 R1036（通用 balance-breaking net -12 回退）。本轮找到正确 gate，转 **net +4**。

**R1036 误判纠正**：回归案 break-inside:avoid 全 0（含 overflow-unsplittable）→ avoid 非前置。读结构：overflow-unsplittable-001 = `overflow:scroll + height:auto`（monolithic 滚动容器）；span-all-children-height-004a = `height:200px`（explicit length）。**真区分器 = explicit height**（CSS Fragmentation monolithic 元素 overflow≠visible/scroll/auto-height 不可分）。

**实现（crates/layout-engine/src/multicol.rs）**：① `is_explicit_height(style)` helper（height 非 Auto/Calc/FitContent/MinContent/MaxContent）；② `assign_children_to_columns_balanced` 加 `explicit_height: &[bool]` 5th 参数，R1036 breaking 分支 gate on `is_explicit && child > target && target > 0`；③ 两 caller（非 spanner + spanner）从 styles 算 explicit_height；④ **zero-height 守卫**：`container.content_height > 0.0` 才传非空（避 zero-height 容器误触）；⑤ 8 既有测试加 `&[]`，2 新 R1037 单测（explicit split / auto no-break）。

**A/B（stash 严格对照，baseline 131/452）**：v1（explicit-height gate）134/452（+3，仍含 zero-height-002 0.91→6.42 强回归）→ v2（+ zero-height 守卫）**135/452（+4）**。**6 flip**（column-height-020/fill-balance-005/030/nested-023/031/spanner-fragmentation-005）。**2 小回归**（clip-scrolled-content 0.93→1.14 +0.21pp、no-balancing-after-column-span 0.73→1.77 +1.04pp 语义）。explicit-height gate 消 R1036 的 15 个回归 + zero-height 守卫再消 zero-height-002 = R1036 18 回归 → R1037 仅 2 小回归。

**★ 目标簇 span-all-children-height 改善（R1035 multirow + R1037 breaking 协同）**：002 **26.76→8.56（-18pp）**、007 16.28→6.85（-9.4pp 接近 flip）、004a 30→18、004b 28→19、003 23→18、006 22→15、013 1.91→1.23（接近）、001 0.72→0.40（PASS 改善）。多数仍 FAIL（残余 = multi-row+breaking 协同精度 + region 高度交互）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（workspace 零失败，layout-engine 1021 测 +2 R1037）/ **product-smoke welcome 16.57% < 20% gate** ✓。21 multicol 测全过（2 新 R1037）。

**意义**：R1036 通用 breaking + explicit-height gate（CSS Fragmentation monolithic gate）+ zero-height 守卫，转 net -12 → **net +4**。继 R1035（+1）后第二个 landed multicol win，**累计 R1035+R1037 = css-multicol 130→135（+5）**。纠正 R1036「break-inside:avoid 是前置」误判（回归案 avoid 全 0，真前置 = monolithic/explicit-height gate）。详见 [`evidence/r1037-balance-breaking-explicit-gate-landed-2026-07-05.txt`](./evidence/r1037-balance-breaking-explicit-gate-landed-2026-07-05.txt)。

**▶ 下会话**：① **目标簇残余深挖**——002/007/013 接近 flip（<2%），查 multi-row+breaking 协同精度（002 region leftover 与 breaking 片段边界交互）+ region 高度计算；② 2 小回归（clip-scrolled-content 滚动容器 / no-balancing-after-span 语义）逐案查；③ 或转 R109 vertical / position:relative 其它结构性。css-multicol 累计 +5，仍可继续挖（目标簇接近 flip）。

### R1036 balance-mode column-breaking 通用应用 net -12 已回退·目标簇大改善但 18 回归·真前置 = break-inside:avoid plumbing·零 net 源码·纯调查

承 R1035 CONTINUE（span-all-children-height-002 真前置 = balance-mode column-breaking，LAYOUT_DUMP 证 block1 200px 应拆 2 列各 100px 让 region0=100px 留 room 给 region1 multirow）。本轮实现 + A/B，**net -12 回退**。

**实现（已回退）**：`assign_children_to_columns_balanced` 加 oversized 分支——child > target_height 时按 target 边界跨列拆分（复用 fragment_y_offset/visual_height）。3 单测 + 19 multicol 测全过。

**A/B（stash 严格对照）**：css-multicol **131→119（-12 net）**。**6 flip**（column-height-020/fill-balance-005/030/nested-023/031/spanner-fragmentation-005）。**18 regress**（overflow-unsplittable-001/002/003 强回归 0.16-0.92→2.08-2.65 = **break-inside:avoid 未尊重**、span-all-dynamic-add 6+ 案、span-all-006/007/008、no-balancing-after-column-span、multicol-zero-height-002 0.91→6.42）。**目标簇大改善（仍 FAIL）**：span-all-children-height-002 **26.76→8.56（-18pp 驱动案）**、004a 30→18、span-all-rule-001 17→1.25、rule-nested-balancing-003 20→4。

**★ gate 探索均失败**：① percentage-height gate——目标案 002 有 height:% 但同簇 004a/006/007 多无 percentage，非通用签名；② spanner-only gate——18 回归中 ~12 是 spanner 案（span-all-dynamic-add/006/007/008/no-balancing-after-column-span），spanner-only 不能救回。无 clean gate 区分目标 vs 回归。

**★ 真前置（多 session）**：① **break-inside:avoid plumbing**——overflow-unsplittable 簇强回归明示须尊重 break-inside:avoid（ZW 现 break-inside layout 全无消费，R1027 标死值仅 paint 指示器）；balanced assign 须接收 break-inside 信号，oversized avoid 子不拆。② spanner dynamic 案（span-all-dynamic-add 6+ + span-all-006/007/008）逐案查结构。③ zero-height / target_height=0 守卫。

**意义**：通用 balance-breaking net -12（6 flip/18 regress）。目标簇 + 6 flip 证机制正确，但 break-inside:avoid 未尊重 + spanner dynamic + edge 致回归。**真解锁 = break-inside:avoid plumbing（独立多 session slice）+ balanced enable_breaking flag**。R1035 multirow 基础设施已 land，balance-breaking 是其天然放大器但须先解 break-inside:avoid。详见 [`evidence/r1036-balance-breaking-net-negative-2026-07-05.txt`](./evidence/r1036-balance-breaking-net-negative-2026-07-05.txt)。

**▶ 下会话**：① **break-inside:avoid layout plumbing**（独立多 session——break_inside 现 layout 全无消费，须传入 balanced assign + with_breaking，overflow-unsplittable 簇 + dynamic-change-inside-break-inside-avoid 受益，是 R1036 balance-breaking 的前置 + 独立 spec-correctness lever）；② 解 break-inside:avoid 后重试 balance-breaking（带 enable_breaking flag + avoid 守卫，预期 net 正——目标簇 + 6 flip 保，overflow-unsplittable 等 avoid 回归消失）；③ 或转 R109 vertical / position:relative 其它结构性。**勿以「通用 balance-breaking 无 gate」单 session 重试**（已证 net -12）。

### R1035 ★multicol spanner 路径 multi-row 列模型 LANDED = css-multicol Oracle 130→131（+1 PASS）+ 2 大改善·零回归·plateau 五轮后首个 landed code win

承 R1034 CONTINUE（multicol multi-row 须在 spanner 路径做，R1034 非 spanner 实验 net-negative 回退）。本轮精准在 `layout_multicol_with_spanners` 加 per-region overflow → multi-row，**clean win**。

**驱动案（multicol-span-all-children-height-002）**：article column-count:2 height:200px（definite，balance）+ spanner(height:25%) + block2(height:100%=200px)。region1（spanner 下方）仅剩 50px leftover，block2(200px) overflow → 应 4 列=2 行×2 列各 50px。缺失机制 = `region_available = container_height − y_base`（每区域 leftover 高度）。

**实现（crates/layout-engine/src/multicol.rs）**：① 新增 `assign_children_to_columns_multirow`（overflow 换行非截断末列，含跨行 breaking）；② `position_multicol_children` 加 `row_height: f32` 参数（row=col_idx/col_count, y=y_base+row×row_height；非 spanner 调用者传 0.0 不变）；③ `layout_multicol_with_spanners` 区域循环加 multirow gate：`region_available=(content_height-y_base).max(0)`，`use_multirow = !empty && !sequential_fill && region_available>0 && total>col_count×region_available+1 && !has_nested_multicol`。

**★ Gate 演进（关键）**：v1（无 `!sequential_fill`）A/B +1 但 fill-auto-block-children-002 +0.12pp 回归（auto+spanner 下 multirow 语义偏差）；v2（加 balance-only gate）+1 保持 + 回归消除；`!has_nested_multicol` 守卫避 R1034 multicol-nested-019 谱系。

**验证（chromium Oracle + stash 严格 A/B）**：css-multicol **130→131（+1 net）**。**1 flip**：multicol-span-all-012 1.65→**0.88% PASS**。**2 大改善（仍 FAIL）**：span-all-children-height-003 34.08→23.59%（-10.49pp）、-007 19.78→16.28%（-3.50pp）。**0 回归**。3 新 R1035 multirow 单测 + 19 multicol 测全过。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（workspace 零失败）/ **product-smoke welcome 16.57% < 20% gate** ✓（welcome 无 multicol 零影响）。

**未解（下会话 lever）**：① **span-all-children-height-002（驱动案）26.76% 未变**——LAYOUT_DUMP 实测证 **block2 h=200px（percentage 解析正确）**，真前置 = **balance 模式须对超 target 的单 child 做 column-breaking**：block1(200px) balance 应拆 2 列各 100px（region0=100px），但 ZW balanced 把单超 target child 整体留 col0（region0=200px 占满），y_base 推进到 250 → region_available=200−250<0 → multirow gate 不 fire。**R1035 旧「percentage-height 失败」hypothesis 已 dump 推翻**。修 balance-mode column-breaking（单 child>target 时跨列拆分，复用 with_breaking fragment 机制）可望翻 002 + 004a/004b/006 = 独立高价值 lever，是 R1035 multirow 的天然放大器。② 003/007 改善但残余（同 balance-breaking 精度 + column-rule 交互）。③ auto+spanner sequential row-fill（fill-auto-block-children 簇）多 session。

**意义**：R1029 后、R1030-R1034 五轮 plateau 后首个 landed code win。证实「spanner 路径 + 紧 gate（balance-only + definite 高 + 无 nested multicol）」是 multicol multi-row 可产方向（纠正 R1034「multi-row 全 ruled out」——仅非 spanner ruled out，spanner 路径可产）。span-all-children-height 簇首次实质进展。详见 [`evidence/r1035-spanner-multirow-landed-2026-07-05.txt`](./evidence/r1035-spanner-multirow-landed-2026-07-05.txt)。

**▶ 下会话**：① **balance-mode column-breaking**（`assign_children_to_columns_balanced` 单 child 超 target 时跨列拆分，复用 with_breaking fragment；是 002 + 004a/004b/006 的真前置 + R1035 multirow 天然放大器；风险=影响全 balanced multicol，须全 css-multicol A/B 守回归）；② 003/007 残余深挖；③ auto+spanner sequential row-fill（独立多 session）。**勿以「percentage-height-against-multicol」为名重试**（已 dump 证 block2 h=200 正确）。

### R1034 multicol non-spanner multi-row 实验 net-negative 已回退 + font-wall subpixel x 角度分析（fresh 但 WPT yield 低）·零 net 源码·纯调查

承 R1033 CONTINUE（plateau 四轮确证，须 committed 多 session，#1 = font-wall fresh angle = subpixel positioning）。本轮先分析 font-wall subpixel，再实验 multicol multi-row，**均无 yield**（前者分析低 yield，后者 A/B net-negative 回退）。

**① font-wall subpixel x 角度 = fresh 但 WPT yield 低（纯调查）**：CODE 调查证实 ZW 用 `font.rasterize(char, size)`（loader.rs:217，整数位置）无 `rasterize_subpixel`；glyph.x **已是 float 累积**（text.rs:496-510 `char_x += measure_char_for_paint(...)`，paint 期 cpu/mod.rs:457 `gx.round()` 才舍入）→ **无漂移累积**，仅每字 ±0.5px 量化。R834/R836/R849 全是**垂直侧**（baseline/strut/v_offset，已 R953/R990 解决），水平 subpixel x **确实未试**=fresh。**但**：WPT text dir 主体是 Ahem，而 Ahem 被 `rasterize_ahem_glyph`（loader.rs:208-274）特殊化为**完美整数方块**（size.ceil()×size.ceil() 全 255）→ subpixel x 对 Ahem corpus **几乎零收益**；非 Ahem 产品页或受益但 LCD vs 灰度 AA 使 oracle 匹配复杂。**结论**：subpixel x 即使技术成功也基本不动 WPT 通过率，R1033「font-wall 解 text-rendering 全 dir」对 Ahem corpus 不成立。列为低优先（DC-13 产品页价值，非 WPT lever）。

**② multicol non-spanner multi-row 实验（net-negative 已回退）**：转 R1033 #2（span-all-children-height 簇 15-34% 最高 yield）。实现：`assign_children_to_columns_multirow`（overflow 换行非截断末列）+ `position_multicol_children` 加 `row_height` 参数（spanner 路径传 0.0 不变）+ `layout_multicol` gate（sequential_fill && height_limit>0 && total>col_count×height_limit）。3 单测全过 + layout-engine 1019 测零回归。**A/B css-multicol oracle（stash 严格对照）**：目录 130/452 持平；per-case **2 regressed**（nested-past-fragmentation-line 1.77→2.80%、multicol-nested-019 1.76→2.79%，各 +1.03pp）+ **2 improved 无 flip**（spanner-fragmentation-000/002 0.83→0.73%，本就 PASS，噪声级）= **net-NEGATIVE** → 按代码准则回退。

**★ 回归根因（multicol-nested-019 实查）**：外层 columns:2 column-fill:auto height:100px + 内嵌 multicol + orphans/widows/break-before:avoid + 100px green div overflow = **nested multicol + fragmentation** 复杂交互。naive row-wrap（overflow 简单换 row 2）对此**错误**——旧 clip-to-last-column 反而更近正确（chromium nested fragmentation 有 orphans/widows/avoid 规则，非简单 row-wrap）。**无 clean gate 可区分「简单 overflow→multi-row」vs「nested fragmentation→clip」**，须完整 fragmentation 模型。

**★ 真结论**：① 非 spanner multi-row net-negative（触发案全 nested/fragmentation）；② **真 multi-row yield 须 spanner 路径**（span-all-children-height 有 spanner，走 `layout_multicol_with_spanners` 本轮未改）+ 正确 fragmentation 交互（多 session 硬核）；③ font-wall subpixel x fresh 但 WPT yield 低（Ahem 特化）。plateau 五轮确证（R1030-R1034）。详见 [`evidence/r1034-multicol-multirow-fontwall-subpixel-2026-07-05.txt`](./evidence/r1034-multicol-multirow-fontwall-subpixel-2026-07-05.txt)。

**▶ 下会话**：① **multicol multi-row 在 spanner 路径**（`layout_multicol_with_spanners` per-region overflow → 额外列，是 span-all-children-height 簇 15-34% 的真解锁路径，本轮已证须 spanner 侧 + fragmentation 配合）；② R109 vertical block-flow（writing-modes 87% dir，大 yield 大重构）；③ position:relative converter-layer 统一（R1020-cont 证 postprocess 不可解，+12 potential）；④ font-wall 在产品页 DC-13（subpixel x fresh，非 WPT lever）。**勿以「非 spanner multi-row」或「font-wall subpixel x 对 WPT」为名单 session 重试**（已证）。

### R1033 css-text-decor multi-value + position:relative-table 复核 = 全 font-wall/structural 阻塞·plateau 综合再确证·零源码·纯调查

承 R1032 CONTINUE。本轮复核两个候选，均确证 blocked：

**css-text-decor text-decoration-line multi-value（63 文件 grep，R724 标 single-value enum）**：text-decoration-line.html（11%）测 multi-line + cascade/!important/blink（entangled）。near-pass 多值案（text-decoration-style-multiple 0.96%、shorthands 0.61-1.79%、underline-offset-overline 1.02%）**全 font-wall 阻**——装饰线 y 位置跟随 baseline（fontdue vs chromium baseline 度量），multi-line 支持只改「画几条线」不改线位置 → 实现后仍 +0（同 thickness R875/R914/R1020-cont 3× net-negative 谱系）。enum→bitset 重构（~30 match sites）+ paint multi-line = 中 effort，**yield +0 不 justified**。裁决：不实现（font-wall 阻 flip）。

**css-position position-relative-table 簇（8 案 ~1.32%，R1020-cont target）**：apply_relative_offsets（postprocess.rs:994）已递归应用到 table-internal 盒（含 tbody/tr/td），relative offset MOSTLY 生效（1.32% 残余非 offset 全缺）。R1020-cont global postprocess fix overshoot（table.rs 已接近正确），1.32% 残余 = table 路径 abspos/relative 定位精度，**structurally entangled**（须 converter/taffy 层统一非 postprocess，多 session）。再确证 R1020-cont 结论。

**plateau 综合再确证（R1030-R1033 四轮）**：跨 multicol / css-text-decor / css-fonts / css-values / css-writing-modes / css-position / css-tables / css-flexbox / css-grid **全 WPT reftest dir**，单 session clean lever 已尽。残余 forward motion 全多 session 结构性：
- **font-wall**（text-rendering 全 dir 主体 blocker）：R953 baseline_offset（+102）+ R990 ascent 0.928（+138）已尽 metric/positioning 侧；残余 = glyph raster accumulation（R388 单 glyph ≈chr 但累积发散）+ advance width（R375b accurate net-negative）= 硬核多 session（fontdue→chromium 光栅协调或 FreeType 替换）。
- **R109 vertical block-flow**（css-writing-modes 87% + text-emphasis-position vertical）：结构性多 session。
- **multicol Phase 2 multi-row**（span-all-children-height + span-all-rule + nested-balancing）：结构性多 session。
- **position:relative offset**（structurally entangled，R1020-cont 证 postprocess 不可解）：多 session。
- **feature gaps**（text-decoration-inset/thickness CSS4 + 表单控件 + ::backdrop + scroll-container）。

**▶ 下会话**：plateau 已四轮确证，**勿再扫找 clean lever**（边际为零）。须 committed dedicated 多 session：① **font-wall glyph raster accumulation**（最高价值，解 text-rendering 全 dir——但 R834/R836/R849 多轮 net-negative，须 fresh angle 如 paint-side per-glyph subpixel positioning 或 fontdue raster hinting）；② multicol multi-row 模型；③ R109 vertical block-flow；④ position:relative converter-layer 统一。**选一 dedicated 推进，单 session 不求完成，求可验证的 incremental slice**（避免 R376 dormant WIP：每个 slice 须独立 yield 或 net-neutral correctness）。

### R1032 css-fonts font-size-adjust 簇 Phase 0 + Ahem slice 实现 A/B（+0，font-wall 阻 flip）已回退·机制确证·零 net 源码·纯调查

承 R1031 CONTINUE 转 font-wall per-font ascent，转 probe css-fonts（fresh oracle 98/287 34.8%）。定位最高密度簇 **font-size-adjust**（40 文件，7+ top-worst 13-34%），**完全未实现**（css-parser/style/layout/engine 全零命中）。

**Phase 0 机制确证**：CSS Fonts §9.4 `font-size-adjust: <number>` → `used_font_size = font_size × (number / font_aspect)`，aspect = font x-height/em。基础单数语法 18 文件 + 高级两值（ch-width 等 CSS4）10 文件。006/007/008 PASS 是因测「无效值（%）须忽略」——ZW ignore-all 巧合匹配（正确实现也忽略无效值→无回归风险）。font-size-adjust-001-ref 实证 formula：`font:40px/40px Ahem; font-size-adjust:0.9` → ref `font-size:45px`，即 0.9/0.8×40=45（aspect=0.8 = R547 ex-height）。

**Ahem slice 实现 A/B（已回退）**：完整 plumbed（types/computed_style/default/registry 4处+is_inherited/apply/inherit/lib.rs 应用，aspect=0.8 hardcoded for Ahem）。**A/B css-fonts 全量**：001 **2.01→1.81**、002 **4.25→1.25**（改善但**均未 flip <1%**），005/014/units-001 不变（非 Ahem 或高级），006/007/008 持平 0.73（无回归）。**oracle 98→98（+0 net，0 regression）**。

**★ +0 根因 = font-wall**：formula 正确（45px = ref），aspect 0.8 正确（R547），调整已应用（diff 减），但**残余 1.25-1.81% = 调整后尺寸的 glyph 渲染精度**（ZW is_ahem synthetic / fontdue 光栅 vs chromium Ahem.ttf at 45px/10px）——同 R388/R631 font-wall。aspect 调精也无效（45px 已精确，残余在 glyph 光栅非 size 计算）。

**裁决（回退）**：+0 oracle + hardcoded magic constant（0.8，不泛化非 Ahem）+ font-wall 阻 flip = 按 code-guidelines「不做零价值修改 / 不做推测性开发」**回退全部实现**（7 文件 git checkout）。真实现须 ① R887 font-metric provider（real fontdue aspect via `font.metrics('x', px).bounds.height / px`，替代 hardcoded 0.8，泛化所有字体）；② font-wall 解除（glyph 光栅对齐，多 session）。两者皆多 session，premature 单做。

**意义**：font-size-adjust 是 css-fonts 最高密度簇（40 文件），机制/formula 完全确证（45px ref 实证），但**同 css-text/-decor = font-wall gated**——font-wall 是 text-rendering 全 dir（css-fonts/-text/-text-decor）的 pervasive blocker。再确证 R1031 结论：单 session clean lever 全尽，#1 multi-session lever = font-wall（R887 provider + glyph 光栅对齐）。

**▶ 下会话**：font-size-adjust 机制已确证（formula + 45px ref），R887 font-metric provider 落地后可一次性实现（real aspect + 全字体泛化）。但 font-wall glyph 光栅对齐仍可能阻 flip（须 fontdue→chromium 光栅协调，R834/R836/R849 多轮证 net-negative）。建议：① font-wall 是 pervasive blocker，dedicated session 攻 R887 provider + glyph 光栅（最高价值，解 text-rendering 全 dir）；② 或转 multicol multi-row / R109 vertical / position:relative（其它多 session 结构性，非 font-wall）。font-size-adjust 勿以 hardcoded aspect 单 session 重试（已证 +0）。

### R1031 多 dir 复核确认单 session clean lever 全尽·multicol balance-breaking + css-text-decor thickness 均结构性/已 ruled-out·零源码·纯调查

承 R1030 CONTINUE 转 balance-mode column-breaking。本轮多 dir 复核找 clean lever，**结论：单 session clean lever 全尽**。

**balance-mode column-breaking 复核（纠正 R1030「相对窄前置」）**：R1030 称 balance-breaking 是 span-all-children-height 的「相对窄前置」。本轮 probe 其驱动案 multicol-fill-balance-030（1.14%）= **nested multicol + column-fill:auto + break-before:column + 百分比宽 + overflow**（外 columns:2 column-fill:auto 内嵌 columns:2，子含 break-before:column + 50%宽 180/250px 高）——非简单 balance 单案，是 nested+overflow 结构性。balance-breaking 单做不 flip 这些案（需 nested multicol + multi-row 模型）。**R1030「相对窄」纠正为「同 multi-row 结构性」**。

**css-text-decor 复核（fresh oracle 108/242 45%）**：① **text-decoration-thickness（3× ruled-out 再确证）**——paint effects.rs:213 硬编码 `line_width=(font_size*0.06).max(1.0)`，属性零消费；但 R875/R914/R1020-cont 三轮实现均 net-negative（diff 由 table 边框 / font-wall / position:relative offset bug 主导非 thickness）→ **勿再以 thickness 为 lever**。② css-text-decor near-pass 1-3.5% 带 = **text-emphasis-position-property 簇（12+ 案 1.0-1.02%，全 vertical-lr/rl writing-mode 驱动，blocked by R109 vertical block-flow）** + text-emphasis-ruby（1.11-1.19%，ruby+emphasis 交互）+ text-underline-offset（1.02-1.09%，position:relative bug blocked，R1020-cont 回退）。**全 vertical-writing-mode / font-wall / position:relative blocked，无 clean lever**。

**css-values**：0 reftest oracle（testharness JS 单测，非 reftest dir），不可 oracle 测。

**战略结论（再确证 R882/R999/R1026 plateau）**：跨 multicol / css-text-decor / css-values / css-writing-modes / css-tables / css-flexbox / css-grid 全 WPT reftest dir，**单 session clean lever 已尽**。残余 forward motion 全在多 session 结构性：
1. **font-wall**（Phase A IFC font-metric 统一 / per-font ascent provider R887 5-layer / webfont 加载）——解 css-text + css-text-decor + welcome/morning 主体 diff。
2. **R109 §9.2.1.1 匿名块 + vertical block-flow**——解 css-writing-modes 87% 簇 + text-emphasis-position vertical 簇。
3. **multicol Phase 2 multi-row column 模型**——解 span-all-children-height + span-all-rule + nested-balancing 簇。
4. **position:relative offset bug（structurally entangled）**——R1020-cont 证 postprocess 不可解，须 converter/taffy 层统一；解 +12 potential。
5. **feature gaps**（text-decoration-inset CSS4 draft / 表单控件 / ::backdrop / scroll-container）。

**▶ 下会话**：单 session clean lever 已尽，**须 committed 多 session 结构性**。推荐优先级：① **font-wall per-font ascent provider R887**（最高价值——解多 dir 主体 diff，welcome/morning + css-text + css-text-decor；前置 = webfont 加载已 wired（R1027-cont 确证），R990 常数 0.928 已证机制，per-font 增量；5-layer plumbing 多 session 但首层（fontdue ascent → FontLoader → layout）可单 session probe）；② multicol multi-row 模型 dedicated session；③ R109 vertical block-flow dedicated session。**勿再扫近 pass 找 clean lever**（全 blocked）。

### R1030 span-all-children-height 簇 Phase 0 探测 = multi-row column model + overflow columns（多 session 结构性，纠正「row-fill」诊断）·零源码·纯调查

承 R1029 CONTINUE 转 column-fill:auto + spanner row-fill 模型。本轮 Phase 0 探测 span-all-children-height 簇（002/003/004a/004b/006/007 @ 20-34%）真机制。

**关键发现**：此簇非上会话推测的「row-fill 模型」单点，而是 **multi-row column model + overflow columns + 百分比高度** 三层耦合：
- **multicol-span-all-children-height-002** 驱动案注释明示：「Column container has only 25% height left, so **two extra overflow columns are created. Total 4 columns, each 50px**」——block2（height:100%）在 spanner 下方的剩余高度不够，**创建溢出列**，形成 2 行 × 2 列的 column grid。
- **001**（0.72% PASS）：单 spanner + 百分比高度，R1028 balanced-per-region 巧合通过。
- **005/008**（1.03-1.04%）：nested multicol + multi-spanner + column-fill:auto，结构复杂非单点。
- **balance-mode column-breaking**：block1（100px）需跨列拆分以均衡（chromium 行为），ZW balanced 不拆分单 block（block 整体入 col0）→ region 高度错。

**CSS 规范根源**：CSS Multicol §3 与 CSS Fragmentation 的 multi-row column 模型——column-fill:auto + 明确高度时，内容溢出末列会创建新行（multi-row），spanner 在行间断开。这是 taffy 不支持、须自建 row-column grid 的硬核结构性，与 R383 Phase 2 同级。

**裁决**：span-all-children-height 簇 = 多 session 结构性（multi-row + overflow + 百分比 + balance-breaking 四层），**非单 session slice 可产**。下会话勿以「row-fill 单点」重试。真解锁须 dedicated session 实现 multi-row column 模型（row-column grid + overflow column 创建 + spanner row 断开），或先做 balance-mode column-breaking（block 跨列拆分以均衡）这个相对窄的前置。

**multicol near-pass 复核**（1-3% 带）：count-non-integer/negative（1.0-1.17%）= Ahem font-wall（parser 已正确拒负值）；gap-negative/large（1.07-1.18%）= Ahem；baseline-001/004/006（1.02-1.03%）= multicol baseline；column-height-012/multicol-height-001/rule-003（1.13%）= Ahem 精度；intrinsic-size-001（1.14%）= R1020 谱系；fill-balance-030（1.14%）= balance。**全 Ahem font-wall 或已 ruled-out 谱系，无新 clean lever**。

**战略**：css-multicol Oracle 130/452（28.8%）残余 worst 全结构性（multi-row / nested-balancing / breaking / paged / subpixel）。单 session clean lever 在 multicol 已尽（R1027 break-after +1、R1028 column-span:all +8、R1029 rule-split foundational 是本窗口期）。forward = ① multi-row column 模型（多 session 硬核，最高 yield——span-all-children-height + span-all-rule 簇）；② balance-mode column-breaking（前置，相对窄）；③ 转 font-wall webfont per-font 或 writing-modes vertical（同多 session）。

**▶ 下会话**：① **balance-mode column-breaking**（assign_children_to_columns_balanced 对单 child 超过 target_height 时跨列拆分，复用 with_breaking 的 fragment 机制）——这是 span-all-children-height 001/002 + balance 簇的前置，相对窄可单 session probe；② 备选 multi-row column 模型 dedicated session（硬核多 session）；③ 备选转 font-wall/writing-modes。css-multicol near-pass 勿重扫（全 Ahem/ruled-out）。

### R1029 column-rule 在 spanner 处分段（CSS §6.1）LANDED·net-neutral oracle·spec-correctness·foundational

承 R1028-cont CONTINUE 转 column-rule-spanner。CSS Multicol §6.1：column-span:all spanner 使 column-rule 在 spanner 处中断。R1028 spanner 布局后 paint_column_rules 仍画整条 rule（穿过 spanner）。

**实现**（paint/painter/text.rs::paint_column_rules）：① 检测直接子元素 spanner（in-flow + column_span_offsets.is_empty()【R1028 清空，非 spanner 列子元素被填充】+ width >= content_w-1.0【spanner 全宽，列子元素 narrow 到 col_w】）；② 收集 spanner Y 区间；③ 把 rule 的 [0, content_h] 按 spanner 区间分段（区间减法）；④ 每条列 rule 按 segments 循环绘制（Solid/Dotted/Dashed/_ 四 style）。**非 spanner 容器 → spanner_ranges 空 → segments = [(0, content_h)] 单段，行为不变（零回归 gate）。**

**验证**：css-multicol Oracle **130 → 130（0 net）**——multicol-span-all-rule-001 17.30%/rule-002 25.65%/rule-nested-balancing-002/003/004 全 diff 不变。**rule 分段正确但 yield 零**——这些案 diff 由 layout 主导（balanced 模式不拆分单 block 跨列，需 column-fill:auto + spanner row-fill 模型，多 session）。rule-through-spanner 是这些案的小部分。0 regression（column-rule 仅 css-multicol 用）；welcome 16.57% 不变；engine 1179 测 0 failed。

**意义**：spec-correctness（CSS §6.1 rule 中断）正确实现，net-neutral on oracle 但 **foundational**——一旦 column-fill:auto + spanner row-fill 模型实现，rule 部分自动正确无需再改 paint。gate 严格（非 spanner 容器零行为变化）低风险。详见 [`evidence/r1029-column-rule-spanner-split-2026-07-05.txt`](./evidence/r1029-column-rule-spanner-split-2026-07-05.txt)。

**▶ 下会话**：R1029 yield 零，真 yield 解锁需 column-fill:auto + spanner row-fill 模型（span-all-children-height 簇 15-30% + span-all-rule 簇，多 session 结构性，是 R1028 column-span:all 基础上的下一阶）。或转其它结构性 dir（writing-modes vertical block-flow / font-wall webfont per-font）。

### R1028-cont ★spanner width 不强制（尊 taffy auto-stretch / 显式 width）= nested-with-padding-and-spanner 回归修复 + Oracle 129→130（+1）

承 R1028。R1028 初版 `if spanner.width < full_width { spanner.width = full_width }` 误覆盖显式 width spanner（nested-with-padding-and-spanner 的 `width:100px; column-span:all`），致 0.73→3.86% 回归。CSS §6.1：column-span:all 使 spanner containing block 全宽，但 spanner 自身显式 width 须尊重。taffy 已按 block 子规则把 auto 宽拉伸到容器 content_width，故 width 不须强制——移除 override block。

**验证**：nested-with-padding-and-spanner 3.86→0.73% PASS（回归修复）；r1028 单测 release 通过（auto-width spanner 仍全宽 ~400，证 taffy 拉伸有效）；css-multicol Oracle 129→130（+1）；welcome 16.57% 不变；engine/style/layout release 测全 0 failed。

**★ 本会话累计 css-multicol Oracle 120→130（+10 net）**：R1027 break-after:column（+1）+ R1028 column-span:all（+8）+ R1028-cont width fix（+1）。column-span:all 攻克 css-multicol 最大结构性缺口，spanner-fragmentation 簇整体翻 PASS，是 R990 后 multicol 最大单 session yield。

**⚠️ debug-link 预存基础设施问题（wgpu_core debug-info 溢出 lld 32-bit offset）**：layout-engine 测试 binary 链接 wgpu_core（经 render-foundation 依赖），其 debug-info 超 rust-lld 32-bit 偏移上限。**clean R1027-cont（stash 本会话变更）同样链接失败**——预存非本变更引起。临时解法：`RUSTFLAGS="-C debuginfo=1" cargo test` 可链接（debuginfo=1 仅行号）。release 测试全绿不受影响。须后续单独修：`[profile.test] debug=1` 或 layout-engine 解耦 render-foundation 依赖。

### R1028 ★column-span:all spanner 布局 LANDED = css-multicol Oracle 121→129（+8 net）·spanner-fragmentation 簇翻 PASS·Phase 2 第一步

承 R1027 CONTINUE 转向 multicol Phase 2 第一阶：column-span:all。此前完全未实现（css-parser/style-system/layout 全无消费，intrinsic_sizing 用 leaf-guard proxy 近似），185 css-multicol 文件用 column-span:all = css-multicol 最大结构性缺口。

**Phase 0 de-risk**：~10 个 "spanner" 案当前 PASS，核查发现这些案的 column-span:all 元素是 NESTED（非 multicol 直接子，如 spanner-in-opacity 在 opacity div 内）→ column-span:all 仅作用于直接子 → nested 案不受影响，回归风险可控。

**实现**（crates/style-system + crates/layout-engine/src/multicol.rs）：
- style-system（mirror column-fill plumbing）：types.rs `ColumnSpanComputedValue { None, All }` + ColumnSpan 变体；computed_style.rs `column_span` 字段；default_impl None；registry initial + known list（★中途误删 object-fit 已修）；apply_advanced parse none/all；inherit.rs inherit+reset 分支（非继承属性）。
- multicol.rs：① `layout_multicol` 收集期检测 `has_spanner`（直接子 column-span:all），有则分支到 `layout_multicol_with_spanners`（原单区域路径零回归）；② `position_multicol_children` 加 `y_base` 参数 + 返回 region_height（单区域调用传 0.0，行为不变）；③ `layout_multicol_with_spanners` 划分 regions（spanner 作边界，N spanner → N+1 区域）→ 每区域 balanced 分配 → 逐区域定位（y_base 累加）→ spanner 全宽插入（width=content_width，column_span_offsets.clear 正常 block 渲染）。

**限制（R1028 初版）**：每区域 balanced（多数 span-all 用 column-fill:balance 默认）；column-fill:auto + spanner 的 sequential row-fill 暂不支持（更复杂 multi-column row 模型，多 session）。

**验证（chromium Oracle + 三态门禁）**：**css-multicol Oracle 121→129（+8 net）**——9 newly PASS（**spanner-fragmentation-000/001/002/003/004/006/007 簇** + replaced-content-spanner-auto-width + column-height-019），1 regression（nested-with-padding-and-spanner 0.73→3.86%，nested+padding 复杂案，gate 演进可后续修）。top-worst 改善：multicol-span-all-rule-002 28.73→25.65%；column-balancing-paged 81.18→77.32%。welcome 16.57% 不变（<20% gate，无 multicol）。1 新单测 r1028_column_span_all_spanner_is_full_width + 19 既有 multicol 测全绿（position 签名变更零回归）；engine 1179 / layout-engine 1927 / style-system 1016 全 0 failed。

**★ zero-browser bin suite OOM（test-guard 6GB/8GB per-proc）= PRE-EXISTING**：stash 本变更后 clean R1027-cont 同样 OOM 6.3GB。zero-browser 无 multicol 测试，非本变更引起。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / 受影响 crate make test 全绿。

**意义**：攻克 css-multicol 最大结构性缺口（column-span:all），实现 spanner region-split 基础。+8 oracle pass 是 R990（+138）后单轮最大 multicol yield，spanner-fragmentation 簇整体翻 PASS。详见 [`evidence/r1028-column-span-all-2026-07-05.txt`](./evidence/r1028-column-span-all-2026-07-05.txt)。

**▶ 下会话**：R1028 残余 ① nested-with-padding-and-spanner 回归（gate 收紧 nested 守卫，或 spanner 在 nested multicol 内的边界处理）；② span-all-children-height 簇（15-30%，需 column-fill:auto + spanner row-fill 模型，多 session）；③ multicol-span-all-rule（列 rule 在 spanner 处中断）。或转其它结构性 dir（writing-modes vertical / font-wall webfont）。column-span:all 基础已落地，可作 children-height/rule 扩展前置。

### R1027 ★break-after:column 死值消费（R903 mirror）LANDED = multicol-break-000 PASS（1.12→0.81%）·css-multicol Oracle 120→121（+1）零回归·spec-correctness

承 R1026 后续扫 multicol 近-pass（26.5% baseline）。复查 §9.7 dead-value 簇发现 `break_after` 仅在 paint/effects_indicators.rs 画调试指示器时读，**layout 侧从未消费**——与 R903 前的 `break_before` 完全对称的死值（R513 §9.7 扫描遗漏项）。CSS Fragmentation §3.3：`break-after: column` = 放置完当前子元素后强制推进到下一列。

**实现**（crates/layout-engine/src/multicol.rs，mirror R903）：① 收集期加 `forced_breaks_after: Vec<bool>`（`style.break_after ∈ {Column, Page}`）；② 三 assignment 函数（balanced/with_breaking/sequential）签名加 `forced_breaks_after`，每个在放置完子元素 i（含其全部 breaking 片段）后 gate `if forced_breaks_after[i] && current_col + 1 < col_count { advance }`。末列守卫防越界 + 不创建尾随空列（与 R903 break-before 的 `current_col_height > 0.0` 前导空列守卫互补）。

**验证（chromium Oracle + 三态门禁）**：multicol-break-000.xht（`div > div { break-after: column }`）**1.12%→0.81% PASS**（flipped，★纠正首轮「balanced 巧合覆盖」推测——break-after 确为必需）；spanner-fragmentation-012 2.02→1.5%（FAIL 内改善）；**css-multicol Oracle 120→121（+1）零新翻 FAIL**（top-15 worst 完全一致）；column-wrap-no-constraints-002 1.02→1.76%（FAIL 内恶化，该测试用 `column-wrap:wrap` css-multicol-2 草案特性 ZW 不支持，根本性 unsupported，恶化是更正确 break-after 在不支持 row-wrap 下的副作用非真回归）；welcome <20% gate（multicol-only，零影响）。3 新 R1027 单测（break-after balanced/breaking/last-col-noop）+ 9 既有 R903 测更新签名；19 multicol 测全绿。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / make test ✓（workspace 零失败）。

**意义**：关闭 break-after:column 死值（R513 §9.7 遗漏，R903 break-before 对称项）。R903-R907 multicol spec-correctness 簇续延。yield 小（+1）但 dead-value→consumed 真实 spec-correctness，同 R903/R1021 先例。详见 [`evidence/r1027-break-after-column-2026-07-05.txt`](./evidence/r1027-break-after-column-2026-07-05.txt)。

**▶ 下会话**：css-multicol 余下 worst 全结构性（span-all-children-height 需 column-span:all 解析 + Phase 2 rebalancing / nested-balancing / breaking / paged）。forward = ① multicol Phase 2（多 session，column-span:all 解析是前置）；② css-writing-modes vertical block-flow（R109 §9.2.1.1 + vertical flow，本会话 oracle 实测 7% top-worst 全 87% vertical block-flow 结构性）；③ font-wall per-font line-height（须 webfont/font-metric provider）。break-after 死值扫描方法可外推到其它属性（page-break-after legacy alias 等，预期零 yield 因 ZW 不做 paged media）。

**★ break-inside:avoid 死值复核（同 R1027 谱系，已 ruled out）**：break_inside 在 layout 全无消费（仅 paint 指示器），23 multicol 文件用 break-inside:avoid。看似同 break-before/after 死值杠杆，**实测非 clean lever**：① break-inside:avoid 仅在 with_breaking 的 `else`（child_height > max_col_height 拆分）分支有意义；② 驱动案 multicol-br-inside-avoidcolumn-001（200px 子 / 300px 列高）走 `else if`（整体移动不拆分）分支，**根本不经拆分路径**——残差是 overflow 放置（avoid 子溢出末列被 overflow:hidden 裁剪的边界 diff），非拆分 bug；③ multicol-nested-008/022（100px 子 / 100px 列高）也走 else if（==max_col_height），非拆分；④ 多数 break-inside:avoid 案（balance-break-avoidance-000/001、multicol-nested-027/028 等）已 0.73% PASS（balanced 不拆分）。结论：break-inside:avoid 消费仅在「子严格高于列高 + avoid」罕见场景有效（corpus 极少），且改 with_breaking 风险高，yield 窄。**勿以「同 break-before/after 死值」为名单点重试**——已 ruled out。

### R1026 css-tables 近-pass 残余扫描 + table-cell leaf 扩展（net-neutral）+ 空 cell strut 实验（net-negative，font-wall）·零源码 net·纯调查

承 R1025 后扫 css-tables（64.3% baseline）近-pass 残余找下一 lever。**结论：css-tables 近-pass 残余 = 精度 / 空 cell 渲染（font-wall 阻塞），无 clean 单会话 lever**。

**table-cell leaf 扩展实验（net-neutral 已回退）**：R1024 leaf pattern 扩展到 table-cell（`|| matches!(computed.display, DisplayValue::TableCell)`）——table-cell 含文本+br 同 bug 形态。A/B css-tables **74→74 持平**（net-neutral），welcome 16.57% 不变。table-cell 走独立 table auto-layout sizing，leaf 测量不影响列宽分配 → 无 yield。已回退（table 布局风险 + net-neutral，按 code-guidelines 不做零价值修改）。

**★ 空 cell strut 实验（net-negative 已回退，font-wall 阻塞）**：css-tables **46 文件含空单元格** `<td></td>`（cluster）。空 cell 实测 ZW h=6（仅 border，0 内容）vs chromium h~25（cell strut = 一行 line-height）。CSS §17.5.3 空 cell 应有 strut（虚拟行盒）。实施：measure_text_content 空 cell（display:TableCell + 无 inline 内容）返回 height = line-height（resolve_font_metrics[1]）。**A/B net 负**：css-tables **74→73（-1）**，row-margin-border-padding 1.47→1.68、row-group 1.47→1.71（**变差**）。根因：ZW line-height:normal = **1.2×fs**（NORMAL_LINE_HEIGHT_RATIO），chromium 空 cell strut = 字体固有 line-height（DejaVuSans ~**1.16×fs**）→ ZW strut 偏高 → 空 cell 过高 → diff 增。**这是 R989/R990 font-wall**（line-height:normal 常数 1.2 vs 字体固有，R989 证 1.15 对 welcome net 负、1.2 corpus 最优）。空 cell strut 受同一 font-wall 阻塞，非单 session 修。已回退。

**row-margin-border-padding 实查**：ZW dark=4607 vs CHR=8204（缺 ~12/16 空 cell 表的 border，空 cell 塌缩致表不可见）。证实空 cell strut 缺口，但修受 font-wall 阻。

**意义（负面结果）**：css-tables 近-pass 残余**确证无 clean 单会话 lever**（table-cell leaf net-neutral，空 cell strut 受 font-wall 阻）。46 空 cell 案的 yield 阻塞 = line-height:normal 常数 1.2 vs 字体固有 1.16（R989 font-wall，corpus 最优 1.2 已尽）。下会话**勿以空 cell strut 重试**（font-wall，须 per-font line-height 多 session）。

**战略**：R1024 leaf pattern 扩展已尽（flex/grid +6 yield，inline-block/float/table-cell 全 net-neutral correctness）。forward 转向 (a) multicol Phase 2（多 session 结构性）；(b) font-wall per-font line-height（须 webfont 前置或 font-metric provider）；(c) 其它 dir（css-writing-modes 7% 结构性 / css-fonts 35% rustybuzz）。css-tables + css-flexbox + css-grid 近-pass 勿重扫（全精度/font-wall/结构性 plateau）。

**▶ 下会话**：① multicol Phase 2（多 session 结构性，css-multicol 26.5% 唯一非 font-wall 阻塞的近-pass dir）；② css-writing-modes 扫 top-worst 找非 vertical-rl-clearance 的可翻案（7% baseline，R164 vertical-rl clearance 死锁勿重试，但 writing-mode-011/012 等其它簇可能）；③ font-wall per-font line-height（须 webfont/font-metric provider，R1004 dormant 基建）。table-cell leaf / 空 cell strut / R1024 leaf 残余扩展勿重试（已证 net-neutral/negative）。

### R1025 css-flexbox/grid 近-pass 残余扫描 = 精度/结构性 plateau 确认 + ★inline-block 含文本+br 误填满父宽修复 LANDED（R1024 leaf pattern 扩展到 inline-block）·net-neutral oracle·correctness

承 R1024 后扫 css-flexbox + css-grid 近-pass 残余找下一 lever。**结论：两 dir 近-pass 残余 = 精度（text/color 1-2px）/结构性 plateau，无 clean 单会话 lever**。调查中定位 inline-block 同 bug 形态并修复 LANDED（commit d85ed534）。

**css-flexbox 残余复核（post-R1024 59.4%）**：
- **row-reverse 簇**（flexbox_direction-row-reverse 14.5% / flex-direction-row-reverse 11.6%）：LAYOUT_DUMP 实测 **item 顺序正确反转**（`<span>first..forth</span>` 渲染 x=647/487/327/167，从右起 pack = row-reverse 正确语义）。非 order bug，残差 = 颜色（red/green bg）+ 精度。
- **flex-flow-001/002**（23%）：4 方向（row/row-reverse/column/column-reverse）+ wrap 变体，dump 实测**全方向正确**。残差 = 60×60 容器内 4 item shrink 精度（28px→15px）+ 小数字「1-4」渲染 + 多容器累积。
- **flex-0-0-0 簇**（1.31% ×6+）：`flex: 0 0 0%` item 实测 w=26/27/44/35（= min-content，min-width:auto floor）**正确**（chromium 同）。残差 = text/color 精度。
- **near-miss 1.0-1.3%**（align-self/baseline/overflow-padding 等）：全精度，无 clean lever。

**css-grid 残余复核（post-R1024 40.8%）**：
- top-worst 全 R999 已定性结构性（replaced-element-in-grid-nested-in-flex 33.9% / table-grid-item-dynamic 25.8% / grid-container-baseline-synthesized 16-17% ×4 / nested-grid-item-block-size 13.76%）。
- near-miss 1.0-1.6%：anonymous-grid-items-001（1.08%）= **纯文本 grid item**（`<div grid>anonymous item</div>`），本就走 leaf+has_inline_content 正确路径（非 R1024 bug）。残差 = 精度。

**rootpos br 诊断纠正（R1025 纠正 R1024-cont）**：R1024-cont 称「has_inline_content 把 br 测单行高度 → body 高度 = 单行 → 文本溢出」**错误**。复查 measure_text_content：Element-with-inline-content 路径（inline_finalization.rs:914+）**用 IFC 测量**，IFC 识 `<br>` 强制换行 → body 高度正确（多行）。rootpos 残差（4.52%）真因 = body 作 flex item 在 stretched html 内的 main-axis 分配 + 文本定位 diff（flex-basis:auto 在 stretched container 内的 grow/shrink 与 chromium 不同），结构性。已纠正 ruled-out 条目。

**意义（负面结果·有价值）**：css-flexbox + css-grid 近-pass 残余**确证无 clean 单会话 lever**（row-reverse/flex-flow/flex-basis/anonymous-item 全部功能正确，残差全精度/结构性）。下会话**勿重扫此两 dir top-worst**（边际已尽，同 R996-R999 五 dir 穷尽结论的扩展确认）。R1024 是此两 dir 最后一个 clean win（anonymous-flex-item 塌缩）。

**战略**：forward motion 转向 (a) 多 session 结构性（multicol Phase 2 / grid-baseline-synthesis / replaced-element-in-grid-nested-in-flex 三层）；(b) font-wall（per-font ascent，须 @font-face webfont 前置）；(c) R1024 leaf 测量扩展到 inline-block/float/table-cell 等其它 content-sized 上下文（同 bug 形态，但需逐上下文 gate + A/B，潜在 css-tables/css2 yield）。

**★ R1025 inline-block 扩展 LANDED（commit d85ed534）**：调查中复现 `<span display:inline-block>text<br>text</span>` w=800（误填满父宽，应 shrink-to-fit ~338）——同 R1024 bug 形态。修复：R1024 leaf gate 加 `|| matches!(computed.display, DisplayValue::InlineBlock)`（content-sized block = flex/grid item OR inline-block）。验证：复现案 800→338 ✓；chromium Oracle 全 dir（css2/css-flexbox/css-grid/css-position/css-text-decor/css-multicol/css-tables）**全 R1024 baseline 零回归**（net-neutral，inline-block+text+br 罕见于 WPT corpus）；welcome 16.57% 不变；1 新单测 test_r1025_inline_block_with_text_and_br_shrink_to_fit；fmt/clippy/make test 全绿。意义：R1024 leaf pattern 扩展到 inline-block，correctness（真实网页 inline-block 含 br/span+a 文本常见），同 R903/R904/R1021 spec-correctness 先例。net-neutral on oracle（无 driving WPT 案）。

**★ R1025 float 扩展 LANDED（commit f111092b）**：同 inline-block bug 形态，`<div float:left>text<br>text</div>` 误填满父宽（800→340）。leaf gate 加 float 条件。net-neutral oracle 全 dir 零回归，correctness。

**▶ 下会话**：① R1024 leaf pattern 残余扩展候选 **table-cell / auto-abspos**（同 bug 形态，但 table-cell 走独立 table auto-layout sizing 须谨慎 A/B，auto-abspos content-sized 须 gate position）——预期 net-neutral（WPT corpus 不驱动），correctness 收益；② 备选 multicol Phase 2（多 session 结构性）；③ 备选 font-wall webfont 前置（@font-face 加载，R1006/R1007 已部分）。css-flexbox/grid 近-pass 勿重扫（已证精度 plateau）。R1024 leaf pattern 已覆盖 flex/grid item（+6 yield）+ inline-block + float（net-neutral correctness）。

### R1024 ★flex/grid item 含文本+inline Element 子塌缩 w=0 修复 LANDED = css-flexbox Oracle 289→295（+6）+ css-multicol +1 ·零回归·紧 gate（parent-flex/grid）·三方 gate 演进

承 R1023b 精确根因（block flex item 含文本+Element 子如 `<br>` 塌缩 w=0）。本轮 Phase 0 验证高度耦合 + 三轮 gate 演进 + LANDED。

**Phase 0 高度耦合验证（决定安全性）**：`remeasure_text_with_float_exclusions`（inline_finalization.rs:1129）对 block content_height 是 **max/overwrite-if-larger**（`if text_height > content_height { content_height = text_height; height += diff }`，非 add）。但「文本 leaf 作 block 子」会**破坏 inline flow**（每文本 run 成独立 block 行，普通 mixed 块高度膨胀）——实验粗 gate（全 mixed 块作 leaf）welcome **16.57→29.36% 回归**（已回退）。结论：文本 leaf 作 block 子是错的（破坏 inline flow），正确做法是「整 inline 内容作一个 IFC 单位测量」。

**正确修复（紧 gate）**：build_subtree 默认 block 路径（tree.rs:871 else 分支），当容器**同时**满足：
1. **父为 flex/grid 容器**（`is_flex_grid_item`，content-sized item）——fill-width block（multicol 容器、普通 div）不入此路径。
2. **全子为 inline 级**（文本 + display:Inline 元素如 br/span/a，无 block/inline-block/img 等需独立 taffy 子树的子）。
3. **各 inline Element 子无 Element 子**——含 Element 后代（如 span 内嵌 abspos/block）的 inline 须保留 taffy 子树，否则 abspos-in-inline 簇的 span 内 abspos 失去 CB。
→ 整容器作 **leaf**（context=dom_id），measure 回调经 `has_inline_content` 把全部 inline 文本作一个 IFC 单位测量，intrinsic 宽含文本。

**Gate 三轮演进（关键决策审计）**：
- **粗 gate**（全 mixed 块 + 全 inline 子）：welcome -0.17pp 改善，但 **css-position -3**（abspos-in-inline：span 含 abspos 子，span 丢 taffy 子树 → abspos 失 CB）+ **css-multicol -6**（multicol 容器被 leaf 化破坏列分布）。
- **中 gate**（粗 gate + inline Element 无 Element 子）：修 css-position -3（abspos-in-inline span 保 CB），但 css-multicol 仍 -6（multicol 容器仍被 leaf 化）。
- **终 gate**（中 gate + parent-flex/grid）：仅 flex/grid item 走 leaf 路径，fill-width block（multicol/普通 div）完全不变 → **全 dir 零回归**。

**改动**（commit ba7cc0be）：
- `tree.rs`：新 `is_flex_grid_item(doc, styles, dom_id)` 辅助 + 默认 block 路径加 R1024 leaf gate（has_text_child && has_element_child && all_inline && is_flex_grid_item）。
- `anonymous_flex_item_tests.rs`：`test_r1024_flex_item_with_text_and_br_not_collapse`（`<div flex><div id=item>text<br>text</div></div>` → item w>100，证不塌缩）。

**验证（chromium Oracle + 三态门禁）**：
- **css-flexbox Oracle 289→295（+6 零回归）**——anonymous-flex-item-001~006（0.89-0.91% 翻 PASS，直接对应本 bug：flex item 含 inline 文本）+ align-self-001~013 簇（0.73% 改善）+ aspect-ratio-intrinsic-011 等。
- **css-multicol 119→120（+1 零回归）**。
- css-position 55 / css-text-decor 108 / css-tables 74 / css2 9 / css-grid 20 **全 baseline 零回归**。
- welcome 16.57% 不变（<20% gate ✓）。
- per-case A/B 见 [`evidence/r1024-flex-item-text-collapse-fix-2026-07-04.txt`](./evidence/r1024-flex-item-text-collapse-fix-2026-07-04.txt)。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（workspace 零失败，layout-engine +1 R1024 测）/ product-smoke welcome 16.57% < 20% ✓。

**意义**：R1023b 定位的「block flex item 含文本+inline Element 子塌缩 w=0」bug **修复**——CSS Flexbox §4 inline 子应作 IFC 单位测量（非每文本 run 独立 block 行）。anonymous-flex-item 簇 + align-self 簇受益。紧 gate（parent-flex/grid）保证只修 content-sized item，不波及 fill-width block（multicol/普通 div），全 dir 零回归。R1023「flex+text per-word」误判经 R1023b 纠正后，本轮完成真正修复。

**★ rootpos 4 案（position-{absolute,fixed}-root-element-{flex,grid}）**：body 现作 flex item leaf（w 正确），但 root-abspos stretch（R1023 已测绘，dormant）未启 → html 仍 shrinkwrap → rootpos 未翻。下会话 root-abspos stretch + 本修复合做可望翻 rootpos 4 案（root-abspos 此前 net-negative 是因 body 塌缩，现 body 已修）。

**▶ 下会话**：① root-abspos stretch 重试**已做仍 net-negative**（4.05→4.52%，body br 多行高度未解，见 ruled-out），第三次回退——rootpos 真前置 = has_inline_content 的 height 须识 `<br>` 强制换行（leaf 测量按 br 分段累加），独立多 session；② 备选其它 css-flexbox 残余（59.4% baseline 后续 lever，扫 top-worst）；③ 备选 css-multicol Phase 2 / per-font ascent（webfont 前置）等结构性。R1024 已 land（css-flexbox +6 零回归），forward motion 转 css-flexbox 残余或其它近-pass dir 扫描。

### R1023 per-font ascent（mono）re-confirm R1005 + root-element abspos stretch 实验 net-negative 已回退·flex+匿名文本 per-word 堆叠 bug 定位（root-abspos 前置阻塞）·零源码 net·纯调查

承 R1021/R1022 后转 css-position 4 案簇 root-element abspos（4.05% twin）。本轮先验证 per-font ascent 对 monospace 是否 lever，再试 root-abspos stretch fix。

**① per-font ascent for monospace = 死路（re-confirm R1005）**：fontdue 0.9.3 实测——DejaVuSans **ascent/fs=0.9282**，DejaVuSansMono **ascent/fs=0.9282（完全相同）**。R990 is_ahem-gated 常数 0.928（DejaVuSans 派生）**对 monospace 精确正确**。css-text-decor 簇（用 `font: monospace`）行盒 ascent 已被 R990 覆盖，无 per-font lever。R1005「per-font wiring 当前零 yield」结论对 monospace 角度再确认。per-font wiring（R1004 step-2）须等 @font-face webfont 才有 yield，premature。

**② root-element abspos stretch 实验（net-negative 已回退）**：css-position 4 案（position-{absolute,fixed}-root-element-{flex,grid}，4.05% twin）= 根 `<html>` position:absolute/fixed + 全 length inset + auto 尺寸。taffy 对 root 不应用 abspos stretch（root 在 taffy 树中无 CB）→ ZW shrinkwrap 到文本，border 不绘。实施 `apply_root_position_stretch`（engine.rs，紧 gate：仅根 + abspos/fixed + 全 Px inset + auto 尺寸 + HorizontalTb → taffy 设 Relative + size=viewport−insets + left/top 偏移，mark_dirty 重跑）**结构正确**：border 5px dashed 正确填充视口（inset 10/20/30/40），dark-pixel 3734→13759（近 CHR 11216）。但 A/B **4 案 4.05%→4.76%（+0.71pp net 负）**——border 增益被 flex+匿名文本 per-word 堆叠 diff 反超。

**★ flex+匿名文本 per-word 堆叠 bug 定位（root-abspos 前置阻塞 + 广泛影响）**：根 `<html>` display:flex 的直接文本子（"The black border should encompass..."）在 ZW 每个 **word** 当独立 flex item 垂直堆叠（CSS Flexbox §4 应为每 contiguous inline text run 包成**一个**匿名 block-level flex item）。独立验证：`<div style="display:flex;width:400px">The quick brown fox jumps over the lazy dog</div>`（单文本节点，~300px 应单行）ZW 渲染为多行 per-word 堆叠。该 bug (a) 阻塞 root-abspos 4 案 yield（须合修），(b) 影响所有 flex 容器直接文本子（真实网页常见，如 flex nav/footer 直接放文本）——潜在高 EV 多 session lever，独立于 root-abspos。

**裁决（回退）**：root-abspos fix 结构正确但 net 负（4 案 +0.71pp），按 code-guidelines「不做负价值修改」**已 git checkout 回退**（零 net 源码）。root-abspos 须与 flex+匿名文本修复合做才 net-positive。flex+text bug 入 ruled-out 记录并标为下会话 lever。

**门禁**：本轮纯调查 + 实验（已回退），make test 未跑（零 net 源码，dev build 通过）。css-position oracle A/B 4 案数据已采（4.05→4.76 证 net 负）。

**★ R1023b 纠正（per-word 堆叠结论错误，真根因 = block flex item 含文本子时 w=0）**：R1023 上文「flex+匿名文本 per-word 堆叠 bug」**归因错误**——`<div style="display:flex">The quick brown fox...</div>`（单文本子）实测**正确渲染**（text 包成 1 个匿名 flex item，正常换行，非 per-word）。LAYOUT_DUMP 复测驱动案 rootpos 真因：`<html display:flex>` 的子是 `<body>`（**Element flex item，display:block**），body 的子 = 文本 + `<br>` + `<br>` + 文本。**block 容器默认 build 路径（tree.rs:876）只处理 Element 子，跳过文本节点** → body 的 taffy 子树 = [br, br]（文本被丢），body 成 `new_with_children`（非 leaf）→ measure 回调**不触发** → body intrinsic width = 仅 br 贡献 = 0 → body w=0 → 文本 wrap 到 ~0 宽度垂直堆叠。独立验证：`<div class=flex><div class=item>text<br><br>text</div></div>` → div.item **w=2**（塌缩）；同结构去 `<br>` → div.item w=342（正确，纯文本时成 leaf + measure has_inline_content 触发）。

  **精确机制**：block flex item（auto 宽）的 intrinsic 宽来自 taffy 子树测量。纯文本子时 item 成 leaf（context=dom_id），measure_text_content 的 has_inline_content 分支测文本宽 → 正确。但**有 Element 子（如 br）时 item 成 new_with_children 非 leaf**，measure 不触发；而默认 block build 路径不收文本节点 → 文本不进 taffy 子树 → item intrinsic = Element 子之和（br=0）→ 塌缩。**触发条件**：flex/grid item 是 block 容器 + 含文本子 + 含至少一个 Element 子（br/span/a 等任意 inline Element）。

  **影响范围（诚实·小于 R1023 误判的「广泛」）**：仅 flex/grid item 是 block 容器且**同时**含文本子 + inline Element 子时触发。纯文本 flex item（常见，如 flex nav 直接文本）**不受影响**（leaf + has_inline_content 正确）。驱动案：rootpos 4 案（body 含 text+br）+ 任何 flex item 内有 `<br>` 或 inline 元素混文本的场景（罕见于真实网页主导航，多见于测试页）。

  **修复风险（非单 session clean）**：候选修复「block 容器 build 路径也收文本子作 anon taffy leaf（类 flex 路径 tree.rs:732）」——但 taffy 会把文本 leaf 当 block 子垂直堆叠，**block 容器高度可能 double-count**（taffy 算一份 + paint IFC 又算一份），回归风险高（影响所有 block 容器高度）。正确修复须 (a) 仅对 flex/grid item 的 block 子收文本 leaf + 守卫不 double-count 高度，或 (b) 启用 R109 §9.2.1.1 匿名块包裹（gated，已知有回归）。多 session。root-abspos 须与此合做。

**▶ 下会话**：① **block flex item 含文本+Element 子塌缩 w=0** 修复（精确机制如上）——首做 Phase 0 确认修复方案不 double-count 高度（可能须仅对 flex/grid parent 的 block 子启用，或 anon leaf 高度置 0 让 paint IFC 独占高度），A/B rootpos 4 案 + 全量 oracle 零回归。② root-abspos stretch（R1023 已测绘，dormant）须与此合做才 net-positive。③ 备选 R1020 ruled-out 结构性。per-font ascent 已闭（monospace = sans = 0.928）。

### R1022 <ruby> 渲染最小实现 LANDED·rb inline + rt zero-width annotation 上移·net-neutral oracle（108/242 持平，font-wall 阻塞 yield）·零回归·spec-correctness

承 R1021「ruby 是 text-emphasis 混合案 + ruby 专属簇的共同阻塞（71/242 文件 footprint）」。本轮实施 R1021 测绘的 ruby 最小切片（CSS Ruby Layout L1）。

**改动**（commit 3228b358，跨 2 crate）：
- layout-engine/inline/mod.rs：`collect_text_excluding` 辅助（递归收集文本，跳过指定 local_name 子树）+ `collect_inline_items` 对 `<ruby>` 元素改用 `collect_text_excluding(ruby, ["rt","rp"])` 替代 `text_content`——rb 文本正常进 inline 流，`<rt>`/`<rp>` 文本被排除（不再当行内字符与 rb 混排 "F●i●l●..."）。TextRun node_id = ruby 元素。
- engine/paint/painter/text.rs：`ruby_annotation_chars` + `collect_ruby_rt_text`（paint 期按 fragment owner 是否 `<ruby>` 收集 rt 后代文本）+ Path A / Path B（render_fragment! 宏）逐字符渲染 rt[k] 居中于 rb char k 上方（rt_fs=0.5×frag_fs，几何镜像 text-emphasis over 标记）。
- 测试：`test_r1022_ruby_excludes_rt_rp_from_inline_flow`（parse_html `<ruby><rb>Fi</rb><rt>●●</rt><rp>注</rp></ruby>l` → IFC layout → 断言 ● 与「注」不在 inline 流，rb "Fi"+尾 "l" 保留）。

**★ A/B 实测 net-neutral（诚实·font-wall 阻塞 yield）**：stash A/B chromium-Oracle css-text-decor **108/242 → 108/242 持平**，6 案微改善（text-emphasis-position-over/under-right/left-**002** -0.18pp ×4、color-001 -0.13、shape-001 -0.11），**零回归**。见 [`evidence/r1022-ruby-ab-2026-07-04.txt`](./evidence/r1022-ruby-ab-2026-07-04.txt)。

**★ dark-pixel 实测定位 yield 真因 = font-wall（非 ruby 标记定位）**：filled-001 驱动案 dark-pixel 计数——R1021（rt 当行内字符）**11627** → R1022（rt 上移）**11102** → CHR oracle **7064**。R1022 正确把 rt 移出 inline 流（dark 降 525），但 ZW 总 dark（11102）仍**远高于** CHR（7064）= ZW 文本渲染比 chromium 厚（font-wall，同 R1021 text-emphasis 谱系）。ruby 标记 image-level 证实**正确上移**（rt 形状/over 位置匹配 chromium）。yield 不来 = font-wall 主导 diff，非 ruby 实现错误。

**意义**：`<ruby>` 结构现**正确**（此前 `<ruby><rb>F</rb><rt>●</rt>...` 被 `text_content` 扁平化为 "F●i●l●l●e●r●"，rt 的 ● 当行内字符与 rb 混排；现 rb "Filler" 正常 inline 流 + rt ● 作 zero-width annotation 上移到 rb 字符上方）。CSS Ruby Layout L1 的最小可行切片落地。yield 阻塞 = font-wall（同 text-emphasis R1021，ZW 光栅/度量比 chromium 厚 ~57%），非 ruby 实现。零回归（welcome 16.57% 不变证 inline collection 改动只对 `<ruby>` 分支，非 ruby 页面零影响）。

**★ 限制（多 session 改进点，下会话勿以单 session 期望 yield）**：
1. **rt 标记无 line-box 扩展**：chromium 为 ruby annotation 扩展行盒高度（rb 行之上加 annotation 行），ZW 行盒高度不变 → rt 渲染在行盒上方 may 挤占上一行（line-height 紧的案如 position-over-right-001 `font:20px/1em` 行盒仅 20px，rt 上移到行盒外）。须 layout 期为含 ruby 的行加 annotation 半行高（多 session，影响全 IFC 行盒度量）。
2. **多字符 rt 居中于整个 rb segment 未实现**：当前 rt[k] 配对 rb char k 逐字符方案——WPT 主流案（每 `<rb>` 1 字符 + 每 `<rt>` 1 字符）正确；多字符 rt over 多字符 rb（如 `<rb>Fill</rb><rt>XY</rt>`）会逐字符错位（应居中 XY over Fill 的整 advance）。
3. **`<rtc>` / `ruby-position: under` / 垂直 ruby / ruby-line-break / 级联 annotation 未支持**（CSS Ruby L1 完整特性集，多 session）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（workspace 零失败，layout-engine +1 R1022 测）/ **product-smoke welcome 16.57% < 20% gate** ✓（welcome 无 ruby，零影响，证 inline collection 改动安全）。

**▶ 下会话**：① ruby line-box 扩展（多 session，layout 期为含 ruby 行加 annotation 半行高）——可望翻 position-over/under-*-001 等紧 line-height 案；② ruby 多字符 rt 居中（rt 文本居中于 rb segment advance，非逐字符）；③ 或转 R1020 已 ruled-out 的其它结构性（multicol Phase 2 / R109 / per-font ascent）。ruby 结构已 land（3228b358），font-wall 仍是 text-decor 簇 yield 主阻塞（同 R1021 结论）。

### R1021 text-emphasis-style/position 解析+继承+渲染 LANDED（CSS Text Decoration 3 §3）·net-neutral oracle（108/242 持平）·驱动案混 <ruby> 未实现阻塞·spec-correctness·纯调查定位 ruby 真因

承 R1020 后转 css-text-decor（doc R232 标 text-emphasis 未实现）。实现 `text-emphasis-style`（none / [filled|open] × [dot|circle|double-circle|triangle|sesame] 任意顺序 + `<string>` 首字符）+ `text-emphasis-position`（over/under × left/right，默认 over right）两个**继承**属性，paint 期每个非空白字符上方（over）或下方（under）居中绘制 0.5×font-size 标记字符。

**改动**（commit 019f7fb0，跨 3 crate）：
- css-parser: `parse_text_emphasis_style`（关键字组合→标记字符 U+2022/25CF/25CB/25C9/25CE/25B2/25B3/FE45/FE46；`<string>` 取首字符；空串 None）/ `parse_text_emphasis_position` + `TextEmphasisStyleValue`/`TextEmphasisPositionValue` 枚举。
- style-system: registry 注册两属性（均标继承）+ apply 分支 + inherit + apply_initial + default_impl + computed_style 两字段 + PropertyValue 两变体。
- engine paint（text.rs）: Path A（inline owner-style）与 Path B（`render_fragment!` 宏）两处按 fragment owner 样式取 mark char + over/under，逐字符 `add_glyph`（mark_fs=0.5×frag_fs，居中于 char_pos-advance/2，over 在 frag_base_y 之上 / under 之下）。owner_id 取 fragment 真实 owner（`<span>` 上设的属性生效），非容器 style。
- 6 单测（parse 关键字/组合/string/非法/position 缺省）。

**★ A/B 实测 net-neutral（诚实·零 PASS 翻转）**：stash A/B chromium-Oracle css-text-decor **108/242 → 108/242 持平**；4 案微回归 +0.06~0.12pp（字体噪声级，style-007/008/010 + color-001）。per-case 见 [`evidence/r1021-text-emphasis-ab-2026-07-04.txt`](./evidence/r1021-text-emphasis-ab-2026-07-04.txt)。

**★ 真根因（图像分析定位·纠正前轮「mark 定位」方向）**：渲染 `text-emphasis-style-filled-001.xht` vs chromium oracle PNG 对比——
- ZW **上半行**（`<span style="text-emphasis-style">Filler</span>`）：标记**正确渲染**（circle/dot/double-circle/sesame/triangle 形状区分正确，over 位置在文本上方留 leading 间隙，匹配 chromium）。
- ZW **下半行**（`<ruby><rb>F</rb><rt>●</rt>...</ruby>`）：**零标记**——ZW 完全不识别 `<ruby>/<rb>/<rt>`（`ua_default_display` 未列 → 回落 inline），`<rt>` 的 `●` 当作**行内字符**渲染（与 rb 文字混排在基线上），而非 chromium 的「rt 文字悬浮在 rb 之上」。
- 测试页设计：上半行用 text-emphasis，下半行用 ruby 模拟，期望两者**视觉一致**（"Test passes if upper and lower block identical"）。ZW 下半行缺 ruby 标记 = oracle diff 主导（filled-001 6.30% / position-over-right-001 3.48% 残余**主要由 ruby 缺口构成**，非 mark 定位）。

**前轮「mark y-offset 调参」方向证伪**：上一会话 thinking 深陷 mark_y 偏移推导（over 位置 1.78×fs 等）。本轮 A/B + 图像分析证**mark 定位已正确**（图像分析「floating with a small gap above letters，匹配 chromium」），调参是死路——真阻塞是 ruby。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0**（workspace 零失败，css-parser +150 含 +6 R1021 测）/ **product-smoke welcome 16.57% < 20% gate** ✓（welcome 无 text-emphasis，零影响）。

**★ ruby footprint 测绘（下会话 lever）**：css-text-decor 中 **71/242 文件用 `<ruby>`**（29%，含 text-emphasis-* 用 ruby 作 ref 模拟 + text-emphasis-ruby-* 簇 + ruby-text-decoration-* 簇）+ css-text 3 文件。`ua_default_display`（lib.rs:49）未列 ruby/rb/rt/rtc/rp → 全回落 inline。**ruby 渲染是 text-emphasis 混合案 + ruby 专属簇（~130 案）的共同阻塞**，实现后 css-text-decor 可望从 108/242（45%）显著上跳。

**ruby 多 session 性**：UA 识别（display:ruby 变体 + UA 表）**不能单独做**——无 layout 支持时 `<rt>` 会消失或破坏当前可读的 inline 回退。须并发：① 新 `DisplayValue::Ruby`/`RubyBase`/`RubyText`/`RubyBaseContainer`/`RubyTextContainer` 变体（CSS Ruby Layout L1）② UA 表列 ruby/rb/rt/rtc/rp（rp → none）③ inline layout 把 ruby 当特殊 inline 盒，rt 配对 rb 上移（类 text-emphasis per-char mark，但 per-rb-segment 且 rt 文本变长）④ paint 走 inline text 路径带 rt 偏移。最小 slice = 单字符 rb/rt（WPT 主流案）的「rt 上移 0.5em 作 superscript」近似，但须先有 display 变体 + UA + layout 配对。非单 session。

**★ Phase 0 UA-style hack REFUTED（下会话勿重试）**：考虑用 UA 样式 `rt { font-size:0.5em; vertical-align:0.5em }`（lib.rs:415 `ua_decl_inputs` 机制，已为 body/h1/p/a/hr 等用）让 rt 上浮。**机制不成立**：ruby annotation 必须 **zero-width 悬浮在配对 rb 正上方**（如 text-emphasis mark 居中在 char_pos 上方不占 advance）——`<ruby><rb>F</rb><rt>●</rt><rb>i</rb>...` 期望 "Filler" 上方每字母居中一 ●。若 rt 走 inline run，● 会占自身 advance（0.5em）排在 F **右侧**（F●i● 横向错位），非 F 上方。CSS `vertical-align` 只改 y 不改 zero-width，UA-style 单独**无法**实现 ruby（须 layout 把 rt 文本标为零宽 + 上移到 rb 位置，即 IFC 文本收集期识别 rt-owner run + paint 期不 advance、y 偏移）。下会话 ruby 须从 IFC/inline layout 入手，勿以 UA-style/vertical-align 重试。

**意义**：text-emphasis（doc R232 标未实现）现 parse+store+apply+render 全链工作，是真实 CSS spec-correctness（CSS Text Decoration 3 §3）。net-neutral 因驱动案阻塞在 ruby（独立 feature 缺口），非 text-emphasis 本身错误——image-level 证实标记形状/位置正确。前轮 mark-y 调参方向纠正。ruby 测绘为下会话 dedicated 多 session lever 奠基（71 文件 footprint + UA/layout/paint 三层切片面已画）。

**▶ 下会话**：① **ruby 渲染 dedicated 多 session**——Phase 0 设计 minimal ruby（display 变体 + UA + inline 配对 layout），首 slice 目标单字符 rb/rt 的 rt 上移近似，A/B css-text-decor（目标 +N，71 ruby 文件 + text-emphasis 混合案受益）；② 备选 R1020 已 ruled-out 的结构性（multicol Phase 2 / R109 / per-font ascent）。text-emphasis 本身已 land（019f7fb0），下会话勿重做 mark 定位（已证正确）。

### R1020 ★multicol 容器 intrinsic = N × column-content shrink-to-fit gate LANDED = change-intrinsic-width + intrinsic-width-change-column-count 双 PASS（+2，1.75/2.28→0.73）·修 R1018 已知 multicol 回归·1 已知 leaf-guard 限制（multicol-width-005 +0.94，已 FAIL）

承 R1019 未解 ①「multicol intrinsic = columns × content（intrinsic-width-change-column-count，R1018 已知回归）」。R1018 的 `block_max_content_width` 只测单子 max-content 宽 → multicol `columns:N` 容器被 gate 收缩到单列宽（应 N × 列宽）。本轮**精准在 `block_max_content_width` 加 multicol 分支**。

**改动（1 处，css-multicol-1 §3.4 + css-sizing-3 §max-content）**：`intrinsic_sizing.rs::block_max_content_width` 末尾加 multicol 分支——容器 `column_count:Number(n) ≥ 2` 且**所有 in-flow 子都是 leaf（无元素孙）**时，intrinsic = `n × inner + (n-1) × gap_px + frame`（inner = max 子宽，gap 来自 `column_gap:Px`）。leaf guard 守护 column-span:all 子（含嵌套元素，intrinsic-size-002/003/004）——span:all 子有元素孙 → guard 跳过 N× → 容器取 max(span:all content) 全宽。ZW 暂未解析 `column-span`，用「子是否 leaf」作代理（span:all 通常含嵌套结构）。

**验证（chromium Oracle + ORACLE_DUMP_ALL 全量 per-case A/B，stash 严格对照，css-multicol 453 案全 diff）**：
- **css-multicol +2 PASS**：`change-intrinsic-width` **1.75→0.73%（PASS）**、`intrinsic-width-change-column-count` **2.28→0.73%（PASS，★修 R1018 已知回归 0.73→2.28）**。
- **oracle 净 +2**：118/452 (26%) → 120/452 (26.5%)。**全 453 案 diff 仅 3 案变化**——上述 2 PASS flip + multicol-width-005 8.85→9.79（+0.94，non-flip，见下）。
- **leaf guard 守护目标确证**：intrinsic-size-002 (0.73% PASS) / 003 (0.73% PASS) / 004 (3.84%) / multicol-width-004 (5.37%) **全不变**——span:all 含元素子的案 guard 正确跳过 N×。
- **welcome (DC-13) 16.57% 不变**（<20% gate PASS）；make test exit 0（layout-engine 1007 含 +2 r1020 测）；clippy --workspace -D warnings ✓；fmt ✓。

**★ multicol-width-005 +0.94pp 已知限制（leaf guard 代理失效案，非 PASS flip）**：该页 6 个 `<article>` 中 case 4/5/6 用 `column-count:2` + `.spanner{column-span:all}` 子，spanner 子仅含文本（"spanner"）无元素孙 → leaf guard 误判为 leaf → N× 触发。case 6 spanner `width:250px` 被误乘 2×250=500（应 250，span:all 全宽不乘 N）。**根因**：ZW 未解析 `column-span` 属性，leaf 代理无法区分文本 spanner 与真 leaf 列内容。**为何接受**：① 该案 8.85% 已 FAIL（column-span:all + column-width 双语义均未实现，非本轮范围）；② +0.94 non-flip（8.85→9.79 仍 FAIL）；③ 驱动 +2 PASS 是 R1018 已知 gap ① 的既定目标；④ 修须先解析 `column-span`（独立多 session slice，css-multicol-1 §6）。诚实记录，不掩盖。

**未解（独立 gap）**：① `column-span` 属性未解析（multicol-width-005 leaf-guard 限制 + span:all 渲染独立子问题）；② MinContent 测量（min-content = 最宽词，独立函数，flex-container-min-content-001 4.96% 近 PASS）；③ height:max-content 真正 intrinsic（现 flex 容器 Auto 近似）。

**▶ 下会话**：① **MinContent 测量**（flex-container-min-content-001 4.96% 近 PASS，min-content width 函数，gate 扩 MinContent block 路径，+1 potential）；② column-span 解析（解锁 multicol-width-005 + span:all 渲染簇，多 session）；③ 转其它 dir fresh worst（writing-modes R109 structural / abspos-inset）。clean win 已 land（+2 PASS + 修 R1018 回归）。

**★ R1020 续 MinContent 深度 de-risk（同 session，零源码，纯调查）**：本轮 R1020 land 后 dedicated 调查 MinContent lever 单 session 可行性，结论 = **不 clean，双阻塞**：
- **MinContent 测量基础设施缺失**：`intrinsic_sizing.rs` 现有 `text_content_max_width`/`flex_item_base_size`/`flex_row_intrinsic_width` 全是 max-content 语义（sum 全字符，假设不换行）。min-content 须新 `text_content_min_width`（split 空格取最宽词）+ `flex_item_min_content_base_size` + `flex_row_min_content_width`（~3 函数），且 gate `apply_intrinsic_content_sizing`（engine.rs:656-744）须按 `s.width==MinContent` dispatch 到 min-content 测量（现 is_max_min 含 MinContent 但测量走 max-content）。
- **ch 单位近似阻塞 headline driver**：flex-container-min-content-001 用 `font:10px/1 Ahem` + 48 子案大量 `ch` 单位（0.4ch/1.5ch/2ch/0.2ch flex-basis/width）。`computed.rs:46` 解 `Ch(v)=v*font_size*0.5`（近似 1ch≈0.5em）——**对 Ahem 错**：Ahem 字形 advance=font-size，1ch 应=10px（1em），ZW 给 5px（2× 错）。即使 min-content 测量正确，ch 2× 错仍致 48 子案大量 diff。修须 `Ch` font-aware（Ahem→1em，is_ahem 模式同 `estimate_char_width`，~1 行 + helper）。
- **其它 min-content 案不同机制**：flex-item-max-width-min-content = `max-width:min-content`+vertical-rl（不同 gate 路径）；flex-minimum-width-flex-items-013 = replaced img `min-size:auto`（R428 谱系，非 width:MinContent 测量）。均非本轮 lever。
- **裁决**：MinContent 单 session 须**合做**（min-content 测量 + ch Ahem font-aware）且 yield 不确定（即使合做，48 子案 flex-basis/grow/shrink 交互仍可能残留 >1%）。列为多 session slice。**推荐下会话**：要么合做 MinContent+ch（高复杂度），要么转 fresh dir worst 扫描（找未触 dir 的 clean lever，更可能产 +N PASS）。

**★ R1020 续 css-position fresh 扫描 + position:relative-table 簇诊断（同 session，零源码，纯调查）**：本轮做 css-position fresh worst 扫描（ORACLE_DUMP_ALL，98 案）定位近 PASS 候选。发现 **position-relative-table-{tbody,tfoot,thead}-{left,top}-absolute-child 簇 6 案 @ 1.32%（同根因，+6 potential）**。
- **诊断（layout diagnostic test，已 revert）**：driver `position-relative-table-thead-top-absolute-child` = `<thead class="relative" style="top:50px">` + `<td><div class="absolute" style="top:50px">`，期望 absolute 在 y=100（thead 50 + 自身 50）覆盖 red indicator@100。**ZW 实测**：thead@y=50（offset 已应用 ✓）、absolute@y=50（❌ 应 100）。
- **★ 非 table 特有（DIV 对照同 bug）**：同结构 div 替代 table/thead/td 亦复现——relblock(relative,top:50)@y=50 + 内 absolute(top:50)@y=50（应 100）。**真根因 = 通用 relative-inset × abspos-CB 顺序 bug**：taffy 对 block-level position:relative 元素应用 inset 到 layout.location（engine.rs:370 注释），但 abspos 子元素的 `top` 被解析 against CB 的 **pre-inset 位置**（y=0），未 track CB 的 relative 偏移。故 absolute 落在 y=50（CB 原位 + 自身 top），非 y=100（CB 偏移后 + 自身 top）。
- **为何不 clean 单 session**：① 此路径经 R98/R123/R500 多轮调优（abspos CB resolution），改动顺序高风险（可能破当前 passing 的 abspos+relative 案）；② 修须在 abspos 解析后补传播 CB 的 relative inset 到 abspos 后代（须正确判定 CB 链，非简单 shift）；③ taffy 0.7 内部顺序，可能须 converter 或 postprocess 双侧协调。**列为多 session slice（+6 potential，高复杂度高回归风险）**。
- **css-position 其它近 PASS（不同机制，独立子问题）**：position-absolute-center-001/002/007（1.14/1.85/2.55，abspos margin:auto 居中，R165 未覆盖 abspos 路径）、position-absolute-replaced-intrinsic-size（2.65×2，replaced intrinsic in abspos）、position-absolute-in-inline-003/004/005（2.72-2.99，R109 §9.2.1.1 静态位）。
- **裁决**：css-position 近 PASS 簇均非 clean 单点（abspos CB + 顺序 + R109 结构）。**推荐下会话**：position:relative-table 簇若攻，须 full-corpus A/B 守回归（taffy 顺序改动）；或转 css-tables/css-text-decor fresh 扫描找更低风险 lever。

**★ R1020 续 css-tables fresh 扫描（同 session，零源码，纯调查）**：css-tables 116 案 fresh 扫描，近 PASS 簇 = table-intrinsic-size-001/002/003/004（2.65% ×4，min-content 驱动，MinContent 已 de-risk 双阻塞）、row-margin-border-padding + row-group-margin-border-padding（1.47% ×2）、border-collapse-empty-cell/dynamic-*（1.10-1.45 簇）、colspan-004/table_grid_size_col_colspan（R177 colspan 谱系）。**row-margin-border-padding 深查**：converter:48-56 + painter:454-494/611 **已双层归零** tr/row-group 的 border/padding/margin（layout + paint 均已正确处理），1.47% 残余 = 微妙子案（border-collapse:collapse vs separate 交互 / `.inherited>td{border:inherit}` 继承），**非 clean 单点**。table-intrinsic-size 簇受 MinContent ch 阻塞（同 R1020-cont）。
- **裁决**：css-tables 近 PASS 簇亦均非 clean 单 session（intrinsic=min-content 墙 / row-internal 已双层处理 / colspan=R177 结构）。**3 dir 扫描结论（css-position/css-multicol/css-tables）**：近 PASS 残余全是 (a) font-metric/ch 墙 (b) abspos+relative+table 顺序结构 (c) R109 §9.2.1.1 inline 结构 (d) R177 colspan 结构——均多 session。clean 单 session lever 在 layout/intrinsic/abspos/table 主流 dir **确证耗尽**。**推荐下会话**：选 1 个多 session lever 深做（position:relative-table +6 最高 yield，须 full-corpus A/B；或 MinContent+ch 合做解锁 table-intrinsic-size +4 + flex-container-min-content +1），或转 css-text-decor/css-fonts/css-writing-modes 未扫 dir 碰运气。

**★ R1020 续 css-text-decor 扫描 + text-decoration-thickness/offset 实现→A/B net-negative→REVERTED（同 session）**：css-text-decor fresh 扫描 243 案，最大簇 = **text-emphasis-style/position/ruby 簇（~22 案 1.0-1.3%，全 unimplemented feature，per-glyph 标记绘制，clean 新增低回归风险但大工程）**；次 = text-decoration-thickness/offset（~6 案）。
- **text-decoration-thickness + text-underline-offset 实现（7 件改动：types/computed_style/registry/default_impl/apply/inherit + paint effects.rs）→ A/B css-text-decor 全量 net-NEGATIVE → REVERTED**：text-decoration-thickness-underline-001 **1.26→6.08（+4.8pp）**、overline-001 **1.42→7.92（+6.5pp）**、vertical-001 **0.81→1.20（PASS→FAIL）**、dotted-001/002 +1pp。仅 linethrough-001 1.26→1.01、scroll-001 3.67→2.19 改善（不足抵消）。
- **★ 根因 = position:relative offset bug（同 R1020-cont2 诊断）**：thickness 驱动案 `#text{position:relative;bottom:3em;text-decoration-thickness:4em}` —— `bottom:3em` 须把 #text 上移 60px，4em(80px) 厚 underline 须随之落在 #box(80×20 red) 上覆盖之。ZW 的 position:relative offset 未正确作用于 #text 的装饰绘制 → 80px 厚线落错位（比默认细线更糟）。**thickness 量级正确**（paint_decoration_line Solid 已 grow-down：Rect top=y 向下 line_width），**问题在 #text 相对偏移未到位**。**★ 纠正 R874「text-decoration-thickness 更可控可单 session」推荐**——A/B 实证 net-negative，须先修 position:relative。
- **裁决**：text-decoration-thickness/offset 被 position:relative offset bug 阻塞，单做 net-negative。**position:relative offset bug 现确证阻塞 ≥2 簇**（position:relative-table +6 + text-decoration-thickness +6 = **+12 潜力**），是最高价值多 session 解锁 lever。text-emphasis 簇（+22）独立未阻塞但须 per-glyph 标记绘制（IFC fragment 协调，大工程）。
- **★ 4 dir 扫描 + 1 实现-回退 结论（position/multicol/tables/text-decor）**：position:relative offset bug 是横跨多 dir 的 pervasive blocker。**推荐下会话最高优先 = 修 position:relative offset bug**（解 +12，须 full-corpus A/B 守 abspos 回归），其次 text-emphasis（+22 clean feature）。

**★ R1020 续 position:relative offset 修复尝试→A/B css-position net-NEGATIVE（9 regressed/0 improved）→REVERTED（同 session）**：承 cont4 推荐「修 position:relative offset bug」，本轮**诊断 solidify + 实现 + 全量 A/B**。
- **诊断 solidify（layout diagnostic，已 revert）**：3 组测（simple-top `cb(relative,top:50)+abs(top:50)` / nested `relative-in-relative` / bottom）。**ZW baseline 全错**：abs@y=50（simple/nested 应 100）、abs@y=50（bottom 应 cb-30+50=20）。**机制确证 = abspos 按 CB 静态（pre-inset）位解析 top/left，未 track CB relative 偏移**。bug 通用（非 table 特有、非 nested 特有）。
- **实现（postprocess.rs `propagate_relative_cb_offset_to_abspos` + engine.rs:419 接线）**：遍历树，沿 relative 祖先传 resolve_relative_inset(dx,dy)，对 abspos 加到 x/y；abspos/fixed CB 重置上下文。诊断测 3 组**全对**（abs→100/100/20，与 spec 一致）。
- **★ A/B css-position 98 案全量（stash 严格对照）→ NET-NEGATIVE：9 regressed / 0 improved → REVERTED**。**8 个目标 position-relative-table-* 簇全回归**（thead/tbody/tfoot × left/top，1.32→2.37 +1.05pp ×6；tr-left/top 0.80→1.85 +1.05pp ×2）+ position-relative-004 2.94→4.88。**目标簇本应是受益者却全回归**。
- **★ 矛盾根因（关键）**：诊断（plain div）显示 fix 产 spec-correct y=100，但真实 table 测全回归。说明 **ZW table 布局已通过另一路径（table.rs grid 定位）把 abspos 摆在接近正确位**，fix 的 +50 inset **double-apply/overshoot**（abs 已 ~100，+50→150 越界）。div 测 baseline abs@50（错），table 测 baseline abs 已接近正确（故 1.32% 小 diff）——**两路径 abspos 定位机制不同**，post-hoc 全局传播 inset 对 div 对、对 table 错。
- **裁决**：position:relative offset bug **非简单 postprocess 可解**——须先理解 table 布局如何定位 abspos（为何已接近正确而 div 路径错），可能须在 converter/taffy 层统一（非后处理）。**难度从「risky 多 session」升级为「structurally entangled 多 session」**。R98/R123/R500 谱系 abspos CB 调优路径已多轮，进一步改动须 dedicated table+abspos 交互调查。
- **★ 战略调整**：position:relative offset 修复已证两次间接 blocker（thickness cont4 + 本轮直击）均 net-negative。**下会话转 text-emphasis 簇（+22，clean unblocked feature，per-glyph 标记绘制）更可产**；position:relative 列为长期架构 slice（须 table+abspos 交互 dedicated 调查，非后处理补丁）。

### R1019 ★float:block + flex/grid 子 shrink-to-fit gate LANDED = aspect-ratio-intrinsic-014 14.85→0.60% PASS + css/CSS2/floats-clear +5 PASS（floats-122/145/float-replaced-height-004/005/007）·float-non-replaced-width-007 20.62→5.17（-15.45pp）·零回归

承 R1018 CONTINUE「aspect-ratio-intrinsic-014（float:left block + flex 子 + JS）」。R1018 解 block + width:MaxContent；R1019 解 float:left block + width:auto（float shrink-to-fit 上下文）含 flex/grid 子。

**改动（1 处，engine.rs gate）**：`apply_intrinsic_content_sizing` 的 `is_auto_float` 扩 `DisplayValue::Block`——float:left + width:auto + display:block 触发 gate，用 `block_max_content_width`（R1018 新增，对 flex/grid 子分发到专用 intrinsic）测量。原 `is_auto_float` 仅 Flex/InlineFlex。block + auto-float 的 `should_apply` 走 shrink（`b.width > intrinsic`）。float shrink postprocess（adjust_float_positions_with_context）见此宽度后为 no-op，无双重 shrink。

**机制**：014 的 target（float:left block）含 flex 子（display:flex, height:100%）+ aspect-ratio item。taffy 第一趟无法测 flex 子 intrinsic（aspect-ratio 空 item）→ float 拉满。gate 用 block_max_content_width dispatch flex 子 → flex_row_intrinsic_width → container_cross（box_node.height fallback 解析百分比 height）→ item transferred = 100 → float target shrink 到 100。

**验证（chromium Oracle + ORACLE_DUMP_ALL per-case A/B vs R1018 baseline）**：
- **css-flexbox +1 PASS**：aspect-ratio-intrinsic-size-014 **14.85→0.60% PASS**（-14.25pp）；额外 flexbox_box-clear 9.08→2.13（-6.95pp）。
- **css/CSS2/floats-clear +5 PASS**：floats-122 1.81→0.78、floats-145 3.39→0.55、float-replaced-height-004/005/007 1.40→0.62（×3）；额外 float-non-replaced-width-007 **20.62→5.17（-15.45pp）**、floats-143 6.91→1.27（-5.64pp）。
- **css/CSS2/floats**：float-nowrap-5/6 +0.07/+0.08（噪声，仍 PASS <0.2%），余零变化。
- **css-grid / css-tables / css-position / css-multicol**：未受影响（is_auto_float 仅扩 Block，multicol/grid 非本路径）。
- **welcome (DC-13) 16.57% 不变**（<20% gate PASS）；make test exit 0（layout-engine 1006 含 +1 r1019 测）；clippy --workspace -D warnings ✓；fmt ✓。

**意义**：R1015（block float flex 容器）+ R1017（inline-flex）+ R1018（block + fit-content/max-content）+ R1019（float:block + flex/grid 子）四路径覆盖 flex/grid 容器 shrink-to-fit 主要场景。block_max_content_width（R1018）的 flex/grid 子分发在 float 路径同样生效（gate 统一入口）。float-non-replaced-width-007 等 floats-clear 老案批量改善（float:block 含 replaced/aspect-ratio 子此前未被正确测量）。

**未解（独立 gap）**：① multicol intrinsic = columns × content（intrinsic-width-change-column-count，R1018 已知回归）；② MinContent 测量（min-content = 最宽词，独立函数，flex-container-min-content-001 4.96% 近 PASS）；③ height:max-content 真正 intrinsic（现 flex 容器 Auto 近似）。

**▶ 下会话**：① **MinContent 测量**（flex-container-min-content-001 4.96% 近 PASS，min-content = 最宽不可拆词宽，需独立 min-content-width 函数，gate 扩 MinContent block 路径）；② multicol intrinsic = columns × content（独立多 session）；③ 转其它 dir fresh worst（writing-modes R109 structural / abspos-inset）。clean win 已 land（+6 PASS）。

### R1018 ★bare fit-content 关键字 + block-level max-content shrink-to-fit gate LANDED = aspect-ratio-intrinsic-011 14.85→0.60% PASS + fit-content-item-002/003 PASS（+3）·css-multicol change-intrinsic-width 16→1.75（-14pp）·短路 bug 修复（apply_intrinsic_content_sizing 此前被 `||` 短路）·1 已知 multicol 回归

承 R1017 CONTINUE「011/014 block flex + width:fit-content + JS」。R1017 解 inline-flex（inline-level），R1018 解 block-level + bare `fit-content` 关键字（R97「max-content→0 bug」memory 标「block/inline-block 可独立做」slice）。

**5 件改动（CSS css-sizing-3 §fit-content/max-content + Flexbox §4.5）**：
- **`css-parser/values/types.rs::parse_length`**（★live 函数，非 parse_basic.rs dead-code——R544/R549 trap 规避）：加 bare `fit-content` 关键字 → 映射 `MaxContent`（layout trigger 等价；fit-content(arg) 函数形式仍独立 `FitContent(Box)` 变体）。
- **`intrinsic_sizing.rs`**：① 新 `block_max_content_width`——block-level 容器 max-content 宽，对 flex/grid **子**分发到 `flex_row/column_intrinsic_width`/`grid_intrinsic_width`（通用 `box_content_max_width` 递归对 aspect-ratio 空 item 测 0）；② `flex_row_intrinsic_width` 的 `container_cross` 加 fallback——非 Px height（百分比/auto）用 taffy 第一趟解析的 `box_node.height`（flex 子 height:100% 在 definite-height 父内已解析）。
- **`engine.rs::apply_intrinsic_content_sizing` gate**：① 扩 `is_block`（display:Block）+ `width:MaxContent` 触发 block shrink-to-fit，用 `block_max_content_width`；② block + MaxContent 当 intrinsic 不可测（≤1）时回退 `Dimension::Auto`（fill 父宽），非留 converter 的 0 塌缩（multicol + aspect-ratio 子 box_content 无法度量案）。
- **`engine.rs` 短路 bug 修复（关键 pre-existing bug）**：四趟后处理 pass（r695/pct_padding/ratio_img/intrinsic_sizing）原用 `||` 短路求值，前三趟任一 fire 即跳过 `apply_intrinsic_content_sizing`（R1015/R1017 仅在前三趟都不 fire 时才工作！）。改为先求值 `changed_intrinsic` 再合并，确保四趟总执行。
- **`converter/mod.rs`**：flex/inline-flex 容器的 `height:MaxContent/MinContent` → `Dimension::Auto`（content-based），非 `length(0)`。仅限 flex 容器——grid/block 的 height:max-content 在空 item 时应塌缩（max-content of empty=0），Auto 会触发 align-self stretch 误拉伸（grid-item-non-auto-height-stretch-001 回归）。

**验证（chromium Oracle + ORACLE_DUMP_ALL per-case A/B vs R1017 baseline，5 dir stash 对照）**：
- **css-flexbox +3 PASS**：aspect-ratio-intrinsic-size-011 **14.85→0.60% PASS**（-14.25pp）、fit-content-item-002/003 **2.65→0.60% PASS**（-2.05pp ×2）；额外 flex-container-min-content-001 8.78→4.96（-3.82pp）、flex-container-max-content-001 11.54→10.13（-1.41pp）。
- **css-multicol 大改善**：change-intrinsic-width **16.01→1.75%（-14.26pp 近 PASS）**、intrinsic-size-004 6.98→3.84（-3.14pp）。
- **css-grid / css-tables / css-position：零变化零回归**（display 靶向 + Auto-fallback 守住）。
- **1 已知回归**：css-multicol/intrinsic-width-change-column-count 0.73→2.28（+1.55pp，PASS→FAIL）。根因 = multicol 容器 intrinsic = columns × column-content，`block_max_content_width` 只测单子宽（25）→ gate 应用 25（应 ~75-100）。multicol intrinsic sizing 精度独立 gap（非本轮范围），诚实记录。
- **welcome (DC-13) 16.57% 不变**（<20% gate PASS）；make test exit 0（layout-engine 1005 含 +1 r1018 测）；clippy --workspace -D warnings ✓；fmt ✓。

**★ pre-existing 短路 bug 意义**：`changed_r695 || ... || apply_intrinsic_content_sizing(...)` 的 `||` 在 r695/padding/ratio 任一 true 时短路跳过 gate。这意味着含 aspect-ratio flex item / 百分比 padding / 不明确百分比 height 的页面，flex/grid/block 容器 shrink-to-fit 此前**失效**。修复后 R1015/R1017/R1018 在这些页面也生效（向后放大收益）。R1015/R1017 当时 A/B 的测试恰好不含触发前三趟的元素，故未暴露。

**未解（独立 gap）**：① `aspect-ratio-intrinsic-size-014`（float:left block + flex 子 + JS，14.85% 不变）= float shrink-to-fit 路径需 flex-dispatch 测量（float_positioning 用 box_content_max_width 非 block_max_content_width）；② multicol intrinsic = columns × content（intrinsic-width-change-column-count 回归）；③ MinContent 测量（min-content = 最宽词，独立函数）；④ height:max-content 真正 intrinsic（content height，现 flex 容器 Auto 近似）。

**▶ 下会话**：① **aspect-ratio-intrinsic-014**（float:block + flex 子，float shrink 路径接 block_max_content_width，+1 potential）；② **MinContent 测量**（min-content width 函数，flex-container-min-content-001 4.96% 近 PASS）；③ multicol intrinsic sizing（columns × content，独立多 session）；④ R1015 row+float/inline-flex 已确认覆盖，转其它 dir fresh worst。clean win 已 land（+3 PASS + 多几何改善）。

### R1017 ★inline-flex shrink via IFC `shrink_inline_blocks_to_content` 路径 LANDED = aspect-ratio-intrinsic-003/004 15.67→1.80%（-13.87pp ×2，几何 100×100 精确）·css-flexbox Oracle 291 持平（残余 = `<p>` 字体墙）·零回归·攻克 R1016 IFC 测量墙

承 R1016 CONTINUE「避开 inline-flex IFC 墙，转其它 lever」。R1016 证伪 taffy-gate 路径后定位真因 = 「inline-flex 须经 IFC inline-level 测量层」。本轮**精准在该层落地**（非 taffy gate），机制生效。

**R1016→R1017 路径区别（关键）**：
- **R1016（REFUTED）**：在 `apply_intrinsic_content_sizing`（taffy gate）给 inline-flex `set style.size.width=Length(100)` + 重跑——taffy 0.7 在重跑中仍拉满宽，**set width 被忽略**。
- **R1017（LANDED）**：在 `shrink_inline_blocks_to_content`（float_positioning.rs，**IFC inline-block shrink-to-fit 路径**，R180 基础设施）给 inline-flex 测 intrinsic width。该路径本就处理 inline-level 盒（inline-block），inline-flex 同属 inline-level，**set width 在此层被尊重**（不走 taffy block 布局重跑，而是直接 clamp box width）。

**改动**（2 处，CSS css-sizing-4 §aspect-ratio + css-flexbox §4.5）：
- **`intrinsic_sizing.rs::flex_item_base_size` 加 `container_cross: Option<f32>` 参数**：transferred case main 源优先级 (a) item height Px → (b) item min-height Px 地板 → **(c) R1017 container-stretch cross**（容器 definite height Px，inline-flex `height:100px` 拉伸 item）。`flex_row_intrinsic_width` 读容器 `style.height` Px（区分 box-sizing 转 content height）→ 传 container_cross；`flex_column_intrinsic_width` 传 None（cross=width 是循环）。
- **`float_positioning.rs::shrink_inline_blocks_to_content` 加 fallback**：inline-flex/InlineGrid 当 `box_content_max_width` 测得 ≤0.5（aspect-ratio 空 item，box_content 无法度量）时，fallback 到专用 `flex_intrinsic`（dispatch row/column/grid）；否则保留 `box_content_max_width`（覆盖文本/gap/abspos 等有 content 案，**避免回归**——`>0.5` gate 是回归护栏）。

**验证（chromium Oracle + ORACLE_DUMP_ALL per-case A/B vs R1015 baseline 291，rigorous stash 对照）**：
- **498 案全 A/B，仅 3 案变化，全改善零回归**：
  - `aspect-ratio-intrinsic-size-003`：**15.67%→1.80%（-13.87pp）**
  - `aspect-ratio-intrinsic-size-004`：**15.67%→1.80%（-13.87pp）**
  - `flex-minimum-height-flex-items-015`：1.68%→1.58%（-0.10pp 余波）
- **css-flexbox Oracle 291/497 持平**（003/004 改善但未翻 PASS，仍 >1%）。
- **★ 几何精确（诊断单测 `r1017_inline_flex_definite_height_aspect_ratio_item_shrinks_to_fit`）**：003 驱动 HTML 渲染 = **container 100×100 + item 100×100**，与参考 `ref-filled-green-100px-square-only.html`（100×100 绿方块）**布局维度字节级一致**。
- **★ 残余 1.80% = 字体光栅化墙（非布局）**：003/004 的 `<p>Test passes if there is a filled green square.</p>` 文本在 ZW 默认字体 vs chromium 默认字体下光栅化不同（~8640px = 1.80% 全是 `<p>` 文本 glyph 差异）。布局已完美，残余纯属 R990/R1005 font-wall territory（font-metric 常数已尽，forward = R887 per-font provider wiring，多 session）。**003/004 在当前字体墙下不会翻 PASS**——这是诚实评估，非 R1017 缺陷。
- **welcome 16.57% 不变**（<20% DC-13 gate PASS）；make test exit 0（layout-engine 1004 含 +1 r1017 测）；clippy --workspace -D warnings ✓；fmt ✓。

**★ 与 R1016 的关系（R1017 攻克 R1016 标记的墙）**：R1016 定位「inline-flex 须经 IFC inline-level 测量层」并标「多 session 架构 lever」。R1017 发现该层 = `shrink_inline_blocks_to_content`（已存在的 inline-block shrink 基础设施，R180），**无需新架构**——inline-flex 作为 inline-level 盒本就过此路径，只需在 box_content_max_width 失败时 fallback 到 flex_intrinsic。R1016 的 container_cross 推导逻辑（正确但被 taffy gate 墙阻塞）在 R1017 经 IFC 路径**重新生效**。R1016「inline-flex IFC 测量墙多 session」结论**纠正为「单 session 可解，经 shrink_inline_blocks 路径」**。

**未解（独立子问题，非 R1017 范围）**：`aspect-ratio-intrinsic-size-011/014`（14.85% 不变）= **block-level** flex（`display:flex` 非 inline-flex）+ `width:fit-content` 父 + `height:100%` + **JS 改 height**。完全不同机制（block flex + fit-content + 百分比 height + JS re-layout），R1017 inline-flex gate 不覆盖。需 R370 block-level flex-container-intrinsic-width 扩到 fit-content 上下文（独立 slice）。

**意义**：R370 R1015（block-level float flex）+ R1017（inline-level inline-flex）**双路径覆盖 flex 容器 shrink-to-fit**。R1016 标记的 IFC 测量墙被证明可经现有 inline-block 基础设施解。inline-flex aspect-ratio transferred sizing（css-sizing-4 §aspect-ratio + css-flexbox §4.5）布局维度正确。

**▶ 下会话**：① **011/014 block flex + fit-content + JS**（14.85% ×2，独立子问题，R370 block-level 扩 fit-content 上下文——R1017 inline-flex gate 不覆盖 block-level flex）；② **flexbox-min-height-auto-001（1.87%）** 近 PASS，9 子场景 + 紫色 dotted border，R1015 已从 3.26 改善，残余大概率 dotted border 光栅化 + calc() 子案例；③ **flexbox-collapsed-item-horiz-001（3.77%）** 近 PASS。**★ 已确认无需再做（R1016 遗留清单纠正）**：row+float 对称——R1015 `is_auto_float` gate dispatch 已按 flex_direction 路由 row/column（engine.rs:705-712），row+float **已隐含覆盖**；flex-item-transferred-sizes-padding（0.60%）已 <1% **已 PASS**（R1015 翻）。clean win 已 land。

### R1016 aspect-ratio-intrinsic-003/004 via 容器 cross + inline-flex gate 实验 REFUTED（taffy 不尊重 inline-flex set width）·零 yield 零回归已回退·inline-flex 须经 IFC inline-level 测量·纯调查

承 R1015 CONTINUE「aspect-ratio-intrinsic-003/004 via 容器 definite height + inline-flex gate」。本轮实施 + A/B + 机制证伪。

**实施（已回退）**：
1. `flex_item_base_size` 加 `container_cross: Option<f32>` 参数；transferred case main 源优先级 (a) item height Px → (b) item min-height Px → (c) container_cross（容器 definite height 拉伸）。
2. `flex_row_intrinsic_width` 读容器 style.height Px（区分 box-sizing 转 content height）→ 传 container_cross。
3. `apply_intrinsic_content_sizing` gate 扩 `is_auto_inline`（width:Auto + InlineFlex/InlineGrid，无 float 也触发 shrink）。

**A/B 实测（css-flexbox oracle ORACLE_DUMP_ALL vs R1015 baseline 291）**：**291→291（0 yield，0 回归，0 案 >0.3pp 变化）**。aspect-ratio-intrinsic-003/004 仍 15.67%。零价值零回归。

**★ 机制证伪（TTDBG 探针 + PIL 几何）**：
- TTDBG 确证 gate **触发**（003：InlineFlex width=Auto dir=Row intrinsic=100 current=784 should_apply=true）——intrinsic 计算正确（容器 height:100px × item aspect-ratio:1/1 = 100）。
- 但 PIL 实测 ZW 最终渲染：003 绿盒 (8,16)→(790,115) = **782×99（仍拉满视口，未 shrink）**；CHR (8,50)→(106,149) = 98×99（正确 shrink 到 ~100）。
- 即 **set taffy style.size.width=Length(100) + mark_dirty + 重跑** 对 inline-flex **无效**——taffy 0.7 在重跑中仍把 inline-flex 拉到满宽。

**真因（与 R1015 float:block-flex 对比）**：R1015 transferred-sizes（`float:left + display:flex`，block-level 浮动块）的 set width 被 taffy 尊重 → shrink 生效。**inline-flex 是 inline-level**，taffy 经 IFC inline-level 测量路径，**style.size.width=Length 被忽略**——inline-level 宽度由 IFC 测量决定，非 taffy block 布局。inline-flex shrink 须在 **IFC inline-level 测量**层修（非 taffy style 覆盖）。

**裁决（回退）**：R1016 inline-flex gate + container-cross 逻辑零 yield 零回归，但**机制证伪**（taffy 不尊重 inline-flex set width）。container-cross 逻辑虽正确但被 inline-flex IFC 测量墙阻塞，无 yield。按 code-guidelines「不做零价值修改」**全回退**（git checkout engine.rs + intrinsic_sizing.rs）。inline-flex shrink-to-fit = IFC inline-level 测量多 session（独立架构 lever）。

**意义**：R370 R1015 首切（float:block-flex）yield +2 后，R1016 inline-flex 扩展被 taffy inline-level 测量墙阻塞。**inline-flex/flex 容器 shrink-to-fit 分两条路径**：(a) block-level float（R1015 解，taffy set width 尊重）；(b) inline-level inline-flex（IFC 测量墙，未解，多 session）。aspect-ratio-intrinsic-003/004/011/014（4 案）卡在 (b)。

**▶ 下会话（避开 inline-flex IFC 墙）**：① **flex-row + float 的 row 对称**（R1015 现仅 column+float，row+float 同经 block-level taffy，应可扩，A/B 守回归）；② **flex-item-transferred-sizes 残余 0.60% 拆解**（item padding-left/right:25px box-sizing 精度，可能近 PASS）；③ **inline-flex IFC 测量墙**（多 session，IFC inline-level 测量时传 inline-flex 的 intrinsic 宽，类似 inline-block shrink）；④ 近 PASS 案（flexbox-collapsed-item-horiz-001 3.77% / flexbox-min-height-auto-001 1.87%，R1015 改善后残余）。clean win 阵营：R1015（+2）已 land；inline-flex 是已知墙。

### R1015 ★R370 flex column intrinsic width first slice LANDED = flex-item-transferred-sizes-padding 14.85→0.60% PASS（+2）·css-flexbox Oracle 289→291 零回归·非替换 aspect-ratio transferred-size + column max 变体 + gate 扩 auto+float

承 R1014 Phase 0 测绘。本轮实施 R370 三件中的核心两件（gate 扩 auto+float + column max 变体 + transferred base size），**+2 PASS 零回归**。

**改动**（3 处，CSS css-sizing-4 §aspect-ratio + Flexbox §4.5）：
- **`intrinsic_sizing.rs::flex_item_base_size` 加 transferred case（步骤 2.5）**：width:Auto + aspect_ratio>0 + definite main（height Px 或 min-height Px 地板）→ cross width = main × ratio（`aspect_ratio_transferred_width` helper，区分 border-box / content-box）。覆盖非替换 item（R982/R983 transferred-size 的非替换扩展，R1013 守卫跳过的案的正确方向）。
- **新 `flex_column_intrinsic_width`**：列容器 cross 轴 max-content = **max(item base size + margin)** + frame（镜像 row 的 Σ；列容器主轴垂直，cross=width 取最宽 item）。
- **`engine.rs::apply_intrinsic_content_sizing` gate 扩 + dispatch**：除 MaxContent/MinContent 外，**width:Auto + float≠None + display:Flex/InlineFlex** 也触发（shrink-to-fit 上下文）。flex dispatch：Column/ColumnReverse → `flex_column_intrinsic_width`（max），否则 → `flex_row_intrinsic_width`（Σ）。apply 条件按上下文反转：auto_float 当 current > intrinsic 时 shrink（原 MaxContent 当 current < intrinsic 时 grow）。

**验证（chromium Oracle + ORACLE_DUMP_ALL per-case A/B vs R1013 baseline 289）**：
- **flex-item-transferred-sizes-padding-border-sizing / content-sizing：14.85%→0.60% PASS（-14.25pp ×2，FAIL→PASS）**。
- **额外改善（无翻转）**：flexbox-collapsed-item-horiz-001 15.04→3.77%（-11.27pp）、flexbox-min-height-auto-001 3.26→1.87%（-1.39pp）。
- **css-flexbox Oracle 289→291（+2 PASS，0 回归）**——全 497 案 per-case A/B 仅 4 案变化（上述），无任何他案 >0.5pp 退步。
- **welcome 16.57% 不变**（<20% DC-13 gate）；**CSS2/normal-flow 604/746 (81%) 不变**；**CSS2/floats-clear 66/214 不变**（gate 仅 Flex/InlineFlex，floats-clear/normal-flow 非 flex 容器不受影响）。
- 2 新单测：`r1015_float_flex_column_aspect_ratio_item_shrinks_to_fit`（容器 shrink 到 <200px）+ `r1015_block_flex_column_auto_width_no_shrink`（block flex 不 shrink，零回归）。
- make test exit 0（workspace 绿，layout-engine 1003 含 +2 r1015 测）；clippy --workspace -D warnings ✓；fmt ✓。

**意义**：R370「零杠杆」纠正后首切 +2 PASS。**非替换 aspect-ratio transferred-size**（R982/R983 替换元素 transferred 的自然扩展）+ **flex 列容器 max-content**（镜像 row）+ **width:auto+float shrink-to-fit gate** 三件合做才能 yield（partial slice 无 yield，证 R1014 三耦合件判断）。transferred-sizes ×2 现 PASS。flexbox-collapsed-item-horiz / min-height-auto 改善（近 PASS，潜在下轮翻）。

**★ R1013 与 R1015 的关系**：R1013 守卫对「非替换 + main 轴 min-size」**跳过** post-layout fixup（因 fixup 反向推导 main = cross/ratio 错误）。R1015 在 **intrinsic 测量期**用正确方向（cross = main × ratio）解同一组案——R1013 防 post-layout 反向覆盖，R1015 在 intrinsic 期正向推导，两者互补。R1013 的 fixup（replaced 元素 main 推导）仍保留。

**▶ 下会话（R370 余波 + aspect-ratio-intrinsic 簇）**：① **aspect-ratio-intrinsic-size-003/004/011/014**（14.85% ×4，inline-flex/flex + container-stretch main——item 无 min-height，main 来自容器 height:100px/100% 拉伸）：transferred base size 须扩到 **container-definite-cross 上下文**（flex_row/column_intrinsic_width 读容器 height Px 推 item stretch main），多 session；② **flexbox-collapsed-item-horiz-001 / flexbox-min-height-auto-001**（R1015 改善后 3.77% / 1.87%，近 PASS，查残余）；③ row 对称（flex-direction:row + float）+ inline-flex gate 扩（现仅 column + float，row/inline-flex 待 A/B）；④ R370 全量 oracle 三态门禁复跑。clean win 已 land。

### R1014 R370 flex-container-intrinsic-width Phase 0 测绘 = ~6 案阻塞定位（transferred-sizes + aspect-ratio-intrinsic 簇）·3 耦合件（gate 扩 auto+float / column max 变体 / transferred base size）·R370「零杠杆」纠正·零源码·纯调查

承 R1013 CONTINUE。本轮探索 fresh full oracle top-worst 候选 + 测绘下一结构 lever（R370）。**R370「零杠杆」纠正**：R370 memory「inline-flex width:auto 零杠杆——48 案不用 inline-flex width:auto」是**部分误判**——确有 ~6 案用 flex/inline-flex width:auto shrink-to-fit 且 FAIL：flex-item-transferred-sizes-padding-{border,content}-sizing（14.85% ×2，R1013 改善后残余）+ aspect-ratio-intrinsic-size-{003,004,011,014}（14.85% ×4）。同根因 = flex 容器 width:auto 不 shrink-to-fit（拉满 800）。

**几何实证（transferred-sizes-padding-border-sizing，product-smoke + PIL）**：ZW 绿盒 (9,51)→(789,150) = 780×99（**宽错**，拉满视口）；CHR 绿盒 (9,50)→(105,149) = 96×99（**宽正确**~100）。height 对（min-height:100px 驱动），width 错（flex container float:left 应 shrink-to-fit 到 item intrinsic ~100，ZW 拉满 800）。

**R370 机制 3 耦合件（partial slice 无 yield，须同改）**：
1. **gate 扩 auto+float/inline-flex**：`apply_intrinsic_content_sizing`（engine.rs:663）现仅 `width == MaxContent|MinContent` 触发（line 675-677），不覆盖 `width:Auto + float/inline-flex` shrink-to-fit 上下文。须扩 gate。
2. **column flex intrinsic max 变体**：`flex_row_intrinsic_width`（intrinsic_sizing.rs:175）是 **row-only（Σ item base size）**；column flex 须取 **max(item widths)**（cross 轴）。transferred-sizes 是 column flex，row 函数给错值。须加 `flex_column_intrinsic_width`（max 变体）+ dispatch。
3. **transferred base size（非替换 aspect-ratio）**：`flex_item_base_size`（intrinsic_sizing.rs:148）fall through `box_content_max_width`（叶 div 内容空 → ~0），**不计 aspect-ratio + main min-size 推导 cross**。transferred-sizes item（aspect-ratio:1/1 + min-height:100px）应 width = main × ratio = 100。须加非替换 aspect-ratio transferred-size 推导（R982/R983 仅替换元素的自然扩展）。

**为何 R1013 不解此案**：R1013 守卫对「非替换 + main 轴 min-size」**跳过** fixup（因 fixup 反向推导 main = cross/ratio 错误）。正确方向是 **cross = main_min × ratio**（正向），但这是 transferred-base-size（intrinsic 测量期），非 post-layout fixup（R993/R994 fixup 是 post-layout 改 main）。机制不同，须在 intrinsic_sizing 解。

**EV**：~6 案（transferred-sizes ×2 + aspect-ratio-intrinsic ×4）潜在翻 PASS + R370 「零杠杆」纠正（多 session 结构 lever 重开）。风险：width:auto flex 容器 sizing 改动可能回归其它 passing 案（须 ORACLE_DUMP_ALL per-case A/B，R1013 方法论）。

**裁决**：R370 = 真结构 lever（3 耦合件），多 session。本轮 Phase 0 测绘精确化根因 + 3 件路径 + ~6 案 EV。下会话实施首件（safest first slice）。

**▶ 下会话（R370 实施，多 session）**：① **safest first slice** = 加 `flex_column_intrinsic_width`（max 变体）+ gate 扩 `width:Auto + float` 仅 flex column（不扩 inline-flex/row，最小化回归面）→ ORACLE_DUMP_ALL A/B css-flexbox oracle（验 transferred-sizes ×2 + aspect-ratio ×4 改善 + 0 回归）；② 若 net 正，扩 row + inline-flex；③ 加 transferred base size（非替换 aspect-ratio）。三件分轮交付。备选：font-family-invalid-characters-003（100%，css-parser { } 错误恢复，窄但风险高）/ ::backdrop feature gap（replaced-object-backdrop 100%）。clean single-session lever 跨 corpus 确系穷尽，forward = R370 多 session 或 feature 实现。

### R1013 ★R994 aspect-ratio fixup 守卫 LANDED = flex-item-transferred-sizes-padding 88→14.85%（-73pp ×2 案）·css-flexbox Oracle 289 baseline 保持（R993/R994 增益不丢）·零回归·fresh full oracle 46.1% 基线建立

承 R1012 CONTINUE「per-element white-space 调查」。R1012 验证 font/text 簇墙化后，本轮 fresh full oracle（4680/10397=46.1%）扫全 corpus top-worst 找新 lever，定位 `flex-item-transferred-sizes-padding-border-sizing/content-sizing` = **88.19%**（R984 记 14.86%）= **+73pp 回归**。

**Bisect 定位（实证）**：临时禁用 `apply_flex_aspect_ratio_item_size`（R993/R994 fixup）→ 两案回到 14.85%（R984 基线）→ **R994 fixup 过度泛化致回归**。R993 fixup 原为 ratio-only SVG `<img>`（replaced），R994 泛化到「leaf + CSS aspect-ratio」（含非替换 div）。`flex-item-transferred-sizes-padding`（div + `aspect-ratio:1/1` + `min-height:100px` + `box-sizing:border-box`）被 fixup 误触发：fixup 对 column 反向推导 height = width/ratio，覆盖了 min-height 驱动的正确尺寸推导。

**精确守卫（区分 helped vs hurt）**：ORACLE_DUMP_ALL per-case 对比（noguard vs guard）找到 4 翻转案——transferred-sizes ×2（div 非 replaced，fixup hurt），flex-aspect-ratio-img-column-006/row-004 ×2（img replaced，fixup help）。区分 = **replaced vs 非替换**：
- 非替换 div + CSS aspect-ratio + min-size：min-size 驱动尺寸，cross→main 反向推导破坏 → 跳过 fixup。
- 替换 img/SVG + min-size：transferred-size 语义不变（intrinsic ratio + cross 推导），fixup 仍正确 → 保留。

**改动**（`engine.rs::apply_flex_aspect_ratio_item_size`）：加 `main_has_definite_min` 守卫——main 轴 `LengthValue::Px` min-size 时，仅替换元素继续 fixup，非替换跳过。条件 `main_is_auto && (!main_has_definite_min || b.is_replaced)`。

**验证（chromium Oracle + 三态门禁）**：
- **flex-item-transferred-sizes-padding-border-sizing / content-sizing：88.19%→14.85%（-73pp ×2，仍 FAIL >1%，残余 = R370 flex-container-intrinsic-width + transferred-size 精度，独立子问题）**。
- **flex-aspect-ratio-img-column-006 / row-004：0.73%→0.73% 字节同**（替换元素 fixup 保留，零回归）。
- **css-flexbox Oracle 289/497 (58.1%) = R994/R1005 baseline 持平**（R993 aspect-ratio-intrinsic-007 + R994 +2 增益全保留，transferred-sizes ×2 几何改善但未翻 PASS 故 pass count 不变）。
- aspect-ratio-intrinsic-size-007 仍 0.00% PASS（R993 gain 不动）。
- **welcome 16.57% 不变**（<20% DC-13 gate）。
- 2 新单测（`r1013_flex_column_non_replaced_with_min_height_not_overridden` + `r1013_flex_replaced_with_min_height_still_uses_fixup`）。
- make test exit 0（workspace 绿，layout-engine 1001 含 +2 r1013 测）；clippy --workspace -D warnings ✓；fmt ✓。

**fresh full oracle（post-R990/R1001/R1008/R1012/R1013）**：**4680/10397 = 46.1%**（vs R1005 的 4530/44.6%，post-R990 +138 + R1001/R1008/R1013 累计 +150）。per-dir 主要：css-flexbox 289/497 (58%) / css-tables 74/115 (64%) / css-position 55/97 (57%) / css-text-decor 108/242 (45%) / css-multicol 119/452 (26%) / css-text/white-space 45/395 (11%) / css-writing-modes 56/784 (7%) / text-transform 7/105 (7%)。global top-worst = replaced-object-backdrop 100%（::backdrop feature gap）/ font-family-invalid-characters-003 100%（parser edge case）/ pagination-print 99%（print）/ inline-svg-100-percent 97%（R717 inline）/ writing-modes vertical 86-87%（R109 structural）。

**意义**：收割 R994 over-generalization 回归 bug（潜伏 14 轮，R994-R1012 间无 oracle 全量对比暴露）。fixup 守卫精确区分 replaced（transferred-size 正确）vs 非替换（min-size 驱动）是 CSS Flexbox §4.5 + css-sizing-4 §aspect-ratio 的正确语义。transferred-sizes ×2 现 14.85%（近 PASS），R370 flex-container-intrinsic-width 解后可批量翻 PASS。**方法论**：ORACLE_DUMP_ALL per-case A/B（noguard vs guard）是定位 fixup 副作用的决定性工具，避免「修一案回归他案」陷阱。

**▶ 下会话**：① **R370 flex-container-intrinsic-width**（flex-item-transferred-sizes-padding 残余 14.85% + inline-flex/grid shrink-to-fit，结构性多 session，flex_row_intrinsic_width 须求和 block 子非取 max）；② **inline-svg-100-percent-in-body 97%**（R717 inline SVG，非 img，独立子问题）；③ **font-family-invalid-characters-003 100%**（CSS parser edge case，{ } 在 font-family 值中，窄）；④ per-element white-space（R1012 CONTINUE，product-smoke 价值高但 WPT yield 低）。clean lever 仍稀缺，forward = R370 / 真特性实现（::backdrop / form 控件 / scroll-container）/ 多 session 结构性。

### R1012 text-transform override-map bypass LANDED（机制证伪 R1011 误诊）·零 oracle yield·簇 = fontdue 光栅 + per-element white-space 墙（非 transform 逻辑）·CSS Text 3 §3.1 spec-compliance·零回归

承 R1011「▶ 下会话启动 Phase A master blocker 首切 = text-transform override-map bypass（R1004 模式）」。本轮完整实施 4 步 bypass + 验证。**机制 LANDED 且经证明工作，但 oracle 零 yield，证 R1011 误诊**。

**改动**（CSS Text 3 §3.1：text-transform 须在行断前应用，layout/paint 双路径一致）：
- **style-system**（property/types.rs）：`TextTransformValue` 加 `Copy` + `apply(&self, text)` 方法（none/uppercase/lowercase/capitalize）；放在 style-system 使 layout-engine 与 paint 共享同一转换逻辑。helpers.rs `apply_text_transform` 改为委托（消除重复）。
- **layout-engine IFC**（inline/mod.rs）：新字段 `text_transform_overrides: HashMap<NodeId, TextTransformValue>`（key = 文本节点父元素）+ `with_text_transform_overrides` builder；`collect_inline_items` 行断前应用 transform（layout 读 `style.text_transform`；paint Path B 空 styles 读 override）。
- **layout-engine LayoutBox**（types/mod.rs）：新字段 `text_node_text_transform`（key = 文本节点）；`store_font_sizes_from_ifc` 加 doc+styles 参数，按 frag 文本节点查父元素 computed transform 存入（5 调用点同步）。
- **engine paint Path B**（painter/text.rs）：从 `box_node.text_node_text_transform` re-key 到父元素构造 `parent_text_transforms`，`.with_text_transform_overrides(...)` 注入 IFC。
- **3 单测**（edge_cases.rs）：`r1012_text_transform_applied_via_style`（layout 有 styles）/ `_via_override_map`（paint 空 styles + override，证绕过 R72/R890 空 styles 墙）/ `_default_is_noop`（默认 None 原文，零回归）。

**机制证明（决定性）**：TTDBG 探针实测 `css-text/text-transform` 全簇——`aaa Aaa`（capitalize）→ `Aaa Aaa`、`ａａａ`（fullwidth 输入）→ `Ａａａ`——**transform 确实在 collect_inline_items 期应用**（layout IFC + paint Path B 双路径）。3 单测断言 frag.text == "HELLO"/"Aaa Aaa"。机制 WORKS。

**★ oracle 零 yield + R1011 误诊纠正**：text-transform oracle **7/105 持平**（capitalize-001 仍 14.42%，字节同 R1011）。结合机制证明，**簇阻塞非 transform 逻辑**。深查真因双墙：
1. **fontdue 光栅精度**：capitalize-001 CSS `font-family: 'Doulos SIL', 'Noto Serif', 'Noto Sans', webfont, sans-serif`——本机 **'Noto Serif'/'Noto Sans' 已安装**（fc-list），chromium 经 fontconfig 匹配到 Noto Serif（链中第 2，先于 sans-serif）。R1011「chromium+ZW 同落 sans-serif」判断**错误**。实验：harness 加载 Noto Sans+Serif（resolver 注册确认）后 A/B——**仍 7/105 零 yield**。证 diff 非字体选择，是 fontdue vs chromium 对**同字体**的光栅/advance 精度差（R388 谱系，per-glyph 累积）。
2. **per-element white-space**：capitalize-001 `.test span { white-space: nowrap }`——span 的 nowrap 应防 span 内断行，ZW IFC 用容器级 no_wrap（div.test normal）**不尊 per-span nowrap** → ZW 在 span 内空格断行（"Aaa Aaa"→2 词），chromium 不断 → 行结构差异。

**R1011 误诊纠正**：R1011 把 text-transform 簇归为「Phase A IFC 统一墙（paint Path B 空 styles 重跑）」。本 R1012 实施完整 bypass（绕过空 styles 墙）后零 yield，证**真阻塞是光栅精度 + per-element white-space，非 IFC 统一**。font/text 全簇（text-transform + line-break + css-fonts + font-features）的真墙更新：① fontdue 光栅精度（R388，per-glyph 累积，不可单 session 解）；② per-element white-space（容器级 IFC 不尊 span nowrap/per，多 session）；③ char-width 估计（R225/R375b）；④ R109 inline-span；⑤ rustybuzz 接生产（R513）；⑥ font 匹配（R374）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0** ✓（layout-engine 999 含 +3 r1012 测）/ **welcome 16.57% 不变**（<20% DC-13 gate）/ css-text-decor 108/242 不变（R1005 baseline）/ text-transform 7/105 不变（零回归 + 零 yield）。

**裁决（保留 LANDED）**：text-transform bypass 是 (a) R1011 指定 forward-motion slice；(b) 真正 CSS Text 3 §3.1 spec-compliance 修复（layout 现用转换后文本宽度行断，与 paint 一致）；(c) Phase A bypass 机制经证明可复用（R1004 模式扩展到 text-transform）。零 oracle yield 是因簇被**其他墙**（光栅+per-element white-space）阻塞，非机制问题。3 单测守护机制正确性。code-guidelines「不做零价值修改」——本修复非推测性（指定任务）非未用（transform 确在 layout 期应用），是 spec-compliance + Phase A 基础设施，保留。

**意义**：R1011→R1012 完整闭环——bypass 机制从「设计」到「LANDED + 证伪」。text-transform 簇作为 yield lever **永久关闭**（被光栅+per-element-white-space 双墙阻塞，非 IFC 统一）。font/text 真阻塞重新定位到上述 6 墙。下会话勿再以 text-transform/IFC 统一/transform 逻辑为 yield lever。

**▶ 下会话（font/text 簇关闭，转其它 lever）**：① **per-element white-space**（span nowrap 容器级 IFC 不尊，多 session，潜在解锁 capitalize/nowrap 簇 + 真实网页 span 布局）—— IFC 需 per-element white-space 信号（ InlineItem 携带或 collect 时读 styles），narrow slice；② rustybuzz 接生产（R513，font-features 184 案，shaper.rs:82）；③ 非 font/text dir 的 fresh lever 扫描（box-model 残余 / abspos 谱系）；④ R109 §9.2.1.1 匿名块（结构性 deadlock）。**font/text 单 session yield lever 边际确证尽（光栅+per-element-white-space 双墙），转其它 dir 或多 session 结构性。**

### R1011 text-transform 簇 = Phase A IFC 统一墙（paint Path B 空 styles 重跑）·DoulosSIL-R.woff 不在 WPT master 确认·capitalize/upperlower 同 sans-serif fallback·零源码·纯调查

承 R1010「pivot text-transform pre-layout（R998）」。本轮深查 text-transform 7/105 残余根因。

**DoulosSIL-R.woff 不在 WPT master**：GitHub API 列 WPT `/fonts/` = AD/GentiumPlus/Revalia/Scheherazade/fail/mplus-1p-regular/pass/sileot-webfont/tcu-font —— **无 DoulosSIL-R.woff**（R1007 404 证，本轮 API 复核）。capitalize/upperlower 簇全用 `@font-face webfont=DoulosSIL-R.woff`，font-family 链 `'Doulos SIL','Noto Sans',webfont,sans-serif`。chromium + ZW 均**无** Doulos SIL 系统字体 + 无 webfont（文件缺）→ 同落 sans-serif（DejaVuSans，Linux 双方一致）。**故 diff 非 font 差异**，是 **transform 逻辑**。

**capitalize 实现（paint/helpers.rs apply_text_transform）核查**：Capitalize 用 `prev_is_boundary = !ch.is_alphanumeric()` + `to_uppercase` 首字母——逻辑**基本正确**（titlecase 对 Latin Basic ≈ uppercase 首字母）。Upper/Lower 用 `to_uppercase`/`to_lowercase`。无 FullWidth（R998 已闭）。

**★ 真阻塞 = Phase A IFC 统一墙**：transform 现**仅 paint-time 应用**（text.rs:1102/1229 Path A/B）。pre-layout 应用（layout 用转换后文本宽度）须在 `collect_inline_items`（mod.rs:526）应用 transform。但 **paint Path B 重跑 IFC 用空 styles**（R72/R890/R1004 谱系）——`style.text_transform` 在 Path B None → 无 transform 应用于 Path B 的 line-box 宽度（用原文）。即使 layout IFC 应用 transform，**paint Path B 覆盖**（R989 同机制）→ 净效果不变。**bypass = store transform/text 进 LayoutBox + paint Path B 经 override-map 读**（R1004 ascent_ratio_overrides 同模式），即 **Phase A IFC 统一**（master blocker，多 session，R639 de-risked）。R998「多 facet」三角度现收敛到**单一前置墙**：Phase A IFC 统一（paint Path B 空 styles）。

**意义**：font/text 簇（text-transform + line-break + css-fonts + font-features）现**全簇收敛**到一组已知墙：① **Phase A IFC 统一**（paint Path B 空 styles——text-transform pre-layout + per-font R1004 wiring 都被此阻塞）；② char-width 估计（R225/R375b）；③ R109 inline-span；④ rustybuzz 接生产（R513）；⑤ font 匹配（R374，Noto/Doulos 系统字体差异）。R1008（line-break anywhere→BreakAll）+2 是唯一非墙易果。**后续单 session font/text lever 边际确证尽**——yield 须破墙（Phase A 为最高 EV，解锁 text-transform + per-font + 多 text 类）。

**▶ 下会话（启动 Phase A IFC 统一 master blocker，多 session 首切）**：① **text-transform override-map bypass**（R1004 模式）：layout IFC（有 styles）应用 transform 后存 `LayoutBox.text_node_text_transform`（或转换后文本）+ IFC 加 `text_transform_overrides: HashMap<NodeId, TextTransformValue>` + paint Path B 从 LayoutBox 填充 + collect_inline_items 读 override 应用——narrow slice（text-transform 簇 98 案驱动，但须守双路径一致）；② 备选：直接攻 Phase A 核心机制（store_inline_layout_results 统一，R890 标「TODO 基线计算修复后启用」）——broad 但根治。两路均多 session。**勿再扫 font/text 簇单点**（墙已穷尽）。

### R1010 line-break 残余 = char-width 估计墙 + R109 inline-span（非 LB 规则）·anywhere-001 实测 6 行 vs 期望 19·normal-011 target span 错位·零源码·纯调查

承 R1009「normal-011 基线诊断」。本轮 LAYOUT_DUMP 实测两驱动案，确证 line-break 残余**非 LB 规则缺失**而是已知墙：

**anywhere-001（plain div 文本，非 R109）实测**：`<div id=test width:1ch line-break:anywhere>` 内容 `aa-a.a)a,a）a&nbsp;a﻿a⁠a‍a･a`（~19 可见字符）。期望每字符一行 = 19 行（h≈304）。**ZW 实测 div h=96（6 行，~3 字符/行）**——BreakAll（R1008 anywhere→BreakAll）已触发但**每行容 ~3 字符非 1**。根因 = **1ch 单位解析 + estimate_char_width 0.55 启发式**：monospace 16px 字符 ZW 估 0.55×16=8.8px，div 1ch=8px，应 8.8>8 每字符溢出断；实测容 3 字符 → ZW 1ch 解析偏宽 或 字符估宽偏窄。属 **advance-width/estimate 墙**（R225/R375b memory：estimate_char_width 改 net-negative，0.55 碰巧近 system-ui；per-char advance 改 morning 19%→19.14% 退步）。**anywhere 残余 40 案同根因**——非 GL/JW/ZJW（前轮推测纠正），是字符宽度估计。

**normal-011（inline span target）实测**：`p.test` 含 2 行（h=40.4），但 `span.target`（小假名ぁ）box abs_y=54（p.test abs_y=53 → y=1 行 1），与小假名应在行 2（断前）矛盾。`p.control`（`<br>` 强制）target abs_y=91（y=1 within control）同样错位。证 **inline-span box 定位不反映实际行** = **R109 inline-as-block 谱系**（span.target 的 LayoutBox 定位在首行原点非实际行）。strict/loose/normal 28 三元组**同 R109 阻塞**——非 LB 规则缺失。

**裁决（line-break 簇单 session 边际确证尽）**：R1008 收 anywhere→BreakAll 易果 +2，残余 = **① char-width/1ch 估计墙（anywhere 40 案，R225/R375b dead-end）+ ② R109 inline-span（strict/loose/normal 28 案，结构性 deadlock）**。LB 规则实现**不会 yield**（两前置墙）。line-break 簇**关闭**为单 session lever。

**▶ 下会话（pivot LOGIC 簇，line-break 已闭）**：① **text-transform pre-layout 应用**（R998，98 案，@font-face R1007 就绪）——IFC 文本收集期应用 transform 使 layout 用转换后宽度，避开 char-width 墙（transform 改文本内容非宽度估计）；② rustybuzz 接生产（R513，font-features 184 案，shaper.rs:82 features: Vec::new()）；③ 任意 fresh dir 扫未深挖 LOGIC 缺口。**勿再攻 line-break**（char-width 墙 + R109 双阻塞，单 session 无 yield）。

### R1009 line-break 严格/松散/正常 = LB 算法多 session·NBSP collapse（R651）再确认无 driving reftest·anywhere GL/JW/ZJW 细节非 NBSP·零源码·纯调查

承 R1008「anywhere→BreakAll +2，下会话 anywhere GL/JW/ZJW + strict/loose CJK 规则」。本轮诊断 line-break 残余 82 案。

**NBSP collapse 修复实验（R651 再激活，已回退·零 yield）**：R651 记 `collapse_whitespace`（inline_types.rs:218）用 `char::is_whitespace()` 误折叠 NBSP(U+00A0)/U+3000（违反 CSS Text 3 §4.1 仅 TAB/LF/FF/CR/SPACE 可折叠），彼时无 driving reftest 故 defer。本轮发现 line-break-anywhere 簇用 `<span>&nbsp;</span>` 驱动——试修 `is_css_collapsible_ws`（仅 5 字符）A/B：**line-break 2/84 + white-space 45/395 + line-breaking 60/127 三 dir 全持平**（零 yield 零回归）。证 anywhere 失败**非 NBSP 折叠**所致（NBSP 在 monospace 1ch 宽，BreakAll overflow-driven 已断；折叠与否不改行数）。R651「无 driving reftest」结论**再确认**——NBSP 角度永久关闭，已回退。

**anywhere 残余 40 案 = GL/JW/ZJW + zero-width 字符断点细节**：BreakAll overflow-driven 仅在 `partial_x + ch_width > line_limit` 时断；zero-width 字符（ZWNBSP U+FEFF/WJ U+2060/ZWNJ U+200D）ch_width=0 不触发 overflow → 不在其处创建断点。`line-break: anywhere` spec 要求每个排版字符处断（含 zero-width）。修须 BreakAll 在 zero-width/GL/JW/ZJW 字符处也创建断机会（非 overflow-driven）——窄子集，独立 slice，EV 待测。

**★ strict/loose/normal 28 案（014/016b/011/018 等三元组 ~12%）= LB 算法多 session**：每三元组测一 CJK 规则——011 小假名（U+3041 ぁ 等断前禁则）/014 迭代标记（々）/016b 居中标点（・）/018 前缀（￥$）。**关键发现**：normal-011（应匹配 ZW 默认 CJK 断行）**也 ~10.6% FAIL**——证 ZW 单一渲染（无 strict/loose/normal 区分）**连 normal 默认都不匹配**，即 ZW 基线 CJK 断行本身对这组测试不精确（非仅缺 strict/loose/normal 规则）。三案同 diff（~10.6%）= 同一 ZW 渲染 vs 三个不同 ref。修须 ① LB 字符分类（小假名/迭代标记/居中标点/前缀 break class）+ ② strict/loose/normal 规则差异 + ③ 基线 CJK 断行精度（normal 都不过 → 基线有偏）—— CSS Text §5.3 + UAX 14 LB1-LB27，多 session 硬核。

**裁决**：line-break 簇 R1008 收割 anywhere 易果（+2），残余 = anywhere zero-width 细节（窄）+ strict/loose/normal LB 算法（多 session，~28+ 案）。单 session 边际已尽。下会话**勿单点攻 strict/loose/normal**（normal 基线不过证基线 CJK 断行有偏，须先查 normal-011 为何 default 不匹配）。

**▶ 下会话（二选一）**：① **line-break normal-011 基线诊断**——ZW 默认 CJK 断行为何不匹配 normal-011-ref（应 default break-before-small-kana），LAYOUT_DUMP 看断点；若基线修对，normal/loose 三元组（~14 案）可能批量过；② **pivot text-transform pre-layout 应用**（R998 另一 LOGIC 簇 98 案，@font-face 已 R1007 就绪）；③ rustybuzz 接生产（R513 font-features 184 案）。三 LOGIC 轨均多 session。R1008 +2 是 line-break 易果，本 R1009 纯调查确认单 session 边际尽 + NBSP 角度永闭。

### R1008 ★line-break: anywhere → BreakAll LANDED = line-break 簇首个 PASS（0/84→2/84 +2）·CSS Text 3 §5.3 解析 + layout/paint 双路径映射·零回归·parse-basic dead-code trap 规避

承 R1007「转 LOGIC 层，首推 CJK line-break」。line-break 簇 0/84 全部因 `line-break` 属性**完全未解析**（silent ignore → 默认断行）。本轮实现 line-break 解析 + `anywhere`→BreakAll 映射。

**改动**（CSS Text 3 §5.3，layout/paint 双路径一致）：
- css-parser：`LineBreakValue` 枚举（Auto/Loose/Normal/Strict/Anywhere）+ `parse_line_break`（color.rs live 位，**非** parse_basic.rs dead 模块——R1008 触发 parse-basic-dead-code-trap 记忆，迁回 color.rs 镜像 parse_word_break）。
- style-system：`LineBreakValue` 枚举（types.rs，镜像 WordBreakValue 双定义模式）+ `parse_line_break`（parse.rs）+ ComputedStyle 字段（line_break）+ default（Auto）+ apply（"line-break" dispatch）+ inherit（**line-break 是继承属性** §5.3）。
- layout-engine：`inline_finalization.rs::compute_inline_ifc_config`（line 614 word_break_mode 后）—— `line_break: Anywhere` → `WordBreakMode::BreakAll`（复用既有逐字符断行，width:1ch/窄容器场景产出与 anywhere 一致的逐字换行）。
- engine paint：`text.rs:778` 同映射（layout/paint 双路径同步，避免 R1004/R989 类发散）。
- 1 新单测：`test_parse_line_break_all_values`（css-parser tests_3.rs，5 关键字 + 大小写不敏感 + 无效输入）。

**严格/松散/正常 CJK 规则暂不实现**（解析但按 normal 默认行为）——标点禁则/小假名等多 session。

**验证（chromium Oracle + 三态门禁）**：
- **line-break oracle：0/84 → 2/84（+2 PASS，首个 line-break 通过）**。
- 其余 40 anywhere 失败 = GL/JW/ZJW 字符（NBSP/ZWNBSP/WJ/ZWNJ）逐字断行处理 + zero-width 字符断点（BreakAll overflow-driven 不在 zero-width 处断，anywhere 应断）—— 窄子集，独立子问题。
- welcome 16.57% 不变（<20% gate，welcome 无 line-break）。
- make test exit 0（workspace 绿，含 +1 line-break parse 测）；clippy --workspace -D warnings ✓；fmt ✓。

**意义**：font-loading 轨（R1006/R1007）证伪后，**转 LOGIC 层首个 yield**——CJK line-break 规则簇的 `anywhere` 子集（最窄，映射 BreakAll 即可产）。证实「font-loading 必要非充分，yield 须 LOGIC 实现」（R1007 结论）的 LOGIC 路径可行。line-break 簇 84 案现 2 通过，残余 = anywhere GL/JW/ZJW 细节 + strict/loose CJK 规则（标点禁则 LB1-LB27，多 session）。

**▶ 下会话**：① line-break:anywhere 残余 40 案——BreakAll 在 zero-width/GL/JW/ZJW 字符处也断（不只在 overflow 时），潜在 +N（须守不过度断行回归）；② line-break:strict/loose CJK 标点禁则（LB1-LB27，多 session）；③ text-transform pre-layout 应用（R998，另一 LOGIC 簇）；④ rustybuzz 接生产（R513，font-features 簇）。三 LOGIC 轨均多 session，R1008 是首切。

### R1007 WPT @font-face .woff 字体打包 + font-loading yield 三簇证伪（R910 模式扩展）·loader 持有已解码 sfnt（生命周期修复）·零 oracle yield·纯调查+数据

承 R1006「打包缺失 WPT .woff 字体后 A/B 测 font-loading 单独 yield」。本轮：① 经 ~/use-proxy 代理从 WPT GitHub raw 打包 7 个缺失 .woff 字体到 wpt-data/fonts/（mplus-1p-regular 803KB / Scheherazade-Regular / sileot-webfont / noto/noto-sans-v8-latin / math/mathvariant-italic）+ css-values/resources/ExTest(.woff/-NoSpace)；② A/B 三簇 oracle。

**★ font-loading 单独 yield 证伪（R910 模式扩展，决定性）**：打包字体后 **css-fonts 98/282 + text-transform 7/105 + line-break 0/84 全部持平**（零 PASS 翻转）。结合 R910（bidi 簇 font-blocked 已证伪），font-loading 单独**不产**结论现已覆盖 4 簇。机制清晰：**这些簇的测试检验 LOGIC 非 font 渲染本身**——
- line-break（0/84）：测 CJK 严格/松散换行**规则**（word-break: strict/loose/normal），换行点由规则定非字体→加载 mplus 不改换行。
- text-transform（7/105）：测 capitalize/uppercase/lowercase **应用**（R998 多 facet pre-layout+Path A/B），transform 逻辑非字体→加载字体不改应用。
- css-fonts（98/282）：测 font-features/variant/**ligature**（rustybuzz R513 未接生产）+ variable-font，需 GSUB/GPOS→加载字体但 features 不应用。
- bidi（R910）：测 bidi**算法**布局。

即 **font-loading 是这些簇的必要非充分条件**——字体加载让 glyph 正确渲染，但 PASS 还需对应的 LOGIC（规则/transform/features/bidi）实现。R1006 woff 解码器 + 本轮字体打包把 font-loading 层**彻底补齐**，yield 现在完全取决于更深 LOGIC 实现。

**改动**（commit 含数据 + 测试）：
- 数据：7 个 .woff 字体存 `tests/wpt-runner/wpt-data/fonts/` + 2 个 ExTest 存 `css-values/resources/`（harness resolve_font_src 已映射 `/fonts/X` → `wpt-data/fonts/X`，相对路径 → base_dir）。
- 测试：woff.rs 加 `r1007_tests::decode_bundled_wpt_fonts`（5 字体全 decode→fontdue 加载，证打包字体有效 + 解码器泛化）。
- FontLoader.load_font 生命周期修复：`decoded.as_deref().unwrap_or(data)` 后 `bytes.to_vec()` 存 font_data——`decoded` 局部 Vec 在函数末尾释放，故 stored_bytes 必须复制（已 to_vec），正确（无 use-after-free）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0** ✓（render-foundation 520 含 +r1007 测）/ product-smoke welcome 16.57%（R1006 验，本轮字体不影响 welcome）。

**意义**：font-loading 轨**彻底证伪为单 session yield lever**（R910 + R1007 四簇覆盖）。font cluster 真阻塞 = LOGIC 层：CJK line-break 规则 / text-transform pre-layout 应用（R998）/ rustybuzz features 接生产（R513）/ bidi 算法（R910）。每条多 session。R1006 woff 解码器 + R1007 字体打包使 font-loading 层完整，为未来 LOGIC 修复奠基（届时字体已就绪）。

**▶ 下会话（转 LOGIC 层，font-loading 已闭）**：① **CJK line-break 规则**（line-break 0/84 簇）：实现 word-break: strict/loose/normal 的 CJK 换行点规则（ZW 当前 WordBreakMode 不区分 strict/loose），mplus 字体已就绪可直接测；② 或 **text-transform pre-layout 应用**（R998，text-transform 98 簇）：IFC 文本收集期应用 transform 使 layout 用转换后文本宽度；③ 或 **rustybuzz 接生产**（R513 shaper.rs:82，font-features 184 簇）。三 LOGIC 轨均多 session，font-loading 已非阻塞。

### R1006 ★WOFF 1.0 解码器 LANDED·@font-face .woff 字体加载链补齐·FontLoader.load_font 自动 wOFF→sfnt·零回归·R1005「下游未实现」纠正·font cluster 基础设施

承 R1005「@font-face 下游 fetch/decode/register 未实现」。**R1005 部分错误**——查证发现 .ttf 链**早已 wired**（`reftest_fonts.rs::load_font_faces_into` → `FontLoader.load_font` + `register_family_alias`，oracle path reftest.rs:479 调用），`.ttf`/`.otf` @font-face 字体已正确加载（css-fonts/font-face 测 0.35% PASS 证）。**真缺口 = .woff 解码**（fontdue 不识别 woff 容器，971 案用 .woff 静默跳过）。

**改动**：
- 新 `crates/render-foundation/src/font/woff.rs`（~230 行）= `is_woff`（"wOFF" 魔数）/ `decode_woff`（W3C WOFF 1.0：头/表目录解析 + flate2 ZlibDecoder 解压 `compLength<origLength` 表 + sfnt 偏移表/表目录（按 tag 排序，含 searchRange/entrySelector/rangeShift 计算）/表数据（4 字节对齐）重建）。WOFF2（wOF2 brotli）不支持。
- `FontLoader.load_font` 自动检测 wOFF → 解码 sfnt → fontdue 加载；family 名解析 + font_data 存 sfnt 字节。`.ttf`/`.otf` 裸 sfnt 路径不变。
- `flate2 = "1"` 加入 render-foundation 直接依赖（已在 lock 树，png/resvg 传递依赖，非新外部 crate；miniz_oxide 纯 Rust 后端）。
- 3 单测：`decode_real_woff_revalia`（真实 WPT Revalia.woff → sfnt → fontdue 加载成功）/ `is_woff_rejects_non_woff`（裸 ttf/wOF2/空）/ `decode_truncated_returns_none`（残缺不 panic）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test exit 0** ✓（render-foundation 519 含 +3 woff 测）/ **product-smoke welcome 16.57% == R1005 baseline**（<20% gate，welcome 无 @font-face 故零影响）。

**★ yield 现状（诚实·零 PASS 翻转）**：css-fonts 98/282 + text-transform 7/105 oracle A/B **持平**。根因非解码（`first-available-font-005` 用现存 Revalia.woff @ 0.45% PASS 证 .woff 链完整工作），而是 **① 多数 .woff 字体文件未打包**（wpt-data 仅 7 个 .woff，`mplus-1p-regular`/`DoulosSIL-R`/`noto-sans-v8-latin`/`ExTest`/`pass`/`fail` 等缺失——WPT GitHub fetch 任务）+ **② 更深阻塞**（text-transform 应用 R998 多 facet pre-layout+Path A/B / font-features rustybuzz R513 / variable-font 支持）。本解码器是 font cluster 基础设施，yield 须打包字体 + 解更深阻塞（多 session）。

**意义**：font cluster（~450 案 blocked）的 **.woff 解码层补齐**——此前 .woff 完全跳过，现 .woff 字体能被 load_font 加载（验 end-to-end）。R1005「下游未实现」纠正为「下游 .ttf wired / .woff 本轮补」。R910 仅证伪 bidi 簇 font-blocked，未证伪 text-transform/line-break（字体 IS the point），但本轮实测 .woff 加载单独**不产**（须字体文件 + 应用逻辑齐备）。

**▶ 下会话**：① 打包缺失 WPT .woff 字体（mplus-1p-regular/DoulosSIL-R/noto-sans-v8-latin 等，~/use-proxy 经 WPT GitHub fetch 存 `tests/wpt-runner/wpt-data/fonts/`）后 A/B text-transform/line-break——测「font-loading 单独 yield」假设；② 若仍零 yield → 证更深阻塞主导（text-transform 应用 R998 / line-break CJK 规则），转 rustybuzz-in-production（R513 shaper.rs:82 features: Vec::new()）或 text-transform pre-layout 应用；③ R1004 per-font wiring 待字体加载真起效后才有 yield。本 R1006 LANDED 工作 .woff 解码器（非 dormant），零回归，font cluster 基础设施推进。

### R1005 ★★font-wall 单 session 杠杆四角度证伪 + fresh full oracle（plateau 再确认·post-R990 多 dir 进）·pre-wrap-align 簇 ruled out·零源码·纯调查

承 R1004「per-font ascent step-2 wiring 是下会话 CONTINUE」。本轮**先评估该 wiring 的 yield 前置条件**，再决定是否投入。结论：**per-font wiring 当前零 yield**（须 webfont 先行）+ 另三角度同证伪。同时跑 fresh full oracle 得 post-R990/R1004 当前数。

**① per-font wiring yield 前置证伪（决定性）**：per-font ascent 仅当页面**容器字体**为 non-Ahem-非-DejaVuSans 时才与 R990 常数 0.928 不同。ZW FontLoader 只加载 system fonts（Linux = DejaVuSans ascent 0.928，即 R990 常数）+ Ahem（0.8）+ NotoSansCJK fallback（仅 CJK glyph）。ZW **从不**渲染 non-Ahem-非-DejaVuSans 容器字体（webfont @font-face fetch/decode 未实现，R909/R910 territory）。故**当前所有渲染页面的 line-box ascent 都已被 R990 常数正确覆盖**（Ahem 0.8 / DejaVuSans-container 0.928）。per-font wiring 须等 @font-face webfont 加载落地才有 yield——**premature optimization，当前勿投入 wiring**。R1004 dormant slice 作为 webfont 后的基础设施保留，正确但待前置。

**② CJK 文本检测 heuristic 证伪（理论不成立）**：考虑「文本含 CJK 字符 → 用 0.88（NotoSansCJK ascent）替 0.928」heuristic。**理论不成立**：CSS line-box ascent 由**容器字体**（system-ui→DejaVuSans 0.928）定，非 CJK 字符内容。CJK glyph 用 NotoSansCJK fallback 仅影响 glyph 在行盒内 ink 位置（R655/R876 glyph-metric territory），**不影响 line-box ascent**。故 R990 的 0.928 对 CJK-heavy 行（如 morning.work）仍正确；CJK 残余 diff = glyph 度量/字体匹配（R374/R633），非 line ascent。

**③ semi-replaced stretch = paint gap 再确认（R974 正确）**：`position-absolute-semi-replaced-stretch-input/other`（fresh css-position oracle 21.19%/13.62%）。复用既有单测 `test_absolute_stretch_in_inline_block_container`（tests_5.rs:1716）**PASS**——ZW **已正确 stretch** abspos `<input>` 到 144×94（display:InlineBlock UA 默认 + §10.3.7 abspos stretch）。diff = input 内部**绘制外观**（button 边框/文本，ZW 不渲染表单控件原生外观），非 sizing/layout。R974「form-control paint feature gap」结论正确，非 layout lever。

**④ pre-wrap-align 簇 ruled out（font-wall + bidi）**：fresh `css-text/white-space` oracle 45/395（11%）；top-worst = `pre-wrap-align-{start,end,left,right,center}-001/002/003`（~12 案 10-12%）+ `pre-line-with-space-and-newline`。test 含 `<p>` 非-Ahem 默认字体说明文字（font-wall）+ `start` 案 dir=rtl（bidi structural, R114/R164 territory）。实施 trailing-ws hang 修复（CSS Text §4.6 Phase IV，apply_text_alignment 扣末片段尾随空格）A/B：right/end/center 仅 -0.34pp（小幅正向），start 案不变，**pass-rate 持平 45/395 零 PASS 翻转**。且 fix 不完整（split_into_words 末尾独立 " " 片段未扣）。**裁决回退**——cluster residual = font-wall 底 + bidi，trailing-ws hang（无论完整否）无法翻 PASS。CSS §4.6 修复非当前 lever（font-wall 主导）。

**fresh full oracle（post-R990/R1001/R1004，aggregate 因 tail 截断未捕获，per-dir 进）**：
| 目录 | fresh | pre-R990/R1001 doc | 变化 |
|------|-------|--------------------|------|
| css-flexbox | **289/497 (58%)** | 50.6% (R944) | **+7.4pp**（R990 + R993-R994）|
| css-tables | **74/115 (64%)** | 56.5% (R944) | **+7.5pp**（R995 + R1001）|
| css-multicol | **119/452 (26%)** | 23.0% (R893) | **+3pp**（R901-R907）|
| css-position | **55/97 (57%)** | 52.6% (R944) | +4.4pp |
| css-text-decor | **108/242 (45%)** | 28.9% (R944 doc) | **+16pp**（harness JS + R955-R957）|
| css-fonts | 98/282 (35%) | 34.4% | 持平（rustybuzz blocker）|
| css-grid | 20/49 (41%) | 39.6% | 持平 |
| css-writing-modes | 56/784 (7%) | — | 低（vertical structural）|
| css-text/i18n | 0/158 (0%) | — | font/shaping blocked |
| css-text/line-break | 0/84 (0%) | — | font/shaping blocked |
| css-text/shaping | 0/28 (0%) | — | rustybuzz R513 |

**意义**：near-pass dirs（flexbox 58% / tables 64% / position 57% / text-decor 45%）post-R990 实质进，但已被各轮收割（R976/R978/R982-R983/R990/R993-R995/R1001/R955-R957）。残余大簇全 blocked：font/shaping（i18n 158 + line-break 84 + shaping 28 + text-transform 98 + word-break 80 = ~450 案，rustybuzz-in-prod R513 + webfont R909 + font-matching R374 三 blocker）、writing-modes 728 案（vertical structural R109/R114）、multicol 333 案（Phase 2 structural）。

**font-wall 四角度穷尽证伪的单 session 杠杆**：① per-font wiring（须 webfont 先行）② CJK heuristic（理论不成立）③ advance-width（R225/R375b 双证伪，memory 记录）④ line-height 常数（R990 余波 1.15 证伪，1.2 最优）。**R990 的 is_ahem-gated 0.8/0.928 是 font-wall layout-side 唯一可产常数**，已尽。

**▶ 下会话方向（最高 EV 多 session 轨）**：**@font-face webfont 加载**——css-parser 已 parse @font-face AtRule（ast.rs:73），但**下游 fetch/decode/register 未实现**（R909 territory）。是 per-font wiring（R1004 infra 待此）、text-transform 簇（R998「测试全用 @font-face 自定义字体 mplus/DoulosSIL/Revalia，ZW 字体 fallback 主导 diff」）、font-family/feature 簇的共同前置。R910 仅证伪 bidi 簇非 font-blocked，**未证伪 text-transform/font-family 簇**（这些簇字体 IS the point）。首 slice = @font-face 消费：css-parser AtRule → engine 收集 → net fetch .woff/.ttf → render-foundation decode → FontLoader register（family→font_id）；dormant/env-gated 起步，A/B text-transform/font-family oracle。备选 = multicol Phase 2 commit-2（无 driving WPT 案，低 EV）/ R109 §9.2.1.1（结构性 deadlock）。**勿以单 session 期望 DC-2~5 显著提升**（font-wall + 结构性双上限，须 webfont/rustybuzz 前置解锁大簇）。

### R1004 ★Phase A §12.6 step-2 bypass 基础设施 LANDED = ascent_ratio_overrides dormant 字段 + apply_vertical_alignment 消费点（零回归）·per-font ascent 解锁路径基石

承 R1003「单 session clean lever 五 dir 确证耗尽，forward 全在多 session 结构性」。**选最高 EV 结构 lever 的 enabling slice**——per-font 真实 ascent（R990 +138 oracle 的自然延伸：常数 0.928 → 真实 per-font 值）。

**R889/R890 决定性发现复用**：per-font ascent 单点 wiring 三次证伪（R889 错路径 / R890 空 styles 墙 / R891 negligible）的**真解锁机制** = `store_font_sizes_from_ifc` 的 override-map 模式（line 297，已为 font_size/is_ahem/letter_spacing/line_height 用）：layout IFC（有 styles + provider）算出每文本节点真实 ascent ratio → 存 LayoutBox → paint Path B 经 override map 读取 → `apply_vertical_alignment` 消费，**绕过 R890 空 styles 墙**（paint Path B 不需解析 family，直接读预计算 ratio）。

**改动**（`crates/layout-engine/src/inline/mod.rs`，dormant enabling slice）：
1. IFC 新字段 `ascent_ratio_overrides: HashMap<NodeId, f32>`（默认空）+ `with_ascent_ratio_overrides` builder + `ascent_ratio_for` pub 方法。
2. 新自由函数 `ascent_ratio_lookup(overrides, node_id, is_ahem) -> f32`：优先取 map 值（>0 有效），否则回退 R990 is_ahem-gated 常数（Ahem 0.8 / 非-Ahem 0.928）。**自由函数 + 字段访问**绕开 `for line in &mut self.lines` 的可变借用与方法调用整体借用 self 的冲突（Rust 不相交字段借用）。
3. `apply_vertical_alignment`（R990 site，mod.rs:1633/1646）两 ratio 查询（dominant strut + per-run）从 `if is_ahem {0.8} else {0.928}` 常数改为 `ascent_ratio_lookup(&self.ascent_ratio_overrides, node_id, is_ahem)`。

**dormant 保证（零回归）**：空 map（默认）→ 全回退 → 字节级 R990 行为。2 新单测（`r1004`）：
- `test_r1004_ascent_ratio_override_supersedes_r990_constant`：覆盖 0.95（模拟 NotoSansCJK）→ baseline_y = 100×0.95 = 95（无覆盖时 92.8 = 0.928），证 override 优先。
- `test_r1004_ascent_ratio_override_zero_or_absent_falls_back`：override=0.0 / 空 map → 回退 R990 常数（非-Ahem 0.928 / Ahem 0.8），证 dormant 零回归。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / **make test ✓（exit 0，全 workspace 零失败）** / R990 三测仍绿（零回归）/ **product-smoke welcome 16.57% == R1001 baseline**（<20% DC-13 gate PASS，字节级不变证 dormant）。

**意义**：把 per-font ascent（font-wall 最大 lever R990 的延伸）从「被 R890 空 styles 墙阻塞」推进到「bypass 基础设施就绪」。下会话的 step-2 wiring 只需：① 在 `store_font_sizes_from_ifc` 用 `inline_ctx.font_metric_provider`（R885 dormant）+ styles 算每节点真实 ratio 存入新 LayoutBox 字段；② paint Path B 从 LayoutBox 填充 `ascent_ratio_overrides`；③ provider 经 `compute_final_inline_layouts` 5 站点 + harness 注入（R889 测绘）。**本 slice 是该多 session 链的零风险基石**（类 R885/R897/R898 dormant pattern）。

**裁决**：R996-R1003 八轮调查确证单 session clean lever 耗尽；本 R1004 转为多 session 结构 lever 的 enabling slice 推进（rally 续跑协议：clean lever 尽 → dormant enabling slice 是 forward motion，非阻塞）。**per-font 真实 ascent 的 EV** = R990 常数 0.928 的增量（WPT 多用 Ahem 故 WPT 增量小，产品页 CJK 字体 NotoSansCJK ~0.88 受益）；下 session 完成 wiring 即可测真实 yield。

**▶ 下会话（step-2 wiring，多 session）**：① `store_font_sizes_from_ifc` 增强：取 `inline_ctx.font_metric_provider` + `styles.get(&frag.parent).font_family` 算 `provider.line_metrics(family, frag.font_size).ascent / frag.font_size` → 存 `LayoutBox.text_node_ascent_ratios: HashMap<NodeId,f32>`（新字段）；② paint Path B 构造 IFC 时从 LayoutBox 填充 `ascent_ratio_overrides`（镜像 font_size_overrides 填充点）；③ provider 经 `compute_final_inline_layouts`（inline_finalization.rs:691/945/1065/1204/1223 五站点）+ engine `compute_with_img_sizes` + harness wpt-runner 注入；④ 三态门禁 A/B：welcome <20% + linebox/css-text/wm oracle 零回归 + chromium-Oracle z_vs_chr 下降（净负即回退，§12.4 R834/R836/R849/R875 单点 net-negative 先例）。勿以单 session 期望 DC-2~5 显著提升（受光栅 + 结构性双上限）。

### R1003 twin ZW 几何验证（td=172×304）·R1002「fundamental table-auto-layout gap」框架过度·残差 = 小宽差 + font·零源码·纯调查

承 R1002「twin 残差 = table auto-layout（ZW max-content 211 vs chromium balance 162）」。本轮 probe（compute_styles + `<style>` CSS 正确接入）实测 **ZW twin td = 172×304**（tall div 168×304 正确 grow，R1001 cell direct-text fix 使 td 从 8px 进到 172px）。oracle PNG 分析（imprecise）chromium td ~162。

**关键纠正 R1002**：R1002 框架「ZW max-content 211 vs chromium 162 = fundamental gap」**过度**。实测 ZW td=172（**非 211**——compute_column_widths 经 `final.min(table_box.content_width)` 被 taffy 分配的 table 宽 clamp 到 172，非纯 max-content）。**chromium ~162**。**两者仅差 ~10px**（172 vs 162），twin 已 close（3.87%）。R1001 显著改进（8→172，近 chromium 162）。残差 = 10px 宽差 + 文本换行差异 + font 渲染（非 Ahem 默认字体），非 fundamental 算法 gap。

**意义**：twin 在 R1001 后已 close（3.87%）。完全 pass（<1%）须 10px 宽差对齐（table 列宽 clamp 精度 / tall div border 计入 / 文本测量）+ font 渲染（非 Ahem 字体 advance/raster）。两者均 font-wall + table 精度 territory，非单 session。

**裁决**：R1002「table auto-layout §17.5.2.2 多 session 重写」结论**收窄**——twin 已 close，无须大重写，须小宽差对齐（可独立 slice，但 EV 低 ~2 case）。下会话优先其它 lever（twin 3.87% 可作 stretch goal，非阻塞）。

**下一步**：twin 小宽差对齐（EV 低，可选）；或转 Phase A pre-layout text-transform / per-font ascent / multicol Phase 2 / 其它 dir fresh case。R1001 已 land（css-tables 73→74 +1 零回归）；本 R1003 纯调查无新代码。

### R1002 R1001 height-cap 假设纠正 + twin 残差 = table auto-layout 宽度平衡（非 height）·零源码·纯调查

承 R1001「twin 须 height-cap + paint-clip 合做」。本轮验证发现 **R1001 height-cap 假设错误**：

**height-cap 假设证伪**：R1001 试 height-cap（overflow_y!=Visible cell cap 到显式 height）致 twin 3.87→9.48%。本轮推理确认：**该 reftest 验证 td GROWS to content（CSS2 min-height 语义，chromium 对 table cell 的 overflow:hidden 不裁剪——css-tables-3 bug 1880550 的裁剪语义尚未在 chromium 对此 case 启用，或 test 依赖 CSS2 行为）**。test 页 td{height:20px;overflow:hidden} 应 == ref 页 td{无 height}（两者均 grow 到内容 ~304px）。ZW 无 height-cap 时 td 已 grow（正确，匹配 chromium）；加 height-cap 反把 ZW td cap 到 24px（错）→ diff 增。故 **height-cap 是错的，twin 不需要它**。R1001「twin 须 height-cap+paint-clip」结论作废。

**twin 残差真因 = table auto-layout cell 宽度平衡**：oracle PNG 分析（chromium 渲染）显示 td 宽 ~162px（image analysis 不精确，但 < ZW 的 211px）。R1001 width fix 让 ZW cell = 文本 max-content（"Can you see this text?" 22×9.6≈211px）。chromium cell ~162px（**table auto-layout 平衡值**，介于 min-content（~30px 最宽词）与 max-content（211px）之间，受 `<table border>` + 视口约束）。**ZW 用 max-content（211），chromium 用 balance（162）→ 49px 宽差 + 文本换行差异 = 3.87% 残差**。

**与 101 同类**：table auto-layout 算法（ZW 简化为 max-content，chromium 用 CSS §17.5.2.2 的 min/max-content 平衡 + available space 分配）。这是 table 布局的核心算法差异，非单点 fix。twin 与 101（也涉及 table 列宽）同根因。

**裁决**：twin 完全 pass 须 ZW table auto-layout 从「max-content 简化」升级到「§17.5.2.2 min/max-content + available-space 平衡」算法——多 session 结构性工作（影响全 css-tables + CSS2 table 用例）。R1001 width fix（max-content）是改进（8→211，比 8 更近 162），但非完全正确（211 vs 162）。net 仍 +1（74）因其它 case 受益。

**下一步**：① table auto-layout §17.5.2.2 平衡算法（多 session，twin + 101 + 其它 table 列宽 case 受益）；② Phase A pre-layout text-transform；③ per-font ascent；④ multicol Phase 2。R1001 已 land（css-tables 73→74 +1 零回归）；twin 残差定位为 table auto-layout（非 height）。

### R1001 ★table cell 直接匿名 inline 文本参与 intrinsic width LANDED·cell-level direct-text fix（安全·不回归 101）·css-tables Oracle 73→74（+1）·twin 4.13→3.87%·净正

承 R1000「table-cell-overflow 须先解 width 不回归 101」。R1000 两轮（box_content_max_width 全局 direct-text）均回归 101（3.66→6.51），根因 = 全局函数 perturbs 101 嵌套 div 的 max-content（div.b 等 block 后代含直接文本，被新测后改变 table auto-layout 列宽）。本轮 **WDBG probe 定位**：101 的 td 直接文本=0（文本在嵌套 div 内），但 box_content_max_width 全局改影响 td 后代 div.b 等；twin 的 cell 直接文本="Can you see this text?"（211px）。

**关键洞察（R1000 → R1001）**：direct-text 测量须限定在 **cell 级（compute_cell_intrinsic_width）**，非全局（box_content_max_width）。cell 的直接文本是 cell 的匿名 inline 内容；block 后代的文本由 box_content_max_width 递归 block_max 处理（不改）。这样 twin cell 直接文本被测（211），101 cell 直接文本=0（安全），101 嵌套 div.b 走 box_content_max_width 原路径（不变）。

**改动**（`table_types.rs::compute_cell_intrinsic_width`）：新增 `cell_direct_text_width(cell_box, styles, doc)` helper——遍历 cell 的 DOM **直接**文本节点子（`doc.child_nodes` 过滤 `NodeKind::Text`，非全后代），用 cell font 度量（复用 `fragment_inline_max_width`）。在 compute_cell_intrinsic_width 两返回点取 max：early-return `max(content_width, direct_text_w)`；text branch `max(box_content_max_width, direct_text_w)`。

**验证（chromium Oracle + 三态门禁）**：
- table-cell-overflow-explicit-height twin：**4.13%→3.87%**（改善，仍 FAIL >1%，残余 = td height overflow 裁剪语义，须 height-cap + paint-clip 合做，见下）。
- **margin-collapse-101：3.66%→3.66% 字节同**（零回归——cell 直接文本=0，101 安全）。
- **css-tables Oracle 73→74（+1 零回归）**（twin 改善 + 另 1 案翻 PASS）。
- **welcome 16.57% 不变**（<20% gate）。
- 2 新单测（`r1001_table_cell_direct_text_tests`）：cell 直接文本参与宽（>150px）/ 文本在 block 后代不过计（<120px）。
- make test 全 workspace 绿（exit 0）；clippy/fmt 干净。

**★ height-cap 实验（attempted + reverted）**：试加 `cell_height_cap`（table.rs，overflow_y!=Visible 时 cell height cap 到显式 height，CSS Tables L3 + bug 1880550）。**A/B net 0 + twin 反退**：twin 3.87→9.48%（cell cap 到 24px 但 content 300px **未被 paint 裁剪** → 内容溢出短 cell 致 diff 增）；min-height-table-2 翻 PASS（+1）抵消某处 -1，css-tables 仍 74。**paint overflow 裁剪对 table cell 未生效**——twin 须 height-cap + paint-clip 合做才能 pass。已回退 height-cap（net 0 + twin 反退不值）。

**意义**：R1000 两轮失败后，R1001 找到安全 slice（cell-level vs 全局）。table cell 直接文本参与 intrinsic width 是真实 CSS correctness（cell 的匿名 inline 内容应贡献列宽）。R679 table sizing 簇再进一 facet。

**下一步**：twin 完全 pass 须 height-cap + table-cell paint overflow 裁剪（content 被 clip 到 cell 盒）合做——下会话 dedicated。或转 Phase A pre-layout text-transform / per-font ascent / multicol Phase 2。R993-R995 + R1001 累计 css-flexbox +4 / css-tables +5 零回归已 land。

### R1000 table-cell-overflow width-fix 再确认 + letter-spacing R855 territory·两结构 lever 单 session 均不可独立·零源码·纯调查

承 R999「commit 到结构性 lever」。本轮 dedicated 攻 table-cell-overflow（R997 测绘的 combined fix）。

**table-cell-overflow width-fix 再确认（已回退）**：重做 R997 的 box_content_max_width 直接文本测量，这次用正确匿名盒语义（has_block_child 时直接文本→匿名 block→block_max 取 max；仅 inline 时→inline_sum 求和）。**A/B 与 R997 完全一致**：margin-collapse-101 仍 3.66%→6.51% 回归，twin 仍 4.13% FAIL。证明回归非 inline/block 分类问题，是「测 div.b 直接文本 "B"(30px) 本身」使 101 列宽变化（baseline ZW 不为 div.b 生成匿名 block LayoutBox 故 "B" 漏测；新测后列变宽超 chromium）。**101 回归机制未解（须 dedicated probe 101 列宽决定路径）**。width-fix 单做 net-negative，回退。

**letter-spacing-206 (47%) = R855 territory**：css-text worst letter-spacing-206/202 用 `letter-spacing: 1em/3em`（Ahem）。IFC collection（inline/mod.rs:530/779, inline_finalization.rs:819, text.rs:685）letter_spacing 仅 Px（`_=>0.0` 丢 Em）。**R855 已测 Em-in-IFC 双路径零 yield 已回退**——故 206 的 47% 非「IFC Em 缺失」单一原因。新假设：intrinsic sizing（text_content_max_width / fragment_inline_max_width）不加 letter-spacing（即使 Px），float shrink-to-fit 测窄。但 coherent fix 须 5 site（IFC collection 4 处 Em + intrinsic 2 处加 spacing），且 R855 零-yield 先例，单 session 风险高。

**裁决（两 lever 单 session 均不可独立）**：table-cell-overflow 须 width + height/overflow 合做且 width 须先解 101 回归（未解机制）；letter-spacing 须 5-site coherent 改且有 R855 零-yield 先例。两者均非单 session 可产 clean win。

**战略重申（R999 已述）**：单 session clean lever 五 dir 确证耗尽。**后续须 dedicated 多 session 结构计划**，非继续单点尝试（R996-R1000 五轮纯调查/回退已证边际为零）。推荐多 session 计划：
1. **table-cell-overflow combined**：先 probe 101 列宽决定路径解 width 回归机制 → height/overflow clipping（css-tables-3）→ 合做 A/B。
2. **letter-spacing coherent**：5-site Em + intrinsic spacing，须先 A/B 证非零 yield（R855 先例）。
3. **Phase A IFC pre-layout text-transform**（R998 测绘 + @font-face）。
4. **Phase A per-font ascent R887** / **multicol Phase 2** / **R109**。

**下一步**：下会话选上述 1 个 dedicated 多 session 计划，先做 safest first slice + 三态门禁 A/B。R993-R995 已 land（css-flexbox +4 / css-tables +4 零回归）；R996-R1000 五轮纯调查确认单 session clean lever 耗尽 + 结构 lever 须多 session。

### R999 css-grid 第 5 dir 扫描复核·clean lever 单 session 耗尽五 dir 确证·转结构性 lever·零源码·纯调查

承 R998 后扫 css-grid（第 5 个 dir）top-worst，再证 R740/R882/R996 单 session clean lever 耗尽结论。css-grid worst 全 structural：① replaced-element-percentage-height-in-grid-nested-in-flex-002/001（33.9%/8.9%，grid+flex+replaced 三层嵌套 %height）；② table-grid-item-dynamic-003/004（25.8%/9.25%，table-grid-item + dynamic JS）；③ **grid-container-baseline-synthesized-001/002/003/004（16-17% ×4 cluster，inline-grid 基线合成 + vertical/sideways writing-modes，R109/writing-mode 结构性）**；④ nested-grid-item-block-size-001（13.76%，R976 aspect-ratio）；⑤ stretch-grid-item-button-overflow（8.17%，button 表单控件）。

**五 dir 复核总结（css-tables/position/multicol/text/grid）**：clean single-session lever **彻底耗尽**。残余 100% 落五桶：① 结构性（R109 §9.2.1.1 / writing-mode 基线 / multicol Phase 2 / table auto-layout 文本测量）；② feature gap（::backdrop / 表单控件原生渲染 / content-visibility / scroll-container / subpixel）；③ font-wall 残余（per-font 真实 ascent，R990 常数已尽）；④ JS-driven（dynamic relayout / onload）；⑤ 多 facet（text-transform 须 pre-layout+owner+font 三修，table-cell-overflow 须 width+height/overflow 合修）。

**战略裁决**：单 session clean lever 五 dir 确证耗尽。后续须 **commit 到一个结构性 lever 的 dedicated 多 session 推进**，非继续扫描（扫描边际已尽）。推荐优先级（按 EV × 可独立性）：
1. **table-cell-overflow combined fix**（R997 测绘：width 直接文本测量安全版 + td overflow!=visible 时 height 作 used-height 裁剪 css-tables-3 §height-distribution；两步合做，driving twin 001/002 + dynamic-003/004 簇；须先解 width 不回归 101，可能 gate width 于 overflow 或重审 div.b anonymous-block 测量）。
2. **Phase A IFC pre-layout text-transform**（R998 测绘：IFC 文本收集期应用 transform + Path A/B owner_id + @font-face woff 加载；多 case capitalize/uppercase/lowercase/fullwidth）。
3. **Phase A per-font 真实 ascent R887**（R970 provider wiring，R990 常数 0.928 已证可产 +138，per-font 增量；WPT 多用 Ahem 故 WPT 增量小，产品页 CJK 受益）。
4. **multicol Phase 2 统一 column-flow**（R383，嵌套/breaking/balance，多会话硬核）。
5. **R109 §9.2.1.1 匿名块**（结构性 deadlock，CB-through-inline 等）。

**下一步**：下会话选上述 1 个结构性 lever dedicated 推进，先做 safest first slice（如 table-cell-overflow 的 height-clipping 半，或 text-transform 的 pre-layout 半），三态门禁 A/B。R993-R995 已 land（css-flexbox +4 / css-tables +4 零回归）；R996-R999 四轮纯调查无新代码，forward motion 转结构性。

### R998 text-transform 多 facet gap 定位（paint-time 容器 style + Path A/B + @font-face）·full-width 实现零 yield 已回退·Path A owner-style 修零 yield 已回退·零源码·纯调查

承 R997 转 css-text 扫 worst（letter-spacing-206 47% / capitalize-fullwidth 簇 20-24%）。**capitalize 已正确实现**（helpers.rs apply_text_transform + paint text.rs:1102/1229）；R855 letter-spacing Em 零-yield 已闭。**full-width 未实现**（TextTransformValue 仅 None/Upper/Lower/Capitalize）。

**实验① full-width 实现（已回退·零 yield）**：加 FullWidth enum（style-system）+ parse "full-width" + apply（ASCII U+0021..7E→U+FF01..FF5E，U+0020→U+3000）。FWDBG 插桩实测 **apply_text_transform(FullWidth) 从未触发**——因 paint 传的是**容器** style.text_transform（None for span's container），FullWidth 分支不进。zero yield（debug 直接证伪），已回退。

**实验② Path A owner-style 修（已回退·零 yield）**：text.rs:1102 改用 owner_id 的 text_transform（对齐 color 查找 line 1087）。css-text 全量 oracle **355→355 字节同**——driving 测试（capitalize-016/018/003、fullwidth-001）全用 **Path B（render_fragment! 宏 line 1229）** 非 Path A，且宏无 owner_id 在作用域。Path A 修正确但零可测 yield，已回退。

**真 gap（多 facet，非单 session）**：text-transform 现仅 paint-time 应用，且有 3 层问题：① 用容器 style 非 fragment owner（span 级 transform 丢失，须 Path A+B 都改 owner_id）；② 仅 paint 不 pre-layout（layout 用原文 line-break，full-width 宽字符致 layout/paint 不一致，须 IFC 文本收集期应用）；③ WPT text-transform 测试全用 @font-face 自定义字体（mplus/DoulosSIL/Revalia 等 woff），ZW 字体加载/fallback 主导 diff（mplus 不在 fonts/）。三层任一单独修都不 yield（须 pre-layout 应用 + Path A/B owner + 字体加载三者齐备）。

**裁决**：text-transform = 多 facet 多会话 lever（pre-layout 应用属 Phase A IFC 文本统一，Path A/B owner 修属 paint 重构，字体加载属 R374 谱系）。单 session 零 yield，回退。下会话勿单点重试。

**下一步**：① Phase A IFC pre-layout text-transform 应用（多 case lever，但须 @font-face 字体加载配合）；② table-cell-overflow combined width+height/overflow（R997 测绘）；③ Phase A per-font 真实 ascent（R887）；④ multicol Phase 2。R993-R995 累计 css-flexbox +4 / css-tables +4 零回归已 land main；R996/R997/R998 三轮纯调查确认单 session clean lever 耗尽，forward motion 全在多会话结构性。

### R997 table-cell-overflow direct-text 测量实验 NET-NEGATIVE 已回退·须 width+height/overflow 合修·零源码·纯调查

承 R996 addendum「table-cell-overflow 须直接文本 only 测量」。本轮实施安全版：`box_content_max_width`（intrinsic_sizing.rs）对非叶盒补测**直接文本节点子元素**（复用 `fragment_inline_max_width` + `doc.child_nodes` 过滤 `NodeKind::Text`），仅直接文本非全后代（避开 margin-collapse-101 block-后代文本过计陷阱）。

**A/B 实测 NET-NEGATIVE 已回退**：① table-cell-overflow-explicit-height twin **9.73%→4.13%（改善但仍 FAIL >1%）**——cell 宽测对了（was 8px → ~211px），残余 4.13% = **td height 语义**（ZW td{height:20px} 当 min-height 撑到内容 304px，chromium 现 spec（bug 1880550）对 overflow:hidden 的 td **裁剪到 20px**）。② **margin-collapse-101 回归 3.66%→6.51%**——`<div class="b"><div class="red"></div>B</div>` 的 div.b 有直接文本 "B"（50px font ~30px），ZW 此前测 0（missed），新测 30px → 列宽 +30px → 超 chromium（chromium 列宽由别处定）→ diff 增。css-tables 全量 73→74（+1 真实 pass）但跨 dir 净负（101 worse）。

**裁决（回退）**：width-only 修不充分——driving twin 仍 FAIL（4.13%，需 height/overflow 合修），且净跨 dir 负（101 回归）。**table-cell-overflow 须 width 测量 + td overflow:hidden 裁剪语义 两步同做**才能 net-positive（width 单做 perturbs 101 且 twin 不过）。按 code-guidelines「不做负价值修改」+ R855/R996 零-yield/负-yield 回退先例**回退**。下一会话勿再做 width-only。

**真修路径（更新测绘）**：① width 测量（本轮安全 direct-text 方案，已验证不破坏 leaf 路径）+ ② td/`display:table-cell` 的 `overflow != visible` 时 height 作 used height（裁剪内容，css-tables-3 §height-distribution + bug 1880550）——目前 ZW 把 td height 作 min-height（撑内容）须改为 overflow-gated used-height。两步合做后 twin 应过 + 101 应恢复（width 增被 height 裁剪补偿）。

**下一步**：table-cell-overflow combined fix（dedicated session）；或转 Phase A per-font 真实 ascent（R887 provider wiring，R970 已证常数 0.928 可产 +138，per-font 增量）；或 multicol Phase 2；或 feature gap（::backdrop/表单控件/content-visibility）。R993-R995 累计 css-flexbox +4 / css-tables +4 零回归已 land main，本 R997 纯调查无新代码。

### R996 三 dir 扫描确认 clean lever 单 session 耗尽·css-tables/css-position/css-multicol top-worst 全 structural/feature-gap/JS·零源码·纯调查

承 R995 后系统复核 3 个未深扫 dir 的 top-worst，确认 R740/R882「单 session clean lever 耗尽」结论。逐 dir 分类（全非 clean single-session fix）：

**css-tables**（baseline 73 post-R995）：① table-cell-width-0（20%）= R97 表 intrinsic sizing（structural，doc 早已标）。② percent-height-overflow-auto-in-unrestricted-block-size-cell（17%）= scrollbar feature gap（DC-11 scroll-container 未实现）。③ baseline-vertical（12%）= 表格基线对齐（font/baseline structural）。④ **table-cell-overflow-explicit-height-001/002（9.73% twin）probe 实测**：`<td height:20px overflow:hidden>` + 300px tall div + text——ZW td 渲染 w=**8**（应 ~text 宽 ~150）+ h=304（grew to content，min-height 语义正确）。**真根因 = 表 auto-layout 不测量 cell 内文本内容宽**（text "Can you see this text?" 未参与 cell max-content 宽计算 → cell 塌缩到 div border 8px）。这是 R109/IFC 文本测量在 table cell 上下文的扩展，非单点。⑤ percentages-grandchildren-quirks（已 R995 修）。

**css-position**：① replaced-object-backdrop（100%）/ backdrop-inherit-rendered（47%）= ::backdrop feature gap（position-absolute 元素的 backdrop）。② **position-absolute-semi-replaced-stretch-input/other（21%/13.62% twin）= 表单控件（input/select/textarea/progress/meter）渲染 + abspos stretch**（csswg #6789 semi-replaced）——ZW 不渲染表单控件原生外观，feature gap。③ position-absolute-dynamic-relayout-005/006（11.71% twin）= **JS-driven**（content-visibility:hidden / display:none → visible/block 经 script 切换，需 JS 执行 + 二次布局）。④ position-absolute-in-inline-006（5.10%）= R336 abspos-in-inline 结构性。⑤ position-relative-002/005（4.88% twin）/ hypothetical-dynamic-change-002/003（4.17% twin）= 4-5% font 噪声区或 JS。

**css-multicol**：top-12 worst（81%/37%/30%/28%/28%/23%/21%/20%/16%）**全 structural**——column-balancing-paged（print）、multicol-rule-nested-balancing 簇、multicol-span-all-children-height 簇（column-span:all 子高度）、multicol-breaking（fragmentation）、subpixel-column-rule-width（subpixel 舍入 + computed value）。全属 R383「multicol Phase 2 统一 column-flow」deadlock territory（嵌套/breaking/balance，多会话硬核）。

**裁决**：3 dir 复核零 clean single-session lever。R993-R995（css-flexbox +4 / css-tables +4）是本会话窗口期，后续 forward motion 全在多会话结构性 / feature 实现：
1. **表 auto-layout 文本测量**（table-cell-overflow 真根因：cell max-content 宽须含文本，R109/IFC 扩展）。
2. **Phase A IFC font-metric per-font**（R887 provider wiring，R990 已证常数 0.928 可产 +138，per-font 增量）。
3. **R109 §9.2.1.1 匿名块**（结构性 deadlock，CB-through-inline 等）。
4. **multicol Phase 2 统一 column-flow**（嵌套/breaking/balance，R383）。
5. **Feature 实现**：::backdrop / 表单控件原生渲染 / content-visibility / scroll-container。

**下一步**：选上述结构性 lever 之一 dedicated session 推进（首推 table auto-layout 文本测量——R996 已精确定位 table-cell-overflow twin 真根因，cell max-content 含文本是可独立 slice，driving 案已 probe）。R993+R994+R995 累计 css-flexbox +4 / css-tables +4 零回归已 land main。

**★ R996 table-cell-overflow fix 实验（attempted + reverted，防 re-work）**：试修 `compute_cell_intrinsic_width`（table_types.rs）—— early-return + text branch 取 max(content/direct_text)。**实测 0-yield（cell 仍 w=8）**：probe 显示函数被调用（cell w=782 taffy 满，direct_text_len=22 w=211），但 early-return 未触发（tall div 子 width:auto 填满 cell 782 > cell\*0.95，故 has_explicit_child=false），落入 text branch → `box_content_max_width(cell)` 返回 4（**仅测叶盒文本，不测 cell 直接匿名 inline 文本** "Can you see this text?"）。真修须增强 `box_content_max_width` 测非叶盒的直接匿名文本（须区分「直接文本」vs「block 后代文本」避免多 block cell 过计）。**已 revert**：naive max(intrinsic, direct_text_width) 会**回归 margin-collapse-101**（collect_text_length 对多 block 子过计，正是 R702/R679 改用 box_content_max_width 的原因）。下会话若攻此 lever 须先实现「直接文本 only」测量（DOM direct text-node children，非 text_content 全后代），再 max 进 box_content_max_width 的 inline_sum。

### R995 orphan table-cell shrink-to-fit = percentages-grandchildren-quirks-mode-001/002 14.85%→0.60% PASS·css-tables Oracle 69→73（+4）零回归·R679 表 shrink 簇新 facet

承 R994 后扫 css-tables top-worst 定位 percentages-grandchildren-quirks-mode-001/002（twin，14.85%）。**Phase 0 probe**（layout-engine 临时单测复刻 001 结构：`<div style="display:table-cell;height:100px;background:green"><div style="width:100px"><div style="height:100%;background:red">`，quirks 模式）发现：① quirks %height 正确（red div height:100% → auto → 0，green 应显）；② **真根因 = orphan table-cell（display:table-cell 无 table 祖先）拉伸到 784px 满宽**（应为 100px 收缩到子内容），green 784×100 vs ref 100×100 = 14.85% diff 主因。

**根因链**：orphan table-cell 经 `adjust_table_layout_inner`（table.rs:65 孤立 table-internal 分支）→ `layout_table` → `build_grid` 空（cell 子是 block 非 table-internal）→ `shrink_table_to_block_content`。该函数本应收缩 cell 到 block 子的 max-content 宽，但 **`block_max_content_width` 只递归求和子，不读元素自身显式 Px 宽**——middle div（width:100px）子是 0 内容 red div → max_content_width=0 → 早退（line 82）→ cell 留 784。即使收缩触发，函数用子内容高（0）覆写 cell 自身显式 height:100px → 高度塌缩。

**改动**（`table_shrink.rs`）两处：
1. `block_max_content_width`：把元素自身显式 Px 宽作 max-content 下界（`inner.max(own_explicit_w)`）——shrink-to-fit 不应把 table 收缩到小于 block 子的显式宽。
2. `shrink_table_to_block_content`：table/cell 自身显式 Px height 作 min-height 下界（CSS §17.5.2）——`final_content_height = content_height.max(explicit_height - padding_border)`，不被 0 内容子塌缩。

**验证（chromium Oracle + 三态门禁）**：percentages-grandchildren-quirks-mode-001/002 **14.85%→0.60% PASS**（strict）；**css-tables Oracle 69→73（+4 零回归）**（001/002 +2 + 另 2 案同 shrink 改善）；**welcome 16.57% 不变**（<20%）；table_shrink 6 既有 + 1 新单测（`test_shrink_orphan_table_cell_respects_child_explicit_width_and_own_height`）全绿；make test 全 workspace 绿（exit 0）；clippy/fmt 干净。

**意义**：收割 R679（table shrink-to-fit 簇）新 facet——orphan `display:table-cell` 的 max-content 收缩（显式宽子）+ 显式 height 尊重。R679 此前 landed empty-table（R749）+ deferred R681-R684；本 R995 是「显式宽子 + 显式 height」slice。block_max_content_width 的「显式 Px 宽作下界」修复对真实网页 orphan table-cell（如 `display:table-cell` 布局 hack）是基础正确性。

**下一步**：css-tables 残余 worst（table-cell-width-0 20% R97 表 intrinsic / percent-height-overflow-auto 17% scrollbar / baseline-vertical 12% 基线 / table-cell-overflow-explicit-height 9.73% twin table overflow+height），或转 R990 余波 per-font ascent / R370 flex-container-intrinsic-width。R993+R994+R995 累计 css-flexbox +4 / css-tables +4 零回归。

### R994 R717 fixup 泛化（leaf + CSS aspect-ratio）= css-flexbox Oracle +2（287→289）·零回归·R993 收割延伸

承 R993。R993 的 post-layout fixup 原仅覆盖 ratio-only SVG `<img>`（`b.is_replaced && img_intrinsic_ratios`）。复查 aspect-ratio-intrinsic 簇残余发现 003/004/011/014 是**不同机制**——CSS `aspect-ratio` 在非替换 **leaf `<div>`**（非 img/SVG）上、flex 容器内（003: `inline-flex;height:100px` + 子 `<div aspect-ratio:1/1>`）。003 容器 height 明确（100px）→ taffy 已正确 derive，15.67% 残差 = inline/text 定位（非 ratio 问题，独立）。

**改动**：把 `apply_flex_ratio_img_size` 重命名为 `apply_flex_aspect_ratio_item_size` 并**泛化触发条件**——不再要求 `is_replaced + img_intrinsic_ratios`，改为：① `b.children.is_empty()`（leaf，无内容决定 main，避免误覆盖文本/子内容 flex item）；② 父 Flex/InlineFlex；③ taffy style `aspect_ratio > 0`（直接读 st.aspect_ratio，涵盖 CSS aspect-ratio 与 SVG ratio-only）；④ item main 轴 CSS Auto（显式 Px 由 converter 处理，不覆盖）；⑤ cross>0 且 main 与 cross×/÷ratio 推导值差 >0.5px。drop `ratios_for_r717` param（fixup 不再需要 ratios map，aspect_ratio 已在 taffy style 上）。

**验证（chromium Oracle + 三态门禁）**：007 仍 0.00% PASS（未回归）；**css-flexbox Oracle 287→289（+2，新翻 2 案 CSS aspect-ratio leaf flex item）**，baseline 285→289 累计 +4；css-grid 40.8%（fixup Flex-only 不触发，未回归）；**welcome 16.57% 不变**（<20%）；R717 三单测仍绿；clippy/fmt 干净；**make test 全 workspace 绿（exit 0）**。

**意义**：R717 fixup 泛化使 CSS `aspect-ratio` 在 flex leaf item 上也正确按 transferred-size 推导 main（taffy 0.7 对 Auto-cross 不自动 derive 的同一缺口，CSS aspect-ratio 与 SVG ratio 共享）。leaf + auto-main 双守卫保证不误覆盖内容/显式尺寸 flex item。

**下一步**：R717 残余 003/004/011/014 = inline-flex 定位 / JS-driven（document.body.offsetTop + style 变更），独立子机制非 ratio；或转 R990 余波 per-font ascent / R370 flex-container-intrinsic-width。R993+R994 累计 css-flexbox +4 零回归。

### R993 ★★R717 ratio-signal 全链 LANDED = aspect-ratio-intrinsic-size-007 33.65%→0.00% PASS·css-flexbox Oracle +2 零回归·4-crate plumbing + post-layout flex-ratio fixup·净正

承 R992「decode-level 死路彻底关闭，下会话必须做完整 ratio-signal」。R992 三次 decode-level definite-size 尝试（R980/R991-设计/R992-实测）全失败，证明正确路径是 **ratio-only signal（不设确定 size）+ 让 taffy/flex ratio-derive**。本轮实施 R991 测绘的 4-crate ratio-signal，并**发现 + 修复** R991/R992 未预见的第二阻塞。

**4-crate ratio-signal plumbing（dormant-safe）**：
1. **render-foundation（image_cache.rs）**：`ImageData` 加 `intrinsic_ratio: Option<f32>` 字段 + getter；新 `svg_intrinsic_ratio(bytes)` 解析 `<svg>` 根属性——双绝对 width/height → None（走 image_sizes），否则（% / 缺失 / 仅 viewBox）→ Some(viewBox w/h)；`decode_svg_bytes` 填充。5 单测（绝对/% /viewBox-only/混合/无 viewBox + 端到端）。
2. **engine（pipeline.rs）**：`RenderPipeline.image_ratios: HashMap<u64, f32>` 字段 + `set_image_ratios` + `build_img_intrinsic_ratios`（NodeId-keyed，镜像 build_img_intrinsic_sizes）；4 个 `compute_with_img_sizes` 调点 + pipeline_budget 多行调点同步传 ratios。
3. **layout-engine**：`BuildContext.img_intrinsic_ratios` + `build_layout_tree(_with_r109)` 加参数 + `compute_with_img_sizes` 加参数；`apply_replaced_element_sizing` 加 ratio-only 分支——img 无 HTML 属性、无 image_sizes 条目时，仅设 `aspect_ratio`（**不**设 size，R992 decisive 机制）。
4. **webview + harness**：`WebView.cached_image_ratios` + `set_image_ratios` + `cached_image_ratios()`；`fetch_image_subresources` 返回 (sizes, ratios) tuple（ratio-only SVG 进 ratios、不进 sizes）；`async_load` 两处 lazy/batch 图片解码同步；wpt-runner `extract_image_metrics`（返回 sizes+ratios tuple，`extract_image_sizes` 保留作 sizes-only 旧调点）+ reftest.rs pipeline/webview 两调点。

**★ 第二阻塞发现 + 修复（R991/R992 未预见）**：plumbing 落地后 A/B 实测 aspect-ratio-intrinsic-size-007 **反退 33.65%→64.03%**（img collapse）。R717DBG 插桩定位：ratio-only 分支正确触发（aspect_ratio=2.0 已设），但 `apply_flex_transferred_min_size`（build_layout_tree 期）**提前返回**——它读 `parent_style.width` 仅接受 `LengthValue::Px`，007 的 flex 容器 `<div>` width 为 Auto（解析为视口 800，但 computed style 是 Auto）→ cross=None → return → 无 transferred min → img 无确定 size + 无 min → collapse 到 0。**taffy 0.7 自身不**从 aspect_ratio + Auto-cross 推导 leaf flex item main 尺寸。

**修复**：新 post-layout pass `apply_flex_ratio_img_size`（engine.rs）——第一趟布局后 LayoutBox 已含解析出的 cross 尺寸（经 align-stretch / 包含块解析），对 ratio-only img flex item 按 **cross_resolved × ratio（row）/ cross_resolved / ratio（column）** 推导 main，改写 `size.main = Length(...)` + mark_dirty，由调用方重跑 taffy（与 R695/百分比 padding/intrinsic-sizing 共用一次重算）。仅水平 WM；仅 cross>0 且 main 与推导值显著不同（>0.5px）时触发。007：第一趟 img width=800（stretch）height=0 → 推导 height=800/2=400 → 重跑 → 800×400 ✓。

**验证（chromium Oracle + 三态门禁）**：
- aspect-ratio-intrinsic-size-007：**33.65%→0.00% PASS**（strict 真通过）。
- css-flexbox Oracle 全量：baseline 285/497 (57.3%) → **287/497 (57.7%)，净 +2 零回归**（007 翻 PASS +1，另一 aspect-ratio 案翻 PASS +1；003/004/011/014 维持 ~15% 未回归，不同子机制）。
- **welcome (DC-13) 16.57% 不变**（<20% gate PASS；welcome 无 SVG img，零影响）。
- make test **全 workspace 绿**（exit 0，layout-engine 988+3 R717 / engine 1179 / render-foundation 516 / webview 518 全 0 failed）。
- clippy --workspace --all-targets -D warnings ✓；fmt ✓。
- 3 新单测（r717_flex_ratio_img_tests.rs）：flex column（800×400）/ flex row（200×400 对称）/ 非 flex 父不强推。

**意义**：关闭 R717 aspect-ratio-intrinsic 簇驱动案（2 年长期 gap，R980/R991/R992 三次 decode-level 尝试失败后）。**关键纠正 R992 结论**：R992「ratio-only signal 即足够，taffy/flex 会 ratio-derive」**只对了一半**——ratio-only signal 是**必要**条件（decode-level definite-size 死路确认），但**不充分**：taffy 0.7 对 Auto-cross flex 容器内的 leaf item 不自动 ratio-derive，须 post-layout pass 用**解析出的**（非 computed style 的）cross 尺寸推导 main。R991 测绘的 4-crate plumbing 完整落地 + R992 未预见的 post-layout transferred-size 补丁共同构成完整修复。

**下一步**：R717 残余（003/004/011/014 ~15%，独立子机制——可能 CSS aspect-ratio 显式值或非 flex 上下文，待逐案）；或转 R990 余波继续扫（per-font 真实 ascent R887 provider，或 R370 flex-container-intrinsic-width）；或 master.md 已 ruled-out 的多会话结构性。本会话 R993 全维度净正零回归。

### R991 R717 Phase 0 调查 = SVG `<img>` viewBox-as-intrinsic bug 定位·4-crate ratio-signal 修复面精确测绘·零源码·纯调查

承 R990 余波结束转 R717（aspect-ratio-intrinsic 簇，非 font 角度）。本轮 read-only Phase 0 调查 R717 驱动案 aspect-ratio-intrinsic-size-007（33.65%，css-flexbox）的精确根因 + 测绘修复面。

**驱动案**：`<div style="display:flex;flex-direction:column"><img src="large-green-rectangle.svg"/></div>`，SVG = `<svg width="100%" height="100%" viewBox="0 0 7500 3750">`（百分比 dim + viewBox ratio 2:1，无绝对固有）。REF = `<svg viewBox="0 0 1000 500" style="background:green">`。期望 img 在 flex column 内 width 拉伸 800、height = 800/ratio = 400（800×400 绿）。

**根因精确**：`decode_svg_bytes`（render-foundation/image_cache.rs:421）用 `usvg tree.size()` 对 % dim SVG 返回 **viewBox 维度（7500×3750）作 intrinsic size**。CSS 规范：**% dim SVG 无 intrinsic size（仅有 viewBox ratio）**。这 7500×3750「假固有」经 `img_intrinsic_sizes`（tree.rs:54）→ `apply_replaced_element_sizing`（tree.rs:168）→ taffy，使 flex transferred-size suggestion 错误（按假固有 7500×3750 而非 ratio+definite-cross 推导）。

**R980 trap 重述 + 区分**：R980 测 **ratio-推导（50×25）decode fix** → all-auto 反退（rendered 50×25 vs chromium 300×150）。**ratio-推导错**：chromium 对 % dim SVG `<img>` 用 **300×150 CSS 默认**（非 ratio-推导）。R980 未测 300×150-specific（测的是 ratio-推导）；但 300×150 对 all-auto 正确，对 explicit-dim（width:40 + viewBox ratio 2:1）经 aspect_ratio 推导 height=20 也正确（ratio 2.0 同）。但若 viewBox ratio≠2:1（如 1:1），300×150 默认 ratio 2:1 与真 ratio 1:1 冲突 → 须「300×150 默认盒 + 缩放到真 ratio」（CSS §10.3.2）。

**R717 真 fix（4-crate ratio-signal，已测绘）**：
1. **decode（render-foundation）**：区分 **真固有**（PNG/both-attr-SVG）vs **ratio-only**（%-dim/viewBox-only SVG）—— 须 parse SVG width/height attr 判定是否 %（usvg 不暴露原始 attr 类型）；ratio-only 不填 image_sizes，仅填并行 `image_ratios: HashMap<u64, f32>`（viewBox w/h）。
2. **pipeline（engine）**：加 `image_ratios` 字段（镜像 image_sizes），经 extract_image_sizes（harness）填充，转发 painter/layout。
3. **tree.rs §10.3.2 消费（layout-engine）**：img 无 HTML/CSS dim 时——若 image_sizes Some（真固有）用之；elif image_ratios Some 用 **300×150 默认盒缩放到 ratio**（如 ratio=2 → 300×150，ratio=1 → 150×150，ratio=0.5 → 75×150）；else 300×150。explicit-dim 一侧时 aspect_ratio 用 image_ratios（非默认 2:1）。
4. **flex transferred-size（R982/R983 §4.5）**：确保 transferred-suggestion 用 ratio + definite-cross 而非假固有。

**范围**：跨 4 crate（render-foundation decode + engine pipeline + layout-engine tree.rs + wpt-runner harness extract）+ Option 类型变化（image_sizes → Option 或并行 map）。driving 案 5-9 个（aspect-ratio-intrinsic 簇 007 33% / 003/004 15% / 005/009/008 2-3%）。中等 yield（~5 case）高 effort。

**裁决**：R717 是真 multi-session 架构（4-crate + Option 变化 + SVG attr parse），非单 session 可完成。Phase 0 已精确定位根因（usvg viewBox-as-intrinsic）+ 测绘修复面（4 步跨 crate）。**勿以 R980 ratio-推导 decode 单点 fix 重试**（all-auto 反退 trap 已证）；须完整 ratio-signal（decode 区分 + 并行 map + tree.rs 消费 + transferred-size）。下会话 dedicated session 实施。

**▶ 下会话**：① **R717 实施（dedicated session）**——按 Phase 0 测绘的 4 步：decode 加 SVG attr % 检测 + image_ratios map（additive Phase 0 先 land dormant 零回归）→ tree.rs §10.3.2 消费 300×150-scaled-to-ratio → A/B aspect-ratio-intrinsic 簇 + 全量 oracle；② 备选 per-font 真实 ascent（R887 provider wiring，R990 已证常数可行 per-font 增量）；③ R370 flex-container-intrinsic-width。font-wall layout-side 常数 lever 已尽（R990 + line-height + site-3 三证），forward 是 R717 replaced-element sizing 或 per-font provider。

**★ R717 decode-level 300×150-scaled-to-ratio 实验 REFUTED（2nd decode-level 失败·定 key 机制）**：试 R991 测绘的 narrow 路径——decode_svg_bytes 对 %-dim SVG 返回 **300×150-scaled-to-viewBox-ratio**（非 R980 的任意 ratio-推导，是 CSS 默认 + ratio 缩放）。实施 `svg_root_attrs_absolute`（parse `<svg>` width/height attr 判定是否绝对）+ 非绝对时按 viewBox ratio 算 300×150-scaled（ratio≥2 → 300×300/ratio / 否则 150×ratio×150）+ Transform 缩放栅格化。**A/B NET 负**：aspect-ratio-intrinsic-**007 33.65%→54.65%（+21pp 显著回归）**，003/004/005/008/009 持平，welcome 16.57%（无 SVG 未变）。已 `git checkout` 回退。

**★ 决定性机制（decode-level 为何 2 次失败）**：给 %-dim SVG 任何**确定 intrinsic size**（无论 7500×3750 usvg 默认 / 50×25 R980 ratio-推导 / 300×150-scaled 本轮），taffy 都把它作 img 的**确定高度**用，**阻止 ratio-derivation**。007 期望：flex column 内 width 拉伸 800 → height 应 = 800/ratio = 400（chromium 行为，因 SVG 无 intrinsic size 故 height 不定、由 ratio+definite-cross 推导）。给确定 intrinsic → taffy 用该 height（300×150 给 150；7500×3750 给 3750）而非 400。**正确 fix 须让 img 无确定 intrinsic size（仅 ratio signal），taffy/flex 才会 ratio-derive**。这**推翻 R991「tree.rs 消费 300×150-scaled-to-ratio」方案**——300×150-scaled 仍是确定 size，错。

**真 R717 fix（更新测绘）**：decode 须对 %-dim SVG **完全不填 image_sizes**（signal ratio-only via 并行 image_ratios）→ tree.rs `apply_replaced_element_sizing` 的 no-intrinsic 分支须**不设确定 size**，仅设 `aspect_ratio`（从 image_ratios），让 taffy 按上下文推导（all-auto → taffy 默认或 300×150；flex column width-stretched → height=width/ratio）。即 image_sizes 须变 Option（或仅真固有插入），image_ratios 并行，tree.rs no-intrinsic 分支用 aspect_ratio-only。**3 次 decode-level definite-size 尝试（R980/R991-设计/R992-实测）全失败，decode-level 死路彻底关闭**——下会话必须做完整 ratio-signal（image_sizes Option + image_ratios + tree.rs aspect_ratio-only no-intrinsic 分支），非 decode 单点。

**意义**：**font-wall 首次实质突破**——R388/R631/R633/R643/R668 五证把 font-wall 定性为「不可消除」，但那些是**光栅化/选择/advance/bolding/rustybuzz** 五角度；**line-box ascent ratio**（apply_vertical_alignment 行盒度量）是**未被 R891 测过的第 6 角度**（R891 concept ② 是 paint-only `render_fragment baseline_offset` 字形定位，**不**改行盒高度；本 R990 改 apply_vertical_alignment strut/run ascent → 行盒高度本身，机制不同）。R989 已纠正此传递性证伪误判。**+138 oracle-pass 证实**：非-Ahem 文本行盒高度修正（0.8→0.928）让 138 案接近 chromium 的真实行盒度量跨过 1% 阈值。这是迄今单轮最大 DC-14 yield 之一，打破 R964-R989 调查/neutral 为主的僵局。

**纠正 R989「7 次 strut 先例」悲观预期**：R834/R836/R849/R875/R889/R890/R891 七次 strut net-negative **全部是 provider 单点或 paint-only 变体**（受 empty-styles / 不改行盒高度 双重局限），**is_ahem-gated ratio 是首个真正改变行盒度量的方案，且 net 正**。font-wall 非「永久 plateau」——line-box ascent 角度可修，证明结构性 plateau 内仍有 narrow slice 可挖。

**交付**：mod.rs apply_vertical_alignment 改 + 2 新单测（`test_r990_ahem_ascent_ratio_0p8` / `test_r990_non_ahem_ascent_ratio_0p928`，断言 line-height:1 时 baseline_y = fs×ratio）+ 2 既有测更新（`test_vertical_align_sub_in_line` / `_super_in_line` 的 baseline_y 期望从 0.8 改 0.928，因用 is_ahem_font=false）。

**门禁全绿**：fmt ✓ / clippy --workspace --all-targets -D warnings ✓ / make test ✓（layout-engine 988 passed 0 failed，workspace 零失败）/ product-smoke welcome 16.57% < 20% gate ✓。

**▶ 下会话**：① **R990 余波扫描**——非-Ahem 行盒高度变了，潜在更多 dir yield（borders/css1/visudet/values/generated-content 等 CSS2 子目录 + writing-modes 非-Ahem 簇），逐 dir fresh-oracle 扫确认净正 + 找次级 lever（writing-modes 实测 5.6%→7.1% 已 +~12，backgrounds 持平 228，generated-content/lists/borders/values 待 stash A/B）；② **0.928 是单常数**，真实 per-font ascent 不同（DejaVuSans 0.928 / NotoSansCJK ~0.88 / system-ui 视 fontconfig）——若 R970 font-metric provider wiring（R887 5-layer）完成，可用真实 per-font ascent 替代常数，潜在再 yield（但 R990 已证常数 0.928 即 +138 + welcome/morning 双改善，per-font 是增量优化，多会话）；③ R717 / R370（独立 multi-session）。**R990 全维度净正零回归（纠正首轮 welcome 误判），font-wall 已破，下一步深挖 line-box 度量 + 扫余 dir yield + per-font 增量**。

**★ R990 余波 site-3 REFUTED（paint-side baseline_offset 勿动·layout/paint 不对称是正确的）**：R990 改 apply_vertical_alignment（layout 侧行盒 strut/run ascent）到 0.928 后，试把 paint 侧 site-3（painter/text.rs:1383 R953 路径 `baseline_offset = fragment.height - 0.8·fs`）同步到 is_ahem-gated 0.928（假说：保持 layout/paint 一致）。**A/B 实测 NET 负**：css-text **355→339（丢全部 R990 +16 增益）**+ welcome **16.57→16.88%（更差）**。已 `git checkout` 回退。**结论**：layout 侧 0.928 / paint 侧 0.8 的**不对称是 productive 的**——paint 侧 site-3 的 0.8 不是「与 layout 不一致」，而是互补（layout 定行盒高度、paint 定 glyph 在行盒内位置，两侧用不同 ratio 恰好正确；同步会 double-count ascent 修正致 glyph 偏高）。**painter/text.rs 的 v_offset/baseline_offset 常数（site-1 multicol 1.0·fs / site-2 stored 1.0·fs / site-3 R953 0.8·fs）勿以「与 R990 一致」为名同步改**——R891（site-2 concept ② → negligible）+ 本轮 site-3（→ net 负）双证 paint-side 改是死路。真 lever 仅 layout-side apply_vertical_alignment（R990 已尽）。下会话勿重试 paint-side baseline 同步。

