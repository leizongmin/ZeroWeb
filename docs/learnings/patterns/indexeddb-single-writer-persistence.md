# IndexedDB persistence requires a single writer

**日期**: 2026-08-18
**相关模块**: `zero-storage`, `zero-page-runtime`, browser/renderer IPC

## 问题描述

Browser、独立 renderer 和嵌入式 WebView 当前都能创建 `StorageManager`。如果每个 manager 独立加载同一 IndexedDB 文件并直接写回，任一进程都可能用旧快照覆盖另一进程的新提交，跨 connection transaction ordering 也无法成立。

## 根因分析

原子 rename 只能保证单次文件替换不产生半文件，不能解决多个内存副本之间的 lost update。文件锁也只能串行化写入瞬间，无法让已加载的旧 database snapshot 自动更新。因此“所有进程共享一个目录”不是 storage ownership。

## 解决方案

+ 磁盘数据库由一个 browser-side owner 持有。
+ Renderer 通过有 request ID 的 IPC 请求/响应执行 factory、schema、transaction 和 cursor 操作。
+ `StorageManager` 在候选数据库上应用 transaction，完成原子落盘后再替换 live state。
+ 嵌入式 WebView 没有 browser 进程时，在宿主进程内使用单一共享 owner。
+ 测试必须覆盖两个 connection 的交错 transaction、owner 重建读回和落盘失败时 live state 不变。
