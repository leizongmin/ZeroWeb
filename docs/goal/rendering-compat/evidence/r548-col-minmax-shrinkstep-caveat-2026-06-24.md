# R548 col/colgroup min/max-width — shrink-step min-width caveat（doc-side 代码分析）

**日期**: 2026-06-24
**性质**: read-only 代码核查（doc-maintenance），未改代码；供 R548 落地后验证参考
**相关**: master.md item 4 col/colgroup lever（seam `crates/layout-engine/src/table.rs:1131` `resolve_col_width`）

## 背景

并行 code agent 正在实现 item 4 钉住的 col/colgroup min/max-width lever（table.rs 未提交，
跨多轮 doc-maintenance）。本轮 doc-side 读 `compute_column_widths` 全函数核查实现完整度与 CSS 正确性。

## 实现核查结论（完整且正确——对宽表）

agent 的实现（table.rs `compute_column_widths`）：
- 新增 `resolve_col_min`/`resolve_col_max` 闭包（读 `s.min_width`/`s.max_width` 的 `Px` 分支）。
- 在**全部 3 个 col 处理分支**应用：① colgroup（`TableColumnGroup`）无内 col → gmin/gmax；
  ② colgroup 含内 col → 每 inner col 的 cmin/cmax；③ 裸 col（`TableColumn`）→ cmin/cmax。
- min → `col_max_widths[idx].max(mn)`（下限），max → `.min(mx)`（上限）。语义正确。

→ 对 **applies-to-005/006**（colgroup/col `min-width:1in`，宽表）会翻转 PASS（宽表不收缩，min-width 作为下限生效）。
精确匹配 item 4 seam 预测。实现质量好。

## ★ 正确性 caveat（收缩场景 min-width 非硬下限）

`compute_column_widths` **只有一个列宽数组 `col_max_widths`**（无独立 min-content 数组），
min-width 被烘进 `col_max_widths`。函数末尾的比例收缩步（约 line 298-301）：

```rust
if has_explicit_width && !fixed_capped && total_width > ew {
    let ratio = ew / total_width;
    for w in &mut col_max_widths {
        *w *= ratio;   // 比例收缩——含 min-width 提升过的列
    }
}
```

**问题**：当表有显式窄宽（`has_explicit_width`）且列总和超出（`total > ew`）时，**所有列按 `ratio`
等比收缩，包括被 min-width 提升过的列** → min-width 被等比缩小，**不再是硬下限**。

CSS Tables §17.5 + CSS §10.4：min-width 应是列宽的硬下限（收缩时不可低于，除非内容 min-content 要求更小且
min-width:auto）。当前实现对「显式窄宽 + col/colgroup min-width」场景违反此约束。

## 影响范围

- **applies-to-005/006（10 案）= 宽表** → 不触发收缩步 → min-width 生效 → **会 PASS**（R548 主目标达成）。
- **未测边缘 case** = 显式窄宽表 + col/colgroup min-width → 收缩后 min-width 被违反。
  WPT css-tables 现有失败集（35 案）是否含此类收缩+min-width 用例待 R548 A/B 后确认。
- 若 R548 A/B 出现收缩场景回归（css-tables 全量 35 fail 中新增），此 caveat 即根因。

## 建议（供 R548 验证 / follow-up）

1. R548 落地后跑 css-tables 全量 A/B：确认 applies-to-005/006 翻转 + **零回归**。
2. 若有回归且为收缩场景：min-width 须在收缩步**之后**再次 `.max(mn)` re-clamp（或收缩时跳过
   min-width-raised 列，把 deficit 分摊给无 min-width 的列）。
3. 此 caveat 不阻塞 R548 主 yield（10 案 applies-to），但属 CSS 完整性 follow-up。

## 不改 master.md 的原因

agent 正在活跃编辑 master.md（R548 条目 + 可能更新 item 4 标 LANDED）。本分析写 evidence 目录
（additive、永不冲突），待 R548 落地后据其 A/B 结果决定是否回写 master.md follow-up。
