# R5334 — scroll container flex item 自动最小=0（css-flexbox-1 §4.5 × css-overflow-3 §3.1 × R1743 回填臂）

- **日期**: 2026-10-30
- **驱动案**: `css/css-flexbox/fieldset-as-item-overflow.html` ✗1.91% → **✓0.20%**（passed，残差 = 滚动条绘制细节过阈）
- **corpus**: 800F（R5333 交付态；同树 A/B）→ **799F**（16590 案；**FIXED=1**，REGRESSED=0），通过率 15791/16590 = **95.2%**
- **域**: column flex item automatic minimum × R1743 父高回填豁免半区
- **代码**: `0d4430775`（fix(flex): scroll container item 自动最小=0——fieldset-as-item-overflow ✓ 回填豁免半区）

## 定性

案情（chromium bug 1149714）：column flex 容器 100×100 + border solid，fieldset item
（margin/border/padding 0 + `overflow-y:scroll;overflow-x:hidden` + `flex:1`）含
200px 高子 div。期望 fieldset 被 shrink 到容器内（scrollable overflow 滚动呈现）；
ZW 渲 item 高 200 溢出容器（PNG dump：test 200px 绿块出界 + 竖滚动条 / ref 收缩
+ 底部滚动条位）。

**三级分诊链**（临时诊断测试 `r5334_diag` 用完即删，变体矩阵 + kill-switch 扫描）：

1. **div/fieldset 对照**：div item 同构同败（200）——**非 fieldset 元素特例**；
   style 层 computed = Hidden/Scroll 正确到达（`ComputedStyle.overflow_y=Scroll`
   实证），converter 映射（`convert_overflow` Scroll→taffy Scroll）在位。
2. **变体矩阵**（column 容器 100×100 border，item `flex:1` + 子 200px）：

   | 变体 | item 高 | 判定 |
   |---|---|---|
   | A `overflow` visible | 200 | §4.5 正确（automatic min = min-content 200） |
   | B `overflow-y:scroll` | **200 ✗** | 应 100——automatic min 应为 0 |
   | C B + `min-height:0` | 100 ✓ | 指定 min 生效，flex 分配臂无恙 |
   | D `min-height:0` only | 100 ✓ | 同上对照 |
   | E row + `overflow-x:scroll` | 宽 100 ✓ | row 主轴 min-width:auto 禁用生效 |

   taffy :870 `child.min_size.or(child.overflow.map(maybe_into_automatic_min_size))`
   轴无关——B/E 差异**不在 taffy**。
3. **kill-switch 扫描**：`ZW_IFC_PARENT_HEIGHT_BACKFILL=0` → B 200→**100 ✓**，
   其余变体不变——定谳 **R1743 父高回填 pass**。回填把 height:auto 的 item
   （`box_node.declared_height_auto`）当「内容决定型 auto 高容器」撑到子 max-bottom
   200。R5330 豁免谓词 `is_col_flex_item_no_backfill` 仅含**指定 min-height≠auto**
   半区（豁免 automatic minimum 被显式关闭的 item），min-height:auto + scroll
   container item 落在「taffy automatic min 短板补偿位」名义下被保留回填——但
   scroll container 的 automatic minimum **本就是 0**（§4.5 + css-overflow-3），
   taffy 侧已按 0 定尺，回填把正确的 flex 定尺重新撑爆。

规范依据：css-flexbox-1 §4.5「automatic minimum size of a scroll container is
zero」；css-overflow-3 §3.1 传播（一轴非 visible/clip → 另轴 visible→auto）；
R3754 ar_transferred 臂同型先例（:1902「滚动容器无 content-based minimum，内容
不撑盒」）。

## 实现

- **converter/mod.rs**：`computed_overflow_pair` 提 `pub(crate)`（visibility only；
  R4265 §3.1 传播单语义源，postprocess 复用避免口径分裂）。
- **postprocess.rs `is_col_flex_item_no_backfill`**：指定 min-height≠auto 半区
  不动，**或**分支新增 scroll container 判定——computed pair 传播后任一轴非
  `Visible|Clip`（hidden/scroll/auto）即豁免回填。row 容器（cross 域）不涉；
  kill-switch 沿用 R5330 `ZW_FLEX_COL_ITEM_NO_HEIGHT_BACKFILL=0`（default-on）。

