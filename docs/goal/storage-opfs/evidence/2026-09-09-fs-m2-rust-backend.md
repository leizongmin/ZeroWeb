# OPFS File System (fs/) WPT 基线 — M2 Rust 真实后端接线后（storage-opfs M2 切片 1）

- 日期：2026-09-09
- 对照基线：`2026-09-09-fs-baseline-inmemory-shim.md`（内存 shim 版，48 Pass / 85 Fail / 133）
- 后端：`__zw_opfs` 宿主命令 → zero-page-runtime `opfs_host` → zero-storage `opfs` 模块
  （per-origin 树；wpt-runner 走 `IndexedDbOwner::in_memory` 即纯内存态，持久化 e2e 属 M3）
- 执行通道：`make testharness-fs`

## 成绩

| 阶段 | Cases | Subtests | Pass | Fail | 通过率 |
|---|---:|---:|---:|---:|---:|
| M1 基线（内存 shim） | 13 | 133 | 48 | 85 | 36% |
| M2（Rust 后端） | 13 | 133 | **126** | **7** | **95%** |

**净 +78 subtests**（85 失败 → 7；基线 10 类失败聚类中 1-9 全部清零或大幅收敛）。

## 剩余 7 失败（M3+ 处理队列）

| 失败 | 根因 | 归属域 |
|---|---|---|
| isSameEntry × 3（postMessage 克隆句柄） | structuredClone 不支持句柄（transferable 面） | **skip 域**（Support Envelope 排除） |
| createWritable() can be called on two handles | 上游 pinned-rev 用例签名错误（`createDirectory(t, name, parent)` 传 3 参，helper 只收 2） | 上游用例缺陷（后续 rev 已修）；不充数、入已知失败清单 |
| write() invalid blob should reject | blob 失效快照检测未实现（getFile 后 removeEntry 再写，headless blob 同步取无失效语义） | M3 深化项 |
| piped × 2（fetch response.body → pipeTo） | `fetch('data:...').body` 返 null（fetch 流面缺失） | fetch/stream 域（非本 goal 面） |

## 语义修复清单（本轮）

1. keys/values/entries/句柄 `[Symbol.asyncIterator]`（spec async iterable；句柄直接迭代 = entries 对）
2. DOMException 错误类型（NotFoundError=8 / TypeMismatchError=17 / NoModificationAllowedError=7 /
   InvalidModificationError=13 / SyntaxError=0 新式）+ TypeError 参数面
3. `handle.remove({recursive})`（含根删除 = 清空整树的 Chromium 沙箱根扩展）
4. `getUniqueId()`（路径+kind 键控缓存，file/dir 同路径异 ID）
5. `resolve(child)`（后代路径数组 / 非后代 null）
6. truncate 钳指针 + 显式 position 不前进指针 + keepExistingData 快照 + 单 close 成功
7. removeEntry 守卫序：有打开流 NoModificationAllowedError 先于非空 InvalidModificationError
8. 已删除路径 listEntries → NotFoundError；createWritable 于已删除路径 → NotFoundError
9. getFile() Blob 元数据（name/lastModified）
10. 全局 FileSystemFileHandle / FileSystemDirectoryHandle 构造器（instanceof 断言面）
