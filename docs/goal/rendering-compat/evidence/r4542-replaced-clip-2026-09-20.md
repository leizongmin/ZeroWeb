# R4542 证据：replaced 元素 border-shape 内形状裁剪 + 缺失图片资产补齐（replaced-img/self 双翻绿）

- 日期：2026-09-20（R4542 C 轮——R4541 下轮方向②，helpers 1 新 fn + 1 单测 + 资产 1 枚）
- 谱系：R4541 slice 2 overflow 内形状裁剪 → 本轮把 replaced 内容纳入同一机制。

## 实现

1. **`clip_images_to_polygon_inplace`**（helpers.rs）：images 多边形条带裁剪，与 fills v2
   同一不变式——以 image rect 为几何扫描线条带化；0 条带 → rect 清零；1 条带 = 原 rect
   → 保留；部分相交 → 原 image 清零 + 逐条带追加新 ImagePrimitive（`rect` = 原始 rect
   （**source 映射保持不重采样**，render_image 的 crop 语义 R294）+ `clip = Some(条带
   rect)` 窗口），原 `Image(i)` op 槽位替换/插入条带 op（z 序保序、零索引位移）。
   `source: Some`（border-image 9-slice）图元跳过（条带破坏 slice 映射）。
2. `clip_all_primitives_to_polygon` v2 分支接线（`from.images` 起段）。
3. 单测 `test_clip_images_inplace_registers_strip_ops_without_index_shift`。

## 资产补齐（关键归因）

`/images/green-256x256.png` 在 wpt-data **从未存在**（reftest 内 404 → test/ref 两侧同
空 → 假通过）。本轮生成 8-bit RGB 256×256 纯 lime（0,255,0，与既有 green-100x50 同色）
入 `wpt-data/images/` 并登记 `imported-resources.txt`。

**资产激活暴露 2 个新红（归因在案，非本轮代码回归）**：`display-none-inline-img`
（15.62%）与 `dynamic-filter-changes-001`（13.65%）均引用该图且走 reftest-wait.js 动态
路径——资产激活前两侧同空假通过，激活后暴露真实渲染差。stash 验证：两案在 R4541 基线
代码 + 资产在位时**同值复现**（与 R4542 代码无关）。按归因纪律记档为「新激活覆盖的真
实渲染 gap」，域 = 动态 reftest-wait + filter/JS img 切换，独立挂账。

## A/B（reftest-upstream 全语料 vs R4541 基线 1459）

- 翻绿：`border-shape-overflow-replaced-img`（0.05%）、`border-shape-overflow-replaced-self`
  （0.04%）——图片条带机制 + 资产补齐合力；
- 新红（资产激活归因如上）：display-none-inline-img、dynamic-filter-changes-001；
- box-shadow-003 flake 绿相位（−1）。
- 净计 **1458 fail**（15136/16594 = **91.20%**；其中 2 案为 newly-activated 真覆盖）。

## 探针

data-URI 绿图 + border-shape circle(50%) + 10px 边框 + overflow:hidden：img 内容精确裁剪
到内缘圆（r=40），黑环 + 内缘绿圆与 ref 同形。

## 门禁

make test **19,371P/0F**（+1）；fmt 干净；clippy 双 feature 组 `-D warnings` 零输出；
product-smoke welcome **15.40% 同值** struct PASS；bench-gate 定向 zero-engine **GATE
PASS**（26 指标）。

## 下轮方向

1. corner-shape 余 6 fail 深域勘察（backdrop-filter ×2/iframe/video/inset-shadow/
   render-corner-shape 多 variant）；
2. 资产激活新红二案勘察（dynamic reftest-wait + filter/JS img 域）；
3. flex item 子元素不绘制 bug 勘察（跨域 layout-engine）；
4. 或守成轮。
