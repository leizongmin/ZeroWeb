# M2 切片 58（s58）——tabular-data 域：R5265 create/delete 派生族 + caption/tHead/tFoot setter 族收口

日期：2026-10-24。tabular-data 域 testharness 67P/67F → **94P/40F**（GAIN 27 / REG 0），interactive-elements 全域 439P/44F/0T 复跑逐值不变。

## 修复面

### 1. table create/delete 派生族（part03，element proxy set/get trap 方法分支）

- `createTBody`：插**最后一个 HTML-ns tbody 元素子之后**（`_zwTblChildIdxLast` 倒序扫描；嵌套 tbody 不作锚）——createTBody.html 15 案钉定（[before, tbody, after] 顺序断言）。
- `createTHead`：有守卫（已有直接子 thead 返回既有）；插入位 = 首个 tbody 子前，无 tbody 则首个 tfoot 子前，再无则 append。
- `createTFoot`：有守卫；末尾 append。
- `createCaption`：有守卫；插为**首子**。
- `deleteCaption`/`deleteTHead`/`deleteTFoot`：删首个匹配直接子（`victim.remove()`），无匹配静默。
- 门与读路统一 `_realTag(sel, handle)` + `.split(':').pop()` 取后缀——createElementNS 前缀产物（'FOO:TABLE'）放行（createTBody prefixed 案）；namespaceURI 判 HTML-ns 沿 part03:1573 口径（null/''/xhtml）。

### 2. caption/tHead/tFoot IDL getter 收紧（part03）

getter 从「首个匹配子」改 `_zwTblChildIdx` 精确索引读（caption_001「caption IDL attribute is null」等 4 案）；getter 内非方法上下文用 `_wrapSelector(sel)`/`_wrapHandle(handle)` 自代理（getter 分支 `this` 非 proxy，R2845 写法有因）。

### 3. caption/tHead/tFoot IDL setter（part05 set trap 新分支）

- `= null/undefined` → 移除首个匹配直接子（t.tHead=null 案）。
- 非 caption/section 形态 → JS TypeError（assert_throws_js 案）；section 家族但 localName 不符 → HierarchyRequestError（tbody 入 tHead / thead 入 tFoot）；node 为本表祖先 → pre-insert HierarchyRequestError（t2.tHead=t2thead 案）。
- 插入位：caption = 移除旧 + **首子**（caption_001 firstChild 断言钉定，非原位替换）；thead/tfoot 有既有 = 先移除旧子再按 `childNodes[have+1]` 原位插（防 selector 重解析误删同形新子，zz-probe13 实证）；thead 无既有 = before tbody/tfoot；tfoot = append。
- set trap 签名补 `receiver`（part04）：setter 内自代理优先取 receiver（页面持有 proxy 本体，insert 经其记 `_zwNodeParent` 反链），兜底 `_makeProxy`。

### 4. 本轮 bug 两个（同 commit 修）

- **caption 插入段永不可达**：首版类型检查用嵌套 if/else 链，caption 臂真后链直接结束，插入段困在末臂 else——仅 thead/tfoot 可达。诊断链：先误诊旧子后置 remove 歧义 → 换 _makeProxy → 加 receiver 均无效；zz-probe17 insertBefore 入口插桩实锤 setter 路径从未调 insertBefore，分型证据（HierarchyRequestError 祖先案走 else 链正常而 caption 不工作）指向控制流。修：卫语句式类型检查（不满足即 throw）+ 平铺共享插入段。
- **createTBody 插入位置错**：12/15 案 assert_array_equals 失败，两侧打印相同 `<tbody>`——真因 spec 语义「插最后 tbody 后」非「首 tbody 前」，identity 断言失败消息无法区分身份/顺序问题，逐因子探针（12 个）排除后插桩抓到 `at:0` 实锤。教训：同型元素 identity 断言失败优先怀疑顺序而非包装。

## 验证

- tabular-data 域 **67P/67F → 94P/40F**（GAIN 27 / REG 0）：createTBody 15/15、caption_001 5/5、tHead 3/3、tFoot 2/2 全绿；table-insertRow 前缀案由 "is not a function" 进到 rows 数组断言面
- interactive-elements 全域 98 files **439P/44F/0T** 复跑逐值不变（REG 0）
- zero-engine lib **3448P/0F**
- fmt/clippy 不适用（diff 仅 JS shim）

## 未解决项（tabular-data 余 40F 挂账序）

- **tr 族 21F**（下一切片）：sectionRowIndex 8 / insertCell 5 / rowIndex 4 / cellIndex 3 / cells 1
- **table-rows 5F**：instanceof HTMLCollection——读侧类型重做（参照 R3908 selectedOptions 形模式）
- **remove-row 3F + thead/tbody/tfoot rows 3F**：rows live-sync / created section 读路径
- **table-insertRow 3F**：rows 数组断言面
- **tBodies 1F**：DOMParser XML documentElement.getElementsByTagName 缺
- **rowspan-0 / col-span-limits 4F**：处理模型，待分诊
- 跨流不变式：interactive-elements 44F 全为渲染流几何/UA/绘制门 ~21F（按名取案见 s57 账本）+ shadow 底座 20F + divergence 1F；parser form pointer 4F 用户缓行
