# R133 — M4 nodes：insertAdjacentElement 入口校验（1F→0F + 涟漪 1F；sel 子移动深结构评估记录）

**日期**: 2026-08-20
**Driving WPT**: `dom/nodes/Element-insertAdjacentElement.html`（SyntaxError 用例 1F→0F；
剩 5F 记深结构）
**账本**: `tests/wpt-runner/imported-tests.txt`（R133 条目）

## 已修（入口校验三层）

1. **非法 position → SyntaxError DOMException**（同步抛，ASCII case-insensitive 四值
   匹配）：旧实现 catch 吞 host 错恒返 null——WPT "Inserting to an invalid location
   should cause a Syntax Error exception"。
2. **element 参数非节点 → TypeError**（WebIDL Element 参数校验；旧宽容返 null）。
3. **documentElement 的 beforebegin/afterend → HierarchyRequestError**（spec pre-insert
   step 6：插 sibling 进 Document 造成第二元素子）：host 侧 `insert_nodes_at_position`
   的报错经 mutation **异步 apply** 不可达（同步 turn 内抛不出），JS 侧按目标身份
   （html 无元素父）前置判定。

## 深结构评估（未竟，首版已回退）

**剩余 5F 同根因**：sel 子（静态页面元素 `#test1-4`——无 handle）的**同步移动**。
用例形态 `target.insertAdjacentElement('beforebegin', document.getElementById('test1'))`
后**同一同步 turn 内**读 `target.previousSibling.id`——host mutation 在脚本 turn 后
apply，同步读依赖 JS 侧 pending overlay。

**首版实现**（已回退）：`_zwSelParentOf` 移动链（sel → {parentSel, nextSibling}）+
新/旧父 pending 桶记账 + `__zw_insert_adjacent_html` 序列化插入。两处翻车：
① 自建桶 `{added, removed}` 与 `_zwPendBucket` 工厂的并行集（`addedSet`/`removedSet`）
不一致——`_zwHCLiveInvalidate` 的 `_pb.addedSet.has` 直接 TypeError（探针定位
`@before-notify` phase）；
② 二次移动（同一 element 移两处）overlay 状态漂移——`__zw_parent` 混合 host 快照
与 overlay 视图，旧父判定不稳定。

**结论**：sel/handle 双形态的移动语义统一是 **M1 L2（polyfill-live 合一）的正解**
——live Document 单一权威后无「两个视图同步」问题。master.md 已记深结构清单。

## A/B 验证

- **Element-insertAdjacentElement**：SyntaxError 用例 1F→0F + 涟漪 1F（nodes 内另一
  校验消费方）。
- **dom/nodes 全量**：polyfill 8427→**8429P（+2）** fail 197→**196（逐文件 diff
  零新增）**；native 7648→**7649P**。
- **回归面**：events 422P/27F、collections 49P、traversal 1589P/15F 与 R132 逐项
  一致。
- **单测**：engine `test_insert_adjacent_element_validation_r133`（SyntaxError 大小写
  不敏感/参数 TypeError/docEl HRE/合法路径返值）。

## 教训

1. **host 报错经 mutation 异步 apply 不可达**——同步抛错的校验必须在 JS 侧前置
   （spec 入口步骤 1-3 全部前置到 shim；位置约束类[step 6/7]按可判身份前置）。
2. **pending 桶的并行集是结构契约**——`_zwPendBucket` 工厂的桶带 `addedSet`/
   `removedSet`，任何自建桶若进入 `_zwHCLiveInvalidate` 消费路径必炸；新记账一律
   经工厂（探针 phase 标记法快速定位）。
3. **同步 overlay 的二次移动是试金石**——「同一节点移两处」暴露视图混合的漂移，
   polyfill 双形态（sel/handle）的移动语义统一只能靠 L2 live-document，overlay
   补丁路走不通就果断回退记深结构（沉没成本不加码）。
