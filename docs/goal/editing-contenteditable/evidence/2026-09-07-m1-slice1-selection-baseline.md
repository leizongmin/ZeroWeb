# M1 切片 1 — WPT selection/ 基线（2026-09-07）

**用例来源**：上游 WPT `web-platform-tests/wpt` @ `315976933870b34d6ea30e3f6643403edae678ba`
（与 fetch-dom-subset.sh 同 pin），`fetch-selection-subset.sh` 拉取到 `wpt-data/selection/`
（gitignored）。20 用例 + 1 helper（common.js）。

**导入清单**（selection/ 根目录主线程 .html）：
getSelection / addRange / collapse / collapseToStartEnd / getRangeAt / isCollapsed /
removeAllRanges / removeRange / selectAllChildren / setBaseAndExtent /
deleteFromDocument / deleteFromDocument-HTMLDetails / type / anchor-removal /
script-and-style-elements / toString-ff-bug-001 / extend-exception / Document-open /
onselectionchange-on-document / onselectionchange-on-distinct-text-controls

**排除项**（有据，非静默）：
- `dir-manual.html` — manual 交互（真拖拽），headless 不可执行
- `*-repaint*` / `canvas-click/drag` — 渲染域断言（非 Selection API 面）
- `modify*.tentative.html` — `selection.modify()` 未定规范 + shim 无该面
- `caret/` `textcontrols/` `contenteditable/` `bidi/` `shadow-dom/` `anonymous/` 子目录 — 后续切片
- `addRange-*.html` 数字系列（00-56）— 依赖 `common.js` 的 assert_selection 基建与真渲染选区，headless 首批不导
- `editing/include/editor-test-utils.js` 缺失 → `deleteFromDocument-HTMLDetails.html` Fail（外部依赖，后续切片补拉）

**执行入口**：`make testharness-selection`（test-guard 包裹；`FILTER=<子串>` 透传）

## 基线（首轮，2026-09-07）

```
subtest total: 45 Pass / 2559 Fail
pass-rate: 1.7%
```

| 用例 | P | F | 主要失败根因 |
|---|---|---|---|
| getSelection.html | 1 | 17 | document.getSelection 未绑定；Selection 构造器 instanceof 断；iframe 面 |
| addRange.htm | 1 | 0 | ✅ 全过 |
| collapse.htm | 0 | 1 | collapse() 后 toString 未清空（range collapse 语义） |
| collapseToStartEnd.html | 4 | 53 | 空 selection 调用须抛 InvalidStateError |
| deleteFromDocument.html | 0 | 60 | 方法未实现（`detachedComment.firstChild` null——detached 节点面） |
| getRangeAt.html | 2 | 2 | 越界抛 IndexSizeError 部分缺 |
| isCollapsed.html | 17 | 12 | setBaseAndExtent 未实现（依赖面） |
| removeAllRanges.html | 0 | 1 | 依赖 createRange 面缺口 |
| removeRange.html | 0 | 29 | 越界/不存在参数抛 TypeError 语义 |
| selectAllChildren.html | 0 | 2242 | 方法未实现 |
| setBaseAndExtent.html | 2 | 118 | 方法未实现 |
| type.html | 17 | 12 | 同 setBaseAndExtent 依赖 |
| extend-exception.html | 0 | 1 | 空 selection extend() 须抛 InvalidStateError |
| script-and-style-elements.html | 0 | 1 | toString 对 display:block 的 style/script 文本收集差异 |
| anchor-removal.html | 0 | 2 | 用例自身 helper 未定义（`parentParagraph`——`common.js` 变量面） |
| Document-open.html | 0 | 1 | iframe.contentWindow.getSelection 未绑定 |
| onselectionchange-on-document.html | 0 | 4 | setPosition 别名缺失 + selectionchange 派发未接 |
| onselectionchange-on-distinct-text-controls.html | 0 | 2 | 文本控件 selectionchange 派发未接 |

## 失败聚类 → 修复队列（M1 切片 2）

