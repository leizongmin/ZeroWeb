# M3 S1 SVG 形状命中测试——elementFromPoint SVG 面 + 事件目标化 SVG pass（2026-10-21）

S4c（evidence/2026-10-21-m2-s4c-pathdata-landing.md）M2 收口后进入 M3。
本切片为 M3 事件目标化主簇：runner headless 无宿主 HitTestCache，SVG 形状
在渲染路径缺位（`document.elementFromPoint` host-only 恒 null、形状 gBCR
全零、computed fill 空串），hit-test 族案全灭。实现纯属性几何自建命中，
单文件 part06.js（`_zwSvgHitSel` 块 + 两个集成点），轮廓采样复用 part03
`_zwSvgGeomLens`（path 走 `_zwSvgPathParse`+flatten 同源，runStarts 子路径
语义随 S4a/S4b 底座继承）。

## 实现（spec SVG2 interact.html §hit-testing + §PointerEventsProp）

- **fill 区域**：rect 精确实心（含边界）；circle/ellipse 归一化半径²≤1；
  path/polygon/polyline 逐子路径 nonzero 缠绕（runStarts 分段）+ 未闭合
  子路径 fill 隐式闭合（回连段，Z 已闭合时退化零长度）+ **闭域边界点命中**
  （轮廓采样距离 ≤1e-9 按 fill 命中——winding 对边界点可计 0，
  svg-small-big-path (5,110) 底边顶点定谳面）。
- **stroke 区域**：rect 精确环带（外方 `[x−s,x+w+s]×[y−s,y+h+s]` 内且非
  严格内开区间，90° miter 角实定——rect-hittest-002 (60,20) 角点定谳）；
  circle/ellipse 96 点轮廓采样距离；path/polyline 轮廓段距离（moveto 跳段
  不连线）+ 数据顶点 join 圆盘（round 近似，miter 尖 FIXME(svg-compat
  M3)，slice-1 语料未消费）；line 仅 stroke（无 fill 区域）。
- **pointer-events 关键字表**（SVG2）：none/visiblePainted（缺省）/
  visibleFill/visibleStroke/visible/painted/fill/stroke/all——继承属性就近
  祖先回溯；visibility 继承门（visible* 族要求）+ display 属性门；
  painted 判定走属性面（fill 缺省 black=painted、stroke 缺省 none）。
  bounding-box 关键字未实现（FIXME(svg-compat M3)，归 style-system 协同
  切片）。
- **transform 链逆映射**：祖先链收集后自根向叶左乘复合（M = L_外 · … ·
  L_元素）；最外层 svg 局部 = gBCR left/top 平移 ∘ transform 属性（runner
  body 8px margin 偏移经 gBCR 是唯一 client 基准源；实测 root svg
  gBCR=(8,8,420,300)）；嵌套 svg 局部 = x/y 属性平移 ∘ transform 属性
  （shim gBCR 不含嵌套 x/y，实测 nested gBCR=(8,8,100,30) 不可用）；
  transform 属性文法自析（translate/rotate/scale/matrix/skewX/skewY，
  rotate 三参中心式，列表左到右后乘，未知命令合法前缀截断）。
- **命中序**：`querySelectorAll('svg, svg rect, …, svg path')` 文档序逆序
  首中即最顶层（SVG 绘制序 = 文档序）；svg 视口命中面在自身用户空间
  0 基（视口缺省宽高回落 innerWidth/innerHeight）；无 SVG 命中 → null。
- **集成两点**：① `document.elementFromPoint` 无宿主时回落 `_zwSvgHitSel`
  （不落 gBCR 最小盒启发式——SVG 形状在旧近似下恒不可见）；②
  `_zwHitTestSel`（指针事件定向）host 路径后、gBCR 近似前插 SVG pass。
- viewBox 缩放/百分比尺寸/clip/嵌套 svg 视口裁剪未及（FIXME(svg-compat
  M3)）；fill/stroke/stroke-width 祖先属性继承未建模（SVG 继承属性，
  slice-1 靶面全部元素直署属性，零 corpus 消费——见卡点）。

## 修出三轮

1. **root 判定方向 + 复合顺序**：首版 rootSeen 自叶向根判定——嵌套 svg
   被误当最外层（吃 gBCR），真根吃 x/y；且复合自叶起右乘致顺序倒置。修正：
   rootIdx 取链中下标最小 svg（自根向叶左乘）。诊断页实测注入（QSA 文档序
   确认 + gBCR 数值取证）。
