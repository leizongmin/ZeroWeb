# R5172 证据：table-caption 宽度 shrink-to-fit（padding-top/right-applies-to-015 翻绿，corpus 净 −2）

日期：2026-10-18。基线：R5171 tip（fed827e60）。kill-switch：`ZW_CAPTION_SHRINK_TO_FIT=0` 回退 fill 行为（LazyLock 进程级快照，R3858 教训；判别经外部 env corpus OFF 轮验证）。

## 域核

R5172 轮指令首选 ①vertical 域表 rtl 列序（R5170 FIXME）——勘察**无案驱动**：fail 清单 1,106 案中 vertical writing-mode × table × rtl 组合零命中（css-tables 目录不在 imported-tests.txt；css-writing-modes-3 目录零表案；css-tables/collapsed-border-vertical-rtl-overflow 焦点是 collapsed-border overflow 非列序且不在导入清单）。R5170 FIXME 维持挂账，转 ②0.5-2% 带续扫择轻。

0.5-2% 带 455 案域分布：multicol 66（RFC 用户门挂账域）、transforms 48、sizing 23、flexbox 22、filter-effects 20、cascade 16、tables 13……2.08% 恒值簇 180 案经历史轮（R5151/R5152/R5153/R5165）确认是「同量级非同因」（480000px² 页面差 100×100 块 = 2.0833%），不作单因修。择轻落 tables 邻域 margin-padding-clear 的 **padding-top-applies-to-015（1.22%）/ padding-right-applies-to-015（1.67%）** 对案。

**域核定谳**（REFTEST_DUMP PNG 取证）：案为 `display: table-caption` 的 padding 应用测试——ref 侧 caption 宽 shrink-to-fit（CSS2 §17.4.1），ZW 侧 caption 被拉满视口宽（blue/orange border 线错位；top 案 blue 线 800px 应 200px、right 案 orange border 落视口右缘应 x=260）。根因：converter 把 `TableCaption` 映射为 taffy `Display::Block`（taffy 无原生 table 支持）→ caption 盒 fill 满包含块宽，shrink-to-fit 语义缺失。

## 实现（engine.rs + engine/sizing.rs，+约 90 行）

1. **`apply_caption_shrink_to_fit_width` pass**（engine/sizing.rs，R3929 abspos shrink 同骨架）：taffy 布局后遍历，对 `TableCaption` + `width: Auto` + 非 float/abspos/contain:size + 水平书写模式 + **有子盒**的 caption 补测收缩——`target_bw = min(block_max_content_width(b), cb_width)`（block_max_content_width 返回 border-box），只缩不扩（`b.width ≤ target_bw + 0.5` 跳过），按 taffy `box_sizing` 口径写回 `style.size.width` + `mark_dirty`。
2. **无子盒空 caption 维持 fill**（关键门）：margin-applies-to-015（R3813 driving case）chromium ref 实证空 caption fill 表可用宽（320−margin 100 = 220，完整 border 方框），收缩语义会致 border 缺边翻红——加 `b.children.is_empty()` 排除；纯 inline 内容 caption（无子盒）同样维持 fill（css-tables-3 caption intrinsic 尺寸 FIXME 案驱动）。
3. **预扫描位标**（R4215/R4216 模式）：`SizingTargets.caption` 位标（styles 扫描 `DisplayValue::TableCaption`），无 caption 页面整 pass 跳过；早退条件同步加 `&& t.caption`。
4. **re-run 组**：与 abspos shrink 同一 re-run 组（changed 汇合后重跑 taffy）。
5. min-content 下限钳（§10.3.5 preferred_min）未实现（FIXME 案驱动补）；CB 可用宽按父 border-box 近似（含 frame 表盒偏大 FIXME）。

## 验证

### 回归测试（crates/layout-engine/src/engine/tests/r5172_caption_shrink_to_fit.rs，3 案）

