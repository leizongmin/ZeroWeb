# line-clamp 实现 RFC（CSS Overflow 4 §line-clamp）

**日期**：2026-08-02
**承接**：R2430 survey 识别 line-clamp 为 css-overflow 最大 fail 簇（~130 reftest），parsed-not-consumed。
**状态**：设计完成，待切片实施（rally 自主模式：lint Fail=0 即放行实施，不等用户确认——R2407 先例）。

---

## 0. 执行摘要

- **目标**：让 `line-clamp: N`（含 legacy `-webkit-line-clamp`）真正生效——把块容器内容夹到 N 行，第 N 行末加省略号（…），box 高度 = N 行。
- **范围**：basic line-clamp（block 容器，水平书写模式）+ ellipsis；modern `line-clamp` 与 legacy `-webkit-line-clamp` 共用同一路径（apply 已合并，apply_advanced.rs:1057）。
- **明确排除**：vertical writing-mode line-clamp、`continue: discard` 级联精确、scroll-markers（另一 CSS4 feature）、overflow-clip-margin（另一 feature）。
- **核心约束**：① 零文本布局回归（line-clamp 仅在声明时生效，非 clamped 块逐字节不变）；② 双 IFC 路径一致（layout 期与 paint 期两趟 IFC 必须夹到同一 N 行，否则 box 高度 ≠ paint 行数）；③ ellipsis 第 N 行末定位（非单行 text-overflow 的 content_right）。
- **推荐方案**：IFC `layout()` 读容器 `line_clamp` → `break_items_into_lines` 后 cap `self.lines` 到 N + 标记 clamped → height 自然 = N 行（下游用 lines）→ painter 第 N 行末渲 ellipsis（新增「vertical-triggered ellipsis」，复用 measure/ahem 逻辑，区别于单行 horizontal-triggered）。
- **首个落地步骤**：slice 1——IFC 加 `line_clamp: Option<usize>` + `with_line_clamp` builder + `layout()` 读样式 + cap；先验「cap 后 height = N 行 + 非 clamped 零回归」（unit test + product-smoke），暂不 ellipsis。

---

## 1. 背景与现状

### 1.1 现状（R2430 实证）

