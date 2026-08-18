---
date: 2026-08-14
modules:
---

# 系统字体按需解析：避免启动时展开全量字形轮廓

## 问题描述

renderer 和 compositor 改为加载完整平台字体集后，即使只打开简单页面，两个进程的
Private Bytes 合计也从几十 MB 上升到约 677 MB。空闲且没有页面帧时，单个进程仍占用
约 300 MB，说明增长发生在启动初始化而非页面绘制或图像缓存阶段。

## 根因分析

`FontLoader::load_font_at_index` 在注册每个系统字体时立即调用
`fontdue::Font::from_bytes`。fontdue 会解析 cmap/GSUB，并把该 face 的全部 glyph
轮廓展开到堆内存。Windows 默认字体集中包含约 3 万 glyph 的微软雅黑和约 6 万 glyph
的 Segoe UI Emoji，因此每个进程仅注册字体就会产生数百 MB 常驻内存。

只把 fontdue 初始化延迟到“首次字符命中”仍然不够：一个汉字就会触发整张 CJK face
的全量轮廓展开。

## 解决方案

- 注册阶段只保留字体原始字节和 face index，并用 ttf-parser 做轻量格式校验。
- 字体覆盖判断、行度量和 shaping 度量直接读取原始 sfnt 表。
- 常规栅格化继续走已有 FreeType face 缓存，不创建 fontdue 实例。
- 仅无 FreeType 的纯 Rust 回退或显式调用旧 `get` API 时，通过 `OnceLock` 首次解析并缓存
  fontdue face；`duplicate` 共享该状态，避免重复解析。

## 效果

同一台 Windows 机器、release 构建、相同简单页面下：

| 进程 | 修复前 Private Bytes | 修复后 Private Bytes |
|---|---:|---:|
| renderer | 约 340.5 MB | 67.6 MB |
| compositor | 约 336.5 MB | 63.7 MB |
| 合计 | 约 677 MB | 约 131 MB |

无页面/IPC 的进程空闲态分别降到 renderer 44.8 MB、compositor 36.1 MB。

## 如何避免

大字体的“注册”“查询元数据”和“展开所有字形轮廓”必须是三个独立阶段。启动路径只应
执行前两者；评审字体库调用时，要确认构造函数是否隐含全字体 outline 预解析，并用
CJK/Emoji 系统字体做进程级 Private Bytes 回归测试。