2. **探针坐标基准**：runner body 8px margin 经根 gBCR 入链——探针断言点
   全部换算 client 坐标（user + (8,8)，nested 另加 (0,270)）；「视口外
   null」断言点改根 client 盒外真外部。
3. **fill 闭域边界**：winding 对轮廓上点计 0 → (5,110) 顶点漏命中（期望
   path、实返 nested svg）。修正：轮廓采样距离 ≤1e-9 按 fill 命中。

## 验证

- **探针 m3-probe.html 重写 11/11 Pass**（重写为断言修复后行为：环带/
  miter 角/pe 关键字/visibility/rotate 逆映射/polyline 隐式闭合/line 仅
  stroke/嵌套 svg 平移/根视口面+视口外 null；旧版断言 gBCR≈60 与
  computed style 的段落删除——两者属挂账面非本切片语义）。
- **靶面四案 86/86 全绿**（基线全灭）：rect-hittest-001 4/4、
  rect-hittest-002 53/53（含 miter 角/rotate 案）、ellipse-hittest 24/24
  （含 pointer-events="none" 属性装饰椭圆/旋转椭圆 stroke 带）、
  svg-small-big-path 5/5（嵌套 svg x=−250000 大数偏移）。
- 同目录连带盘点（corpus 全量 elementFromPoint 消费面 9 文件）：
  svg-pointer-events-bbox 4/23（4 个 Pass 为 fill 缺省黑与不在圆内巧合，
  19 子挂账——全案走 CSS 类规则 `.test { fill: blue; pointer-events:
  bounding-box }`：CSS cascade + bounding-box 关键字 + text 字形格 +
  image bbox 均未及）；text-hittest-001/002/003 0/3（Ahem 字形格 +
  per-run visibility，归文本几何挂账）；svg-root-border-radius 0/1（CSS
  border-radius 裁剪，归 style-system 协同）。
- 全 corpus 对账（基线 = S4c 收口 1104P/590F/41T = 63.6%）：见下节。

## 全 corpus 对账

基线 = S4c 收口全量（/tmp/s4c-corpus-r4-1827.log 口径，1104P/590F/41T =
63.6%）。本轮 r1（runner 二进制含本切片改动，guard 1800s 全程一次跑完）：

- **原始全量 1201P/504F/41T/1746 = 68.8%**（+97P/−86F，分母 1735→1746
  为 m3-probe 重写 4 子→11 子、+11 全部本切片探针）。
- 转移矩阵：1104 Pass→Pass、504 Fail→Fail、**86 Fail→Pass**、
  41 Timeout→Timeout——**零 Pass 回归、零 DOWN、Timeout 集合恒等**。
- 探针剔除同分母（1701）：1070P/590F/41T = 62.9% →
  **1156P/504F/41T = 68.0%**（恰 +86P/−86F）。
- +86P 升级聚类全部落在 hit-test 靶面：rect-hittest-002 53、
  ellipse-hittest 24、rect-hittest-001 4、svg-small-big-path 3、
  svg-pointer-events-bbox 2——无靶面外移动，无归因悬案。
- DC-4：`cargo fmt --all --check` EXIT=0；定向 `cargo test -p zero-engine`
  （test-guard 4G/8G/3000s）**3373P/0F，EXIT=0**（uievents/pointer 定向面
  共用 `_zwHitTestSel`，回归网由此覆盖）。

## 卡点与划界

- **CSS cascade 命中面**（svg-pointer-events-bbox 19 子 + border-radius
  1 子）：class 规则的 fill/pointer-events/visibility + border-radius 裁剪
  几何——style-system 协同切片，M4 归口。
- **文本字形命中**（text-hittest-001/002/003）：Ahem 字形格（1em 方格、
  baseline 0.8em）+ 逐 run visibility——文本几何挂账面（M4 定稿归口，与
  getComputedTextLength 族同批）。
- **祖先 paint 属性继承**：fill/stroke/stroke-width 为 SVG 继承属性，
  当前只读元素自身属性；slice-1 靶面零消费，升级排 M3 后续切片。
- **miter join 尖**：path stroke 顶点 join 圆盘为 round 近似；slice-1
  语料锐角 path stroke 无消费者，FIXME(svg-compat M3)。
- **viewBox/百分比/clip/嵌套视口裁剪**：FIXME(svg-compat M3)。
