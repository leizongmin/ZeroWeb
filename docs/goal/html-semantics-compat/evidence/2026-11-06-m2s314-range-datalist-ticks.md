# M2 s314 — `<input type=range>` datalist tick 刻度：list 语义 + step 对齐 + rtl 镜像（the-input-element 簇 3 红 → 1 红跨流移交，reftest 25 案 22P → 24P）

日期：2026-11-06。切片号 s314（承 s313 下一步候选①小簇排队，勘根→施工型切片）。

## 勘根/边界确认

- **簇红页形态**：the-input-element 25 reftest 案 = 3 红：range-tick-marks-01/03
  （mismatch 型 ✗0.00%）+ range-intrinsic-size（match 型 ✗3.52%）；22 绿中含
  tick-marks-02/04/05（match 型 ✓0.00%）+ repaint 八页（match 型 ✓0.00%）。
- **tick 五页语义矩阵**（全为 `<input type=range>` + `<datalist>`）：
  - tick-01（mismatch）：test 带 `list` vs notref 不带——tick 存在性可分；
  - tick-03（mismatch）：min/max 不同（-40..40 vs -100..100）→ tick 像素位置
    按 fraction=(value-min)/(max-min) 映射可分；
  - tick-04（match）：disabled option 不产生刻度（两侧 tick 集同为 {-30,50}）；
  - tick-05（match）：step 不对齐 value 不产生刻度（step=3 min=-5，{-4,-2,0,2,4}
    仅 {-2,4} 对齐，两侧同集）；
  - tick-02（match）：test `dir="rtl"` vs ref `transform: scaleX(-1)`——tick 须随
    direction 镜像才与翻转后像素域一致。
- **repaint 八页回归面核对**（range-list-added/change/duplicate-id/nonexistent/
  option-add/option-remove/option-value-change-repaint + setattribute-value）：全
  match 型且终态收敛到 range-tick-marks-05-ref（{-2,4} step 对齐）——该族本就是
  为 tick 渲染设计的动态页（JS 改 list/id/option 后 reftest-wait 截图），要求
  list→datalist 逐 paint 解析而非静态快照；duplicate-id 页依赖 id 属性 rename 后
  id_map 重挂载（现 HashMap 插入序实现：rename 后入表者胜 = 新表 {4,-2}，与
  chromium 树序首个裁决同果）。setattribute-value 页两侧均无 list → 零刻度不扰动。
- **intrinsic-size 归属**：match 型 ✗3.52%，全页 30+ 行 flex/grid（width:0 容器、
  min-width:0、justify-items）内 range 固有尺寸交互——纯 layout-engine 域
  （flex/grid 固有尺寸 + min-width:auto 语义），paint 侧无从下手；UA range 固有
  宽（现落 R1659 `_` 文本臂 178px，chromium ~129px）为其中的 style-system 子项。
  **移交渲染流域**（s308/s309 归属勘根先例），本流零施工。
- **现势**：`paint_input_value` 对 range 早退（无 value 文本）；range 落 R1659
  UA `_` 臂（白底 + 2px #767676 边框 + 178×24 文本类估宽）——无任何滑轨/tick
  绘制面；knowledge 库无 range/datalist 绘制面记载（既有条目为 DOM 表单状态域）。

## 施工（crates/engine/src/paint/painter/{controls.rs,mod.rs}，engine 单侧）

1. controls.rs 新 `paint_range_ticks`（R1687）：input[type=range] 门 + appearance
   None 门（≡ s313）→ `list` 属性 → `Document::get_element_by_id` 逐 paint 解析
   datalist → 收集 option（disabled 排除；语义值 = value 属性否则文本内容；非法
   数值跳过）→ step 对齐过滤（(v-min)/step 取整距 ≤1e-6 容差）→ fraction 截
   [0,1] → `direction: rtl` 镜像（dir 属性经 presentational hint 映射 direction，
   style-system 既有臂）→ 2px 竖线绘于 content-box 下半带（RANGE_TICK 灰）。
2. mod.rs 接线：`paint_element_specials_before_text` 4b5 位（paint_select_value
   之后）——主走查与脏矩形走查共享单点，双路径自动覆盖。
3. 单测 3 面 + pipeline 1 面：step 对齐仅 {-2,4}（位置/几何断言）；disabled 排除
   + rtl 镜像（x=71 断言）；无 list/不存在 id 零刻度；pipeline 全链 tick-01 形态
   （with = without + 4 fills）。

