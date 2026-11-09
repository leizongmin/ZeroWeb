# 45 号 M0——挂接点审计 + reftest 盘点 + 数值门（2026-11-07）

DC-1 交付：渲染管线挂接点审计（JS→Rust slot 通路 / 布局树挂接 / 级联入口三点位）
+ shadow-dom reftest corpus 盘点 + DC-4 数值门落定。全部结论按当前源码核实
（HEAD a896a0707 近亲），任务文档过期基线在第 1 节修正。

## 0. 方法与口径

- 三点位独立审计（样式/级联、布局/绘制、JS→Rust slot 通路），逐条带 file:line。
- 基线实测：`make reftest-upstream FILTER=shadow-dom`（test-guard 包裹；release
  runner 本流 10:22 构建）。语料 = 本地 `tests/wpt-runner/wpt-data/`（zeroweb-wpt-data
  v1.10 快照 + 42 号滚动），上游 pin `WPT_REV=315976933870b34d6ea30e3f6643403edae678ba`
  （goals/lib.sh 同源）。

## 1. 对账修正：任务文档「基线事实」已过期

任务文档（v1.0，2026-11-04）判「Rust 渲染侧未建——shadow 树内容不出现在渲染输出」。
**不成立**，两块渲染消费早已落库：

- **布局扁平化**：commit `c8e849320`（2026-05-31）——`crates/layout-engine/src/tree.rs`
  `build_subtree` shadow 分支（:4014 shadow root 走 `collect_shadow_children`；:4028
  shadow 树内走 `collect_shadow_slot_children` 做 slot 替换），消费
  `Document::shadow_root`（`crates/dom/src/document/shadow.rs:80`）与
  `get_assigned_nodes`（shadow.rs:379）。单测
  `crates/layout-engine/src/tree/tests.rs:1053`。
- **样式 scope 管道**：s168——`StyleSystem::compute_styles_with_shadow`
  （`crates/style-system/src/lib.rs:584`）+ `ShadowScopeStylesheets/ShadowScopeView`
  （lib.rs:53-79），`compute_styles_recursive` 逐子边界切换 scope（lib.rs:1541-1551）；
  host→shadow 继承贯通（ShadowRoot 非元素，parent_ref 直连，lib.rs:1494/1546-1557）。

### 真实缺口集（收窄后，M1~M3 的工作面）

- **R1 渲染管线不消费 scoped sheets**：渲染主链调 plain `compute_styles`
  （`crates/engine/src/pipeline/mod.rs:1825/:1286/:2120`，空 `shadow_scopes`）——
  shadow `<style>` 块被排除出 document sheet list（`crates/dom/src/document/mod.rs:1810`
  `:1830` 跳 ShadowRoot 子树），scoped 收集管道（`collect_shadow_scoped_sheets`
  `pipeline/mod.rs:4056`）仅被 getComputedStyle 桥消费（computed_style.rs:95）。
  **净效果：shadow 作者样式在渲染面整体不生效**（UA 默认 + host 继承仍在）。
- **R2 `:host` 族/`::slotted` 匹配缺失**（DC-2 原判成立且更重）：matcher 无 arm——
  `matches_pseudo_class` fall-through 恒 false（`crates/style-system/src/matcher/mod.rs:354`）；
  `:host()` 与 `::slotted()` 在 css-parser **非法**（不在 known-function/known-pseudo-element
  表，`crates/css-parser/src/parser/mod.rs:1112/:1117-1152`）→ 整规则丢弃；
  `:host-context`/`:has-slotted` 合法死臂（parser/mod.rs:1095/:1112）。`::slotted` 宽容
  解析恒不匹配仅 dom/query.rs（R159，query.rs:1394/:1627）。
- **R3 slot 分配仅声明式解析时点写**：`resolve_slots`（shadow.rs:317）唯一生产调用 =
  `materialize_declarative_shadow_roots`（shadow.rs:238；FIXME :235-238 明示命令式
  attachShadow / childList / slot 属性改名不重解析）→ 动态面分配表 stale/empty。
  命令式 attachShadow 有真 Rust shadow 树（`DomMutation::AttachShadow` →
  `attach_shadow_full`，js_dom_bridge.rs:2307-2332），但不跑分配。
