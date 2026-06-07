# ZeroWeb 深度审查报告 — Layout / Render / Storage 模块

> **摘要**
>
> **审查范围**：`crates/layout-engine/`（Tree、Converter）、`crates/render-foundation/`（GPU Renderer）、`crates/storage/`（LocalStorage、IndexedDB、Cache API）
>
> **关键发现**：共发现 8 个问题（高 2 / 中 2 / 低 4）
>
> **最高优先级**：`computed_style_to_taffy` 将 `align_content` 映射到 `justify_content`，导致 CSS `align-content` 属性完全失效
>
> **验证状态**：已验证（2026-06-07）— 7 verified, 1 dismissed

## 审查上下文

| 字段 | 内容 |
|------|------|
| **审查对象** | layout-engine/tree.rs、layout-engine/converter/mod.rs、render-foundation/gpu/renderer.rs、storage/lib.rs、storage/local_storage.rs、storage/cache_api.rs |
| **审查维度** | 实现缺陷、性能、数据完整性 |
| **代码版本** | main 分支，commit f5eb85b |

---

## 问题清单

### 高优先级（Critical）

#### LAY-01 [实现缺陷] computed_style_to_taffy 将 align_content 映射到错误的属性

- **位置**：`crates/layout-engine/src/converter/mod.rs:74`
- **置信度**：0.95
- **状态**：verified
- **描述**：`align_content` 字段从 `style.justify_content` 而非 `style.align_content` 读取。这导致 CSS `align-content` 属性的值被 `justify-content` 的值覆盖，`align-content` 值被完全丢弃。任何同时设置不同 `justify-content` 和 `align-content` 的 CSS 规则都会渲染错误。
- **触发条件**：CSS 规则 `display: flex; justify-content: center; align-content: stretch;` 实际渲染时 `align-content` 也为 `center`。
- **代码证据**：
  ```rust
  align_content: convert_alignment_to_align_content(&style.justify_content),
  //                                                        ^^^^^^^^^^^^^^^^ 应为 align_content
  ```
- **影响**：Flex/Grid 布局中 `align-content` 属性完全失效
- **建议修复**：
  ```rust
  align_content: convert_alignment_to_align_content(&style.align_content),
  ```

---

#### LAY-02 [实现缺陷] GPU Atlas 清除破坏已生成顶点的 UV 坐标

- **位置**：`crates/render-foundation/src/gpu/renderer.rs:296-328`
- **置信度**：0.70
- **状态**：verified
- **描述**：当 `GlyphAtlas` 满时，CPU 端 atlas 被 `clear()` 但 GPU 纹理未清除。更重要的是，已为之前 glyph 生成的顶点数据仍引用旧 atlas UV 坐标，但清除后这些放置记录已不存在。当前帧的后续渲染使用随机纹理数据。
- **触发条件**：渲染包含大量不同 glyph 的文本时（如复杂 Unicode 文本），atlas 容量耗尽触发重建。
- **代码证据**：
  ```rust
  None => {
      self.atlas.clear(); // 清除放置记录
      // 重试放置当前 glyph
      self.atlas.place(key, width, height, x_offset, y_offset, advance)
          .map(|result| { ... })
  }
  ```
- **影响**：文本渲染出现随机乱码（视觉损坏）
- **建议修复**：atlas 满时返回指示，让调用者重启整个 glyph 收集过程；或保留旧 atlas 纹理，仅在新帧开始时清除。

---

### 中优先级（Major）

#### LAY-03 [性能] GPU 渲染器每帧创建一次性 uniform 和 vertex 缓冲区

- **位置**：`crates/render-foundation/src/gpu/renderer.rs:587-611`
- **置信度**：0.80
- **状态**：verified
- **描述**：每次 `render_vertices()` 调用都分配新的 GPU uniform 缓冲区和 vertex 缓冲区。60 FPS 下每秒创建 120+ 个短命 GPU 缓冲区，对 GPU 驱动造成分配压力，可能导致资源碎片化和帧卡顿。
- **建议修复**：在 `GpuRenderer` 中保持持久缓冲区（或环形缓冲池），每帧用 `queue.write_buffer()` 更新。

