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

## 变体：assert_eq!/assert! 宏的格式化参数二次 lock（2026-11-12 补录）

同一生命周期规则在宏展开下更隐蔽。`assert_eq!(m.lock().unwrap().len(), 0, "…{:?}", m.lock().unwrap())` 形态：`assert_eq!` 展开为 `match (&(左值), &(右值)) { … }`，判别式里第一个 `lock()` 的 guard 临时存活到**整个 match 语句结束**；断言**失败**时 panic 分支才求值格式化参数，此时第二个 `lock()` 与仍存活的判别式 guard 自死锁。症状比 match 变体更迷惑：断言通过时一切正常，只有「失败路径」挂死——测试不报失败也不报超时，libtest 只显示 `has been running for over 60 seconds`。

- 规则同一条：**凡宏展开会出现同一表达式求值两次以上、且其中一次可能创建 guard**，先 let 绑定（或 `clone()` 出 owned 值）再进宏。测试代码同样适用（html-semantics s409 桥测实测：断言失败触发 panic 格式化路径即挂，pass 路径永不暴露）。
- 诊断变体：宏内失败路径挂死优先读宏展开（`cargo expand` 或直接读 std 源）确认临时存活域；测试二进制挂死且 yama `ptrace_scope=1`（gdb 只许直系父进程 attach、后台 setsid 进程不可及）时，用**相位探针**替代——测试内 `AtomicUsize` 相位标记 + 看门狗线程超时打印并 `exit(9)`，等价定位挂点，无需 ptrace。
