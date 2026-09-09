# M3 — skip 清单正式化 + createSyncAccessHandle 评估定论（storage-opfs）

日期：2026-09-09

## 1. createSyncAccessHandle 评估定论

**判定：记入 skip（不做）。**

评估理由（对照 Support Envelope 覆盖范围 5「Worker 环境专用，评估后决定做或记入 skip」）：

1. **域归属**：`createSyncAccessHandle` 仅在 Dedicated Worker / Shared Worker 环境开放
   （spec https://fs.spec.whatwg.org/#api-filesystemfilehandle-createsyncaccesshandle——
   window 环境调用直接抛 TypeError）。它的语义面完全服务于 worker 内同步文件 IO。
2. **环境前提缺失**：本仓 headless runner / WebView 的 Worker 面无真线程（shim 近似，
   部分桥接路径为 stub）。同步句柄的核心价值（worker 线程内阻塞式读写、与主线程
   `createWritable` 的互斥锁语义 `readwrite`/`read-only`/`readwrite-unsafe`）在无真线程
   环境下无法成立——即使实现也只是无意义的同步近似。
3. **WPT 验证面**：上游 `FileSystemSyncAccessHandle-*.https.worker.js` 五用例全依赖
   worker 环境执行（`.worker.js` 变体），且本流已导入的 13 个 window 用例中无 SAH 断言
   ——不做 SAH 不影响现有通过率分母。
4. **上游先例**：与 postMessage transferable（同列 Support Envelope 排除域）同级——
   依赖运行环境能力（真线程 / 结构化克隆）而非纯语义实现。

**重启条件**：若未来 Worker 面落地真线程（worker goal 的后续架构演进），SAH 可作为
该 goal 的附带项实现——本 goal 不阻塞父目标 Tier 2 存储收口。

## 2. skip 清单正式化（不充数、不误排除）

当前 13 用例 / 133 subtests 分母内，**通过率统计的排除项**（均为域外依赖，非本 goal 语义缺口）：

| # | 排除项 | subtests | 排除理由（归属域） |
|---|---|---:|---|
| S1 | isSameEntry with a *handle cloned via postMessage*（×3） | 3 | 结构化克隆 transferable 面（engine 克隆基建域）；OPFS 侧 isSameEntry 语义本身已实现且被其余 7 个 isSameEntry subtest 断言通过 |
| S2 | createWritable() can be called on two handles | 1 | 上游 pinned-rev（31597693）用例签名缺陷：`createDirectory(t, 'parent_dir', root)` 三参调用与同 rev helper 二参签名不符（上游后续 rev 已修，helper 改变参）；任何实现下该用例必炸 |
| S3 | write() with an invalid blob to an empty file should reject | 1 | blob 失效快照检测（getFile 后 removeEntry 再写）——headless blob 同步字节取无失效语义；低优先深化项，保留在通过率分母内（未排除，如实计为失败） |
| S4 | piped: plays well with fetch / abort() aborts write（×2） | 2 | fetch `response.body` 流面（fetch/stream 域，`data:` URL body 为 null）；pipeTo 本身已实现且被其余 6 个 piped subtest 断言通过 |

**汇总**：S1（3）+ S2（1）+ S4（2）= 6 subtests 属域外依赖；S3（1）保留分母如实计失败。

## 3. 域外 fetch 脚本头注释（fetch-fs-subset.sh）复核

fetch 脚本头注释已列 postMessage*/create-sync-access-handle*/move/FileSystemObserver*/
IndexedDB/buckets/opaque-origin/bfcache/idlharness 十类 skip 域（导入前排除，不进分母），
与本文件 §2 的「分母内排除」互补：前者是用例级不导入，后者是 subtest 级已知失败归类。
