# RFC: per-element white-space in IFC（Phase-A 结构切片）

**状态**：⚠ **可行性下调（R1902 实测）**——minimal preserve-only slice 完整实现 + A/B 证 target
cluster **NET-NEGATIVE**（含 uniform-pre 案 white-space-pre-001/002 +3.6/+4.7pp）→ REVERTED。
R1901「orthogonal to storage / Ahem 可独立 yield」估计过乐观：实测连 uniform-pre 都恶化，
示 per-element-ws 触达 cross-element break coherence 深层（非单 preserve 字段可解），须完整
break_at_newline + pre-line split_collapsible + cross-element 断行边界 line-breaker 重构 +
dual-path override-map 同步 + 测量路径同步 = **high-risk multi-session IFC 重构（R125/R213/R1052
deadlock 谱系）**。从「pre-authorized 可推进」下调为「high-risk，须先解 cross-element break
coherence + dual-path」。详见 [`evidence/r1902-per-element-ws-slice-impl-ab-netnegative-reverted-2026-07-23.txt`](./evidence/r1902-per-element-ws-slice-impl-ab-netnegative-reverted-2026-07-23.txt)。

**原状态**：Ready for slice-by-slice 实现（ruling #4 预授权：Phase-A 深水区允许 dedicated 多会话推进）
**日期**：2026-07-23（R1901）
**前置裁决**：master.md 顶部裁决包 ruling #4（Phase-A 多会话预授权）+ ruling #2（font-stack rebuild RFC-only，本 RFC **不**触及 font-stack）
**相关**：[`evidence/r1899-preline-nl-fix-impl-unreachable-per-element-ws-2026-07-23.txt`](./evidence/r1899-preline-nl-fix-impl-unreachable-per-element-ws-2026-07-23.txt) / [`evidence/r1900-cs2-text-plateau-phaseA-ws-coherence-2026-07-23.txt](./evidence/r1900-cs2-text-plateau-phaseA-ws-coherence-2026-07-23.txt) / [`multicol-layoutside-IFC-implementation-handoff.md`](./multicol-layoutside-IFC-implementation-handoff.md) line 20

---

## 1. 问题

CSS `white-space` 是**继承 + per-element** 属性：每个 inline element 的 `white-space` 值
独立支配其自身文本的空白折叠/保留/换行（CSS 2.1 §16.6 / CSS Text 3 §4）。

**ZW 现状**：IFC 在 **container 级**取**单一** white-space（`self.preserve_whitespace` /
`self.no_wrap` / `self.break_at_newline` / `self.word_break`，由 container 的 computed style
推导），**忽略 child inline element 的 white-space**。

已记录（`multicol-layoutside-IFC-implementation-handoff.md:20`）：`collect_inline_items`
读 per-element `font_size`/`line_height`/`letter_spacing`/`word_spacing`/`is_ahem`，但
**white-space/word-break/overflow-wrap 是 config 级**。

## 2. 实测验证（root cause 确认）

driving reftest 都把不同 white-space 放在 **child inline element**，container 另取一值：

| case | diff | container ws | child ws | 现象 |
|------|------|-------------|----------|------|
| `CSS2/text/white-space-mixed-001` | **39.14%** | `pre` | nested `normal`/`nowrap`/`pre` spans + 新行/多空格/tab 跨 span 分布 | ZW 全按 container `pre`，忽略 child normal/nowrap → 全错 |
| `CSS2/text/white-space-collapsing-bidi-001` | 16.16% | — | mixed + bidi | per-element-ws + bidi |
| `CSS2/text/white-space-normal-001/002/004` | 12-15% | `normal`* | block-in-inline 空白边界 | R109 + per-element 折叠 |
| `CSS2/text/white-space-pre-001/002` | 12-14% | `pre`* | — | pre 保留（部分 per-element） |
| `css-text/white-space/pre-line-with-space-and-newline` | 16.06% | `normal`(span) | `<i>pre-line` + `&#10;` | ZW 按 span normal 忽略 `<i>` pre-line |
| `CSS2/generated-content/content-white-space-001` | 3.34% | `normal`(#test) | `::before{pre-line}` + `\A` | ZW 按 #test normal 忽略 ::before pre-line（+ Latin font-wall） |

★ **R1899 实证**：container-level pre-line\n fix（break_at_newline 字段）unit-correct 但
**0 pixel effect**——driving case 全 child-element ws，container-level fix 不触达。

## 3. yield scope（Ahem，font-wall 可分离）

★ 关键区分：本切片的 driving case **全 Ahem**（exact 1em 度量，font-wall-free）：

```
white-space-mixed-001 / normal-001/002/004 / pre-001/002 / collapsing-bidi-001  → 全 AHEM
css-text pre-line-with-space-and-newline                                       → AHEM
```

故 per-element-ws 修后**预期 flip**（正确断行 + Ahem 精确度量 = 匹配 chromium）。
区别 R1760/R1526「broad-authoritative-storage 被 font-wall 阻塞」——**per-element-ws 与
storage 机制正交**：R1526 是「强制 paint 复用 layout 的 estimate_char_width 行断」（非 Ahem
estimate 发散），per-element-ws 是「每个 run 用自己的 white-space 规则」（Ahem 度量精确）。
non-Ahem case（content-white-space-001 Latin）仍 font-wall，但 **Ahem cluster 可独立 yield**。

预估 yield：~7-8 Ahem white-space case（CSS2/text 6 + css-text 1）+ 可能旁及（white-space
簇跨 subdir 的 ripple）。非 headline（不动 57% plateau 大数）但**真实结构 correctness + 跨
多 subdir 的 white-space 簇**，且 ruling #4 预授权。

## 4. 设计（per-run white-space）

**核心**：TextRun / InlineItem 携带其 effective white-space；split_into_words + break_lines
按 per-run white-space 处理（非 container config）。

### 数据结构
- `TextRun` 增字段：`preserve_whitespace: bool`、`break_at_newline: bool`（+ 可选 `no_wrap`、
  `word_break` 若 slice 需要）。由 collect_items 从 element computed style 填充。
- container config 保留作**回退**（run 未填时用 container 值，保 net-0 切片安全）。

### 处理流
- `collect_inline_items`：每个 text node 读其 element 的 `white_space` → 设 run 的
  `preserve_whitespace`/`break_at_newline`（映射同 inline_finalization.rs:729 / text.rs:497）。
- `split_into_words`：改为 **per-run** 调用——每个 run 用自己的 `preserve_whitespace`/
  `break_at_newline` 切词（container config 不再单点驱动）。产出 words 携带「是否 break-at-newline」
  标记。
- `break_lines`：消费空串断行标记时按 **per-word 标记**（非 global `self.break_at_newline`）。

### 难点
1. **跨元素断行**：一个 `<span>` 的文本跨多行，child `<span>` 不同 white-space 在行边界切换。
   split per-run 后 break_lines 须按 word 流顺序处理（标记随 word 走）。
2. **CJK per-char breaking**：per-run split 须保 `split_collapsible` 的 CJK 逻辑（R1214）。
3. **layout↔paint 双路径**：collect_items（layout）+ paint text.rs 两处 IFC 构造都须填 per-run ws
  （R630/R632 双路径 coherence）。
4. **测量路径**：`measure_text_content`/`remeasure_*`（resolve_no_wrap_for_ifc_measure 只传
   no_wrap）——per-element ws 须同步到测量（否则 box 高度测错）。但测量只影响 sizing，可后置 slice。

## 5. 切片计划（ruling #4：可回退 + kill-switch + A/B net≥0）

### Slice 1（plumbing，net-0，无 kill-switch 需）
- TextRun 增 `preserve_whitespace`/`break_at_newline` 字段（默认 = 现 container 行为）。
- collect_items 填充（读 element white_space）。
- split_into_words / break_lines **暂不改**（仍用 container config）。
- **验证**：make test 全绿（行为不变）+ A/B CSS2/text 全 dir net-0。
- 价值：建立 per-run ws 数据通道，slice 2 在此之上改行为。

### Slice 2（core，kill-switch `ZW_PER_ELEMENT_WS`，default-off → A/B 后 default-on）
- split_into_words 改 per-run（用 run 的 preserve_whitespace/break_at_newline）。
- break_lines 改 per-word 标记消费。
- kill-switch gate：`ZW_PER_ELEMENT_WS=0` 时回退 container config（slice 1 行为）。
- **A/B（kill-switch on/off）**：CSS2/text（408 案）+ css-text/white-space（449 案）+ generated-content
  （226 案）+ product-smoke + make test。**net≥0 且 white-space Ahem 簇 flip 才 default-on**。
- 预期 flip：white-space-mixed-001（39%→<1%）+ normal/pre 簇 + pre-line-with-space-and-newline。

### Slice 3（refinement + 测量路径同步）
- measure_text_content / remeasure_* 同步 per-element ws（修 pre/nowrap 容器测量，R645 谱系扩展）。
- broad A/B（CSS2 全 + css-text + css-writing-modes）守 net≥0。

## 6. 风险与缓解

| 风险 | 缓解 |
|------|------|
| IFC deadlock 史（R125/R213/R1052） | per-element-ws ≠ metric-coherence deadlock（不同 IFC 子系统）；kill-switch + scoped A/B；Ahem-first 验证 |
| layout↔paint 双路径分裂（R630/R632） | slice 1 两处 IFC 构造同步填 per-run ws；A/B 守 |
| broad blast radius（IFC 核心） | kill-switch default-off；slice 2 仅 white-space 相关，A/B 全 CSS2 + css-text |
| CJK/bidi 回归 | split_collapsible 复用保 CJK；A/B 含 writing-modes |
| 跨元素断行边界 nuance | slice 2 先做简单 case（container pre + child normal/nowrap），bidi/复杂嵌套 slice 3 |

## 7. 与 ruling #2（font-stack）的关系

本 RFC **不触及 font-stack**（不引入 C 字体依赖、不改 metric/raster/shape 管线）。per-element-ws
是纯 IFC 结构（white-space 规则的 per-run 应用），font-wall 独立。non-Ahem white-space case 的
残余 diff（font-wall）仍归 font-stack rebuild（ruling #2），不在本 RFC scope。

## 8. 成功标准

- Slice 2 default-on 后：CSS2/text white-space Ahem 簇（≥6 case）+ css-text pre-line-with-space-
  and-newline **flip to oracle-pass（<1%）**。
- CSS2/text + css-text + generated-content 三 dir **net≥0**（无回归）。
- make test 全绿 + product-smoke welcome 字节一致。
- kill-switch `ZW_PER_ELEMENT_WS` 保留（可回退）。
