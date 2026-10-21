# M2 S4b-2 paint 几何盒——getBBox stroke 外扩 / markers 顶点放置 / clip 交集（2026-10-21）

S4b-1（evidence/2026-10-21-m2-s4b1-getbbox-landing.md）收口后接续。单文件落地：
part03.js（flatten verts/vAnchors/zeros/runStarts 追踪 + _zwSvgGeomLens 同款字段 +
collect paint 线程化 + text 定位点集重写 + S4b-2 子系统块 + getBBox 装配）。
本 session 端到端持有。

## 实现

### SVGBoundingBoxOptions 分旗语义（spec types.html §SVGBoundingBoxOptions）

- 全 paint 关闭（fill:false 且无 stroke/markers）→ (0,0,0,0) 早退（-09 定谳）。
- **stroke 面**：`_zwSvgStrokeBbox`——rect/circle/ellipse 公式盒 ±sw/2（零维
  形状非渲染不复活，#1018）；line/polyline/polygon/path 走 `_zwSvgGeomLens`
  + `_zwSvgStrokeWalk`：逐样本法向 offset 点 + run 内部样本 sw/2 外逼近盘 +
  数据顶点 join（round 圆盘 / miter 尖 h/sinHalf·bisector 且 1/sinHalf ≤
  miterlimit，超出落 bevel / bevel 由 offset 点覆盖）+ run 端 cap（butt 无
  延伸 / round 圆盘 / square 延伸）+ 命令性零长段 zeros（round/square 产点，
  butt 不产；moveto-only run 不产——-09 定谳）。
