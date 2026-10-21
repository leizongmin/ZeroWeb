# SVG 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../svg-compat.md](../svg-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-21（M3 S1 命中测试落地）

## 当前状态

**M3 S1 完成——SVG 形状命中测试（2026-10-21）**：runner headless 无宿主
HitTestCache（elementFromPoint host-only 恒 null、形状 gBCR 全零、computed
fill 空串）是 hit-test 族全灭根因。part06.js 新增 `_zwSvgHitSel` 纯属性几何
自建命中：fill 区域（rect 精确实心 / circle·ellipse 归一化半径² / path·
polygon·polyline 逐子路径 nonzero 缠绕 + 未闭合隐式闭合 + 闭域边界点命中）+
stroke 区域（rect 精确环带含 90° miter 角 / 轮廓采样距离 + 顶点 join 圆盘
round 近似 / line 仅 stroke）+ pointer-events 关键字表（缺省
visiblePainted，属性面继承回溯）+ visibility/display 门 + transform 链自根
向叶左乘逆映射（最外层 svg 局部 = gBCR 平移——runner body 8px margin 实测
取证；嵌套 svg = x/y 属性平移——shim gBCR 不含嵌套偏移实测不可用）+ 文档序
逆序首中。集成两点：elementFromPoint 无宿主回落 + `_zwHitTestSel` SVG
pass（host 路径后、gBCR 近似前）。修出三轮：root 判定方向 + 复合顺序、
探针 client 坐标基准（8px margin）、fill 闭域边界点（winding 计 0）。
靶面四案 **86/86 全绿**（rect-001 4 + rect-002 53 + ellipse 24 +
small-big-path 5，基线全灭）。全 corpus 对账（S4c 基线 1104P/590F/41T =
63.6%）：原始 **1201P/504F/41T/1746 = 68.8%**，转移矩阵 86 Fail→Pass
**零 Pass 回归零 DOWN、Timeout 集合恒等**；探针剔除同分母 62.9%→**68.0%**；
+86P 聚类全部落靶面无归因悬案。DC-4：fmt EXIT=0 + 定向 zero-engine
（见 evidence）。挂账：pointer-events-bbox 19 子（CSS cascade +
bounding-box + text/image）、text-hittest 3 子（Ahem 字形格）、
border-radius 1 子（style-system 协同）、祖先 paint 属性继承、miter join
尖、viewBox/百分比/clip。下一步：M3 CSV/收尾 → M3 后续面（linking/scripted
href 族、load-error events）→ M4。

（M2 收口记录见 evidence/2026-10-21-m2-s4c-pathdata-landing.md 与下方
S4c 切片条目：corpus 63.6%，path/interfaces 54P/2F，syncbase F(2)→T
分类移动机制已溯，DC-4 定向 3372P/0F。）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | svg/ corpus 导入 + 基线 + 渲染断言面占比盘点 | ✅ M1（2026-10-21，8.8% 基线） |
| P2 | SVG DOM 类层级 + SVGSVGElement viewBox/CTM 最小面 | ✅ M2（2026-10-21，S1–S4c 全切片 ✅，corpus 63.6%） |
| P3 | SVGGeometryElement 几何接口 + 事件目标化 | ⏳ M3（S1 命中测试 ✅ 68.8%，几何接口已随 M2 S4a 落地） |
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
  `_zwSvgBboxIntersect` 严格相交，两轮修出后全量对账）。corpus 两段测量：
  探针剔除同分母 1012P/650F/40T/0NR = 59.5%（恰 +15P/-15F，全绿案
  133→135，零 DOWN 零子测试回归）；原始全量 1046P/650F/40T/0NR = 60.3%
  （+34P 为 5 个本地探针案断言 6+7+5+6+10，分母 1702→1736——初记
  「属 navigation/对方新增」系误归因，逐文件核验后更正）；-05 4/15→15/15、
  -09 20/24→24/24。第七次会话重叠：并行看护 session 跑全量 corpus 对账
  并外部精修 evidence，本 session 互让等待后核验 runner 二进制含本切片
  改动采其收尾数字，随后清杀孤儿 runner 以唯一日志名复跑取原始全量口径；
  源码面/master.md 归本 session。fmt 门禁绿（fa6ffacc9 修复），定向
  zero-engine（test-guard）3358P/0F；DC-4 全 workspace `make test`
  复跑 EXIT=0（树基 c674eccb3；首跑唯一红 = cdp_focus_governance 6 子，
  TV-5 陈旧 bin 守卫由本切片 shim 改动触发，重编 zero-renderer 后 6/6
  过）。证据：evidence/2026-10-21-m2-s4b2-paint-geometry-landing.md。
