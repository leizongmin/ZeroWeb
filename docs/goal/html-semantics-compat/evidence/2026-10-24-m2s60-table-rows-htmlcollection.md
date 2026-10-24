# M2 切片 60（s60）——tabular-data 域：table.rows HTMLCollection 形态收口

日期：2026-10-24。tabular-data 域 testharness 122P/12F → **127P/7F**（GAIN 5 / REG 0）：table-rows 5F 全绿（instanceof/named/ownKeys/live 断言面）。interactive-elements 全域 439P/44F/0T 复跑逐值不变，zero-engine lib 3454P/0F（+5 为 oracle 流新增测试）。

## 修复面

### 1. `_zwMakeHTMLCollection` rebuild 钩子（part05）

[SameObject] 集合的 live 读重建形态：`liveSpec.rebuild` 函数时 `current()` 每次属性读重算成员表（排序/过滤天然最新），不注册写侧 live 维护（无 matches → 不进 `_zwLiveCollections`）。与既有 matches 形态互斥（守卫分支，无 rebuild 的调用方行为零变化）。

### 2. rows getter 换 HTMLCollection 形态（part03，R5267）

s59 裸数组 → `_zwMakeHTMLCollection(rowsArr, { rebuild })`：

- [SameObject]：`_zwRowsCollCache`（WeakMap 键 = 元素 proxy，R3908 selectedOptions 同口径；换代后 sel 重解析自然换键弱引用自清）。
- live：rebuild 每读重算 `_zwTableRowsAll`（table）/`_zwTblChildAll`（section）——removeChild 后 namedItem→null、`id in rows`→false 断言天然过。
- 断言面复用既有完备工厂：instanceof（`_zwHCPrototype` 链上 HTMLCollection.prototype，R120 先例）、named getter（id 全元素 + name 仅 HTML-ns，R120 不对称）、ownKeys（[indices…, names…] 集合序——Complicated case named 顺序 = rows 序 ✓）、gOPD（`hasOwnProperty(i)`）、canonical 索引。
- 闭包持 sel/handle 原始值不持 proxy（防 WeakMap 键自持环）。
- 跨域 detached created 表（createElement 链）由 childNodes 融合视图承载（s59 地基复用）；非 HTML-ns tr/div 内 tr 排除（直接子 + ns 判定）。

### 3. 范围界定

仅 rows 收进集合形态；cells/tBodies 保持数组（真浏览器亦 HTMLCollection——类型一致化留挂账，本切片不扩大 REG 面）。`_rowList`/`_cellList` 消费面（length/索引读）经 trap 语义兼容零改动。

## 验证

- table-rows 5/5 全绿；连带面逐文件复核零回归（remove-row 6、insertCell 7、rowIndex 12、sectionRowIndex 19、cells 1、cellIndex 6、createTBody 15、table-insertRow 1P/2F 持平）
- tabular-data 域 **127P/7F/0T**（GAIN 5 / REG 0）
- interactive-elements 全域 **439P/44F/0T** 逐值不变
- zero-engine lib **3454P/0F**
- 拼接 shim node --check 通过；fmt/clippy 不适用（diff 仅 JS shim）

## 未解决项（tabular-data 余 7F 挂账序）

- **table-insertRow 2F**：insert into a tbody, not into thead/tfoot（无 tr 时插入位语义——现首 tbody，spec 末 tbody）
- **tBodies 1F**：DOMParser XML documentElement.getElementsByTagName 缺
- **rowspan-0 / col-span-limits 4F**：处理模型，待分诊
- cells/tBodies HTMLCollection 类型一致化（无测试断言，低优先挂账）
- 跨流不变式：interactive-elements 44F 全为渲染流几何/UA/绘制门 ~21F（按名取案见 s57 账本）+ shadow 底座 20F + divergence 1F；parser form pointer 4F 用户缓行
