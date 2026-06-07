# ZeroWeb 深度审查报告 — DOM / Engine 核心模块

> **摘要**
>
> **审查范围**：`crates/dom/`（Document、Parser）、`crates/engine/`（DomBridge、Animation、DirtyTracker）
>
> **关键发现**：共发现 7 个问题（高 2 / 中 3 / 低 2）
>
> **最高优先级**：`replace_child` 缺少循环检测，可导致 DOM 树循环和无限递归崩溃
>
> **验证状态**：已验证（2026-06-07）— 7 verified, 0 dismissed

## 审查上下文

| 字段 | 内容 |
|------|------|
| **审查对象** | dom/document.rs、dom/parser.rs、engine/dom_bridge.rs、engine/animation.rs、engine/dirty.rs |
| **审查维度** | 实现缺陷、数据完整性、性能 |
| **代码版本** | main 分支，commit f5eb85b |

---

## 问题清单

### 高优先级（Critical）

#### DOM-01 [实现缺陷] replace_child 缺少循环检测和文档根检查

- **位置**：`crates/dom/src/document.rs:367-415`
- **置信度**：0.95
- **状态**：verified
- **描述**：与 `append_child`（208-220 行）和 `insert_before`（313-322 行）不同，`replace_child` 未检查 `new_child == self.root`（CannotInsertDocumentRoot）也未调用 `self.is_ancestor(new_child, parent)` 进行循环检测。这允许节点的祖先替换自身的子节点，在 DOM 树中创建循环。
- **触发条件**：调用 `parent.replaceChild(grandparent, child)` 其中 grandparent 是 parent 的祖先。
- **代码证据**：
  ```rust
  pub fn replace_child(&mut self, parent: NodeId, new_child: NodeId, old_child: NodeId) -> Result<NodeId, DomError> {
      if !self.contains(parent) || !self.contains(new_child) || !self.contains(old_child) {
          return Err(DomError::NodeNotFound(parent));
      }
      // ❌ 无循环检测、无根节点检查
  ```
- **影响**：DOM 树循环导致无限递归和栈溢出崩溃
- **建议修复**：
  ```rust
  if new_child == self.root {
      return Err(DomError::CannotInsertDocumentRoot);
  }
  if self.is_ancestor(new_child, parent) {
      return Err(DomError::WouldCreateCycle);
  }
  ```

---

#### DOM-02 [数据完整性] set_text_content 未清理被移除子节点的 id_map

- **位置**：`crates/dom/src/document.rs:655-678`
- **置信度**：0.90
- **状态**：verified
- **描述**：当 `set_text_content` 清除所有子节点时（如 `element.textContent = "foo"`），子节点从父节点脱离但未调用 `remove_id_map_recursive`。被移除的带有 `id` 属性的子元素（及其后代）将永久留在 `id_map` 中。后续的 `get_element_by_id` 将返回已脱离文档的节点。
- **触发条件**：对一个包含 `<div id="foo">...</div>` 的元素调用 `textContent = "new"`，随后 `document.getElementById("foo")` 返回已脱离的 div。
- **代码证据**：
  ```rust
  let children: Vec<NodeId> = self.nodes.get(id).map(|n| n.children.clone()).unwrap_or_default();
  for child in &children {
      if let Some(child_data) = self.nodes.get_mut(*child) {
          child_data.parent = None; // 仅清除 parent，未清理 id_map
      }
  }
  ```
- **影响**：`getElementById` 返回脱离文档的节点，导致 UI 更新失败或页面状态不一致
- **建议修复**：
  ```rust
  for child in &children {
      self.remove_id_map_recursive(*child);
      // ... 然后清除 parent
  }
  ```

---

### 中优先级（Major）

#### DOM-03 [性能] DomBridge::register 对重复检测执行 O(n) 线性扫描

- **位置**：`crates/engine/src/dom_bridge.rs:219-230`
- **置信度**：0.85
- **状态**：verified
- **描述**：`register` 方法遍历 `handle_map` 所有条目检查 `node_id` 是否已存在，这是 O(n) 操作。复杂页面上大量 DOM 节点通过 bridge 注册时成为性能瓶颈。
- **建议修复**：添加反向映射 `HashMap<u64, u64>`（node_id → handle）实现 O(1) 双向查找。

---

#### DOM-04 [性能] 递归遍历方法在每个节点访问时克隆 children 向量

- **位置**：`crates/dom/src/document.rs:1405,1428,1446,1479,1500,1513,1540,1577`
- **置信度**：0.95
- **状态**：verified
- **描述**：`collect_by_tag_name`、`collect_by_class_name`、`find_first_matching` 等方法在每次递归调用时克隆整个子列表 `children.clone()`。N 节点文档的总克隆次数为 O(N)，每次分配 O(子节点数)。将原本 O(N) 的遍历变为 O(N × 平均子节点数)。
- **建议修复**：先收集子 ID 再遍历：
  ```rust
  let children: Vec<NodeId> = node_data.children.iter().copied().collect();
  for child in children { ... }
  ```

---

#### DOM-05 [性能] DirtyTracker::merge_overlapping 具有 O(n³) 最坏情况复杂度

- **位置**：`crates/engine/src/dirty.rs:66-111`
- **置信度**：0.85
- **状态**：verified
- **描述**：外层 `while merged` 循环每次迭代最多合并一对，O(n) 迭代 × O(n²) 嵌套循环 × O(n) `remove` 操作 = O(n³)。大量脏区域时可能导致帧卡顿。
- **建议修复**：使用单遍合并方法或扫描线/排序算法替代。

---

### 低优先级（Minor）

#### DOM-06 [实现缺陷] cubic_bezier 二分搜索仅 8 次迭代（精度低）

- **位置**：`crates/engine/src/animation.rs:135`
- **置信度**：0.70
- **状态**：verified
- **描述**：贝塞尔曲线求解器仅使用 8 次二分搜索迭代，精度约 1/256。标准浏览器使用 Newton-Raphson，4-6 次迭代可达 10⁻⁶ 精度。动画可能看起来略显生硬。
- **建议修复**：增加至 20+ 次迭代或实现 Newton-Raphson 求解器。

---

#### DOM-07 [实现缺陷] DomBuilder::into_document 将 ShadowRoot 转为 DocumentFragment 丢失元数据

- **位置**：`crates/dom/src/parser.rs:84-96`
- **置信度**：0.60
- **状态**：verified
- **描述**：`ShadowRoot` 节点被转为普通 `DocumentFragment`，丢失 mode（open/closed）和 host 引用。当前 HTML 解析器不生成 ShadowRoot（由 JS API 创建），影响较低。
- **建议修复**：使用 `create_shadow_root` 替代 `create_document_fragment`。

---

## 统计总览

| 维度 | 高 | 中 | 低 | 合计 |
|------|----|----|----|------|
| 实现缺陷 | 1 | 0 | 2 | 3 |
| 数据完整性 | 1 | 0 | 0 | 1 |
| 性能 | 0 | 3 | 0 | 3 |
| **合计** | **2** | **3** | **2** | **7** |

## 修复建议优先级

| 优先级 | 问题 | 建议动作 | 预估改动量 |
|--------|------|---------|-----------|
| P0（立即） | DOM-01, DOM-02 | 添加循环检测、清理 id_map | 各约 5-10 行 |
| P1（本迭代） | DOM-03, DOM-04, DOM-05 | 反向映射、避免克隆、优化合并算法 | 各约 20-50 行 |
| P2（后续跟进） | DOM-06, DOM-07 | 增加迭代次数、修复 ShadowRoot 转换 | 各约 10-20 行 |
