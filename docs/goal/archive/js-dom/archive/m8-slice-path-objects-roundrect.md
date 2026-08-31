# M8 切片 R56 — path-objects 接手首切片：用例导入 + roundRect panic 修复 + 语义/几何对齐

**日期**: 2026-08-15
**轮次**: R56（R55 文档收尾同轮完成后的正片）
**目标**: docs/goal/js-dom.md DC-8 / M8（canvas path-objects JS 侧 API 语义，v1.2 接手）

## 接手第一动作：用例导入（DC-8 首条）

canvas 流的 `fetch-canvas-subset.sh` 已把 `path-objects` 列入 SUBDIRS + `CANVAS_TEST_SUBDIRS`
（`9cb32ab9`），但本地 wpt-data 目录为空（fetch 脚本整跑 5min 超时于 GitHub Contents API
逐目录列举）。改为：单次 API 列目录 → 逐文件 raw 拉取（pinned `WPT_REV=315976933870b`），
205/205 `.html` 全部就位（含 roundrect 56 个）。

## roundRect 批量 panic（接手第一优先级，DC-8 第二条）

首跑 `testharness-canvas path-objects`（test-guard 包裹）即 abort：

```
panicked at library/core/src/slice/sort/shared/smallsort.rs:854:
user-provided comparison function does not correctly implement a total order
```

backtrace 栈：`blit_path_to_pixels` → `fill` → `canvas_context_op`。根因：4 处
`sort_by(|a,b| a.partial_cmp(b).unwrap_or(Equal))` 在数组含 NaN 时违反全序传递性
（NaN 与任何值比较 Equal，但 1<2），driftsort 检测后 panic。roundRect 非有限 radii
产生 NaN 路径顶点 → 扫描线交点 NaN。修复：全部改 `f32::total_cmp`。

**并行流叠加**：同轮并行 canvas 流 land `afd8ec08` 做了同款 total_cmp 修复（5 处，
多覆盖 path.rs 一处）+ arc 角度归一化 + arc 负半径校验。rebase 冲突仅 2 处注释级
（代码行相同），取并行流版。main 终态 = 双流叠加。

## roundRect 语义（shim `zwNormRadius`）

spec `dom-context-2d-roundrect` 对齐：

- 序列空或 >4 项 → RangeError（`radius.none` / `radius.toomany`）
- 任一半径负 → RangeError（`radius.negative`，含 DOMPoint/DOMPointInit 形式）
- NaN → 0 不抛（spec：ToNumber NaN 按 0 处理）
- BigInt → TypeError（WebIDL unrestricted double 不收；unary `+0n` 原生抛，`badinput`）
- **单个 DOMPointInit**（DOMPoint / `{x,y}` 字典）→ `p<x>,<y>` 角对编码（旧版落
  `r='0'` 半径全丢——`1.radius.dompoint*.single.argument` 直接驱动）

## host 角对解析 bug（probe 链定位）

`p40,20` 由 JS `parts.join(',')` 产生，host `split(',')` 拆成相邻两项 `'p40'`+`'20'`。
旧代码 `split_once(',')` 在 `'p40'` 上失败 → `unwrap_or((pair, pair))` →
DOMPoint(40,20) 被解成 **(40,40)**。定位路径：像素扫描 y=1..20 边界曲线 → 椭圆拟合
(rx,ry)≈(25,12.5) = 0.625×(40,20) → scale 推导发现 (40,40) 被 h/2 clamp。修复：
`'p<x>'` 项与**下一项**配对 `(x, y)`。

## 负 w/h 归一化 + 角序镜像（`2d.path.roundrect.negative/winding`）

spec 角序 `[tl,tr,br,bl]` 相对**参数坐标系**（tl 恒贴 `(x,y)` 参数角），负 w/h 翻转
边走向即镜像矩形，角随边镜像到对侧。实现：归一到包围盒 `(x+w.min(0), y+h.min(0))`
后对展开的 4 角半径做 swap（垂直 0↔3/1↔2、水平 0↔1/3↔2、双向 0↔2/1↔3）。
**关键细节**：swap 必须在 1-4 项 radii 展开之后做（`[a,b]` 展开序 tl,tr,br,bl=a,b,a,b，
先换位再展开会错角——单测捕获）。

