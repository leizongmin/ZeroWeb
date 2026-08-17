# M1 IndexedDB factory 首批 WPT 基线

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`
**范围**: `IndexedDB` factory/global/event 首批 9 个 `.any.js` window 变体

## 结果

| 指标 | 数值 |
|---|---:|
| 文件 | 9 |
| subtest | 50 |
| Pass | 6 |
| Fail | 44 |
| Timeout / Unsupported / NotRun | 0 |
| 通过率 | 12.00% |

完整分类数据见 [同名 JSON](2026-08-17-m1-factory-baseline.json)。

## 失败聚类

| 数量 | 根因 |
|---:|---|
| 16 | `open`/`cmp` 缺 WebIDL `TypeError` 参数校验 |
| 14 | `IDBRequest` 缺 `addEventListener` EventTarget 表面 |
| 5 | `open.transaction` 升级事务缺失 |
| 5 | `IDBFactory.cmp` key 类型及 binary 排序错误 |
| 2 | `cmp` 非法 key 未抛 `DataError` |
| 1 | `open` version 转换错误 |
| 1 | `IDBVersionChangeEvent.oldVersion/newVersion` 缺失 |

## 复现

```bash
make testharness-indexeddb
make testharness-indexeddb FILTER=idbfactory_cmp
```

该命令在存在失败时返回 1，这是通过率基线的预期结果。JSON 原始输出可用受 `test-guard`
包裹的 runner 命令加 `--json` 获取。

## 结论

WPT 获取、`.any.js` window wrapper、META support 脚本和 testharness 结果采集链路已跑通。
首个轻量修复选择 `IDBFactory.cmp`：12 个 subtest 已有 4 Pass，剩余 8 项集中在参数校验和
key 排序，不依赖 Rust bridge 或持久化设计。
