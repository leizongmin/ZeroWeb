# Shaping cache key 必须包含 face 元数据

- 日期：2026-08-13
- 相关模块：`zero-render-foundation`、`zero-wpt-runner`

## 问题描述

字体 shaping cache 改为按字体字节内容寻址后，css-fonts Oracle 的 `--jobs 1` 与
`--jobs 8` 结果不同。同一测试单独运行稳定，但在完整目录并行运行时会得到更好的假结果。

## 根因分析

不同 `@font-face` 可以引用相同字体字节，同时声明不同的 `size-adjust`。duplicate
loader 共享 cache，而旧 key 只有字体字节 hash，没有 face descriptor。先完成 shaping
的 loader 会把结果错误复用给另一个 loader，最终数值取决于并行调度顺序。

## 解决方案

内容寻址 key 除字体字节 hash 外，还必须包含所有会改变 shaping 输出的 face 元数据。
本次将每个 face 的 `size-adjust` scale 纳入 key，并用共享 cache、相同字体字节、
不同 descriptor 的双 loader 测试锁定隔离行为。

缓存优化验收必须同时比较单案、目录 `--jobs 1` 和目录默认并行结果；通过率相同但单案
像素比例不同也应视为失败。