## 语义边界（记录）

- thumb 滑块、轨道外观、value 位置镜像不实现（corpus 五页+八页仅需 tick 面）；
  range 原生外观仍走通用素盒（UA 白底边框），与 chromium 滑轨视觉非逐像素等价
  （self-consistent match 域不受影响；DC-14 strict 分类下为近似通过面）。
- step 解析不含 `any` 特例（语料无消费，FIXME 不标——超出本切片驱动的场景不设
  占位）；id_map 树序首个语义与 target.rs 注释「树序首个」的措辞出入（HashMap
  插入序实为后写者胜）为 dom crate 既有行为，本切片零触碰、语料同果，只提及。
- perf-gate 豁免：改动为 input[type=range]+list 属性专属路径（门序前置 return，
  非门元素零新增谓词进入公共路径），无 bench 面；承 s311/s312/s313 先例。

## 验证

- 簇 solo reftest：**25 案 22P/3F → 24P/1F**——tick-01 ✗0.00% → **✓0.03%**、
  tick-03 ✗0.00% → **✓0.03%**（mismatch 阈超过 0.01%）；tick-02/04/05 ✓0.00% 保绿
  （镜像/排除/过滤三语义实测同域）；repaint 八页全数保绿（动态解析 + id rename
  实测一致）；intrinsic-size ✗3.52% 维持（跨流移交面，零扰动）。日志
  [logs/s314-input-after.log](logs/s314-input-after.log)（修前基线
  logs/s314-input-initial.log：22P/3F）。
- 单测：paint_range 3 案 + test_s314_range 1 案全绿；s313 meter 门回归绿。
- clippy：`-p zero-engine --lib -- -D warnings` 零告警（首版 doc 注释行首
  `>0.01%` 被 rustdoc 误判 blockquote，改写「超过 0.01%」）。fmt：本切片文件
  rustfmt 后零漂移；另解除上游 f021e11fb 于 wpt-runner/testharness.rs 的 1 处
  格式漂移（T58 fmt 门禁前置阻塞全仓 make test，纯格式闭包内联零语义，跨流
  信号记档）。
- 产品 smoke（UA/表单面）：make product-smoke-legacy 42 fixture **struct-check
  failures: 0**；make product-smoke **PASS**。日志
  [logs/s314-legacy-smoke.log](logs/s314-legacy-smoke.log) /
  [logs/s314-smoke.log](logs/s314-smoke.log)。
- engine lib（test-guard 全量）：**3781P/8F**（总 3789 = s313 基线 3785 + 本切片
  4 新案全绿）——8F = 已知 7 键（CSSOM cssText 5 + 跨流 websocket 2）+ **跨流新键**
  `test_fetch_filtered_response_type_matrix`（上游 f021e11fb fetch M3 redirect 引入；
  stash 探针干净树同败实锤归因，与本切片零关联）。日志
  [logs/s314-engine-lib.log](logs/s314-engine-lib.log)。
- make test（guarded-test，后台全量）：**80 腿 ok + engine lib 3781P/8F**（8F 同上
  已知+跨流集），guarded-test 退出 101（唯一失败腿即 engine lib），其余 80 腿零回归。
  日志 [logs/s314-make-test.log](logs/s314-make-test.log)。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：
- s314-input-initial.log（修前 22P/3F 基线）
- s314-input-after.log（24P/1F 终态）
- s314-engine-lib.log / s314-legacy-smoke.log / s314-smoke.log（回归门禁）

代码锚：crates/engine/src/paint/painter/controls.rs（paint_range_ticks +
RANGE_TICK）；crates/engine/src/paint/painter/mod.rs（4b5 接线单点）；
crates/engine/src/paint/tests/content_objectfit.rs（3 新案）；crates/engine/src/
tests/pipeline.rs（s314 1 案）。

注记：本切片 diff 全落 engine 4 文件；style-system/layout-engine 零触碰。

## 下一步候选

1. 小簇排队其余（link 3/embed 3/object 2/option 1/iframe 1/wbr 1）逐簇勘根；
   br 1 勘根前先 solo 复核 editing-compat br 刀（fe760dc57）外溢现势。
2. range-intrinsic-size 跨流移交跟踪（渲染流域 flex/grid 固有尺寸 workfront；
   UA range 固有宽 178→129px 为 style-system 子项，须与 layout 语义协同落）。
3. （挂账承前）shaped 值行 ellipsis 接线；生产动画重绘驱动等跨流协同项。