- **M2 S4c（2026-10-21）**：pathData API + SVGAnimationElement 控制面
  stub + `_zwSvgPathParse` Z 尾数死循环根修（编码与定向验证在前一审
  session 完成、5 小时限额中断，corpus 复跑/DC-4/对账由接续 session
  补完收口）。path/interfaces 0P/56F→**54P/2F**；全 corpus 原始
  1104P/590F/41T = 63.6%、探针剔除同分母 62.9%（对 S4b-2 基线恰
  +58P/-60F 零 Pass 回归零 DOWN，54 属靶面、4 属时序案抖动不记归因）；
  syncbase F(2)→T 分类移动机制已溯（SMIL 挂账面预期移动）。
  **M2 至此全部切片完成**。证据：
  evidence/2026-10-21-m2-s4c-pathdata-landing.md。
- **M3 S1（2026-10-21）**：SVG 形状命中测试（本 session 端到端持有——
  根因定位、`_zwSvgHitSel` 几何自建 + 两集成点、诊断页取证三轮修出、
  全量对账）。靶面四案 86/86 全绿（基线全灭）；全 corpus 原始
  1201P/504F/41T = 68.8%（86 Fail→Pass 零 Pass 回归零 DOWN，+86P 全落
  靶面）；探针剔除同分母 68.0%。DC-4：fmt EXIT=0 + 定向 zero-engine
  3373P/0F。挂账：CSS cascade 命中面（pe-bbox 19 子 + border-radius）/
  文本字形命中（text-hittest 3 子）/祖先 paint 属性继承/miter 尖/
  viewBox/百分比/clip。证据：evidence/2026-10-21-m3-s1-hittest-landing.md。

## 下一步计划

1. **M3 S1 CSV 行**：docs/compat/trends/wpt-suites.csv 追加 M3 S1 数据行
   （原始 68.8% / 探针剔除 68.0%，S1 证据列为来源）。
2. **M3 后续面评估**：linking/scripted href 族（22F+3T）、script/image
   load-error events（2T）——切片边界划定后推进；祖先 paint 属性继承
   （fill/stroke/stroke-width）随命中面后续切片升级。
3. **M4 SMIL 分类账重跑**：S3 脚手架已释放时序面（seeking-events 族 F→T、
   invalid-value 族平凡全绿），M4 SMIL 评估须以 S3 后口径重跑分类（见
   evidence/2026-10-21-m4-smil-assessment-prep.md + S3 landing 排片对位）；
   S4c 后新增 syncbase-escaped-dots F(2)→T 移动一并归口。
4. **上下文换算补面（S2a FIXME）**：SVGLength 关联元素的 em/ex/percentage
   上下文（SVGLength-viewport 8F / percentage-directionality 6F）——与
   getBBox 几何上下文同批评估。
5. **rendering-compat 协同项**：getScreenCTM/getCTM 族（S3 划界，需 CSS
   transform 感知布局合成）——与渲染域会话对齐后排期。
6. **挂账面（M4 定稿归口）**：文本几何（-02 tspan + -05 text 三子 +
   getComputedTextLength 族 + text-hittest 3 子，字形度量依赖）；image
   内在尺寸（-03 image1/2 + svg-image-intrinsic-size 族，图片加载态依赖）；
   CSS cascade 命中面（pe-bbox 19 子 + border-radius 1 子，style-system
   协同）。

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
