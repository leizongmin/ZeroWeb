# Archive — morning-work 4× 高度根因调查旅程（R255 / R256 / R257）

**归档自**: `docs/goal/rendering-compat/master.md`（2026-06-18，治理压缩，master.md 已闭环此调查仅留结论指针）
**性质**: 历史过程记录（archive 只追加）。morning-work 4× 高度调查历经 R228b/R247/R252/R253/R255/R256/R257 共 6+ 轮收敛，root cause 已定位 + 修复（`a2b169e`）+ 验证。本文件保留 4 个 section 的**原文 verbatim**（R255 修复落地 + R255 触发因子算术 + R256 自纠正[含被证伪横幅] + R257 真根因确认）+ 夹带的 stale 候选清单，供历史追溯。
**最终结论（也保留在 master.md）**: `<article>` 缺 `ua_default_display` block 条目 → 回落 inline → 含 block 子 → R109 触发 → 匿名块继承 `.article{min-height:200;margin:4em}` → 每对 block 兄弟间 ~328px 幻影盒 → body 累积 4.2× 高。修复 = a2b169e 补 article/aside/details/hgroup/menu/search 到 ua_default_display。验证 = body 25301→5677px、fullpage chr-diff 89.14%→48.65%、reftest 438/490 零回归。
**方法学教训**: 推理代码路径前须核实元素实际计算值（display）；R256 据「article 应是 block」常识假设（而非实测 computed）下结论被 R257 推翻——article 实为 inline（UA 缺失）。

---

## R255 — morning-work 4× 高度幻影间隙修复落地（原文，2026-06-18）

承接 R253（morning-work 89% diff 真主因定位为元素间幻影垂直间隙，最小复现「未重现」留 BISECT）。本轮系统 BISECT + 落地修复，**首个打破 morning-work plateau 的真实修复**。

**BISECT 隔离（纠正 R253「最小复现未重现」误判——因最小用例未带 .article 类/外链 CSS）**：
1. `<article class="article">` + 外链 CSS（min-height:200px）→ **复现**幻影盒（每对块级子元素间一个 `article.article` w=4 h=200 mt=64 匿名盒）。
2. `<article>` + **空 CSS**（纯结构）→ **仍复现**（幻影盒 h=0 w=4）→ 证明非 CSS 触发，是结构层。
3. `<div class="article">` → **干净**（排除 .article 类/min-height）。
4. **逐标签扫描**：`<article>`/`<aside>`/`<details>` 触发幻影；`<div>`/`<section>`/`<main>`/`<nav>`/`<header>`/`<footer>`/`<blockquote>`/`<figure>` 干净。

**根因（精确）**：`ua_default_display`（style-system/src/lib.rs:49）block 列表**缺 article/aside/details/hgroup/menu/search**。CSS display 初始值=inline → 这些标签回落 inline；含 block 子元素（h2/p）时 `inline_block_split::inline_has_block_child` 返回 true → tree.rs R109 路径（line 554-591）为每段连续 inline 内容（块级子元素间空白文本节点）生成匿名块盒，继承父 node_id（→ dump 标签显示 article.article）+ 应用 .article 样式（min-height:200、margin:4em）→ 每对块子元素间插入 h=200+mt=64≈264px 幻影高度，累积 4×。

**修复**：`ua_default_display` block 列表对齐 **HTML Living Standard UA 样式表**（13.1.1 display:block 列表），补 `article|aside|details|hgroup|menu|search`。单点改动（style-system/src/lib.rs:52-55），未触 layout/paint/converter。

**验证**：

| 指标 | 修复前 | 修复后 |
|------|--------|--------|
| morning-work body 高度 | 25301px（4.2× chr） | **5677px**（≈0.95× chr 5981） |
| full-page chr-diff（800×5981） | 89.14%（R253） | **48.65%**（-40pp） |
| LAYOUT_DUMP 幻影盒 | 每对块子元素间 article.article w=4 | **消除** |
| reftest 自源 | 438/490 | **438/490 零回归** |
| make test | — | **12235 passed / 0 failed** |
| clippy -D warnings / fmt | — | 干净 |

**单元测试**（style-system ua_display_tests）：① `test_html_block_level_sectioning_elements_default_to_block` 钉死 article/aside/details 等 HTML block 标签默认 Block；② `test_inline_elements_remain_unset` 防 span/a/code 等被误标 block。

