# M1 第二批完成

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| Pass | 158 / 166 | 166 / 166 | +8 |
| Fail | 8 | 0 | -8 |
| 通过率 | 95.18% | 100.00% | +4.82pp |

当前 imported 21 文件全部通过。该结果不代表上游 IndexedDB 目录整体通过率。

## 修复

+ IDBIndex.openCursor 复用 index query entries
+ cursor 分离 index key 与 primary key
+ 重复 index key 按 primary key 排序
+ continue(key) 支持 next/prev 定向跳跃
+ 空 index keyPath 使用 value 本身作为 index key
+ runner 仅等待 1 秒内仍 active 的 step timer，不等待 harness 10 秒 watchdog

## 下一步

继续扩展真实 WPT 分母，并开始 M2 JS↔Rust bridge。
