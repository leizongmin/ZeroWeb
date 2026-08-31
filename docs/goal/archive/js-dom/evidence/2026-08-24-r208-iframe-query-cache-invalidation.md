# R208 Evidence — iframe doc 查询缓存的同键失效（restoreIframe r1+ 轮 919F 簇根因修复）

**日期**: 2026-08-24
**切片**: M4——R207 精确定位的「iframe doc 查询的 live-tree 同步」；全量 **49541P/5004F/22T → 50796P/4238F/21T（净 +1255P/-766F，零真回归）**
**改动面**: `part03.js`（三处 removeChild 的缓存失效）+ `part21.rs`（回归单测）

## 一、根因（探针链 11 轮实证）

restoreIframe 每轮重建**结构全等**的 `div#test`（querySelector('#test') → removeChild →
createElement + body.insertBefore）。detached doc 查询缓存 `_zwQWrapCache` 的键是
`tag\x1fid\x1fouterHTML`——新节点的键与已移除节点**完全相同**。`body.removeChild`
把节点从 live `_tree` 摘除但缓存条目残留：下一次同键查询 cache-hit 返回**已移除节点**
（parentNode=null）→ `td.parentNode.removeChild(td)` 崩 "Cannot read properties of null
(reading 'removeChild')"（Range-surroundContents/insertNode r1+ 轮 919F 簇）。

探针链（沙箱内复刻 restoreIframe 循环，三轮 q/remove/insert）：
- probe1/2：r1 命中正常（pn=BODY、inBody=true），r2 命中 pn=null——精确复现
- probe7/9/10：r2 的 query 返回 **nd1（上一轮已移除节点）**而非 nd2（live 新节点），
  nd1/nd2 outerHTML 全等（键撞车实证）

## 二、修复（三处失效点）

| # | 位置 | 失效动作 |
|---|------|----------|
| ① | `body.removeChild`（part03） | 移除元素子时 `_zwQWrapGen++` + `_zwQWrapCache.clear()` + `_tree._zwNodeIdx = null`（DFS 真节点索引） |
| ② | doc 级 `removeChild`（part03） | 同款（restoreIframe 清首/末子路径；`_tree` 存在性守卫） |
| ③ | `_zwMEl.removeChild`（part03） | 经 `_zwOwnerTree`/`_zwOwnerDetDoc` 溯源槽找源 doc，赋值 `_zwQWrapBump`（setter 清缓存+桥表）；不可溯源零变化 |

spec 依据：`dom-node-remove`——移除即脱离文档，后续查询不得命中。
https://dom.spec.whatwg.org/#concept-node-remove

## 三、验证链

- **单文件**：surroundContents **0P/1840F → 645P/1195F**；insertNode **1P/1839F → 486P/1354F**
- **全量**：49541P/5004F/22T → **50796P/4238F/21T**（净 +1255P/-766F）
- **零真回归**：逐 subtest 状态转移比对（before/after 同键 Pass→Fail = **0**）；
  364 条「新失败行」全部是**新暴露的 subtest**——mega-case 文件旧版在 r0 轮即崩
  （每文件只报 3-4 个 subtest），现在跑到更深轮次（cloneContents 4 → 187、
  extractContents 4 → 187、deleteContents 3 → 125）
- **native 对照**：fail 集逐条一致（4238F = 4238F）；唯一 status 分歧 =
  Node-parentNode-iframe 的既存 flaky Timeout（polyfill Timeout / native Pass，
  历史轮次同形态）
- **engine 单测**：2347 全绿（含新增 `test_iframe_doc_query_cache_invalidation_r208`）；
  fmt/clippy 干净
- **make test**：`window_surface_present_smoke`（XOpenDisplayFailed 无显示环境，
  R203-R207 同款豁免）+ 3 个 `service_worker_runtime::*`（**SW 流已知红灯**——其
  master.md「CI 守护记录」明记预存失败；stash 实证干净 main 上同样失败，跨流归因
  run-rules §10）

## 四、r1+ 轮次的新失败面（R209 靶点输入）

surround/insert 的 r1+ 轮已跑通至断言层，新聚类：
- surroundContents：`node2.compareDocumentPosition is not a function`（230）/
  `node.hasChildNodes is not a function`（138）——**eval 产节点形态缺方法**（common.js
  的 testNodes eval 产物——doctype/PI/foreign 域节点的方法面）
- insertNode：`assert_throws_dom HIERARCHY_REQUEST_ERR must be thrown`（396）——
  异常路径语义；`range.startContainer.splitText is not a function`（242）
- cloneContents/extract/delete：`Cannot read properties of undefined (reading '0')`（95）/
  `isEqualNode is not a function`（64）——**返回 fragment 的子形态缺方法**（cloneContents
  返 fragment 的子节点为轻量对象）

## 五、commit

`83adf18e6`（rebase 后；原 `6ac61ec10`）