**诚实结论**：① 800×600 顶部视口 chr-diff 仍 28.72% **不变**（顶部区 nav/title/item-tags 在折叠上方不受幻影盒影响；残余=item-tag R109 + font-weight R229，独立）。② full-page 48.65% 残余=font-weight(R229)+fontdue CJK 度量+item-tag R109+hljs(需JS)+body ~300px 高差，均已知独立子问题。③ 本修复是**正确几何 bug 修复 + 零回归**：消除 morning-work 4× 高度，HTML 区块元素 display 正确化，DC-13 morning-work 从「4× 高+89%」降至「~1× 高+48.65%」，剩余由 font-weight/item-tag 独立推进。证据 evidence/r255-morning-work-phantom-gap-ua-display-fix-2026-06-18.txt + product-static/morning-work/*-fullpage.png。

### 夹带的 stale 候选清单（原文，orphaned 块，引用旧 435/490 基线，历史保留）

经 R140（独立穷尽验证）+ R141b（R109 6 轮不可解）+ R144（属性审计穷尽）三重确证，**435/490 为单会话零回归平台期**。剩余 55 失败全属结构性多轮里程碑，按预期收益/风险排序的候选路径：

1. **R109 inline-block ownership 架构项目**（多轮，高风险，潜力 +2 集群 css-flexbox-row/test1 + flexbox-column-row-gap-004）
   - 根因：vertical-rl flex 容器中 inline-block 色块（经 IFC 绘制）的 x 坐标不受 `mirror_vertical_rl_block_children` 后处理影响（R141b 已证 6 轮单兵不可解）。PIL 确认 css-flexbox-row.html 4 色块 TEST x[670,789] 右 vs REF x[10,39] 左。
   - 真正修复需统一 paint 的 inline-block 绘制路径到 IFC（DC「Layout/Paint IFC 双路径」），消除绕过 paint_node 与 paint_text 非存储的两条已知路径。属多次会话架构重构。
   - 见 [[r109-writing-mode-flex-arch]]、[[r142-vertical-rl-sibling-shift-axis]]、[[r140-cleanwins-exhausted-verified]]。

2. **R131 multicol 列感知 IFC 碎片化**（多轮，高风险，潜力 ~16 测试但回归 ~20 通过用例）
   - 根因：multicol 塌缩的零高子元素是行内级 DOM 节点(text/br)，「行高回写」不可行；真正修复=列感知 IFC 按行盒（非子元素）碎片化到各列。R122 记录 paint 路径改动 net -5（multicol 39→34/57）。
   - 前置：需先确认 R131 block-children 分布会回归的 ~20 个单列回退恰好通过的用例（multicol-breaking-000/001/002/003、baseline-000~006 等）。
   - 见 [[r131-multicol-fragmentation-arch]]、[[r113-nested-multicol-twopass-plan]]。

3. **R114b writing-mode 垂直轴 float/clearance 参数化**（多轮，中等风险）
   - 需把 ~150 行 `adjust_float_positions_with_context` 做 block/inline 轴参数化，非 surgical flag，对通过的 floats-clear 有高回归风险。R133 已建实现地基（converter 不换 float，物理 left/right→block 方向）。
   - 见 [[r114-writing-mode-characterization]]、[[r133-vertical-float-impl-ground]]。

**对后续会话的明确指引**：
- 不要重试 R125 large-font 三路径死锁、R140 gap-fix、R141b R109 单兵镜像——均已验证 net-negative 或 +0。
- 不要再做属性实现完整性审计——R144 已穷尽（197 注册属性全有 apply 分支）。
- 单会话若需推进，唯一现实路径是启动上述 3 条结构性多轮项目之一的**第一个安全子步骤**（如 R109 的「定位 inline-block 背景 fill 实际绘制入口」插桩诊断，不改逻辑、零回归），并明确标注为多轮项目的第 N 步。

---

## R255 — morning-work 4× 高度幻影间隙触发因子定位：`.article{min-height:200;margin:4em}` 算术恒等式（原文，read-only，2026-06-18）

**承接**：上一 docs 轮 d8f18d3（标号误用 R253，与 000a462 R236 multicol 的 R253 冲突）写了 morning-work 4× 高度调查 evidence（`evidence/r253-morning-work-4x-height-phantom-gaps-2026-06-18.txt`）但**未写 master.md section**，且其"下一轮隔离根因"开放问题未答。本轮 read-only 文本核对（article.html + article.css + R253 dump 算术）**回答该开放问题**，并把 morning-work 调查正式补进 master.md（用 R255 避开 R253 标号冲突）。零源码/零渲染。

**决定性恒等式**：R253 dump 的 article 子元素间隙 5 个中 **4 个 = 328px**（h2→p/p→p/p→h2/h2→h3/h3→p 全 328，与元素类型无关），1 个 272px。328px 恰为 `.article` 盒属性之和——

```
.article { min-height: 200px; margin-top: 4em; margin-bottom: 4em; }   /* article.html 内联 <style> 77-81 行 */
  min-height 200 + margin-top 64(4em×16) + margin-bottom 64 = 328px    /* .article font-size:16px(article.css) 确认 4em=64 */
