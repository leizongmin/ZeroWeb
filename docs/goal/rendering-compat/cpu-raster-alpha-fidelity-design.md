# 设计: CPU 光栅 alpha 保真通道（filter-function 离屏元素层化 + 透明底语义）

**状态**: ✅ SLICE-3 IMPLEMENTED（三切片全部落地：slice 3 于 R4284；DropShadow 真实现延后）
**日期**: 2026-09-13
**轮次**: R4283（设计 + R4282 注入链修复 + slice 2 实施）、R4284（slice 3 实施）
**前承**: R4282-F（isolate ImageCache 共享，R4283-⓪ 补齐断裂注入链）、R4275（SourceGraphic
隔离设计 = 本设计的机制前承）、R4276-R4281（isolate 机制 7 门禁落地与 oracle 验证）
**领域**: source-alpha-001（opacity() 真透明）、css-filters opacity/backdrop-filter alpha
深域、isolate 离屏白底漏色、drop-shadow alpha 轮廓

## 1. 问题

R4282 混合列表解除尝试（试后回退）暴露了 CPU 光栅的根本性 alpha 缺陷：
**离屏先函数后 resvg url 的声明序结构正确**，但 `opacity()` CPU 模拟不改 alpha
（帧缓冲 alpha 恒 255）→ `opacity(0)` 产黑盒而非透明，source-alpha-001 仍红且
tainting-css-dropshadow 对绿转红（净负回退）。

根因三层（本轮实证定位）：

1. **`apply_filter` 后处理模型**（`render-foundation/src/cpu/effects.rs`）：
   滤镜作用在**已合成主帧**上——背景已烧入、alpha 恒 255、每算子硬编码写回
   `255`。`Opacity(amt)` 只能「乘 RGB 变暗」模拟（白底下 `opacity(0)` = 黑盒；
   正确 = 透明/混背景色）。其余色算子同样丢 A。
2. **离屏 isolate 白底**（`cpu/mod.rs:201`）：`render_full_scene_region` 基底
   `new_filled(255,255,255,255)`——filter url() 的 SourceGraphic 在元素盒外
   （filter region 余量）是**不透明白**而非透明。hueRotate/feImage 族（灰度不变
   色算子）侥幸不受影响（oracle 16/18 实证），但 blur/drop-shadow/位移类链在
   非白背景会漏白盒、在 region 余量产白边。
3. **painter 逐图元流式发射**：function 列表没有元素级分组，主帧后处理无法拿到
   「仅该元素子树」的像素（背景混入后 alpha 信息已不可逆丢失）。

## 2. 方案选型

| 方案 | 描述 | 评估 |
|------|------|------|
| A. 仅修 `apply_filter` alpha 分量 | 主帧上直改 | ✗ 主帧背景已烧入，`opacity` 无法知道局部背景色（彩色底不可能正确）；只有透明底才有真 alpha 语义 |
| B. 复用 P2-7 blend 元素层（render-foundation 内） | DrawOp::Filter 前置标记 + 独立 src 缓冲 | ✗ blend_src 为**连续兄弟共享**单缓冲，filter 需按元素隔离（blur 会把兄弟内容一起糊）；per-element 缓冲簿记 = render 循环新机制 |
| **C. 扩展 engine FilterIsolate 机制（本设计）** | function 列表走 R4276 隔离机制：子树旁路离屏 → CPU alpha 保真应用 → 占位回贴 | ✓ 复用已验证的 7 门禁 / paint_skip / 占位 ImagePrimitive / `render_image` alpha 回贴（sa<255 已走 blend_pixel）；R4282 回退结构可直接复活 |

选 C 的关键既有事实：
- `render_image`（cpu/mod.rs:1037）对 `sa<255` 已做 alpha 混合、`sa=0` 跳过 →
  **isolate 透明输出回贴通道现成**；
- `FilterIsolate { primitives, region, filter_node_ids, key }` 结构与
  `apply_filter_isolates`（pipeline/mod.rs）已承担 url() 链，扩展 functions 字段
  即可；
- `blend_rgba`（effects.rs:357）已有 src-over alpha 合成公式可复用。

## 3. 数据流（方案 C）

```
collect_filter_isolates（painter，主遍前）：
  门禁放宽——除 url() 引用外，纯 function 列表 / 混合列表（url+function 声明序
  交错）也进 isolate（门禁④⑥⑦ transform/region 原样沿用；门禁②③ url 专属）。

paint_isolate_subtree（既有）：子树旁路图元 → 独立 RenderPrimitives sink。

apply_filter_isolates（pipeline，paint 后）：
  1) 离屏栅格化：render_full_scene 透明底变体（slice 2）→ offscreen fb（直
     alpha，A 承载元素形状）
  2) 链/函数应用（按 CSS filter-value-list **声明序**逐项）：
     - function 项 → render-foundation apply_filter（slice 1 重写：直 alpha
       语义，Opacity 写 A 分量，blur 在 premultiplied 域，DropShadow 由 A 轮廓
       派生）作用于 offscreen fb
     - url 项 → build_wrapper_svg + resvg（既有；SourceGraphic = 上一步输出，
       resvg 原生透明语义）
  3) 输出 rgba → canvas_images 回注（既有）→ 占位 ImagePrimitive →
     render_image alpha 混合回贴主帧（既有）
```