- **R4 shadow 树 Text 节点不进布局**：`collect_shadow_children/collect_shadow_slot_children`
  只收 Element（tree.rs:5072-5075/:5083-5085/:5105-5107）；光树的匿名 text item/IFC
  拆分臂（tree.rs:4094-4199/:4213-4225）shadow 分支不经过。
- **R5 slotted 继承走 light 父**：光域子继承 light 父，flat-tree 语义应为 slot 元素继承
  （slotted 继承切父属 M2/M3）。
- **R6 增量样式对 shadow 全量回退**：`is_shadow_enclosed` → full recompute
  （lib.rs:732-735/:847）——正确性保守臂，性能面非本 goal 目标。

## 2. 挂接点审计（DC-1 三点位）

### 2.1 JS→Rust slot 通路

- **结构面 renderer-ready (a)**：`ShadowRoot` 为 host.children 内真节点
  （`attach_shadow_full` shadow.rs:58-62，parent=host + `Document.shadow_roots`
  侧表 shadow.rs:68）；封装靠遍历门（`is_shadow_root_node` 跳过，document/mod.rs:1933）。
  命令式路径经 mutation batch 物化真树。声明式模板物化落真树 +
  resolve_slots（parser.rs:654/:130-139）。
- **分配面 JS-only (b)**：`assignedNodes/assign()`/slotchange 全在 js_dom_shim .js
  侧（part03.js:12946+、part05.js:5620+/:6409+），Rust 无 per-node assigned-slot
  指针、无 `assignedSlots`；JS 分配结果不镜像 Rust（`DomMutation` 无分配类）；
  `AttachShadowOnHandle` 臂宽松 no-op（js_dom_bridge.rs:2333-2338）。
- **缺 (c)**：manual slotAssignment 字段声明未接线（node.rs:382 恒 Named，无 writer）；
  slot 分配表不随 bridge 拷贝（js_dom_bridge.rs:2683-2685 注释，物化路径既有 FIXME）；
  嵌套 shadow-in-shadow 分配不遍历（shadow.rs:97 FIXME）；Rust 侧无通用 flattened
  tree/composed path API（`query_selector_including_shadow` shadow.rs:98 为唯一近似）。

### 2.2 布局树挂接

- **构建入口**：`build_layout_tree`（tree.rs:719）→ `build_subtree`（:3187，
  `in_shadow` 参数 :3193）三分支 :4014/:4028/:4041；taffy 节点创建 :5028-5034，
  dom↔taffy 映射 :5037-5040；`LayoutEngine::compute_with_img_intrinsic`
  （engine.rs:553）→ `compute_layout_with_measure`（engine.rs:575）。
- **失效模型**：结构 mutation 无增量布局——`render_with_dom_mutations_persistent`
  （pipeline/mod.rs:2269）每批序列化 DOM 全量重跑 parse→style→layout→paint（:2415；
  增量臂因几何错误已撤，注释 :2354-2368）。paint 区域 dirty（`DamageTracker`
  dirty.rs:27/:43）与 style 增量回退（R6）并存。**M3 路线由此定**：分配表新鲜度
  补在 mutation apply 臂（attachShadow 臂 js_dom_bridge.rs:2327 + append/remove 臂
  补 `resolve_slots`），管线天然重建即消费，不引入增量布局。
- **绘制/几何零结构改动**：paint 纯 LayoutBox 消费（`Painter::paint` painter/mod.rs:2012，
  递归 children :3748）；gBCR（hit_test.rs:150/:359/:373 + rect_bridge.rs:98）、
  elementFromPoint（element_from_point.rs:39 → hit_test.rs:232/:680）均按 node_id
  绑定 LayoutBox——slotted light 节点保光域 NodeId（tree.rs:5087-5098），样式/矩形/
  命中自然绑光域元素，符合 spec。唯一待验点：inline 文本读 DOM 子（painter/text.rs:1421-1430）
  与 slot 替换后 DOM 子含 `<slot>` 的双计风险（未发生实证，M3 验收覆盖）。
