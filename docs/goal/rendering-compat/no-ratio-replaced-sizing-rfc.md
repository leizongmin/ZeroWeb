# Spec + RFC：no-ratio 替换元素尺寸（CSS §10.3.2）

**版本**：v1.0
**日期**：2026-07-14
**作者**：AI Assistant（rally autonomous）
**状态**：已确认（rally 自主执行，承接 R1436 root cause）

---

## 0. 执行摘要

- **一句话目标**：修复无固有宽高比的 SVG（`width` XOR `height` 属性、无 viewBox）作为 `<img>` 替换元素时被错误按 `w/h` 强制比例缩放的 bug，使其符合 CSS §10.3.2 no-ratio 算法。
- **本期范围**：仅 `<img>`（无 HTML width/height 属性、解码自 SVG）的 no-ratio 替换尺寸；新增 no-ratio 信号从 `image_cache` 经 `pipeline` → `layout-engine` → `tree.rs` 透传并消费。
- **明确排除**：`<video>`/`<iframe>`/`<canvas>`；HTML width/height 属性显式给定的替换元素（既有分支不变）；`apply_indefinite_percent_height_to_auto`（engine.rs:1014）的第二处 aspect_ratio 设置（pre-existing、无 driving reftest，留 forward）；inline SVG 形状渲染（goal line 118 OOS）。
- **核心约束**：① 默认 object size = 300×150（CSS §10.3.2）；② no-ratio 案**不得**设 `aspect_ratio`；③ 既有 both-abs SVG / ratio-only SVG / PNG / JPEG 路径零回归；④ 全 corpus img A/B net≥0、0 pass→fail。
- **推荐方案**：新增 `image_no_ratio: HashMap<u64, (Option<f32>, Option<f32>)>`（真实固有 w/h，None=无该维固有尺寸），与既有 `image_sizes`/`image_ratios` 并列透传；`tree.rs` no-HTML-attr 分支优先消费 no-ratio。
- **首个落地步骤**：`render-foundation/src/image_cache.rs` 把 `svg_intrinsic_ratio` 重构为 `svg_intrinsic_kind`（enum BothAbs / RatioOnly / NoRatio），在 `ImageData` 加 `no_ratio` 字段。

---

## 1. 背景与目标

### 1.1 背景

R1436 已确证 root cause：`width-50-no-ratio.svg`（`width="50"`、无 height、无 viewBox）经 usvg 解码得 intrinsic `(50, 100)`（usvg 对缺 height 的 SVG 默认 h=100）；`pipeline.build_img_intrinsic_sizes` → `tree.rs:389-390` 无条件 `aspect_ratio = w/h = 0.5`；CSS `height:20 width:auto` → taffy `width = 20×0.5 = 10`（应 **50**）。

`img_intrinsic_ratios` 仅含 ratio-only SVG（`svg_intrinsic_ratio` 返回 `Some` 的 viewBox 案），width-50-no-ratio 不在其中（`svg_intrinsic_ratio` 返回 `None`）→ 落入 `img_intrinsic_sizes`，tree.rs 无信号区分「真双 abs dim（ratio 有效）」vs「单 abs dim no-ratio（h 是 usvg 默认，ratio 无效）」。

### 1.2 目标

- 业务目标：visudet replaced-elements 簇（height-20 / width-40 / max-height-20 / max-width-40 共 4 案及 min 变体）从 FAIL 翻 PASS，对齐 Chromium Oracle。
- 用户目标：无固有宽高比图片按 CSS 规范正确尺寸化，logo/图标不变形。

### 1.3 范围边界

- **在范围内**：SVG 单 abs dim 或零 abs dim（无 viewBox）作为 `<img>` 替换元素的尺寸算法；no-ratio 信号端到端透传。
- **不在范围内**：非 SVG 图像（PNG/JPEG/both-abs SVG 走既有 sizes 路径，不变）；ratio-only SVG（既有 ratios 路径，不变）；HTML 属性显式 width/height（既有分支，不变）；engine.rs:1014 第二处 aspect_ratio 设置。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | R1435/R1436 root cause + visudet reftest |
| 功能需求 | 是 | §3 |
| 非功能需求 | 是 | §4（零回归） |
| 接口需求 | 是 | §5（内部数据结构） |

---

## 3. 功能需求

### FR-001：no-ratio SVG 信号识别

- **描述**：当 SVG 根元素 width/height 非双绝对（缺失/百分比/auto）且无可用 viewBox 宽高比时，系统必须将其标记为 no-ratio，并记录其真实固有宽高（仅 abs 属性存在的维，缺失维为 None）。
- **优先级**：必须
- **来源**：R1436 root cause + CSS §10.3.2

