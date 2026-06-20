# 归档：IFC 统一技术参考（R69+ 代码级上下文）

**归档日期**：2026-06-22（R398 doc-maintenance 治理轮）
**归档原因**：本节原为「R69+ 执行 agent 提供精确代码级上下文」。R69 距今约 320 轮，其「三套 IFC 路径 / override 缺口 / 已穷尽路径 / 完成度清单 / Taffy Fork 状态」等代码级细节已被 R305+ plateau 框架（master.md 顶部「综合裁决」表）+ 现代 Phase A 设计文档（[`../phase-a-IFC-unification-design.md`](../phase-a-IFC-unification-design.md)）取代。代码级 file:line 锚点随代码演进可能漂移，作为活跃参考价值衰减，故迁出归档（内容 100% 保留）。无入站引用（归档前核查：仅 master.md 自引）。

> 本文件为 master.md `## IFC 统一技术参考` 节的 verbatim 迁出，只追加、不修改。

---

## IFC 统一技术参考

> 本节为 R69+ 执行 agent 提供精确的代码级上下文，避免重复探索。

### 三套 IFC 运行路径

当前系统在三个不同时机运行 IFC，使用不同的上下文参数：

| # | 函数 | 文件:行 | styles | 时机 | 作用 |
|---|------|---------|--------|------|------|
| ① | `measure_text_content` | `engine.rs:1462` | 真实 `styles` | taffy 布局中（measure callback） | 返回 `Size{width, height}`，taffy 据此计算块级位置 |
| ② | `remeasure_text_with_float_exclusions` | `engine.rs:2150` | 真实 `styles` | taffy 布局后（step 6） | 重新 IFC + float exclusion，存储 5 个 override map，更新容器高度（shrink） |
| ③ | `paint_text` → IFC | `text.rs:884` | **空 `HashMap::new()`** | paint 阶段 | 生成 glyph 图元，依赖 5 个 override map 回退字体度量 |

**当前 use_stored 路径**：`text.rs:788` 检查 `box_node.inline_layout.is_some()` 且宽度匹配时直接复用存储的 IFC 片段位置，不运行 IFC ③。此路径代码就绪但 `inline_layout` 当前始终为 `None`（因为 `compute_final_inline_layouts` 在 `engine.rs:198` 被注释禁用）。

### Paint IFC override 覆盖缺口

`collect_inline_items`（`inline/mod.rs:708`）在 styles 为空时通过以下 override 回退。**粗体**标记影响行断（字符宽度）的属性：

| 属性 | 覆盖机制 | 覆盖状态 | 影响 |
|------|---------|---------|------|
| **`font_size`** | `font_size_overrides[parent_id]` → 16px | ✅ 已覆盖 | **宽度 + 行高** |
| `line_height` | `line_height_overrides[parent_id]` → fs×1.2 | ✅ 已覆盖 | 仅行高 |
| **`letter_spacing`** | `letter_spacing_overrides[parent_id]` → 0 | ✅ 已覆盖 | **宽度** |
| **`word_spacing`** | 无覆盖机制 | ❌ 始终为 0 | **宽度** |
| **`is_ahem_font`** | `is_ahem_overrides[parent_id]` → false | ✅ 已覆盖 | **宽度** (0.55fs vs 1.0fs) |
| `vertical_align` | 无覆盖机制 | ❌ 始终为 Baseline | 行盒对齐 |
| **`margin_left/right`** (inline 元素) | 无覆盖机制 | ❌ 始终为 0 | **宽度** |
| `padding/border` (inline 元素) | 无覆盖机制 | ❌ 始终为 0 | 行高 |

### 已穷尽的不可行路径（R37-R68 共 32 轮）

以下所有路径均已尝试并回退，**R69+ 不需要重试**：

| 路径 | 结果 | 根因 |
|------|------|------|
| 修改 glyph advance (render_fs) | 回归 | 字形推进与 IFC 片段位置不一致 |
| 传递完整 styles 到 paint IFC | 回归 -5~-6 | 行断行为改变 |
| 存储 layout IFC 结果（所有变体） | 回归 -4~-6 | 存储 IFC 上下文与 paint 时不同 |
| font_size_overrides 启用 | 零改进（R45）/ 可能回归 | 行断变化 |
| is_ahem glyph advance 修改 | 回归 -2~-3 | 字形推进与 IFC 行断不一致 |
| letter_spacing_overrides 启用 | 零改进 | — |
| line_height_overrides 启用 | 零改进 | 仅影响垂直，不影响水平 |
| inline_element_metrics 启用 | 零改进 | 仅影响垂直 |
| default_font_metrics 传递 | 回归 -6 | font_size 变化导致行断不一致 |
| taffy measure callback IFC 缓存 | 回归 -5 | 多次 measure 调用的 available_space 不同 |
| 外边缘边框完整厚度 | 回归 -2 | taffy 单元格定位冲突 |

### 存储 IFC vs Paint IFC 基线计算差异

存储 IFC 的 fragment 基线位置计算与 paint IFC 的 glyph 基线位置不同：

```
存储 IFC：baseline_y = frag.y + frag.height     （line-height 盒底边）
paint IFC：baseline_y = frag.y + font_size      （当前 paint 使用）
差值 = (line_height - font_size) / 2            （半行距）
```

启用 `use_stored` 路径时，需要统一基线计算。推荐以存储 IFC 的 `frag.y + frag.height` 为准（CSS 规范中 line-height 半行距分布在文字上下）。

### IFC 统一完成度检查清单

```
✅ LayoutBox.inline_layout: Option<Vec<InlineLayoutLine>>    (engine.rs:167)
✅ LayoutBox.inline_layout_width: f32                          (验证容器宽度匹配)
✅ compute_final_inline_layouts() 函数实现                    (engine.rs:1175)
✅ paint 侧 use_stored 路径                                   (text.rs:788)
✅ store_font_sizes_from_ifc() 5 个 override map              (engine.rs:874)
✅ remeasure 高度收缩 → sibling reflow (shrink)               (R68)
❌ compute_final_inline_layouts 启用                          (engine.rs:198 被注释)
❌ remeasure 高度增长 → sibling reflow (grow)                  (仅处理 shrink)
❌ table/multicol 后处理后重新运行 IFC                         (宽度可能改变)
❌ 存储 IFC vs paint 基线计算对齐                             (frag.height vs font_size)
```

### Taffy Fork 状态

项目已 fork taffy 0.7.7 到 `crates/taffy-local/`（~16,400 行），通过 workspace `[patch.crates-io]` 替换 crates.io 版本。当前仅有一个自定义补丁：

- `cached_baselines()` 访问器（`cache.rs:187`, `taffy_tree.rs:853`）— 暴露 taffy 内部缓存的 `first_baselines`，供 inline-flex/inline-grid 基线提取

**结论**：不需要深度修改 taffy。IFC 统一通过在 remeasure 后处理阶段（taffy 布局完成后）存储完整 IFC 结果并传播高度变化来实现，不涉及 taffy 内部算法变更。