- **同高度先例**：`::before/::after` DOM 物化（`inject_pseudo_text_nodes`
  pipeline/mod.rs:3370，style→layout 之间 :1866 调）与 `display:contents` 穿透
  （tree.rs:4099-4159）——flattened 插入的正确高度在 DOM/styles 层或
  `collect_shadow_*` 分支，taffy-local 无 slot 概念（vendored taffy 0.12，patch 面
  README.md:3-8，纯盒树）。

### 2.3 级联入口

- **匹配死点**：`matches_pseudo_class`（matcher/mod.rs:283）`Simple(name)` 臂
  :285-355，`_ => false` :354 = `:host` 死点；伪元素 `SubclassSelector::PseudoElement`
  → false（:174-177）；per-rule 入口 `collect_style_rule_decls`（:2771，匹配调
  :2783-2784）；伪元素变体 `matches_selector_for_pseudo`（:2275，剥尾伪元素重匹配
  :2302）= `::slotted` 挂点。组合子走 `doc.parent_node/previous_sibling` 光域边
  （:47 matches_selector_recursive）——slotted 祖先链需 flat-tree 语义。
- **style-system 自带 matcher**，不调 dom/query.rs——两套并行匹配器，R159 宽容记账
  在 query.rs 侧独立成立，DC-2 的「恒不匹配」翻转只在 style matcher + parser 落。
- **scope 传参缺口**：matcher 只收 `(&Document, NodeId)`，`:host`/`:host-context`
  需要 owner ShadowRoot + 光域祖先链 → scope context 需穿进 matcher。
- **host 级联缺口**：host 元素只对 document sheets 匹配——shadow sheets 里的
  `:host` 规则永不级联到 host（`collect_matching_declarations_with_media`
  :2175 只见 per-scope sheets；render 链 scope 恒空见 R1）。
- **级联序**：`CascadeOrder::sort_key`（cascade.rs:77-110）= (important, origin,
  style_attribute, unlayered, layer_idx, specificity, position)；origin 两段
  （UA=0/User=1/Author=2，cascade.rs:16-23）——`:host` shadow-author-on-host 需要
  「低于外树 author、高于继承」的树语境段，现有 position 方案无第三段
  （装配 lib.rs:2511-2535 UA/author 各自 position 从 0）。
- **css-parser 合法性表（现状）**：

| 选择器 | 解析 | 匹配 | 备注 |
|---|---|---|---|
| `:host` | 合法（is_known_simple_pseudo_class :1077） | 恒 false | matcher 无 arm |
| `:host(...)` | **非法**（不在 :1112 函数表）→ 整规则丢弃 | — | 需 AST 载 compound 参数 |
| `:host-context(...)` | 合法（:1112 表内）参数吞 | 恒 false | 死臂 |
| `::slotted(...)` | **非法**（不在 :1117-1152/:1159-1161）→ 整规则丢弃 | — | 需 Functional/载 selector list |
| `:has-slotted` | 合法（:1095） | 恒 false | 死臂（chrome 弃用，仅记账） |

- specificity（css-parser/src/selector.rs:17，pseudo-class b+=1 / pseudo-element
  c+=1）无 scoping 特例——`:host()` 参数计 (a-b) 伪类位、`::slotted` 参数计入，
  落地时随 AST 载参一并处理。

## 3. reftest corpus 盘点（渲染级验收标尺）

- **池完备性**：上游 pin 下 shadow-dom topdir + focus/ 的 reftest 形态页全量 = 本地
  20 案，无漏拉。本地另有 14 个 `-ref.html` + `reference/empty.html` 在场；共享依赖
  `/css/reference/ref-filled-green-100px-square.xht`、`/fonts/ahem.css` 在场
  （css/fonts 语料所有）。
- **基线实测**（2026-11-07，FILTER=shadow-dom：35 案子串命中含 15 案他域误中；
  shadow-dom 池 **20 案 16P/4F**）：

