# V8、QuickJS 与沙箱宿主回调

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 `Sandbox`、`SandboxConfig`、`execute_json`、`register_callback` 与超时实现。这里描述执行器，不代表宿主已安装页面 API。

### User Task Navigation

| 用户问题 | 源码与判据 |
| --- | --- |
| 重复执行后回调变慢、旧函数调用异常 | `v8_runtime.rs` / `quickjs_runtime.rs` 的回调注册及同名替换测试 |
| 引擎组合与 feature 优先级 | [跨宿主脚本 feature](../../runtime/conditions/script-features.md) |
| 产品验证 | [zeroweb-verify](../../../../zeroweb-verify/SKILL.md)，初始化资产已在 d0f281d509de3bf4f9dcd7e8dd01dac3464a663b 提交，本域未实跑 |

## Gotchas
- `persistent_context` 默认关闭，每次执行重新建 Context；WebView 和页面 worker 显式开启持久上下文，因此孤立沙箱测试的全局变量寿命不能代表页面。 (`crates/script-sandbox/src/lib.rs`, `crates/script-sandbox/src/v8_runtime.rs`, `crates/script-sandbox/src/quickjs_runtime.rs`, `crates/webview/src/webview.rs`)
- 同名 `register_callback` 替换原注册而非追加；页面每段脚本重注册宿主桥，追加会使全量重挂成本随脚本数形成二次方增长。 (`crates/script-sandbox/src/v8_runtime.rs`, `crates/script-sandbox/src/quickjs_runtime.rs`, `git:68d883466ee74d26c84c0cd3d4c399d32e9fc96f`)
- V8 回调表属于线程 TLS 而非单个沙箱；Drop 只清本实例拥有的槽位且不复用空洞，整表清空或复用洞会使其他仍存活沙箱的函数越界或撞写。 (`crates/script-sandbox/src/v8_runtime.rs`, `git:fba2e0f78cf5b6a10151fc14f05a442ab66551a7`)
- JS 捕获旧函数后，同名替换的跨引擎语义不同：V8 函数经固定槽位找到最新闭包，QuickJS 函数持有旧 Arc；仅比较注册名调用结果会漏掉此差异。 (`crates/script-sandbox/src/v8_runtime.rs`, `crates/script-sandbox/src/quickjs_runtime.rs`, `git:fba2e0f78cf5b6a10151fc14f05a442ab66551a7`)
- `reset_context` 丢弃 JS Context 但保留宿主回调；原生 DOM 的 TLS 缓存必须由上层另外 reset，不能把 Context 重建当成全部宿主状态已清。 (`crates/script-sandbox/src/v8_runtime.rs`, `crates/script-sandbox/src/quickjs_runtime.rs`, `apps/renderer/src/js_worker.rs`)

## Architecture
- V8 超时由持久 watchdog 接收 Arm/Disarm 序号，旧任务的撤销不能解除新任务的武装；QuickJS 用 interrupt handler 检查 deadline，并通过作用域守卫在错误返回时清理 deadline。 (`crates/script-sandbox/src/v8_runtime.rs`, `crates/script-sandbox/src/quickjs_runtime.rs`)
- 原生绑定安装是 `Sandbox` 的持久 Context 扩展入口，DOM 实现仍归 engine；`script-sandbox/src/dom_bindings.rs` 是低层 PoC，不是生产 DOM API 的安装者。 (`crates/script-sandbox/src/lib.rs`, `crates/script-sandbox/src/dom_bindings.rs`, `crates/engine/src/dom_bindings/mod.rs`)
- `execute_json` 才提供对象 JSON 序列化契约；普通 `execute` 的对象字符串化不能作为 ES module namespace 的结构化输出。 (`crates/script-sandbox/src/es_module.rs`, `crates/script-sandbox/src/v8_runtime.rs`, `crates/script-sandbox/src/quickjs_runtime.rs`)