- **parsed + computed 就位**：`parse_line_clamp`（css-parser parse_extended_visual.rs:521）→ `LineClampValue::{None,Count(n)}`；apply（style-system apply_advanced.rs:1057 `"line-clamp" | "-webkit-line-clamp"`）→ `ComputedStyle.line_clamp: LineClampComputedValue`（computed_style.rs:476，default `None`）；default/inherit 全就位。
- **layout 零消费**：layout-engine / engine 全 codebase grep `line_clamp` 零命中（除 style-system）→ 夹行逻辑完全缺失。
- **ellipsis 基建（单行）已有**：painter `text-overflow: ellipsis`（text.rs:762 触发 + 1425 截断 + 1474 渲 `…`），但它是 **horizontal-triggered**（`g.x >= content_right`，单行横向溢出），**不能直接复用**于 line-clamp（vertical-triggered：超过 N 行 → 第 N 行末加 `…`）。
- **driving 簇**：css-overflow/line-clamp/* ~80 + webkit-line-clamp ~28 + block-ellipsis ~13 + line-clamp-with-{abspos,floats,fixed-pos} ~33 ≈ 130 reftest。

### 1.2 目标

- 业务：line-clamp 是卡片 UI / 文章摘要多行截断的事实标准（modern + webkit 双语法），ZW 不支持 = 大量真实页面截断失效。
- 用户：`line-clamp: N` 块容器渲染 N 行 + `…`，rest 被 overflow 裁切。

### 1.3 范围边界

- **在范围内**：block 容器、horizontal-tb、modern `line-clamp` + legacy `-webkit-line-clamp`（apply 已合并）、basic ellipsis `…`（U+2026 三点）、第 N 行末 ellipsis 截断（保留 ellipsis 宽度，截第 N 行末尾内容）。
- **不在范围内**：vertical-rl/lr line-clamp（writing-mode-aware，深，defer）；`block-ellipsis` 自定义 ellipsis 字符串（`block-ellipsis: "…"` string value，需 parse 扩展，slice 3）；`continue: discard` 与 cascade 精确交互；line-clamp 与 multicol 列流嵌套（slice 3）；scroll-markers / overflow-clip-margin（独立 feature）。

---

## 2. 关键设计决策

### 2.1 IFC cap 注入点（layout）

`InlineFormattingContext::layout()`（inline/mod.rs:615）已有 `styles` + `container` 访问 → 读 `style.line_clamp`，存入新字段 `self.line_clamp: Option<usize>`（Count(n) → Some(n)）。在 `break_items_into_lines(items)`（mod.rs:633）**之后**：

```text
if let Some(n) = self.line_clamp {
    if self.lines.len() > n {
        self.lines.truncate(n);
        self.clamped = true;  // 标记：第 N 行有更多内容（触发 ellipsis）
    }
}
```

**为什么 truncate 在 break 之后**：行已生成（含 floats/abspos exclusion 已算），truncate 仅丢尾部行，保留前 N 行的完整几何（float exclusion / inline-box 对齐不变）。下游 height 计算（inline_finalization.rs:798 `root.content_height` 从 lines 推）自动得到 N 行高度。

### 2.2 双 IFC 路径一致性（关键风险）

IFC 在两处独立运行：
- **layout 期**：inline_finalization.rs:862/1310（算 box height / fragment）。
- **paint 期**：postprocess.rs:272/398（painter 重跑 IFC 取 fragments）。

**风险**：若仅 layout 期 cap，paint 期跑全量行 → box height（N 行）≠ paint 行数（全量）→ paint 溢出 box（被 overflow 裁切显示，但 ellipsis 位/行数错）。

**对策**：`line_clamp` 必须在**两处** IFC 都读 + cap。两处都经 `layout()`（读 container style），故 cap 逻辑在 `layout()` 内即可双覆盖——**无须额外 plumbing**（只要 painter 的 IFC 也调 `layout()` 读样式，已如此）。slice 1 须验 paint 期 IFC 同样 cap（render 验证第 N 行后无内容）。

### 2.3 ellipsis（vertical-triggered，第 N 行末）

**区别于单行 ellipsis**：单行 text-overflow 在 `content_right`（box 右边界）插 `…`，截横向溢出内容；line-clamp 在**第 N 行（最后可见行）末**插 `…`，该行内容若 + `…` 超宽则截该行末尾为 `…` 让位。

**实现**（painter text.rs，新增分支，与单行 ellipsis 平行）：
- 触发：块 `line_clamp` 为 `Count` 且 `clamped == true`（有截断）。
- 定位第 N 行（最后可见行）的末尾 fragment/glyph。
- 若末尾内容 + `…` 宽 ≤ 行宽：直接在末尾 append `…`。
- 若超宽：从行末回退到「`…` 能放下」的最后一个 break opportunity（同单行 ellipsis 的 cutoff 逻辑 text.rs:1452-1464，复用 `ellipsis_char_width` / ahem 判定），截后续 + 插 `…`。
- `…` y = 第 N 行 baseline（非单行的 content_y）。

**复用**：measure_char_for_paint('.', fs, container_is_ahem)（text.rs:1447）、ahem 判定（text.rs:763）、add_glyph（text.rs:1475）全复用。新增的是「定位第 N 行末」+「vertical-triggered 分支」。

### 2.4 kill-switch + 默认

- env `ZW_LINE_CLAMP=0` 关闭（default-on，镜像 R2248 margin-trim 模式）。
- 非 clamped 块（line_clamp == None）：cap 逻辑零触发 → 逐字节不变（零回归守）。

---

## 3. 切片计划

### slice 1（基础 cap + height，无 ellipsis）— 验证双路径 + 零回归

- IFC 加 `line_clamp: Option<usize>` + `clamped: bool` 字段 + `layout()` 读样式 cap。
- **不**做 ellipsis。
- **预期 reftest**：net ~0（多数 line-clamp 测试需 ellipsis，无 ellipsis 仍 fail），但**建立 cap 基建 + 验证双路径一致 + 零回归**。
- **门禁**：unit test（cap 后 lines.len()==N、非 clamped 不变）+ `make test` 零回归 + product-smoke + product-smoke-legacy（文本布局零回归）+ scoped css-overflow line-clamp（cap 后 height 对、ellipsis 缺）。
- **裁决**：slice 1 net-0 reftest 但 cap 基建 + 零回归验证 = 可 land（基础设施 + 守；非 speculative——slice 2 紧接消费）。

### slice 2（ellipsis）— 真 reftest 收益

- painter 第 N 行末 ellipsis（§2.3）。
- **预期 reftest**：css-overflow line-clamp 簇 net 大幅正向（basic line-clamp / webkit-line-clamp / block-ellipsis basic 案应 PASS；with-abspos/floats 视交互）。
- **门禁**：unit test（第 N 行末有 `…` glyph、第 N 行内容截断正确）+ 全量门禁 + scoped css-overflow A/B。

### slice 3（边缘 + 精度）— 按需

- `block-ellipsis` string value parse（自定义 ellipsis 字符串）。
- line-clamp × multicol 列流嵌套。
- vertical writing-mode（writing-mode-aware 行方向）。
- with-abspos/floats 精确交互（abspos 不应被 clamp 裁切定位，仅 inline 内容夹行）。

---

## 4. 影响范围与风险

| 影响项 | 程度 | 说明 |
|--------|------|------|
| IFC（inline/mod.rs + break_lines.rs） | 中 | 加字段 + cap（additive，gated）；非 clamped 零变 |
| painter text.rs ellipsis | 中 | 新增 vertical-triggered 分支（复用 measure/ahem） |
| inline_finalization.rs height | 低 | height 从 lines 推，cap 后自动 N 行（无须改） |
| 双 IFC 路径（layout/paint） | 高（风险） | 须两处都 cap（§2.2），slice 1 验证 |
| 文本布局全局回归 | 高（风险） | product-smoke + legacy 守（welcome/morning 文本） |

**最大风险**：双路径不一致（layout cap / paint 不 cap）→ box 高 N 行但 paint 全量行溢出。slice 1 专门验证此点（render 对比第 N 行后是否空）。

---

## 5. 验收（slice 1 → slice 2）

- **FR-001 cap**：`line-clamp:4` 块（5 行内容）→ IFC `lines.len()==4`、box height==4 行高。验证：unit test + render bbox。
- **FR-002 零回归**：非 clamped 块 / 无 line-clamp 块 → 逐字节不变。验证：`make test` + product-smoke + legacy 零 delta。
- **FR-003 双路径**：paint 期 IFC 也 cap（render 第 N 行后无内容）。验证：render 对比。
- **FR-004 ellipsis**（slice 2）：第 N 行末有 `…` glyph；内容 + `…` 超宽时截断为 `…` 让位。验证：unit test + css-overflow line-clamp reftest A/B net 正向。
- **FR-005 kill-switch**：`ZW_LINE_CLAMP=0` → cap 不触发（逐字节等同无 line-clamp）。验证：env A/B。

---

## 6. Spec Lint（rally 自主放行）

- 结构完整性：执行摘要 ✅ / 验收场景 ✅（FR-001~005）/ 异常路径 ✅（FR-002 零回归、FR-005 kill-switch）/ 测试绑定 ✅。
- 一致性：范围边界 ✅（vertical/multicol/string-ellipsis 显式排除 slice 3）；双路径风险显式（§2.2/§4）。
- 实现来源闭合：IFC cap（仓内 inline/mod.rs）/ ellipsis（复用 text.rs measure+add_glyph，新增 vertical 分支）/ line_clamp parse（已就位 R2430 实证）。
- **门禁**：Fail = 0 → 放行实施（rally 模式覆盖「等用户确认」，R2407 先例）。

---

## 7. 实施交接

**首批改动顺序**：
1. `crates/layout-engine/src/inline/mod.rs`：IFC struct 加 `line_clamp: Option<usize>` + `clamped: bool`；`layout()` 读 `style.line_clamp`（Count(n)→Some(n)）+ break 后 cap + kill-switch。
2. unit test：cap 行数 + 零回归（非 clamped 不变）。
3. 验证双路径（render 第 N 行后空）+ 全量门禁（make test + product-smoke + legacy）。
4. → slice 2：painter text.rs vertical-triggered ellipsis。

**允许修改**：`crates/layout-engine/src/inline/**`、`crates/engine/src/paint/painter/text.rs`、相关 tests。
**禁止修改**：css-parser line_clamp parse（已就位）；style-system apply（已就位）。
