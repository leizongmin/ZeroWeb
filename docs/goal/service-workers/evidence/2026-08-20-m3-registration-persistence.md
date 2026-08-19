# M3-7 Service Worker Registration Persistence

**日期**：2026-08-20
**状态**：complete

## 实现

- production `BrowserServiceWorkerOwner` 是 normal profile 持久化的单一写入者；renderer、
  WebView adapter 和 private tab 不写共享文件。
- active registration 持久化为 versioned pure-value snapshot：canonical `origin`、`scope`、
  top-level `script_url` 和经过 UTF-8 校验的原始 `script_source`。
- 默认路径位于平台 data directory 的
  `ZeroBrowser/Storage/ServiceWorkers/registrations.json`；设置 `ZERO_STORAGE_DIR` 时使用
  该目录下的 `service-workers.json`。
- activation 与 unregister 后使用 temporary file、file sync、atomic rename 和 Unix
  directory sync 提交；恢复多条记录时等待全部 settle 后才重写，避免 partial restore
  截断尚未处理的有效记录。
- 启动恢复重新 evaluate top-level script 以注册 fetch/message handlers，随后直接恢复
  active slot；不会重放 `install`、`activate`、`waitUntil()` 或 `clients.claim()`。
- 不恢复 runtime heap、event queue、Document client、message cursor 和 claim 状态。
  registration/version ID 仍是进程内 identity，由新 browser owner 重新分配。

## 信任边界

- schema version 必须为 1；文件和 script source 总量最多 64 MiB，registration 最多 32 条。
- 每条 URL 必须 canonical、secure-context compatible、同源且 scope key 唯一。
- 缺失文件视为空 profile；损坏、超版本、超限或非法 URL 文件 fail closed，不阻断浏览器。
- 单条 script compile failure 只移除该记录；其他 scope 完成恢复后提交新的有效 snapshot。
- `ZERO_PRIVATE=true` 和独立 private tab 都使用纯内存 owner，不读取或写入 normal snapshot。

## 回归

- page-runtime：persisted active runtime 恢复为 `Activated`，且 event log 中没有
  `LifecycleSettled`、`InstallCompleted` 或 `ActivationCompleted`。
- browser owner：写盘后销毁/recreate owner，恢复 active registration、controller 和
  update target；unregister 后再次启动不复活。
- multi-record：两个 scope 中一个脚本被改成 compile failure，另一个仍恢复，最终文件只
  保留有效 scope。
- fresh renderer：销毁第一组 `ProcessTabBackend`/renderer 后，以同一 persistence file
  创建第二组；不重新 fetch script，匹配 Document 在 commit 时获得 activated controller。
- private/invalid：private registration 不创建 normal 文件；unsupported schema 不进入
  normal manager。
- core WPT：13/13 case、37/37 subtest Pass，连续两轮 deterministic。
- `make test`：V8 WebView 620/620、QuickJS 573/573、QuickJS WPT runner 113/113、
  adapter GPU 94/94 和 fresh peers 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## 性能

`make bench-gate` 报告 `benchmark_20260820_065515.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- startup：94.53 ms，peak RSS：153.94 MiB；
- page total p95：15.17 / 428.75 / 117.60 ms；
- retained form p95：0.0357 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。