## 扫描线段式迭代（多子路径虚假边）

`2d.path.roundrect.negative` 逐步 probe 实证：2 个 roundRect 同 fill 正常、加第 3 个
**几何上不接触该扫描线**的子路径后 (25,12) 翻色。根因：三个 fill 光栅消费端
（`blit_path_to_pixels` / `blit_path_gradient` / `rasterize_path_coverage`）把 flatten
输出的**段对序列**当作**多边形顶点链**处理（`points[i]↔points[i+1]` + `% len` 环绕）
——多子路径时产生跨子路径虚假连接边，扫描线交点数变奇数 → 配对错位 → 整片翻色。
修复：按独立段 `chunks_exact(4)` 迭代。

连带修复（同根因暴露）：

- **closepath-on-fill**：段式迭代丢掉旧链式环绕的隐式闭合 → 开放子路径
  （`M+L+L` 无 close 的 fill）填不出。`flatten_path_opts(close_open_subpaths)` 拆分：
  fill/clip/isPointInPath 闭合（MoveTo 边界 + 命令流末尾两处补段，判定
  `dx>EPS || dy>EPS`——初版 `&&` 笔误致纯垂直/水平位移闭合失效，probe 定位）；
  stroke/isPointInStroke/stroke_outline 走 `flatten_path_open` 保持开放。
- **ctx.rect() 补 closePath**（bridge 层；spec：rect 子路径闭合）。
- **roundRect 子路径起点基准**：隐式闭合须以 roundRect 自身边界为准（负 w/h 归一化后
  起点 ≠ 外部 MoveTo 参数点，隐式闭合会补出斜穿矩形的虚假边）。
- **join 三角/四边形闭合段扇**：bevel/miter 顶点链 8 floats 在段式消费下缺 a→b 边。
- **退化矩形分支自包含**：删 `current→corner0` 连接段（current 恰等于 corner3 时与
  闭合边重复 → 奇数交点）。

## 凸凹角门控试验（负结果记录）

为修 `line-styles/2d.line.join.bevel`（(84,16) 凹角外不得涂）试验了叉积凸凹门控——
被双例证伪（^ 形与右下拐同号旋转、Chrome 两种处理；实际失败真因是绿三角覆盖时序，
由 closepath-on-fill 的 `&&`→`||` 笔误修复连带解决）。门控已回退，`join_visible`
保持 bool 语义。

## 验证

| 项 | 结果 |
|----|------|
| zero-canvas 单测 | 777→783 全绿（+3 新单测：neg-h bbox/mirror/zero-radius；计数断言语义更新 11 处；诊断测试已删） |
| path-objects WPT | **139→156 pass / 64→47 fail**，panic 消除全程跑完（双流叠加后） |
| 全 canvas WPT | 449 pass / 53 fail（shadows 6 个 HEAD 既存，clean-tree 验证非本切片） |
| fmt / clippy | 无 diff / v8+quickjs 双矩阵零警告 |
| make test | 唯一失败 `default_actions_work_without_javascript` 为并行流既存（clean HEAD 同败，表单导航域） |

**5 个新 fail**（换 12 修复 + panic 消除的边缘代价）：arc.shape.5 / arcTo.curve2
（arc 精度边缘）、fill.overlap / winding.add（需 nonzero fill rule——纯偶奇光栅嵌套
同向矩形挖洞，独立切片）、rect.end.2（closePath+lineTo 组合，同族 end.1 已过）。

## 剩余（下轮 ROI）

1. nonzero fill rule（fill.overlap/winding 族）
2. arc/arcTo 形状精度（~14 fail）
3. stroke cap/join 边缘（closed / end.3 / prune 族）
4. isPointIn* scaleddashes
