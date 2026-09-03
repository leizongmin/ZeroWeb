# flex §9.9 suggestion 链实施设计（R4002 前置设计，user-gated 实施前置）

- **日期**：2026-09-04
- **状态**：设计完成，待用户放行实施（goal Execution Protocol §4「深结构必须先写
  可回退实施设计」）
- **驱动面**：css-flexbox 残差 img-row-009/017、img-column-004/013/018、
  flex-aspect-ratio-intrinsic-padding-001、flex-minimum-height-flex-items-022、
  intrinsic-size-007/010（共约 10 案离散带，单案 1-64%）
- **历史约束**：R3998b 负结果（反解比 + content 重推不可行）、R1013/R1366（flex
  transferred 既有语义）、R3829/R4001（table 同族先例：先标记后排序）

## 1. 问题定性

ZW 的 flex item sizing 依赖 taffy 0.12 的 `aspect_ratio` + definite 尺寸输入，
缺 css-flexbox §9.9 的「suggestion 链」——flex base size 确定前对 replaced item
的 content-based 贡献按以下顺序合成：

1. **specified size suggestion**（§9.9.1.1）：item 有 resolvable 定 main size
   （含可解析 %）→ 用该值。
2. **transferred size suggestion**（§9.9.1.2）：item 有固有比 + 定 cross size →
   用 cross ×/÷ 比（**content-box 语义**——但 frame 归属按案例族分立，见 §3）。
3. **content size suggestion**（§9.9.1.3）：min/max-content。

且三者均受 **automatic minimum size**（§9.9.2 min-width/height:auto floor，含
scroll-container 单轴豁免——上游 2026-05 新语义，见 §5）钳制。

## 2. 案例族 ↔ 缺口映射

| 案 | 现象 | 缺失机制 |
|---|---|---|
| img-row-017 | `width:100%` + `flex:0 0 10px` → ZW 停在 10px | (1) specified suggestion 未消费可解析 % main size |
| img-column-004 | `height:100%`（容器 min-height 定）+ 300×150 → ZW 塌 0 | (1) % main size 对 min-height 定容器的 resolvability 判定 |
| img-row-009 | SVG w=50 attr + border-left 50 in 50px 容器 → ZW 50×100 | (2)+(3) shrink 后 content 0 时 transferred 高未随 shrink 重推；§9.7 min-content flooring |
| img-column-013 | SVG h=50 attr + border-top 50 → ZW 宽 200 应 100 | (2) transferred 的 frame 归属（见 §3） |
| intrinsic-padding-001/022 | 对称 padding / 仅 border-bottom | (2) frame 对称性决定归属（R3998b 实证边界） |
| intrinsic-size-007 | flex column svg ratio-only → 64% | (2) 无 abs 尺寸 svg 的 transferred（R4000 已给 0×0 used，flex 路径未接） |

## 3. 核心设计难点：transferred 的 frame 归属

§9.9.1.2 的 transferred size 按规范作用在 content box；但 R3998b 实证两个边界：

- **对称 padding**（intrinsic-padding-001）：chromium 结果与 border-box 推导一致
  ——naive content-box 重推翻红。
- **仅 border-bottom**（flex-minimum-height-flex-items-022）：chromium 结果与
  border-box 推导一致——同上。

推断（chromium 行为反推）：taffy/chromium 的 transferred 推导作用在
**margin-box 参照的 def 主轴尺寸减去非主轴 frame 后的 content 尺寸**，且
automatic minimum flooring 在后。**首版设计不做完整 §9.9 合成**，只做可证明
正确的最小面：

### Slice 1（§9.9.1.1 specified suggestion，% resolvability）

- 输入面：replaced flex item（`is_flex_grid_item && is_replaced`），CSS main size
  为 `Percentage` 且 CB main size **definite**（taffy 输入已可判定：容器 display
  flex + 容器 height/min-height Px）。
- 行为：`LengthValue::Percentage` 的 main/cross 经解析后作为 flex base 的
  specified 值参与（当前被 resolve_inline_block_dimension / converter 丢弃为 auto）。
- 落点：`tree.rs` `apply_replaced_element_sizing` 的 flex-item 臂前置 pass。
- 驱动验证：img-row-017（100→100px base）、img-column-004（height 100%→500 →
  ratio 传宽 250→容器钳 100）。
- 可回退：kill-switch `ZW_FLEX_SPECIFIED_SUGGESTION=0`。

### Slice 2（§9.9.1.2 transferred 的 svg ratio-only 面）

- 输入面：R4000 后 viewBox/CSS-ar-only svg used size = 0×0；flex item 面改为
  「ratio + 定 cross」语义：flex column（cross=width 定）→ transferred main
  height = cross × ratio 分母；flex row 对称。
- 落点：同一前置 pass，在 Slice 1 之后：无 specified 时按 cross 定 → transferred。
- 驱动验证：intrinsic-size-007（column, viewBox 2:1 → cross 784 → main 392）、
  010（max-height 100 钳）。
- 风险：taffy aspect_ratio 已做部分推导——pass 需在「taffy 未推导出非 0」时才写
  （R3929 先例：taffy 已解非 0 不覆写）。
- 可回退：`ZW_FLEX_TRANSFERRED_SUGGESTION=0`。

### 明确不做（本轮边界）

- frame 归属的完整修正（013/009/001/022 族）——等 Slice 1/2 落地后的 A/B 数据
  再定（可能随 automatic-minimum 交互自然收敛，也可能需 §9.9.2 完整实现）。
- §9.9.2 automatic minimum 完整实现（含 scroll-container 单轴豁免——上游新语义
  需独立归因轮）。

## 4. 回退与门禁

- 每 Slice 独立 kill-switch；两 Slice 均为 tree.rs 前置 pass（postprocess 免疫
  R3998b 教训——布局期输入面而非事后改写）。
- 每轮全量 fail-set XOR 净负即回退（R3914 先例：四态全负完整回退零残留）。
- 门禁：make test + make reftest 687 + product-smoke 同值 + bench-gate layout-engine。

## 5. 上游动态备注

css-flexbox 上游 main 较本地 wpt-data 快照多 502 文件（percentage-heights 波 ×24
等），但**全部为 testharness.js**（rel=match 扫描 0 命中）——reftest corpus 无新
可导入面；css-tables 新 77 文件同为 testharness。corpus 扩容需等待上游 reftest
新增或转向 testharness 计分体系（后者 = 计分口径变更，**user-gated**）。

上游语义动态：flex/grid automatic minimum 的 scroll-container 判定已改单轴
（chromium 1392145）——§9.9.2 实现时须按新语义。
