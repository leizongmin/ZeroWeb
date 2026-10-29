# R5336 证据 — block-in-inline 切片段竖向框块轴语义（CSS2 §10.8.1）

日期：2026-10-30 · 轮次：R5336 · 基线：R5335 交付态 798F @ `ee26d526a` · 代码 commit：`7616aecf4`

## 0. 结论速览

- insert 家族剩余挂案（010 1.11% 等 5+ 案）的主共性定性为**三层竖框膨胀**：R109 切片段中被提升为 taffy 块盒的 inline 盒（匿名块片段、split 壳、切片链 inline 祖先）按块级盒模型把竖向 border/padding 计入块轴；CSS2 §10.8.1 inline 级语义是竖框**只绘制、溢出行盒、不撑高**。
- taffy 层修复后 insert-010 页面几何精确对齐 chromium（html 高 154→142、body 784×126、7 个 slice 全 18px strut 行）。
- corpus A/B **净零**（15792/16590 = 95.2%，798F，fail set 与基线逐项 IDENTICAL）：paint 侧切片带锚定不消费 taffy 几何（R5335 collect + inline_finalization sync 臂 + R1442 带绘制三处复合记账），R5337 重锚。

## 1. chromium oracle 探针法

本地 headless chromium 截图对照（`google-chrome --headless --screenshot`），先以 **test == ref 逐像素一致**校准 oracle 基线，再对 ZW 有歧义的语义做对照实验。本轮产物留存 `/tmp/r5336-oracle/`（insert-010 test/ref 截图）与 `/tmp/r5336-probe2/{p1,p2,p4}.html/png`。

### 1.1 insert-010 oracle 数据（逐行 census）

| 项 | chromium 实测 |
|---|---|
| 页面 test vs ref | 0 像素差（oracle 基线可信） |
| body content box | 784×126（html 高 = 126 + 16 body margin = 142） |
| slice1（首个匿名块）content | y 8–26 → **18px 高**（非 24px：竖框 3+3 不进块轴） |
| 外层蓝 border 上横带 | y 5–7，可见 x 8–10（其余被内层 cyan 带后画覆盖） |
| 内层 cyan border 上横带 | y 5–7，x 11–105（shrink-to-content 宽度） |

### 1.2 探针页（P1/P2/P4）——chromium 切片绘制模型

- **P1** `span > div`（span 四边 3px 蓝 border）：只渲染**竖带**——空片段（element-only strut）的片段盒宽 ≈0，左右竖带重合成 3px 竖条，上下横带零宽不可见。证明片段盒 = **shrink-to-content 宽度**，非满宽。
- **P2** `span > [x, div, y]`：四边带齐全且横带宽度 = 内容宽（shrink），不是 body 满宽。
- **P4** 嵌套 `span(blue) > span(cyan) > div`：外层蓝横带与内层 cyan 横带**同排**（y 5–7），child 后画 → 可见部分只剩内层带外侧缝隙（x 8–10）——overpaint 层序 = 外层先、内层后。
- 横带锚定：贴行盒顶外侧（y5 而非 y8），即**带锚 = 行顶 − border_top**，溢出行盒不占布局。

## 2. ZW 三层膨胀定性（LAYOUT_DUMP 树证据）

修复前 insert-010 树 dump（对照 chromium）：

| 盒 | ZW 修复前 | chromium |
|---|---|---|
| html | 高 **154** | 142 |
| body | 784×**142** | 784×126 |
| 28v1（split 壳 taffy 盒） | (0,0)，竖框 3+3 占位 | 竖框 0（只绘制） |
| 29v1（首个 anon 片段块） | (3,0) 778×**24**（3+18+3） | 18px strut 行 |
| 7 个 slice | 含竖框累计 | 全 18px |

三层来源：①anon 片段块继承被拆 inline 的盒模型 → border 3+3 进 taffy 块轴 → 24px；②split 壳自身竖框占位；③切片链上 inline 祖先盒（taffy 子含 split_parent）竖框占位。3 slice 累计 **+18px** 页高。

**控制实验**（排除通用路径）：plain/inline 对照页（无块级子的纯 inline 竖框页）ZW 与 chromium 块几何一致——通用 inline 路径已正确（`inline_types.rs:192` box_height = line_height），膨胀限于 R109 切片链。010 剩余 +2px 墨迹独立归 R4383 行盒 strut 域（plain 页块几何一致仍 +2 低，与本轮竖框域无关）。

## 3. 实现（layout-engine/tree.rs +39 / runtime_flags.rs +6）

