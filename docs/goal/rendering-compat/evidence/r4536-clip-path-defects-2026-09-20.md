# R4536 证据：corner-shape 红簇根因链全解码——clip-path 组合裁剪三缺陷定位（P 轮，实验未收敛回退）

- 日期：2026-09-20（R4536 P 轮——R4535 下轮方向①续，0 net code 落地）
- 谱系：corner-shape 凹角簇（~10 案 1.4-1.9%）+ clip-path 嵌套裁剪机械。

## 根因链全解码（product-smoke 任意页渲染探针 + ZW_CLIP_DEBUG/REFTEST_DUMP 取证）

1. **ref 页构型**：notch/scoop 系 ref = `.target`（红 bg + **外层 clip-path 凹多边形**）
   + `::before`（绿 bg + **内层 clip-path**，position:absolute + inset:0）。
2. **探针逐层分离**（/tmp/zwprobe 五连探针，RGB 逐图核对）：
   - ::before + inset:0 + absolute **单 Independently ✓**（inset 简写/伪元素/绝对定位均工作）；
   - ::before 自身 clip-path **单独 ✓**（绿色内十字正确）；
   - 父 clip-path + 纯绿 ::before ✓（父裁剪覆盖子 ✓）；
   - **父 clip-path + 子 clip-path（组合）✗ 绿色整层消失**——真 div 与伪元素同现，
     与伪元素无关。
3. **三缺陷定位**：
   - **缺陷 A（主犯）**：clip-path Polygon 臂把「被多边形完全覆盖的背景」改写为
     `path_fill` **追加到图元列表尾**——path_fills 渲染序高于子树 fills → 重发的父
     背景 z 序高于 ::before/子内容 → **父红背景盖死绿色子层**（ref 全红直接根因）。
   - **缺陷 B**：`clip_fill_to_polygon` 扫描线在**条带顶采样** + 半开边规则
     (y1 < y && y2 >= y)——多边形顶点恰落在采样线上（整数坐标测试页常态）时漏计
     自该顶点起始的边 → 整带丢失（「R4248 整块丢弃」的真机制）。**中点采样修正已
     实现并验证**（条带中点与整数顶点天然错开；单 clip 探针条带精确）。
   - **缺陷 C**：嵌套 clip 页的内层 clip 消费快照范围异常（子 clip 消费 in=18 覆盖
     全页 fills，含父条带；父 clip 消费 in=2 漏自身背景）——counts_before 快照与
     改写/拼接的索引联动在嵌套场景失准，需独立梳理。
4. **本轮strip-splice 实验结果**：中点采样 + 原位条带拼接后，单页正确但组合页出现
   「背景整层白 + 仅剩 dashed 指示线」（缺陷 C 放大）→ 按零新红纪律整体回退，
   树态还原 R4535。

## 续作蓝图（下轮，按依赖序）

1. **Fix B 独立落地**：中点采样一行修正（几何严格更正确）+ clip-path 全语料 A/B
   （单 clip 页条带归属变化，需零新红验证）。
2. **Fix A**：覆盖背景改写从「path_fill 追加尾」改为「原位条带拼接」（保 z 序）；
   依赖 Fix B 的条带精确性。
3. **Fix C**：嵌套 clip 快照范围梳理（子 clip 消费的 counts_before 覆盖面——需完整
   读 paint_node 裁剪块与 defer_abspos 交错）。
4. 三修落地后 corner-shape 凹角簇（gate 撤销 + sweep 修正 + 负指数整档采样，几何
   已验证）+ notch/scoop/bevel 系 ~10 案预期翻绿；border-shape slice 2 overflow
   裁剪（同机械）同步解锁。

## 门禁

零代码落地（实验回退），无门禁义务；树态 = R4535 干净基线。