```

**幻影盒非 HTML 字面重复**：article.html **全文仅 1 个** `<article class="article">`（168–352 行），故 R253 dump 里 `h=200 w=4` 的幻影嵌套 article **是 ZeroWeb 生成**（h=200 又精确=min-height）→ 与 328 恒等式同源：ZeroWeb 正按 `.article` 盒模型生成幻影盒，~328px/对插在每个 block 兄弟边界。`min-height`/`margin` 为 CSS 非继承属性，匿名盒本不该继承 → 指向**选择器过匹配 / 幻影盒被赋予 `.article` 身份 / inter-block 空白文本节点被包成匿名块且错误套用 `.article` 规则**，属 R109 谱系（匿名块/inline→block/IFC 边界）相邻缺陷。

**纠正 R253 最小复现失败根因**：R253 用 h2+p+pre 复现 328px gap 失败，因复现**缺 `.article{min-height:200;margin:4em}` 容器**——触发因子不是 h2/p/pre 标签或 line-height/word-break，而是「带 `{min-height:200;margin:4em}` 的 `.article` 容器 + 多个 block 子元素」组合。**修正最小复现**：`.article` 容器(min-height:200;margin:4em)+h2/p/h2/p → 预期 ZW 间隙≈328px（chromium ≈16/52）。

**下一步（合规运行时验证，交付实现/运行时轮）**：① 修正最小复现经**合规路径**（临时 wpt-data 自源 reftest + `<link rel="match">` + `make reftest` + `LAYOUT_DUMP=1`，**禁裸 cargo test**，测后删临时文件恢复 490 基线）渲染确认 328px；② 单变量 BISECT（删 min-height / 删 margin:4em / 删 .article class）锁 328 各分量来源 → 定位 ZeroWeb 哪条路径把 `.article` 盒属性套到幻影/匿名盒；③ 修复后 morning-work 4×→~1× 高，product-smoke A/B 量化 chr-diff 降幅（应远超 R252 的 0 改善）；④ wpt-data 自源 reftest 438/490 中性回归。w=4 细节次要未释（h=200 才是信号）；272 离群≈328−56 疑边界 margin 折叠，主信号 328 不受影响。

**与既往诊断关系**：不动 R253 主结论（4×高=布局层幻影垂直间隙，非字体噪声/非 pre 堆叠/非内容压缩），仅精化"疑似匿名块"→"`.article` 盒属性泄漏到幻影盒，328=200+64+64 可证"；与 R247/R252 pre 多行（已修，独立）正交；与 reftest 杠杆三部曲（R251/R253/R254）正交——本轮是 **DC-13 product-smoke 杠杆**（morning-work 89%），非 reftest 438/490 杠杆。**本轮 read-only 无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r255-morning-work-phantom-gap-arithmetic-identity-2026-06-18.txt`。

---

## R256 — 【自我纠正 R255】（原文；⚠️ 本节定性结论已被 R257 证伪，仅历史保留）

> **⚠️⚠️ 本节「R255 机制被否定」的定性结论已被证伪，仅保留作历史记录——见 R257 + 并行 agent 提交 `a2b169e`（ua_default_display 补 article 等，经验证 morning-work body 25301→5677px、fullpage chr-diff 89.14%→48.65%、reftest 零回归）。** R256 的错误在于前提「article 计算 display:block」——实际 article 因 UA 缺失计算为 **inline**，故 R109 **确实触发**，R255 的「.article 盒属性泄漏到幻影匿名盒（R109 谱系）」机制 + 328=200+64+64 算术**全部正确**。R256 仅「converter 映射无误 / build_subtree 对*真正 block 容器*不生成幻影盒」的代码事实成立（但 morning-work 的 article 非 block，故不适用）。**勿据 R256 否定 R255 或改 build_subtree/converter/R109——缺陷在 ua_default_display（已由 a2b169e 修）。** 下文为 R256 原文。

