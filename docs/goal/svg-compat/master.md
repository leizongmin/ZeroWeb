# SVG 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../svg-compat.md](../svg-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-21（M2 S4b-2 落地）

## 当前状态

**M2 S4b-2 完成（2026-10-21）**：SVGBoundingBoxOptions paint 几何盒——
stroke 面（`_zwSvgStrokeBbox` 公式盒 ±sw/2；line/poly/polyline/path 走
`_zwSvgGeomLens`+`_zwSvgStrokeWalk`：逐样本法向 offset + run 内部样本
sw/2 外逼近盘（曲线对称极值 chord 零欠冲根修）+ 数据顶点 join（round
圆盘 / miter 尖含 1/sinHalf ≤ miterlimit 门槛）+ run 端 cap（butt 无延伸
/ round 圆盘 / square）+ 命令性零长段 zeros（round/square 产点、butt 不
产、moveto-only run 不产））+ markers 面（start/mid/end 顶点放置：
markerUnits 缺省 strokeWidth、viewBox→markerWidth/Height、refX/refY 对
位、orient 数值（auto FIXME→0）、任意 marker 盒并入 fill 盒）+ clipped
面（clipPath 子级并集：circle∩circle 精确 lens 盒 + use 子解引用 + 子自
身 clip 递归；objectBoundingBox 以 fill 盒为参照；SVG1.1 clip 属性
rect(t,r,b,l) 相对元素原点；引用缺失/非 clipPath/空 clipPath 全裁零盒；
严格相交——边点接触→EMPTY；查询元素为 use 时区域随 (x,y) 平移）。
clip/marker 子树渲染判定走 skipTagGuard（defs 归属豁免）。text 定位点集
重写（tspan 子定位点并集 + x/y 缺省 0）。全 corpus 486 案
**1012P/650F/40T/0NR = 59.5%**（S4b-1 997P/665F 同分母 +15P/-15F，Timeout
持平，全绿案 133→135，逐案零 DOWN）。-05 4/15→**15/15**、-09 20/24→
**24/24**、getBBox 全量 146P/2F（唯二 F 既有挂账）；path/parsing 116P/0F
零回归。卡点①：S4c pathData API（path/interfaces 0P/56F 未实现面）。
卡点②：fo5 零盒机制溯源未闭环（行为两侧对齐 Chrome，机制未溯，探针待
跑，不阻塞）。fmt 门禁已绿（fa6ffacc9 修复 comp-anim 流带入红，
`cargo fmt --all --check` EXIT=0），跨流碰头解除；定向
`cargo test -p zero-engine`（test-guard）3358P/0F。下一步：S4c pathData
API 评估 → S4 完成后 M2 CSV 行回填。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | svg/ corpus 导入 + 基线 + 渲染断言面占比盘点 | ✅ M1（2026-10-21，8.8% 基线） |
| P2 | SVG DOM 类层级 + SVGSVGElement viewBox/CTM 最小面 | ⏳ M2（S1 ✅ S2a ✅ S2b ✅ S2c ✅ S3 ✅ S4a ✅ S4b-1 ✅ S4b-2 ✅ 2026-10-21，S4c 进行中） |
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
- **M2 S4a（2026-10-21）**：SVGGeometryElement 几何面（本 session 端到端
  持有——路径数字文法 + 解析器 + 弧长取样 + 六形状 mixin + getNumberOfChars，
  探针三轮修正，全量对账）。corpus 858P/804F/40T/0NR = 50.4%（+124P/-124F，
  全绿案 110→125，零 DOWN，Timeout 持平）——**通过率跨过半数线**；
  path/parsing 11 案 126 子全绿。全 workspace `make test` DC-4 门禁
  EXIT=0（TIME_LIMIT=3000 守卫跑）。CSS d property 2+5 子挂账 CSS 域协同。
  证据：evidence/2026-10-21-m2-s4a-geometry-landing.md。
