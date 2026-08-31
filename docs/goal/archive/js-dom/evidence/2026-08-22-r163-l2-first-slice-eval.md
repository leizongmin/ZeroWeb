# R163 Evidence — L2 首片落地（fragment 真实节点优先）+ 评估结论（M1）

**日期**: 2026-08-22
**Commit**: `c9d2f00fe`（rebase 后；原 `ff35cc02f`）
**切片**: M1 L2 首片——fragment querySelectorAll 真实节点优先 + L2 整体切片评估

## 落地：fragment QSA 真实节点优先

fragment 的 `querySelectorAll` 查询结果若与子树真实节点（`_zwMEl` deepClone
产物）按 tag+id+outer 键命中 → **直接返回节点本体**（查询结果与
traverse/firstChild 遍历、mutation 面同 identity——L2 方向的首个落点）；
miss 回落 wrapper 缓存（原行为零变化）；命中键把真实节点 pin 进 identity
缓存（后继查询同对象）。

## 评估结论（L2 整体切片方向）

fragment tree-order 测试的探针实证（`zz-probe-r163`）：
- 查询面 `querySelectorAll("*")` 返 **309**（序列化源含 `<head>` 内容——
  bodyInner 全文进 h），遍历面 traverse 只走 **1**（frag 子树 = 单根
  deepClone，节点遍历可达面浅）
- **内容分歧**：查询序列化源与活子树**内容不同**（不只 identity 不同）——
  per-site 映射（本轮真实节点优先）无法闭合 tree-order

**结论**：tree-order / dynamic NodeList / ns 簇 / `[*|TiTlE]` 的完全收口
需要 **L2 统一 live document 的 detached 工厂改造**——detached factory 的
queryBody 直接查自身 `_tree`（不经序列化 → host re-parse → JSON 往返），
查询源与活树同源。part05 已有 JS 客户端匹配器（`_parseSelectorListOf` /
`_matchComplexAgainst` + `_handleSubtreeNodes` 的 nodeInfo 树上下文）可
复用；风险面 = queryBody 消费者广（getElementById/getElementsByTagName/
querySelector 全族），须分片迁移 + 每片全量回归。

### 建议切片序（R164+）

1. **L2-d1**：detached factory `queryBody` 对**纯 tag 选择器**改查自身 `_tree`
   （JS 匹配器最简形态），JSON 往返保留为组合器/伪类回落
2. **L2-d2**：`#id` / `.class` / `[attr]` compound 形态迁移
3. **L2-d3**：组合器（descendant/child/sibling）+ 伪类（nodeInfo 已有兄弟/
   祖先链）迁移
4. 每片：`make test` + 全量 dom WPT 双路径 net≥0 才 land

## A/B 双路径

全量 dom WPT **9516P/347F/18T** 双路径逐计数一致（+1P vs R162——ident 优化
的正向命中；无回归）。

## 验证

- `cargo test -p zero-engine`：2308 全绿；`make test` 全绿；fmt/clippy 干净
