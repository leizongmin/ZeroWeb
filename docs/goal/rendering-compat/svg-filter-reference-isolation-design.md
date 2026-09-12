# 设计: CSS filter url(#id) 非常量链的 SourceGraphic 隔离（engine 两遍绘制 + resvg 链应用）

**状态**: 📐 DESIGNED（待实施——本设计按「Phase A 只做设计后实施」成文，实施轮须按
§7 门禁逐项过闸）
**日期**: 2026-09-12
**轮次**: R4275（设计轮，0 net code）
**前承**: R4273-F（url() 常量输出链已落地——feFlood / 常量 feColorMatrix 走填充
快速路径）、R4274-F（SVG2 形状禁用）
**领域**: filter-effects 130 红中 effect-reference 非常量链族 ~5-8 案
（source-alpha-001/002 @2.08%、merge-no-inputs、add-hw、lighting-no-light @4.17%、
on-span @1.21%、obb-dimensions @1.50% 实证归因），并解锁 opacity()/backdrop-filter
alpha 合成深域（同一隔离机制）

## 1. 问题

R4273 只覆盖**常量输出链**（输出与 SourceGraphic 无关 → 无需元素像素即可求值）。
非常量链（feMerge/feComposite/feColorMatrix 带输入系数/feDiffuseLighting…）的输出
依赖**元素自身渲染像素**（SourceGraphic / SourceAlpha），要求：

1. 元素（含子树）绘制到**隔离层**（不含页面背景/兄弟内容）；
2. 对隔离层像素按 SVG filter 语义逐原语求值；
3. 链输出替换元素渲染（filter 失败/引用不可解析 → 无 filter）。

现有机制缺口：painter 逐图元流式发射，元素像素未按元素分组；resvg 在 engine，
FrameBuffer 后处理（apply_filter）在 render-foundation——**跨 crate 应用层缺桥**。

## 2. 方案选型

| 方案 | 描述 | 评估 |
|------|------|------|
| A. render-foundation 内嵌 resvg | 链应用下沉 render-foundation | ✗ 职责混写（SVG DOM 语义在 engine）；重依赖倒挂 |
| B. engine 注册 IoC 回调 | render-foundation 定义 applier trait，engine 启动时注册 | ✗ 新抽象层 + 全二进制注册点遗漏风险（静默无 filter） |
| **C. engine 两遍绘制（本设计）** | 隔离子树重绘到离屏 + resvg 链应用 + 主遍抑制替换 | ✓ 全部复用既有机制（paint_skip_nodes / render_full_scene / resvg 包装 / ImagePrimitive），零新跨 crate 协议 |

## 3. 方案 C 数据流

```
主绘制前（pipeline 每帧一次，engine）：
  style walk 收集 filter 列表含 Url 引用且链为「非常量」的元素 → isolates[]
  （常量链维持 R4273 填充快速路径，不进本机制）

对每个 isolate（engine painter）：
  1) 离屏绘制：以该元素为根调 paint_node（同一 painter 代码路径，独立
     RenderPrimitives sink）→ render_full_scene → offscreen FrameBuffer
     （尺寸 = 元素盒外扩 filter region；坐标平移使盒原点对齐 region 原点）
  2) 链应用（engine，paint/svg_filter_chain.rs 新模块）：
     offscreen region 像素 → PNG 编码（复用 canvas 快照编码路径）→ data URI
     → 包装 SVG：`<svg><filter id="f">{原链 XML}</filter>
       <image width=W height=H href="data:..."/></svg>`
     → resvg::render（同 paint_svg_element 通路）→ rgba
     SourceGraphic = image 像素；SourceAlpha 语义由 resvg 原生处理
  3) 主遍抑制：paint_skip_nodes.insert(元素)（R2197 机制）→ 元素自身图元不发射
  4) 替换：链输出 rgba 以 ImagePrimitive 发射于 filter region
     （链应用失败 → 不抑制不替换 = 无 filter，net-neutral 回退）
```

## 4. 复用清单（全部既有机制，零新抽象）

- `paint_skip_nodes`（R2197）：主遍抑制
- `render_full_scene`（render-foundation 公共 API）：离屏栅格化
- `paint_svg_element` 的 resvg 包装模式：data-URI image + 序列化链 XML
- `ImagePrimitive` + ImageCache ctx_id 槽（canvas 同路径）：结果回注
- `svg_filter_region`（R4273）：region 计算复用
- R4270 taint / R4274 shape-disable：序列化源级预处理的既有先例

## 5. 边界与已知风险

1. **性能**：isolate 元素子树双绘制 + PNG 往返，仅 url() 非常量链元素承担
   （corpus 内 ~个位数元素/页；产品页 0 个）。bench-gate 定向 zero-engine 观察
   page/morning、page/medium 档。
2. **inline 元素**（on-span）：行内碎片化子树以 IFC 片段盒为绘制根，首版仅支持
   单片段（多片段 span 逐片段 isolate 留后续）。
3. **区域外溢**：filter region 可能大于元素盒（userSpaceOnUse）；离屏以 region 为
   界，元素绘制平移量 = region 原点 − 盒原点。
4. **will-change/transform**：isolate 子树内 transform 由同一 paint 路径自然继承；
   元素自身 CSS transform 与 filter 共存顺序 = spec 先 filter 后 transform，
   主遍 TransformPrimitive 作用于 ImagePrimitive ✓。
5. **数据面**：data URI PNG 尺寸随 region 增长；首版不设上限，依赖 region 合理性
   （后续可加 4096² cap + 超限回退无 filter）。

## 6. Kill-switch 与回退

- `ZW_SVG_URL_CHAIN=0`（default-on）：整体旁路 → 行为回 R4273 态（常量链仍走
  快速路径），零回归面。
- 净负回退：A/B 全量 reftest 出现任一新增红 → 先关 kill-switch 记账，再修。
- 常量链快速路径不迁移（R4273 5 案已绿，字节级不动）。

## 7. 实施门禁（下一轮按序过闸）

1. `make test` 67 套件 0 失败
2. `make reftest` 687/687
3. reftest-upstream 全量：目标 +4 案以上（source-alpha-001/002、merge-no-inputs、
   add-hw 预期；lighting-no-light/on-span 视首版覆盖），**零新增红**
4. `make product-smoke` 20.30% 同值 + struct PASS；`make product-smoke-legacy`
   0 struct FAIL
5. `ZERO_WEB_BENCH_CRATES=zero-engine make bench-gate` GATE PASS NEW=0
6. fmt + clippy -D warnings 全 workspace
7. 驱动案 `make import-wpt` 入常驻集 + 控制面记账

## 8. 预期收益

- 直接：effect-reference 非常量链族 4-6 案（@1.21%-4.17% 带）
- 战略：SourceGraphic 隔离机制 + resvg 链应用通路一旦落地，opacity() /
  backdrop-filter 的 alpha 合成与 backdrop 隔离（现挂「待用户决策」深域）即有
  现成机制可迁移——同一隔离层，链应用从 resvg 换 CPU 矩阵/模糊即可
