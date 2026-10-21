# M2 S4 几何面（SVGGeometryElement + 文本几何）——全 corpus 需求盘点（2026-10-21）

看护轮产出（S1 持有轮 `make test` 仍在跑——07:05/5400s，page_scripts 测试段
推进中；源码面照旧退避）。数据源 v2 基线 JSON（同前看护轮约定）。S 系列 Prep
至此齐备（S2/S3 见同日两份盘点）。

## 全 corpus 几何/文本几何成员账

| 成员 | F | 案 | 分布 | 计算性 |
|---|---|---|---|---|
| getBBox | 183 | 14 | types 10 / geometry 2 / coordinate-systems 1 / text 1 | **形状 bbox 属性级可算**（rect/circle/ellipse/line/polygon/polyline/path 直接由几何属性推导，含 transform 合成）；text/use/foreignObject 的 bbox 需布局——分面处理 |
| getTotalLength | 92 | 14 | path 12 / types 2 | **纯路径数学**（d 解析 + 弧长参数化；SVGGeometryElement 六形状 mixin） |
| setPathData/getPathData | 54 | 13 | path 13 | SVG 1.1 pathData API（SVG2 已移除、Chromium 仍实现）——纯 path data 解析/序列化，S4 评估项（排 S4c） |
| getPointAtLength | 28 | 7 | path 7 | 纯路径数学（弧长取样） |
| getRotationOfChar | 25 | 1 | text 1 | **字形布局依赖**（glyph 定位/旋转）——文本几何簇 |
| getStartPositionOfChar | 16 | 5 | text 5 | 同上（char 单元几何） |
| getNumberOfChars | 14 | 7 | text 7 | 纯文本内容长度（唯一无布局依赖的文本几何成员） |
| getCTM | 1 | 1 | types 1 | 同 S3 CTM |

## S4 排片建议

1. **S4a SVGGeometryElement 核心**：path/rect/circle/ellipse/line/polygon/
   polyline 的 getTotalLength/getPointAtLength/pathLength（120F 靶面）——
   d 属性解析 + 贝塞尔/圆弧弧长数学，零布局依赖；
2. **S4b getBBox 属性级近似**：形状 bbox（183F 靶面，text/use/foreignObject
   子面先记账）——与渲染边界核对后落 engine 侧（spec SVG2 getBBox 定义为
   「几何属性推导」，Chromium 对无渲染文档同样可返回解析 bbox）；
3. **S4c pathData API 评估**：54F；SVG1.1 面但 Chromium 兼容保有——建议实现
   （path data 解析器与 S4a 共享）；
4. **文本几何（getRotationOfChar/getStartPositionOfChar 等约 55F）挂账候选**：
   字形定位依赖布局/字体度量——按 goal 划界与 rendering-compat 协同，M4
   定稿归口；getNumberOfChars 纯面可随 S4a 顺带。

## 操作注记

S1 持有轮 `make test` 进行中（/tmp/make-test-m3.log）——其完成结果与本方
无关的 DC-4 门禁复核；S1 落地判定仍按 master.md 恢复约定（git log + 工作区）。
