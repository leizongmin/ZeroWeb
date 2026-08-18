---
date: 2026-08-13
modules: crates/canvas/src/context/context_impl.rs（CanvasContext::new / get_image_data / create_image_data）
---

# canvas ImageData 尺寸计算的 u32 溢出回绕

**调用链入口**: `crates/engine/src/js_dom_bridge/canvas.rs`（`getImageData` / `getContext2d` op，从 JS 经 wire 解析任意 `u32`）

## 问题描述

三处 RGBA 缓冲区大小计算沿用同一反模式：

```rust
// 旧实现（context_impl.rs）
let size = (width * height * 4) as usize;        // get_image_data / create_image_data
let buffer_size = (width as usize) * (height as usize) * 4;  // new()
```

其中 `width`、`height` 为 `u32`（JS `getImageData(x,y,w,h)` / canvas `width`/`height` 属性）。
`get_image_data` / `create_image_data` 的写法 `(width * height * 4) as usize` 是**先在 u32 域做乘法再转 usize**——
当 `width * height * 4` 越过 `u32::MAX`（最小触发点 `getImageData(0,0,65536,65536)` → `65536*65536*4 = 2^34`），
u32 中间结果**回绕为一个小值**（此处为 0）。

## 根因分析

- **回绕后果（get_image_data，确定性 panic）**：`data = vec![0u8; 0]` 分配 0 字节缓冲区，
  随后复制循环 `data[dst_start..dst_start+copy_len].copy_from_slice(...)` 在 `copy_from_slice` 的
  **长度检查**上 panic（`slice index out of bounds`）。这与「算术溢出」是两回事——切片边界检查
  在 debug 与 release 下**都触发**（release 不优化掉切片长度校验）。即：在有内容的小画布上调
  `getImageData(0,0,65536,65536)`，渲染进程 / tab 必然 panic 崩溃，DoS 级。
- **回绕后果（create_image_data / new）**：静默分配错误尺寸缓冲区，后续写入越界或内存状态错乱。
- `new()` 的写法虽已是 `(width as usize) * (height as usize) * 4`（usize 域），但 32-bit `usize` 平台
  仍会回绕；统一改 `saturating_mul` 既防 64-bit 回绕也防 32-bit 回绕。

### 为什么 debug 没在算术处拦下

`cargo test`（debug build）对**变量**的算术溢出会 panic，但本 bug 的复现需 `width*height*4` 真正越过
`u32::MAX`（最小 16GB 量级），CI 不会构造这么大输入；既有测试都用小尺寸（`getImageData(0,0,50,50)` 等），
故 bug 长期潜伏，仅在页面/攻击者可控的大尺寸 JS 调用时爆发。

## 解决方案

三处统一改为 usize 域 `saturating_mul`：

```rust
// R3354 修复
let size = (width as usize).saturating_mul(height as usize).saturating_mul(4);
```

- 64-bit usize 下，`65536*65536*4 = 2^34` 是合法 usize 值（不再回绕），分配走真实尺寸；
- 极端超大尺寸（接近 `usize::MAX`）saturating 钳到 `usize::MAX`，由 `Vec` 分配层处理
  （OOM abort，而非静默回绕致内存损坏）。
- W3C 语义不变：`getImageData` 仍返回请求 `width×height` 的 `ImageData`，画布外像素透明黑。

## 如何避免

1. **像素缓冲区大小计算恒用 usize 域 + saturating_mul**：凡从外部 `u32`/`i32` 维度推导字节数
   （`w*h*4` / `w*h*channels`），一律 `(w as usize).saturating_mul(h as usize).saturating_mul(channels)`。
   不要先在窄类型域乘再 `as usize`——这是 stack 中反复出现的整型溢出反模式
   （参见 R3292 `draw_image_sized` unsigned 下溢、R3347 wasm-sandbox `read_memory` offset 溢出、R3346 net cookie day=0）。
2. **切片长度校验在 debug/release 都生效**：不要把「算术溢出（debug panic / release 回绕）」与
   「切片越界（两模式都 panic）」混为一谈。即便算术在 release 静默回绕，后续切片访问仍会 panic——
   故 release 构建并不能掩盖此类 bug，只是把它从「算术 panic」变成「切片 panic」。
3. **审计时关注 `* 4) as usize` / `* channels) as usize` 模式**：grep `as usize) \* ` 与
   `\* 4) as usize` 可快速定位窄类型域乘法。

## 相关测试

`crates/canvas/src/context/tests/context_impl_coverage.rs`：
- `test_context_get_image_data_size_calc_uses_usize_r3354`
- `test_context_create_image_data_size_calc_uses_usize_r3354`
- `test_context_create_image_data_overflow_saturates_r3354`（usize saturating vs u32 wrapping 对照）

注：panic 复现需 16GB 量级分配，不适合 CI；以 usize 计算正确性 + saturating 不变量间接锁修复。
