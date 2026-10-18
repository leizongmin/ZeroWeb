date: 2026-10-18
modules: engine

# border-radius+border 边框带按直角条带绘制——弧带缺失被 corpus 自洽掩盖

## 问题

R5179 corner-shape-square 案形残差 168px 全集中在 BR 圆角：test 侧（corner-shape
ring 管线）画出真圆弧，ref 侧（普通 `border-bottom-right-radius: 25px` + `border`）
黑边框带一直延伸到盒角（直角）。

## 根因

CPU 管线里**常规 border-radius 的边框带从未按弧画**：`paint_borders` 的 shaped
ring 分支只对「corner-shape 非 all-round」生效，plain round 落回四边条带（直角）。
背景裁剪是圆的（纯 radius 无 border 的页面对角边界 93 ✓ 几何 92.7），只有边框带
是直角。corpus 是 self-source（test/ref 都由 ZW 渲染）：radius+border 案两侧都直角
→ 自洽 PASS，缺口长期被掩盖；直到 corner-shape 套件混用「shaped ring（test）
vs clip-path/radius 管线（ref）」才暴露。

## 解决

未修（R5180 候选）：`paint_borders` 对 plain round 补「外弧+内弧 ring path_fill」，
可直接复用 shaped ring 的成对多边形代码（CornerShapeKind::Round 臂已有圆弧采样）。
风险面：改动后 self-source corpus 的 radius+border 案两侧同变，理论零翻红，但需
A/B 全量确认。

## 避免

- self-source corpus 的「PASS」只证两页自洽，不证符合规范；新功能（corner-shape）
  与旧功能（border-radius）在 ref 页混搭时，旧功能的既有缺口会以新案形式冒出来。
- 判别方法：渲染单页 PNG 后沿对角线/扫描线找边界，和几何值（圆心+半径）对表。
