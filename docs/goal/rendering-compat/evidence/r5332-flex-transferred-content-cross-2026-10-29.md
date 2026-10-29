# R5332 — border-box flex 容器 transferred/stretch 目标 content cross 化（css-flexbox-1 §4.5 × §9.4）

- **日期**: 2026-10-29
- **驱动案**: `css/css-flexbox/flex-cross-size-border-box-001.html` ✗1.58% → **✓0.00%**
- **corpus**: 802F（R5331 交付态）→ **801F**（16590 案；**FIXED=1**，REGRESSED=0），通过率 15789/16590 = **95.2%**
- **域**: flex item transferred size suggestion × align-self:stretch 目标 × 容器 box-sizing
- **代码**: `fbf4a7be5`（fix(layout): border-box flex 容器 transferred/stretch 目标按 content box cross）

## 定性

001 驱动腿：容器 `display:flex; width:400px; height:200px; box-sizing:border-box;
border:10px solid` + img item（1×1 解码固有、ratio 1、auto 尺寸）。期望 img 被
stretch 到容器 **content box** cross = 200 − 2×10 = 180 → 180×180；ZW 渲 200×200。
像素差 7600 = 200² − 180²（img 多出的面积），单案归因自洽。

**两处同根缺陷**（容器 CSS 尺寸在 border-box 下含 frame，原值直取当 content cross）：

1. **tree.rs transferred-suggestion 臂**：容器 CSS height **原值**（border-box
   200）被当作容器 cross → transferred = 200×ratio → item `min_size.width = 200`
   （row 主轴）→ `FlexItem::new` 的 `maybe_apply_aspect_ratio` 摊入 min_height=200
   → taffy `determine_hypothetical_cross_size` `clamp(固有 1px, min 200) = 200`。
2. **sizing.rs R1364 stretch 写回臂**：修复①后 img 仍 200×200——观测插桩第三行
   `child_style_cross=Some(200.0)` 暴露：`apply_flex_aspect_ratio_item_size` 把
   `st.size.height = parent_cross_definite`（= 容器 CSS 原值 200）写回 taffy style
   → 第二趟 img cross 变 definite，不再 stretch。

**定谳方法**（learning 判例应用）：手工推演落空后立即转单点观测插桩
（`ZW_R5332_DIAG` env 门控 eprintln + 临时测试，用完回退）——
`determine_hypothetical_cross_size` 打 `child_cross_after_clamp/min/max` 定谳
`min=Some(200)` clamp 顶掉固有值；修复①后 `child_style_cross=Some(200.0)`
暴露第二处写回臂。LAYOUT_DUMP 分层（div 层全对 img 层错）先行收窄域。

规范：css-flexbox-1 §4.5 transferred size suggestion 基于 item **computed cross**
= 容器 **content box** cross（box-sizing:border-box 时 CSS 值含 frame）；
§9.4 align-self:stretch 目标同为容器 content cross。

## 实现（layout-engine 两文件、一个开关）

- `tree.rs` transferred-suggestion 臂：容器 cross 解析后按容器
  `box_sizing=BorderBox` 扣 frame——column 用 `frame_width`（水平 frame）、row 用
  `frame_size().1`（垂直 frame）；`frame_width`/`frame_size` 从私有提为
  `pub(crate)` 复用 R4163/R4165 helper（border-style none/hidden 归零语义与
  converter 同规）。
- `sizing.rs` `apply_flex_aspect_ratio_item_size`：`parent_cross_definite`
  解析处同扣（stretch 写回与 cross 推导 `cross_resolved = parent_cross…`
  共用这一处，两消费点一次修齐）。
- **kill-switch = `ZW_FLEX_TRANSFERRED_CONTENT_CROSS=0`**（default-on，两处同根
  同开关）；content-box 容器原值即 content（零变化），%/不可解析形态（None）
  不受影响；无驱动案的邻臂（R4988 `container_main_definite`、R4991 item 自身
  `cross_definite_css`）不扩范围。

## 交付

- corpus 同树 A/B（基线 R5331 交付态 802F，fail-list `/tmp/r5331-after2.txt`）→
  **801F**，集合 diff 恰 {−flex-cross-size-border-box-001}，REGRESSED=0。
- kill-switch 双向：=0 复现 img 200×200（min 180↔200 翻转，观测插桩实证）。
- trend 行 @ `fbf4a7be5` 已录。

## 回归

+3 pin（`r5332_border_box_flex_cross_tests`）：

1. **row 驱动腿**：001 形 border-box 容器 + img → content 高 180、img 180×180；
2. **content-box 对照腿**：容器无 border-box → img 200×200（CSS cross 原值即
   content，修复零变化守卫）；
3. **column 腿**：border-box 容器 width:400 + border 10 → img 380×380
   （`frame_width` 水平扣减路径，与 row 的垂直路径互补）。

## 门禁

- fmt 0（`cargo fmt --all -- --check` 全绿——此前并行流 text_encoding.rs drift
  已消，前置门恢复）；taffy-local flexbox.rs 本轮净 diff = 0（临时插桩用完回退）。
- clippy `zero-layout-engine --all-targets -D warnings` 0（guarded）。
- workspace 测试腿（guarded）：46 测试二进制 45 ok，唯一红 = zero-engine
  js_dom_bridge fetch 10F **既有家族零位移**（clean-HEAD stash 实证同红，
  账本既有维持项）。
- 本地 725 套件 724/725（唯一红 float-shrink-inline-children-zw-001 = R5311
  既有，clean-HEAD 同败实证）。
- product-smoke welcome **15.68%** ≤ 20% 零位移 + struct PASS；legacy 42
  fixtures struct-check 0 失败。
- **bench-gate 定向 zero-layout-engine GATE PASS 25 指标 NEW=0**（taffy-local
  无自有 bench 条目，端到端由 layout_bench 覆盖）。

## 挂账

- **收口** flex-cross-size-border-box-001。
- **菜单**（本轮 802F 扫描量化，未驱动勿臆断同修）：flexbox-collapsed-item-horiz-002
  ✗1.59%（visibility:collapse strut 语义）；flex-grow-009 / flex-wrap-006 /
  flex-wrap-flexing-003 各 ✗2.08%；auto-margins-003 ✗2.20%；align-self-016
  ✗4.17%；flex-svg-no-intrinsic-column-001 ✗4.69%；flex-container-max-content-001
  ✗7.33% / flex-container-min-content-001 ✗13.35%；definite-sizes-003/004 各
  ✗9.38%。
- **维持**（R5331 承接）：flex-minimum-width-flex-items-015（AR transferred ×
  % min-height，R3859 谱系另臂）、taffy automatic min 短板（flex-aspect-ratio-026
  回填补偿位依赖）、zero-webdriver 11F 未就绪家族、inline-flex 内在宽 0 观察、
  zero-engine fetch 10F + details 闪断、R5327 fragment 臂结构域、safe/unsafe
  overflow-position 域、content-none-select-1 脆弱族、c44-003 残差、va 族余案、
  c548 待用户决策。fmt drift 已消（本轮 fmt 前置门全绿）。
- **下一步 R5333**：①801F 账本续扫择轻（避开 va 域与 OOF 锚域）②
  flexbox-collapsed-item-horiz-002（1.59% 最轻，collapse strut 语义）分诊驱动
  择机 ③multicol 嵌套分片专项预研（8×1.04% 聚簇）。
