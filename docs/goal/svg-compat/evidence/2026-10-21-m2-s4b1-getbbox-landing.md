# M2 S4b-1 getBBox 属性级面——SVGGraphicsElement.getBBox 形状/容器/非渲染/transform（2026-10-21）

S4a 收口（306613a55）后接续。单文件落地：part03.js（getBBox 递归收集 + 形状
公式盒 + 容器并集 + 渲染判定 + use 解引用；flatten 增 anchors/drawn 追踪与
三处退化修正）。本 session 端到端持有；并行看护 session（第六次重叠）侦察
在途即退避，其断言面账与本方实现互证（见 session-collision-watchdog 第六
次记录）。

## 实现

### 语义基线（全 #1018 决议口径，WPT getBBox-01..10 对位）

- **非渲染 → 零盒**：display:none（自身或任一祖先）+ defs/symbol/clipPath/
  mask/marker/pattern 子树 → (0,0,0,0) 不抛；visibility:hidden/opacity:0 仍
  算渲染。display 三源任一 none 即非渲染（computed style + presentation 属性
  + style 内联——runner computed 不反射 presentation 属性时不可单源定谳）。
- **形状**：rect w/h≤0（缺省 auto→0）、circle r≤0 非渲染零盒；ellipse 负
  半径取另轴（geometry.html §Sizing）后 ≤0 零盒；渲染则公式盒（circle/ellipse
  用 (c±r) 非采样——96 点 ring 采样在 EPSILON 2^-24 下必挂）。line/polygon/
  polyline/path 点集 min/max；**零面积盒保留定位**（'M40 20h0' → (40,20,0,0)）；
  path 无产线段点取**最后 moveto 锚点**（-07 path_multi_moveto (30,40,0,0)
  定谳，非锚点并集）；无点 → 零盒。
- **容器**（g/a/svg/switch + 兜底）：渲染子盒并集，子 transform 属性列表
  **右先**映射入容器用户空间（S2c _zwSvgTransformParse 复用）；空盒子跳过、
  零面积定位盒参与（-01 g8 (40,20,60,80)）；查询元素**自身 transform 不参
  与**（-08 H 面「bbox 在元素自身坐标系」）；svg 根 = 子并集非 viewport 盒。
- **use**：href（含 xlink:href）解引用递归 + (x,y) 平移；**影子树语义**——
  引用 defs/symbol 内容照常渲染（tag 归属守卫豁免，display:none 阻断保留，
  -05 use with fill 定谳）；未解析/引用空盒 → 零盒。深度 32 防引用环
  （shim 代理身份不保证，访问集不可靠，深度帽为主 guard）。
- **image/foreignObject**：渲染且 w/h>0 → 属性盒；零/缺省/负维 → 零盒。
- **text 族**：定位零面积盒 (x,y,0,0)——字形度量挂账（FIXME(svg-compat)，
  与 getComputedTextLength 同归口 M4）。-02 tspan 精确值面、-05 text 三子
  在此口径下仍 F。
- **SVGBoundingBoxOptions**：fill:false 且无 stroke/markers/clipped → 全
  paint 关闭 (0,0,0,0)（-09 fill:false stroke:false empty bbox 定谳）；
  stroke/markers/clipped 请求回落 fill 盒近似（zero-length 与 fill 同盒断言
  子成立，几何外扩子挂账 S4b-2——FIXME(svg-compat) 标注）。

### flatten 三处退化修正（共享路径，getTotalLength 零回归）

- arc 端点重合 → 弧省略不产点（F.6.2 退化分支；否则中心退化公式产幻影弧点，
  -07 path_arc_same_point 定谳）。
- cubic/quad 全控制点重合 → 不采样（浮点误差产 9.999… 幻影偏移，-07
  path_degen_quad assert_equals 定谳）。
- anchors/drawn 追踪入 lens 返回值（bbox 消费；getTotalLength/getPointAtLength
  路径不变）。

## 验证

- 机制探针 10/10 Pass（gitignored wpt-data/svg/s4b-probe.html）：形状公式盒/
  零面积定位/零维零盒、容器并集空盒跳过、transform 自身不参与+子右先合成、
  退化几何（负维/半径替换/多 moveto/重合弧/空 points）、弧盒内接不越界、
  渲染判定（visibility/style-none/继承/defs）、use 平移/未解析、text 定位
  零面积、detached + DOMRect 返回型、动态增删逐调用重算。探针迭代两轮修出：
  transform 列表顺序反了（translate(5,7) scale(2) 先 scale）+ 本条各退化面。
- **getBBox 靶面**（-01 19/19、-04 1/1、-06 3/3、-07 49/49、-08 20/20、
  -10 4/4、replaceitem 1/1、transform-dynamic-update 1/1 八案全绿；
  -03 11/12、-05 4/15、-09 20/24 分面实收）。
- **全 corpus（486 案，探针剔除）：997P/665F/40T/0NR = 58.6%**（含 NR 分母
  口径，S4a 858P/804F/40T = 50.4% 以来 **+139P/-139F**，Timeout 40 持平，
  全绿案 125→133，逐案零 DOWN，子测试级零回归，分母稳定）。
- 意外面：geometry/svg-image-intrinsic-size-with-cssstyle-auto 0/32→6/32
  （案内 getBBox 消费子随面翻绿，image 加载面仍挂账）。
- path/parsing 复跑 116P/0F 与 S4a 基线一致（flatten 三修正零回归）。
- **DC-4 门禁口径（本轮）**：全 workspace `make test` 在第一步 `cargo fmt
  --check` 红于**并行 comp-anim 流** `apps/compositor/src/frame_dump.rs`
  （9799e6435 引入，origin/main 同红，非本流文件——run-rules §10 归因，
  §9 不越流硬解，碰头信号记 master.md）。测试段改走 quality-gates 定向
  守卫入口：`cargo test -p zero-engine`（本切片唯一受影响 crate）
  test-guard 包裹 EXIT=0。goal 级 DC-4 全绿由 fmt 红修复方（comp-anim 流）
  或合流后轮次复核。

## 卡点与划界

- **S4b-2 SVGBoundingBoxOptions 扩展**（-05 11F + -09 4F）：stroke 外扩
  （miter/bevel/round join + butt/round cap，-05 path with stroke 精确值
  6.2 = miterlimit 1 戦 miter 数学）、markers（markerUnits/viewBox/refX/
  refY/orient 顶点放置）、clip（clipPath 三单位制 + 嵌套 clip-on-clip +
  use-in-clipPath + 空表零盒 + 非法引用不裁）——paint 几盒子系统，独立切片。
- **文本几何挂账**（-02 tspan Ahem 精确值 + -05 text 三子）：字形度量依赖
  布局/字体——按 goal 划界 rendering-compat 协同，M4 定稿归口。
- **image 内在尺寸挂账**（-03 image1/2 + geometry svg-image-intrinsic-size
  族）：负维→auto→intrinsic ratio 需图片加载态（naturalWidth）——图片加载
  协同。