**验收场景**：

```
场景: width-only no-ratio SVG 识别
  假设 SVG 为 <svg width="50">（无 height、无 viewBox），usvg 解码得 (50,100)
  当 decode_svg_bytes 解码该 SVG
  那么 ImageData.intrinsic_ratio == None 且 no_ratio == Some((Some(50.0), None))
  验证: image_cache 单测 test_svg_no_ratio_width_only

场景: height-only no-ratio SVG 识别
  假设 SVG 为 <svg height="25">（无 width、无 viewBox）
  当 decode_svg_bytes 解码
  那么 no_ratio == Some((None, Some(25.0)))
  验证: image_cache 单测 test_svg_no_ratio_height_only

场景: 零维 no-ratio SVG 识别
  假设 SVG 为 <svg>（无 width/height/viewBox）
  当 decode_svg_bytes 解码
  那么 no_ratio == Some((None, None))
  验证: image_cache 单测 test_svg_no_ratio_none

场景: both-abs SVG 不被误判为 no-ratio
  假设 SVG 为 <svg width="40" height="40">
  当 decode_svg_bytes 解码
  那么 no_ratio == None 且 intrinsic_ratio == None（走 image_sizes 既有路径）
  验证: image_cache 单测 test_svg_both_abs_not_no_ratio

场景: ratio-only SVG 不被误判为 no-ratio
  假设 SVG 为 <svg viewBox="0 0 100 50">（无 width/height）
  当 decode_svg_bytes 解码
  那么 no_ratio == None 且 intrinsic_ratio == Some(2.0)（走 image_ratios 既有路径）
  验证: 既有 svg_intrinsic_ratio 单测保持
```

### FR-002：no-ratio 替换元素尺寸算法

- **描述**：对 no-ratio `<img>`（无 HTML width/height 属性），系统必须按 CSS §10.3.2 计算 used size：**不设 aspect_ratio**；auto 侧使用真实固有尺寸（若有），否则默认 object size（宽 300 / 高 150）；显式 CSS 侧（含 Px / min-max 钳制）由 converter+taffy 处理。
- **优先级**：必须
- **来源**：CSS 2.1 §10.3.2/§10.6.2 + css-sizing-3 §5.1 + visudet reftest refs

**算法表**（`(w_opt, h_opt)` = 真实固有宽高，各 Option；default 300×150）：

| CSS width | CSS height | used width | used height |
|-----------|-----------|------------|-------------|
| auto | auto | w_opt.unwrap_or(300) | h_opt.unwrap_or(150) |
| 显式 | auto | （CSS width） | h_opt.unwrap_or(150) |
| auto | 显式 | w_opt.unwrap_or(300) | （CSS height） |
| 显式 | 显式 | （CSS width） | （CSS height） |

min/max-width/height 由 converter 设入 taffy，对上述 used size 独立钳制。

**验收场景**（visudet replaced-elements reftest，`--base-dir` 加载 SVG）：

```
场景: height:20px（height 显式、width auto）
  假设 img5 = width-50-no-ratio.svg（w_opt=Some(50), h_opt=None）
  当 CSS img{height:20px}
  那么 渲染宽度 = 50（intrinsic width），高度 = 20
  验证: make reftest DIR=css/CSS2/visudet replaced-elements-height-20 → PASS

场景: height:20px + 无 intrinsic width
  假设 img7 = no-ratio.svg（w_opt=None, h_opt=None）
  当 CSS img{height:20px}
  那么 渲染宽度 = 300（default），高度 = 20
  验证: 同上 replaced-elements-height-20（img4/img7 span width 300）

场景: width:40px（width 显式、height auto）
  假设 img5 = width-50-no-ratio.svg（w_opt=Some(50), h_opt=None）
  当 CSS img{width:40px}
  那么 渲染宽度 = 40，高度 = 150（default object height）
  验证: make reftest replaced-elements-width-40 → PASS

场景: width:40px + 有 intrinsic height
  假设 img4 = height-25-no-ratio.svg（w_opt=None, h_opt=Some(25)）
  当 CSS img{width:40px}
  那么 渲染宽度 = 40，高度 = 25（intrinsic height）
  验证: 同上 replaced-elements-width-40

场景: all-auto + 有 intrinsic width
  假设 img5 = width-50-no-ratio.svg
  当 CSS width/height 均 auto
  那么 渲染 50×150（intrinsic width × default height）
  验证: make reftest replaced-elements-all-auto → PASS

场景: max-height:20px（height auto、max-height 钳制）
  假设 img5 = width-50-no-ratio.svg
  当 CSS img{max-height:20px}
  那么 渲染 50×20（width=intrinsic 50，height=default 150 被 max-height 钳到 20）
  验证: make reftest replaced-elements-max-height-20 → PASS

场景: max-width:40px（width auto、max-width 钳制）
  假设 img5 = width-50-no-ratio.svg
  当 CSS img{max-width:40px}
  那么 渲染 40×150（width=intrinsic 50 被 max-width 钳到 40，height=default 150）
  验证: make reftest replaced-elements-max-width-40 → PASS

场景: both-abs / ratio-only SVG 不回归
  假设 img1=height-25-width-50.svg（both-abs），img2/img3=ratio-2 系
  当 同一组 reftest
  那么 这些 img 渲染与基线字节一致（走既有 sizes/ratios 路径）
  验证: 全 corpus img A/B net≥0、0 pass→fail
```

