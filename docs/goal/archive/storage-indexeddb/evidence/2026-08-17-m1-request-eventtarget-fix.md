# M1 IDBRequest EventTarget 修复

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`
**Driving WPT**: `idbfactory_open.any.js`、`idbfactory_deleteDatabase.any.js`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| `idbfactory_open` | 0 / 29 | 4 / 29 | +4 Pass |
| `idbfactory_deleteDatabase` | 0 / 3 | 1 / 3 | +1 Pass |
| 首批 9 文件 | 14 / 50 | 19 / 50 | +5 Pass |
| 通过率 | 28.00% | 38.00% | +10.00pp |
| Fail | 36 | 31 | -5 |

分类数据见 [同名 JSON](2026-08-17-m1-request-eventtarget-fix.json)。

## 修复

+ `IDBRequest.addEventListener/removeEventListener`
+ 函数监听器和 `handleEvent` 对象监听器
+ request property handler 与监听器统一派发
+ open success 不再绕过统一事件路径
+ 基础 `preventDefault/stopPropagation/dispatchEvent` 表面

## 验证

`support.js` 原先产生的 14 个 `rq_open.addEventListener is not a function` 全部消失。
真实 WPT 继续执行后，5 个 subtest 直接通过，其余转化为版本升级、事务和
`IDBVersionChangeEvent` 的更深语义失败。

## 下一步

修复 `indexedDB.open` 的 WebIDL unsigned long long version 转换与范围校验，目标消除
15 个同步 `TypeError` 失败并修正 `1.5 → 1`。
