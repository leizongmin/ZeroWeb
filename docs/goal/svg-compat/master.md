# SVG 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../svg-compat.md](../svg-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-09-12（立项）

## 当前状态

SVG 现状 = usvg 静态栅格化（rendering-compat 域）；SVG 文档模型几乎全零
（SVGSVGElement 2 处 / createSVGPoint 0 / getScreenCTM 0）。本 goal 只做
DOM/几何 JS 面；渲染正确性与 SMIL 归 rendering-compat / 挂账。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | svg/ corpus 导入 + 基线 + 渲染断言面占比盘点 | ⏳ M1 纯资产 |
| P2 | SVG DOM 类层级 + SVGSVGElement viewBox/CTM 最小面 | ⏳ M2 |
| P3 | SVGGeometryElement 几何接口 + 事件目标化 | ⏳ M3 |
| P4 | SMIL 评估 + 渲染断言面挂账定稿 | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：corpus fetch（DOM 面清单优先）+ 导入 + 基线（goals/80 编号脚本可跑
   fetch 步）+ 渲染面占比盘点 + suites CSV 回填

**待用户决策清单**：（空——启动顺序由用户点名）
