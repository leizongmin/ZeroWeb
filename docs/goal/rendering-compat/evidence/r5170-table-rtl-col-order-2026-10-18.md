# R5170 证据：rtl 表列序镜像（direction-applies-to-013/014 翻绿，corpus 净 −2）

日期：2026-10-18。基线：R5169 tip（e001bd5ca）。kill-switch：`ZW_TABLE_RTL_COL_ORDER=0` 回退 ltr 列序（table.rs `TABLE_RTL_COL_ORDER_ON` LazyLock 构造期单读，R3858 教训；判别经外部 env corpus OFF 轮验证）。

## 域核

direction-applies-to-013/014（各 1.04%，恰 50px Ahem 方块错一列位）：`direction:rtl` 的 display:table（013）/ inline-table（014，共享 `position_cells`）固定布局 100px、2×2 cell——首 cell "X" 应落**右**列（css-writing-modes-3 §7.1：direction 决定表列序，rtl 首列在右），旧实现恒按 ltr 左起排。rtl 的消费先例 engine.rs:1855（float 物理侧），表列序无消费。

## 实现（table.rs，+51 行）

1. **rtl 判定**（`position_cells` 头）：`table_rtl_col_order_enabled() && !table_box.writing_mode.is_vertical_block_flow() && 表盒样式 Direction::Rtl`——vertical 域 rtl 列序 FIXME 挂账（transpose 路径正交）。
2. **镜像基准**：`grid_width` = Σ 非折叠列宽 + (非折叠列数−1)×`spacing_x`（与 `cell_x` 逐 cell 推进口径一致，不含周界）；`cell_x_start` = 本行 `cell_x` 初值（direct-cell 匿名行 = `perimeter_x`，正常行 = 0.0）。
3. **落位镜像**：`cell_x` 仍按 ltr 推进（R5168 占位 entry / visibility:collapse 折叠跳过 / colspan 累加逻辑全部不动），仅赋值处把 `[cell_x, cell_x+cell_width]` 在 `[cell_x_start, cell_x_start+grid_width]` 内左右翻转。ltr 路径恒等式还原，非 rtl 零行为差。

## 验证

### 回归测试（engine/tests/r5170_table_rtl_col_order.rs，3 案）

- `r5170_rtl_table_first_cell_lands_right_column`：013 形态（100px 固定布局、两列各 50、border-spacing:0）两行首 cell x=50 / 次 cell x=0（旧缺陷首 cell x=0）。
- `r5170_ltr_table_first_cell_lands_left_column`：ltr 对照——镜像恒等式还原（首 cell x=0 / 次 x=50）。
- `r5170_rtl_inline_table_first_cell_lands_right_column`：014 形态 inline-table 同走镜像。
- kill-switch 判别经 corpus OFF 轮外部 env 承担（进程内 set_var 不可靠）。

### corpus ON/OFF A/B（16,590 案，同 binary）

| 轮 | fail | 备注 |
|---|---|---|
| ON（默认） | **1,108** | **终态**（基线 R5169-ON 1,110，净 −2） |
| OFF（`ZW_TABLE_RTL_COL_ORDER=0`） | 1,110 | 恰等于 R5169 基线；013/014 回红 1.04% |

- ON ↔ R5169 基线 fail 清单 diff（前缀归一化）：**翻绿恰 2**（direction-applies-to-013/014）、**翻红 0**。
- ON ↔ OFF 同二进制 diff：差异恰两目标案（OFF-only 翻绿 0、ON 独红 0——语义为 OFF 红两案、ON 全绿），kill-switch 判别成立。
- bidi-text 域红色集合（bidi-004..010b、bidi-breaking、first-line-bidi 等 14 案）ON/OFF 双态同值——bidi 文本重排域预存红，与本改动零归因。

### 门禁

- fmt / clippy（-p zero-layout-engine --all-targets -D warnings）：清。
- layout-engine lib：**1,986 全绿**（r5170 3 案新入库）。
- `make reftest`：**712/712**（100%）。
- `make test`：workspace 腿全绿；唯一中断 = 预存环境红 `zero-browser --tests` 腿（paste_and_go_blurs_and_clears_preedit + window_surface_present_smoke，无头 X display 缺失，R5168 挂账 5 同款——该 recipe 行缺 `ZW_GUARD_RUNNER_PREFIX`）——**xvfb-run 前缀复跑全绿**（650P+9P+10P）。余腿逐条补跑：gpu adapter 96P+1P、quickjs crates 1,896P、quickjs renderer lib 270P（bin 腿 0P 为 slice35 后用例归属 lib target 的现状，lib 腿已覆盖）。
- product-smoke：welcome oracle diff **17.07%**（≤20，R5166-R5169 同值）+ 全 struct PASS。
- bench-gate：**29 FAIL / NEW=0**（113 指标）——FAIL 集横跨零变更 crate（wasm-sandbox ×5、storage ×2、webview ×9、browser-shell、host-runtime、script-sandbox），本改动唯一触碰 crate（layout-engine）4 项微基准 FAIL 与 R5167 同型（block_layout 6.77ms 超预算 2.94ms）；表列序镜像无 wasm/storage 调用路径，广度即环境负载签名，判 R5149-R5169 噪声族延续，不 relax，空闲复跑挂账。

## 挂账与下一步

1. vertical 域表 rtl 列序（transpose 路径镜像语义，FIXME 留 table.rs 判定处）。
2. R5169 挂账延续：跨 white-space 折叠全局对齐、纯文本 run 匿名 cell 合成、boxless cell 网格定位边缘、vertical 表匿名 cell 域、单行孤儿 row retrofit、纯匿名 cell 行行高。
3. `zero-browser --tests` 腿缺 `ZW_GUARD_RUNNER_PREFIX`（R5168 挂账 5 延续，测试基建另案）。
