# 设计草案：布局依赖值的两遍样式解析（layout-dependent computed values）

**版本**：v0.1（证据汇编 + 方向草案，**待用户决策**——深结构专项，未获点名不自主开工）
**日期**：2026-09-09
**状态**：立案待批（归「待用户决策清单」）
**关联**：R4124/R4125（cq 容器链）、R4180（cq 静态推导边界）、R4182（line-clamp 约束盒）、R4185（flex line clamp）、R4188（% 高度对 stretch 父）；master.md「待用户决策清单」

---

## 0. 执行摘要

- **一句话目标**：让「值依赖布局结果的 CSS 长度」（容器查询单位 cq*、对布局期定高父的百分比、line-clamp 约束盒行预算）在布局后按**实际尺寸**重解析，对齐 Chromium 的单遍内联求解语义。
- **核心问题**：ZW 的 computed style 在**布局前**一次性解析所有长度；当解析依赖的尺寸要到布局期才确定（flex stretch、aspect-ratio 传递、容器查询链）时，求值拿到 unknown → fail-closed（cq→0、%→忽略、预算全额放行），与 Chromium 产生系统性偏差。
- **实证家族（本 goal 4 轮勘察归因，3 个独立证据源收敛）**：
  1. **cq 单位**：容器尺寸来自布局期 transfer → 链条目 None → `100cqh` 解析 0（R4180 探针；R4125 已挂账「auto 容器尺寸静态推导边界」）。
  2. **百分比高度**：flex item 被 stretch/钳制定高 → 子 `max-height:100%` style 期父高 Auto → % 被忽略 → 内容撑爆（R4188 像素带：绿块 9999px 涂满页面；R695 只解 % *height* 且在 taffy 期，不覆盖 max-height 对 stretch 父）。
  3. **line-clamp 约束链**：clamp 点落约束盒（max/min-height 定死）内时需「最后可行 clamp 点」回溯搜索（R4182/R4187 勘察；R4181 系已用局部规则清偿 4 案，剩余 043/044/045 需约束链求解）。
- **受益面（当前 fail 集可归入本家族的案）**：container-units-sharing-via-rule-node 10.75% + flexbox-definite-sizes-003/004 10.44%×2 + inline-size-bfc-floats 7.67% + line-clamp auto-043/044/045 ~5.2/3.96/4.25% + canvas-as-container ×4 / multicol-inside-container 2.08%×5 ≈ **14 案，sum ≈ 70pp**。
- **推荐方案**：三段式（见 §3）——A) cq 单位布局后 re-resolve（paint 前替换 styles 长度字段）；B) stretch 定高 flex item 的子百分比在 taffy 二趟重解析（复用 flex-grid-two-pass 重跑基建）；C) line-clamp 约束链独立求解器（不改样式管线）。
- **预估**：A 1-2 轮（复用 resolve_length_with_container 既有链）；B 2-3 轮（taffy 重跑 + 收敛守卫）；C 2-3 轮。合计约 5-8 轮、+10~14 案。

---

## 1. 证据档案（三轮独立归因，同一结构性根因）

### 1.1 cq 单位对布局期定尺寸容器（R4125 挂账 + R4180 探针）

| 实测 | 结果 |
|------|------|
| `flexbox-definite-sizes-003` 同构探针：`container-type:size` + `aspect-ratio:1` 容器（尺寸来自布局期 AR transfer） | computed width/height = **Auto** → `container_entry_for` 链条目 (None,None) |
| 子 `height:100cqh`（容器实际 100×100） | style 期解析为 **Px(0)**（fail-closed：链轴 unknown → 0） |
| 受益案 | container-units-sharing-via-rule-node 10.75%（auto 宽容器 + 跨元素共享） |

规范语义（css-conditional-5 §container-lengths）：cq 单位按最近查询容器的 **used size** 求值——「used」即布局后实际尺寸。ZW 的静态推导（R4124，只读显式 Px 声明）在容器尺寸本身依赖布局时系统性缺失。

### 1.2 百分比高度对布局期定高父（R4188 像素带）

| 实测 | 结果 |
|------|------|
| `flexbox-definite-sizes-003`：outerFlex(max-height:100) > innerFlex(flex item, stretch) > block(`max-height:100%`) | innerFlex 高度=布局期 stretch 求值（R4185 clamp 臂后的 LayoutBox 终值），style 期 computed height=**Auto** |
| block `max-height:100%` | % 父高 unknown → **忽略** → block=内容 9999px，绿涂满 y=151..599（44.9k px diff 带） |
| Chromium | 布局期 % 求值 → 100×100 |

关联既有机制：R695 `apply_indefinite_percent_height_to_auto` 只在「CB 不明确时把 % height 改写 auto」（防 taffy 错误拉伸），不做「CB 布局期定值后 re-resolve %」；taffy 对**子**的 %（height/max-height）按父 taffy size 求值，但父 stretch 高度在单趟内对 % 子不可见（taffy 0.12.1 行为，flexbox-definite-sizes 簇即其外露）。

