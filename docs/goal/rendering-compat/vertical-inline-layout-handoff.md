# 实施 handoff：R109 vertical inline 布局（IFC 双模式字符推进）

**版本**：v1.1（R1052，2026-07-05，**纠正 R1051 诊断 + 耦合系统裁决**）
**状态**：实施 handoff 蓝图（多 session 架构）—— R1052 实测后 §0/§2 已纠正，§3 Slice 1 单点修复经实证 **net-negative 已 ruled out**，须多层同步修
**承接**：R1043（vertical-rl block-flow 方向，postprocess mirror ruled out）→ R1050（vertical inline 布局根因初定位）→ R1051（本 handoff v1.0）→ R1052（**纠正诊断 + 耦合裁决**）
**关联**：
- [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)（IFC 统一，vertical 是其子问题）
- [`r109-anonymous-block-spec.md`](./archive/r109-anonymous-block-spec.md)（R109 anonymous block，不同子问题，已归档）
- [`evidence/r1052-vertical-ifc-container-width-zero-2026-07-05.txt`](./evidence/r1052-vertical-ifc-container-width-zero-2026-07-05.txt)（R1052 实证 + 探针代码）
- master.md R1043 / R1050 / R1051 / R1052

---

## 0. 问题陈述（★ R1052 VIFCDUMP 实测纠正 R1050/R1051 诊断）

**R1051 v1.0 诊断（已过时）**：断言 IFC 对 vertical 文本「水平布局，chars x 递增」是因「缺轴交换（current_x/current_y 未互换）」，须新建 IFC 双模式 char 推进。

**R1052 VIFCDUMP 实证推翻**：轴交换代码**早已存在且正确**（commit 942a2948，2026-06-09）：
- `break_items_into_columns`（mod.rs:1450）已做轴交换：字符沿 y 推进（`current_depth`/`partial_depth`，frag.y 递增），列沿 x 推进（`col.y`，frag.x = col.y 列内常量）。
- paint 端（painter/text.rs:1392-1450）`char_advance_is_y` 已正确：glyph_x = frag_base_x（常量），glyph_y = char_pos（每字符递增）。

**真根因（R1052 定位）**：vertical IFC 的 `container_width = 0.0` → `max_depth = 0`。
- `break_items_into_columns` 里 `let max_depth = self.container_width`（mod.rs:1452）。
- `container_width` 来自 `root.content_width`（inline_finalization.rs:619）/ `box_node.content_width`（painter/text.rs:797）—— 是元素的**水平 block 尺寸**。
- vertical-lr 容器的 content_width = block 轴（水平）尺寸，auto 时 = 0 → max_depth=0 → `current_depth + word_height > 0` 恒真 → **每字符触发列断 → 每字符各占一列、列沿 x 排列 → chars 横向排列**（x 递增、y 恒 0）。
- vertical 应取 `content_height`（竖直 inline 尺寸 = 字符向下推进的可用深度），非 content_width。
- R1050 EMPHDBG 测到的「chars x=8,24,40,56,72 递增」正是 max_depth=0 致每字符一列的副产物，**非**缺轴交换。

VIFCDUMP 原始输出（006d，HEAD 状态）见 [`evidence/r1052-*.txt`](./evidence/r1052-vertical-ifc-container-width-zero-2026-07-05.txt) §1c。

**双层 vertical 缺口**（与 R1043 互补，R1052 确认）：
1. **block-flow 方向**（R1043）：taffy 0.7 `Display::Block` 不支持 rl/lr 方向 packing（首子在右/左），postprocess mirror net-negative（float-exclusion/margin-collapse 状态丢失）。
2. **inline-flow 方向**（R1050/R1052，本 handoff）：IFC `container_width=0`（取 content_width 而非 content_height）→ max_depth=0 → 每字符一列横向排列。

两层均 taffy 0.7 / IFC 接线限制。inline-flow 修法已明确（container_width WM-aware，见 §2），但**单修 net-negative**（见 §3 R1052 实证）。

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

## 2. 实施路径（★ R1052 纠正：container_width WM-aware，非新建双模式）

**R1051 v1.0 路径（已过时）**：「IFC 双模式 char 推进」「轴交换 current_x/current_y」—— 错误诊断，轴交换已存在（§0）。

**R1052 正确修法（2 处接线 + trailing-space 裁剪）**：

Fix A — container_width WM-aware（2 处）：
- `inline_finalization.rs:619`（compute_final Path A stored）：
  ```rust
  let is_vertical_wm = !matches!(root.writing_mode, WritingModeValue::HorizontalTb);
  let container_width = if is_vertical_wm { root.content_height } else { root.content_width };
  ```
- `painter/text.rs:797`（paint Path B re-run）：同上（`box_node.content_height`）。
- horizontal-tb 路径取 content_width，字节一致零回归（gate 隔离）。

Fix B — trailing-space 裁剪（break_items_into_columns，mod.rs:1489 词循环头）：
- `split_into_words` 为非末词加 trailing space（mod.rs:1983-1987），但注释明示 CJK per-char 词「不带尾部空格」——矛盾 bug。vertical 下 `word_height` 虚高（fs+space_w，16+4=20）→ 列断提前 + 位置漂移。
- Fix：`let word = if self.preserve_whitespace { word.as_str() } else { word.trim_end_matches(' ') };`，空词跳过，`word.clone()`→`word.to_string()`。

Fix A+B 实测 006d：chars 几何**完全规范正确**（col0 run0..4 x=0 常量，y=0,16,32,48,64 连续 fs 间隔，单列）。

