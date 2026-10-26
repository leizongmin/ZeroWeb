# R5293 — auto 高 abs 盒内容派生高不回灌为子代百分比基 + table box 仅 bottom 侧 margin 隔离——child-bottom-margin-in-unresolvable-percentage-height ✓1.67→✓0.00

日期：2026-10-27。改动：`crates/taffy-local/src/compute/block.rs`（+101/−14）、`crates/taffy-local/src/style/block.rs`（+7）、`crates/taffy-local/src/style/mod.rs`（+16）、`crates/layout-engine/src/converter/mod.rs`（+3）。新 kill-switch 2（均 default-on）：`ZW_ABSPOS_AUTO_H_PCT_BASE` / `ZW_TABLE_MARGIN_END_ISOLATE`。前轮 session（R5292 后启动）于 429 限流中断，本轮接续收尾。

## 案面

`child-bottom-margin-in-unresolvable-percentage-height`（R5232 起留池案 1.67%）：红 100×100 相对定位容器内 abs 绿盒（`width:100%`，高 auto）> `inner`（`height:100%`）> `gc`（`height:20px; margin-bottom:80px`）。规范期望（CSS2 §10.5 × §10.6.4）：abs 盒高 auto → 高来自内容；inner 的 `height:100%` 以「依赖内容的包含块高」为基 → 百分比高按 §10.5 计算为 auto；gc 的 bottom margin 借 collapse-through（§8.3.1）溢入 abs 盒高（abs 立 BFC 拦停外溢）→ 绿盒高 = 20 + 80 = 100 盖满红盒。ZW 修前绿盒高 20，红盒下 80px 露红。

## 根因

taffy `perform_absolute_layout_on_absolute_children` 对样式/inset 均未定高的 abs 盒：先以 `SizingMode::ContentSize` 量出内容派生高 `final_size`，随后把该高**回灌**进 `perform_child_layout` 的 `known_dimensions.height` 重排——内容派生高被当成子代百分比高的解析基（定 CB 高），§10.5 的「包含块高依赖内容时百分比高 → auto」无从发生；inner 被锁死定高后自身 margin 塌穿断开，gc 的 80px bottom margin 丢失。

## 实现

1. **机制①：定高回灌重排闭臂**（compute/block.rs，`ZW_ABSPOS_AUTO_H_PCT_BASE`）。修正域三重门：kill-switch + `known_dimensions.height.is_none()`（样式/inset 均未定高）+ `content_block_only`（子树纯块级——margin 塌穿是块布局现象，flex 列主轴高语义（dynamic-change-simplified-layout 依赖定高回灌）与表子树不适用）。击发时重排高传 `None`，取回 `layout_output.size.height` 参与定位与 tree 记录，并补外层 min/max 钳（ContentSize 模式叶盒忽略自身 min/max，absolute-non-replaced-max-height-* 依赖）。
2. **机制②：table box 仅 bottom 侧 margin 隔离**（style/block.rs + style/mod.rs + converter/mod.rs + compute/block.rs，`ZW_TABLE_MARGIN_END_ISOLATE`）。新 `Style` 字段 `item_margins_end_isolated`（converter 对 `display:table` 置真）+ `BlockItemStyle::margins_end_isolated()` trait 缺省 false；`perform_final_layout_on_in_flow_children` 折叠旗标由三值细分——同 BFC 且 end-isolated 时 `Line { start: true, end: false }`（caption margin-top 照常塌出表顶，margin-bottom 留盒内，CSS2 §8.3.1 × §17.5.1）。动机：机制①单独生效时 margin-applies-to-015 翻红 1.54%——ZW 表几何（R3813）已把 caption margin-bottom 计入表高，abs 重排采信内容高后表 wrapper 的塌穿溢出即双计；taffy 层无表子树结构化识别，先以端侧隔离在下游兜住（FIXME(table-margin) 留根因挂账）。

回归测试 +2（`r5293_abs_auto_height_pct_base_tests`）：①目标案结构锚（绿盒高 100 = 20+80、inner 高 20、gc 贴顶）；②absolute-non-replaced-max-height-002 守护锚（重算高受 max-height 钳）。前轮遗留的 tests/mod.rs doc comment 错挂（r5211 注释被移挂到 r5293 mod）本轮修正归位。

## 交付

- 目标案 `child-bottom-margin-in-unresolvable-percentage-height` **✗1.67 → ✓0.00**。
- 全量 corpus **16590/15730P/860F** vs R5292 交付态 15729P/861F：失败集恰移出目标案一案、零新增翻转（归一化 diff 唯一差异即目标案；3 条 `.htm` 后缀差异为归一化格式噪声同案）。
- kill-switch 判别双向成立：`ZW_ABSPOS_AUTO_H_PCT_BASE=0` OFF 轮 861F，唯一翻转 = 目标案翻回红；`ZW_TABLE_MARGIN_END_ISOLATE=0` OFF 轮 861F，唯一翻转 = margin-applies-to-015 翻红（机制①单独已知代价，机制②消解后 ON 态 015 保持 ✓0.00）。

## 验证

- make reftest 716/716；layout-engine lib 2162P（+2）；taffy-local 自身 dev-dep 测试编译为预存不可用（vendored 缺 scripts/gentest，改动前如此），以 corpus A/B + engine lib 为门禁（R5281 判例）。
- fmt 0 diff；clippy `-p zero-layout-engine -p taffy` 告警集与干净 HEAD 逐条一致（110 条全预存，零新增）。
- product-smoke 6/6 struct PASS + welcome oracle 15.50% 持平；bench-gate 定向 zero-layout-engine GATE PASS 25/25（NEW=0）。
- make test：workspace / renderer bin / renderer lib / browser bin / gpu 探测 / quickjs 组合腿绿。**跨流预存红 2 处（均干净 HEAD standalone 复现、与本轮改动面无关，归因见下）** + 网络调度并行 flake 1 处（same_origin_image_burst peak=7 limit=6，standalone 0.61s 过，R5288/R5292 同族）。

## 跨流归因（非本轮改动面）

- `cdp_navigate_evaluate::default_async_probe_gives_up_frame_attached_on_busy_iframe_page`：干净 HEAD（695f120c3 与 0ab30804c）standalone 均 3/3 复现红（frameAttached 在 0.29s 内到达，负向钉翻红）；同文件其余 8 测试全过（含 kill-switch 对侧钉）。R5292 交付时 make test 该腿绿，窗口内合并的 PR #230（自动化挂起求值 dispatch 面，apps/renderer+browser）为最近嫌疑。归对端流域，本轮不越界修。
- `image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight`：干净 HEAD standalone 2/2 红（R5292 记录为「standalone 过」的已知 flake，现性质变化）。同属 renderer 域。

## 教训

- 排查「测试只在我机器上翻红」时，browser 多进程测试 spawn 的 `target/debug/zero-renderer` 独立于 `cargo test` 的重建链，陈旧二进制会污染归因——干净对照必须显式 `cargo build -p zero-renderer -p zero-compositor` 后再跑。
- `git stash` 后随手 `git stash drop` 曾误删本轮 WIP；`git fsck --unreachable` 按提交时间定位 `WIP on main` 悬空 commit 后 `git stash apply <sha>` + `git stash store` 完整找回。教训：stash 后要么立即 pop，要么 store 命名留档，不裸挂匿名 stash。