---

## 4. 非功能需求

### NFR-001：零回归
- **描述**：既有 both-abs SVG / ratio-only SVG / PNG / JPEG `<img>` 尺寸与背景图 `image_sizes` 消费路径字节一致。
- **测量标准**：全 corpus reftest A/B net≥0、0 pass→fail；product-smoke welcome diff 不升。
- **优先级**：必须

### NFR-002：最小改动面
- **描述**：`compute_with_img_sizes` 公开签名不变（测试零改动）；no-ratio 经新方法 `compute_with_img_intrinsic` 注入。
- **优先级**：应该

---

## 5. 接口需求（内部数据结构）

### IF-001：`ImageData.no_ratio`
- **类型**：内部字段
- **规格**：`no_ratio: Option<(Option<f32>, Option<f32>)>` — `Some((w, h))` 仅 no-ratio SVG；`w`/`h` 为真实固有宽高（abs 属性维，缺失维 None）。`pub fn no_ratio_intrinsic(&self) -> Option<(Option<f32>, Option<f32>)>`。
- **默认动作**：非 no-ratio 图像为 None。

### IF-002：pipeline / layout-engine no-ratio map
- **类型**：内部 HashMap
- **规格**：`image_no_ratio: HashMap<u64, (Option<f32>, Option<f32>)>`（URL-hash 键，pipeline）；`img_intrinsic_no_ratio: HashMap<NodeId, (Option<f32>, Option<f32>)>`（DOM 键，layout-engine）。`set_image_no_ratio` / `set_img_intrinsic_no_ratio` setter；`build_img_intrinsic_no_ratio(doc)` 解析器。

---

## 6. 约束与假设

### 6.1 必须约束（Must）
- no-ratio 案不得设 `taffy_style.aspect_ratio`。
- 默认 object size 宽 300、高 150。
- 既有 sizes/ratios 路径与背景图 `image_sizes`（pixmap 尺寸）消费不变。

### 6.2 禁止约束（Must Not）
- 不得改 `compute_with_img_sizes` 公开签名（保持测试零改动）。
- 不得把 no-ratio 图像从 `image_sizes` 移除（背景图 `background-size:auto` 仍读 pixmap 尺寸）。
- 不得为 no-ratio 设 aspect_ratio 后再「抵消」（须直接不设）。

### 6.3 已定决策
- 信号以独立第三 map `image_no_ratio` 透传，不混入 sizes/ratios。
- 生产入口新增 `LayoutEngine::compute_with_img_intrinsic(.., no_ratio)`；`compute_with_img_sizes` 委托（no_ratio=empty）。
- no-ratio 真实固有维取自 SVG 属性解析值（与 usvg pixmap 该维一致）。

### 6.4 技术约束
- 仅水平书写模式（vertical 模式轴互换，保守跳过，与既有 R1363 门控一致）。

### 6.5 假设
- usvg 对 `width="50"`（无 height）解码 pixmap 宽=50（R1435 实测 w=10=20×0.5 反推 (50,100)）— 状态：已验证。
- 默认 object size 300×150 为 CSS §10.3.2 / SVG2 规范值 — 状态：已验证（reftest refs 一致）。

### 6.6 代码变更边界
- **允许修改**：`crates/render-foundation/src/image_cache.rs`、`crates/engine/src/pipeline.rs`、`crates/engine/src/pipeline_budget.rs`、`crates/layout-engine/src/engine.rs`、`crates/layout-engine/src/tree.rs`、`crates/webview/src/webview.rs`、`tests/wpt-runner/src/reftest/resources.rs`、`tests/wpt-runner/src/reftest.rs`、对应 `#[cfg(test)]` 单测。
- **禁止修改**：`apply_indefinite_percent_height_to_auto`（engine.rs:1014）第二处 aspect_ratio — 原因：无 driving reftest、pre-existing，留 forward。

