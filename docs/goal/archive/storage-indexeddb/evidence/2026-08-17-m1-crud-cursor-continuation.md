# M1 CRUD cursor continuation

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| Pass | 45 / 54 | 51 / 54 | +6 |
| Fail | 9 | 3 | -6 |
| 通过率 | 83.33% | 94.44% | +11.11pp |

add/put 的 6 个 autoIncrement cursor 用例全部转绿。

## 修复

+ object store openCursor 返回稳定 IDBCursor 实例
+ cursor 按 IndexedDB key 顺序遍历记录
+ continue 推进同一 cursor，并在同一 request 上再次异步派发 success
+ 末尾 request.result 为 null
+ query、next/prev direction 和 transaction pending 计数沿用共享路径

## 下一步

实现 duplicate primary key 与 unique index 的 ConstraintError request 语义，消除最后 3 个失败。
