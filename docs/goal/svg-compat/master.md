# SVG 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../svg-compat.md](../svg-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-21（M2 S3 落地）

## 当前状态

**M2 S3 完成（2026-10-21）**：SVGSVGElement 面落地——动画控制状态机 stub
（pauseAnimations/unpauseAnimations/getCurrentTime/setCurrentTime，per-element
状态，M4 SMIL 时钟接入后合流共享 timeline）+ currentTranslate（[SameObject]
只读 SVGPoint，SameObject-identity 的 DOMPointReadOnly 断言假绿转真绿）+
currentScale（outermost 判定：直接父不在 SVG ns；嵌套 getter 恒 1/setter
no-op）+ deselectAll（清文档 selection）+ viewBox 负维度→零矩形（WebKit
tentative）。实现期发现并修复引擎缺口：元素 proxy set trap 无原型 setter 派发
（R98 仅 CE 门控）——S3 名称门控分支（setter 调用/getter-only 静默拒绝不落
expando）。全 corpus 486 案 **734P/928F/40T/0NR = 43.1%**（含 NR 口径，S2c
681P/965F/32T/18NR = 40.2% 以来 **+53P/-37F**，全绿案 76→110，逐案零 DOWN）；
S3 靶面 4 案全绿 + 动画面脚手架释放 30 案翻绿（smil-clock 18NR→11P/7F +
invalid-value 族 29 案平凡满足全绿，M4 复核面）；**Timeout 32→40**：seeking-
events 族 9 案 F→T（SMIL 时钟缺，挂账面分类移动非 Pass 损失，M4 输入）。
机制探针 5/5。全 workspace `make test` EXIT=0（2026-10-21 S3 轮）。
卡点：无。下一步：M2 S4 几何面（S4a getTotalLength/
getPointAtLength → S4b getBBox → S4c pathData 评估；盘点见
evidence/2026-10-21-m2-s4-geometry-prep.md）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | svg/ corpus 导入 + 基线 + 渲染断言面占比盘点 | ✅ M1（2026-10-21，8.8% 基线） |
| P2 | SVG DOM 类层级 + SVGSVGElement viewBox/CTM 最小面 | ⏳ M2（S1 ✅ S2a ✅ S2b ✅ S2c ✅ S3 ✅ 2026-10-21，S4 进行中） |
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
- **M2 S2c（2026-10-21）**：List/Transform 批（本 session 端到端持有——
  List 案语义精读、值列表引擎 + Transform live 对象存储、探针一轮修正、
  两轮全量对账）。types 491P/191F/0T（+92P）；全 corpus
  681P/965F/32T/18NR = 40.2%（+127P，全绿案 63→76，零真回归）；S2c 靶面
  13 案全绿（SameObject-identity 81/81 解锁）。证据：
  evidence/2026-10-21-m2-s2c-list-transform-landing.md。
- **M2 S3（2026-10-21）**：SVGSVGElement 面（本 session 端到端持有——
  WPT 靶案断言精读、S3 块 + set trap 派发缺口修复、探针一轮修正、全量
  对账）。corpus 734P/928F/40T/0NR = 43.1%（+53P，全绿案 76→110，零
  DOWN；Timeout 32→40 为 F→T 挂账面分类移动）；S3 靶面 4 案全绿 +
  动画面脚手架释放 30 案翻绿；SameObject-identity currentTranslate 假绿
  转真绿。getScreenCTM 划界 rendering-compat 协同。证据：
  evidence/2026-10-21-m2-s3-svgsvgelement-landing.md。

## 下一步计划

1. **M2 S4 几何面**：S4a getTotalLength/getPointAtLength 纯路径数学 →
   S4b getBBox 形状属性级 bbox（S2c 残余 svgtransformlist-replaceitem
   卡点即 getBBox）→ S4c pathData API 评估；文本几何 ~55F 挂账候选
   （盘点见 evidence/2026-10-21-m2-s4-geometry-prep.md）。S4 完成后
   M2 CSV 行回填。
2. **M4 SMIL 分类账重跑**：S3 脚手架已释放时序面（seeking-events 族 F→T、
   invalid-value 族平凡全绿），M4 SMIL 评估须以 S3 后口径重跑分类（见
   evidence/2026-10-21-m4-smil-assessment-prep.md + S3 landing 排片对位）。
3. **上下文换算补面（S2a FIXME）**：SVGLength 关联元素的 em/ex/percentage
   上下文（SVGLength-viewport 8F / percentage-directionality 6F）——与 S4b
   getBBox 的几何上下文同批评估。
4. **rendering-compat 协同项**：getScreenCTM/getCTM 族（S3 划界，需 CSS
   transform 感知布局合成）——与渲染域会话对齐后排期。

**待用户决策清单**：（空）

**会话并行注记（2026-10-21）**：本 clone 今日出现四次会话重叠（前三次：
延续轮看护 session 代做 M1 收口 c10396886 + CSV 合规修正 b5382745a，均经
独立验证后确认；第四次：S2a 同一未提交实现面真竞争，并行 session 就地
精修后以 identical-content 提交 f96e87aee 收敛——均见
evidence/2026-10-21-session-collision-watchdog.md）。约定：同 clone 同
goal 的后到会话先读本文件与 git log，活跃实现面被持有时退避为看护/侦察，落
 evidence 不抢源码面。
