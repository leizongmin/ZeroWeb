# R332 — normalize childList MutationObserver records（2026-08-28）

## 问题

`MutationObserver-childList.html` 全文件 Timeout：38 个注册测试中 13 个 async_test 永挂起。
R184 实化 `Node.normalize()`（no-op → 真合并）时漏发 childList MutationObserver record——
spec `dom-node-normalize` 的每步兜接/移除都经 `concept-mutationobserver-queue-mutation-record`
逐次入队，WPT `runMutationTest` 的 async_test 依赖回调结算。

挂起 13 例：n20/n21（normalize ×2）、n30-n32/n34/n35（insertBefore 族）、n40-n42/f44
（appendChild 族）、n53（self-replace）、n91（insertNode children）。本轮修复 normalize 2 例，
其余 11 例为 move-record 族（另切片）。

## 根因与修复（part04.js）

探针链（v1-v13）：`direct:null|pend:0` → sel 分支 no-op 实证；`pendAfterTextAppend:2` →
handle 子 append 的 record 通道正常；隔离出 normalize 的两条容器路径。

1. **handle 容器**：`_r184NormArr` 增加 recsOut 累加器参数，逐移除步推 childList record。
   首版把 record 逻辑放在函数外、引用其局部 `out` → ReferenceError 被 try/catch 吞 →
   record 永不派发（探针 pend:0 实证）。
2. **sel 容器**（getElementById 元素；快照解析文本为单一串 + 同步 append 的 handle 文本经
   pending overlay 挂尾）：旧是 no-op。现：保留头 Text 的 data 兜接被并文本
   （`__zwWriteChildText` → host `SetChildText`）+ `__zw_remove_handle` 移除被并文本 +
   每步一条 childList record。
3. **record 兄弟字段按 live 数组语义**：spec queue-mutation-record 的 sibling 取自
   **移除时刻**的树形态。WPT n21 期望两条 record：r1 removed=[AN] prev=CH next=GED、
   r2 removed=[GED] prev=CH——首版按初始快照索引取 prev 命中已移除的 AN。

## 验证

- `MutationObserver-childList.html`：pending 13 → 11；`Node.normalize mutation` /
  `Node.normalize mutations` 双双 Pass（27P/1T）
- `Node-normalize.html`：4P/4 维持（normalize 合并语义零回归）
- MutationObserver 全族：119P/4F/4T，fail 集与 R331 备档恒等（cross-realm 1F +
  document parser 3F + 4 Timeout 并发慢族）
- 回归测试 `test_normalize_childlist_mo_records_r332`：handle 容器 1 record +
  sel 容器 2 record（prev/next 逐字段断言）
- engine v8 2471 / quickjs 1466 绿；fmt/clippy 双矩阵干净

## 教训

1. try/catch 吞 ReferenceError 是 shim 双刃——作用域 bug 静默化为 no-op，
   探针打点（pend:0）比读代码更快定位。
2. spec 的「每步 record」语义：normalize/surround 等**多步操作**的 record 数量与
   兄弟字段都按逐步树形态计算，不能按初始快照一次算完。