---

## 7. 实施交接（Implementation Handoff）

### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险 |
|----------|------|------|------|
| `render-foundation/src/image_cache.rs` | 修改 | `svg_intrinsic_kind` enum + `ImageData.no_ratio` 字段 + 单测 | 低（重构私有 fn） |
| `engine/src/pipeline.rs` | 修改 | `image_no_ratio` 字段 + setter + builder + 4 处 render 入口注入 | 中（pervasive） |
| `engine/src/pipeline_budget.rs` | 修改 | 2 处 render 入口注入 no_ratio | 中 |
| `layout-engine/src/engine.rs` | 修改 | `compute_with_img_intrinsic` + 透传 build_layout_tree_with_r109 | 低 |
| `layout-engine/src/tree.rs` | 修改 | BuildContext 字段 + apply_replaced_element_sizing no-ratio 分支 + 单测 | 中（核心算法） |
| `webview/src/webview.rs` | 修改 | fetch_image_subresources 建 no_ratio + cached + sync | 低 |
| `tests/wpt-runner/src/reftest/resources.rs` | 修改 | extract_image_metrics 返回 no_ratio | 低 |
| `tests/wpt-runner/src/reftest.rs` | 修改 | 2 处 set_image_no_ratio | 低 |

### 推荐修改顺序

1. `image_cache.rs`：重构 `svg_intrinsic_kind` + `no_ratio` 字段 + 单测（验证 FR-001）。
2. `pipeline.rs` + `webview.rs` + `reftest/resources.rs`+`reftest.rs`：no_ratio map 透传到 pipeline（不接 layout，先编译过）。
3. `engine.rs` + `tree.rs`：`compute_with_img_intrinsic` + `apply_replaced_element_sizing` no-ratio 分支 + 单测（验证 FR-002 算法）。
4. 6 处 render 入口接 `compute_with_img_intrinsic`。
5. A/B：`make reftest DIR=css/CSS2/visudet` + 全 corpus img 回归 + product-smoke。

### 首批提交建议

| 提交 | 范围 | 预期结果 | 验证 |
|------|------|----------|------|
| Commit 1 | no-ratio replaced sizing 全链路 | visudet replaced 4+案 FAIL→PASS、全 corpus net≥0 | `make reftest DIR=css/CSS2/visudet` + `make test` + clippy + product-smoke |

---

## 8. 技术设计（RFC）

### 8.1 现状分析

- `decode_svg_bytes`（image_cache.rs:444）调 `svg_intrinsic_ratio`（返回 `Option<f32>`）：both-abs→None、ratio-only→Some、no-ratio→None。no-ratio 与 both-abs 都返 None，落入 `image_sizes`（pixmap 尺寸，no-ratio 的缺失维是 usvg 默认 100）。
- `tree.rs:380` no-HTML-attr 分支读 `img_intrinsic_sizes`，无条件 `aspect_ratio = w/h`（line 389），对 no-ratio 用 bogus 比例。
- `img_intrinsic_ratios`（R717）只承载 ratio-only，no-ratio 无独立信号。

### 8.2 目标状态

- `svg_intrinsic_kind` 三态枚举区分 both-abs / ratio-only / no-ratio；no-ratio 携带真实固有维 `(Option<f32>, Option<f32>)`。
- no-ratio 图像**额外**入 `image_no_ratio`（仍留 `image_sizes` 供背景图）。
- `tree.rs` no-HTML-attr 分支优先消费 `img_intrinsic_no_ratio`：不设 aspect_ratio，按算法表设 auto 侧 used size。

### 8.3 数据流

```
decode_svg_bytes (image_cache.rs)
  → svg_intrinsic_kind(bytes) → BothAbs | RatioOnly(r) | NoRatio{w,h}
  → ImageData { intrinsic_ratio: Option<f32>, no_ratio: Option<(Opt,Opt)> }

fetch_image_subresources (webview) / extract_image_metrics (reftest)
  → ratio Some  → image_ratios
  → no_ratio Some → image_no_ratio   ← NEW
  → else        → image_sizes（pixmap，含 no-ratio 供背景图）

pipeline.build_img_intrinsic_no_ratio(doc) → HashMap<NodeId,(Opt,Opt)>
  → LayoutEngine::compute_with_img_intrinsic(.., img_no_ratio)
  → build_layout_tree_with_r109(ctx.img_intrinsic_no_ratio)
  → apply_replaced_element_sizing: no-ratio 分支（优先于 sizes/ratios）
```

