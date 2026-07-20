# RFC：vertical-rl/lr 容器 block-size（物理 width）sizing deadlock

日期：2026-07-20（R1846 + R1847 实证）
状态：**deadlock confirmed**（autonomous scoped slice 不可行；须 fundamental taffy-level vertical 支持）
关联：R1050/R1052（vertical IFC container_width=0）、R1043/R1544/R1545（vertical block-flow，已 LANDED）、R1542（vertical sizing postprocess 三证 net-neg）、R1099（vertical IFC 断列已修）

## 1. 问题（empirical 复现）

vertical-rl/lr 容器的**物理 width（block-size）**不从 IFC 结果推导，致容器尺寸错：

| 容器类型 | 实测（LAYOUT_DUMP） | 应为 |
|---|---|---|
| block-level `writing-mode:vertical-rl; height:200px` + 文本 | **w=0**（塌缩） | w ≈ 列数×列宽（1 列≈20px） |
| inline-block `writing-mode:vertical-rl; height:200px` + 文本 | **w=784**（拉满父宽） | w ≈ 列数×列宽（shrink-to-fit） |

文本断列本身**已正确**（R1099 `inline_finalization.rs:704`：vertical IFC `max_depth=content_height`，WM-aware）。缺口在断列后**容器 block-size 不从列数回填**。

**product-visible 影响**：任何 CJK vertical-text 页（`writing-mode:vertical-rl`，东亚网页常见）→ 容器 w=0 或 w=满宽，文本溢出/布局断裂。

## 2. 根因（两层）

### Layer A：taffy/vertical 维度错配（layout-time）
taffy 内部按 horizontal-tb 语义计算容器 size。对 vertical-rl 容器：
- 物理 WIDTH = block-size = 应从 vertical IFC 列数推导（列数×列宽）。
- taffy 把 vertical 容器当 horizontal-tb，其「width」= inline-size 语义 → 不从 vertical IFC 列数算 → 给 0（block-level，无子盒撑）或填满（inline-block 默认拉伸）。

### Layer B：postprocess 修复 net-negative（R1542 三证）
postprocess 设 vertical 容器 width（如 R1842 inline-grid grow 模式）→ **block-level sibling 交互破坏**（R1047/R1542 实证 net-neg 三证：sibling 位置依赖兄弟 width，postprocess 改 width 不重算 sibling → 错位）。inline-level（inline-block）postprocess 或可（sibling 交互异），但其 content-width 测量依赖 vertical IFC 列数（Layer A），同样未解。

## 3. 为何不是 scoped slice（deadlock 论证）

- **postprocess 路径**（似 R1842 grow）：block-level 被 R1542 net-neg 墙挡；inline-level 依赖 Layer A 测量。
- **layout-time 路径**（converter/taffy）：须 taffy 把 vertical 容器的 block-size 从 vertical IFC 列数推导 = taffy-level vertical 原生支持（fundamental，非 scoped slice）。R1544/R1545 block-flow 是 layout-time + postprocess 混合且仅处理**子位置 + 容器物理 width**（block-size 经 compute_vertical_block_flow Σ 子宽），但**仅对 block-level 子盒**——inline 文本（IFC）无子盒，其列数不经此路。
- **R1099 只修断列**（max_depth），不动容器 block-size 回填。

⇒ vertical 容器 block-size 从 IFC 列数回填 = 跨 taffy layout + IFC + postprocess 的 fundamental 缺口，**非 autonomous scoped slice 可解**。

## 4. 解锁条件（非本轮可交付）

1. **taffy-level vertical 原生支持**：taffy 按 writing-mode 把 block-size 从 vertical IFC 列数推导（须 taffy 上游或 fork 深改，multi-month）。
2. 或 **layout-time two-pass**：第一趟 IFC 算列数 → 回填容器 block-size taffy size → mark_dirty 重跑（R1542 已试 height-set，float/inline-block 回归 default-off；width-only 对 block-level 子盒工作即 R1545，但 IFC 文本无子盒不触）。
3. 或 **IFC 列数 post-IFC 暴露 + inline-level 专属 postprocess**：仅 inline-block vertical-rl（避 block-level R1542 墙），用 IFC 列数 shrink-to-fit。**候选 scoped slice**，但依赖 IFC 列数 post-IFC 访问（当前未暴露）+ 须证 inline-level postprocess 不触 sibling 墙。

## 5. 裁决

vertical-rl/lr 容器 block-size sizing = **deadlock for autonomous scoped slices**（R1846+R1847 双证，block w=0 + inline w=784）。R1099 已修 IFC 断列子层；残余 block-size 回填需 fundamental vertical/taffy 支持（multi-session project，非本轮）。

**勿再以「vertical 容器 sizing」为 autonomous lever**（须 fundamental 项目或候选 #3 inline-level 专属 slice 先暴露 IFC 列数）。headling unlock 仍 = font-wall（用户 A/B/C）。

## 6. forward

- 候选 #3（inline-level vertical-rl shrink-to-fit via IFC 列数）作未来 scoped slice：先评估 IFC 列数 post-IFC 暴露的工程量 + inline-level postprocess sibling 安全性。
- fundamental vertical/taffy 支持（候选 #1/#2）= dedicated multi-session project，须用户授权范围。
- 期间 plateau 维持 + 低频 opportunistic empirical hunt（非 vertical-sizing 角度）。
