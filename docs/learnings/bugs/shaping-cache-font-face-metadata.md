# Shaping cache key 必须包含 face 元数据

- 日期：2026-08-13
- 相关模块：`zero-render-foundation`、`zero-wpt-runner`

## 问题描述

字体 shaping cache 改为按字体字节内容寻址后，css-fonts Oracle 的 `--jobs 1` 与
`--jobs 8` 结果不同。同一测试单独运行稳定，但在完整目录并行运行时会得到更好的假结果。

## 根因分析

不同 `@font-face` 可以引用相同字体字节，同时声明不同的 `size-adjust` 或
`unicode-range`。duplicate loader 共享 cache，而旧 key 只有字体字节 hash，没有完整
face descriptor。

此外，cache value 中的 `ShapedGlyph` 保存 loader-local `font_id`。两个 loader 若以不同
顺序加载相同字体，即使字节 hash 相同，同一个数字 ID 也可能指向不同字体。先完成 shaping
的 loader 会把 glyph size、fallback face 或 local ID 错误复用给另一个 loader，最终数值
取决于并行调度顺序。

## 解决方案

内容寻址 key 除字体字节 hash 外，还必须包含所有会改变 shaping 输出的 face 元数据。
R3373-F 将每个 face 的 `size-adjust` scale 纳入 key；R3409-F 继续纳入
`unicode-range` 和 loader-local `font_id`。回归测试使用共享 cache 与相同字体字节，
分别配置不同 scale、相反 range 和相反加载顺序，锁定 glyph size、fallback face 与
cache value ID 的隔离行为。

缓存优化验收必须同时比较单案、目录 `--jobs 1` 和目录默认并行结果；通过率相同但单案
像素比例不同也应视为失败。