- `r5172_caption_shrinks_to_content_with_padding_right`：right 案形态——caption border-box = 270（div 210 + padding 50 + border 10）。
- `r5172_caption_shrinks_to_content_width_only`：top 案形态——caption 宽 = 200（水平无 frame）。
- `r5172_empty_caption_keeps_fill`：空 caption 对照——fill 220（320 − margin 100），锁定不收缩。

### corpus ON/OFF A/B（16,590 案，同 binary）

| 轮 | fail | 备注 |
|---|---|---|
| ON（默认） | **1,104** | **终态**（基线 R5171-ON 1,106，净 −2） |
| OFF（`ZW_CAPTION_SHRINK_TO_FIT=0`） | 1,106 | 恰等基线；两驱动案回红 |

- ON ↔ 基线 fail 清单 diff（前缀归一化）：**翻绿恰 2**（padding-top/right-applies-to-015）、**翻红 0**。
- applies-to-015 族复核：margin-applies-to-015（R3813 driving）首轮无「有子盒」门时新红 1.67%，加门后回绿基线同值——门因果闭环。

### 门禁

- fmt / clippy（-p zero-layout-engine --all-targets -D warnings）：清。
- zero-layout-engine lib：**1,991 绿**（r5172 3 案新入库）。
- `make reftest`：**712/712**。
- `make test`：workspace 全腿绿。腿 1（zero-engine lib 3,114 测试）flaky 1 案 `js_dom_bridge::tests::test_t8g_npo_iframe_scan_no_collection_leak`（3,113P+1F，part42.rs:304）——隔离复跑 3/3 绿 + 整套复跑 3,114 全绿（R5151 先例同款并行连坐；与 ddcd5252e M2 切片 8 域相邻但零归因）；补腿 renderer 272P、browser xvfb 650P+1I、gpu 96P、browser --tests 腿预存环境红（R5168 挂账 5 同款缺 `ZW_GUARD_RUNNER_PREFIX`）`ZW_GUARD_RUNNER_PREFIX="xvfb-run -a"` 复跑全绿（6P/4P/9P/650P+1I/10P）；quickjs 三腿绿（768P+59I / 701P / 270P 等）。
- product-smoke：welcome oracle diff **17.07%** 同值（R5147 起 platform 恒值）+ 全 struct PASS + EXIT=0。
- bench-gate（定向 `ZERO_WEB_BENCH_CRATES=zero-layout-engine`）：ON 轮 **4 FAIL（NEW=0）**；kill-switch OFF（回退基线行为，同 binary 仅 env 差）同窗对照 **9 FAIL ⊇ ON 集且量级更大**——`block_layout_1000_elements` OFF 5.39ms / ON 3.16ms / 预算 2.94ms，`deep_nesting_50_levels`/`flex_layout_1000_elements`/`grid_layout_100_elements`/`incremental_layout`/`wide_tree_500_children` 5 指标仅 OFF FAIL，`page/medium/layout_ms`（152 vs 155 同超）双态同超——基线行为反更慢 = 环境负载噪声族（R5149-R5171 判例延续），不 relax；ON FAIL 集为 OFF 真子集，修改面零 bench 归因。

## 挂账与下一步

1. vertical 域表 rtl 列序（R5170 FIXME 延续）：corpus 无案驱动，导入清单出现该域案时案驱动。
2. 空 caption / 纯 inline 内容 caption 的 shrink 语义（css-tables-3 caption intrinsic 尺寸）：现维持 fill（chromium 实证对齐），案驱动时补。
3. min-content 下限钳（§10.3.5 preferred_min）：caption 内容含不可断长词 + 显式窄 CB 案驱动。
4. R5171 挂账延续：inline 扩张轴内容区锚定（须与 fragment/selection 站同切片）、跨 white-space 折叠全局对齐、纯文本 run 匿名 cell 合成、vertical 表匿名 cell 域。
5. (A) row-2 / multicol-2 / 动画 RFC（用户门）。
