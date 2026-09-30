---
date: 2026-09-30
modules: build-support,crates/script-sandbox
---

# git worktree 里构建 rusty_v8：`.cargo/rusty_v8/archive` 符号链接不随 worktree 继承

## 问题描述

在 git worktree（如 `/tmp/` 下的修复基线 worktree）中执行 `cargo run --release --bin
zero-wpt-runner`（任何启用 v8 默认 feature 的构建）失败，报
`v8 build.rs:937 NotFound`。把主仓的
`target/release/gn_out/obj/librusty_v8.a` 拷进 worktree 同路径后**仍然失败**，
容易误判为「gn_out 产物缺失」而继续在 target 目录打转。

## 根因分析

`.cargo/config.toml` 设置 `RUSTY_V8_ARCHIVE = ".cargo/rusty_v8/archive"`（相对路径）。
主仓的这个路径本身是一个**符号链接**，指向用户级缓存
`~/.cache/zero-web/rusty_v8/<ver>/librusty_v8_release_x86_64-unknown-linux-gnu.a.gz`
（由 `make setup-rusty-v8` 建立）。

- worktree 只继承 Git 追踪的文件；`.cargo/rusty_v8/archive` 符号链接不在 Git 内，
  worktree 的 `.cargo/rusty_v8/` 目录整个缺失。
- rusty_v8 的 build.rs 打开的是 **RUSTY_V8_ARCHIVE 路径**（解压 .gz 得到 .a），
  而不是 `target/.../gn_out/obj/librusty_v8.a`——后者是 build.rs 的输出物，
  手工拷贝它绕不过 build.rs 对 archive 的读取。

## 解决方案

在 worktree 内重建同名符号链接（共享用户级缓存是安全的，只读消费）：

```bash
mkdir -p <worktree>/.cargo/rusty_v8
ln -s "$HOME/.cache/zero-web/rusty_v8/v150.2.0/librusty_v8_release_x86_64-unknown-linux-gnu.a.gz" \
      <worktree>/.cargo/rusty_v8/archive
```

之后 `cargo run --release` 直接走 archive 解压路径，无需重编 v8。

## 如何避免

- 在 worktree 开工 checklist 中加入「`.cargo/rusty_v8/archive` symlink 存在性」检查。
- 判别口诀：v8 build.rs NotFound 时先 `ls -l .cargo/rusty_v8/`，再考虑 target 产物。
