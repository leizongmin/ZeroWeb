# R5409 证据 — visibility:collapse 多线 strut 迁移（flexbox-collapsed-item-horiz-002 ✗1.59%→✓0.00%，corpus 真态 718F→717F FIXED=1 REGRESSED=∅）

日期：2026-11-04。承接 R5408 下一步②（flexbox collapsed-item 算法切片，两次顺延后本轮驱动）。修复代码与测试成形于上轮 session 429 中断时的工作树在途稿，本轮完成调试残留清理（空 `eprintln!`）、fmt、家族/全量 A/B、kill-switch 敏感性、全门禁与落账。

## 1. 归因

- 驱动案 `flexbox-collapsed-item-horiz-002`（账面 ✗1.59%）：wrap flex 容器内 `visibility:collapse` item。css-flexbox-1 §4 Collapsed Items 规范语义——collapsed item 先按可见参与断线（strut 所在线的交叉尺寸记为该 item 的 strut），随后以零主尺寸重跑断线，item 可能迁入其他线，strut 随之迁移并贡献给落线（「carries the strut between lines」）。
- ZW 现状：converter `ZW_VC_NONFLEX_STRUT`（R1834 判例，default-on）对非-flexible collapsed（flex-grow≤0）保留 flex-basis 作 strut 保宽——单线容器（horiz-001）语义正确（✓0.51% 判例冻结），但 wrap 容器下保留 basis 使 collapsed item 以原主尺寸占位，阻断「零主尺寸迁移 + strut 贡献落线」（002 Row2/Row3 形态整页错位 1.59%）。
- 真值口径（chromium oracle 2026-11-04 探针，上轮在途稿定谳）：chromium **不实现** collapsed strut 迁移——002 case 页与 ref 页在 chromium 内即不一致（上游双挂）；本仓 corpus 为 self-source（ZW case vs ZW ref），002 修复目标 = ref 页几何（编码规范语义），002 ref 逐行解码与「首趟可见断线 + 零主尺寸重跑 + strut 迁移」两趟语义全吻合。

## 2. 修复（新增 engine/flex_collapse.rs 单 pass）

`apply_flex_collapsed_strut_migration`（taffy 单树两趟近似）：

1. **触发预扫描**（engine.rs styles 扫描位标同源）：`sizing_targets.flex_collapse_wrap` = 页面含 `visibility:collapse` style **且** 含非-nowrap flex/inline-flex style（任一命中即可，pass 内按容器精筛）；无 wrap×collapsed 页面整 pass 跳过。
2. **首趟线分组**：首趟布局即 converter strut-basis 形态（=规范首趟「按可见布局」），对每个水平书写 wrap flex 容器，按 taffy 输出子项 y 分组断线（0.5px 容差），每线交叉尺寸 = 线内最大 border-box 高（strut 源）。
3. **迁移 patch**：非-flexible collapsed 子项（`flex_grow<=0`，converter 同门）`flex_basis→0`（零主尺寸断线；已零免写）+ `size.height→strut`（仅 strut > 自身首趟高 + 0.5 时写，相等保留 CSS height 语义），`set_style + mark_dirty` 入既有 R1018 re-run 组重跑。

限定与门控：

- kill-switch `ZW_FLEX_COLLAPSE_STRUT_MIGRATE=0`（default-on，OnceLock 进程级快照，guideline #9）。
- converter 前提 `ZW_VC_NONFLEX_STRUT=0` 时本臂自动惰性（basis 已全零，「当前布局=规范首趟」前提不成立；趟级单读）。
- 水平书写限定（vertical 主/交叉轴互换另案，R4076 同门，guideline #29 未门控书写模式审计：vertical 无臂、converter R1834 语义原样保留）；仅 wrap 容器（单线 horiz-001 判例语义不受扰动）；仅非-flexible collapsed（flexible collapsed 首趟 flexing 语义 FIXME 挂账，converter 已对其置零 basis）；strut 取线内最大高不含 margin（corpus 案 margin 全 0，边缘另案）。

