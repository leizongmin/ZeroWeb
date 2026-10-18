# R5179 corner-shape 真几何（css-borders-4 §3.7/§3.9.4.2 分 regime）

日期：2026-10-18 ｜ 前轮：R5178（替换元素 min/max content 关键字） ｜ 基线 corpus：1,074（R5178 ON 终态）

## 案形与域核

css-borders-4 §corner-shaping 12 案族（基线失败 0.04-5.33% 不等）：`corner-shape` 关键字/函数（round/bevel/scoop/notch/square/squircle/superellipse(K)）配 border-radius 的形角渲染。域核发现**五层根因叠加**：

1. **解析层**：`superellipse(-infinity)`/`superellipse(infinity)` 被 `is_finite()` 拒收（Rust `f64::from_str` 本接受 ±infinity 文本）→ None 回退 round；`squircle` 关键字未映射 superellipse(2)。
2. **几何层**：存储的 K（CSS 原始参数）被当指数用——§3.7 超椭圆单位曲线 x^(2K)+y^(2K)=1 的指数是 n=2K，且 §3.9.4.2 渲染按 K 分 regime：凸侧 K≥0 心=内角点 C、凹侧 K≤−1 心=盒角点 P 的 T^(1/n) 权重参数化、−1<K<0 切线交点族。旧实现单臂极角采样 K，形状显著偏圆/反向。
3. **R4248 门控**：scoop/负 K → None 回退 round，形角几何完全不达消费端（bg clip、border ring、overflow clip、backdrop-filter clip 全部失效）→ notch-mixed 案的未裁剪 legacy 条带、backdrop-filter 两案全红。
4. **边框环内圈半径**：凹侧（notch/scoop/深 K）按 §3.9.4.2 adjusted 公式（r−bw）腿与外圈重合、带宽归零——chromium 实渲染的边框带 = 沿外轮廓的 bw 均匀内描边，凹侧内圈腿 = 内盒角+全半径 R（WPT ref 判例 30px 5px 系；ref 是判准）。
5. **overflow-clip-margin**：shaped overflow clip 多边形建在未扩展盒上——css-overflow-4 §3 要求 clip 边 = border box 外扩 clip-margin（半径不变）。

规范依据：css-borders-4 §3.7（superellipse(K) 单位曲线与关键字等价：round≡K=1、bevel≡K=0、scoop≡K=−1、notch≡K=−∞、square≡K=+∞）、§3.9.4.2（分 regime 渲染算法 step 37/38）、css-overflow-4 §3。

## 实现（四处 + 类型文档）

- **解析映射**（`crates/css-parser/src/values/parse_extended_border.rs`）：`superellipse(±infinity)` → Square/Notch，NaN 拒收，`squircle` → Superellipse(2.0)；+3 解析测试（±∞ 映射/有限参数保留+NaN 拒收/squircle 关键字与四值序）。
- **类型文档修正**（`values/types.rs`）：CornerShapeKind 存原始 K，n=2K 由几何层解释，±∞ 解析期映射——round/bevel/scoop/notch/square 等价关系写全。
- **分 regime 几何**（`crates/engine/src/paint/helpers.rs` `shaped_corner_polygon_in_box`）：R4248 None 回退门摘除；凸侧 K>0 极角采样 |dx/r|^n+|dy/r|^n=1（n=2K）；凹侧合并臂（Scoop ≡ K=−1 → n=2 圆弧；K≤−1 → T^(1/n) 权重；−1<K<0 → 切线族 (1−(1−T)^½, 1−T^½)）；square 段终点换盒角点 P；**深凹 K（n≥64）收拢 notch 阶梯**（偏差 ≤ r·√2·(½−2^(−1/n)) ≈ 0.38px @ r=25，低于 8 段圆弧弦高容差 ~0.049r——亚像素贴边经无 AA path_fill 放大成整像素腿位溢出，判例见 learnings）。
- **边框环内圈**（`paint/painter/border.rs`）：凹侧（Notch/Scoop/Superellipse(k≤−1)）内圈半径 = 全 R，凸侧维持 r−bw 同心内缩（§3.9.4.2 语义）；FIXME 留 scoop 真描边弧（心在外盒角 r+bw，全半径内弧偏差 ≤~1px）。
- **overflow clip 扩展**（`paint/painter/mod.rs`）：shaped fallback 按 overflow_x/y==Clip 各轴外扩 `overflow_clip_margin_length`；R4547 过期注释同步。

kill-switch：无独立开关（R4248 门控删除；回退需还原 helpers 几何 + border.rs 内圈半径 + 解析映射三处，OFF 反证由 corpus 基线承担）。

## 迭代定谳（corpus A/B 同 binary）

