# M1 CRUD key range 修复

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`
**Driving WPT**: object store get/delete/count

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| subtest | 51 | 54 | +3 |
| Pass | 15 | 22 | +7 |
| Fail | 36 | 32 | -4 |
| 通过率 | 29.41% | 40.74% | +11.33pp |

分母增加 3 是 transaction pending-request 跟踪修复后，delete 用例中此前未执行的后续 subtest
进入统计；不是新增测试文件，也没有缩减失败分母。

## 修复

+ `IDBKeyRange.bound/only` 与闭/开区间 contains
+ `get(range)` 按 key 序返回范围内第一条记录
+ `delete(range)` 删除范围内全部记录
+ `count(range)` 和 `count(key)`
+ transaction 等待嵌套 request callbacks settle 后再 complete

## 下一步

补 object store 生命周期 guard：deleted store 的 InvalidStateError、readonly 的 ReadOnlyError、
aborted transaction 的 TransactionInactiveError，共 10 个失败。
