# 归档：R312 详细记录

> 从 `docs/goal/rendering-compat/master.md`「最近轮次详细记录」迁出（R332 归档，保持最近窗口 ≤20 轮）。
> 当前状态以 `master.md` 顶部「综合裁决」与「最近轮次详细记录」为准；本文件为历史记录，只追加。

---

### R312 — baseline-export 双侧探针精确定位：inline-flex 容器 taffy_baseline 错值 + multicol 项 None（read-only 探针，基线 loose 438/490 / strict 295/490 持平）

**承接**：R310 探针确认 multicol flex 项 `taffy_baseline=None`；R311 fresh cross-validate 标 `flexbox-baseline-align-self-baseline-horiz-001`（17.64% chr，inline-flex 容器基线导出）为未深查的 baseline-export 变体。本轮探针精确定位 inline-flex 容器侧的根因，与 R310 multicol 侧合拢为统一 baseline-export 图景。

**探针（env IBBL_PROBE=1，已 100% 回退）**：在 `adjust_inline_block_positions` 的 baseline_overrides 闭包（engine.rs:988-993，优先用 taffy_baseline 分支）加 eprintln。对 flexbox-baseline-align-self-baseline-horiz-001 实测：
- 3 个 inline-flex 容器（NodeId 25v1/37v1/49v1，均 child_h=35）导出 **taffy_baseline = 30.0 / 20.0 / 30.0**。
- 这些值经 `with_baseline_overrides` 喂入父 IFC 决定 inline-flex 在 body 行内的垂直位置 → 17.64% chr diff（inline-flex 整体垂直错位）。
- **关键**：baseline_overrides 的 step-3（taffy_baseline 优先，line 989-992）**总是命中**（taffy 为 flex 容器算 first_baseline），step-4（ZeroWeb 自有首行近似，line 995+）被旁路。故 inline-flex 基线 = taffy 的合成值，**该值对「混合 font-size flex 项」错**（R295 wrap-reverse/混合项基线合成结构性聚类）。

**双侧合拢（baseline-export 统一图景）**：
| 侧 | 用例 | taffy_baseline | 根因 |
|----|------|----------------|------|
| **multicol flex 项**（R310） | baseline-003 | **None** | taffy 仅 flex/grid 容器算 first_baseline，block/multicol 项无 → flex `align-items:baseline` fallback 错位 |
| **inline-flex 容器**（R312） | flexbox-baseline-align-self | **错值**（30/20/30） | taffy 算了 first_baseline 但对混合 font-size 项合成错 |

两侧共同缺口 = **ZeroWeb 缺「为 flex/multicol 项计算正确 first baseline」的能力**（CSS-align baseline-export）。修复须 layout 侧：
- multicol 项：计算首列首行 baseline（block first-baseline 递归：直接文本取首 IFC 行基线，block 子元素取首子首基线，空取 strut）。
- inline-flex 容器：合成正确 first baseline（取 baseline-aligned 项的 max 基线，而非依赖 taffy 错值）—— 但须在 taffy layout 期间或经 measure-func 注入，post-layout 填字段无法修正 taffy 已完成的 flex 对齐（R310 约束）。

**裁决**：baseline-export（baseline-000~008 + flexbox-baseline-* 聚类，~10+ 案）确认为**结构性多轮**，需 CSS-align baseline-export 的 spec-rfc 实施（block/multicol first-baseline 计算 + flex 对齐注入路径）。双侧探针已精确定位根因（multicol=None / inline-flex=错值），区别并超越 R266「field-fill 净 0」结论。这是 css-multicol + css-flexbox baseline 近-pass聚类的统一解锁钥匙，但非单会话 clean win。

**本轮 read-only 探针**：env-gated 探针（engine.rs:989 IBBL_PROBE）**已 100% 回退**（`git diff -- '*.rs'` 空）；rebuild 干净。零代码变更，基线持平。next = baseline-export spec-rfc（block/multicol first-baseline 计算来源 + flex 对齐注入路径设计），或转 @font-face 字体加载特性（css-fonts 24 案聚类，但 fontdue-vs-chromium 度量噪声限制收益）。
