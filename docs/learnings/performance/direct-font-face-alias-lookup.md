# 已规范化字体 alias 应直接查表

- 日期：2026-08-17
- 相关模块：`zero-render-foundation` 字体 face matching

## 问题描述

字体匹配每次先遍历 resolver 做大小写不敏感 base-key 匹配，再遍历 resolver 收集 `:face=N`。Painter 已维护全小写 resolver，FontLoader 也为同 family variant 注册精确 base alias 与连续 face aliases，这两次扫描没有提供额外语义。

## 根因分析

查询逻辑没有利用 resolver 的构造不变量。随着 author faces 和 fallback aliases 增加，每个文本 fragment 的 face matching 都退化为两次 O(n) 字符串扫描、临时小写转换和排序。

## 解决方案

精确 base key 命中时直接 HashMap lookup，并从 `:face=0` 起顺序读取连续 aliases；精确键缺失时保留原大小写不敏感扫描。`ZW_FONT_FACE_DIRECT_LOOKUP=0` 可恢复旧路径。快速路径必须只依赖注册端明确保证的连续索引，兼容路径不能删除。

## 验证

200-key production-like resolver 的百万次查询 checksum 均为 `2001000000`。两组反序 task-clock 从 `10.35→0.542s`、`7.96→0.532s`；medium profile 中 `lookup_faces` self 从约0.48%降至0.22%，旧全表 `filter_map` 栈退出。render-foundation `655/655`、reftest `687/687`、产品与完整性能门通过。
