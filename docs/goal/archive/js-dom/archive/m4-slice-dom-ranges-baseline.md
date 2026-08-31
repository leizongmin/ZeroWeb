# M4 切片 R42 — 导入 dom/ranges 基线 + Range/StaticRange/Attr 端点 API 面

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）/ DC-3
**证据**: [../evidence/2026-08-14-r42-dom-ranges-baseline.json](../evidence/2026-08-14-r42-dom-ranges-baseline.json)

## 切片内容

### 1. 导入 dom/ranges（44 用例 + Range-mutations.js）

- `testharness.rs DOM_TEST_SUBDIRS` + `fetch-dom-subset.sh SUBDIRS` 加 `dom/ranges`
- raw.githubusercontent 间歇超时——FORCE=1 批量 fetch 多次 curl timeout 后改**逐文件 5 次重试循环**（45/45 全拉齐）
- 跳过 `crashtests/`（非断言型）与 `tentative/`（实验 API——OpaqueRange 等）

### 2. API 面修复（基线聚类驱动）

- **`getAttributeNode`/`getAttributeNodeNS`**（25 fail 最大簇）：part04 get trap 新增——经 `_zwMakeAttr` 返真 Attr 节点（instanceof Attr true / value / name / ownerElement），缺省 null
- **`new Range()`**：空函数 stub → 返 `_makeRange()` 真实实例（用例普遍 `new Range()` 而非 createRange）
- **`StaticRange(init)` 构造器**：readonly 四属性（accessor）+ collapsed 派生 + 非 Node 容器抛 TypeError（WPT StaticRange-constructor 14 subtest）
- **Range setStart/setEnd spec 校验**：容器仅拒 DocumentType（**Attr 允许**，length=0——offset 0 合法、>0 抛 IndexSizeError）；字符数据节点 offset>length 抛 IndexSizeError；**元素容器免上限校验**（detached/handle-only proxy 的 childNodes 视图恒空，与真 0 不可区分，放宽防误伤）；setStartBefore/After/selectNode 无 parent 抛 InvalidNodeTypeError

### 3. 迭代修正

初版校验把 Attr 一并拒绝 → "setStart() to an Attr node at offset 0 is allowed" 误伤（spec 仅拒 DocumentType）；元素容器 offset 校验在 detached proxy 上误抛 → 放宽。两轮修正后净 +30。

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| Range-attribute-nodes | 0P/26F | **15P/11F** |
| StaticRange-constructor | 0P/17F | **14P/3F** |
| dom/ranges polyfill | 9P/89F（9.18%）| **39P/59F = 39.80%** |
| dom/ranges native | 9P/89F | **39P/59F = 39.80%（对等差 0pp）** |

零回归：events 189P / nodes 2503P / collections 17P / traversal 9P。

## 失败聚类（~59 fail 剩余 = 深结构）

- **iframe 驱动 mega-case**（cloneContents/deleteContents/extractContents/insertNode/surroundContents/cloneRange）：经 `iframe.contentWindow.setupRangeTests()` 驱动——跨文档（R12 iframe 深结构 html-compat 域）
- ShadowRoot range 用例（cloneContents-in-ShadowRoot）
- comparePoint/isPointInRange/intersectsNode 缺方法（需 live 树节点位置几何，M1 L2）
- detached createDocument/appendChild 基础设施

## 验证门禁

- 单测 `test_range_apis_r42`（11 断言组）
- engine v8 2123 / quickjs 1415 / wpt-runner 171 / webview 595 全绿；quickjs 矩阵 14 crate 全绿
- clippy 双矩阵零警告，fmt 无 diff（cargo fmt 后）
