# R169 Evidence — master.md 恢复 + d3c gate 尝试（iframe 双工厂发现）

**日期**: 2026-08-22
**Commit**: `ac2a56e97`（master.md 恢复）+ `3772390e2`（learning）+ `f0939b188`（attr 修复 + d3c 回退记录）
**切片**: R169——① 控制面事故恢复；② L2-d3c 尝试（回退 + 根因定位 + 前置项确立）

## 一、master.md 恢复（R169 勘误）

R168 的 docs 提交（`470b0e0e6`）把 master.md 从 **441KB 截断到 3.4KB**——python
块级 replace 的边界 `src.find('**上轮**: R167')` 与预期不符，把文件主体整个包进
old_head。修复：`git show 6431f125f:...` 恢复完整内容 + 行级编辑重做 R168 记录。
**教训沉淀** learning（`docs/learnings/patterns/2026-08/2026-08-22-python-block-
replace-truncates-large-files.md`）：大文件禁块级 find/replace；行级 split +
startswith；提交前 `git diff --stat` 看量级（+30 预期 vs -845 实际一眼可见）。

## 二、d3c 尝试（gate 启用 → 大回归 → 根因 → 回退）

### 过程

1. 启用 R165 预留的 compound 解析（`else if (false)` → `else`）。
2. 定向跑：ParentNode 33→**905F**、Element-matches 3→291F、bubbles 0→6F——
   R165 同款回归形态。
3. 沙箱探针（createHTMLDocument）：**全形态全命中**（nullTag/cls/id/tagCls/
   attr/attrV 全 1）。
4. runner 内探针（iframe srcdoc）：**全形态 0 命中** + `bodyHtml:0`；同页
   createHTMLDocument 对照全命中。

### 根因（d3 前置项确立）

**iframe doc 双工厂**：WPT Document 上下文 = iframe contentDocument，其树/查询
走 part05 iframe 工厂（host 快照持有 iframe 内容）；`_makeDetachedDocument` 的
`bodyHtml` 对 iframe 场景**恒空**。compound gate 拦截后 `_tree` 空 + detHtml
空 → 零命中（gate 前这些查询经 iframe 工厂自己的路径命中 host 快照）。
**d3c 须先统一 iframe doc 与 detached doc 的树源**。

### 保留物

- **`[attr]` 存在性匹配修复**（`_queryTreeByCompound`）：R165 首版 op null 落
  else 恒 false——任何未来 gate 开启都会使 `[attr]` 选择器全灭。gate 关闭时
  该分支不可达（零行为变化），是 d3c 重启的必要前置修复。
- 回退注释完整记录尝试与根因（part03 queryBody 内）。

## 三、验证

| 门 | 结果 |
|----|------|
| 全量 dom WPT polyfill | **9522P/343F/18T**（= R168 逐计数一致，零行为变化确认） |
| `make test` | 66 套件 **18071P/0F** |
| fmt / clippy | 干净 |

## 四、下一步（R170）

- **d3 前置项**：iframe doc 与 detached doc 树源统一（iframe 工厂的 bodyHtml
  注入或查询路由统一）——d3c/d3d 的共同依赖。
- 或转 M6 域（native dom_bindings 补齐）等其它 ROI 面。