## 4. 切片划分

### Slice 1 — alpha 基座（render-foundation，行为零变的重写）

`apply_filter` 重写为直 alpha 语义（输入帧 alpha 任意）：

- **Opacity(a)**：`A' = A * a`，RGB 不变（直 alpha 定义）。
- **Blur**：premultiply → RGBA 四通道 box-blur → unpremultiply（A 形状随模糊
  扩散——真高斯 alpha 轮廓）。
- **DropShadow(ox,oy,blur,color)**：由 A 通道提取轮廓 → 平移+模糊 → 按 color
  着色 → src-over 合成在源之下（替换 R3851 的 border-box 矩形近似在 isolate
  路径的精度；painter 的 ShadowPrimitive 前置发射路径暂不动）。
- **色算子**（Brightness/Contrast/Grayscale/HueRotate/Invert/Saturate/Sepia）：
  RGB 公式不变，**A 透传**（不再硬编码 255）。

主帧路径逐位不变（主帧 A 恒 255 → 色算子透传 255、Opacity 例外见下）：主帧
`Opacity` 是唯一行为变化点——**slice 1 内主帧 Opacity 保持旧模拟**（`RGB*=a` +
A=255），用 `FilterPrimitive` 上的调用方区分（或入口参数），保证全量 corpus
零翻转可验证；slice 3 落地后主帧路径只剩 no-isolate 回退语义，再评估是否换
「向白混合」近似。

**验证**：render-foundation 单测（透明底 opacity(0)=全零、opacity(0.5) 直 alpha
A=128、blur 边缘 alpha 渐变、drop-shadow 轮廓）；全量 corpus `--json` 逐案对照
零翻转。

### Slice 2 — isolate 离屏透明底【已实施，R4283-F】

**实施时修订**：原方案（`render_full_scene_region_into` 到 `FrameBuffer::new`
透明基底）被事实否定——`blend_pixel`（cpu/mod.rs:1016）**硬编码 `d[3]=255`**，
所有图元写入方按不透明底语义工作，直渲染透明底会产「RGB 暗化 + 假不透明」
（P2-7 blend_src 路径今日的潜在暗边缺陷）。对全部写入方做 alpha 保真改造 =
光栅栈大改，爆炸半径不可接受。

**落地方案：双色底 matte（dual-matte）直 alpha 提取**——零侵入光栅栈：

- `render_full_scene_straight_alpha`（render-foundation cpu/mod.rs 新公共入口）：
  子树对白底与黑底各渲染一次，逐像素解直 alpha——src-over 凸组合下
  `Cw = C·α + 255·(1−α)`、`Cb = C·α` ⇒ `α = 1 − (Cw−Cb)/255`（三通道均值降噪 +
  钳位防非凸写入方）、`C = Cb/α`。成本 2× region 渲染（isolate 仅 filter 元素
  触发，可接受；单遍 alpha 保真 sink 为后续 OPTIMIZATION）。
- `apply_filter_isolates`（engine pipeline）离屏栅格化切换至该入口。

**A/B 实证（全过）**：
- effect-reference 族 oracle **14/18 → 15/18（83.3%）**，真通过 8→11；
  on-span 1.05%→0.61% 翻绿。
- filter-effects 全目录 credible 163（50.5%）→**159/309（51%）**、真通过
  84→88；fecolormatrix-type 3.69%→0.64%、feComposite-intersection-feTile-input
  1.80%→0.00% 翻绿（resvg 链受益于真 alpha SourceGraphic）。
- corpus self-source **14726→14728（净+2 零翻红）**，翻绿两案与 oracle 侧
  交叉印证。
- 单测 ×2（不透明内部/余量透明、半透明凸组合可逆）。
- 白底漏白盒潜伏 bug（彩色背景页 filter region 余量涂白）一并消除。

### Slice 3 — function/混合列表 isolate 化【已实施，R4284】

- `FilterStep`（Function/Url）声明序步骤链——`<filter-value-list>` 交错顺序保真
  （filter-effects-1 §7.1）；`IsolateSpec`/`FilterIsolate` 从 filter_node_ids
  迁移到 steps。
