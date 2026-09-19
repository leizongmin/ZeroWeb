# R4541 证据：border-shape slice 2——overflow 内容内形状裁剪（border-shape-overflow 翻绿）

- 日期：2026-09-20（R4541 C 轮——R4540 下轮方向①，helpers.rs 2 新 fn + 1 共享化 + painter 1 接线点 + 1 单测）
- 谱系：R4533 蓝图 slice 2 → R4535 定谳可实施 → R4540 v2 条带机制铺底 → 本轮实施。

## 实现

1. **`border_shape_overflow_polygon`**（helpers.rs，css-borders-4 §border-shape）：
   border-shape 元素的 overflow 内容裁剪多边形。fill mode（双形状）= 内形状顶点
   （`inner` + `inner_box` 引用盒解析）；stroke mode（单形状）= 形状在引用盒各侧内缩
   `rel_width/2`（描边居中于形状路径，内缘 = 半宽内缩；width=0 ≡ 形状本身——
   border-shape-overflow 无边框页直接裁到形状）。kill-switch `ZW_BORDER_SHAPE=0` 整体回落。
2. **`border_shape_relevant_side`**：relevant side（§7.6）选取逻辑从 `border_shape_plan`
   机械抽出共用（plan 零行为变化）。
3. **`clip_with_polygon_rewrite`**：R4248 覆盖改写 + R4539 op 原位替换 + R4540 v2 条带
   裁剪统一入口（clip-path polygon 臂与 border-shape overflow 裁剪共用；轴对齐矩形
   多边形守卫内聚）。painter polygon 臂改为薄调用（逻辑零变化）。
4. **painter overflow 裁剪点接线**（paint_node `needs_clip` 块）：padding-box 矩形裁剪
   之后，若元素声明 border-shape → 以内形状多边形走统一入口（覆盖填充改写保 z 序 +
   部分相交条带 v2 渲染）。

## 验证

- **探针**：border-shape-overflow 形页（diamond + overflow:hidden + abspos 绿子）渲染
  = 绿色 diamond，与 SVG ref 逐像素同形；circle(50%) + 10px 黑边框 + 绿子探针 = 黑环
  + 内缘绿圆（内缘 r=40 = half-border r45 − 半宽 5，几何精确）。
- **单测**：`test_border_shape_overflow_polygon_inner_edge`——stroke mode 内缘 diamond
  顶点 (50,10)(90,50)(50,90)(10,50) + fill mode 内形状半径 ≈30。
- **A/B（reftest-upstream 全语料 vs R4540）**：1459 fail，diff = border-shape-overflow
  翻绿（0.04%）+ box-shadow-003 flake 绿相位；零新红。corpus **15135/16594 = 91.20%**
  （R4540 记账 15134 修正 +1——flake 相位归位后净计）。

## 残差记账

- `border-shape-overflow-child-clip`（1.64% 持平）：#with-child 的绿色内容系 flex item
  的 static 子 div——**flex item 子元素不绘制为既有独立 bug**（bscircle4-probe 黄底无
  border-shape 对照页同样缺子，与本轮改动无关），flex 布局域独立挂账。
- `border-shape-overflow-replaced-{iframe,img,self}`（1.64-1.65% 持平）：replaced 元素
  内容绘制路径（ImagePrimitive/iframe 域）未在内形状裁剪覆盖面，独立挂账。
- `clip-border-area-border-shape-overflow`（4.42%）：background-clip:border-area +
  形状超出 border-box（108%/-8%）复合域，独立挂账。

## 门禁

make test **19,370P/0F**（+1 单测）；fmt 干净；clippy 双 feature 组 `-D warnings` 零
输出；make reftest（本地 687）0 failed；product-smoke welcome **15.40% 同值** struct
PASS；bench-gate 定向 zero-engine **GATE PASS**（26 指标）。

## 下轮方向

1. corner-shape 余 6 fail 深域勘察（backdrop-filter ×2/iframe/video/inset-shadow/
   render-corner-shape 多 variant）；
2. replaced 元素 border-shape overflow 裁剪（img/iframe/self ~3 案，ImagePrimitive
   路径内形状裁剪）或 flex item 子元素不绘制 bug 勘察（跨域：layout-engine）；
3. 或守成轮。