| 迭代 | 结果 | 归因 |
| --- | --- | --- |
| 基线 | corner-shape 12 案失败（0.04-5.33%） | 五层根因叠加 |
| ON v1（解析映射+分 regime+门摘除） | 1,074→**1,069 翻绿恰 5**（backdrop-filter、backdrop-filter-overflow、notch、superellipse-bevel、superellipse-negative-inf） | 门摘除使形角几何首次抵达全部消费端 |
| v1 回归发现 | overflow-clip-margin 0.11%→**12.50%** | shaped clip 活跃但未按 clip-margin 扩展 → mx/my 扩展 → 0.30% |
| ON v2（凹侧内圈全 R） | scoop、superellipse-scoop 翻绿；notch-mixed 0.70→0.01%、negative-100 0.70→0.04%（均未达阈） | §3.9.4.2 adjusted 公式凹侧腿重合 ≠ chromium 均匀内描边 |
| v2.5（8→16 段采样实验） | 5 残案逐项**零变化** → 回退 8 段 | 弦高 AA 非残差来源，加密只增顶点成本（最小实现原则） |
| ON v3（深凹 K≥32 收拢 notch） | **1,065 翻绿恰 2**（notch-mixed、superellipse-negative-100） | 无 AA 保守光栅化 × 亚像素贴边（learnings 判例） |

终态残差 3 案：iframe-border 5.33%、overflow-clip-margin 0.30%、square 0.03%（见挂账）。

**像素取证两新知**（已沉淀 learnings/bugs/2026-10/）：①CPU path_fill 无 AA 保守光栅化——run-length 扫描判别（全填=保守、混合色=有 AA），深凹曲线 ±0.26px 贴边被放大成整像素 ±1px 腿位溢出；②常规 border-radius+border 的边框带从未按弧绘制（背景圆、边带直角），self-source corpus 两侧同错自洽掩盖，corner-shape 套件混搭 ref 管线暴露。

## 验证

- unit：`r5179_corner_shape_geometry.rs` ×6——scoop 圆方程采样（盒角心 r=25 容差 1e-3）、superellipse(-100) 收拢 notch（每角恰 [start,C,end] 12 点）、superellipse(0)≡bevel 逐点相等、superellipse(24) 超椭圆方程断言（u^n+v^n=r^n 相对 1e-3，比距角阈值强且与采样参数化无关）、notch 环内圈全半径管线判例（内圈 (30,30) 非 (25,25)）、superellipse(-infinity)→Notch 计算值。css-parser corner_shape ×3。
- 既有钉维持：zero-engine lib **3,183 绿**；fmt（touched crates）干净；clippy 影响域（zero-engine+zero-css-parser --all-targets -D warnings）干净。**workspace 全量 clippy 红为 main 既有**：今日 6d7d6338c（dnd M3-s5）给 HitTestNodeSnapshot 加 `drag_barrier`/`draggable` 字段未同步 `apps/browser/src/tests.rs:5598/5610` 字面初始化（E0063）——本轮未触碰该结构，按准则不顺手修，留 dnd 域跟进。
- corpus 16,590 A/B 同 binary：基线 1,074 / ON **1,065**（净 −9：corner-shape 12→3，翻绿恰 9 / 翻红 0，逐项 diff 全部限于 corner-shape 案）。
- make reftest **712/712** EXIT=0；product-smoke **EXIT=0** 全 struct PASS（layout/paint 变更必跑）。
- bench-gate 未跑：本轮改动面为 corner-shape 激活路径（此前 None 回退不走多边形构建），非热路径；paint/bench 基线族无 corner-shape 案。

## 挂账（新增/延续）

- **square 0.03%（168px）= 常规 border-radius 弧带缺失**（R5180 首选）：`paint_borders` plain round 分支补外弧+内弧 ring（可复用 shaped ring 成对多边形代码）；TR superellipse(24) vs ref 直角另有 ~1px 内缩斜边小残差。
- **overflow-clip-margin 0.30%（1,434px>64）**：ref 结构 = 外层 bg 形角裁剪 + box-shadow spread 填充，test = 内层 round shadow 被 shaped clip——box-shadow 形状语义（css-backgrounds-3 #shadow-shape × corner-shape 联动）与 clip/ref 两路径自洽性独立域核。
- **iframe-border 5.33%**：shaped ring 前提 `background != Transparent`（内圈须遮孔），iframe 透明背景需环带原语或 shaped band 路径。
- **scoop 真描边弧 FIXME**（border.rs）：真弧心在外盒角、半径 r+bw；全半径内弧偏差 ≤~1px，像素级对齐时再引入逐角真描边。
- 延续：outline-028、transforms 余 44、filter-effects 20、cascade 16、backdrop-root 三层联动、preserve-3d、rotateY reveal、dirty-rect 相位、body-background 传播、IFC 宽轴 min 臂、R5172 挂账组。
