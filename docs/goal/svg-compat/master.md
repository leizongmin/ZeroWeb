# SVG 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../svg-compat.md](../svg-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-21（M2 S1 落地）

## 当前状态

**M2 S1 完成（2026-10-21）**：SVG DOM 类层级路由落地——`__zwSvgTagIface`
全表 + `_zwSvgIfaceParents` 继承链 + R80 SVG/MathML ns 原型路由（解析元素
同收，词表 gating 保 HTML 热路径零开销）。验证：全 workspace `make test`
EXIT=0；FILTER=types/struct 零翻转零回归；机制探针 2/2 Pass（四道门见
evidence/2026-10-21-m2-s1-landing.md）。基线 8.8% 不变（instanceof 收益
待 S2 值类型释放）。卡点：无。下一步：M2 S2a——SVGLength 单位模型 +
SVGAnimatedLength/Number 反射（靶面 259F，盘点见
evidence/2026-10-21-m2-s2-value-types-prep.md）。M1 基线与 S1 收尾接管
经过见 evidence/2026-10-21-session-collision-watchdog.md 与
evidence/2026-10-21-m2-s1-landing.md。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | svg/ corpus 导入 + 基线 + 渲染断言面占比盘点 | ✅ M1（2026-10-21，8.8% 基线） |
| P2 | SVG DOM 类层级 + SVGSVGElement viewBox/CTM 最小面 | ⏳ M2（S1 ✅ 2026-10-21，S2-S4 进行中） |
| P3 | SVGGeometryElement 几何接口 + 事件目标化 | ⏳ M3 |
| P4 | SMIL 评估 + 渲染断言面挂账定稿 | ⏳ M4 |

## 已完成切片

- **M1（2026-10-21）**：`testharness-svg` runner 通道（7b9499062——深挖 fetch +
  Makefile target + skip 规则两侧同源）+ helper 补拉（SVGAnimationTestCase，
  175 案假失败转真实子测试）+ 基线 8.8% + 渲染面占比 76.4% 入档。证据：
  evidence/2026-10-21-m1-baseline.md（分目录表 + M2 失败簇对位）。
- **M2 S1（2026-10-21）**：SVG DOM 类层级路由——持有 session 编码 + 本看护
  session 收尾接管（验证四道门 + 提交）。types/struct 案级零翻转零回归 +
  探针 2/2 Pass；S2-S4 切片 Prep 盘点齐备（同日四份 evidence）。

## 下一步计划

1. **M2 S2a 值类型底座**：SVGLength 单位模型（value/valueInSpecifiedUnits/
   valueAsString/unit 换算 + reserialization）+ SVGAnimatedLength/
   SVGAnimatedNumber baseVal/animVal 反射初值语义——直接靶面 259F，单点
   收益最大（聚类与排片见 evidence/2026-10-21-m2-s2-value-types-prep.md）。
   验证：`make testharness-svg FILTER=types` 对 S1 后账本复核翻转。
2. **M2 S2b/S2c**：SVGMatrix/Rect/Point/Angle 值对象 + Animated* 接口对象
   存在性 → List 类型（SVGNumberList/LengthList parse-invalid 语义）。
3. **M2 S3 SVGSVGElement 面**：动画控制 stub（pauseAnimations 族，释放
   动画面 275 案脚手架）+ 工厂族 + viewBox/currentTranslate 反射 + CTM
   （盘点见 evidence/2026-10-21-m2-s3-svgsvgelement-prep.md）；S3 落地后
   M4 SMIL 分类账须重跑（当前时序面被脚手架遮蔽，见
   evidence/2026-10-21-m4-smil-assessment-prep.md）。
4. **M2 S4 几何面**：S4a getTotalLength/getPointAtLength 纯路径数学 →
   S4b getBBox 形状属性级 bbox → S4c pathData API 评估；文本几何 ~55F
   挂账候选（盘点见 evidence/2026-10-21-m2-s4-geometry-prep.md）。

**待用户决策清单**：（空）

**会话并行注记（2026-10-21）**：本 clone 今日出现三次会话重叠（延续轮看护
session 代做 M1 收口 c10396886 + CSV 合规修正 b5382745a，均经独立验证后确认；
记录见 evidence/2026-10-21-session-collision-watchdog.md）。约定：同 clone 同
goal 的后到会话先读本文件与 git log，活跃实现面被持有时退避为看护/侦察，落
 evidence 不抢源码面。
