# M2 s326 — Face F filterable select `:filtered`/beforefilter 落地：filterable-select 3P/9F→5P/7F，select-beforefilter 全绿（主目标）；display/keyboard/datalist 三面按 A1/A2/C 挡口定性不可达

日期：2026-11-07。切片号 s326（承 s323 五面分派末位「F filterable-select 3+2 页低优先暂记」）。

## 面与基线

Face F：filterable select（whatwg/html #12050 谱系， customizable select 的过滤查询
面）。语料 `tests/wpt-runner/wpt-data/html/semantics/forms/the-select-element/filterable-select/`
（s323 勘根批带入，gitignored 本地数据）三页：

- `select-beforefilter.tentative.html` 1 子测——beforefilter 事件分发/可取消，**完全可达**（本切片主绿目标）
- `filterable-select-filtered-pseudo.tentative.html` 8 子测——`:filtered` DOM 查询 +
  display 呈现断言（display 面被 R1679 挡，见不可达定性）
- `filterable-select-keyboard-behavior.tentative.html` 4 子测——键盘行为
  （`:active-option`/`:open`/gBCR = Face A/C 挡口）

另 `customizable-combobox/beforefilter.html` 12 子测：`filter="datalist"` + `:popover-open`
用法，datalist 关联面 + popover 呈现面（A/C 挡口），仅作回归不施工。

基线（r1）**filterable-select 3P/9F、combobox 0P/12F**。r1 的 3P 全部 vacuous：html5ever
0.38 input-pops-select 使嵌套容器 `select.options` 为空（r1 时 `:filtered` 选择器本身
还报 SyntaxError，可过测的只有空 options 空转臂）。

## 轮次进展（日志 evidence/logs/s326-fs-r{1..4}.log、s326-cb-r{1,2}.log）

| 轮 | filterable-select | 变化 |
|----|------------------|------|
| r1 | 3P/9F | 基线（`:filtered` → SyntaxError 面） |
| r2 | 3P/9F | css-parser `is_known_simple_pseudo_class` 加 `filtered`——**未动**：`__zw_selector_valid` 走 dom 自带 `parse_pseudo`，与 css-parser 是两套解析器 |
| r3 | 4P/8F | dom 侧 `PseudoClass::Filtered` 机器（enum/parse/matches_full 延迟真）→ SyntaxError 面消除，sibling-dropdown-unsupported 绿 |
| r4 | 5P/7F | `:not(:filtered)` 否定嵌套修复（Not 延迟 + re-eval 守卫臂）→ **select-beforefilter 绿（主目标）** |

r4 终态 **5P/7F**，12 子测逐条归因：

- 5 Pass = 3 vacuous（r1 已过：nested listbox ×2 + nested dropdown case folding）+
  sibling-dropdown-unsupported（r3 新绿：listbox gate 正确把 size=1 dropdown 挡在
  过滤外）+ select-beforefilter（r4 新绿）
- 7 Fail = sibling idref ×2（display 面，见不可达）+ nested dropdown behavior
  （`':open' is not a valid selector`，A2 面）+ keyboard ×4（`':active-option' is not
  a valid selector`，C 面）

combobox 两轮回归 0P/12F 零位移（r2 带过滤机器后复验，符合 datalist/popover 面不可达定性）。

## 施工面（Rust ×6 + JS shim ×2，8 文件）

### `:filtered` 两级解析闸 + 状态语义（dom/css-parser/style-system）

- `crates/css-parser/src/parser/mod.rs`：`is_known_simple_pseudo_class` 加 `"filtered"`
  （样式侧解析有效性，#12050 注）。
- `crates/dom/src/query.rs`：`PseudoClass::Filtered` 变体 + `parse_pseudo` 臂 +
  `matches_full` 返回 true（延迟真——与 `:active` 同款两阶段模型）。
- `crates/dom/src/document/filtered.rs`（新）：`is_filtered_option` 查询，镜像
  `modal.rs` 模式；模块文档引 #12050 与同步通道。
- `crates/dom/src/document/mod.rs`：`filtered_options: Vec<NodeId>` 字段（双初始化器）
  + `set_filtered_options` setter + `element_matches_selector` re-eval 环 `Filtered` 臂
  与 `:not(:filtered)` 守卫臂（见下）；FIXME 记样式面挂账。
- `crates/dom/src/document/snapshot.rs`：pristine_snapshot 深拷 `filtered_options`
  （运行时注入态，与 `focus_element`/`modal_dialogs` 同组）。
- `crates/style-system/src/matcher/mod.rs`：matcher 臂 `"filtered" =>
  doc.is_filtered_option(element)` + `SUPPORTED_SIMPLE_PSEUDO_CLASSES` 登记。

### `:not(:filtered)` 否定嵌套（r4 修复，blast radius 锁 Filtered）

