# M2 证据：`::slotted(...)` 命中翻转 + flat-tree 组合子（DC-2 收口）

**日期**: 2026-11-07
**范围**: M2（[goal](../../shadow-dom-rendering.md) 里程碑 §M2；M0 计划 §6 M2 四步）
**基线**: M1 [evidence](2026-11-07-m1-host-segment-and-focus-wire.md)——`:host` 族已落，
`::slotted` 非法整规则丢弃（css-parser），matcher 恒不匹配（记账）

## 1. 变更面

| 面 | 文件 | 内容 |
|---|---|---|
| AST | `crates/css-parser/src/ast.rs` | `PseudoElementSelector::Slotted(Box<CompoundSelector>)` 变体（参数单化合物） |
| 解析 | `crates/css-parser/src/parser/mod.rs` | `::slotted` 专臂（Ident+`(` 与 Function token 双形态）：无参裸形态非法（不经 known 表）；参数 = `consume_compound_selector`（组合器链非法）；`slotted_seen` 缀后限制——子类起始形态（`#`/`.`/`[`/`&`）粗门 + 双冒号/单冒号/Function 三分支白名单门（仅 tree-abiding 伪元素）；`consume_selector` 收尾检查 Slotted 只在末化合物（缀后组合器非法） |
| 特异性 | `crates/css-parser/src/selector.rs` | 抽 `compound_specificity`；Slotted 臂 = 伪元素基 (0,0,1) + 参数化合物 specificity（`::slotted(#id)` (1,0,1) > `::slotted(*)` (0,0,1)，slotted-specificity.html driving） |
| 匹配 | `crates/style-system/src/matcher/mod.rs` | `collect_slotted_declarations_with_media` 收集器（镜像 host 段走查：@media/@layer/@supports 递归 + `selector_tail_for_pseudo` 伪元素尾对齐）；`matches_slotted_target` 三步（参数化合物匹配被分配元素 → 余部匹配 slot → 前缀链 `matches_selector_flat_recursive` 沿被分配元素 flat tree 上行）；`flat_tree_parent`（assigned_slot → slot；light 父是 ShadowRoot → host + featureless 标记）；`matches_compound_flat`（featureless 仅 `:host` 族命中，形态判定镜像 `matches_host_target`）；宿主段收集器加 `pseudo_name` 参数（`:host::before` 尾） |
| 级联源 | `crates/dom/src/document/shadow.rs` | `assigned_slot(node)` 反查（`get_assigned_nodes` 对偶；分配表空快返零开销） |
| 收集接线 | `crates/style-system/src/lib.rs` | ① slotted_matching 收集（assigned_slot → slot 沿 light 父链找 ShadowRoot → `view.for_root` → 收集，元素/伪元素路径同用）；② 宿主段伪元素接线（`pseudo` 作尾下传）；③ attach_pseudo_elements 加 `shadow_view` 参数穿线（before/after/marker/placeholder/selection/scroll-marker 族/scroll-button 全 compute 调用）；④ 早返门并入宿主段 + slotted 段非空；⑤ cascade 展开 slotted 循环 tree_context=1；⑥ CascadeKey 六源（hash/eq/lookup/insert 全链） |
| R5 继承 | `crates/style-system/src/lib.rs` | 子循环两遍走（ShadowRoot 子树先行 → slot 样式/自定义属性先于 slotted light 子入 styles 表）；slotted light 子 `parent_style`/`parent_custom` 切 slot 已算结果、样式键缓存禁用（parent_key 刻画 light 链）；非 shadow 页 assigned_slot 空表快返，filter 首遍恒空零行为 |
| 增量门 | `crates/style-system/src/lib.rs` | `compute_styles_incremental` 退化门扩 slotted 面：变更节点是 shadow host 或自身/祖先被分配到 slot → 全量重算（增量走查无 shadow 作用域输入，slotted 段会静默丢失；与 shadow 封闭退化同口径） |
| 序列化 | `crates/engine/src/js_dom_bridge/css_wire.rs`、`crates/css-parser/src/tests/css_nesting.rs` | `::slotted(<compound>)` best-effort 序列化臂（类型/id/class 直排，保真口径同 `::scroll-button`） |

