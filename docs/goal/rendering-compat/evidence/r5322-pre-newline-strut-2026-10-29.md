# R5322 — pre 容器 `\n` 强制断行空行 strut 写回（preserve 语境收窄）（CSS Text 3 §5.2 / CSS2 §10.8.1）

- **日期**: 2026-10-29
- **驱动案**: `css/CSS2/text/white-space-008.xht` ✗6.53% → **✓0.65%**
- **corpus**: 816F → **813F**（案数 16592 → **16590**，账本清洁 −1F−1P；**FIXED=2**，
  REGRESSED=0），通过率 15777/16590 = **95.1%**
- **域**: css-text white-space（pre `\n` 强制断行空行行盒）× R1286/R1476/R5298 行盒 strut 链

## 定性（R5321 下一步 ②CSS2/css1 邻案顺藤）

`white-space-008`：test 页 green div（`white-space:pre`）源码 `\n\nX` 须 3 行
（2 空行 + X），red 参考页 `<br/><br/>X` 同 3 行全遮。ZW 实渲染 green 仅 **1 行**
（18px），red 下缘露出（dump 实测 lime y=50..67 h=18、red 可见 y=68..103；ZW 的
red 侧本就 3 行 54px 正确）。顺藤案 `c548-ln-ht-000`（✗3.71%）控制块
`<pre style="font:30px/1">` 空行行高仅 20px——墨迹行距 50px ≠ line-height 语义
的 60px。

共享根因（break_lines.rs）：`\n` 强制断行 marker 消费点推空行时只推进
`current_y`（est_height）**不写回 `current_line.height`**，块高按行盒 height
合计（同函数末尾 debug 断言同口径）时空行蒸发；且 est_height 落到硬编码
`default_line_height = 20.0`，不吃容器 used line-height。R1286 的 `<br>` 空行
路径正是显式 `current_line.height = est_height` 才保住 `<br><br>X` 3 行（本轮
控制腿 54px 实证）——`\n` marker 与 `<br>` 同为强制断行，语义应同权
（CSS Text 3 §5.2）。

### c548-ln-ht-000 结构性余差（另案，不挂账）

dump 逐区分解：block1（.test div）残差 0.428%、block2 3.265%。block2 本刀修复后
空行 20→30px、行距归位 60px，但 ref 是 table 版式（`border-spacing:28px` 行距
58px、content 顶 inset 29px）——`line-height:30px` 的行距 60px 与 table 行距
58px **按 CSS 语义不可收敛**，block1 的 1–3px 系统错位同理。该案为 CSS1 时代
像素容差套件的结构性余差，留在账本（是否引入「结构性案」排除口径属策略决定，
见待用户决策）。

## 实现（break_lines.rs 两处，kill-switch `ZW_PRE_NEWLINE_STRUT=0` 回退）

1. **preserve 容器空行 strut 取容器行高**：新增 `preserved_empty_line_strut`
   ——R5298 `container_strut_line_height > 0`（`layout()` 有容器 styles）时取
   strut 值，否则 20px legacy；供 `\n` marker 空行 est。**只收窄到 preserve
   语境**：首版全量换共享 `default_line_height` 绑定实测
   `ruby-align-001/001a` 0.77% → 1.02% 越阈翻红（`16px/3` 容器的非 preserve
   `<br>` 空行 20px 地板与浮动排除估算是 corpus 承重近似；kill-switch A/B 归因：
   OFF 0.77 ✓ / ON 1.02 ✗）→ 回退 legacy，另案入账。未捕获（paint Path B/旧
   调用，值 0）与容器显式 `line-height:0`（R5303 两义分离，strut 盒 0 高语义
   未建模 FIXME）落 20px。
2. **`\n` marker 空行写回 height**：marker 消费点 `current_line.height =
   est_height`（与 R1286 Br 臂同构；非空行 est ≡ 现值恒等；kill-switch off =
   不写回，精确回退旧行为）。marker 仅在 preserve/break_at_newline run 产出，
   天然限于 preserve 语境；pre-line 的 marker fragment 行由 R1476 裸空塌缩接管
   （既有口径，本刀不动）。

## 回归测试

`crates/engine/src/paint/tests/r5322_pre_newline_strut_tests.rs` +3 pin：

