# M1 indexedDB.open version 转换修复

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`
**Driving WPT**: `IndexedDB/idbfactory_open.any.js`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| Driving case | 4 / 29 | 19 / 29 | +15 Pass |
| 首批 9 文件 | 19 / 50 | 34 / 50 | +15 Pass |
| 通过率 | 38.00% | 68.00% | +30.00pp |
| Fail | 31 | 16 | -15 |

分类数据见 [同名 JSON](2026-08-17-m1-open-version-fix.json)。

## 修复

+ version 参数先执行 Number 转换
+ 有限值向零截断，`1.5` 转为 `1`
+ `0`、负值、NaN、Infinity 和大于 `Number.MAX_SAFE_INTEGER` 的值抛 `TypeError`
+ 显式 `undefined` 与省略参数均使用默认版本 `1`
+ 对象参数沿用 JS ToPrimitive/Number 转换顺序

## 验证

真实 WPT 的 15 个非法 version 同步 `TypeError` subtest 全部转绿。合法 version 用例继续执行后，
剩余失败准确落到尚未实现的 upgrade transaction，不再被参数转换遮挡。

## 下一步

实现 versionchange transaction 最小真实表面，覆盖 `request.transaction`、`abort()`、
`objectStore()` 与 transaction complete/abort 顺序；同时补 `IDBVersionChangeEvent`。
