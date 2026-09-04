---
date: 2026-09-04
modules: layout-engine, engine
---

# collapse 边框中心线迁移：taffy 拉伸伪影 × paint 盒内绘制模型的双耦合面

## 问题描述

`border-collapse: collapse` 的 CSS2 语义是「边框中心线在网格线上」：列宽 = 边框中心
间距，cell 对列/行的尺寸贡献 = border-box − ½(外侧边框)，边框带绘制跨网格线。ZW 的
实现把它拆在三处且各按旧语义：(1) taffy 对 TableCell 不零边框（converter
`is_table_internal` 仅含 row/row-group），cell 以全额边框进布局；(2)
`compute_cell_intrinsic_width` 用 post-taffy 拉伸宽估算内容固有宽（95% 阈启发式）；
(3) paint 层 `border.rs` 把边框画在 cell 边缘**内侧**（内共享边各画半由两邻居合成）。
四轮试修（R4025/R4027/R4028/R4030/R4031）证明任一单点改都会被其余两处反弹。

## 根因

三个语义面互为前提：

1. **尺寸维**：列/行宽贡献含全额边框 → 表尺寸偏大 ½ 边框（bc-006：列 150 应 75）。
2. **固有宽维**：auto 表先被 taffy 拉伸到容器宽，cell 后代同步拉伸——95% 阈启发式
   无法区分「真实宽内容」与「拉伸伪影」（80.8% 宽的空 div 被当真实内容）。
3. **绘制维**：边框画在 cell 内侧，cell 盒含全额边框时恰好视觉正确；一旦尺寸维改
   半宽语义，绘制立即错位（错位量 = 外侧半宽）。

## 教训与模式

- **净 0 / 净负的混合结果也须回退**：R4028 +5/−4、R4030 +1/−3、R4031 +2/−5 全部
  回退；真回归不可与真收益对冲入库。
- **每轮失败的案名清单是下一轮的 gate 设计输入**：R4028 的 css1/c5501 失败暴露
  「显式宽 cell 的 min-content floor 须维持旧启发式」（R4029 的
  `for_explicit_floor`）；cv-095 失败暴露「abspos 子回落 laid-out 宽」（ZW abspos
  以 cell 为包含块）。R4029 净+4 正是这两个 gate 补全的结果。
- **结构隔离证明可达性**：bench-gate 对 block-only（零表）基准 FAIL 时，可直接证明
  表维改动不可达该路径，归因宿主负载。
- **criterion 直测定音**：包裹式 bench-gate 对宿主噪声敏感（同负载下 clean HEAD
  4.25ms vs 改动 3.06ms），直测是唯一可信仲裁。
- **最终形态需 oracle 校准**：collapsing-border-model-007 的 chromium 渲染（蓝带
  220px 全宽）无法从像素反推语义（border-centerline 推导给出不同形状），需要 CDP
  实测 html/td 盒几何。涉及「布局语义 × paint 几何 × 布局器伪影」三耦合面的迁移，
  应先捕获 oracle 数据再动代码——已按 user-decision 门禁挂账。

## 关联

R4025-N/R4026-N/R4027-N/R4028-N/R4029-F/R4030-N/R4031-N（rendering-compat goal 控制面）
