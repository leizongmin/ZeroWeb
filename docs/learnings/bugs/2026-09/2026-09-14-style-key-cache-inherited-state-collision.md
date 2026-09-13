---
date: 2026-09-14
modules: zero-style-system
---

# S11 style-key 缓存的不可键父链继承态碰撞（dir=rtl 泄漏实证）

## 问题

`direction: rtl` 通过 HTML `dir` 属性（presentational hint）设置在容器上时，**后代元素不继承**，甚至继承到**错误来源**的值。最小复现（`StyleSystem::compute_styles`）：

```html
<ul dir="rtl"><li>A</li></ul>
<ul><li>B</li></ul>
```

两个 `li` 的 computed `direction` 都是 `Rtl`——`li#B` 的父是普通 `ul`（Ltr），却拿到了 `li#A` 的值。交换两个 `ul` 的顺序，两个 `li` 都变 `Ltr`。`<div dir=rtl><span>` 同理（span 拿不到 Rtl）。

## 根因

S11（commit `4928771e5`，perf(style)）给「属性仅含 class/id」的元素建了 StyleKey 缓存：`StyleKey { tag, classes, id, parent: Option<Rc<StyleKey>> }`，`parent` 是**父元素的键**。父元素带 class/id 外属性（style/dir/lang/title…）时父不可键 → `parent=None`。

缺陷：**所有「不可键父」的子元素键都是 `(tag, classes, id, None)`**——文档里先算的那个（父 A）与后算的同键元素（父 B）键完全相同，后者直接 `style_cache` 命中，**clone 前者在其父继承态下的整份 computed style**。继承属性（direction/color/white-space/字体族…全部）随键碰撞一起错传。

S11 的文档注释明确写着「父无键（不安全/属性超集）时子元素禁用缓存（继承链不完整）」——**但实现从未落地这条门**。文档意图与代码行为从合入起就不一致。

## 修复（R4322）

子键加 `unkeyed_parent_chain: u64` 组件：父可键时为 0（继承态已由 `parent` 键链完整刻画）；父不可键时为**祖先 (tag, attrs) 属性链 FNV-1a 指纹**（递归传入）。cache_safe 样式表已排除属性选择器/伪类/var()，子元素计算态 = f(自身 attrs, 祖先 attrs 链)，指纹完整刻画继承态来源。

## 修复路上的弯路（为什么不是另外两种形态）

1. **父元素 NodeId 进键**：碰撞消除 ✓，但 medium 基准的 4000 个 `.item` 分布在 4000 个同构 row 下——身份组件把「同构不同节点」的复用全打散，`page/medium/style_ms` 21.6→182ms（8.4×），bench-gate FAIL。S11 的价值恰恰是跨子树复用，身份键与之互斥。
2. **父不可键则子禁缓存**（S11 注释的字面语义）：正确性 ✓，但 medium 的 row 恰是 style 属性容器（不可键）→ 整片子树失去缓存，style_ms 292ms，仍 FAIL。

属性链指纹是第三种形态：同构父链同指纹（复用保住，medium 回落预算内），异构父链必异指纹（碰撞消除）。

## 如何避免

- 继承 perf 机制时，**文档注释里的安全性声明必须找到对应的代码断言**；注释声明「X 时禁用」而代码没有 X 判断，几乎总是漏实现。本次 S11 的注释写了禁用条件，代码却只有一个 stylesheet 级 `cache_safe` 门。
- 缓存键的「父组件」为 `Option` 时，`None` 是一个**具体值**而非「任意」——所有落到 None 的元素互相碰撞。凡键组件可缺席，必须回答「缺席态之间是否真的等价」。
- 该 bug 的可视信号：同 tag/class 的兄弟元素在不同属性容器下「渲染成第一个的位置/方向/颜色」。`dir=rtl` 页面整体左对齐是典型症状。
