# OPFS File System (fs/) WPT 基线 — 内存 shim 版（storage-opfs M1 切片 1）

- 日期：2026-09-09
- WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`（与 storage-indexeddb 导入 pin 同版）
- 执行通道：`make testharness-fs`（test-guard 包裹；wpt-runner `testharness-fs` 模式）
- 用例：13（window 变体；`.any.js` 由 runner 包装）
- Subtests：133（Pass 48 / Fail 85，通过率 36%）

**这是 JS shim 内存虚拟树（part02.js navigator.storage 段）的表现基线，即 M2/M3 真实化后的对照起点。**

## 分用例

| 用例 | Subtests | Pass | Fail |
|---|---:|---:|---:|
| `fs/root-name.https.any.js` | 1 | 1 | 0 |
| `fs/FileSystemBaseHandle-isSameEntry.https.any.js` | 14 | 8 | 6 |
| `fs/FileSystemBaseHandle-getUniqueId.https.any.js` | 11 | 0 | 11 |
| `fs/FileSystemBaseHandle-remove.https.any.js` | 9 | 0 | 9 |
| `fs/FileSystemDirectoryHandle-getDirectoryHandle.https.any.js` | 10 | 5 | 5 |
| `fs/FileSystemDirectoryHandle-getFileHandle.https.any.js` | 13 | 8 | 5 |
| `fs/FileSystemDirectoryHandle-iteration.https.any.js` | 6 | 0 | 6 |
| `fs/FileSystemDirectoryHandle-removeEntry.https.any.js` | 13 | 4 | 9 |
| `fs/FileSystemDirectoryHandle-resolve.https.any.js` | 5 | 0 | 5 |
| `fs/FileSystemFileHandle-getFile.https.any.js` | 3 | 0 | 3 |
| `fs/FileSystemWritableFileStream.https.any.js` | 9 | 2 | 7 |
| `fs/FileSystemWritableFileStream-write.https.any.js` | 31 | 20 | 11 |
| `fs/FileSystemWritableFileStream-piped.https.any.js` | 8 | 0 | 8 |

## 失败聚类（根因 → 影响 subtests）

| 根因聚类 | 影响 |
|---|---|
| 1. `keys/values/entries` 返回数组而非 async iterator（无 `[Symbol.asyncIterator]`/`next()`），目录本身无 `Symbol.asyncIterator` | iteration 5 + 各用例 cleanup/getSortedDirectoryEntries 依赖 → ~25 |
| 2. 错误类型：`TypeError` 应为 `DOMException`（NotFoundError code 8 / TypeMismatchError 17 / NoModificationAllowedError 7） | getDirectoryHandle/getFileHandle/removeEntry ~10 |
| 3. `handle.remove({recursive})`（FileSystemHandle.remove）缺失 | FileSystemBaseHandle-remove 10 |
| 4. `getUniqueId()` 缺失 | FileSystemBaseHandle-getUniqueId 11 |
| 5. `resolve(child)` 缺失 | FileSystemDirectoryHandle-resolve 5 |
| 6. writable 流不是真 WritableStream：`pipeTo` 拒绝、`getWriter` 缺失、queued 命令语义（close 二次 reject/锁释放）缺失 | piped 8 + write ~8 |
| 7. Blob write 路径空结果（`_zw_blobBytes` headless 近似返空）；keepExistingData 定位错误（`new Uint8Array(node.data)` 引用 vs 复制） | write blob / keepExistingData ~6 |
| 8. `getFile()` Blob 属性缺失（name/lastModified）+ slice 语义空 | getFile 3 |
| 9. truncate 语义：truncate(size) 后 pos 应保持（写 45 在 offset 3），shim 把 pos 钳到 size；write 前导零填允 vs 钳位 | write cursor ~4 |
| 10. postMessage 克隆句柄（DataCloneError）— transferable 面，Support Envelope 排除域 | isSameEntry 3（skip 候选） |

## 聚类 → 里程碑映射

- 聚类 1/2/3/4/5 → M2（接线后 Rust opfs 模块实现 async iterator + DOMException + remove/getUniqueId/resolve）
- 聚类 6/7/8/9 → M2（真 WritableStream 语义 + Blob 写路径 + getFile 元数据）
- 聚类 10 → skip list（浏览器 shell/结构化克隆域）
