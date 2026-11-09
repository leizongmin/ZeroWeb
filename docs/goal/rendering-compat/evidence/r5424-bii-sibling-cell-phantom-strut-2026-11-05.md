# R5424 — block-in-inline-001/002 弃案归因：sibling cell 触发的幻影空白行 + R109_BACKFILL 高度级联

日期：2026-11-05。上一轮：R5423（letter-spacing 族弃案 + rotateY/3D TP 定谳，825cafe2f）。
本轮为**归因轮**（产品零代码改动）：block-in-inline-001/002（1.39%/1.40%）从 w0 红案
续扫进入，实验链完整后判定落入 **block-in-inline 深域**（多轮在册挂账面），弃案挂账。

## 基线与候选

w0 fail 名单 711F（R5423 终态账面）。候选 block-in-inline-001/002：CSS2
box-display 块入行内拆分域，语料族 85 文件（css/CSS2/box-display/），此前无逐案
归因条目（域级「block-in-inline 深域」挂账已多轮在册——本轮补齐该域内 001/002
的机制证据）。

## 症状（REFTEST_DUMP PNG + 管线探针实测）

test 侧左 td（BII 内容 cell）顶部多出 **18px 幻影空白行**：td 53.2×**78**（应为
60），Line 1/2/3 整体下移 18（行位 71/89/107，应 53/71/89）；右 td（ref 结构
cell）被下推 18（y=68，应 50）。ref 侧三 div 正确。差值 ≈1.4% 超 lenient 容差。

## 实验链（全部实证，探针用后即删）

1. **单 cell 语境不复现**：复制品（单 td）与 REAL 去右 td、REAL 左 td 单独成页
   （含 REAL 原空白符、默认 body margin）三变体全部正确（td 60、行位 53/71/89）。
   `<p>` 前置块剥离不影响——触发面**纯在 sibling cell**。
2. **sibling 内容高隔离**：右 td 置空或仅 1 行 div → 左 td 正确；右 td 3 行 div
   （行高源 ≥ 左 td 自然高 60）→ 幻影复现。触发条件 = **行高分配把左 td 预增高
   到 ≥ 自然高**（position_cells 实测：REAL 行 row_height=60 → 左 td pre_h=6 →
   h=60；空右 td row_height=6..18 → 左 td h=18 不触发）。
3. **position_cells 出口盒树（临时仪表实证）**：两臂左 td 子树**相同**——单个
   18px 匿名盒（y=0）；右 td 3 div 正常。分歧不在表格布局 pass 出口。
4. **终树分歧**：REAL 左 td 的 span.inline（R109 三片结构 18/18/18）被插入在
   匿名盒**之下**（y=18，追加）——td 内容 72 = 18 幻影 + 54；REPLICA 匿名盒被
   **替换**（span y=0）→ rebuild pass 的追加/替换语义分歧，门 = cell 预增高。
5. **kill-switch 矩阵**（管线探针逐项）：`R109_BACKFILL=0` → 左 td 回 60、右 td
   回 y=50（幻影行仍在，Line 1 恒 y=71 = span 追加不回退）→ 高度/位移级联归
   R109_BACKFILL；`ZW_IFC_GROW_SHIFT=0`、`ZW_CELL_IFC_STORE=0`、
   `PHASEA_STORE_EXT=0` 全部无效（非本病机制）。
6. **右 td 下推来源**：postprocess.rs `shift_siblings_after_ifc_grow` 的
   cumulative_shift 级联消费左 td 增高 delta（+18），同源同值。

## 机制链闭环

taffy 壳层（左 td 单 18px 匿名盒）→ position_cells 行高分配预增高（sibling 内容
≥3 行时 h→60）→ 后续 R109/IFC rebuild 把 span 三片结构**追加**在匿名盒之下而非
替换（追加 vs 替换由预增高翻转）→ 幻影 18px 空白行 → R109_BACKFILL 把增高 delta
回写祖先 cell（60→78）并经 sibling 位移/级联把右 cell 下推 18。

## 弃案理由（为何非轻量）

- 追加/替换分歧点在 compute_final 期 R109 匿名块重建语义（core inline 模型面），
  与 cell 预增高（表格 §17.5.3 行高分配）**跨子系统耦合**——单一闸门修复需同时
  解释「为何预增高翻转 rebuild 语义」，该门当前**无解释模型**（本轮实验只钉住
  现象面与开关归属）。
- `R109_BACKFILL=0` 单独不足（Line 1 恒 +18，reftest 仍红）；幻影行本身在 paint
  Path B IFC 域（layout 不存 cell 行盒，store 探针空输出实证）。
- BII 域为多轮在册深域（穿插锚/嵌套 wrapper 锚/前驱正 mb 同域）；修复方向属
  「R109 重建替换语义 + 表格预增高协同」的结构性改造，爆炸半径覆盖
  block-in-inline × table 全语料族。归入既有深域挂账，不单独开 A/B。

## 旁产物（本轮副记录）

- bench-gate 定向 zero-engine 静窗补跑（R5420 遗留 + R5422 first_paint_wall_ms
  drift 合并）：**GATE PASS 19 指标（NEW=7）**，first_paint_wall_ms 三页全 PASS
 （welcome 15.21/medium 327.08/morning 148.56 vs 基线 17.07/280.31/175.22）。
  7 个 zero-engine 微基准首次纳入基线的 capture 因静窗前对端 A/B 热残留
 （page/medium/layout_ms +25.6% 触收紧优先拒绝）未落地，待真静窗补 capture。

## 清理记录

两个临时诊断测试（layout-engine r5424_bii_probe / engine r5424_bii_pipeline_probe）
及 mod.rs 注册删除；table.rs / inline_finalization.rs 临时仪表（ZW_R5424_DEBUG
打印块）git checkout 还原；target/reftest-dump 2 张 PNG 清理。产品零改动 →
corpus 无 A/B 义务。
