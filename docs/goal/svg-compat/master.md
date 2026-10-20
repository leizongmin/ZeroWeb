# SVG 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../svg-compat.md](../svg-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-21（M1 收口）

## 当前状态

**M1 完成（2026-10-21）**：svg/ corpus 导入 2144 文件（pin 31597693）+ 基线
v2 = 486 案 1527 子测试 **134P/1339F/36T/18NR = 8.8%**（20 案全绿）；渲染断言
面 76.4%（1637/2144）记 rendering-compat 域；suites CSV 已回填。卡点：无。
下一步：M2 S1 类层级路由（`__zwSvgTagIface` + R80 SVG ns 原型路由，实施计划
见 evidence/2026-10-21-m2-implementation-plan.md）。会话碰头与 v1/v2 基线
经过见 evidence/2026-10-21-session-collision-watchdog.md。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | svg/ corpus 导入 + 基线 + 渲染断言面占比盘点 | ✅ M1（2026-10-21，8.8% 基线） |
| P2 | SVG DOM 类层级 + SVGSVGElement viewBox/CTM 最小面 | ⏳ M2 |
| P3 | SVGGeometryElement 几何接口 + 事件目标化 | ⏳ M3 |
| P4 | SMIL 评估 + 渲染断言面挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1（2026-10-21）**：`testharness-svg` runner 通道（7b9499062——深挖 fetch +
  Makefile target + skip 规则两侧同源）+ helper 补拉（SVGAnimationTestCase，
  175 案假失败转真实子测试）+ 基线 8.8% + 渲染面占比 76.4% 入档。证据：
  evidence/2026-10-21-m1-baseline.md（分目录表 + M2 失败簇对位）。

## 下一步计划

1. **M2 S1 类层级路由**：`__zwSvgTagIface`（localName→接口名，SVG2 element
   index 全表）+ part05.js R80 分支 SVG ns 路由（命中表→接口 prototype；
   miss→SVGElement.prototype）+ 缺失接口构造器补齐（desc/metadata/textPath/
   view/geometry/animation 族）；MathML ns 同法。验证：`make testharness-svg
   FILTER=types`（70 案 0P 基线）+ FILTER=struct。
2. **M2 S2-S4**：值类型底座 → SVGSVGElement 面 → SVGGraphicsElement + 几何
   （切片序见 evidence/2026-10-21-m2-implementation-plan.md）。

**待用户决策清单**：（空）
