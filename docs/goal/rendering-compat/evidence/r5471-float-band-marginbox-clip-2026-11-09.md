# R5471 — own_floats 排除带 margin-box 判弃/裁顶口径（c5525-fltmrgn ✓0.28%→✓0.19%）＋ r5446 提交态红勘误

日期：2026-11-09
范围：`crates/layout-engine/src/inline_finalization.rs`（compute_final_inline_layouts own_floats 构造）

## 1. 根因链

R5469/R5470 之后 c5525-fltmrgn-000 残余 ✓0.28% 中，test4 形态（img float `margin-top:−30px`）：

- float 带的 border 顶 p-rel y=−30，margin-box 带 [0,15]（mt −30 抵消 30px 后可见段 15px）；
- 旧 own_floats 构造按 border 顶 `rel_y < 0` **整带丢弃** → stored IFC float-unaware（首行 x=0，应 45）；
- paint use_stored 重放按 stored IFC 排墨 → float 图元右侧 45×15 红标记露出。

即 stored IFC 与步 6 remeasure（R5442 同构臂，margin-box 口径）两路径口径劈叉：布局趟算对了，存储趟把带丢了。

## 2. 修法（CSS2 §9.5 float margin box）

`compute_final_inline_layouts` own_floats 构造改口径（默认开）：

- **判弃**：按 margin-box 带底——`band_bottom <= 0`（整带在容器顶上方，不约束任何行）才丢；
- **跨顶带裁顶**：`clipped_top = max(band_top, 0)`，保留可见段 `[0, band_bottom)`——IFC 行盒 y ≥ 0，裁顶对既有 rel_y ≥ 0 带的行排不可见；
- 判弃与裁顶对正带（band_top ≥ 0）输出与旧口径逐位相同（`band_top/带高` 原值）。

kill-switch：`ZW_FLOAT_BAND_MARGINBOX=0` 回退旧口径（判弃 border 顶、不裁顶）。flag 反证由 corpus OFF 轮承担（进程内 set_var 不可靠，R3858 教训）。

## 3. r5446 提交态红勘误（非本轮回归，随本轮修复归位）

`r5446_nested_wrapper_float_at_left_edge_and_contained` / `r5446_rtl_wrapper_float_right_at_edge` 两案在 R5470 提交态（ef44292df）即红（`git show HEAD:` 回退复跑实证 2F）：旧断言「p 高 ≥50 容纳 float 底」是 nested float_bottom 折返时代的巧合固化，R5470 §10.6.3 高度门翻默认后巧合断——普通块 auto 高不含 float extent（chromium oracle 真值 p_h=18）。

勘误断言为「p 高不被 float 撑高（<50）」。实测复核（反转断言探针）：001b 形 ZW p_h=36（2 行）vs chromium 18（1 行）——行排差异（Kitty 换行域）记 FIXME 另案（行盒归属缺陷在册案邻域）；RTL 形 ZW p_h=18 与 chromium 逐位一致。

## 4. corpus A/B（同二进制 16599，release runner）

- 全量：**ON 688F / OFF 689F**。
- 真修复：c5525-fltmrgn-000 **✓0.28%→✓0.19%**（ON solo ×3 恒 0.19%，OFF solo ×3 恒 0.28%——OFF 与 R5470 基线同值，确定性成立）。
- movers 3 案（intrinsic-percent-replaced-dynamic-011、border-collapse-dynamic-column-001、table-anonymous-objects-152）：双窗 solo ×3 全 ✓0.00% 恒＝批跑噪声指纹家族扩员 +2（tao-152 R5467 已在册）。**真回归 0**。

## 5. 验证矩阵

- 常驻 judgment test＝`engine/tests/r5471_float_band_marginbox_tests.rs` 3 案：跨顶带裁顶避让（frag.x=45）＋整带上方不约束（frag.x=0）＋正带对照（新旧同 45）；kill-switch 反证由 corpus OFF 轮承担。
- 内测 `make reftest` **725/725**（Layout 521 + Text 204）。
- layout-engine 全腿（test-guard compile-first）：**2543P/0F/2I**（lib 2453 含新钉 3 案＋集成＋doc-tests）。
- fmt clean；clippy `-p zero-layout-engine --all-targets -D warnings` 清。
- `make product-smoke` 全腿 PASS：welcome **17.23%**（与 R5465-R5470 六轮同值零漂移）＋ struct-check 全 PASS（含 375/320 窄屏）。
- bench-gate：定向 zero-layout-engine **GATE PASS**（25 gate 指标全部在预算内，NEW=0；静窗 1min=10.2 通过）。

## 6. 挂账更新

- **销账**：R5470 下一步①「c5525 残余 ✓0.28% paint 域择优」之 test4 img 右缘红标记（p4-band，stored IFC 带丢失主体）；R5446 两案提交态红（勘误归位）。
- **维持**：c5525-fltmrgn 残余 ✓0.19%（R5469 §2 记录的 test6 墨 z 序 + test4/blob3 bg 绘制原点域维持 paint 域在册；下轮在新基线重取 diff 带最小页二分）；**行盒归属缺陷**（display:inline 中间层 lines=0，floats-149 域）；fsa 度量单 owner 切片；缺陷 4 viv 行内 fill；R5467 缺陷 2；缺陷 3＝待用户决策⑥；噪声家族扩员 +2；及控制面既有长账全维持。
- **下一步 R5472**：①c5525 残余 ✓0.19% paint 域择优——test6 图元发射序查证或 test4/blob3 新基线 diff 带最小页二分 ②行盒归属缺陷归因或 fsa 单 owner 切片择优 ③重腿静窗纪律维持（movers solo×3 先采样定谳）。
