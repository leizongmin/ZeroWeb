# M2 切片 59（s59）——tabular-data 域：tr 族索引/集合读侧 childNodes 化收口

日期：2026-10-24。tabular-data 域 testharness 94P/40F → **122P/12F**（GAIN 28 / REG 0）：tr 族 21F（sectionRowIndex 8 / insertCell 5 / rowIndex 4 / cellIndex 3 / cells 1）全绿，连带 remove-row 3F、thead/tbody/tfoot rows 3F、table-insertRow 1F 收口。interactive-elements 全域 439P/44F/0T 复跑逐值不变。

## 修复面

### 1. 根因：selector 查询三缺陷（part03）

tr 族全部索引/集合 getter（rowIndex/cellIndex/sectionRowIndex/rows/tBodies/cells）旧实现经 `_wrapSelector(...).querySelectorAll(...)`：① **detached script-created 子树不可解析**（selector 世界按 document 锚定）恒空——rowIndex.html 12 案全 detached、sectionRowIndex mkTrElm 11 案全 detached；② 匹配**全部后代**非直接子；③ **无 namespace 过滤**。统一换 childNodes 直接子扫描。

### 2. 新共用地基（part03，`_zwTblChildIdxLast` 后）

- `_zwIsHtmlNsEl`：**严格 HTML-ns 判定**（namespaceURI 恒等 xhtml 串）。探针 zz-probe59 实证：createElementNS("", x) 产物经 part04 namespaceURI getter（`_nsHandles` 印记直通）读 **null**（非 ''），解析树/createElement 产物读 xhtml 串——rowIndex 7 个 null-ns 案（期望 -1）+ cellIndex「not a HTML tr」案 + cells foo-ns td 排除案全靠此判定。区别于 `_zwTblChildIdx` 宽松口径（null/'' 记 HTML，s58 create/delete 族在用不动）。
- `_zwTblChildAll(proxy, tagA, tagB)`：直接子收集（nodeType 1 + localName ∈ {tagA, tagB} + HTML-ns；保 childNodes 树序）。
- `_zwTableRowsAll(tableProxy)`：spec dom-table-rows 过滤器 + 排序——thead 行（树序）→ table 直接 tr 子 + tbody 行（树序一趟）→ tfoot 行（树序）。
- `_zwIdxOfSelf`：身份计位（childNodes 产物为缓存 proxy，appendChild 返回值 === 槽位探针实证；selector+handle 串齐等兜底）。

### 3. 六 getter 重写（part03，R2842/R2843/R3243 面 rowIndex 选项=index 分支不动）

- `rowIndex`：spec 门槛 **parent**（HTML table）或 parent 为 HTML section 且 **grandparent** 为 HTML table——非「最近祖先」链（table>div>tr 案 -1）。
- `sectionRowIndex`：parent HTML table（取 table rows 集合）或 HTML section（取直接 tr 子）；其余 -1。
- `cellIndex`：parent HTML tr → 直接 td/th 子集合计位（非 td/th 兄弟跳过自然达成）。
- `rows`：TABLE → `_zwTableRowsAll`；section → 直接 tr 子。
- `tBodies`/`cells`：直接 tbody / td+th 子。
- detached created（handle-only）路径全覆盖（`sel ? _wrapSelector : _wrapHandle`，s58 R5265 同款）。

### 4. has trap 名册（part05，R5249 tag-gate IIFE 模式）

`"cellIndex" in th` 等恒 false（get trap 供给面在 target 无 own key）——补 cellIndex（TD/TH）、rowIndex/sectionRowIndex/cells（TR）、rows（TABLE/THEAD/TBODY/TFOOT）、tBodies（TABLE）名册，':' 后缀容差。

### 5. 连带 lib 测试契约修补（js_dom_bridge_tests/part15.rs，R3243 三测试）

rows/cells 读侧走 childNodes 融合视图后，测试**换代点只 bump 宿主代际未调 shim 侧挂钩**的缺口显形：跨场景残留 pending 桶（s1 的 t>tbody 与 s2 的 t2>tbody 结构 sel 同串）污染读取（探针实证 t2.rows len=5 含 ①-③ 建行）。按 part12/part38/part41 既有契约（`__zw_reset_pending_state` = SetDomSnapshot 挂钩）补调用；同场景续跑的两步（deleteRow 省略参 / deleteCell(-1)）先重播种 pristine 快照（活视图末元素已非测试意图目标）。测试期望本身是宿主纯净视图模型，非 spec 活语义断言——WPT 侧（行为权威）同路径全绿。

## 验证

- tabular-data 域 **94P/40F → 122P/12F/0T**（GAIN 28 / REG 0）；五目标文件 45 断言全绿（sectionRowIndex 19、insertCell 7、rowIndex 12、cells 1、cellIndex 6）
- interactive-elements 全域 **439P/44F/0T** 逐值不变（REG 0）
- zero-engine lib **3449P/0F**（r3243 三测试修复后全绿）
- fmt --check 干净；clippy -p zero-engine 无告警；拼接 shim node --check 通过（build 序 13 段）

## 未解决项（tabular-data 余 12F 挂账序）

- **table-rows 5F**：instanceof HTMLCollection——读侧类型重做（参照 R3908 selectedOptions 形模式）
- **table-insertRow 2F**：insert into a tbody, not into thead/tfoot（无 tr 时插入位语义）
- **tBodies 1F**：DOMParser XML documentElement.getElementsByTagName 缺
- **rowspan-0 / col-span-limits 4F**：处理模型，待分诊
- 跨流不变式：interactive-elements 44F 全为渲染流几何/UA/绘制门 ~21F（按名取案见 s57 账本）+ shadow 底座 20F + divergence 1F；parser form pointer 4F 用户缓行