- **markers 面**：`_zwSvgMarkerBbox`——marker-start/mid/end → url(#id) →
  tag==='marker' → content 盒（fill-only，viaUse 豁免 defs 归属）；
  markerUnits 缺省 strokeWidth；viewBox→markerWidth/Height 缩放；refX/refY
  对位；orient 数值角度（auto 未实现按 0，FIXME(svg-compat)）；任意 marker
  盒存在时并 fill 盒（Chrome 定谳 markers-only 含 fill 盒，-05 x=10）。
- **clipped 面**：`_zwSvgClipRegion`（clip-path）+ `_zwSvgClipAttrRegion`
  （SVG1.1 clip 属性）交集：
  - 引用解析 /url(#id)；引用缺失或 target 非 clipPath → 全裁零盒；
    空 clipPath → 全裁零盒（rect-none/image11 定谳）。
  - clipPathUnits=userSpaceOnUse（缺省）绝对坐标；objectBoundingBox →
    **fill 盒**映射（stroke/markers 外扩不入参照——path10-12 定谳：oBB 内切
    圆区域 AABB = fill 盒）；circle 子等比保留圆信息，非等比退 AABB。
  - clipPath 子级并集（spec：子先各自受自身 clip-path 裁切再 OR）——子级
    嵌套 clip：circle∩circle 走 `_zwSvgLensBox` 精确 lens 盒（rect-1
    (10,10,140,140) 定谳），其余 AABB 交；clipPath 自身 clip-path 属性交
    （clipOnClip1）；use 子解引用（含 circle 信息 + (x,y)）。
  - use **查询元素**：区域随 use (x,y) 平移（getBBox-05 use1-7 定谳：
    circle8 (110,20) + use(50,50) → 区域中心 (160,70)）。
  - SVG1.1 clip 属性 rect(t,r,b,l)：四值全自左/上边缘量、相对元素 (x,y)
    原点（CSS2.1；image1/fo1/fo4 定谳）。
  - `_zwSvgBboxIntersect` **严格相交**：重叠零宽/零高（相离或仅边/点接触）→
    EMPTY——退化点盒（text 定位面）被正面积区域裁切即消失（text5/6 对比面：
    Chrome 两侧等值依赖真实字形度量，退化模型下两侧同归零一致）。
- clip 几何消费面渲染判定走 skipTagGuard（同 use 影子树口径）——clipPath/
  marker 子处 defs 子树不因容器 tag 归属被砍（首跑全零盒根因），display:none
  阻断保留。

### text 定位点集重写（S4b-1 定位零面积盒升级）

- text 有直接非空白文本或无元素子时计自身 (x,y) 点；有 tspan/textPath 子时
  取各子定位点并集，text 自身 x/y 不并置（-05 text3 布局定谳：Chrome 盒 =
  有效字形定位点集）。x/y 缺省按 0（非整体跳点）。无点 → (0,0) 零面积。
- 字形度量盒仍挂账（FIXME(svg-compat)，-02 tspan Ahem 精确值，M4 归口）。

### flatten/_zwSvgGeomLens 追踪面

- verts（命令端点）/vAnchors（run 首尾标记）/zeros（命令性零长段）/
  runStarts（moveto 分段）入两处 lens 返回值，供 stroke walk 消费；
  pts/cum/total/anchors/drawn 路径不变（getTotalLength/getPointAtLength
  零改动）。

## 验证

- getBBox 全量（FILTER=getBBox-）：**146P/2F**——唯二 F 均为既有挂账（-02
  tspan Ahem 字形度量：tspan 无 x/y 属性、期望值纯字体度量推导，归口 M4；
  -03 negative sizes：image intrinsic size 图片加载态）。
- **-05 15/15 全绿**（S4b-1 4/15 → +11）：image1-14 clip 全表、path1-13
  stroke/markers/clip 全表、use1-8 区域平移全表、fo1-8、masking-path-07-b
  rect-1/2/3+g1（lens/嵌套 clip/自 clip/use-in-clipPath 全语义面）。
- **-09 24/24 全绿**（S4b-1 20/24 → +4）：stroke 零维面全对位（零长线
  butt 不产盘、'h0'+round 产盘、非渲染零盒 stroke 不复活、moveto-only 不
  产盘）。
- path/parsing 复跑 **116P/0F** 与 S4a 基线一致（flatten 追踪面零回归）；
  path/interfaces 0P/56F 为 S4c 未实现面（下一步切片），非回归。
- 两轮修出（首跑 → 定位 → 复跑）：
  1. clip 面全零盒：clipPath 子沿祖先链命中 defs tag 守卫 → 子全跳过 →
     区域恒 EMPTY。修正：skipTagGuard 豁免（collect viaUse=true 同步）。
  2. path1 stroke 高度 82.578≠83：Q 曲线对称顶点（t=0.5）邻样本重合 →
     chord 切向零 → 法向 offset 欠冲 0.42。修正：run 内部样本补 sw/2 外
     逼近盘（run 端不放——butt cap 无延伸面）。
- 全 corpus（486 案，探针剔除）对账：**1012P/650F/40T/0NR = 59.5%**——
  同分母 1702 断言对 S4b-1 997P/665F/40T 恰 **+15P/-15F**（-05 +11、-09 +4
  与分面实测吻合，其余案零移动），Timeout 40 持平，全绿案 133→135，逐案
  零 DOWN。runner 二进制构建时间晚于 part03.js 末次修改，确认本次对账
  含本切片全部改动。

## 卡点与划界

- **fo5 零盒溯源未闭环**：fo5 (250,250,200,200) clip-path=url(#rect9)
  （clipPath rect (100,100,200,200)）按绝对坐标交集语义应为 (250,250,50,50)，
  Chrome 期望 (0,0,0,0) 且**本引擎实跑亦返零盒**（-05 15/15 全绿含 fo full
  子测试）。同 clip 的 fo6 (0,0,200,200) → (100,100,100,100) 两侧同吻合；
  同区域下 fo5/fo6 结果分叉意味着区域求值路径对两者不等价（引擎侧机制未溯，
  疑解析树位或异常吞没路径）。纸面穷举（绝对/局部/±(x,y) 平移）均无法同时
  解释 fo3/fo5/fo6 三值——行为两侧对齐，机制溯源挂账（探针面 s4b2-probe
  待跑，不阻塞本切片）。
- **path7-9 quirk 判定撤除**：S4b-1 期判定「期望值不可解析」系误判——
  rect5/6/7 本就是合法 clipPath（rect (0,0,100,100) 等），circle4 AABB 恰
  等于 rect5 盒 → path4-6 与 path7-9 期望值自然一致，区域 AABB 语义下全过。
- **文本几何挂账**（-02 tspan Ahem 精确值）：字形度量依赖布局/字体，
  rendering-compat 协同，M4 定稿归口。
- **image 内在尺寸挂账**（-03 negative sizes）：负维→auto→intrinsic ratio
  需图片加载态，图片加载协同。
- **orient=auto 挂账**（FIXME(svg-compat)）：-05 用例 orient 缺省（数值角
  0）定谳面不受影响；auto/auto-start-reverse 需路径切向，M3 几何面后评估。