| 分域 | 案 | 基线 |
|---|---|---|
| B1 slot 回退/分配渲染 | slot-fallback-content-001..008 | 6/8（004 ✗1.54%、006 ✗4.83%——动态分配面，R3） |
| B2 布局失效/静态分配 | imperative-slot-layout-invalidation-001、invalidate-sibling-different-slots、layout-slot-no-longer-assigned/fallback、manual-slot-assignment-no-wrong-unassign | 5/5 |
| B3 样式失效/继承 | directionality-001/002、shadow-style-invalidation-vw-units | 2/3（vw-units ✗2.08%——R1/R2 面） |
| B4 `:host` 伪类族 | focus/focus-pseudo-on-shadow-host-1/2/3 | 2/3（-1 ✗2.08%——R2 `:host` 面） |
| B5 交互耦合 reftest | nested-hover-pseudo-class-removal | 1/1（静态面；真交互面具名挂账） |

- **形态注记**：
  - 空白合法案（负断言）3 案：layout-slot-no-longer-assigned/fallback、
    manual-slot-assignment-no-wrong-unassign——ref = `reference/empty.html`，
    近纯色旗为期望形态（「错误内容不得出现」守卫），M1+ 改动后必须恒白。
  - 交互耦合弱验案（B5）：nested-hover-pseudo-class-removal——reftest-wait +
    testdriver 悬停臂 runner 内 no-op，今日 0.00% 为**静态等价通过**，真交互面
    此 runner 不可验（具名挂账，不按真通过记账）。
  - B1 001/002/003/005/007/008 等通过案主由回退内容渲染承担（声明式解析时点分配 +
    fallback），004/006 红在命令式 attachShadow + 动态 remove/append 重分配（R3）。
- **邻域盘点（selection/shadow-dom/ 9 案，9/9 绿）**：cross-shadow-boundary-1..6、
  -img、-select-document、-select-root——attachShadow + shadow 树文本渲染 +
  selection 高亮跨边界，真 shadow 渲染耦合 reftest（FILTER=shadow-dom 子串命中即入
  runner 报表）。分母外 2 案：cross-shadow-boundary-extend（无 match 声明）、
  selection-at-nodes-not-part-of-flattened-tree（testharness 形态）；tentative/
  testharness 页归 23 号既有账。归属注记：selection 语料（23 号
  selection-compat-v2 已收口归档，脚本保留），**不并入 45 号分母、不改其既有账**；
  其 shadow 渲染消费与本 goal M1~M3 共用管线，回归防护走 rendering-compat 全量
  零回归门，M4 复测并跑对账（9/9 不回落）。渲染级 shadow 耦合 reftest 总面 =
  本 goal 池 20 案 + 邻域 9 案 = 29 案，仍 < 60，分批决策不变。
- **testharness 渲染耦合补充池（10 案，42 号 skip 规则排除归 45 号，wc_case_skipped
  testharness.rs:3279-3284 与 fetch 脚本头注释同源）**：offsetParent-across-shadow-
  boundaries、offsetTop-offsetLeft-across-shadow-boundaries、focus-within-shadow、
  scroll-restore-shadow、scroll-to-the-fragment-in-shadow-tree、
  Document-caretPositionFromPoint.tentative、DocumentOrShadowRoot-prototype-
  elementFromPoint、HighlightRegistry-highlightsFromPoint、
  MouseEvent-prototype-offsetX-offsetY、accesskey.tentative（交互面随 nested-hover
  口径挂账）。辅助标尺——M1 通道落地后进 B6 计门。

## 4. DC-4 数值门（决策规则落定）

**地址池 20 案 < 60 案 → 按子域分批验收，不足部分具名挂账**：

- 分批门值：B1 8/8、B2 5/5、B3 3/3、B4 3/3、B5 1/1（静态面），M4 收口 =
  **20/20 全绿且逐批不回落**（逐批记账进控制面；B2 三案空白合法形态恒白守卫）。
- M4 并跑对账：selection/shadow-dom 邻域 9 案 9/9 不回落（不改其归属）。
- M3+（补充池通道落地后）：B6 testharness 渲染耦合池 10 案计门，门值按通道落地
  实测回填。
- rendering-compat 全量账零回归门不变：shadow-dom 池与 selection/shadow-dom 邻域
  均已在 `make reftest` 全量内（reftest-skip-list.txt 无 shadow-dom 条目，76 行核），
  M1+ 每批合入全量零回归。
