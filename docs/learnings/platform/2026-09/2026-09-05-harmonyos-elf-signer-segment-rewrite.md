---
date: 2026-09-05
modules: release, npm-cli, harmonyos
---

# HarmonyOS ELF 签名工具可能重写可加载段

## 问题描述

将 `aarch64-unknown-linux-musl` 静态 ELF 交给官方 Java `binary-sign-tool sign -selfSign 1` 后，`display-sign` 返回成功，HarmonyOS PC 直接执行也不再报 `Permission denied`，但进程立即以 139（`SIGSEGV`）退出。第三方用户态 ELF loader 执行同一程序正常。

## 根因分析

官方签名工具重建 ELF 时移动了包含 TLS、GOT、RELRO 和全局数据的可写 `PT_LOAD` 文件偏移，但没有同步改变虚拟地址。目标机器页大小为 4096，签名后：

```text
p_offset % 4096 = 0xe48
p_vaddr  % 4096 = 0x7c8
```

这破坏了加载映射要求。签名工具虽然输出多个 section/segment changed 警告，最终仍返回 `sign success`；`display-sign` 只验证签名内容，不能证明 ELF 仍可正确加载。

社区 `codex-harmonyos` 使用来自 `ohos-bst-light` 的 append-only 签名器：在文件尾部追加 `.codesign` 并搬移 section-header table，不重写 program headers。同一个未签名 ELF 经该实现处理后，官方 `display-sign` 验证通过且所有 `PT_LOAD`、TLS、RELRO header 保持不变。

## 解决方案

- 长期方案：使用 OHOS SDK 构建真正的 `aarch64-unknown-linux-ohos` ELF，再用官方工具签名。
- 短期兼容 Linux-musl 预编译 ELF：使用固定、已审计版本的 `ohos-bst-light` append-only 签名器，并做鸿蒙 PC 真机验证。
- 发布门禁必须比较签名前后 program headers，并校验每个 `PT_LOAD` 的 `p_offset % page_size == p_vaddr % page_size`。
- 将签名工具输出的 section/segment changed 警告视为失败，不能只检查退出码、`.codesign` 是否存在或 `display-sign success`。
