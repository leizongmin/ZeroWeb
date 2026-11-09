# R5467 证据档：vrl-007 残余 3.98% 三缺陷定谳 ＋ 列压缩重测结构 cell 臂（缺陷 1 修复）

日期：2026-11-08。驱动案 `css/css-writing-modes/sizing-orthog-htb-in-vrl-007.xht`（R5466 后 ✓3.98% >strict 带）。

## 0. 轮次结论

- **归因定谳**：残余 3.98% 非亚像素域（R5466 挂账三候选全部证伪），是**三个结构性缺陷**的合成：
  - **缺陷 1**（本轮修复）：ref 臂 td#data `display: block` 逃逸 R4298 列压缩重测 → 内容高停留压缩前 taffy 值（4 行 80 vs 最终列宽 394 换 6 行真值 120），行/表高 86 vs 126。
  - **缺陷 2**（挂账）：ref 臂 htb cell 内 vertical p 测量 **w=0 h=20**（oracle：20×94.17 / 20×106.61）——正交叶（htb 父＋vertical 子）测量域，R5456（vrl 父＋htb 叶）的镜像缺口；w=0 致 paint 文本外溢到盒外错位。
  - **缺陷 3**（待用户决策⑥）：UA p margin **物理** `margin: 1em 0`（style-system lib.rs:2151）vs chromium UA **逻辑** `margin-block-start/end: 1em`——物理 margin 下双臂列位差 16/48px **结构性不可匹配**（§2 推导），strict 达标必须 UA 表逻辑化＋竖排块流边距折叠机制（现 `vertical_block_flow.rs` 无 collapse 实现；水平路径 `margin_collapse.rs` 已有）。
- **修复**：vrl-007 **✓3.98%→✓3.55%**（缺陷 1 收口）。corpus 双窗净 **+5 真 improved**（5 案 ✗→✓0.00% trusted）、**0 真回归**（4 apparent 全部 solo 证伪＝批跑噪声）。

## 1. 几何定谳（双臂 LAYOUT_DUMP ＋ chromium oracle）

chromium oracle（`/tmp/r5467-oracle/{test,ref}-probe.xht`，`--headless=new --dump-dom` title 注入 getBoundingClientRect，XML 需 CDATA 包裹脚本）：

| 元素 | chromium test 臂 | chromium ref 臂 | 恒等关系 |
|---|---|---|---|
| 正交块 ob / td#data | **[244,644] w=400 h=126** | **[244,644] w=400 h=126** | 逐位一致 |
| p after / p#sentence-after | pa [208,228] 20×h | p0 [208,228] **20×94.17** | 逐位一致 |
| p before / p#sentence-before | pb [660,680] 20×h | p1 [660,680] **20×106.61** | 逐位一致 |
| 表 | — | tab [108,8] 672×126（页内注释自认 Chrome 57+ quirk 位） | — |

chromium 双臂**逐像素一致**的机制：UA 逻辑 margin 使 vrl 列按 margin-box 堆叠（52+400+52 = 504 = ref 表 content 宽 504 逐格对齐；首子 block-start margin 塌缩出容器 → pb border 右缘 = content 右缘 680）。

ZW 修复后双臂 LAYOUT_DUMP（`LAYOUT_DUMP=1 make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-007`）：

| 元素 | ZW test 臂 | ZW ref 臂（本轮修复后） | 差异 |
|---|---|---|---|
| ob / td#data | [280,680] w=400 **ch=120 h=126** | [264,664] w=400 **ch=120 h=126** ✓ | 高度已齐；x 差 16（缺陷 3） |
| 表 | — | h=**126**（修复前 86）✓ | — |
| cell 内 vertical p | — | **w=0 h=20**（缺陷 2 未动） | 应 20×94.17 |

## 2. 缺陷 3 的双臂不可匹配推导（挂账依据）

ZW 物理 UA margin（p 无水平 margin）下：

