# WPT reftest @font-face loader 缓存：键必须等于构造函数输入（+Arc 共享解析结果）

**日期**：2026-08-07
**相关模块**：`tests/wpt-runner/src/reftest.rs`（`FRESH_LOADER_CACHE`）、`crates/render-foundation/src/font/loader.rs`（`FontLoader::duplicate`、`fonts: HashMap<u32, Arc<fontdue::Font>>`）
**触发**：接续 `wpt-reftest-font-parse-cost.md`（BASE_FONT_LOADER 单例）——声明 `@font-face` 的 case 仍每次 `create_font_loader` 全量重解析 19MB CJK（~480ms/case × 2 render，Ahem 系慢 case 根因）。

## 问题描述

上一轮缓存了「无 @font-face」case（BASE_FONT_LOADER 单例），但**声明 @font-face 的 case 每 render 仍重解析默认字体集**（~0.5s × test/ref 2 render）。计划：按 CSS 内容哈希缓存 `Arc<FontLoader>`。

## 根因与两次修正（比计划更深的坑）

**初版按 `font_scan_css`（整份合并 CSS + 内联样式）哈希缓存——两个问题**：

1. **键过宽（缓存不生效）**：整份 CSS 每页几乎唯一 → Ahem 系 case 的 test 页各建一个键，**ref 页（reference/ 子目录，`ref_base_dir ≠ base_dir`）也各建一个键**。实测 fullwidth 9 案仍付 4-5 次 ~485ms。
2. **键缺 base_dir（正确性漏洞）**：相同 CSS 文本 + 不同目录 → 相对 src 解析到不同文件，却共享首个创建者的 loader → 静默错误渲染。

**修正 1：键 = 构造函数的真实输入**。`load_font_faces_into` 只消费 `extract_font_faces` 的 faces 列表 + `base_dir`（经 `resolve_font_src`）。键改为**按 base_dir 解析后的 src 路径列表**：绝对 src（WPT 通用 `/fonts/*`）与 base_dir 无关 → test/ref（不同目录）共享键；相对 src 解析为各自路径 → 正确区分。全字符串键，无哈希碰撞。

**修正 2：纯 Ahem 捷径**。`load_font_faces_into` 跳过 Ahem family（harness 合成方块，base 已含 Ahem.ttf）→ 仅声明 Ahem 的 fresh loader 与 BASE 内容等价 → 直接复用 BASE，零创建。

**修正 3（再压每键成本）：`FontLoader::duplicate` + Arc 共享解析结果**。每 distinct 键仍付一次 `create_font_loader`（~485ms，CJK 解析占 ~430ms）。新增 `duplicate()` 从 BASE 深拷贝——但 `fontdue::Font` 持有 65K 字形预解析轮廓（clone ~150ms，仅 3× 优于重解析）。**把 `fonts` 内部改为 `HashMap<u32, Arc<fontdue::Font>>`（`font_data` 同改 `Arc<Vec<u8>>`）**：duplicate 变引用计数 + 映射拷贝（~10ms），每键成本降为「仅自定义字体本身的读盘+解析」（~40ms）。

## 效果（实测，release）

| 指标 | 优化前 | 优化后 |
|------|--------|--------|
| fullwidth 9 案总时长（串行） | ~19s（每案 2×485ms 重解析） | **0.78s**（1×BASE init + 1×woff 键） |
| css-fonts 287 案 | 每 @font-face 案 2×~0.5s | 仅 4 案 >0.1s（BASE init + 3 个 distinct 键），余均 <0.1s |
| 每 distinct 键成本 | ~485ms（含 CJK 重解析） | ~40ms（仅自定义字体加载） |
| **全量 upstream（~16600 案，8 jobs）** | **465.7s**（每 @font-face 案 2×0.5s 重解析） | **24.4s（~18×）** |

正确性零回归：全量 upstream 失败集合与 HEAD（无缓存）逐案一致（3228 案，`comm -3` 空）；css-fonts 287 案 pass/fail 与 diff 率逐项一致；inline 686/686 通过；`cargo test --workspace` 全过。

## 附带发现：预存在的 harness 竞态（与本优化无关）

对比全量跑时发现 `line-clamp-019.html` 结果在 0.00%（pass）与 10.46%（fail）间**硬币式翻转**——同一 HEAD 二进制 3 连跑为 pass/fail/pass。根因指向 `runner_text_metrics::MEASURE_CTX`（thread_local 存 `*const FontLoader` 原始指针）与引擎内部线程化绘制的组合：SAFETY 注释「runner 单线程渲染」在 `--jobs ≥ 2`（rayon 工作线程渲染）时被违反，布局/paint 落在无 ctx 的线程上会回退 `estimate_char_width`（0.55×fs）→ 行宽不同 → line-clamp 截断差异。`--jobs 1`（主线程渲染）确定性失败暴露了它；修复方向：测量上下文改为可跨线程（如 rayon 池内 install）或引擎层传递。**未在本优化内修复（独立议题）。**

## 经验

1. **缓存键必须等于构造函数的真实输入**（此处 = faces 列表 + base_dir 解析后的路径），而非「调用处的某个更大字符串」。键过宽 = 缓存不生效；键缺输入的一部分 = 静默错误。
2. **路径类键要先解析再入键**：base_dir 直接入键会过度区分（test/ref 不同目录但绝对 src 相同）；解析后的路径天然反映真实依赖。
3. **复用已解析大对象用 Arc 而非深拷贝**：`duplicate()` 深拷贝 65K 字形预解析轮廓 ~150ms，Arc 共享后 ~10ms。`&self` 只读 + 无内部可变性（fontdue 全库无 Cell/RefCell）→ 跨 rayon 线程 Arc 共享安全。
4. **并行下「阻塞等锁」也会显示为 case 耗时**：`[case-stages]` 里多个 ~0.5s 的 test render 可能是 OnceLock/Mutex 等待，不是各自创建——串行重跑即可区分。

## 后续

- 每键 ~40ms 已是「自定义字体真实工作」下限；若仍需压，唯一方向是 `@font-face` src 的并发预加载（当前 miss 时在 Mutex 内串行创建）。
- 度量：`ZW_RENDER_STAGES=1`（per-render 阶段）+ `ZW_CASE_STAGES=1`（per-case test/ref）+ `REFTEST_TIME_LOG=1`（per-case 总时长）。
