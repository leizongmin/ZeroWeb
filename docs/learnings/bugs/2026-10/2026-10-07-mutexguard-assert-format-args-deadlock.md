---
date: 2026-10-07
modules: engine
---

# MutexGuard 借用穿过 assert! format_args 后 NLL 不释放 → 同线程二次 lock 死锁

## 问题

`js_dom_bridge_tests/part02.rs` 的 `test_console_host_bridge_r3256` 扩展后挂死：测试进程 18 线程全部 `futex_do_wait`、主线程 Sleeping、CPU 时间零增长（13 分钟 utime/stime 不变）。测试逻辑无循环、无显式等待，`--nocapture` 探针显示挂点在两次 `captured.lock().unwrap()` 之间。

问题代码形态（简化）：

```rust
let got = captured.lock().unwrap();          // MutexGuard 绑定变量
let (_, _, values) = &got[before];           // values 借用 guard
assert!(values.contains(...), "got: {values}");  // 借用穿过 assert! 的 format_args!
// 期待 NLL 在最后一个 values 使用后释放 guard
let before = captured.lock().unwrap().len(); // ← 同线程二次 lock，futex 死锁
```

## 根因

`assert!` 展开为 `format_args!`，消息参数（`{values}`）借用 guard。edition 2024 下该借用在 MIR 中存活越过肉眼预期的 NLL 释放点（format_args! 参数临时值作用域规则与 panic 路径借用延长），guard 的 drop 点被推迟到后续语句之后——`lock()` 自死锁。Rust 的 `Mutex` 非重入，同线程二次 `lock()` 是未定义行为，实测为永久 futex 等待、零 CPU。

独立最小复现（`rustc --edition 2024`，10 行）：

- 显式 `drop(got)` 再 `lock()` → 正常退出（exit=0）
- 删掉 `drop(got)` 依赖 NLL 隐式释放 → 5 秒 timeout 杀掉（exit=124，死锁复现）

## 解决方案

测试中所有「lock 后读值再断言」一律 `lock().unwrap().clone()` 立即释放 guard，断言作用于克隆值：

```rust
let got = captured.lock().unwrap().clone();  // guard 立即释放
let (_, _, values) = &got[before];
assert!(values.contains(...), "got: {values}");  // 借用的是本地 clone，无锁关联
```

修复后该测试 0.05s 通过。

## 如何避免

- 持 `MutexGuard` 时不要让借用穿过 `assert!`/`panic!`/`format_args!` 等格式化宏；要格式化引用锁内数据，先 clone 出局部值。
- 判据：同线程挂死 + `futex_do_wait` + CPU 时间零增长 = 典型二次 lock 自死锁，先查 guard 生命周期，不必怀疑业务逻辑。
- 编译器对「guard 未及时释放」不报错不警告（借用检查合法，只是 drop 晚于预期）；`drop(got)` 显式释放是最便宜的保险。