**承接**：R255 据 R253 dump 摘要 + 算术巧合（328=200+64+64）推断 morning-work 4× 高度机制=「`.article{min-height:200;margin:4em}` 盒属性泄漏到 block 兄弟间幻影/匿名盒（R109 谱系）」。本轮 read-only 读 build_subtree（tree.rs:362-664）全路径 + inline_block_split.rs + converter/mod.rs 核实该机制——**结论：机制被否定**（**注：此结论因前提 article=block 错误而无效，见上方横幅与 R257**）。

**决定性代码事实（build_subtree 对 `<article>` block 容器）**：
- **R109 匿名块拆分不触发**（tree.rs:546）：`r109_segments` 仅当 `inline_has_block_child(...)` 为真才算；该函数（inline_block_split.rs:59-61）首条要求 `display==Inline`——article 是 Block → false → r109_segments=None → 走 tree.rs:618 else，不进 559-591 匿名块生成。
- **非 flex/grid 容器只收 Element 子**（tree.rs:618-627）：623 行 `matches!(&n.kind, NodeKind::Element(_))` **显式跳过文本/空白节点**（正确 CSS：block 容器 block 兄弟间空白不生成盒）。
- **每 element 子一个 taffy 节点**（632-644），article 自己一个（654）+ node_map 一对一（660）。**无第二个带 article.node_id 的盒**。
- **converter 映射正确**（converter/mod.rs:81-88）：min_size.height←min_height、margin←margin，只作用于 article 一个节点，不泄漏到子/幻影。

→ R255 三个因果子机制（inter-block 空白→匿名块 / 幻影盒被赋 .article 身份 / 选择器过匹配）**在 build_subtree 里都不成立**。dump 标签语义（reftest.rs:1043-1047 `b.node_id→DOM labels`，缺才 `(anon)`）+ build_subtree 一对一映射 → R253「幻影嵌套 article w=4 h=200」**不应存在**，疑人为筛选摘要伪影。

**R255 降级**：328=200+64+64 算术**相关性真实但无因果机制**，且依赖 R253「人为筛选 6 行摘要」（非原始递归 dump）。**勿据 R255 改 build_subtree/converter/R109（无对应缺陷）**。观察层（ZW body 25301 vs CHR 5981=4.2× 高；tall-viewport 内容下移）是 product-smoke A/B 实测，仍可靠、仍需解释，但机制不是盒泄漏。

**强制性下一步（须原始递归 dump + clean tree）**：只有原始递归 LAYOUT_DUMP（reftest.rs:1033 带深度缩进全量打印）能区分「真幻影盒→定位生成它的 post-pass」vs「摘要伪影/跨层 abs_y 误读」。**须在 clean tree 跑**——当前并行 agent 未提交 `style-system/src/lib.rs`，脏树 dump 会编译进其 WIP 致证据不可靠。合规命令：`./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest <filter> --wpt-data <path>` + `LAYOUT_DUMP=1`（test-guard 包裹=非裸跑；`make reftest` 无 filter 跑全量，单测用 test-guard 直 prefix）。步骤：① 树干净后渲染 morning-work/最小复现；② 读**完整递归 dump** 核对 h2↔p 间到底有哪些盒（node_id/abs_y/h/mt/dmt），判断 328px 是真间隙还是跨层差；③ 若有带 article.node_id 第二盒→grep extract_layout/engine.rs compute() post-pass（build_subtree 已排除）；若无→R253/R255 间隙观察是伪影，须重做 morning-work 89% 根因。

