---
date: 2026-11-04
modules: engine, testing
---

# SIGKILL 构建链可在 target/debug 留下"指纹新鲜、内容陈旧"的投毒构件，编译假红要先用 clean -p 验伪

## 问题

selection-compat M1 收口轮跑 `make test`，workspace 腿编译失败：
`zero-layout-engine (lib)` 21 错 + `zero-style-system (lib test)` 7 错 +
`(lib test)` 32 错，全部是 `AlignmentValue::SafeStart` /
`taffy::style::AlignItems::SAFE_START` / `zero_css_parser::values::CaretShapeValue`
"不存在"型 E0599/E0433——而这些符号在当前源码里都有（R5401 `c5de82c9e`、
caret-shape `a43e8a52c` 均已在 main）。rustc 的 help 甚至建议出旧世界名字
（`AlignmentValue::FlexStart`、`CornerShapeValue`），即编译单元链接到的
zero-css-parser 单元是**旧 API 时代的产物**。与此同时 `cargo check
--workspace --all-targets` 全绿，单包 check 也绿。

## 根因

同 session 的上一轮 rally 实例（僵尸）在被终止前有过被 SIGKILL 的
test-guard 构建链（test-guard 重建竞态，整链 kill）。rustc 被杀时正在
写出的构件/指纹状态落在了中间态：`target/debug` 里 zero-css-parser 的
构件内容是旧源码时代的 rlib/rmeta，而指纹账面（.fingerprint）声称对
当前源码新鲜。于是：

- `cargo check`（rmeta 校验路径）通过——check 自己的单元是新鲜的；
- `cargo test`（真实 codegen + 链接）消费到投毒的下游构件——下游
  （style-system/layout-engine 对 css-parser 的 API 引用）对着旧 API
  报"符号不存在"。

错误形态极具迷惑性：报错文件与行号内容是**当前**源码（converter/mod.rs
的 SafeStart 行原样出现在 error snippet 里），缺的却是**依赖 crate 里**
的符号——让人先怀疑 patch/lockfile/feature 统一，而不是构件陈旧。
清空 `target/debug/incremental`（4.1G）不能修复——投毒不在增量缓存。

## 解决与避免

- 判别试验：`CARGO_INCREMENTAL=0` 重跑同样失败面 → 62 错全部消失
  （构建干净，只会因守卫内存帽被打断）→ 与增量无关，指向构件投毒。
- 确认当前源码确实含被报"不存在"的符号（grep 源码 + `git log -S`）
  之后，`cargo clean -p <上游包> -p <报错包>` 定向清构件，复跑即绿。
- 遇到"check 绿 / test 编译红"且报错指向依赖 crate 缺符号时，第一
  反应应该是构件陈旧而非代码问题，先 `cargo clean -p` 再排查别的。
- SIGKILL 构建链（守卫超时、竞态清理、OOM 杀树）之后，同 target 目录
  的下一次完整门禁前建议先定向 clean 刚在编译的包；盲目清整个
  target/debug 代价高，`clean -p` 三包 ~7G 即可。
- 与 [include_str! shim 二进制谱系](2026-11-04-include-str-shim-binary-lineage-ab.md)
  同族但不同层：那条是"磁盘二进制代际与源码脱节"，这条是"cargo 自身
  构件账面被杀写损坏"；共同教训是**账本声称的状态 ≠ 磁盘内容**，
  关键判读前先做内容指纹级验证（strings token / 干净重建 A/B）。
