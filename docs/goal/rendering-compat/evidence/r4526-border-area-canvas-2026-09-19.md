# R4526 证据：border-area 传播绘制定谳 + 实底色环带——004-rna 翻绿（88.61→0.00）

- 日期：2026-09-19（R4526 C 轮——painter 2 处）
- 用例：`clip-border-area-on-body-not-propagated-to-root.html`（background-clip 族最高单案
  88.61%）+ 姊妹页 `clip-border-area-on-body-propagated-to-root.html`（propagated 谱系守卫）。
- 方法：chromium headless 探针（004 双页真实截图）+ ZW 逐带对照。

## 双页律定谳

| body 形态 | chromium 渲染 |
|---|---|
| clip:border-area + **有 border**（20px transparent） | **不传播**：body 自绘 border 环带（20px 绿环于 body border 位置）+ 环外白 |
| clip:border-area + **无 border** | **传播全画布**（border-area 无 border ≡ 全盒；propagated ref 全绿） |

## ZW 病灶两处 + 修复

1. **paint_background 实底色无环带臂**（R3908 环带仅覆盖图像 tile 路径）→ 实底色
   BorderArea 臂：有 border = 4 条带环带（border-box − padding-box，同 R3908 几何）；
   无 border 回落全盒臂。
2. **画布传播填充无视 clip**：body fallback 全画布涂绿（ZW-TEST 全绿 vs ref 白底绿环）
   → BorderArea 臂：body border 环带 4 矩形（几何取 body 盒已解析 border 宽）；
   无 border 回落全画布。

## A/B

- corpus 1476→**1475**（15119/16594 = 91.10%）；fail-list diff 唯一 004-rna 移除。
- background-root 谱系 017/018 既有 fail 持平（R720 域）✓；border-area 余 11 fail =
  border-shape × clip 交互 / blend-mode / multiple-backgrounds（独立深域）。
