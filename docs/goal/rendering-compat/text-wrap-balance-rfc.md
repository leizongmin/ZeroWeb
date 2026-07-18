# text-wrap: balance — 行内平衡换行实现 RFC

**日期**: 2026-07-18
**承接**: R1662 forward（struct-sweep 五 dir 穷尽，下一安全独立 lever = CSS-Text-4 未实现特性）
**目标案**: `css/css-text/white-space/text-wrap-balance-003.html`（+ 004/005/align/before-after/float 系列，共 28 案，当前 5/27 oracle-pass）
**规范**: CSS-Text-4 §3.2 [`text-wrap`](https://drafts.csswg.org/css-text-4/#text-wrap)（balance 值）+ Chrome Layout NG 平衡算法（UA-defined，须逆向匹配）

## 现状

`text-wrap` 属性**已解析已存储**（`TextWrapComputedValue::{Wrap,Nowrap,Balance,Pretty}`，[`apply_advanced.rs:955`](../../crates/style-system/src/property/apply_advanced.rs) + ComputedStyle.text_wrap），但**布局侧完全未消费**——`grep text_wrap crates/layout-engine` 仅命中 multicol balancing（无关）。故 `text-wrap:balance` 当前等同 `wrap`（普通贪婪换行），balance 测案 oracle FAIL。

## 算法（balance-003 实证验证，确定性匹配 chromium）

balance 的语义：在保持**与普通换行相同行数 N** 的前提下，找**最小换行宽 W**，使各行长度尽量均等。

```
1. N = break_at(container_width).line_count        // 普通换行的行数
2. if N < 2: 无需平衡（单行），wrap_width = container_width
3. 二分搜索 W ∈ (0, container_width]：              // 找最小 W 使 line_count(W) == N
     line_count(W) 单调不增（W 越大行数越少）
     求 min{ W : line_count(W) <= N }
4. wrap_width_base = W                              // 窄化换行基准宽
5. 最终 break 用 wrap_width_base；text-align 仍用 container_width（全宽居中/右对齐）
```

**balance-003 验证**（Ahem 15px @ 35ch 容器，"The quickest brown fox jumped over the lazy dog" 44 chars）：
- 普通换行 @ 35ch → N=2（"The quickest brown fox jumped over"(33) / "the lazy dog"(12)）
- 二分：W=24 → line1 "The quickest brown fox"(22)≤24，+jumped=29>24 break；line2 "jumped over the lazy dog"(24)≤24 ✓ → N=2 ✓
- W=23 → line2 24>23 → wrap line3 → N=3 ✗
- 故 min W for N=2 = 24ch → "The quickest brown fox" / "jumped over the lazy dog" **逐字匹配 ref**（ref 用 `<br>` 强制同断点）

结论：算法对 Ahem（固定字宽）**确定性精确匹配** chromium；二分只需 sub-char 精度收敛到字宽边界。

## 实现切片

### Slice A（最小可 land，纯文本路径）

1. **IFC 加字段**（[`inline/mod.rs`](../../crates/layout-engine/src/inline/mod.rs)，≡ text_autospace 谱系 line 68/514）：
   - `pub wrap_width_base: f32`（`new()` 初始化 = container_width）
   - `pub text_wrap_balance: bool`（默认 false）
   - `.with_text_wrap_balance(b: bool) -> Self`
2. **换行宽解耦**：[`effective_content_area`](../../crates/layout-engine/src/inline/mod.rs:549) line 572
   `available = (self.container_width - left_offset - right_reduction).max(0.0)`
   → 改 `(self.wrap_width_base - left_offset - right_reduction).max(0.0)`。
   text-align / 行定位仍用 `container_width`（全宽），仅换行决策用 `wrap_width_base`。
   **默认 wrap_width_base == container_width → 零行为变化**（normal-wrap 字节一致）。
3. **threading**：[`inline_finalization.rs:832`](../../crates/layout-engine/src/inline_finalization.rs) IFC 构造点加
   `.with_text_wrap_balance(matches!(style.text_wrap, TextWrapComputedValue::Balance) && !no_wrap)`。
   （其他 IFC 构造点 1124/1174/1312/1473/1493 暂不接 balance——multicol/fragmentation 路径，balance 子集先不覆盖。）
4. **二分 + 行计数**：`break_items_into_lines` 入口（line 1209 vertical-check 后）：
   ```
   if self.text_wrap_balance && !self.vertical && kill_switch_on() {
       let items_ref = &items;  // 不消费
       let n = self.count_lines(items_ref, self.container_width);
       if n >= 2 {
           let w = binary_search_min_width(items_ref, self.container_width, n);
           self.wrap_width_base = w;
       }
   }
   // ... 原 break 核循环（用 effective_content_area → wrap_width_base）...
   self.wrap_width_base = self.container_width;  // 复位（防外层复用 IFC）
   ```
5. **`count_lines` 难点**：行计数须跑换行核心逻辑。两选一：
   - **(a) 抽取核心循环**为 `fn break_core(&self, items:&[InlineItem]) -> Vec<LineBox>`（`&self` 只读，返回行；不 mutate self.lines），break_items_into_lines 改为 `self.lines = self.break_core(&items)` 的 thin wrapper。抽取 ~450 行（1231-1679 循环体）机械搬移，须 `&self` 化（current_y/last_was_collapsible_ws 等局部变量保持局部；读 self 字段不变）。**A/B 守 normal-wrap 字节一致**（抽取本身零语义变化）。count_lines = `self.break_core(items).len()`。
   - **(b) 简化贪婪计数器**：`fn greedy_line_count(&self, items, width) -> usize` 仅按 word/inline-block 宽累加（忽略 float exclusion / vertical-align），用于二分；最终 break 走原核心循环。**风险**：float/复杂 inline 案计数与真 break 不一致 → W 选错 → 回归。balance-float-* 边缘案可能受影响。

   **裁决**：Slice A 选 **(b) 简化计数器**（避免 450 行抽取的高风险），但**仅对「无 float_exclusions」IFC 激活**（`self.float_exclusions.is_empty()` 守卫）——无 float 时简化计数器 = 精确贪婪 = 与真 break 一致。有 float 案 balance 暂不激活（回落 normal wrap，不回归）。Slice B 再做 (a) 抽取覆盖 float。

### Slice B（float 覆盖，后续）

抽取核心循环（选 a），count_lines 走真 break_core，移除 `float_exclusions.is_empty()` 守卫，覆盖 text-wrap-balance-float-* 5 案。

## 验收

1. **load-bearing 单测**：构造 IFC + Ahem 文本 "The quickest brown fox jumped over the lazy dog" @ 35ch，with_text_wrap_balance(true)，断言 `break_items_into_lines` 产出 2 行，行内容断点 = "The quickest brown fox" / "jumped over the lazy dog"（匹配 balance-003 ref）。
2. **oracle A/B**：`make reftest-oracle DIR=css-text`（或 filter text-wrap-balance），baseline 5/27 ↔ treatment **target ≥ 15/27**（Slice A 覆盖无 float 纯文本案 003/004/align/before-after 等 ~12 案）。全 css-text 1826 案 **net ≥ 0**（balance gated + kill-switch，无 float 守卫 → 无 float 案不受影响）。
3. **回归门禁**：`make product-smoke`（welcome 字节一致——welcome 不用 text-wrap:balance）+ `make test`（全 workspace 0 failed）+ clippy `-D warnings` + fmt。
4. **kill-switch**：`ZW_TEXT_WRAP_BALANCE=0` 关闭（默认 on），A/B 证 net<0 可即时回退。

## 风险与边界

- **核心换行路径**：effective_content_area 改 wrap_width_base 影响所有换行决策；**默认 wrap_width_base==container_width 保证 normal-wrap 字节一致**（A/B 必守）。
- **UA-defined 算法**：CSS-Text-4 未规定精确平衡算法，须匹配 chromium。本 RFC 的「min-width-for-N-lines」与 Chrome Layout NG 一致（balance-003 实证），但 Chrome 可能对边界（半像素、forced break、line-clamp）有特殊处理；text-wrap-balance-005（line-clamp 交互）+ dynamic-* 案可能需额外适配（Slice B+）。
- **`text-wrap-style: balance`**（CSS-Text-4 新名，balance-005 用）：parser 须补 `text-wrap-style` longhand（Slice A 先只接 `text-wrap: balance` shorthand；style longhand Slice B）。
- **性能**：二分 ~10 次试验 × 每次跑 count_lines = ~10× 换行开销，仅 balance 元素触发（罕见），可接受。
- **不覆盖**（Slice A）：float 案（守卫回落）、line-clamp 交互、`text-wrap-style` longhand、vertical writing-mode、multicol/fragmentation 路径 IFC 构造点。

## 不做的事

- 不实现 `text-wrap: pretty`（Chrome 未稳定，无 reftest 覆盖，defer）。
- 不重构 break_items_into_lines 的 450 行核心（Slice A 用简化计数器避之；Slice B 再评估抽取 ROI）。