**方法学教训（印证 R164/R203/R241）**：R255 据 curated 6 行摘要 + 算术巧合推断机制、未读生成路径源码即下结论；R256 读 build_subtree 推翻。**单点/摘要推断须源码实证，不能据 curated 输出 + 算术巧合接力**。后续 morning-work 调查从原始递归 dump 重起。**本轮 read-only 无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r256-morning-work-phantom-box-mechanism-refuted-2026-06-18.txt`。

---

## R257 — morning-work 4× 真根因【确认 + 纠正 R256】（原文，read-only，2026-06-18）

**承接 + 关键转折**：读并行代码 agent 未提交 WIP（style-system/src/lib.rs），其回归测试注释明确揭示 morning-work 4× 高度**真根因**。结合源码核实——**R255 的机制+算术正确，R256 的「反驳」错误**（前提「article=block」假）。本 R257 取代 R256 定性，收口 morning-work 4× 调查（历经 R228b/R247/R252/R253/R255/R256/R257 共 6+ 轮）。

**真根因链（全源码闭合）**：
1. **触发源**：committed `ua_default_display`（style-system/src/lib.rs）block 列表**不含 `article`**（亦缺 `aside`/`details`/`hgroup`/`menu`/`search`）→ 函数返 `None` → 元素回落 CSS 初始 `display:inline`。
2. **R109 触发**：article 计算为 inline + 含块级子（h2/p）→ `inline_has_block_child`（inline_block_split.rs:59-61 查 `display==Inline`）**返 true** → tree.rs:546 `r109_segments=Some` → 进 559-591 匿名块生成。
3. **盒模型泄漏**：tree.rs:386 `computed`=**article 计算样式**（含 `.article{min-height:200;margin:4em}`）；tree.rs:571-572 每个 R109 匿名块用 `computed_style_to_taffy(&computed)` 构建（强制 display:Block）→ **匿名块继承 article 的 min-height:200+margin:4em** → h≥200+margin64/64=**328px/对**。
4. **4× 高**：每对 block 兄弟间插 328px 幻影盒，累积 ~4.2×（ZW body 25301 vs CHR 5981）。R253 dump 的 328/328/328/328 间隙 + 「幻影嵌套 article h=200 w=4」（带 article node_id 的 R109 匿名块，dump 标签 `b.node_id→DOM labels`）全部吻合。

**纠正 R256**：R256 读 build_subtree 后断言「article 是 Block→R109 不触发→build_subtree 不生成幻影盒→R255 机制否定」。**代码路径阅读本身正确，但前提「article=block」错误**（article 实为 inline）。故 `inline_has_block_child(article)=true`，R109 **确实触发**，R255 的「.article 盒属性泄漏到幻影匿名盒（R109 谱系）」+ 328=200+64+64 算术**全部正确**。R256 降级为「基于错误前提的部分分析」——其「converter 映射无误 / build_subtree 对*真正 block 容器*不生成幻影盒」的代码事实成立但对 morning-work 不适用（article 非 block）；其「勿改 build_subtree/converter/R109」的*操作结论*仍成立（缺陷在 ua_default_display），但「R255 机制否定」的*定性结论*错误，本 R257 取代。

**与并行 agent WIP 对齐（非我改动）**：并行 agent WIP（未提交）style-system/src/lib.rs 把 `article`/`aside`/`details`/`hgroup`/`menu`/`search` 加入 ua_default_display block 列表 + 新增 `ua_display_tests` 回归（注释引用「R253 morning-work 4× 根因」）。修复后 article 计算 block→`inline_has_block_child`=false→R109 不触发→无幻影匿名块→正常 block 布局。**从源头消除触发，正确最小**。inline/mod.rs 仅 rustfmt（3 行无逻辑）；新增 capture-fullpage.mjs（puppeteer fullpage 截图，服务 morning-work tall-viewport 验证）。

**预期 + 验证（已由并行 agent 提交 `a2b169e` 经验证实）**：修复后 morning-work 4×→~1× 的预期**已获经验确认**——并行 agent fullpage A/B（capture-fullpage.mjs）实测 morning-work body **25301px→5677px**（≈0.95× chr 5981，4× 消除）、fullpage chr-diff **89.14%→48.65%**（-40pp）、LAYOUT_DUMP 幻影盒消除、reftest 自源 **438/490 零回归**、make test 12235/0、clippy/fmt 干净。其逐标签 BISECT 进一步精化：`<article>`+空 CSS（无 .article 类/min-height）**仍复现**（h=0 w=4）→ **纯 UA display 缺失触发，非 CSS**（.article 盒模型只决定幻影盒*高度* 200+margin，不决定*是否触发*；触发仅由 article=inline 决定）。**morning-work 4× 线收口**。残余 fullpage ~48.65% = 独立子问题（font-weight R229 / fontdue CJK 度量 / item-tag span→block R109 / hljs 语法高亮缺 JS / body ~300px 高度差），非本根因，留后续轮。⚠️ ua_default_display 改动使 article/aside/details 等从 inline→block——a2b169e 已跑 reftest 438/490 零回归，但若后续扩大导入含重用这些标签的用例，须复核。

**方法学教训**：推理代码路径前须**核实元素实际计算值**（此处 display），UA 默认值是隐藏假设；R256 据「article 应是 block」常识假设而非实测 computed 下结论被推翻。morning-work 4× 根因 = 最朴素的 UA 默认 display 缺失，历经多轮才定位。**本轮 read-only（仅读并行 agent WIP + 源码核实），无代码/reftest 变更，基线 438/490 持平。** 详见 `evidence/r257-morning-work-rootcause-confirmed-ua-display-2026-06-18.txt`。
