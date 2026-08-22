# R178 Evidence — Range 点查询方法族 + Attr-rooted mutation 守卫（M4）

**日期**: 2026-08-22
**切片**: M4 轻量——Range-attribute-nodes 11F → 0F（26P/26 全 100%），全量净 +11P/-12F
**改动面**: part06（Range 4 方法 + insertNode/surroundContents Attr 守卫）+ page-runtime（跨流 clippy 修复）

## 一、根因

`Range` 缺 4 个点查询方法（`comparePoint` / `isPointInRange` / `intersectsNode` /
`compareBoundaryPoints`——直接 `is not a function` 8F）+
`insertNode`/`surroundContents` 对 Attr-rooted range / Attr 参数不抛
HierarchyRequestError（静默 no-op 3F）。

## 二、实现

| 方法 | 语义（spec） |
|------|------|
| `_rootOf178` | 沿 parentNode 上行到根（concept-tree-root）；Attr 的 parent null → 根是自身——与文档根永不相等，是 WrongDocumentError 短路族的判据 |
| `comparePoint(node, offset)` | root 不同 → WrongDocumentError；offset 超 node length → IndexSizeError；同容器按 offset 差、跨容器经 compareDocumentPosition |
| `isPointInRange(node, offset)` | root 不同返 **false 不抛**；同容器按 [start, end] 区间 |
| `intersectsNode(node)` | root 不同返 false；**parent null（node 即根）恒 true**（range 与自身根必交——首版误加 collapsed 前置被 WPT 抓回：Attr-rooted collapsed range 与 Attr 期望 true）；否则按父内偏移区间交 |
| `compareBoundaryPoints(how, source)` | 两 range root 不同 → WrongDocumentError；how ∈ 0-3（START_TO_START/START_TO_END/END_TO_END/END_TO_START）；同容器按 offset 差 |
| `insertNode` 守卫 | WebIDL TypeError（null/非 Node）+ nodeType 2（Attr 参数或 Attr-rooted 容器）→ HierarchyRequestError |
| `surroundContents` 守卫 | 同款 Attr 双形态 HierarchyRequestError |

## 三、跨流 clippy 修复（page-runtime）

`zero-page-runtime/cache_storage_host.rs` 两处 `chunks_exact(4)` 触发 clippy
1.98 新 lint `chunks_exact_to_as_chunks`（上游 service-workers 流引入、干净树
复现、阻塞全仓 clippy 门）。按既有 `as_chunks` 惯例（canvas/render-foundation
先例）机械替换 2 处——公共面小修，非工作面冲突。

## 四、验证

| 门 | 结果 |
|----|------|
| Range-attribute-nodes | 15P/11F → **26P/0F（100%）** |
| Range-comparePoint-2 | 连带转绿（方法实现前 is-not-a-function） |
| 全量 dom WPT polyfill | **9624P/239F/20T**（R177 9613P/251F——净 +11P/-12F；insertBefore-iframe-crash 已知 flaky F↔T 互换） |
| 全量 dom WPT native | **9625P/239F/19T**，per-file 唯一分歧 = 同一 flaky |
| `make test` | 66 套件 **18127P/0F** 一次通过 |
| fmt / clippy | 干净（含跨流 page-runtime 修复） |

## 五、下一步（R179）

- 全量 fail Top 簇：Event-dispatch-single-activation-behavior 14F /
  node-creation-realm 13F。
- tree order 2F 记 RFC（identity 归一域）。
- M2/M6 面：S6 高层 API 去字符串 / native dom_bindings 补齐。
