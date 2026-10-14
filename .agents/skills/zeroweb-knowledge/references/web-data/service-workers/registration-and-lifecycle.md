# 注册、安装、激活、更新与 controller

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取 register/update/unregister、install/activate waitUntil、controller 与恢复问题。注册数据在 storage，状态仲裁在 ServiceWorkerManager，真实脚本在 ServiceWorkerRuntime；browser owner 持有 profile 注册数据，renderer host 托管脚本执行。

## Gotchas
- 注册校验接受 HTTPS 或 HTTP localhost/loopback，script/scope 必须 http(s)、与可信 document 同源且不含编码路径分隔符；默认 scope 来自 script 目录，当前校验不允许 scope 超出该目录。 (`crates/page-runtime/src/service_worker_registration.rs`)
- 注册请求提交脚本 URL 而非脚本源码；browser 获取脚本后仍校验成功响应、JavaScript MIME、UTF-8、大小和 final URL，同源注册参数不等于网络返回已经可信。 (`apps/browser/src/service_worker_owner.rs`, `crates/protocol/src/tests/service_worker_protocol.rs`)
- 同一 origin/scope 有 installing、waiting、active 版本槽，网页发现只返回一个代表版本；替换安装期间仍以原 active 为代表，不能把 registration id 当成永不变化的单槽。 (`crates/page-runtime/src/service_worker_manager.rs`)
- install waitUntil 拒绝会废弃新版本并保留旧 active；activate waitUntil 拒绝在 poll 中仍按成功推进替换，生命周期结果 succeeded=false 不等于回滚激活。 (`crates/page-runtime/src/service_worker_manager.rs`)
- 初次成功安装没有旧 active 时自动激活；替换版本通常停在 waiting，skipWaiting 或宿主 activate_waiting 才推进，不能仅凭 Installed 推断 controller 已更新。 (`crates/page-runtime/src/service_worker_manager.rs`)
- controller 查询允许 Activating 或 Activated，而 fetch 控制查询要求 Activated；替换开始会先转移既有 client 的 controller，所以两个 API 在激活过程中结果不同。 (`crates/page-runtime/src/service_worker_manager.rs`)
- unregister 删除未来匹配，但已有受控 client 继续使用 incumbent active；最后一个 client 移除后 poll 才收集 runtime，不能在 unregister 返回时立即判内存泄漏或强停受控脚本。 (`crates/page-runtime/src/service_worker_manager.rs`, `git:bf4b2578755ca91df46fd1a5db267c68c98537ed`)

## Architecture
- 更新比较主脚本及已导入脚本图，主文件字节相同不能代表版本未变；updateViaCache 的 Imports/All/None 分别决定主脚本和 import 是否绕过 HTTP cache。 (`crates/page-runtime/src/service_worker_manager.rs`, `crates/storage/src/service_worker.rs`)
- 恢复导出仅包含 active 注册的脚本、import source、script type、updateViaCache 与 registration-local cache 快照；恢复运行不重放 install/activate，不能持久化整个运行中线程状态。 (`crates/page-runtime/src/service_worker_manager.rs`, `crates/webview/src/tests/service_worker_runtime.rs`)
- browser 普通 profile 共用注册 manager，Private(TabId) 每标签独立且不进入 persist_normal；这与网络 private loader 按 private 分区共享的粒度不同。 (`apps/browser/src/service_worker_owner.rs`, `apps/browser/src/fetch_proxy.rs`)
- runtime 默认脚本超时 5 秒、heap 上限 64 MiB，manager 另限制 32 runtimes、脚本与依赖图体积；不能只限制注册条数而忽略每版本资源和事件预算。 (`crates/script-sandbox/src/service_worker.rs`, `crates/page-runtime/src/service_worker_manager.rs`)