- **M2 S4b-1（2026-10-21）**：SVGGraphicsElement.getBBox 属性级面（本
  session 端到端持有——断言面精读、递归收集 + 公式盒 + 容器并集 + use
  影子树 + #1018 非渲染判定，探针两轮修出 transform 列表右先序，全量对
  账）。corpus 997P/665F/40T/0NR = 58.6%（+139P/-139F，全绿案 125→133，
  零 DOWN 零子测试回归）；getBBox 八案全绿；S2c 残余 replaceitem 卡点
  解锁翻绿。第六次会话重叠：并行看护 session 侦察在途退避（零改对方面，
  见 watchdog 第六次记录）。workspace make test fmt 门禁红于并行 comp-anim
  流（9799e6435 非本流文件），本切片经 test-guard 定向 zero-engine
  3355P/0F 验证。证据：evidence/2026-10-21-m2-s4b1-getbbox-landing.md。
- **M2 S4b-2（2026-10-21）**：SVGBoundingBoxOptions paint 几何盒（本
  session 端到端持有——stroke walk + markers 顶点放置 + clip 交集装配，
  三处根因修出：clip 子树 defs 豁免 skipTagGuard / run 内部样本外逼近盘 /
  `_zwSvgBboxIntersect` 严格相交，两轮修出后全量对账）。corpus
  1012P/650F/40T/0NR = 59.5%（+15P/-15F，全绿案 133→135，零 DOWN 零子
  测试回归）；-05 4/15→15/15、-09 20/24→24/24。第七次会话重叠：并行
  看护 session 跑全量 corpus 对账并外部精修 evidence，本 session 互让等待
  后核验 runner 二进制含本切片改动采其收尾数字；源码面/master.md 归本
  session。fmt 门禁绿（fa6ffacc9 修复），定向 zero-engine（test-guard）
  3358P/0F。证据：evidence/2026-10-21-m2-s4b2-paint-geometry-landing.md。

## 下一步计划

1. **M2 S4c pathData API**（path/interfaces 0P/56F 未实现面）：getPathData/
   setPathData（normalize 选项、malformed d 截断、detached 元素、SMIL/Web
   Animations base value 域划界）评估切片边界 → S4 完成后 M2 CSV 行回填。
2. **M4 SMIL 分类账重跑**：S3 脚手架已释放时序面（seeking-events 族 F→T、
   invalid-value 族平凡全绿），M4 SMIL 评估须以 S3 后口径重跑分类（见
   evidence/2026-10-21-m4-smil-assessment-prep.md + S3 landing 排片对位）。
3. **上下文换算补面（S2a FIXME）**：SVGLength 关联元素的 em/ex/percentage
   上下文（SVGLength-viewport 8F / percentage-directionality 6F）——与
   getBBox 几何上下文同批评估。
4. **rendering-compat 协同项**：getScreenCTM/getCTM 族（S3 划界，需 CSS
   transform 感知布局合成）——与渲染域会话对齐后排期。
5. **挂账面（M4 定稿归口）**：文本几何（-02 tspan + -05 text 三子 +
   getComputedTextLength 族，字形度量依赖）；image 内在尺寸（-03 image1/2 +
   svg-image-intrinsic-size 族，图片加载态依赖）。

**跨流碰头信号（2026-10-21，run-rules §9/§10）——已解除**：workspace fmt
红于 comp-anim 流 `apps/compositor/src/frame_dump.rs`（9799e6435 引入，
origin/main 同红），已由 fa6ffacc9 机械重排修复；S4b-2 轮
`cargo fmt --all --check` EXIT=0 复核绿，全 workspace 门禁恢复可用。

**待用户决策清单**：（空）

**会话并行注记（2026-10-21）**：本 clone 今日出现四次会话重叠（前三次：
延续轮看护 session 代做 M1 收口 c10396886 + CSV 合规修正 b5382745a，均经
独立验证后确认；第四次：S2a 同一未提交实现面真竞争，并行 session 就地
精修后以 identical-content 提交 f96e87aee 收敛——均见
evidence/2026-10-21-session-collision-watchdog.md）。约定：同 clone 同
goal 的后到会话先读本文件与 git log，活跃实现面被持有时退避为看护/侦察，落
 evidence 不抢源码面。第七次（S4b-2 轮）：看护 session 跑全量 corpus 对账 +
 外部精修 evidence 终版，持有 session 互让等待（不并行双 runner，前轮两次
 corpus 被 SIGTERM 即资源互杀实证），核验二进制含本方改动后采其收尾数字。
