# 实施 handoff：R109 vertical inline 布局（IFC 双模式字符推进）

**版本**：v1.0（R1051，2026-07-05）
**状态**：实施 handoff 蓝图（多 session 架构；零代码风险 spec，供后续 session 实施首切片）
**承接**：R1043（vertical-rl block-flow 方向，postprocess mirror ruled out）→ R1050（★vertical inline 布局根因定位）→ 本 handoff
**关联**：
- [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)（IFC 统一，vertical 是其子问题）
- [`r109-anonymous-block-spec.md`](./r109-anonymous-block-spec.md)（R109 anonymous block，不同子问题）
- master.md R1043 / R1050

---

## 0. 问题陈述（R1050 实测根因）

ZW 的 InlineFormattingContext（`crates/layout-engine/src/inline/mod.rs`）对 `writing-mode: vertical-rl/vertical-lr` 文本**仍做水平布局**：

- LAYOUT/EMPHDBG 实测（`text-emphasis-position-property-006d`，vertical-lr，"試験テスト"，fs=16）：
  - 試@frag.x=8, 験@frag.x=24, テ@frag.x=40, ス@frag.x=56, ト@frag.x=72（**x 每字符递增 Δ16=fs**）
  - 即 IFC `current_x += char_width`（mod.rs:1042 等）水平推进每个字符
  - paint 端（text.rs:1392）`is_vertical` 设 `char_advance_is_y=true` 并旋转 glyph 90°，但底层 frag.x 仍是水平推进结果
- 规范要求（CSS Writing Modes §3）：vertical-rl/lr 中 inline base direction 是**垂直**，chars 应**同 x 列、y 递增**推进，line-break = column-break（新列在 x 方向）。

**双层 vertical 缺口**（与 R1043 互补）：
1. **block-flow 方向**（R1043）：taffy 0.7 `Display::Block` 不支持 rl 方向 packing（首子在右），postprocess mirror net-negative（float-exclusion/margin-collapse 状态丢失）。
2. **inline-flow 方向**（R1050，本 handoff）：IFC 总水平推进 chars，不支持垂直推进。

两层均 taffy 0.7 / IFC 架构限制，须 layout 重构或 taffy 升级（R304 多 session）。

---

## 1. 影响范围（ruled-out 子域）

vertical inline 布局缺失致以下 vertical-mode 子域**全部 R109-blocked**，勿再以这些为单 session lever（R1050 已证 text-emphasis vertical net -8）：

- `text-emphasis-position` vertical 簇（003/005/006 d/e/f/g，~12 案 1.00-1.02%）
- `ruby` vertical annotation（R1022 只支持水平 rt 上移）
- `text-decoration` vertical（underline/overline 在 vertical 应 left/right）
- `bidi-vertical`（bidi-007 簇残余部分）
- `css-writing-modes` vertical 用例（block-flow-direction-* / line-box-direction-*，86-87% worst）
- `logical-props` vertical 映射（R1048/R1049 WM-aware 化已对，但 vertical 渲染缺此布局）

**解锁 yield**（估计）：css-writing-modes 56/784 (7%) → 大幅提升（vertical 用例占 ~250 案，多数 86-87% 因水平布局错位；vertical inline 修对后，这些案的字符位置正确，残余仅 font/aa 噪声，潜在 flip ~30-80 案）。这是当前 corpus 最高 yield 单轨道。

---

## 2. 实施路径（IFC 双模式）

IFC layout（mod.rs:973-1200+）当前用 `current_x`（char 推进）/ `current_y`（line 推进）。vertical 模式需轴交换：

| 维度 | horizontal-tb（当前） | vertical-rl/lr（目标） |
|------|----------------------|----------------------|
| char 推进轴 | current_x（→） | current_y（↓） |
| line/column 推进轴 | current_y（↓） | current_x（→ 或 ←，依 rl/lr） |
| line-box 几何 | width=avail, height=line-height | height=avail, width=line-height |
| 字符宽度 | advance_width | font_size（CJK square）/ advance_height |
| 换行触发 | current_x + word_w > avail_width | current_y + word_h > avail_height |

