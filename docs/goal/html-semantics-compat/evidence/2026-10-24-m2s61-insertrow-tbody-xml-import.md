# M2 切片 61（s61）——tabular-data 域：insertRow tbody 目标 + XML 导入链收口

日期：2026-10-24。tabular-data 域 testharness 127P/7F → **130P/4F**（GAIN 3 / REG 0）：table-insertRow 2F 全绿（thead/tfoot-only 表新建 tbody 案）、tBodies 1F 全绿（XML documentElement.getElementsByTagName + importNode 深导入案）。interactive-elements 全域 439P/44F/0T 复跑逐值不变；testharness-dom 套件超时另验（见验证节）；zero-engine lib 见文末。

## 修复面

### 1. table.insertRow append 位 = 最后 tbody，无 tbody 则新建（part03，R5268）

旧实现 `hasSection`（tBodies[0] || tHead || tFoot）分支：rows 空、存在 thead/tfoot 时新行**直挂 table**（`this.appendChild(tr)`）。spec 要求 append 位是**最后一个 tbody 元素子**；无 tbody（thead/tfoot 不是插入目标）则新建 tbody 挂 table 再入行。

- WPT table-insertRow 案 2/3（thead/tfoot-only 表 insertRow(-1) 期望 `table.childNodes === [head, foot 之外的新建 body]`）实证实修。
- 修法：删 `firstTbody`/`hasSection` 与 rows 空早退分支，append 臂统一 `_zwTblChildIdxLast(this,'tbody')` → 命中取 `childNodes[idx]`，未命中 `document.createElement('tbody')` + appendChild 后入行。连带收益：多 tbody 下 append 位从「最后有行的 tbody」纠正为「最后 tbody 元素子」（spec 口径，probe 第 4 案钉定）。
- 范围内行为变化：rows 非空 + 无 tbody（DOM 拼装的直挂 tr 表）append 时新行入**新建 tbody** 而非直挂 table——spec「no tbody descendants → create」分支，无测试钉定旧形态，随统一分支自然纠正。

### 2. 快照元素级 getElementsByTagName（part02，R5268）

`_zwParseEl`（DOMParser 解析产物的只读元素面）缺 `getElementsByTagName`（仅 doc 级 `_zwParsedDoc.getElementsByTagName` 有）——`doc.documentElement.getElementsByTagName('table')` 抛 `is not a function`。与 doc 级同口径委托 `querySelectorAll(String(tag))`（outerHTML 重解析 + R188 镜像剔除 + R158 缓存），一行委托无新状态。

### 3. importNode 快照域产物物化为 handle 树（part06，R5268）

tBodies 案第二级缺口：`document.importNode(table, true)` 经 `_zwParseEl.cloneNode` 产出 raw `_zwMEl` 副本，append 进活树后 `landed === 原对象`、`.tBodies` 读 undefined（probe s61b C 实证）——快照产物无元素 proxy trap 面。

- spec importNode = clone + adopt，产物须落目标文档完整接口面。t8q adoptNode 已有同判据物化先例（`_zwParseEl`/`__zwIsMEl` → `_wcRebuildAsHandle`），importNode 补同款：**deep 且源为快照域产物**时物化返回 handle 树。
- 不进本分支：真 wrapper 源（`__zwHandle`/`__zwSelector`）零变化；浅克隆不物化（child 剥离语义会丢）；t8i template content 视图自导入标记在更早分支已返回。

## 验证

- probe（wpt-data 树内 zz-probe61.html，验证后已删）：xml parse + query + importNode 形态 / append 落地身份 / landed tBodies 面 3/3 绿；insertRow thead/tfoot-only/最后 tbody 3/3 绿
- table-insertRow 3/3；tBodies 1/1
- tabular-data 域 **130P/4F/0T**（GAIN 3 / REG 0；余 4F = processing-model rowspan-0×2 + col-span-limits×2 挂账）
- interactive-elements 全域 **439P/44F/0T** 逐值不变
- testharness-dom 套件（importNode/DOMParser 直接 WPT 面）超出前台 10 分钟墙钟未跑完——lib 桥测（R132/R2818/R3243 族）为等效回归面，见下
- 拼接 shim node --check 通过；fmt/clippy 不适用（diff 仅 JS shim）
- zero-engine lib **3454P/0F**（R132/R2818 importNode 面 + R3243 表格族零回归）

## 未解决项（tabular-data 余 4F 挂账序）

- **rowspan-0 / col-span-limits 4F**：处理模型，待分诊
- cells/tBodies HTMLCollection 类型一致化（无测试断言，低优先挂账）
- 跨流不变式：interactive-elements 44F 全为渲染流几何/UA/绘制门 ~21F（按名取案见 s57 账本）+ shadow 底座 20F + divergence 1F；parser form pointer 4F 用户缓行