- `collect_filter_isolates` 门禁①放行 function 列表（url 或 function 任一即候选）；
  `apply_filter_isolates` 按声明序交错应用（function→CPU
  `apply_filter_straight_alpha`、url→resvg），isolate 根自身 FilterPrimitive 在
  旁路绘制期间抑制（`isolate_root_suppressed`，防离屏双应用）。
- `apply_filter_straight_alpha`（render-foundation，设计 slice 1 的直 alpha 变体）：
  Opacity 写 A 分量、Blur premultiplied 四通道（前缀和盒模糊）、色算子 A 透传、
  DropShadow no-op；主帧 `apply_filter` 原体未动（主帧行为零变化）。
- **实施中新增门禁**（A/B 回归实证逐条落地）：
  - ⑧布局根（html）旁路——根 filter 与 canvas 背景传播/fixed 交互
    （hidpi-invert-filter-background 0→100% 回归）；
  - ⑨含 DropShadow 列表旁路——ShadowPrimitive border-box 近似在 isolate 直
    alpha 语境退化（filters-drop-shadow-002、tainting-css-dropshadow-currentcolor
    回归；R4282 试退同教训）——真 alpha 轮廓 drop-shadow 延后；
  - ⑩含 backdrop-filter 旁路——isolate 抑制后 backdrop 图元丢失
    （backdrop-filter-plus-filter 回归）；
  - function 列表 region = 元素盒+blur/offset 外扩 **∪ 子树布局边界**（
    filtered-block/inline-is-container 实证：region 须随源内容界扩展；
    `subtree_layout_bounds` 保守上界，overflow 裁剪由子树 ClipPrimitive 照常
    生效）。首版「positioned 后代旁路」门禁被 A/B 否定（clip-under-filter 依赖
    isolate 裁剪语义、feComposite-intersection 依赖 url isolate 与 ::after
    isolate 协同）。
- root-element-with-opacity-filter-001 因门禁⑧回退 5.82%（接受成本，oracle
  inconsistent 记账）。

**验证门禁核销（全过）**：
1. source-alpha-001 oracle 2.08%→**0.00%** ✓（effect-reference 族 15/18→**16/18
   88.9%**，真通过 12）；
2. filter-effects 全目录 credible 159→**165/309（53%）**、真通过 88→**92**；
   逐案 0 回归、6 案翻绿（fixed-pos-filter-clip ×2 99%/26%→0、
   will-change-blur-filter-under-clip 5.20→0、filters-opacity-002 9.14→0.81、
   css-filters-animation-opacity 2.77→0.71）——css-filters/backdrop-filters 无
   oracle shots（capture 为后续项），以 corpus 为主门禁；
3. tainting-css-dropshadow 保绿 ✓（currentcolor 0.00%、001 @2.08% 为修复前
   既有值）；
4. corpus `--json` 逐案 **14728→14737 净+9、零翻红**（blur-text、
   blur-clip-stacking-context-002、filtered-inline-applies-to-float、
   overflow-scroll-resize-visibility-hidden 等）；
5. 定向 bench-gate（zero-engine + zero-render-foundation，34 指标）**GATE
   PASS**（负载噪声首跑 FAIL 经安静复测证伪——memset 基准 ±70% 方差为噪声
   证据）。

## 5. 复用清单（零新跨 crate 协议）

- FilterIsolate / apply_filter_isolates / 7 门禁（R4276-R4281）
- paint_skip_nodes 主遍抑制（R2197）
- canvas_images 回注 + 占位 ImagePrimitive + render_image alpha 回贴（R3268）
- resvg 包装 SVG 通路（R4276）
- blend_rgba 的 src-over 公式（effects.rs）

## 6. 明确不做

- GPU 路径（gpu/renderer/filters.rs）不同步——GPU 已有独立 blur pipeline，alpha
  语义对齐待 CPU 通道验证后另立切片。
- backdrop-filter：依赖 backdrop 快照（背景层读回），机制不同，另立设计。
- 主帧 Opacity「向白混合」近似：slice 3 后按残余调用方再评估。
- displacement-negative-scale resvg vendor patch：仍挂待用户决策，不阻塞本设计。

## 7. oracle 现状基线（2026-09-13，slice 2 落地后）

- effect-reference 族 **15/18 oracle-pass（83.3%）**；真通过 11（R4281-N 基线
  11/18=61.1%、R4283-⓪ 修复后 14/18=77.8%）。余 3 案：displacement-negative-scale
  @2.08%（vendor 待决策）、source-alpha-001 @2.08%（本设计 slice 3 目标）、
  rename-001 @1.59%（chromium 截图侧偏差——ZW-rename≡ZW-after 0.00%、
  ZW-after≈CHR-after 0.08%、CHR-rename≠CHR-after 0.87% 二分实证，非引擎差异）。
- filter-effects 全目录 credible 159/309（51%；链路 47.7%→50.5%→51%）。
- corpus self-source 14728/16594（88.8%）。