## 2. 范围裁决：命令式面归 M3

M0 审计的 shadow-dom reftest 池中 `::slotted` 红案约 26/30 为**命令式 attachShadow**
语料——`resolve_slots` 生产调用点仅声明式解析时点（`materialize_declarative_shadow_roots`），
命令式 attach 与宿主 childlist/slot 属性变异不触发重解析（`shadow.rs:236` FIXME 在册）
⇒ 命令式页面分配表恒空 ⇒ `::slotted` 机制落了也无语料可翻绿。

M2 交付机制五件套 + R5 继承切父 + 单测；**声明式子集翻绿面** = slotted-has ×4
（`make reftest-upstream FILTER=slotted` 验收）；命令式面随 M3 变异应用臂接线验收
（M0 计划 §6 M3 第 1 步）。DC-2 原文验收 = 「恒不匹配」记账翻转 + 级联顺序落单测，
本里程碑即收口——语料翻绿非 DC-2 硬门。

## 3. 勘误：`:has` 在 `::slotted` 参数内合法

前轮（M0 evidence 引 css-scoping 旧版时）记录「`:has` 在 `::slotted` 参数内非法」
有误。经语料源文件复核（slotted-parsing.html + slotted-has-001..004）：

- **参数内** `:has()` 合法且须匹配（slotted-has-001..004 期望绿＝规则生效；
  csswg-drafts#10586 形态）——本实现经 `matches_selector` 全选择器匹配参数落定；
- **缀后**形态 `::slotted(*):has(span)` 非法（slotted-parsing.html 唯一 `:has`
  测试为 `test_invalid_selector`）——`slotted_seen` 单冒号分支门拦截。

## 4. query.rs 恒不匹配面不动（记账）

`crates/dom/src/query.rs` 的 `::slotted(...)` 宽容语法合法、匹配恒不匹配（R159），
属 JS 查询面（querySelectorAll 穿 shadow 语义），与本里程碑渲染级级联面分账——
不翻也不动，DC-2 收口记账以 style-system 侧为准。

## 5. 单测

- **css-parser**（`tests/slotted_selector.rs`，14 案）：合法形态（`::slotted(*)`/
  化合参数/`slot.named::slotted(em)`/`:host` 前缀链/参数内 `:has`/tree-abiding 缀后）
  + 非法形态逐案（裸形态/空参/组合器链参/缀后组合器/缀后子类/缀后 `:has`/非
  tree-abiding 缀后/二次 slotted）+ specificity 四形态。
- **style-system**（`tests/shadow_slotted_segment.rs`，14 案）：命中翻转（分配子命中、
  回退内容/宿主不命中）、参数过滤、参数内 `:has` 动态臂、树上下文（normal 外层胜/
  !important 内层胜）、段内 specificity、`:host` 前缀正反臂、树内祖先前缀（包裹槽位/
  裸槽位）、slot 余部化合物（named 槽命中 + other 槽不命中）、R5 继承（slot color +
  slot 自定义属性 var 链）、`::slotted(*)::before`、`:host::before`、未分配元素零命中。
- 全量回归：css-parser 3049 + style-system 2500 + zero-dom 893 全绿（数字见 §7）。

## 6. 实现期修复记录

1. **缀后粗门误伤终止符**：初版 `slotted_seen && !Colon → None` 把
   `::slotted(*) {` 的空白/`{` 也判非法——改为仅拦子类选择器起始形态
   （Hash/Delim('.')/LBracket/Delim('&')），终止符照常 break。
2. **Function token 分支缺缀后门**：`::slotted(*)::slotted(*)` 经 Function 分支
   二次合法化——补 `slotted_seen && !tree_abiding → None` 门（tree-abiding 含函数
   形 `::scroll-button`，白名单放行后仍走其参数解析）。
3. **前缀链锚点错位**：初版把前缀链锚在 slot（末前缀化合物匹配 slot 本身）——
   语义上 `div.wrap ::slotted(x)` 的 `div.wrap` 是**被分配元素**的 flat 祖先
   （slot 是其 flat 父入链，Descendant 时 slot 本身也合格）；改
   `matches_selector_flat_recursive(doc, element, parts, len-1)`（末化合物已在
   element 求值完，前缀按组合子自 slot 起继续）。