- **具名挂账（不足部分）**：
  1. 交互面真验（nested-hover 悬停臂、accesskey 激活臂）——runner 无真实输入注入
     reftest 面，静态等价通过不记真通过；回池条件 = webdriver 输入管线接 reftest 通道。
  2. hit-test 四面与 selection/Highlight registry 面交叠部分，按 M1 通道归因定档。

## 5. 边界核对（划界回写）

- **constructable stylesheets / adoptedStyleSheets**：42 号 M3 已归口 rendering-compat
  （9 文件 5P/23F 双侧留痕在案）。本 goal 的 engine adoptedStyleSheets 槽位
  （pipeline/mod.rs:3892-3896）仅收集不消费——不双改，M1 scoped-sheets 接线不含它。
- **`::part()`/`::theme`**：shadow-dom 池无 part 案，corpus 零依赖——维持范围外。
- **declarative shadow DOM**：parser/template 层 42 号已收口（setHTMLUnsafe 物化
  654P/0F）；渲染级消费经同一 `materialize_declarative_shadow_roots` 管线，两 goal
  分账维持。
- **JS 语义面**：slotchange/assignedNodes/composed path 归 42 号已收口；本 goal R3
  修复只动「渲染消费时点的分配表新鲜度」（resolve_slots 触发时机），不改 JS 语义。
- **rendering-compat 碰头面**（M1+ 触及时双方控制面留痕）：style matcher、cascade
  装配、tree.rs collect 分支、mutation apply 臂四处。

## 6. M1~M3 推进序（按审计最小改动面）

- **M1（`:host` 族 + scoped sheets 进渲染管线，DC-2 前半）**：
  1. render pipeline 换 `compute_styles_with_shadow`：`collect_shadow_scoped_sheets`
     接线进渲染链（修 @handle host 与 nested-host 两个 FIXME，pipeline/mod.rs:4053）；
  2. matcher `:host`/`:host()` arm + scope context 传参（owner root + 光域祖先链）；
  3. css-parser `:host()` 合法化 + AST 载 compound 参数（specificity 计参）；
  4. cascade 树语境段：`:host` 规则低于外树 author、高于继承（CascadeOrder 扩展）；
  5. host 元素附加 shadow-sheet 级联段。
  验收：B3 vw-units、B4 focus-pseudo-1 翻绿；B1/B2 零回落。
- **M2（`::slotted` 命中，DC-2 收口）**：
  1. css-parser `::slotted()` 合法化（Functional 伪元素或 `SubclassSelector::Slotted`
     载 selector list，specificity 计参）；
  2. matcher `::slotted` arm（`matches_selector_for_pseudo` 挂点）；
  3. 组合子 flat-tree 语义（slotted light 子祖先链 = slot → shadow host → …）；
  4. slotted 继承切 slot 父（R5）。
  验收：B1 004/006 若含 `::slotted` 断言面随 R3 修复一并翻绿；DC-2 收口单测/WPT 案。
- **M3（布局/绘制消费收口，DC-3）**：
  1. mutation apply 臂 `resolve_slots` 触发（attachShadow / childList / slot 属性，
     js_dom_bridge.rs:2327 + append/remove 臂）——B1 004/006、B2 动态面翻绿主刀；
  2. `collect_shadow_*` Text 节点通道（匿名 item/IFC 同光树处理，R4）；
  3. inline 文本 × slot 替换双计风险验证（painter/text.rs:1421-1430）；
  4. B6 补充池通道（testharness channel + Makefile target + skip 规则两侧同源）。
  验收：reftest 池逐批全绿 + 补充池基线 + painting-compat 零回归。
- **M4（复测收口，DC-4）**：20/20 逐批不回落（+邻域 9/9 并跑对账） + rendering-compat 全量零回归 +
  `make test` + 头条不回落；数值门回填控制面。

## 7. 语料预置

`tests/wpt-runner/scripts/goals/45-shadow-dom-rendering.sh`（本提交同落）：显式清单
20 案 + 14 refs + `reference/empty.html` + 补充池 10 案，fetch_raw 幂等（FORCE=1 续拉），
missing=0 断言。共享依赖（css/reference 绿方格、fonts/ahem）归 css/fonts 语料清单，
不重复拉取，脚本内断言在场。
