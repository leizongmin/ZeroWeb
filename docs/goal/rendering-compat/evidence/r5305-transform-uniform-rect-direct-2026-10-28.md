# R5305 — css-transforms-1 均匀 scale 图元级直映（TP 场景重采样 backdrop 外扩缺陷）——background-rounded-image-clip-002 ✗0.85→✓0.00 翻绿，附带 huge-length-tiny-scale ✗10.08→✓0.00、transform3d-preserve3d-005 ✗2.08→✓0.00，corpus 15742P/850F→15745P/847F 恰三案翻绿零翻转

**日期**: 2026-10-28
**驱动案**: `css/css-backgrounds/background-rounded-image-clip-002.html`（✗0.85%，fuzzy
maxDifference=0-5 / totalPixels=0-100）
**corpus**: **16592/15745P/847F**；失败集 vs R5304（15742P/850F）归一化 diff 恰
{−目标案，−huge-length-tiny-scale，−transform3d-preserve3d-005}，**零意外翻转**。

## 取证链

1. **diff 形状**（PNG 直方图 + bbox）：ref 全绿 200×200；ZW 绿 bbox 两侧逐字节同
   (8,50)-(207,249)，唯红环 (3,45)-(212,254) 4100px。红环 bbox = wrapper 200×200
   **中心放大 1.05 的足迹**（210×210，±5px）——页面唯一红色源是 `.wrapper{background:red}`，
   却出现在 wrapper 矩形之外。
2. **探针分离**（wpt-data 自匹配 5 页，已删）：E（无变换，透明边框 + padding-box 渐变）
   渲染正确（红严格限 wrapper）→ 缺陷在变换路径；D（同 E + `scale(1.05)`）复现外扩
   （红 (3,3)-(212,212)）→ 缩放足迹即病灶；A/C（`background:green` border-box 裁剪）
   绿满足迹不可见红，未隔离——探针须逐字复刻驱动案结构。
3. **机制**（源码定位）：TP `TransformPrimitive.rect` = R3915 并集（原 rect ∪ 变换后
   四角 bbox）；`apply_transform_post`（render-foundation cpu/mod.rs）对整个并集
   **先清白、再对保存的全场景像素反向采样**。scale-up 时扩展带内输出像素的源点落回
   原 rect 内 → 采样到 rect 内 **backdrop**（wrapper 红底）→ backdrop 被复制到元素框外。
   规范语义（css-transforms-1 §transform-rendering）：变换作用于**元素自身渲染**，
   backdrop 原位不动——像素级全场景重采样无法区分元素内容与 backdrop（R3901 已记录
   同类「清白祖先/兄弟内容」缺陷，纯 translate 因此走图元级平移）。
4. **附带两案定谳**（stash 单因子 A/B：基线三案全红 10.08/2.08/0.85，修复全绿 0.00；
   kill-switch `ZW_TRANSFORM_UNIFORM_RECT_DIRECT=0` OFF 三案精确回基线红值）：
   - `huge-length-tiny-scale`：`scale(0.005) origin 0 0`，40000×40000 盒 + 2000px
     border——TP 对近全屏巨盒清白重采样破坏边框带保真；直映按精确几何重绘。
   - `transform3d-preserve3d-005`：三层嵌套全纯 translate，但 `translatex(200%)` 是
     TranslateMixed（% 形态走 matrix 路径发 TP）——直映门（a==d，b=c=0，含平移分量）
     使混合列表获得 R3901 图元级平移同语义，消 TP 场景清白。

## 实现（engine paint，kill-switch `ZW_TRANSFORM_UNIFORM_RECT_DIRECT=0`）

- **`map_uniform_scale_primitives_since()`**（paint/transform.rs，pub）：TP 矩阵
  b=c=0 且 a==d>0（均匀轴对齐 scale，含平移分量）且自 `counts_before` 快照起新增图元
  **仅 {fill, gradient}**（rounded_rects/shadows/images/glyphs/strokes/filters/
  blend_modes 计数全零增量）时，以 TP 全矩阵仿射直映图元几何：fill rect 四角映射
  （axis-aligned 保形）；gradient rect/clip 端点/圆心映射、径向半径 ×a（均匀保圆）、
  锥形仅圆心（均匀不改角）。返回 false = 未动图元，回落 TP。
- **接线**（painter/mod.rs TP 发射 `Some(mut tp)` 臂）：直映门 = kill-switch +
  `parent_perspective.is_none()`（R5174 折叠缩放随深度变化非纯均匀）+
  `compositor_transform.is_none()`（导出动画须 TP 载体）+ `!inside_transformed_ancestor`
  （同 R5085 豁免：祖先二次重采样相位分化）+ **transform 列表无 `Perspective()` 函数**
  （R5174/R5175 折叠域 corpus 校准面保持 TP——首轮漏此门致 r5175/r5176 两个既有单测
  FAIL，加门后 3570P/0F）+ 助手返真。path_fills/path_strokes 无快照计数（canvas/SVG
  桥域）不入（助手返 false 回落）。
- 语义：变换作用于元素及其整个子树的渲染（子树图元全覆盖），backdrop 零扰动——
  TP 路径保留给含字形/圆角矩形/图片/阴影等图元类与非均匀/旋转/倾斜矩阵（corpus
  校准域，零改路）。

## 门禁

- 全量 corpus 两次（Perspective 门前后各一）最终 **16592/15745P/847F**，失败集 diff
  恰 {−3 翻绿}；perspective 家族 FILTER 失败集与基线逐案一致。
- kill-switch 双向：OFF → 三案精确回基线红值（0.85/10.08/2.08）；stash A/B 同证。
- fmt 0 diff；clippy zero-engine `--all-targets -D warnings` 0 告警；engine lib
  **3570P/0F**（+4：直映映射真值/纯平移矩阵覆盖/非均匀与旋转拒绝/glyph 子树回落）。
- product-smoke PASS（welcome oracle **15.50% 持平** + struct 全 PASS）。
- bench-gate 定向 zero-engine **GATE PASS**（19 指标 NEW=7 全在预算内；首轮 loadavg 16
  忙判定 ABORT 按设计未强制，1min 降回 8 后空闲重试过）。
- make test 全量未跑（改动面 = engine paint transform 发射臂 + 同 crate 测试；消费面
  由 corpus 两次全量 + product-smoke + engine lib 覆盖，R5301-R5304 同判例）。

## 挂账

- TP 场景重采样的结构局限在**含字形/非均匀/旋转矩阵 + 透明内容**域仍开放（本路径
  只覆盖均匀 scale 矩形图元子集）；R5007 双线性 AA 相位与直绘 AA 的 fringe 差异在
  fuzzy 阈内未显红，后续遇 1-2% 变换族残红可先查改路不对称。
- R5304 挂账全维持（002.tentative、types.rs 注释、R5299 三项、013 ellipsis host、
  vertical-rl 域、flake 账本）。