- test 臂列栈 = border-box 邻接 20+400+20 = 440，右锚 body content 右缘 700：pa [260,280]、ob [280,680]、pb [680,700]。
- ref 臂 cell 分割 [212,264]/[264,664]/[664,716]；cell 内 p auto-margin 左贴：p0 [212,232]、data [264,664]、p1 [664,684]。
- 对照：ob vs data 差 **16px**、pa vs p0 差 **48px**、pb vs p1 差 **16px**——即使缺陷 1/2 全修，双臂仍结构性失配。
- 若 UA p margin 逻辑化而**无折叠机制**：兄弟间隙变 32px（16+16 不折叠），test pa 位置又与 ref cell 内 margin-box 填满模型（p margin-box 52 = cell 宽 → 居中 [228,248]）失配——需**兄弟 margin 折叠＋首子塌缩出容器**（chromium 实测行为）一起落地才收敛。
- blast radius：UA 表 p margin 语义切换影响全部含 p 竖排页（vrl/vlr 族近似通过带全动），且水平页语义不变（margin-block ≡ margin-top/bottom）——深结构，按执行模式记入待用户决策清单。

## 3. 缺陷 1 修复（本轮落地）

- **病理**：`remeasure_cells_at_compressed_widths` walk 的 cell 判定只认 `ComputedStyle.display == TableCell`（table_cell_content.rs:80-83），而 grid 收集（build_row）把行盒**全部非 inline 元素子**收为 cell（含被作者块化的 td，CSS2 §17.2.1）。
- **对照实证**：同一 ref 页去掉 `display: block` → td#data ch=120/h=126、表高 126（与 chromium oracle 同值），driver 3.98%→3.55%——gate 逃逸为唯一病因。
- **修法**：walk 增 parent_style 参数，cell 判定加结构臂 `parent display == TableRow`；kill-switch `ZW_TABLE_STRUCT_CELL_REMEASURE=0` 回退旧径（LazyLock 进程级单读）。
- **判例**：`tests/r5467_table_struct_cell_remeasure.rs`（钉 2：ref 形态块化 cell ch=120/h=126/表高 126＋窄表压缩泛化域）＋ `..._kill_switch.rs`（锚 1：OFF 复现 ch=80/h=86 旧径，独立进程 R3858 教训）。
- 同形缺口挂账：`constrain_table_cell_content_widths`（R767）同 gate 形态（块化 cell 的 width:auto block 子不 clamp）——本轮无驱动案证据不动，有案再收。
- 匿名 cell（is_anon_table_cell，借行 node_id）不入结构臂（parent 非 TableRow 的形态不在本轮证据域），维持旧径。

## 4. 验证矩阵

- 判例 3P；layout-engine 全腿（guarded systemd scope，`--no-fail-fast`）**lib 2444P/2I＋38 集成二进制全过＝2536P/0F**；make reftest **725/725**；fmt clean；clippy `-p zero-layout-engine --all-targets -D warnings` 清零。
- **corpus kill-switch 双窗（16594）**：w1(OFF 基线) **693F**/10331 可信、w2(ON) **692F**/10330 可信——✗→✓ 5 案（table-anonymous-objects-130/150/152、insert-block-in-blocks-n-inlines-begin-003、insert-inline-in-blocks-n-inlines-end-001，全部 solo ✓**0.00%** trusted 定谳）；✓→✗ 4 案（tao-146 3.08%、gradient-currentcolor-linear/radial-repaint 2.04%、intrinsic-percent-replaced-dynamic-008 6.25%）**solo 全部不复现**（146 OFF ✓0.00%＋ON ×2 ✓0.00%；余三案 ON solo ✓0.00%）＝批跑噪声指纹家族（R5464/R5466 同族扩员），**真回归 0**。
- product-smoke 全腿 PASS（welcome **17.23%** < 20 阈值，struct-check 全过）；bench-gate 定向 zero-layout-engine **GATE PASS 25 指标（NEW=0）**（layout_ms medium 159.04/预算 218.54）。
- 外域观察：workspace clippy 红于 `tests/wpt-runner/src/testharness_py_routes.rs` needless_return（46号流 M3 切片二 `54c5f4cea` 引入，非本轮域，不代修）。

## 5. 残余分解与下一步

vrl-007 ✓3.55% 残余主体 = 缺陷 2（w=0 测量＋文本外溢）＋缺陷 3（16/48px 列位失配）。下一步 R5468 候选：

1. **缺陷 2 归因开案**（正交叶 htb 父＋vertical 子测量 w=0 域）：R5456 镜像域，有驱动案（vrl-007 ref 臂双 p＋vrl 族多案同形态）。
2. 缺陷 3 待用户决策（UA margin 逻辑化＋竖排边距折叠机制）。
3. c5525-fltmrgn-000 ✗3.92% 归因探针（余账择优）。