### 1.3 line-clamp 约束盒行预算（R4181 系局部清偿 + R4182/R4187 深域）

| 已清偿（局部规则，4+2 案） | 深域残余（需约束链搜索） |
|------|------|
| R4181：clamp 点入带 bmp 子盒 → 行预算扣 bmp（auto-018/020/021/022 翻绿） | auto-043：clamp 点落 max-height 盒 **overflow 域** → 约束盒原子消费 + own-IFC cap 路径 ellipsis 接线 |
| R4181b：min-height 托底内容驱动盒 → clamp 点退盒前（auto-042） | auto-044：嵌套 max/min-height 链「最后可行 clamp 点」全局搜索（044 曾被 R4181b 无门控版翻红 1.68→4.62，局部规则不可达） |
| R4181c：Count 内截定高盒不收缩（013） | auto-045：min-height × margin-collapse × float clearance 复合 |

同构性：三者的共同形态都是「style 期单值解析 < 布局期全局约束求解」。

---

## 2. 非目标（明确排除）

- **不引入通用多遍样式引擎**：只对三个已实证子域做定向 re-resolve，不做「所有布局依赖值」的通用框架（过度设计，run-rules §2）。
- **不动 style 缓存语义**：R4178 的 cq 禁缓存边界维持；布局后 re-resolve 发生在 styles map 写回点，缓存层无感。
- **不重写 taffy**：B 案复用 `flex-grid-two-pass-design.md` 的重跑基建（mark_dirty + compute_layout_with_measure），不改 vendored 依赖。

## 3. 推荐方案：三段式定向 re-resolve

### 3.A cq 单位布局后重解析（1-2 轮，+1~3 案）

- **时机**：布局完成后、paint 前（engine 管线在 layout 与 paint 之间遍历 styles 的既有点位，如 `inject_pseudo_text_nodes` 相邻）。
- **算法**：自顶向下遍历 LayoutBox，对每节点查其最近 container-type 祖先的 **LayoutBox 实际 content 尺寸**；对 styles 中仍含 `LengthValue::Cq*` 的字段（width/height/margin*/inset*/font-size 等既有 `resolve_computed_style_with_container` 覆盖面）按 `resolve_length_with_container(container=Some((w,h)))` 重解析并写回。
- **守卫**：仅当首轮解析值与重解析值不同才写回（幂等）；`ZW_CQ_TWO_PASS=0` kill-switch；R4124 静态推导路径保留（显式 Px 声明首轮已对，重解析 no-op）。
- **风险**：styles 写回后 paint 侧读取一致（styles map 本就布局后可变——R4169 已有先例）；缓存键不含布局尺寸（R4178 已禁含 cq 样式表缓存，无键污染）。

### 3.B stretch 定高 flex item 的子百分比重解析（2-3 轮，+2~4 案）

- **时机**：taffy 首趟 + R4185 clamp 后，检测「子含 % 长度字段 && 父 flex item 高度布局期确定（stretch/clamp）且 style 期 Auto」→ 将父的 **used height** 写回 taffy style `size.height` → `mark_dirty` → 重跑 taffy（第二趟自然按 definite 父解析子 %）。
- **守卫**：只跑一趟额外重布局（无迭代收敛）；变更集合为空即跳过（零成本 fast path）；`ZW_PCT_TWO_PASS=0` kill-switch。
- **风险**：二趟布局的全局扰动——以全量 corpus A/B 门禁（零新增红）逐案验证；flex 家族 80→79 失败现状为基线。

### 3.C line-clamp 约束链求解器（2-3 轮，+3~5 案）

- **算法**：auto 模式 clamp pass 前置一趟「约束盒标定」自底向上——对子树内 max/min-height definite 盒标注 (可内截性, 高度地板/天花板)；主 pass 遇约束盒时按标注做原子消费（043）或全局回退到「最后可行 clamp 点」（044，需回溯已消耗行预算——现有 ellipsis_host 机制扩展一个可回退栈）。
- **风险**：触及 R3768-R3793 十余个既有门控；必须每子步独立 A/B + 净值纪律回退（run-rules 既定）。

## 4. 验收标准

1. 逐案：受益案清单（§0）diff 归零，且全量 corpus 零新增红（A/B XOR 逐案比对）。
2. 家族单测：每段 ≥1 锚单测（cq 重解析前后值、% 重解析前后值、044 回退点）。
3. 门禁：make test / make reftest 687 / product-smoke 同值 struct PASS / clippy -D warnings / bench-gate 定向（layout-engine）无回归。
4. 每段独立提交，净值 ≤0 即回退（三段互相独立，可分别放行）。

## 5. 决策请求

本专项属深结构（样式管线新增布局后 pass + taffy 重跑），按 run-rules §11 待用户点名。三段可独立放行（A 最小、收益确定；B 中等；C 风险最高）。若用户点名，按 A→B→C 顺序切片实施。