1. **`document.getSelection` / `iframe.contentWindow.getSelection` 绑定**（getSelection
   / Document-open，~22F）——spec：`Document.getSelection()` 与 window 同一 Selection。
2. **`Selection` 构造器 + instanceof 链**（getSelection，若干 F）——`window.getSelection()
   instanceof Selection` 须真；shim `globalThis.Selection` 为空函数 + selection 字面量未接原型。
3. **`selectAllChildren` / `setBaseAndExtent` / `deleteFromDocument` / `setPosition`
   （=collapse 别名）方法缺失**（selectAllChildren/setBaseAndExtent/type/isCollapsed/
   deleteFromDocument/onselectionchange-on-document，~2450F）。
4. **空 selection 的 `collapseToStart/collapseToEnd/extend/getRangeAt` 抛 InvalidStateError /
   IndexSizeError**（collapseToStartEnd/extend-exception/getRangeAt/removeRange，~85F）。
5. **collapse() 语义**（collapse.htm）——collapse 到点后 toString 须空。
6. **selectionchange 事件派发**（onselectionchange-*，6F）——文本控件/document 面。

---

# M1 切片 3 — editing/ 首批导入（2026-09-07，同日追加）

**新增**：`editing/event.html`（beforeinput/input 事件面，180 subtests）+
`editing/other/delete-editing-host.html`、`editing/other/body-should-not-deleted-even-if-empty.html`
（execCommand delete 编辑宿主语义——M2/M3 前置基线）。

**首批基线**：

| 用例 | P | F | 聚类 |
|---|---|---|---|
| editing/event.html | 104 | 76 | 76F 全是「number of input events fired expected 1 but got 0」——execCommand format 类命令不派发 input 事件（E4/E5 缺口，M2/M3 领域） |
| editing/other/delete-editing-host.html | 2 | 0 | ✅ 全过（execCommand delete no-op 语义 + isConnected 断言） |
| editing/other/body-should-not-deleted-even-if-empty.html | 0 | 2 | execCommand delete 实应用缺失（M2 领域） |

**组合面**（selection 20 用例 + editing 3 用例）：23 用例 1930P/854F。

**排除项**（有据）：`editing/run/`（43 用例 execCommand 全命令面——M3 里程碑领域）；
`editing/other/` 其余 ~95 案（按 M2 失败聚类逐批追加）；`editing/manual/`（真交互）；
`editing/include/editor-test-utils.js` 依赖面（后续切片补拉）。

---

# M2 切片 1 — execCommand editing-host 事件序（2026-09-07）

**修复**（commit 74a0c879b，js_dom_shim/part06.js）：
editing/event.html 76F 聚类（全为「number of input events fired expected 1 but got 0」）
→ execCommand 编辑类命令选区落 editing host 时派发 beforeinput（cancelable、trusted）+
input（bubbles、cancelable=false、trusted），inputType 规范映射。

**关键落点**：
- editing host 判定 = startContainer 祖先链首个 contenteditable 元素，**存在即判**
  （空串 ≡ true——首版只比对 'true' 值漏布尔形态，probe 实证修正）
- 全选区在单一 host 内才应用（end 越出 → 0 事件）
- input target 按 input 派发时刻选区现解析（handler 改选区语义）
- beforeinput preventDefault 阻断 input
- insertText/insertHTML：activeElement 为 text control 走 R57 控件路径，否则落
  editing-host 分支
- undo/redo/styleWithCSS/useCSS/selectAll 不派 editing-host 事件（WPT target=null 断言）

**结果**：event.html 104P/76F → **179P/1F（99.4%）**；selection 面 1824P 零回归；
组合 2005P/779F。

**残余 1F**：`Changing selection from handler: input event`——target 身份比较
（expected/got 均 [object Object]）。上游用例的 finalTarget 断言（second host）依赖
浏览器特有行为路径，上游 dashboard 该案形态属 legacy（「Thanks, Gecko.」注释段），
记录不追。

**单测**：`test_execcommand_editing_host_events_r3254_m2`（六组：事件序+属性/
布尔属性形态/越界 no-op/无 host no-op/undo 表外/preventDefault 阻断）。