## 交付

- corpus 同树 A/B：800F → **799F**，fail-list 集合 diff 恰 `{−fieldset-as-item-overflow}`
  （`diff /tmp/r5333-after-fails.txt` 新清单唯一删行），REGRESSED=0。
- 驱动案单跑 ✗1.91% → **✓0.20%**（passed，>strict 近似通过档：残差为滚动条
  绘制细节——ref 底部横滚条位 vs ZW 竖滚条呈现，过阈非语义差）。
- trend 行 @ `0d4430775` 已录（15791/16590 = 95.2%）。

## 回归

+3 pin（`r5334_flex_scroll_item_min_tests`）：

1. **驱动腿**：fieldset item（驱动案原元素）保持 flex 分配高 100 不被撑回 200；
2. **对照腿**：overflow visible item automatic minimum = min-content 200 保留
   （R5330 短板补偿位仍在，防豁免过宽到 visible 域）；
3. **守卫腿**：row 容器 overflow-x:scroll item 宽保持 flex 分配 100（cross 域
   既有行为锚定）。

## 门禁

- fmt 0——**wpt-runner 既有 drift 修复后恢复**：`8a71e4f29` 的
  `run_css_corpus_dir` 单签名超宽使 `cargo fmt --all -- --check` 持续红、
  T58 fmt 前置门阻断 `make test` 全量入口；单文件 rustfmt 重排零行为变化，
  独立 commit `bf476fd73`。
- clippy `zero-layout-engine --all-targets -D warnings` 0（guarded）。
- workspace 测试腿 46 binary **45 ok**，唯一红 = zero-engine fetch 家族
  **3600P/9F**（既有维持项零位移，R5333 同签名）。
- 本地 725 套件 **724/725**（唯一红 = R5311 既有 float-shrink-inline-children-
  zw-001 3.68%，零位移）。
- product-smoke welcome **15.68%** ≤ 20% 零位移 + struct PASS + legacy 42
  fixtures 0。
- **bench-gate 定向 `zero-layout-engine`**：静窗（loadavg 0.73/9.54/11.19）
  **GATE PASS 25 指标 NEW=0** 全在预算内。
- **跨面门禁恢复**（engine 流域，独立 commit `78754deb3`）：29923b68f 的 s151
  compatMode 桥把 `__zw_get_compat_mode` 闭包直指 v8 门控 `dom_bindings` 模块，
  quickjs 腿（`--no-default-features --features quickjs`）E0433 编译断——
  `quickjs_dom_bindings` 补同语义 twin（自有 with_dom + QuirksMode 回落
  CSS1Compat 同口径）+ callbacks `compat_mode_callback()` 按 feature cfg 取源
  （v8 / quickjs / 双缺三臂，双缺回落常量同口径）；v8 路径零变化；两 feature
  组合 `cargo check` 验证 + 重跑 make test E0433 清零。

## 挂账

- **收口** fieldset-as-item-overflow。
- **菜单/维持**（R5333 承接）：flexbox_align-items-baseline ✗1.77%、
  css-flexbox-test1 ✗1.99%、flex-wrap-006、auto-margins-003 ✗2.20%、
  align-self-016 ✗4.17%、flex-svg-no-intrinsic-column-001 ✗4.69%、
  flex-container-max/min-content-001 ✗7.33%/13.35%、definite-sizes-003/004
  各 ✗9.38%、015（AR transferred × % min-height）、taffy automatic min 短板、
  zero-webdriver 11F、inline-flex 内在宽 0 观察、zero-engine fetch 9-10F +
  details 闪断、R5327 fragment 臂结构域、safe/unsafe overflow-position 域、
  content-none-select-1 脆弱族、c44-003 残差、va 族余案、c548 待用户决策、
  bench NEW=8 基线纳入（静窗窄捕获）、taffy shrink ∞ 臂。
- **下一步 R5335**：①799F 账本续扫择轻（避开 va 域、OOF 锚域、writing-mode
  域）②flexbox_align-items-baseline（1.77% 最轻）/ css-flexbox-test1（1.99%）
  分诊驱动择机 ③multicol 嵌套分片专项预研（8×1.04% 聚簇）。