**关键改动点**（mod.rs）：
- `layout()`（line 500）：根据 `self.is_vertical` 选择推进轴
- `current_x`/`current_y` 语义互换（vertical：x=line/column, y=char）
- `split_into_words` + word-wrap：宽度比较改高度比较
- `LineBox` 几何：vertical 下 width=line-height, height=avail
- `effective_content_area`（float exclusion）：vertical 下 float 区在 x 方向
- `estimate_char_width` → vertical 用 font_size（CJK）或垂直 advance

**paint 端**（text.rs）：`char_advance_is_y` 分支已存在（line 1392, 1446），frag.x/frag.y 语义需对齐 IFC vertical 输出（frag.x=列 x 常量，frag.y=char y 推进）。R1050 的垂直 emphasis 块（已回退）可在 IFC vertical 修对后重新启用。

---

## 3. 首切片建议（紧 gate，多 session 起步）

**Slice 1（enabling，目标 net-0 守horizontal-tb）**：
- 在 IFC `layout()` 加 `if self.is_vertical { ... }` 分支，**仅处理最简 case**：
  - 单列（无 column-break，内容不溢出 avail_height）
  - 无 float exclusion
  - 无 inline-block 子（ib_sizes 不参与）
  - 纯 CJK 文本（每个字符 advance = font_size，无 word-wrap）
- 输出 frag.x = 列 x（常量），frag.y = char_y（每字符 fs 递增）
- gate：`is_vertical && 单列 && 无 float && 无 ib && 纯 CJK`
- 验证：1 个 vertical-lr CJK 单行用例（如 006d）frag 几何对（chars 同 x，y 递增）；horizontal-tb 字节一致（gate 隔离）；make test 全绿；product-smoke welcome 持平（welcome 非 vertical）。

**Slice 2+（后续 session）**：
- word-wrap（vertical 换列）
- float exclusion（vertical float 区）
- inline-block 子（ib_sizes 轴交换）
- Latin 字符 advance（垂直 advance 而非 font_size）
- text-orientation: sideways/mixed（glyph 旋转 + 推进方向）

每个 slice 独立 A/B 守回归、零回归即留（类 R639/R1048/R1049 foundational 模式）。

---

## 4. 风险与依赖

- **taffy 0.7 限制**：vertical BLOCK flow 方向（R1043，rl packing）仍 taffy-blocked。本 handoff 只解 inline flow（char 推进方向）；block flow 方向须 taffy 升级（R304）或 converter 层镜像。两层独立可分步。
- **paint Path B 空-styles**（R890）：IFC vertical 输出须与 paint Path B 协调（store_font_sizes_from_ifc 已是 override-map 模式，可扩展存 vertical 几何）。
- **line-height/baseline**（R630/R990）：vertical 模式下行盒高度 = line-height，baseline 在 x 方向，须三方同改（类 R834 单点 net-negative 先例）。
- **horizontal-tb 零回归**：所有改动须 `if self.is_vertical` gate，horizontal-tb 路径字节一致。

---

## 5. 验证基础设施

- `LAYOUT_DUMP=1 ... reftest-upstream <case>`：dump frag abs_y/height/margin（已用于 R1050 诊断）
- `EMPHDBG=1`（R1050 加，已回退）：可重新加回诊断 vertical char 位置
- `make reftest-oracle css-writing-modes`：vertical 用例 oracle 率（当前 56/784）
- A/B stash 对照（horizontal-tb 字节一致验证）

---

## 6. 裁决

R109 vertical inline 布局是当前 corpus **最高 yield 单轨道**（css-writing-modes ~250 vertical 案 + 间接解锁 emphasis/ruby/text-decor/bidi-vertical）。多 session 架构，首切片（Slice 1 纯 CJK 单列）可独立提交 net-0 守 horizontal-tb，逐步扩展。本 handoff 提供精确靶点（IFC `current_x`/`current_y` 轴交换 + paint `char_advance_is_y` 协调），后续 session 可按 §3 slice 序实施。

**勿再**以 vertical 子域（emphasis/ruby/text-decor/bidi-vertical）为独立 lever——须先 R109 vertical inline 解锁（本 handoff）。
