# R4537 证据：三修落地深挖——draw_order 重放机制纳入根因链（P 轮，实验未收敛回退）

- 日期：2026-09-20（R4537 P 轮——R4536 下轮方向 Fix B/A/C，0 net code 落地）
- 前置：R4536 三缺陷（A path_fill 改写 z 序倒挂 / B 条带顶采样漏带 / C 嵌套快照范围）。

## 本轮新发现（决定性）

- **渲染默认路径 = DrawOp 重放**：`render_full_scene` → `render_full_scene_region`
  → `render_draw_order`（cpu/mod.rs:580）按 `primitives.draw_order` 逐 op 光栅化；
  `DrawOp::Fill(i)` → `fills.get(i)`。**直接 push 进 typed 向量而不登记 DrawOp 的
  图元不会被绘制**（border.rs 注释「draw_order 是默认渲染路径」的系统性含义）。
- **逐探针验证链**：Fix B（中点采样）+ Fix A（原位条带 splice）后，paint() 末尾
  图元列表完全正确（92 fills：f[0] 红底 + f[2..] 绿色 1px 条带），渲染结果仍全红
  → 条带被 render_draw_order 丢弃（无 DrawOp 登记）。
- **Fix A 改 draw-order 重建版**：原位清零 + 条带尾插 + 依改写前快照整表重建
  draw_order——重建后仅 39 ops / 190 fills（ops 覆盖 y 2..25 即顶臂带，y≥30 的
  宽带 fills 无 op）→ 组合页仍只渲染顶臂。**嵌套场景（父子两层 clip 消费叠加）
  的 op 改写交互仍有一层未解码**：父消费的 rewrite_targets 含子条带（父范围覆盖
  子输出），双重改写在重建后仍有 op 缺失。
- **机制面补充**：`clip_all_primitives_to_polygon` 的 drain/re-add（既有代码，
  非本轮引入）对 DrawOp::Fill(i≥range) 的索引同样具有破坏性——既有 clip-path
  语料通过系 path_fill 改写（追加 op）掩盖；strip 化后该不变式问题显形。

## 结论

三修的正确落地顺序必须以 **draw_order 不变式**为纲：
1. clip_fill_to_polygon 条带化（Fix B，几何已验证 ✓）；
2. 条带 op 的登记与原 op 替换需一次性重建（本轮重建版框架正确，嵌套双消费的
   op 缺失需再一轮定位——疑点：父消费收集时子条带 op 已在 old_ops 中但部分
   丢失，需在 rebuild 后断言 `draw_order.len() == 预期` 并打印差集定位）；
3. z 序随 op 位置天然保序 ✓。
全链收敛后 corner-shape 凹角簇 ~10 案 + border-shape slice 2 overflow 裁剪同步
解锁。本轮按零新红纪律整体回退，树态还原 R4535（= R4534 干净基线）。

## 调试资产（下轮复用）

- 探针页：/tmp/zwprobe/clip{,2,3,5}-probe.html（单 clip / 嵌套 / 父+纯子 / 真div）。
- 开关：`ZW_CLIP_DEBUG=1`（clip 消费 in/out 计数）、`ZW_PRIM_DEBUG=1`（paint 末尾
  fills/draw_order 全量 dump）、`REFTEST_DUMP=1`（test/ref PNG 落盘
  target/reftest-dump/）。
- 排障入口：render_draw_order（cpu/mod.rs:580）+ clip-path Polygon 臂
  （painter/mod.rs ~2895）+ clip_all_primitives_to_polygon（helpers.rs:593）。