## 7. 门禁结果

- `make guarded-test`（workspace）：**3798 passed; 6 failed**——6 案全部为下方
  预存在集合（stash 法证逐案确认），touched 面（css-parser 3049 / style-system
  2505 / zero-dom 893）全绿。
- `make reftest` 全量：**725/725 零回归**（真通过-可信 558 (77.0%) + 真通过-可疑
  113 (15.6%) + 近似通过 54 (7.4%)，不一致 0），对齐 M1 基线。
- `make reftest-upstream FILTER=slotted`：38 案 **15 绿 / 23 红**。验收翻转落定：
  **slotted-has-001..004 全绿（各 0.00%，参数内 `:has` 动态命中）**。绿面另含
  slotted-specificity(-002)、slotted-with-pseudo-element、host-slotted-001、
  slotted-details-content/file-selector-button/placeholder、
  css-scoping-shadow-slotted-rule、remove-slotted-with-whitespace-sibling
  （css-text）、attr-in-slotted（css-values）。
  23 红逐案机制归因（上游无本过滤修前基线运行；零回归权威门 = 全量 725/725）：
  - has-slotted-×20：`:has-slotted()` 伪类死臂（M0 evidence §2.3 合法性表在册，
    chrome 弃用特性恒不匹配）——修前规则同丢同红，M2 未触及；
  - layer-slotted-rule：**命令式 attachShadow**（分配表空，M3 变异接线面）；
  - css-scoping-shadow-slotted-nested：命令式 `attachShadow({mode:'closed'})` ×2
    + **shadow-in-shadow 嵌套分配**（shadow.rs:97 FIXME）——M3 面；
  - pseudo-element-with-slotted-anchor：`anchor-name`/`position-area` 锚定位特性
    整体未实现——范围外。
- `make product-smoke`：**PASS**——form-input-perf-gate 过 + 全 struct-check 过
  （welcome/morning/article 四形态，legacy + compositor 双模式）。
- 定向 bench-gate（`ZERO_WEB_BENCH_CRATES=zero-style-system,zero-css-parser`）：
  **2/2 microbenches PASS**（css_bench + style_bench）；页面 p95 welcome 10.0ms /
  medium 324.4ms / morning 60.2ms，retained form p95=0.016ms jank=0；
  perf-gate 绝对预算 PASS（基准硬件 WARN 为报告性）。报告
  `tests/benchmarks/results/benchmark_20261107_202135.json`。M1 evidence 挂账的
  bench-gate 静窗回填项由本轮一并交付。
- **预存在失败记账**（与本里程碑无关）：`zero-engine` 6 案——
  img_runtime_deferred_settle_r5275、svg_img_settle_density_r5277、
  test_fetch_filtered_response_type_matrix、
  test_iframe_service_worker_messages_dispatch_to_iframe_container、
  test_t8k_websocket_error_nonterminal、test_t8k_websocket_event_flow_open_message_close。
  法证：clean HEAD `bb48bdf55`（`git stash -u`）逐案单跑 **6/6 确定性 FAILED**
  （每案 0 passed; 1 failed，非并行负载抖动；日志
  `.acceptance/m2-clean-head-forensics-201248.log`）。归因 img/fetch/websocket/
  service-worker 异步事件循环面，不属样式管线；挂控制面待查。

## 8. 遗留与 M3 接口

- `:is()`/`:where()` 宽容放行未实现（FIXME(45-M2+)，slotted-parsing testharness 面）。
- 增量路径祖先链重算对 shadow host 祖先仍不带 shadow 视图（M1 起既有缺口，
  非 M2 引入；slotted/host 变更已退化全量，祖先链面挂 M3 统一接线一并收）。
- M3 主刀：变异应用臂 `resolve_slots` 统一挂接（attachShadow/childList/slot 属性）
  → 命令式 26 案翻绿面；shadow Text 布局通道（R4）；B6 补充池通道。
