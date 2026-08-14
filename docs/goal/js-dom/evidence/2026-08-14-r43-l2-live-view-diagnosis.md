# R43 — M1 L2 live 视图最小切片：实现、实测、回退决策（负结果归档）

**日期**: 2026-08-14
**里程碑**: M1 L2 前置评估（入口文档「L2-first 最小只读子集」建议）
**结论**: **不 land**——dom/nodes 净 -10（2503→2493），traversal/ranges 持平零收益。诊断成果归档供 M1 完整方案使用。

## 实现方案（已验证可行，后回退）

`with_query_doc_live(html, mutations, f)`（callbacks.rs thread_local `QUERY_DOC_LIVE_CACHE`）：

- 缓存键 = `(html, mutations.len())`，键变 → **全量重放**（parse 快照 + 一次 `apply_dom_mutations(全部 pending)`）
- **不可增量分批**：`apply_dom_mutations` 的 ephemeral handle map（CreateElement handle→NodeId）是单次调用作用域，分批 apply 时后批 AppendChild 引用前批 CreateElement 的 handle 失配（实测 fail-soft 丢整批）
- fail-soft：整批 apply 失败退回纯快照
- 不消费队列（renderer drain 语义不变）、不写权威 dom_html
- 升级 4 个回调：`__zw_query_match` / `__zw_query_all` / `__zw_parent` / `__zw_child_nodes`（后者需 `child_nodes_json_doc` / `parent_selector_for_doc` doc 版 helper）

单测验证（4 断言全过）：同脚本块内 appendChild(sel-based 父) 后 querySelector 立即命中、setAttribute 后属性选择器匹配、removeAttribute 后不匹配、childNodes 反映新子节点。

## 实测结果（为何回退）

| 子目录 | 前 | 后 live | 判定 |
|--------|-----|---------|------|
| dom/traversal | 9P/46F | 9P/46F | **零收益**——detached 容器 handle-based（`_handleChildNodes` JS 侧 registry），不走 host 查询回调 |
| dom/ranges | 39P/59F | 39P/59F | 持平——剩余失败在 iframe 驱动 mega-case/ShadowRoot |
| dom/nodes | 2503P | **2493P（-10）** | **回归** |
| engine 单测 | 2123 | 2124（+1 新测试；part15 R3190 一处断言需按 live 语义更新） | 语义迁移 |

### dom/nodes -10 根因（语义边界发现）

`case.html` 大小写测试在**独立 document**（iframe doc）建 abc/Abc/ABC 元素，期望主文档 `getElementsByTagName('abc')` 返 `[]`。旧快照语义下 detached 元素查询不到（偶然「正确」）；live 视图下 pending mutations 把这些元素**并进了主文档视图**（AppendChild parent 解析落到主文档容器）→ 返回 9 个 → fail。

**本质**：单文档沙箱里「别的 document 的 mutation」与「本文档的 mutation」在 pending 队列中不可区分。live 视图要正确，须 mutation 带 **document 归属标记**——这正是 M1 L2 完整方案（`Rc<RefCell<Document>>` 三方合一，每 iframe 独立 Document 实例）要解决的问题。最小切片绕不开它。

### traversal 零收益根因

上游用例的 detached 树 = `createElement` 返回 **handle-based proxy**（无 selector），其 `childNodes` 走 shim JS 侧 `_handleChildNodes`（registry，R2927），**不经 host 回调**。host 侧 live 视图升级的是 sel-based 查询路径——与 handle 树正交。handle 树可见性须 JS 侧 registry 增强（append 时同步 registry 子列表）或 M1 L2 的真 live Document。

## 对 M1 完整方案的设计输入

1. live 视图须**按 document 作用域**过滤 pending mutations（每 mutation 标记 owner document；或 iframe 各自队列）
2. handle-based detached 树（`_handleChildNodes` registry）与 sel-based 查询是两套并行数据源——L2 完整方案应以共享 `Rc<RefCell<Document>>` 统一二者，而非在快照视图上叠加
3. 全量重放（非增量）是 `apply_dom_mutations` handle 作用域的硬约束——L2 若保留批处理语义，需 handle map 提升到持续状态（renderer 已有 `handle_selector_map` 可参考）
4. `with_query_doc_live` 的 fail-soft 模式与 thread_local 缓存结构可直接复用

## 状态

代码全部回退（工作树 = R42 dd22f6b7 基线），零残留。诊断归档供 M1 启动时使用。
