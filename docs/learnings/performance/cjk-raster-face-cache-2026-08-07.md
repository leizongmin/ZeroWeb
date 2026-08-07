# CJK 字形栅格化重尾优化：FreeType face 缓存 + 采样哈希

日期：2026-08-07 ｜ 模块：render-foundation（freetype_raster）

## 问题

含 CJK 文本的 WPT case 栅格化重尾（fullwidth 案 570 字形，曾几十秒级挂起）。
此前分析归因「fontdue 本征成本 ~3ms/字」，缓存类杠杆被判无效。

## 根因（探针实测，比归因更精确）

| 项 | 测量 |
|---|---|
| fontdue CJK 栅格化 | **0.019 ms/字**（快——原归因不准） |
| freetype_raster 每次 `new_memory_face2` | 解析 19MB CJK TTC **~6.6ms/次**（每字重新解析整字体） |
| `bytes_hash` 全量遍历 19MB | debug 下 **~50ms/次**（每次栅格化都遍历） |
| FreeType 单字操作（set_char_size/load_glyph/render） | 0.06 ms/字（快） |

**真正的重尾 = 每次栅格化都重新解析字体 + 全量哈希大字体**，不是光栅化本身。

## 修复

1. **face 缓存**（`freetype_raster` 模块）：thread_local `RefCell<HashMap<u64, Face<Vec<u8>>>>`
   - `Face<Rc<Vec<u8>>>` 自含字体字节，一次解析后复用（face 方法均 `&self`，
     RefCell borrow 贯穿免 clone——`Face<Vec<u8>>::clone` 会复制 19MB）
   - 容量上限 8，超限清空重建
2. **采样哈希**：`bytes_hash` 从全量遍历改为**前 4KB + 总长**（O(1)，
   不同字体头部表几乎必然不同，冲突可忽略）

## 效果

| 指标 | 优化前 | 优化后 |
|---|---|---|
| freetype_raster CJK（debug，缓存命中） | 51 ms/字 | **0.024 ms/字**（~2100×） |
| 含首次解析 | — | 0.24 ms/字 |
| CSS2/text 全目录（408 cases） | 曾有 >10s 重尾 | **0 个 >10s**（慢 case 1-1.4s） |

## 经验

1. **性能归因要实测到操作级**：分阶段计时（set_char_size/get_char_index/load_glyph/
   render_glyph 各 0.001-0.04ms）快速定位真实热点——原「fontdue 3ms/字」归因不成立
2. **FreeType 每次 `new_memory_face2` 会完整解析字体**：大字体（19MB TTC）下
   复用 face 是必须的（Face 持有字节，天然可缓存）
3. **哈希缓存键要避免全量遍历大输入**：大字体哈希用采样窗口；缓存查找是高频路径
4. **thread_local 的 `with` 不允许借用逃逸**：RefCell borrow 须在 `with` 闭包内
   完成全部使用（嵌套 with_lib）

## 后续

- 若 1-1.4s 的 CJK 大案仍需压：预栅格化 CJK 常用字 atlas（用户场景字形重复度高）
- 度量：`REFTEST_TIME_LOG=1` per-case 计时（已有 CaseTimer）
