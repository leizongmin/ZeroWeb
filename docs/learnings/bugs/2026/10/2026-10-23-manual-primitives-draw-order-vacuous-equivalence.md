---
date: 2026-10-23
modules: render-foundation
---

# 手工构造 RenderPrimitives 的 region/等价测试：draw_order 不完整导致断言空转

## 问题

为 render-foundation 的 region 契约（物理像素统一）写 scale=2 的 region==full 逐位等价测试：`RenderPrimitives::new()` + `fills.push` + `add_glyph` 构造图元后渲染对比。该测试在修复前（region 裁剪按 CSS 逻辑与物理 region 错配求交，跨界 fill 泄漏区外——同配置 typed_buckets 探针实证区外为 (255,127,127)）依然 PASS，fail-first 验证失败。

## 根因

CPU 渲染入口按 `draw_order` 是否非空分流（`use_draw_order = env != "0" && !draw_order.is_empty()`），两条路径对手工图元的行为完全不同：

- **无 draw_order** → `render_typed_buckets`：按类型桶遍历全部图元，测试有效（但走的是非生产路径）。
- **部分 draw_order** → `render_draw_order`：只画 `DrawOp` 序列登记的图元。`add_glyph` 只 push 一个 `DrawOp::Glyph`，`fills.push` **不**登记任何 op——未登记的 fills 整体被跳过，两种缓冲都只剩白底+字形，region==full 等价断言**空转通过**。
- `cull_invisible` 补救无效：对手工图元只重建出 glyph op（len=1），fills 同样被丢（生产 fills 的 DrawOp 由 paint 导出管线登记，不是 cull_invisible）。

## 解决

手工 push 完整 draw_order（`DrawOp`/`draw_order` 均为 pub）：

```rust
primitives.draw_order.push(DrawOp::Fill(0));
primitives.draw_order.push(DrawOp::Glyph(0));
```

或反向取舍：删掉 glyph 让 draw_order 保持为空、走 typed_buckets——但要意识到这不再覆盖生产 draw_order 分支。

## 避免方式

1. region/等价/逐位比较类渲染测试，先确认被测图元真的进入了渲染路径（修前探针打印像素值，而非只看断言绿）。
2. **fail-first 是唯一可靠的空转检测**：撤掉修复跑红才算数；断言绿灯不能区分「验证了修复」和「两边都没画」。
3. 涉及多渲染分支（draw_order / typed_buckets）的行为契约，两条路径都该有测试——手工测试默认走的往往不是生产分支。
