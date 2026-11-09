# R5466 证据：病 B 三片式表布局——显式宽 floor 词级 min-content＋超宽压缩冻结臂＋正交子块轴测宽——驱动案 vrl-007 ✓4.27%→✓3.98%（ref 臂列宽 52/400/52 逐位锁定）

日期：2026-11-08。基线 HEAD：`6745f9d97`（R5465）。驱动案：`sizing-orthog-htb-in-vrl-007`（R5465 挂账主账「病 B 三片式表布局」，oracle 归因证据 [r5465 §4](r5465-abspos-calc-inset-2026-11-08.md) 即开工输入）。

## 1. 三片修复（oracle 探针组归因 → 分步落地，各带 kill-switch）

- **片 1（floor 词级 min-content，`ZW_TABLE_SPEC_FLOOR_MINC=0` 回退）**：`table_types.rs` `compute_cell_intrinsic_width_impl` floor 语境（`for_explicit_floor=true`，唯一调用点 table.rs R364b 显式宽 cell 列宽下限）95% 臂失败后的 text fallback，从 `box_content_max_width`（max-content）改 `block_min_content_width`（R5036 已有，词级软换行）。语义 = css-tables-3 §3.6.3：指定宽是 used width、内容可断行，floor = max(specified, 词 min-content)（chrome：td width:52 + 块级 p "Sentence after." 侧列 = max(52, 词 min 57.75) = 57.75，max-content 124.8 不参与列宽）。R364b 注释自称 min-content 与实现背离一并勘误；R4028 停用 DOM max 臂防 c5501 撑破的意图由本片同防（新钉含 c5501 防护判例：td 10em + 可断行长文本列宽恰 160 不被内容撑破）。
- **片 2（shrink 冻结臂，`ZW_TABLE_SHRINK_FREEZE=0` 回退）**：`table.rs` `compute_column_widths_inner` 新增 `col_frozen_floor` 快照（col 元素 width 属性与显式 cell 的 used 贡献 w 逐列取 max，含片 1 floor 与 R5447 frame 口径）；R4298 AUTO_SHRINK 超宽压缩臂由纯比例摊薄改为**按各列 slack（当前 − 冻结下限）比例分摊赤字**，显式列不摊薄到下限之下（css-tables-3 §3.4.3 列下限 = max(specified, min-content)；与 R364 扩展填满冻结臂对称——扩展时显式列冻结、超宽压缩同样不摊薄；chrome 探针 57.75/388.5/57.75 同构实证）。floor 和已超 available 或赤字 ≥ 总 slack（无摊薄空间）回退整行比例压缩旧径（CAP 溢出域）。
- **片 3（正交子块轴测宽，`ZW_CELL_ORTHOG_INTRINSIC=0` 回退）**：`compute_cell_intrinsic_width_impl` 新臂——htb cell 且**全部非 OOF 子为 vertical-flow 块级**（无 cell 直接文本/水平块子；无 inline/inline-block 元素子；无块轴 definite 宽子；OOF 子不入固有宽 R5444 同口径不阻断）时，cell 固有宽沿子的**块轴**测：vertical IFC 行内轴 INFINITY 不设限（R4478 同款）→ min-content = max-content = Σ列宽 = 行高，文本 advance 进物理高不进宽（css-writing-modes-3 §7）。旧水平测量把文本 run advance 当宽（vrl-007 ref 臂 nbsp 句 124.8/137.6；chrome：vertical p 块轴 ~20 → 侧列 = max(52, 20) = 52）。混合轴向内容（vertical 子 + 水平块子混排）与块轴 definite 宽子不入臂（FIXME：混合域应逐子按轴测量合成）。

## 2. 验证

- 常驻钉 4＋kill-switch 锚 1＝`tests/r5466_table_floor_shrink_orthog.rs`（①c5501 防护域 td 10em+可断行文本列宽恰 160、邻列 344 维持；②shrink 冻结 [80,120] 非 [42.1,157.9]；③ref 形态联合 52/400/52 列起点 0/52/452；④vertical p cell 固有宽 = 块轴行高 20 非 124.8）＋`..._kill_switch.rs`（三开关全 OFF 复现压缩摊薄 <52 旧径，独立进程 R3858 教训——LazyLock 进程级单读）。
- 驱动案 vrl-007 **✓4.27% → ✓3.98%**（pass 带内残余再收 −0.29pp，仍 >strict；ref 臂列宽病已由常驻钉 ③ 逐位锁定 52/400/52，残差主体待新归因）。
- 内测 reftest **725/725**；fmt clean；clippy **workspace --all-targets -D warnings** 清零（`test-guard --compile-first` 仅支持 cargo test，clippy 走 `make guarded-clippy`）。
- layout-engine 全腿 compile-first 守卫：**38 集成二进制 89P/0F＋lib 2444P/0F/2I**。
- corpus kill-switch 双窗（16594，同 release 二进制，env 经 `env VAR=0 make reftest-upstream` 透传）：w1(ON) **688F** / w2(OFF) **691F**——movers 3 案（tao-127/130/145，OFF 独红）**双态 solo×3 全 ✓0.00%**＝在册 table-anonymous-objects 互翻噪声指纹家族（R5459-R5465 同族先例，ON 独红 ∅）＝**真修复 0、真回归 0 净零扰动**；c5525-fltmrgn-000 双窗同红 ✗ 维持。
- product-smoke 全腿 PASS（welcome oracle 17.23% 逐字节同值＋struct 全站＋320 窄门＋form-input-perf-gate）。
- bench-gate 定向 zero-layout-engine 静窗（1min load 0.36、构建进程零）**GATE PASS 25 指标（NEW=0）**。

## 3. 残余与挂账（未了结债）

- vrl-007 残余 **3.98%**（>strict 1% 带）主体未定谳：ref 臂列宽三片已修，残差归因需新探针（候选域：test 臂 vrl 块流基位/列 restack 亚像素域、字体度量带、abspos calc-inset 求值亚像素差）——下一轮 LAYOUT_DUMP 双臂对照归因后择优。
- 片 3 混合域 FIXME：vertical 子 + cell 直接文本/水平块子混排、块轴 definite 宽子，均不入本臂（旧径维持）——应逐子按轴测量合成，待驱动案。
- R5465 既有挂账全部维持：nested-CB/fixed-stretch pass calc-% inset 同病塌缩、abspos `width/height: calc(<percentage>…)` converter/taffy 路径未核实、calc-% margin 与 % margin 塌缩 0 旧口径、c5525-fltmrgn-000 ✗3.92%（本轮双窗同红再证稳定）等。