**paint 端**（text.rs）：`char_advance_is_y` 分支已存在且正确（line 1392-1450），无需改。Fix A 让 paint Path B 的 ifc_width 也取 content_height 即可。

---

## 3. ★ R1052 实证裁决：Slice 1 单点修复 net-negative，须多层同步修

**R1051 v1.0 Slice 1（纯 CJK 单列紧 gate）已实证 ruled out**：

R1052 按 §2 实施 Fix A + Fix B（gate 宽于 v1.0 Slice 1，但 006d 即「纯 CJK 单列」最简 case）：
- 006d 字符几何修到**完全规范正确**（chars 同 x 列、y 按 fs 连续递增、单列）。
- **A/B（chromium Oracle）**：css-text-decor **108→82（净 -26 PASS→FAIL）**；css-writing-modes 56→56（net-0，block-flow R1043 主导）；006d 单案 1.00%→1.01%（基本持平）。
- 即便字符几何正确，oracle 仍 net -26。

**根因 = vertical 渲染是耦合系统**（R1047/R1050/R1052 三证）：单修 inline-flow（char 推进）不足以匹配 chromium，因其他层仍错：
1. **block-flow 方向（R1043）**：taffy 0.7 Block 不支持 vertical-rl/lr 方向 packing，vertical 容器自身定位错。line-box-direction-vlr-014 修后仍 86.86%。
2. **line-height（break_items_into_columns：col_width = run.line_height）**：006d line-height:5 应让列宽=80，实测 col_width=16（line-height:5 未传 vertical 列宽）。
3. **emphasis/ruby/text-decor vertical**：R1050 site 2 `!char_advance_is_y` 门控跳过（vertical 装饰完全未渲染）。
4. **paint Path B 空-styles（R890）**：vertical override 度量须协调。

单修 inline-flow 致输出**既不同于旧错误布局、又不同于 chromium** → 净负。**Slice 1 单点修复 ruled out**。

**修订实施路径（多 session，同步修）**：
- **Slice α（同步四层）**：container_width fix（§2 Fix A+B）+ block-flow（R1043 converter 层镜像，**非** postprocess）+ line-height vertical 列宽 + vertical emphasis re-enable（R1050 块），四层**同改**后 A/B 守 net-0/正。任一层缺则 net-negative（R1052 实证）。
- 或 **taffy 升级（R304）**解锁 native vertical block-flow，减一层耦合，再单修 inline-flow 可能 net-0/正。

**勿再**单点修 vertical 任一子层（含 §2 Fix A+B 单独）——R1047/R1050/R1052 三证 net-negative。

---

## 4. 风险与依赖（R1052 更新）

- **★ 耦合系统（R1047/R1050/R1052 三证）**：vertical 渲染 = block-flow（R1043）+ inline-flow（container_width）+ line-height（vertical 列宽）+ emphasis/装饰，**任一单修 net-negative**。R1052 实证 inline-flow 单修（即便字符几何完全规范正确）oracle 净 -26。**必须同步修多层**，不可分步单点提交。
- **taffy 0.7 限制**：vertical BLOCK flow 方向（R1043，rl/lr packing）仍 taffy-blocked。须 taffy 升级（R304）或 converter 层镜像（**非** postprocess，R1043 已证 postprocess mirror net-negative）。
- **paint Path B 空-styles**（R890）：IFC vertical 输出须与 paint Path B 协调（store_font_sizes_from_ifc 已是 override-map 模式，可扩展存 vertical 几何）。
- **line-height vertical 列宽**（R1052 新发现）：`break_items_into_columns` 里 `col_width = run.line_height`，但 006d line-height:5 实测 col_width=16（应 80）—— line-height:5 未传 vertical 列宽。须同修。
- **horizontal-tb 零回归**：所有改动须 WM gate（`is_vertical_wm`），horizontal-tb 路径取 content_width 字节一致。

---

## 5. 验证基础设施

- `VIFCDUMP=1`（R1052 加，探针代码见 [`evidence/r1052-*.txt`](./evidence/r1052-vertical-ifc-container-width-zero-2026-07-05.txt) §6）：dump compute_final content_w/h + break_items_into_columns per-col/per-frag 几何 —— vertical IFC 调解决定性工具（须重新加回代码，本轮已 revert）。
- `LAYOUT_DUMP=1 ... reftest-upstream <case>`：dump frag abs_y/height/margin（R1050 用）。
- `EMPHDBG=1`（R1050 加，已回退）：可重新加回诊断 vertical char 位置。
- `make reftest-oracle css-writing-modes` / `css-text-decor`：vertical 用例 oracle 率（当前 css-writing-modes 56/784，css-text-decor 108/242）。
- A/B stash 对照（horizontal-tb 字节一致验证 + oracle 净变化）。

---

## 6. 裁决（R1052 更新）

R109 vertical inline 仍是当前 corpus **最高 yield 单轨道**（css-writing-modes ~250 vertical 案 + 间接解锁 emphasis/ruby/text-decor/bidi-vertical），但 R1052 实证 **inline-flow 单修 net-negative**（-26 css-text-decor），须 **block-flow + inline-flow + line-height + emphasis 同步修**（Slice α 四层同改）或 **taffy 升级（R304）减耦合**后方可 net-0/正。

R1052 已纠正 R1051 v1.0 诊断（轴交换已存在，真根因 = container_width=0）并锁定精确靶点（§2 Fix A+B + §4 line-height vertical 列宽），后续 session 可按 Slice α 同步实施，**勿再单点修**。

**勿再**以 vertical 子域（emphasis/ruby/text-decor/bidi-vertical）为独立 lever——须先 vertical 多层同步解锁（本 handoff Slice α）。
