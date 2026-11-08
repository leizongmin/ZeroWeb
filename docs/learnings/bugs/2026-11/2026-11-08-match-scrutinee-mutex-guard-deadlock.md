---
date: 2026-11-08
modules: net, wpt-runner
---

# match 判别式临时 MutexGuard 存活到整个 match——臂内再锁自死锁

## 问题描述

http-cache M1-b 在 wpt-runner 案级缓存包裹器（`wpt_cache_py_wrapped_handler`）写入如下形态后，request-cache 族整窗挂起直到超时：

```rust
match case_cache.lock().expect("...").lookup(&url, &req.headers) {
    CacheLookup::Hit(cached) => ...,
    CacheLookup::Revalidate { conditional_headers, .. } => {
        // 臂内再次 case_cache.lock() 调 not_modified/put_with_headers
    }
    ...
}
```

构建、clippy 全绿，无任何静态期信号，只有运行时整窗挂死。

## 根因分析

`match` 判别式中的临时值（此处是 `lock()` 返回的 `MutexGuard`）生命周期延长到**整个 match 表达式结束**——即使 `.lookup()` 返回的是 owned `CacheLookup`、guard 本身没被臂内引用，它仍在所有臂执行期间持锁。臂内对同一 `Arc<Mutex<...>>` 的任何再 `lock()` 都是与自己死锁（`std::sync::Mutex` 不可重入），同步阻塞无 panic、无 poison 信号。

`let lookup = case_cache.lock().expect("...").lookup(...);` 先绑定再 match 则 guard 在 let 语句结束即释放，臂内再锁安全。`resource_loader.rs` 的 `submit()` 之所以从未踩中，正是因为它一直是 let 绑定形态。

## 解决方案与避免方式

- 判别式调用返回 owned 值但**创建过 guard** 时，一律 let 绑定后再 match；判别式里只保留纯表达式（无锁、无借用持有）。
- 代码审查时对 `match x.lock()...` / `match self.cache.lock()...` 形态零容忍——构建器不报、测试也可能不踩（单线程路径首次锁即挂），是最隐蔽的死锁形态之一。
- `tracing`/println 不可达（挂死早于任何臂内语句）时，优先怀疑判别式临时存活问题，`ps` 看线程栈（`gdb -p <pid> -ex "thread apply all bt"`）确认双锁等待。