## 3. 常驻测试

`engine/tests/r5409_flex_collapse_strut_tests.rs` 3 针（OFF 轮不设进程内单测——R3858 口径，由 corpus 002 ✗1.59% 精确回退覆盖）：

1. `r5409_collapsed_migrates_into_first_line` ★load-bearing（002 Row2-C1 形态）：迁移后 line1 = collapsed+full1（交叉 25）、line2 y=25、容器高 45、零主尺寸打包 full1 x=0、collapsed 主尺寸 0。
2. `r5409_strut_carries_first_pass_line_cross_size` ★load-bearing（002 Row3-C1 形态）：strut 40 ≠ 自身高 25（首趟与 tall 同线取线交叉 40），迁移后 line1 交叉 40、tall 落 line2 y=40、容器高 80。
3. `r5409_nowrap_container_keeps_strut_basis` 单线守卫：无 wrap 本臂不触发，R1834 basis-keep 现状冻结（collapsed 保主位 20、short x=20 打包其后）。001 ref 几何（主 0 + strut）为另案候选面。

## 4. 验证

- 新针 3/3；layout-engine lib **2394 绿**（2391+3），2 ignored。
- kill-switch OFF 敏感性：002 精确回 ✗1.59%（与账面 BASE 一字不差）；单线 001 双态 ✓0.51% 不变；003 双态 ✓（0.08%/0.12% 亚像素噪声，guideline #35）。
- 家族 A/B：4 案 ON 4/4（002 **✓0.00%**、baseline-001 0.06%、001 0.51%、003 0.08%）。
- corpus 全量 A/B（16594 案双跑窗口，BASE = 同二进制 kill-switch OFF）：BASE **722F** / FIX **723F** 账面；mover 6 案——FIXED 候选 {insert-middle-001、table-anon-130、002}、REGRESSED 候选 {table-anon-149/154、cascade-highlight-002、selection-background-color-revert}——隔离双侧 3×3 **全绿洗清**（在册抖动族：table-anon + insert-middle + highlight 伪元素，guideline #35），真态 **FIXED={flexbox-collapsed-item-horiz-002} REGRESSED=∅，真态 718F→717F**。
- 本地集 **725/725**；fmt ✓；guarded-clippy 全工作区 EXIT 0；product-smoke 主 + legacy 双腿 EXIT 0（struct PASS / struct-check failures: 0）；bench-gate 定向 zero-layout-engine **GATE PASS 25 指标（NEW=0）**；make test 唯一红腿 = js_dom_bridge CSSOM 对端账 5 案（与 R5404–R5408 同名单）其余 65 腿全绿。

## 5. 挂账与下一步

新增挂账：

1. 单线容器零主尺寸候选面——001 ref 几何 = 主 0 + strut 40，现 ZW basis-keep ✗1.42% 宽限过案；chromium oracle case/ref 双侧不一致（上游双挂），self-source 语义修复待独立驱动案定谳。
2. flexible collapsed strut 语义（flex-grow>0 首趟 flexing 域，FIXME 在码）。
3. vertical 书写模式 collapsed 迁移缺口（水平限定臂的对应缺口，guideline #29）。
4. strut margin 计入（corpus 无案驱动）。

维持：R5408 残留四案分层（011/030 竖排域、dynamic-009/010 动态改高）、definite CSS 宽 + max-height 钳域回传缺口（无驱动案）、R5406 家族初判六面、R5405 十面及以前挂账全维持；对端账（js_dom_bridge CSSOM 5 案/bookmark_context_menu/script_fetcher s249/s250/image_batch_e2e/mutation_arm/default_actions/paste_and_go）维持。

**下一步 R5410**：①misc 账本续扫择轻（避开在册深域/对端面/残留四案）②flexbox 残余域续评（writing-mode 竖排 14 案深域在册，baseline-synthesis 5 案侦察候选）。
