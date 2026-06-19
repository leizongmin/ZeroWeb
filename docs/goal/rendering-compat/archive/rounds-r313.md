# 归档：R313 详细记录

> 从 `docs/goal/rendering-compat/master.md`「最近轮次详细记录」迁出（R333 归档，保持最近窗口 ≤20 轮）。
> 当前状态以 `master.md` 顶部「综合裁决」与「最近轮次详细记录」为准；本文件为历史记录，只追加。

---

### R313 — baseline-overrides 杠杆证伪：inline-flex 位置不受 baseline_overrides 控制（read-only 实验，基线持平）

**承接**：R312 探针发现 inline-flex 容器导出基线用 taffy_baseline（30/20/30，错值），暗示「baseline_overrides 改用 ZeroWeb 自有计算」可能是 lever。本轮 env-gated 实证该假设。

**实验（env IBBL_PREFER_COMPUTED=1，已 100% 回退）**：在 baseline_overrides 闭包跳过 step-3（taffy_baseline 优先），强制走 step-4（ZeroWeb 首行近似 `item.y + item.font_size`）。对 flexbox-baseline-align-self-baseline-horiz-001 A/B 实测：
- 默认（taffy 优先）：chromium-Oracle **17.64%**
- 探针（computed 优先）：chromium-Oracle **17.64%（完全相同）**，bbox=(0,0,800,126) 一致

**裁决：R312 的暗示证伪**——baseline_overrides（step-3 vs step-4）**不影响** flexbox-baseline-align-self 的渲染。inline-flex 的垂直位置由 **taffy 的 inline-level-box 布局**（inline-flex 作为 body 行内级盒，taffy 在 body 的 IFC 里定位它，用 taffy 自算的 inline-flex 基线）决定，**ZeroWeb 的 baseline_overrides 后处理对该用例不生效**（post-pass 重跑 IFC 未覆盖 taffy 的行内级盒定位，或该路径对此结构不触发）。

**意义**：纠正 R312「baseline_overrides 是 inline-flex 基线 lever」的暗示——**不是**。inline-flex 基线导出的真根因在 **taffy 对 inline-level flex 盒的基线合成 + body IFC 定位**，非 ZeroWeb baseline_overrides 后处理可触及。这把 baseline-export 的修复路径从「改 baseline_overrides」排除，指向「taffy inline-level-box 基线」或「ZeroWeb 重跑 body IFC 时覆盖 inline-flex 定位」（更结构性）。

**附发现（latent bug，0 reftest 覆盖，defer）**：`line-height: <percentage>` 在 computed.rs:195-206 未解析（Percentage 落 `_=>{}`），同 R308 font-size% 谱系。但 grep wpt-data **0 个 reftest 用 line-height %** → 零测试覆盖，按 code-guidelines「不实现需求之外的功能」defer（非当前目标驱动）。

**本轮 read-only 实验**：env-gated 实验（engine.rs:989 IBBL_PREFER_COMPUTED）**已 100% 回退**（`git diff -- '*.rs'` 空）；rebuild + 复测 self 0.14% 不变。零代码变更，基线 loose 438/490 / strict 295/490 持平。next = baseline-export 真路径需触及 taffy inline-level-box 基线（深，结构性），或 pivot 到 multicol breaking wiring（R131）/ DC-9 blend_mode（独立特性）—— baseline-export 经 R310/R312/R313 三轮探针确认非单会话可解。