dom 两阶段模型既有「`:not(:active)` 否定嵌套局限」（Not(inner) 对含延迟真伪类的
inner 整体延迟为 true）。s326 只为 Filtered 收口：`matches_full` 的 Not 臂当 inner
含 Filtered 时延迟（其余伪类语义不变）；`element_matches_selector` re-eval 环加守卫臂
`Not(inner) if inner 含 Filtered` => `!(static_hit && is_filtered_option(node))`——
static 部分照常求值，仅把 Filtered 从延迟真换成文档态真值。

### 过滤态同步通道（engine 桥）

- `crates/engine/src/js_dom_bridge.rs`：`DOC_FILTERED_SELECTORS` thread_local（选择器
  串槽）+ `sync_filtered_state`（`find_by_selector` 逐条求值 → `set_filtered_options`；
  槽空零成本直返）。**折叠进 `sync_active_state` 尾部**——active/filtered 共用单漏斗，
  live doc/view cache/string re-parse/matches/closest 既有调用点全部继承；
  `parse_html_element_json_full`（_zwMQueryAll/matches 宿侧重解析路径）另显式调用。
- `crates/engine/src/js_dom_bridge/callbacks.rs`：`__zw_set_doc_filtered` 回调——单参
  `\u{1f}` 连接的选择器列表，空串清槽（serde_json 非引擎依赖，故用分隔串）。

### 过滤重算 + beforefilter（JS shim）

- `crates/engine/src/js_dom_shim/part03.js`：`_zwFilterableRecompute(inputEl)`——关联
  解析（①sibling idref：`filter` attr = select id，listbox gate = `multiple` attr 或
  `size≠1`；②nested：input.parentElement 为 select）→ 分发 `beforefilter`
  （bubbles+cancelable，preventDefault 即保持现状）→ 逐 option `textContent`
  vs `input.value` 双侧 toLowerCase 子串匹配（覆盖语料 ç/Ç ÿ/Ÿ ƈ/Ƈ ꜧ/Ꜧ 𐐨/𐐀）→
  命中 option 的 `_zwActiveElSel` 选择器写入槽。无关联 input 不写槽（零税 + 多输入
  状态保持）。FIXME 记多输入真相合并挂账。
- `crates/engine/src/js_dom_shim/part04.js`：input value setter（IDL set 与键入同臂）
  尾部调 `_zwFilterableRecompute`（R2933 通用 on* 机制使 `input.onbeforefilter = fn`
  零注册工作可用；Event 构造器已认 bubbles/cancelable）。

## 不可达定性（余 7F 的挡口，非本切片缺口）

- **sibling idref ×2**（filtered-pseudo behavior/case folding）：测试前置断言
  `option one should not be display:none got "none"`——R1679
  （`ua_default_display("option") = None`，ZW_SELECT_SUPPRESS_OPTIONS）使全 option
  display:none，语料的「未过滤 option 不隐藏」呈现断言需 A2
  （appearance:base-select listbox 呈现）交付后重访。`:filtered` 的 DOM 查询面本身
  已按本切片语义工作。
- **nested dropdown behavior ×1**：测试用 `:open`（A2 挡口 SyntaxError）。
- **keyboard ×4**：测试用 `:active-option`（Face C 挡口 SyntaxError；`:open`/gBCR
  同属 A/C）。

combobox-beforefilter 12F：`filter="datalist"` 关联 + `:popover-open` 呈现，均 A/C
面，不在本切片施工范围。

## 挂账

- **多输入真相合并**：多个关联 input 同写一个槽时后者覆盖前者；规范语义是并集。
  语料单输入页钉不到，留 FIXME（part03 `_zwFilterableRecompute`）。
- **样式面注入**：UA `option:filtered { display:none }` 呈现 + getComputedStyle 同步
  ——镜像 `:active` 的渲染样式面挂账先例（FIXME 记 dom 字段注释），待渲染流 A2
  appearance:base-select 交付后镜像 `inject_focus_state` 逐帧注入。
- combobox datalist 关联面（A/C 交付后）。

## 验证

- `make testharness-html-semantics FILTER=filterable-select` r4 终态 **5P/7F**（12 子测
  逐条归因见上）；`FILTER=customizable-combobox` 0P/12F 两轮零位移。
- **回归**：`FILTER=customizable-select` 全域与 s325 r7 日志 Pass/Fail 行集**逐行 diff
  零位移**（裸 grep 74P/147F 含 gitignored 探针页 2 页 24 子测 assert_false 设计——
  即 s325 real 72P/125F 口径）；`FILTER=customizable-combobox` 0P/12F 零位移。
- `make test` 全量 **3787P/9F**（9F=对端 js_dom_bridge 在册红腿：stylesheet 规则文本
  序列化 ×5 / fetch response type / websocket ×2 / service worker / adopted snapshot，
  与 s324/s325 账本同类同数，零新增）；`make reftest` **725/725**（Layout 521/521 +
  Text 204/204，exit 0）；`cargo clippy -p zero-dom -p zero-css-parser
  -p zero-style-system -p zero-engine --all-targets -D warnings` PASS；
  `cargo fmt --check` PASS（一处守卫臂行超宽由 rustfmt 归一）。
  （make test 全量 ~40 分钟，setsid nohup 后台跑，日志 /tmp 落盘后核对。）
