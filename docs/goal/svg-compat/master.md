# SVG 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../svg-compat.md](../svg-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-21（M2 S2b 落地）

## 当前状态

**M2 S2b 完成（2026-10-21）**：值对象批落地——SVGMatrix/Rect/Point/Angle/
Number 值对象 + SVGAnimated{Integer,Enumeration,Boolean,Angle,Rect,
PreserveAspectRatio} 反射 + 17 接口枚举常量与关键词表 + SVGSVGElement
createSVG{Matrix,Rect,Point,Number,Angle} 工厂族 + ownerSVGElement
（foreignObject 边界清零）+ marker setOrientToAngle/Auto。part04 R3037
type/method 对 SVG ns 放行（style/script tag 门控保留 reflected string）。
types 子面 **399P/208F/0T**（S2a 279P/281F，+120P/-73F）；全 corpus 486 案
**554P/1006F/32T = 34.8%**（+124P/-77F，Timeout 32 持平，全绿案 33→63，
逐案零真回归）；S2b 靶面案全绿（Enumeration initial-values 48/48、
MarkerElement 5/5、SVGMatrix 9/9、ownerSVGElement 16/16 等 35 案）。机制探针
8/8。卡点：无。下一步：M2 S2c——SVGNumberList/LengthList（numberOfItems/
parse-invalid-clears 语义）+ SVGPointList/SVGStringList 存在性，解锁
SameObject-identity 41 断言（transform.baseVal.getItem 死点）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | svg/ corpus 导入 + 基线 + 渲染断言面占比盘点 | ✅ M1（2026-10-21，8.8% 基线） |
| P2 | SVG DOM 类层级 + SVGSVGElement viewBox/CTM 最小面 | ⏳ M2（S1 ✅ S2a ✅ S2b ✅ 2026-10-21，S2c→S4 进行中） |
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
- **M2 S2a（2026-10-21）**：SVGLength 单位模型 + SVGAnimatedLength/Number
  反射（本 session 持有架构 + 并行 session 就地精修，合并态统一落地）。
  types 279P/270F/0T（Timeout 4→0），initial-values 两大案全绿；全 corpus
  430P/1083F/32T = 27.5%（+296P，全绿案 20→33，零真回归）；残余全对位既有
  挂账（S2b/S2c/S3/S4）。证据：evidence/2026-10-21-m2-s2a-value-types-landing.md。
- **M2 S2b（2026-10-21）**：值对象批（本 session 端到端持有——语义源精读、
  编码、三轮探针迭代、四轮 types 对账、两轮全通道对账含 styling 回归修复）。
  types 399P/208F/0T（+120P）；全 corpus 554P/1006F/32T = 34.8%（+124P，
  全绿案 33→63，零真回归）；S2b 靶面 35 案全绿。证据：
  evidence/2026-10-21-m2-s2b-value-objects-landing.md。

## 下一步计划

1. **M2 S2c List 类型**：SVGNumberList/LengthList（numberOfItems/
   parse-invalid-clears-items 语义）+ SVGPointList/SVGStringList 存在性；
   SVGTransform 面评估（transform 反射 + SameObject-identity 41 断言解锁）。
2. **M2 S3 SVGSVGElement 面**：动画控制 stub（pauseAnimations 族，释放
   动画面 275 案脚手架）+ 工厂族剩余 + currentTranslate 反射 + CTM
   （盘点见 evidence/2026-10-21-m2-s3-svgsvgelement-prep.md）；S3 落地后
   M4 SMIL 分类账须重跑（当前时序面被脚手架遮蔽，见
   evidence/2026-10-21-m4-smil-assessment-prep.md）。
3. **M2 S4 几何面**：S4a getTotalLength/getPointAtLength 纯路径数学 →
   S4b getBBox 形状属性级 bbox → S4c pathData API 评估；文本几何 ~55F
   挂账候选（盘点见 evidence/2026-10-21-m2-s4-geometry-prep.md）。
4. **上下文换算补面（S2a FIXME）**：SVGLength 关联元素的 em/ex/percentage
   上下文（SVGLength-viewport 8F / percentage-directionality 6F）——与 S4b
   getBBox 的几何上下文同批评估。

**待用户决策清单**：（空）

**会话并行注记（2026-10-21）**：本 clone 今日出现四次会话重叠（前三次：
延续轮看护 session 代做 M1 收口 c10396886 + CSV 合规修正 b5382745a，均经
独立验证后确认；第四次：S2a 同一未提交实现面真竞争，并行 session 就地
精修后以 identical-content 提交 f96e87aee 收敛——均见
evidence/2026-10-21-session-collision-watchdog.md）。约定：同 clone 同
goal 的后到会话先读本文件与 git log，活跃实现面被持有时退避为看护/侦察，落
 evidence 不抢源码面。
