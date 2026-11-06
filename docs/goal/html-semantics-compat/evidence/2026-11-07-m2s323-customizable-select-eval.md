# M2 s323 — customizable-select testharness 面归属勘根（评估切片 docs-only）：五面分派，Face B+E ≈17 页本流施工立项成立，44+ 页总闸=渲染流 appearance:base-select

日期：2026-11-07。切片号 s323（承 s322 下一步候选①：customizable-select ~30 评估立项价值）。

## Solo 基线

`make testharness-html-semantics FILTER=customizable-select`（日志 evidence/logs/s323-cs-inventory.log）：

- 通道盘点（4 个 customizable-select* 目录，251 html，排除 -ref 页）：**75 testharness
  测试页** + 86 reftest 测试页 + 11 其他（manual/crash）。reftest 86 页 = s303 已移交
  渲染流面（rule 10 只记不追），本切片不涉。
- testharness 通道实测：**189 子测 38P/149F/2T，61 红/9 绿页**（70 页有结果；
  5 页无子测产出 = 2 个 selectedcontent-restore.optional + 3 个 filterable-select
  tentative 页，setup 即中止）。

## 五面分派（勘根实证）

### Face B — selectedcontent 元素族（8 页，本流施工候选①）

`<selectedcontent>` **零实现**（crates/ 全 grep 零命中，解析为未知元素空 innerHTML）。
测试断言语义面（selectedcontent.html 实证）：

- 初始 innerHTML = 选中 option 内容克隆（快照语义）
- select.value 变更/重赋 → 重克隆；option 后代文本/元素/属性变更**不**响应
- 多 selectedcontent 元素同步保持；断连 select 行为；selectedcontentelement
  属性关联（selectedcontentelement-attr.tentative）；insertion/removal steps、
  moveBefore、嵌套、listbox/multiple 变体

spec：HTML Standard #the-selectedcontent-element（whatwg/html #9799 谱系）。
页面无 appearance guard 依赖（失败全为语义断言）。

### Face E — select 解析定制 + option 收集族（~9 页，本流施工候选②）

customizable-select 解析定制整体缺失（zero-dom parser 无 select 插入模式特殊
处理，grep 实证）。逐页实证：

- select-parsing：div/button/datalist 于 select 内**被剥壳**（文本保留、元素
  丢弃；expected `<div>div1</div>…<datalist><option>…` got `div1buttondiv2
  <option>…`）；`</select>` 不闭 open button/datalist；select-in-button-in-
  select 不移除内层 select
- select-options-id：wrapper div 后代 option 不入 select.length（6 vs 5）
- nested-options：datalist 于 select 内整体丢弃 + option 不自动闭合 +
  options IDL 不排嵌套（2 vs 1）
- option-form-ancestor-select：option.form 穿无效嵌套（datalist）误关联 form
- select-button-behaviors：`querySelector('select…button')` null → parser 面
  （button 于 select 被剥壳）
- select-input-parsing.tentative：input 于 select 解析定制

### Face A — :open / picker 状态机（16+ 页，跨流三段门，挂协付点）

- **A2 门（渲染流）**：`appearance: base-select` 计算值——`base-select` 于
  css-parser/style-system **零命中**（grep 实证；appearance 属性名已知，
  UA decls 用 button/itype 值）。11 页显式 guard 失败（两种措辞：
  "This test requires appearance:base-select" ×6 子测 / "appearance:base-select
  must be supported" —— button-in-popover、select-disabled、select-pseudo-open、
  select-synthetic-events、select-optgroup-arrow-keys、select-option-hover-styles、
  select-picker-hover-active-pseudo、select-click-drag-option、select-iterate-
  before-beginning、select-multiple-popup-{keyboard,mouse}-behavior）。
- **A1 门（本流）**：`:open` 查询臂——14 页页级 SyntaxError（':open' is not a
  valid selector；query.rs PseudoClass 无 Open 变体，s320 协付点承继）。本流
  可单侧施工语法臂，但断言消费面仍需 A2+A3（picker 开合态）→ 单侧无案可绿。
- **A3 门（本流）**：picker 状态机（showPicker/open/light dismiss/option 选择/
  键盘鼠标行为）——被 A2 压住，镜像 :modal 通道（inject_modal_state）可迁移。
- 44+ 页（Face A+C+D 合计）解锁总闸 = 渲染流 A2 交付。

### Face C — 键盘/鼠标行为族（~10 页，A 的行为消费面）

keyboard/mouse-behavior、typeahead、home-end-pagedown、arrow-keys、click-drag、
vertical-writing-mode-navigation 等——语义本体在本流（shim 行为机），
被 A2 guard/A1 SyntaxError 压住，与 Face A 同链。

### Face D — 渲染外观面（~13 页，移交记录）

appearance 计算值（switch-picker-appearance）、base-appearance-{inheritance,
computed-style}、::checkmark/::picker-icon 动画、picker top-layer 动画/
exit-animation/@starting-style、hover-styles、accessibility-minimum-target-size、
picker-popover-open-invalid（::picker(select):popover-open 语法有效性 =
css-parser 面，R5359 workfront 谱系）。全部 css-parser/style-system/render-
foundation 面（rule 9），移交记录不改对端控制面（rule 10）。

### Face F — filterable-select（3+2 页，低优先暂记）

:filtered 伪类/beforefilter 事件/filtering 行为，全 tentative，3 页无子测
产出（setup 中止）。独立小面，暂不立项，出现钉形再评估。

### 已收编对照面（防止重复施工）

select.value/selectedOption IDL 核心 6 子测 pass；option-list 族 5 页全绿；
select-multiple-removed/size-change/tabindex-order/anchor-regression 全绿。
customizable-select 的 DOM 语义基础面部分在位（前轮切片收编）。

## 立项价值定谳

**成立**：Face B+E ≈ 17 页为本流清晰施工面（spec 钉形 + 零实现 grep 实证 +
无解锁依赖），预估 2-3 切片（B：selectedcontent 元素语义 1-2 片；E：select
解析定制 1 片）。Face A 44+ 页为渲染流 A2 交付后的高价值跟进面（A1+A3 本流
预施工可先行挂账但不单独立项，避免无案可绿的空转片——s320 先例）。协付点：
渲染流 css-parser/style-system `appearance: base-select` 值文法+计算值。

## 下一步

1. s324 = Face B selectedcontent 元素族施工（selectedcontent.html 主钉 +
   insertion-removal/mutations/movebefore 触发点，parser 容许性随钉面验证）。
2. s325 = Face E select 解析定制族施工。
3. Face A 协付点挂账（待渲染流 A2）；Face D 移交记录已落本账本。