1. **anon 片段块臂**（tree.rs ≈:3988）：anon_style 竖向 border/padding 清零；水平框保留（R5248 border advance 落账、R5335 壳 frame 剥离依赖）。
2. **split 壳 + 切片链 inline 祖先臂**（tree.rs ≈:4635 尾创建点前，紧跟 run_in_merged 块）：谓词 = `display:Inline` ∧ 非 float ∧ Static|Relative ∧ 非 flex|grid item ∧（自身 `inline_has_block_child` ∨ taffy 子 ∈ `r109.split_parents`）。块化语境豁免（float / flex|grid item / OOF）——display:inline 在这些语境块化，竖框照常占位。
3. kill-switch：`ZW_R109_SLICE_VFRAME_OVERFLOW=0`（runtime_flags from_env + value(snapshot, cached, live)，default-on）。
4. 回归：+1 pin `test_r109_slice_vframe_overflow_zeroes_vertical_frame`（span_nodes 全部 border.top/bottom = 0、padding.top = 0、border.left = length(3.0) 保留；release + dev 双 profile 绿）。

## 4. paint 域复合记账全景（R5337 探明材料，插桩已移除）

SLICEDBG 探针（border.rs:1935 `paint_block_in_inline_slice_borders`）实证 R5335 门在 insert-010 有触发：host = 28v1、abs = (8, **2.848**)、bt/bb/bl/br = 3、3 slices——但 host abs_y 应为 taffy 几何 **8**，被三点复合压低至 2.848，带最终落 y2.848 vs chromium y5，残差 **−2.15px**：

1. painter 以 inline-run y（≈ baseline − line_h ≈ 2.848）作 host 盒 abs_y（非 taffy 几何 8）；
2. `inline_finalization.rs:1216` any_text 臂对 split 壳重写 `y = line_y − padding_top − border_top`（0→−2）并从 ComputedStyle 恢复 `border_top = 3` 字段；
3. `collect_slice_regions`（border.rs:2080）逐层 `cy += padding + border` 累计（+3）；
4. 带锚 `strip_top = slice_top − bt`（−3）。

另两处待修：带绘制读 `box_node.border_*` 字段（竖框清零后 bt=0 会灭带——需改读 ComputedStyle，本轮因 host 为 orphan 走 R1442 路径未触发）；外层横带宽度应为 shrink-to-content（chromium P4 实证），ZW 片段盒满宽 772。**净零的归因**：带绘制走 paint 域自有几何链，不消费本轮修正的 taffy 几何，故 corpus 无位移。

## 5. 门禁证据

| 门禁 | 结果 |
|---|---|
| rustfmt | 0 |
| clippy（guarded，zero-layout-engine --all-targets -D warnings） | PASS（workspace 红 = zero-engine quickjs `document_compat_mode_string` dead-code，R5334 commit `78754deb3` 引入，script 域车道移交） |
| make test（test-guard，--no-fail-fast 补全） | 46 binary / 45 ok，唯一红 = zero-engine fetch 家族 3600P/9F 既有零位移 |
| pin 测试 | release + dev 双 profile 绿 |
| product-smoke | welcome 15.68% ≤ 20% 零位移 + struct PASS；legacy 42 fixtures 0 |
| corpus 最终二进制复跑 | 15792/16590 = 95.2%，798F，`diff` 基线 fail set → FAILSET-IDENTICAL |
| bench-gate | 定向 zero-layout-engine，静窗 GATE PASS 25 指标 NEW=0（load15 11.50，日志 `/tmp/r5336-benchgate.log`） |

corpus 日志：`/tmp/r5336-corpus-final.log`、fail 集对照 `/tmp/r5335-fails-sorted.txt` vs `/tmp/r5336-fails-final.txt`。

## 6. 挂账（R5337 下一步）

1. **paint 侧切片带重锚**（insert-010 收口候选，1.09% → 阈下）：①带绘制读 ComputedStyle 带宽（防 box_node 字段清零灭带）；②host abs_y 改锚 IFC 行几何（复合 −2.15px 三记账点）；③inline_finalization any_text 臂对 R109 壳不做竖框减移/恢复；④collect_slice_regions 逐层 frame 累计适配 §10.8.1；⑤片段盒 shrink-to-content 宽度语义。插桩模板：SLICEDBG@border.rs:1935、REWDBG@inline_finalization.rs:1216、BANDDBG@text.rs:3215（本轮已全删，需重加）。
2. 墨迹 +2px（R4383 strut 域）专项。
3. outer background 切片化、复杂 border-style（dashed/double）逐 slice 复刻（FIXME 挂账）。
4. clippy workspace 红移交 script 流（quickjs dead-code，非本流域）。
