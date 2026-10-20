# M2 S3 SVGSVGElement 面——全 corpus 需求盘点（2026-10-21）

看护轮产出（S1 持有轮的 `make test` DC-4 门禁仍在跑——03:51 起
`timeout 5400 make test`，日志 /tmp/make-test-m3.log；源码面照旧退避）。
数据源 v2 基线 JSON（同前看护轮约定）。

## 全 corpus 靶面成员账（274F）

| 成员 | F | 案 | 说明 |
|---|---|---|---|
| pauseAnimations（+unpauseAnimations 同族） | 211 | 208 | **SVGAnimationTestCase helper setup 脚手架**——S3 stub 落地即释放动画面 275 案的真实断言（M4 分类账 m4-smil-assessment-prep.md 的前提） |
| setCurrentTime | 29 | 29 | 同 helper 脚手架 |
| createSVGLength/Matrix/Rect/Point | 25 | 7 | SVGSVGElement 工厂族（types/ + struct/scripted） |
| getScreenCTM | 9 | 2 | viewport 变换矩阵（types/getScreenCTM + geometry） |

viewBox/currentTranslate 未以成员名现身——走 `.baseVal` 读取路径（已计 S2
盘点），S3 实现时与 SVGAnimatedRect 反射合并落地。

## S3 实现面清单（按 SVG2 §Interface SVGSVGElement）

1. **动画控制**：pauseAnimations/unpauseAnimations（pause 语义布尔 +
   getCurrentTime 冻结）/getCurrentTime/setCurrentTime（秒单位时钟）——
   最小面可为纯状态机（不驱动 SMIL），脚手架断言即可转绿；SMIL 真时钟归 M4。
2. **工厂族**：createSVGPoint/Matrix/Rect/Number/Length/Transform——返回
   S2 值对象（依赖 S2a/S2b 先行）。
3. **反射属性**：viewBox（SVGAnimatedRect，invalid 属性 → baseVal 零矩形，
   viewBox-invalid-attribute-baseval 靶面）/width/height/x/y（SVGAnimatedLength）
   /currentTranslate（SVGPoint）。
4. **CTM**：getScreenCTM/getCTM（viewport 矩阵合成；嵌套 svg 场景
   coordinate-systems 面）——与 getBBox 同属几何边界，矩阵计算纯解析可做。

## 边界记账（coordinate-systems 实测）

- **outer-svg-intrinsic-size 9F+1T**：`<svg>` 外层盒固有尺寸（width/height/
  viewBox 推导）——**layout 域**（svg 元素盒尺寸计算），非 JS 接口面；
  按 goal 划界记 rendering-compat 协同项，M4 挂账定稿时归口。
- SVGTransformList baseVal/animVal 18F：SVGGraphicsElement.transform 反射
  ——S2b 值类型家族（transform 属性挂在 graphics 元素，非 SVGSVGElement）。

## 排片结论

S3 依赖 S2 值对象（工厂返回值），实现序维持 S1 → S2 → S3；S3 最小面
（动画控制 + 工厂 + viewBox/currentTranslate 反射）预期释放动画面脚手架
240F + types/ 工厂 25F + CTM 10F ≈ **275F**，并解锁 M4 定稿所需的
animations 真实断言重跑。
