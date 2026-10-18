# R5173 证据：rotateX 轴向收缩投影走直绘 quad（css-transform-3d-rotateX 族翻绿，corpus 净 −5 零新红）

日期：2026-10-18。基线：R5172 tip（1fc933979）。kill-switch：`ZW_TRANSFORM_3D_AXIS_DIRECT=0` 回退 TP 路径（LazyLock 进程级快照，R3858 教训；判别经外部 env corpus OFF 轮验证）。

## 域核

承 R5172 下一步 ①（0.5-2% 带续扫择轻）。候选域 transforms 48 / filter-effects 20 / cascade 16；2.08% 恒值簇不作单因修（R5151-R5165 判例）。择轻落 transforms 域 **css-transform-3d-rotateX-positive / -negative / rotate3d-X-positive / -negative 对案**（各 1.11%，4 案同构：120×240 绿块 `rotateX(±60deg)` 投影成方块盖红块）。

**定谳**（REFTEST_DUMP PNG 取证）：绿块投影几何**正确**（R3833 cos 2D 投影生效）——差异是下层红块 `border: 20px solid black` 的上下横带缺失（左右竖带完好）：缺失带恰 = 红块 border 在绿块**未旋转布局矩形**（x 80..200, y 20..260）内的部分。根因：CPU raster `apply_transform_post`（render-foundation cpu/mod.rs）是「保存 rect 快照 → rect 清白 → 逆采样重放」的全场景像素后处理——收缩投影（scaleY 0.5）后露出区域只能从清白恢复，而下层内容在绿块绘制时已被覆盖、快照内只有绿块自身像素，无下层可还原。R5085 直绘 quad 路径（单实色 fill 四角映射 path_fill）即为此而设，其注释已点名 rotate3d-X 族同谱，但 `pure_2d_transform` 门排除全部 3D 函数。

## 实现（painter/mod.rs 门扩展，+约 30 行）

`axis_exact_3d` 臂并入 R5085 `solid_leaf_fill` 门：**rotateX + X 单位轴 rotate3d**（恰一分量非零）入直绘——矩阵构建器产出精确轴对齐 scale `(1,0,0,cos)`（`rotate3d_unit_axis` 归一），四角映射 = 精确几何与 2D 集合同构；限定**无子盒**（与 R5085 叶实心盒形态风险面隔离）。其余 3D 函数（rotateY/混合轴/perspective/scale3d/translate3d）维持 TP 路径。

**两轮 corpus 迭代（配对拆对面定谳）**：

1. 首轮含 rotateY → corpus 翻红 2（backdrop-filter-3d-transform-perspective / -nested 同族）：配对 test 侧 `.card`（backdrop-filter + rotateY(45deg)）被 backdrop-filter 门锁死 TP，ref 侧**同几何无子盒实心盒**走直绘——45° 非整数 texel 相位（cos45≈0.7071，TP 双线性插值 vs 直绘 AA）单侧改路拆对。rotateY 整轴退回 TP。
2. 中间轮加无子盒门试图隔离该配对——无效（配对两侧 `.card` 均无子盒），门保留作形态隔离，rotateX-only 定版。
3. rotateX 域零拆对实证：60°（cos=0.5，TP 采样恰整数 texel 退化精确拷贝）与 180°（镜像，|d|=1）相位无关，corpus 全域翻绿无拆对。

## 验证

### 回归测试（crates/engine/src/paint/tests/r5173_transform_3d_axis_direct.rs，5 案）

- `test_r5173_rotate_x_solid_leaf_direct_quad`：rotateX(60deg) 直绘四角 = (80,80)-(200,200)（y' = 140+(y−140)×0.5）、零 TP。
- `test_r5173_rotate3d_x_axis_direct_quad`：X 单位轴 rotate3d 同直绘路径。
- `test_r5173_rotate_y_keeps_tp_path`：rotateY 维持 TP（配对拆对面，对照锁定）。
- `test_r5173_with_child_box_keeps_tp_path`：有子盒（零可见图量子盒）维持 TP。
- `test_r5173_mixed_axis_rotate3d_keeps_tp_path`：混合轴恒等近似零发射对照（compute 侧 is_identity 早退既有语义）。

### corpus ON/OFF A/B（16,590 案，同 binary）

| 轮 | fail | 备注 |
|---|---|---|
| ON（默认） | **1,099** | **终态**（基线 R5172-ON 1,104，净 −5） |
| OFF（`ZW_TRANSFORM_3D_AXIS_DIRECT=0`） | 1,104 | 恰等基线 |

- ON ↔ 基线 fail 清单 diff（前缀归一化）：**翻绿恰 5**（css-transform-3d-rotateX-negative/positive、rotate3d-X-negative/positive、transformed-rotateX-3——180° 镜像顺带）、**翻红 0**。

### 门禁

- fmt / clippy（-p zero-engine --all-targets -D warnings）：清。
- zero-engine lib：**3,119 绿**（3,114 + 5）。
- `make reftest`：**712/712**。
- `make test`：workspace 全腿绿。engine lib 3,119P、renderer --bin 272P / --lib 232P、gpu 腿 96P；browser --bin 腿预存环境红 1 案（`paste_and_go_blurs_and_clears_preedit` 前置「测试环境应可写系统剪贴板」panic，649P+1F+1I）隔离复跑 **650P+1I 全绿**——recipe 缺 `ZW_GUARD_RUNNER_PREFIX` 既有挂账延续（R5168/R5170 判例同款），零归因；browser --tests 腿（cdp 集成三件 6P/4P/9P）裸跑同 2 环境红（剪贴板 + `window_surface_present_smoke` 无 display，R385 记录在案），xvfb 前缀复跑 **6P+4P+9P+650P+1I+10P 全绿** EXIT=0；quickjs 三腿绿（crates 768P+59I 等 7 包、renderer --bin 701P+240P+155P+18P、browser 270P）。
- product-smoke：welcome oracle diff **17.07%** 同值 + 全 struct PASS + EXIT=0。
- bench-gate（定向 `ZERO_WEB_BENCH_CRATES=zero-engine`，filter 生效仅 engine_bench）：**GATE PASS**——微基准 1/1，perf-gate 26 指标全 PASS（NEW=0）含 paint_complex_page_500_elements 426µs 优于基线 487µs、end_to_end 325ms < 预算 360ms，EXIT=0 零 FAIL 无需判读。

## 挂账与下一步

1. rotateY 域 reveal 缺陷（transform-3d-rotateY-stair-above-001、transforms-rotateY-degree-60 维持基线红）：45° 类角度存在配对拆对面，须先有路径无关性方案（TP 侧整数 texel 判定或 backdrop 页配对感知），案驱动再探。
2. perspective-origin ×6（各 1.56%）：`perspective` 属性 + perspective-origin 域（矩阵构建器现恒等近似），独立切片。
3. apply_transform_post 露出区域下层恢复（组起点快照 + backdrop 复合）：根修方案，惠及全部收缩变换域，需组边界元数据穿线，独立轮评估。
4. R5172 挂账延续：vertical 域表 rtl 列序（无案驱动）、空/纯 inline caption 的 css-tables-3 intrinsic 语义、caption min-content 下限钳、R5171 挂账组（inline 扩张轴锚定/跨 white-space 折叠/纯文本 run 匿名 cell 合成/vertical 表匿名 cell）。
5. (A) row-2 / multicol-2 / 动画 RFC（用户门）。