---

#### LAY-04 [性能] WebStorage used_size() 为 O(n)，每次 set() 调用时重算

- **位置**：`crates/storage/src/local_storage.rs:58-60, 104`
- **置信度**：0.90
- **状态**：verified
- **描述**：`used_size()` 每次调用都遍历所有条目计算总大小，而 `set()` 每次都调用 `used_size()`。大型存储下每次写入都产生明显停顿。
- **建议修复**：维护增量 `used_bytes` 字段，在 `set()`/`remove()` 时增减更新。

---

### 低优先级（Minor）

#### LAY-05 [实现缺陷] tokenize_track_list 括号深度计数器可能下溢

- **位置**：`crates/layout-engine/src/converter/mod.rs:424`
- **置信度**：0.55
- **状态**：verified
- **描述**：`depth -= 1` 在右括号多于左括号时可能下溢（wrap around），导致 `depth` 在顶层时非零。仅影响格式错误的 CSS 输入。
- **建议修复**：使用 `depth = depth.saturating_sub(1)`。

---

#### LAY-06 [实现缺陷] parse_grid_template_areas 不跳过 "."（空单元格标记）

- **位置**：`crates/layout-engine/src/converter/mod.rs:782-796`
- **置信度**：0.60
- **状态**：verified
- **描述**：CSS `grid-template-areas` 中的 `"."` 表示空单元格，当前代码将其作为普通区域名存储到 `GridAreaMap`，污染映射表。
- **建议修复**：处理前跳过 `token == "."`：
  ```rust
  if token == "." { continue; }
  ```

---

#### LAY-07 [实现缺陷] Cache::put 覆盖时不更新 request 对象

- **位置**：`crates/storage/src/cache_api.rs:123-134`
- **置信度**：0.50
- **状态**：dismissed
- **描述**：覆盖匹配条目时只更新 `response`，保留旧 `request`。
- **dismiss 原因**：CacheRequest 仅存储 url 和 method，匹配基于这两个字段。新旧 request 在匹配字段上完全相同，保留旧 request 无任何可观测的行为差异。当前 `CacheRequest` 仅有 `url` 和 `method`，影响较低，但未来扩展后可能导致状态不一致。
- **建议修复**：覆盖时同时更新 `request`。

---

#### LAY-08 [性能] build_subtree 对 display:none 节点不必要地克隆 ComputedStyle

- **位置**：`crates/layout-engine/src/tree.rs:103`
- **置信度**：0.70
- **状态**：verified
- **描述**：`ComputedStyle` 在函数开头被克隆，但 `display: none` 元素在紧接着检查后立即返回。对隐藏元素占多数的 DOM，造成不必要的堆分配。
- **建议修复**：先检查 `display: none` 再克隆：
  ```rust
  if styles.get(&dom_id).is_some_and(|s| s.display == DisplayValue::None) {
      return LayoutBox::hidden(dom_id);
  }
  let computed = styles.get(&dom_id).cloned().unwrap_or_default();
  ```

---

## 统计总览

| 维度 | 高 | 中 | 低 | 合计 |
|------|----|----|----|------|
| 实现缺陷 | 2 | 0 | 3 | 5 |
| 性能 | 0 | 2 | 1 | 3 |
| **合计** | **2** | **2** | **4** | **8** |

## 修复建议优先级

| 优先级 | 问题 | 建议动作 | 预估改动量 |
|--------|------|---------|-----------|
| P0（立即） | LAY-01 | 修正属性映射（1 行改动） | 1 行 |
| P0（立即） | LAY-02 | 重构 atlas 重建逻辑 | 约 50 行 |
| P1（本迭代） | LAY-03, LAY-04 | 缓冲区复用、增量 size 维护 | 各约 30-50 行 |
| P2（后续跟进） | LAY-05~08 | 防御性编码和小优化 | 各约 5-15 行 |
