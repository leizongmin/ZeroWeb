# M4 切片 R41 — 导入 dom/traversal 基线 + TreeWalker/NodeIterator API 面

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）/ DC-3
**证据**: [../evidence/2026-08-14-r41-dom-traversal-baseline.json](../evidence/2026-08-14-r41-dom-traversal-baseline.json)

## 切片内容

### 1. 导入 dom/traversal（17 用例）

- `testharness.rs DOM_TEST_SUBDIRS` + `fetch-dom-subset.sh SUBDIRS` 加 `dom/traversal`
- 手动补 `dom/traversal/support/` 3 文件（fetch_dir_html 只列顶层不递归子目录）
- **修复 `extract_script_src` 支持无引号属性值**（`src=../common.js` 是合法 HTML 语法，上游 NodeIterator.html/TreeWalker.html 在用；此前只匹配带引号值 → 外部 fetch failed → 整用例不跑）。此修复同时让 dom/nodes +1 pass

### 2. TreeWalker/NodeIterator API 面修复（基线驱动）

- **NodeFilter 常量对齐上游全表**：`SHOW_PROCESSING_INSTRUCTION` 0x10→**0x40**（原写错）；补 `SHOW_ATTRIBUTE(0x2)`/`SHOW_ENTITY_REFERENCE(0x10)`/`SHOW_ENTITY(0x20)`/`SHOW_NOTATION(0x800)` → `NodeFilter-constants.html` 100%
- **root 校验**：`createTreeWalker()`/`createNodeIterator(null)` root 非 Node 抛 TypeError（spec `document-createtreewalker` 步骤 1）
- **readonly**：root/whatToShow/filter getter-only accessor（testharness assert_readonly 的 accessor 分支 `set===undefined` 通过）
- **whatToShow 显式 null → 0**（ToUint32(null)），区别缺省 → SHOW_ALL
- **toString branding**：`[object TreeWalker]` / `[object NodeIterator]`
- **currentNode setter**：赋非 Node（null/{}）抛 TypeError；赋合法 Node 接受并重定位游标（accepted[] indexOf，miss → -1）

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| NodeFilter-constants | 1P/1F | **2P/0F（100%）** |
| TreeWalker-basic | 1P/5F | **5P/1F**（剩 1F = detached 深结构）|
| TreeWalker-currentNode | 1P/3F | 2P/2F（剩 = lazy 续走，M1 L2）|
| dom/traversal polyfill | —（无 js inline）| **9P/46F = 16.36%** 首个真实基线 |
| dom/traversal native | — | 8P/47F = 14.55% |

零回归：dom/events 189P / dom/nodes 2503P（+1 inline 修复解锁）/ dom/collections 17P。

## 失败聚类（~46 fail 主力 = 深结构）

- **detached 树遍历（~30 fail 主力）**：上游用例用 `createElement + appendChild` 建树后**不挂 body**（testElement detached）→ polyfill `childNodes` 恒空（延迟 mutation 快照限制，与 R4 appendChild 闭环同根因）→ walker 空。属 **M1 L2**（polyfill-live 合一后 detached 子树可见）
- TreeWalker currentNode **lazy 续走语义**（root 外位置 nextNode 返回该处 firstChild）——eager accepted[] 模型近似不足，随 L2 lazy 重构
- cross-realm filter（iframe realm 域）+ NodeIterator removal-during-filtering（live iterator 语义）

## 验证门禁

- 单测 `test_treewalker_api_surface_r41`（10 断言组）
- engine v8 2122 / quickjs 1415 / wpt-runner 171 / webview 595 全绿；quickjs 矩阵（749/124/75/547/10）全绿
- clippy 双矩阵零警告，fmt 无 diff
