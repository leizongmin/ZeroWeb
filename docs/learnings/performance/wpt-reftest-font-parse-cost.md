# WPT reftest 单案成本 85% 是 fontdue 字体重解析（缓存默认字体后 ~100× 加速）

**日期**：2026-08-07
**相关模块**：`tests/wpt-runner/src/reftest.rs`（`render_with_layout_inner`、`BASE_FONT_LOADER`）、`tests/wpt-runner/src/reftest/reftest_fonts.rs`（`create_font_loader`）、`crates/render-foundation/src/font/loader.rs`（`FontLoader::load_font` → `fontdue::Font::from_bytes`）
**触发**：WPT reftest 耗时调研（杠杆4：「降低单案串行成本」）

## 问题描述

reftest 单案串行成本 ~850-900ms，怀疑是 CPU 软光栅瓶颈。加 per-case 计时（`REFTEST_TIME_LOG`）
与临时 font-load 计时定位后证伪：**字体加载占 ~85%，光栅/布局只占 ~15%**。

## 根因分析

每次 `render_to_framebuffer`（每 case 调 2 次：test 页 + ref 页）都走 `render_with_layout_inner`，
其中 `create_font_loader()` 无条件重建 FontLoader：

```rust
let mut font_loader = create_font_loader();   // 每次 render 重新读盘 + 重新解析
```

`create_font_loader` 读盘 + `fontdue::Font::from_bytes`（全表解析）这些**跨 case 完全相同**的
默认字体：Liberation Serif、DejaVuSans(+Mono)、LiberationSans、**NotoSansCJK（~16MB）**、Ahem。
实测（warm，OS page cache 命中）：每 render ~0.40s，每 case 2 render ≈ 0.8s，而单案总成本 ~0.9s。
即 **~16MB NotoSansCJK 的 fontdue 解析被重复执行 1372 次（686 case × 2）**。

> 注：OS page cache 让「读盘」几乎免费（~0.02s），贵的是 **fontdue 解析**——缓存原始字节省不掉它。

## 解决方案

`build_font_resolver` / `build_line_metric_map` 均取 `&self`（只读），故可跨 case 共享一个
不可变 FontLoader：

```rust
static BASE_FONT_LOADER: std::sync::OnceLock<FontLoader> = OnceLock::new();

let fresh_loader: Option<FontLoader> = if has_font_face {
    // 少数声明 @font-face 的 case（字体测试）走 fresh owned loader（含 re-parse + 自定义字体）
    let mut fl = create_font_loader();
    load_font_faces_into(&mut fl, base_dir, &font_scan_css);
    Some(fl)
} else {
    None
};
let font_loader: &FontLoader =
    fresh_loader.as_ref().unwrap_or_else(|| BASE_FONT_LOADER.get_or_init(create_font_loader));
```

- 无 `@font-face` 的多数 case：复用进程级单例（全进程只解析一次），零 re-parse。
- `@font-face` case：走 fresh owned loader，保持原行为（自定义字体需 `&mut` 加载）。
- `OnceLock` + `&self` 方法：rayon 多线程并发只读安全（无内部可变性）；全进程只 build 一次
  （比 per-thread 缓存更省内存）。

## 效果（实测，release，16 核）

| 指标 | 优化前 | 优化后 | 加速 |
|------|--------|--------|------|
| 单案串行成本 | ~0.9s | ~0.004s（首案 ~0.45s 一次性建 base） | ~100× |
| inline reftest（686 案）总时长 | 104.5s | 1.0s | ~100× |
| upstream css-position（97 案） | 12.0s | 2.1s | ~6×（startup 主导） |
| 全量 upstream（~9967 案）投影 | ~25min | ~3-5min | ~5-8× |

正确性零回归：inline 686/686（100%）、wpt-runner 131 单测全过、upstream 各子集 pass/fail 数不变
（缓存路径 `==` fresh 路径渲染输出，因 `create_font_loader` 确定 + `build_font_resolver` 纯函数）。

## 如何复用 / 识别同类问题

**模式**：harness 里每个测试 unit 重复执行某「跨 unit 完全相同、且昂贵」的初始化（字体/模型/字典
解析、大文件读入、复杂查找表构建），但只用了结果的一小部分（如这里的 `build_font_resolver` 取 `&self`）。

**识别**：per-case 计时（`REFTEST_TIME_LOG=1`）+ 针对 suspected setup 的临时计时，定位成本占比。
若 setup >> 实际工作，且结果可 `&self` 共享或 cheaply clone → 用 `OnceLock`/`thread_local` 缓存，
按「需 per-unit 变化的部分（如 @font-face）走 fresh、其余复用」分支。

**反例（不可这样缓存）**：若共享对象有 `&mut` 热路径方法或内部可变性，须先确认只读安全；本例因
`build_font_resolver(&self)` 而成立。
