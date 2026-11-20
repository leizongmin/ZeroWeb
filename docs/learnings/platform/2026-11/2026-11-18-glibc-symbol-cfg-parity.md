---
date: 2026-11-18
modules: apps/browser, apps/renderer, crates/runtime-config
---

# glibc 专属分配器符号要跟 libc 依赖用同一个 cfg 门

## 问题

Windows release 构建（`scripts/browser.ps1`）先报：

```
error[E0433]: cannot find module or crate `libc` in this scope
  --> apps\browser\src\mem_trace.rs:110:33
```

给 browser 加上门禁后，脚本第二段构建（renderer/compositor/image-decoder）
又报同一形态的错误：

```
error[E0433]: cannot find module or crate `libc` in this scope
  --> apps\renderer\src\js_worker.rs:1451:33
```

两处都在 `maybe_trim` 里直接写 `unsafe { libc::malloc_trim(0) }`。

## 根因

`libc` 依赖的 cfg 和调用点的 cfg 不一致：

- `apps/browser/Cargo.toml` 只在 `cfg(unix)` 下引入 `libc`；
- `apps/renderer/Cargo.toml` 只在 `cfg(all(target_os = "linux", target_env = "gnu"))`
  下引入 `libc`。

但 `malloc_trim` / `mallinfo2` 本身只有 linux-gnu 的 glibc 提供（musl 无此符号，
macOS/Windows 没有对应单调用）。两个 bin 的 `mallinfo2_snapshot()` 已经正确标了
`#[cfg(all(target_os = "linux", target_env = "gnu"))]`，相邻的 trim 调用却漏了，
于是非 glibc 目标编译到不存在的 crate。

两处不是同一刀引入：renderer 的 `mem_reclaim::malloc_trim_now` 早就有门禁，
`js_worker` 的 T79 诊断副本和 browser 的 T79 副本各自复制时把门禁丢了。单包构建
在 Linux 上看不出来，只有 Windows 目标才会暴露。

## 修复

调用点按符号可用性加门，非 glibc 目标给 no-op 值，诊断行照常落：

```rust
#[cfg(all(target_os = "linux", target_env = "gnu"))]
// SAFETY: ...
let released = unsafe { libc::malloc_trim(0) };
#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
let released = 0;
```

保留 `released == 1` 判读，Windows 上自然输出 `released=false`；`mallinfo2`
本来就是 `n/a`，语义不冲突。

## 避免方式

- 写 glibc 专属调用（`mallinfo2`、`malloc_trim`、`mallopt`）时，调用点 cfg 必须和
  `Cargo.toml` 中 `libc` 依赖的 cfg 逐字对齐；复制诊断代码时把 cfg 一起复制。
- 改完先在 Linux 之外的桌面目标构建对应 bin，单包构建不会被 workspace feature
  合并掩盖。
- 验证口径：`cargo build --release -p zero-browser --features zero-browser/windows-console`，
  再跑 `scripts/browser.ps1 -BuildOnly`。第二段构建单独暴露 renderer 侧遗漏，
  只跑第一段会漏检。
