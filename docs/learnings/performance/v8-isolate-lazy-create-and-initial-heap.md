# V8 isolate 懒创建 + 初始堆限制降低 WebView 常驻内存

- 日期：2026-08-09
- 相关模块：`zero-webview`（webview.rs）、`zero-script-sandbox`（lib.rs / v8_runtime.rs / worker.rs）
- 类型：性能优化

## 问题描述

`make test` 内存占用高：16 核机器上全树 RSS 峰值 ~9.5GB、单测试进程峰值 3.5GB。
经采样归因（`ps` 树 + 每 200ms 快照），大头是测试进程，而测试进程的内存主体是
每个 WebView 无条件创建的 V8 isolate 与页面渲染管线。

## 根因分析

1. **`WebView::new` 无条件创建 V8Sandbox（V8 isolate）**：`webview.rs` 原先在构造
   时 `V8Sandbox::with_config(...).expect(...)`，即使页面没有任何 `<script>` 也会
   初始化 V8 平台 + 创建 isolate（~0.1-0.35G RSS/实例，视平台与页 JS 而定）。
2. **nextest 的并行放大**：nextest 默认 `test-threads = num-cpus`，一个测试二进制
   会被 N 个 worker 进程实例化并行跑（本机 18 进程），每个 worker 内再按
   test-threads 并行测试 → 内存线性叠加。
3. **V8 isolate 初始堆预提交**：`SandboxConfig.heap_limit` 只设上限（默认 0 = 不
   设），initial heap 由 V8 按系统内存推导（大内存机器上初始预提交更大）。

## 解决方案

### 1. 懒创建 isolate（行为等价）

`webview.rs` 的 `js_sandbox` 改为 `None` 起步，新增 `ensure_sandbox()`：

```rust
fn ensure_sandbox(&mut self) -> Result<(), WebViewError> {
    if self.external_script.is_some() {
        return Err(WebViewError::Script("no js sandbox".to_string()));
    }
    if self.js_sandbox.is_some() { return Ok(()); }
    // 首次实际执行脚本时才创建 isolate
    ...
}
```

三个入口接 `ensure_sandbox()`：`execute_script` / `run_page_scripts` / `dispatch_event`。
无脚本页面（多数测试页、简单嵌入页）全程不建 isolate；有脚本页面首次执行时创建，
总成本不变、语义不变（`run_page_scripts` 在无脚本时本就提前返回）。

### 2. 初始堆限小（`SandboxConfig.initial_heap_size`）

`SandboxConfig` 新增 `initial_heap_size` 字段（0 = V8 默认），V8 后端创建 isolate 时
经 `heap_limits(initial, max)` 传入。WebView 的 sandbox 配置 128MB 初始堆（页面轻 JS
场景够用，堆按需增长）。

### 3. 关键坑：V8 `SetHeapLimits` 的 CHECK

```c++
// v8 内部
CHECK(initial_heap_size_in_bytes <= maximum_heap_size_in_bytes);  // 违反即致命崩溃 (SIGTRAP + core)
```

`heap_limits(128MB, 0)` 会触发该 CHECK 崩溃（128MB > 0）。修复：`heap_limit = 0`
（无上限）且设置了 `initial_heap_size` 时，max 显式取 4GB（V8 默认上限量级，实际
不会触发，仅满足 CHECK）。逻辑收敛为共享函数 `v8_heap_limits(&config) -> Option<(usize, usize)>`
（`None` = 全默认，不调用 `heap_limits`，与旧行为一致）。注意该函数仅在 v8 feature
下编译（quickjs 模式 `-D dead_code` 会炸），须 `#[cfg(feature = "v8")]`。

## 效果（本机 16 核 / 46GB）

| 指标 | 优化前 | 优化后 |
|---|---|---|
| 单进程 RSS 峰值 | 3.48G | 3.02G（-13%） |
| v8 测试期 total RSS 中位数 | 4.38G | 3.04G（-31%） |
| 14231 测试墙钟 | 90.3s | 86.4s（-4%，懒创建跳过无脚本页的 isolate 初始化） |

全部 15934 测试通过（v8 + quickjs 矩阵），wpt-runner 131 测试、integration 735 测试
通过——行为等价。

## 经验

- **V8 isolate 是"按需"资源**：无脚本页面/后台页面不要无条件创建 isolate；首次
  执行脚本时创建，成本只发生在实际需要处。
- **V8 `SetHeapLimits` 要求 `initial <= max`**：传 0 给 max 表示"不设置"，与正数
  initial 组合会触发 CHECK 致命崩溃（SIGTRAP + core dump），调试时注意清理
  `core.*` 残留。
- **nextest 内存归因**：`test-threads` 是每二进制内的测试并行数；worker 进程数 ≈
  核数，同一二进制会被多个 worker 实例化并行跑。降低内存要么降单实例占用（推荐），
  要么降并行（时间线性劣化，不推荐）。
- **quickjs 模式下 v8 专用代码必须 `#[cfg(feature = "v8")]`**，否则
  `-D dead_code` 在 quickjs 编译矩阵里变 error。
