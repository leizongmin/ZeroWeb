# 归档：R310 详细记录

> 从 `docs/goal/rendering-compat/master.md`「最近轮次详细记录」迁出（R330 归档，保持最近窗口 ≤20 轮）。
> 当前状态以 `master.md` 顶部「综合裁决」与「最近轮次详细记录」为准；本文件为历史记录，只追加。

---

### R310 — multicol 设计文档自洽修订（v0.3→v0.4）+ baseline-export 探针实证确认（read-only 设计+探针，基线 loose 438/490 / strict 295/490 持平）

**承接**：R309 关闭 POLLUTED 杠杆后转最大失败聚类 css-multicol。`multicol-fragmentation-design.md` 存在**自洽违规**——顶部 R200/R201 纠正（balance 方向证伪、碎片化算法已存在）与底部 §3 Round 1-2（balance 测量工具）+ §5（首步=Round 1 测量工具）**矛盾**，会误导后续轮重复 R199→R200 证伪命运。本轮修订文档自洽 + 探针验证新方向。

**文档修订（v0.4）**：
- §3/§5 重写：Round 1-2 balance 工具方向**废弃**（R200 证列分配 `total/col_count` 顺序填充本就正确，类 A 残余是精度非算法）。
- 重定向为 **Round 1' baseline-export**（最大 near-pass 聚类 baseline-000/003/004/005/006）+ **Round 2' breaking wiring**（R201 Round 4'）+ Round 3' column-rule/精度收尾。
- §4 风险更新（R200/R201 纠正 + Phase A font_size 交互）。

**Round 1' baseline-export 探针实证（env MCBL_PROBE=1，已回退）**：在 `extract_baselines_recursive`（engine.rs:482）加临时 eprintln 打印每节点 `taffy_baseline`。对 baseline-003（`display:flex;align-items:baseline` > "PA" 文本 + `columns:3` multicol > `column-span:all` "SS"）实测：
- **multicol 项（node 19v1, is_multicol=true）`taffy_baseline=None`** ✓
- 其子 spanner（node 21v1 "SS"）也 `None`
- 仅 node 17v1 有 `Some(19.2)`
- **裁决：假设确认**——taffy **仅为 flex/grid 容器计算 first_baselines**（cached_baselines 补丁路径），普通 block（含 multicol）无 first baseline → 父 flex `align-items:baseline` 对 multicol 项用 fallback（box bottom/margin）对齐 → "PA" 与 "SS" 基线错位（chromium-Oracle 1.058%）。

**关键约束发现（区别于 R266）**：R266 查 `LayoutBox.taffy_baseline` **field-fill 净 0**（消费 guard 仅 InlineFlex|InlineGrid）；本轮探针发现真因更根本——**taffy 在 layout 期间已完成 flex `align-items:baseline`**，post-layout 填 `taffy_baseline`（extract_baselines_recursive 在 taffy layout 后跑）**无法回溯修正已发生的 flex 对齐**。故修复须 **layout 侧**：① 计算 multicol/block 的 first baseline（首列首行 / column-span:all 内容基线），② **在 taffy layout 前或期间**喂给 taffy（measure-func 或两趟），或 ③ ZeroWeb flex-baseline post-pass 重对齐（类 `adjust_inline_block_positions` 但针对 flex items）。三者均结构性多轮，非单点。

**意义**：① multicol 设计文档自洽（消除 R199→R200 重复陷阱）；② Round 1' baseline-export 方向经探针**实证确认**（multicol 项 taffy_baseline=None 是根因），区别并精确化 R266 的 field-fill 结论——真阻塞是 taffy baseline 计算范围（仅 flex/grid）+ flex 对齐时机（layout 期间）。这把 baseline-export 从「需 pre-pass 估测」精确为「需 block/multicol first-baseline 计算 + 喂 taffy/flex post-pass」。下轮可据此设计 baseline 计算的 spec-rfc。

**本轮 read-only 设计 + 探针**：env-gated 探针（engine.rs:482 MCBL_PROBE）**已 100% 回退**（`git diff -- '*.rs'` 空）；rebuild + baseline-003 复测 0.12% 不变。零代码变更，基线 loose 438/490 / strict 295/490 / chromium-Oracle 持平。next = Round 1' baseline 计算的 spec-rfc 设计（block/multicol first-baseline 来源 + flex 对齐注入路径），或转 DC-9 blend_mode 独立特性。