1. `pre_leading_newlines_match_br_strut_lines`——white-space-008 真镜像：green
   pre `\n\nX` h=**54.0** vs red `X<br>`+源码换行 h=**54.0** 同高（修前 green
   18/red 54 实测红）；
2. `pre_empty_line_strut_uses_container_line_height`——`line-height:30px` 的
   `\n\nX` = **90.0**（3×30；防「硬编码 20px 修行数」伪修法，修前 30.0）；
3. `br_br_x_control_still_three_lines`——`<br><br>X` 控制腿 54.0（R1286 不回退）。

**mutant 方向**：`ZW_PRE_NEWLINE_STRUT=0`（回退 legacy 实现路径）实测 ①② FAIL
（green 18/30）③ PASS——双向可判别；收窄版复测同构（t6/t6-off 双跑）。

## 账本清洁（corpus 案数 16592 → 16590）

删除历史轮次遗留的未注册探针页（均无 `rel=match/mismatch` 引用、不在
imported-tests.txt、无其他文件引用，grep 核实）：

- `css/CSS2/css1/r5297-probe-a.html`（在账案：`match about:blank` + 可见内容 =
  永久失败污染，−1F）
- `css/css-ui/r5302probe.html` + `-ref.html`（在账通过对，−1P）
- `css/CSS2/fonts/ahem_min.html`、根目录 `probe_{ca,divs,lh,revalia}.html`
  （不在账的惰性杂物）

## 门禁

- fmt 0；clippy zero-layout-engine + zero-engine `--lib -D warnings` 0
  （rebase 拉入上游 net-api M2-S6 提交，顺带修掉 js_dom_bridge 既有 4 lint）。
- **zero-layout-engine lib 2231P/0F**；**zero-engine lib 3591P/9F**（8F = R5320
  记账 js_dom_bridge fetch 既有红 + 1F = R5321 记账 details 闪断案，clean HEAD
  同红非本轮）。
- **corpus 全量（最终树）**：16590 案 **813F**，A/B 对照 816F 旧账：
  −`r5297-probe-a.html`（删除）、−`white-space-008`（FIXED，✓0.65%）、
  −`webkit-line-clamp-032`（FIXED，页面 `white-space:pre` preserve 路径，OFF
  实测回红 5.23% 归因本刀）；**新红 0**。首版全量绑定的额外翻红
  `first-line-line-height-003`/`abspos-block-level-001` 随 preserve 收窄回账
  （strut 邻接案，未来轮次按语境评估）。
- **product-smoke**：welcome vs chromium oracle 15.68%（≤20%）struct PASS；
  **legacy** 42 fixtures struct 0 失败。
- **bench-gate 判定不适用**：strut 求值为每 IFC 一次（LazyLock flag + 字段读），
  断行逐词热循环零新增开销；不在 bench 测量的布局/解析热循环内。

## 挂账

- **收口**：white-space-008（pre `\n\nX` 3 行 strut）、webkit-line-clamp-032
  （preserve 路径顺带）。
- **结构性余差**：c548-ln-ht-000（test/ref 版式几何不等，见上）；是否设「结构性
  案」排除口径待用户决策。
- **另案（本刀收窄回退域）**：非 preserve `<br>` 空行与浮动排除估算的 20px
  地板换 strut——ruby-align-001/001a（0.77%）承重，需先解决 inline ruby 行盒
  与 strut 的几何口径再动；`first-line-line-height-003`/`abspos-block-level-001`
  随之复评。
- **邻案观察（不挂账）**：pre-line marker fragment 行 R1476 裸空塌缩（空 pre-line
  行 0 高，latent，corpus 无案）；white-space-007（nowrap span 空格域）、
  white-space-pre-001（img 网格 oracle + 页面级偏移）未动另案。
- **维持**：bench-gate 静窗复测（跨 R5317/R5318）、非 img inline-block
  content/border 口径分裂、content-none-select-1 负载脆弱族、c44-003 0.47% 残差、
  va 族余 122/baseline-003/intrinsic-size-009/R5314/R5315；zero-engine js_dom_bridge
  fetch 8 案 + details 闪断案。
- **下一步 R5323**：①813F 账本续扫择轻（0.00% 段 + 低 diff 段，避开 va 域与 OOF
  锚域）②CSS2/text 域邻案（white-space-007、pre-001）③css-multicol 103F /
  css-transforms 67F / css-flexbox 52F 域内挑轻。
