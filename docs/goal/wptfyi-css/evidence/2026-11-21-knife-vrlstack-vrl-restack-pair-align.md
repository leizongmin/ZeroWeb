# 刀 VRL-STACK：VRL 非根容器 restack margin 配对 + 交叉轴对齐 + 锚 pass restack 后重解析（2026-11-21）

## 认领与桶

g38/g39（writing-modes-001 1296 组合 432P/864F 揭真）挂账「VRL in-flow
布局正确性刀」。本刀认领竖容器组合三个机制缺陷：

1. **非根 VRL 容器 restack margin 配对**：`vertical_block_flow.rs`
   `apply_inner` 对非根容器用 `compute_vertical_block_flow` 的 Σ-outer
   模型——相邻间隙吃 `(child[i].margin_right, child[i+1].margin_left)`
   对偶；VRL 块流（右到左）物理相邻对是
   `(child[i].margin_left, child[i+1].margin_right)`。oracle 页
   `.vrl-ltr #a2 {margin-right:-600}` 组合 Σ 模型给 a2 右缘 0、正确 600。
   R4429 根层 cursor 公式（`cursor -= mr; x = cursor - w; cursor -= w + ml`）
   已 chromium 实证，仅根层生效——本刀推广到非根层。
2. **交叉轴 y 对齐**：vertical 子自身 `direction: rtl` 时行内起点=盒底
   （css-writing-modes-3 direction 对行内起点映射），容器 definite 高时
   应底对齐（`y = container_h - mb - h`）；现恒顶对齐（`y = mt`）。
3. **锚 pass restack 后过期快照**：11.7c `apply_anchor_position_area`
   在 taffy htb 堆叠几何上收集锚几何与求值 anchor() inset；12.7 restack
   重定位锚盒后快照过期——oracle 页 `target.x = anchor(--a1 right)` 取
   restack 前 a1 左缘（x=0+100=100）而非物理右缘 200。修复 = 12.8a
   restack 完成后重跑锚 pass（raw 声明全量重解析，幂等）。

## face 与 kill-switch

| 文件 | 臂 | kill-switch |
|---|---|---|
| `vertical_block_flow.rs` | 臂 1：非根 VRL margin 配对 cursor 链 | `ZW_VRL_PAIR=0` |
| `vertical_block_flow.rs` | 臂 2：vertical 子 rtl 行内起点=盒底 y 对齐（gate 容器 definite 高） | `ZW_VRL_XY_ALIGN=0` |
| `engine.rs` | 12.8a 锚 pass restack 后重跑（gate 树内 vertical wm prescan ~20µs） | `ZW_VRL_ANCHOR_RERESOLVE=0` |
| `engine/tests/g1f_anchor_pos_tests.rs` | 两钉测（oracle vrl-ltr 端到端 + 交叉轴 rtl 底对齐对照） | —（dev-only） |

零 margin 页两式数学恒等（Σ−Σ telescoping 同式），既绿页零位移预期。
FIXME（css-writing-modes-3 margin collapsing）：相邻块轴 margin 折叠未
建模，沿用 R4429 求和口径（pos+neg 异号对求和恰等于折叠值；同号双正
对残留）。

## 钉测证据

- `g1f_vrl_stack_margin_pair_writing_modes_oracle`：oracle vrl-ltr 组合
  端到端（a1=(100,100)、a2=(500,300)、target gBCR [200,200,500,300]）。
- `g1f_vrl_stack_cross_axis_rtl_bottom_align`：rtl 子底对齐 y=100/x=400
  + ltr 子顶对齐 y=50/x=200 对照。
- 双绿（2 passed）；kill-switch 三键全 `=0` 时 oracle 钉测复现基线红
  ——A/B 臂活性实证。
- 钉测揭缺陷 3 的过程：a1/a2 几何断言先过、target.x=100（锚过期快照）
  → 12.8a 重跑锚 pass 后全绿。

## 质量门

- rustfmt 三 face 文件零 diff。
- clippy scoped `-p zero-layout-engine --all-targets`：零 lint 命中本
  face。6 处 missing_docs 为 HEAD 提交态红（`730dbfc2a` RANDOM3a
  style-system `computed_style.rs:875-885` pub r/rx/ry/x/y 无文档），
  域外不代修；`-D warnings` 形被该 6 处阻断（style-system lib 先编译
  中止，layout-engine 未到 lint 面），以无 -D 全量跑 + grep 佐证。
- 全 lib 套件（2508 passed）：12 处既有红（r5162_table_fixed_dist ×3、
  r3634/r3623/r3624/r3636 inline 残余 margin、r3625/
  extract_inline_visual_metrics/text_indent、collapsed_border
  non_px_real_units——均为 ch/em 真度量解析类，`a5380b4bb` CHMETRIC-2
  提交态面），switch-off/inert 双跑集合同一，本 face 零新增红。

## 电池结果（unit zw-knife-vrlstack-battery-1055，10:55–11:06）

- 门：FM `VRLSTACK vrl-pair-restack` b0=0/b1=1 ✓；M2 md5 异 ✓；S1/S3
  ride 零扰动 ✓（ride 11 文件双臂同载相消）；六腿 JSON 全非空。
- **RECONCILE：anchor-position gained=0/lost=0；writing-modes
  gained=0/lost=1；text gained=0/lost=0。净 +0。**
- **认领前提勘误（本电池 b0 实测定谳）**：oracle 页
  anchor-position-writing-modes-001 在 HEAD 两臂均 **1296/1296 全绿**
  （anchor-size-writing-modes-001 24/24 同）——g38/g39 记载的
  432P/864F 桶已被此前的锚系刀（APC 族等）清偿，本刀认领时取
  master.md g38/g39 记载未复跑 HEAD 核实，前提过期。本刀两臂钉测
  揭的引擎缺陷（Σ-outer 模型在特定 margin 组合下与物理右到左流分歧、
  锚 pass restack 前快照）在现语料**零页 exercised**——页面基 CSS 的
  `#a2 {margin-left:500}` 使 `outer=0`，Σ 模型与 cursor 链对该 margin
  集合 telescoping 同值（本刀钉测 fixture 用 ml=300 才现分歧）。
- **P2F 归因（writing-modes 唯一位移）**：
  `forms/text-input-vertical-overflow-no-scroll.html` 电池 b1 臂
  10→9P：子测 [9] `input[type=search] in vertical-rl` Pass→Timeout、
  [10] 截断 MISSING（CASE_WALL_S=60 墙钟）。b1 二进制单页重跑
  （180s 墙）：11 子测 10P，[9] 恢复 Pass、[10] 页尾 Timeout 与
  b0 电池逐字节同形——**判 flake**（60s 页墙钟压力），非回归；
  [10] 页尾 timeout 为双臂既有形态。实 P2F=0。

## 对账口径

预期上界 864F（oracle 页竖容器组合）按认领时记载预注册；实测该桶
已被清偿（见上勘误笔），推演 chromium-consistent 子集 ~144 不适用。
本刀按 latent-correctness 入库：钉测对（基线红→face 绿）+ 三语料
双臂电池零回归 + flake 逐案归因清账；不动 `compute_vertical_block_flow`
content-size 口径与 R4429/R5464/R4444/R5457 既有锚定臂。
