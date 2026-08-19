# M1 IndexedDB 版本状态修复

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`
**Driving WPT**: `idbfactory_open`、`idbfactory_deleteDatabase`、`idbversionchangeevent`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| `idbfactory_open` | 19 / 29 | 23 / 29 | +4 Pass |
| `idbfactory_deleteDatabase` | 1 / 3 | 3 / 3 | +2 Pass |
| `idbversionchangeevent` | 0 / 1 | 1 / 1 | +1 Pass |
| 首批 9 文件 | 34 / 50 | 41 / 50 | +7 Pass |
| 通过率 | 68.00% | 82.00% | +14.00pp |

## 修复

+ 数据库状态保存 version、stores 和活动 connections
+ 无 version reopen 使用当前版本，不再重复触发 upgrade
+ 低版本 open 发 `VersionError`，高版本 open 发 versionchange
+ `IDBVersionChangeEvent` 构造器、oldVersion/newVersion 与 Event 原型链
+ deleteDatabase 通知活动连接并返回正确版本事件
+ `databases()` 返回真实当前版本

## 验证

首批 50 个 subtest 仅剩 9 个 versionchange transaction 失败；版本状态、delete 事件和
versionchange event 用例全部通过，无 Timeout 或 Unsupported。

## 下一步

实现 `request.transaction` 的 versionchange 生命周期、complete/abort 事件和 abort 回滚。