### 8.4 核心算法伪代码（tree.rs no-HTML-attr `_` 分支，置最前）

```
if let Some(&(w_opt, h_opt)) = img_intrinsic_no_ratio.get(&dom_id) {
    let width_auto  = matches!(computed.width, Auto);
    let height_auto = matches!(computed.height, Auto);
    // 不设 aspect_ratio（no-ratio）
    if width_auto && height_auto {
        taffy_style.size.width  = length(w_opt.unwrap_or(300.0).max(0.5));
        taffy_style.size.height = length(h_opt.unwrap_or(150.0).max(0.5));
    } else if !width_auto && height_auto {
        // width 显式，height auto → default/intrinsic height
        taffy_style.size.height = length(h_opt.unwrap_or(150.0).max(0.5));
    } else if width_auto && !height_auto {
        // height 显式，width auto → default/intrinsic width
        taffy_style.size.width = length(w_opt.unwrap_or(300.0).max(0.5));
    }
    // 两侧显式：converter 处理
    return; // 跳过 sizes/ratios 分支（互斥）
}
// 既有 sizes 分支（both-abs）... 既有 ratios 分支（ratio-only）...
```

> flex row/col skip：no-ratio 无 aspect_ratio，taffy 无法 ratio-derive cross，故不套用 R1363 skip（必须设 auto 侧 definite）。visudet 案为 block 上下文，全 corpus A/B 守回归。

### 8.5 安全考虑
- 无安全影响（纯布局尺寸计算）。

### 8.6 替代方案

| 方案 | 优点 | 缺点 | 决定 |
|------|------|------|------|
| A. 独立第三 map `image_no_ratio` | 自包含、不污染 sizes/ratios、背景图不受影响 | 多一条透传链 | ✅ 选定 |
| B. no-ratio 并入 `image_sizes` + 哨兵 0 | 少一条链 | 污染 R695/背景图读 sizes；哨兵脆弱 | ❌ 拒绝 |
| C. `compute_with_img_sizes` 加第 3 位置参 | 一致 | 30+ 测试站点 churn | ❌ 拒绝（改用 IF 委托） |

### 8.7 测试策略
- 单测：`image_cache` 4 案 no-ratio 识别（FR-001）；`tree.rs`/engine 算法单测（FR-002 各 CSS 组合）。
- reftest：`make reftest DIR=css/CSS2/visudet`（visudet 簇）+ 全 corpus img A/B（net≥0）。
- product-smoke：welcome diff 不升。

### 8.8 回滚计划
- 单 commit，net 负即整体 revert（git revert），无部分留存。

---

## 9. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 |
| 场景存在性 | ✅ Pass | FR-001 5 场景、FR-002 8 场景 |
| 异常路径覆盖 | ✅ Pass | FR-001 含 both-abs/ratio-only 误判否定场景；FR-002 含 max 钳制 |
| 测试绑定 | ✅ Pass | 每场景绑定单测名 / make reftest |
| TBD 清零 | ✅ Pass | 无阻塞性 TBD |
| 实施交接完备 | ✅ Pass | §7 文件清单+顺序+首批提交 |
| 首步可执行性 | ✅ Pass | §7 步骤 1 |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ Pass | 用「设/不设/返回」具体动词 |
| 无量化描述 | ✅ Pass | 300/150/max(0.5) 量化 |
| 非确定性措辞 | ✅ Pass | 「必须/不得」 |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 与 FR 无交集 |
| 约束冲突 | ✅ Pass | Must/Must Not 无矛盾 |
| 代码边界完备 | ✅ Pass | §6.6 允许/禁止 |
| 实现来源闭合 | ✅ Pass | svg_intrinsic_kind 仓内自实现（image_cache.rs） |
| 清单数量一致 | ✅ Pass | 8 文件与 §7 一致 |

**汇总**：15 Pass / 0 Warning / 0 Fail / 0 Skip
**门禁判定**：允许实施

---

## 10. 待定列表

| ID | 项目 | 优先级 | 下一步 |
|----|------|--------|--------|
| TBD-1 | engine.rs:1014 第二处 aspect_ratio（height:% + indefinite CB） | 可选 | A/B 若暴露再扩 no-ratio 信号 |

---

## 11. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-07-14 | 初始版本（承接 R1436） |
