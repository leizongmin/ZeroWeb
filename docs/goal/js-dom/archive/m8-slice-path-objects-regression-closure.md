# M8 切片归档：path-objects 回归收口（R56b，2026-08-15）

**commits**: `ac7240f5`（R56 首切片：roundRect 语义 + 段式扫描线）、`b1446e9b`（R56b 回归收口）
**基线**: pre-R56 `a749de94` = 62 Fail → R56b 后 37 Fail / 166 Pass（净 +25 修复，零回归）

## 背景

R56 首切片（roundRect radii spec 语义 + host p 配对 + 负 w/h 归一化 + 段式扫描线迭代 + stroke closed 末命令判定）land 后， rollover session 以「真基线（pre-R56 commit 构建）重跑全量 path-objects」核验，发现 4 个真回归（基线快照 /tmp/fails.txt 与干净 HEAD 不一致曾致误判——本轮教训：**基线必须来自干净 checkout 构建，不可信工作树旁的快照文件**）。

## 四个回归根因与修复（b1446e9b）

1. **arc span 归一化方向感知**（2d.path.arc.angle.4/5、twopie.5 Fail + 修复过程中的 angle.2/3、shape.2 二次回归）
   - 根因 1：`raw_span % TAU` 对顺时针（acw=false）负差得负 span → 弧走向反向（angle.5 扇形翻侧）。修：同向 mod——顺时针 span ∈ [0,2π)、逆时针 ∈ (−2π,0]。
   - 根因 2：归一化结果已含方向，旧 `angle_span = span * dir` 双重取反再次翻弧（angle.2/3、shape.2）。修：去掉 `* dir`（`let _ = dir` 保占位）。
   - 位置：`crates/canvas/src/context/raster.rs` flatten_path_opts Arc 分支 + `crates/canvas/src/path.rs` 同款。
2. **flatten Arc 缺「current→弧首」连线段**（angle.4 的 (98,48) 缺口）
   - spec dom-context-2d-arc：「If the context has any subpaths, add a straight line from the current point to the start point of the arc」。moveTo(圆心)+arc(整圆) 的 fill 扇形缺这条边，段式扫描线在弧首角配对破裂。
   - 修：`has_any_subpath` 标志（MoveTo 置位）判存子路径，arc 首命令不连（无 (0,0)→弧首 虚假段）。
3. **roundRect 非有限半径抛 RangeError**（roundrect.nonfinite）
   - spec：任一半径 NaN/±Infinity → **静默忽略整次调用**（与 x/y/w/h 非有限同款）。修：`zwNormRadius` 非有限返 null，三调用点（标量/序列/单字典）null → return。
4. **DOMPoint 构造器吞 NaN**（roundrect.nonfinite 的 `new DOMPoint(10, NaN)` 调用组，二分定位 #70）
   - 根因：`this.y = +y || 0` 把 NaN（falsy）吞成 0 → DOMPoint(10,NaN) 变合法半径 (10,0)。spec DOMPointInit 成员是 unrestricted double，NaN/Inf 必须保留。
   - 修：`(y == null) ? 0 : +y`（只对 null/undefined 缺省）。副带修复 Inf 保留。

## stroke closed 语义修复（R56 首切片本轮验证发现的 rect.end.2 回归）

- 根因：`rect()` op 补 `close_path()` 后，stroke 的 `closed = commands.iter().any(ClosePath)` 把「rect 闭合 + 后接 lineTo 的开放末子路径」误判闭合 → 丢 line cap（lineCap=round 半径 225 圆盘覆盖全画布的用例四角红）。
- 修：closed = **末命令**是否 ClosePath（stroke + stroke_path 两处）。

## 验证

- path-objects 全量：pre-R56 62F → 37F（+25 修复，comm 对比零回归）
- 跨目录：line-styles 1 预存 Fail（2d.line.join.round，d3edff50 同 Fail）；drawing-rectangles/reset/transformations 零 Fail
- 单测：canvas +5（span 方向×2、连线段×2、2×2 fill 哨兵）+ 4 旧 arc 计数更新；engine shim +1
- engine v8 2149 / quickjs 1416 全绿（含并行流 83fa7fbd 修复的 canvas_element_bridges_to_image_primitive 预存红灯——布局 flow-root fold 把 canvas 固有高度 2 压成 0，渲染流域）
- clippy 双矩阵零警告，fmt 无 diff

## 教训（过程记录）

1. **stash/pop 链条中输出被 tail 截断会掩盖 pop 失败**——本轮一次 stash pop 未执行致工作树「消失」（实际被并行流 commit 成 6f5ee7d1，reflog 找回零丢失）。长链命令拆开跑，pop 结果单独确认。
2. **基线快照必须来自干净 checkout 的全新构建**——工作树旁 /tmp 快照可能是中途态（本轮 fails.txt 与真基线差 17 条，致 arc.shape.5 等四个「预存」误判，真回归差点漏网）。
3. **同 clone 双流再发生**（run-rules §8 第三次）：并行流把本 session 工作树 commit 成 6f5ee7d1 → rebase → ac7240f5。R51 轮已记录；本轮无损失，但 stash 链条与并行 commit 交织是实际风险源。
