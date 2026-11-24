---
date: 2026-11-24
modules: engine,build
---

# face 指纹探针进 bin：未引用 pub const 被 release DCE 剥离，须 #[used] static 字节数组

## 问题描述

kc5r82 刀需要验证"face 代码真的进了 runner bin"（防陈旧二进制冒充新刀）。在 `animation.rs` 加 `pub const KC5R82_FACE_MARKER: &str = "zw-kc5r82-2026-11-24";` 后，`strings bin | grep` 恒 0 命中——touch 重编两次仍缺，构建日志却显示 `Compiling zero-engine` 正常。

## 根因分析

`const` 是编译期值：未被任何运行时路径引用时，LLVM 在 release 优化下把数据段里的字符串副本一并删除（dead code elimination）。`pub` 只影响 rustc 的名称解析与 lint（不触发 `dead_code` 警告），不约束 LLVM 的物化——**lib 内 pub const 不被下游引用＝零代码生成**。普通 `static` 未引用时同样可被剥；`include_str!` 桥层 JS 字面（shim part03.js 先例）因为是运行时被读的字符串数据，天然物化，所以旧指纹门从未踩过这个坑。

## 解决方案

指纹探针用自包含字节数组 + `#[used]`（强制 LLVM 保留该 static，Rust 1.30 起稳定）：

```rust
#[used]
pub static KC5R82_FACE_MARKER: [u8; 20] = *b"zw-kc5r82-2026-11-24";
```

验证链：b0（无标记树）strings 计数=0、b1（face 树）计数≥1、b0/b1 md5 必须不同（防"刀未进 bin"的恒等假绿）。三重门齐才算配对构建就绪。

## 验证

kc5r82 实测：const 形两次构建 strings 恒 0；改 `#[used] static [u8;20]` 后一次构建 b1 指纹=1、b0 指纹=0、md5 分离（37582004…/d1818ba4…），电池判读据此采信 b1 含刀。

## 避免方式

凡为构建/二进制法证加的源码标记，一律 `#[used]` + `static` 字节数组（或真正被运行时引用的字符串）；不要用裸 `pub const`，也不要在判定脚本里把"Compiling 计数"当作"已入 bin"的证据——前者只证明编译单元重编，后者必须 strings 实测。
