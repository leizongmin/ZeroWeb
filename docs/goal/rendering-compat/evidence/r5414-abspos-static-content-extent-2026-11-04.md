# R5414 证据：块级 abspos 静态位锚取 laid content extent（float 子例外）——abspos-007 ✗1.16%→✓0.16%，corpus 真态 712F→711F FIXED=1 REGRESSED=∅

日期：2026-11-04。驱动案：`css/CSS2/positioning/abspos-007.xht`（`.inline`
span 内文本 + abspos 绿块 10em×1em + in-flow 红块同尺寸；ref = 文本下方纯绿块
无红）。R5227 域案（容器有 absolute block-level 子时记录 `oof_static_line_bottom`
行底标量做静态位锚）。

## 1. 归因（探针 BOX walk，probe_r5414a 用后即删）

修复前布局树（release 探针，abspos-007 形态）：

```
d4 y=0 h=16 ch=16 lb=Some(36.0)   ← 宿容器（taffy 内容底 16，行底标量 36）
  d5 y=36 abs=true blk=true       ← abspos 绿块锚在标量 36
  d5 y=16 blk=true                ← in-flow 红块落位 16（taffy 块游标）
```

**根因**：block-in-inline 单 IFC 模型给 OOF 块级子独立幻影行（不触发 §9.2.1.1
拆分），`oof_static_line_bottom` 标量 = max(行底) 含 **abspos 自身幻影行底**
（文本行底 16 + 幻影行底 20 → 36）。css-position-3 §static-position 真静态位 =
假想盒在流位置 = 前驱在流内容底 16＝后续 in-flow 兄弟 control 的 taffy 落位。
标量锚使绿块 y=36 与红块 y=16 错位 20px，红条整面露出（✗1.16%）。

## 2. 修复

`engine/abspos.rs` `fix_abspos_static_after_inline_content_walk`：锚改取
`box_node.content_height`（taffy laid content extent）——与 in-flow 兄弟落位同源
（taffy 块游标），自洽覆盖。**float 子例外**（邻域回归归因，见 §4）：float 不生成
行盒，标量只见行游标（精确锚），content extent 却含 float 底——容器含 float 子时
维持标量。kill-switch `ZW_R5227_CONTENT_EXTENT=0` 回退行底标量（OFF 精确回退由
corpus A/B 双窗覆盖）。

## 3. 探针 A/B（修复后）

| 形态 | 修复前 | ON（新） | OFF（kill-switch=0） | 应 |
|---|---|---|---|---|
| abspos-007 绿块 layout y | 36 | 16 ✓ | 36（精确回） | 16 |
| between-float-and-text 绿块 | — | 20（float 例外门后） | 20 | 20 |

## 4. 邻域回归归因（between-float-and-text 16.67% → 例外门 → 0.00%）

首版 content-extent 锚（无 float 例外）邻域 series（abspos 191 案）ON 出现
`between-float-and-text.html ✗16.67%`（OFF ✓0.00%，单案 3×3 双侧稳定）：该案
div 内 `&nbsp;` 行 + abspos 绿块 + float 红块 200×200——float 落位 = 行游标 20
（标量精确），content extent 含 float 底 220 → 绿块锚 220 与红块错位 200px。
**机制**：float 不入 IFC 行栈，标量天然免疫 float；taffy content extent 不免疫。
加 `has_float_child` 例外门后单案 ON ✓0.00% 恢复，abspos-007 维持 ✓0.16%。
邻域 series 复跑 ON 181✓/10✗ vs OFF 180✓/11✗——唯一差异 = abspos-007 FIXED，
其余 10 案名单与百分比逐字节同 OFF（零回归）。residual 0.16% = IFC 行度量 vs
taffy 块度量缝隙（release 探针行高 16 / debug 测试构建行高 24 既有面，自源
双边一致，低于 1% 阈值）。

## 5. 常驻测试

`crates/engine/tests/r5414_abspos_static_content_extent.rs` 3 针：

- `r5414_abspos_static_anchor_at_content_extent` ★load-bearing（abspos-007 案形，
  绿块 == 红块同位覆盖；y 断言走相对等 + 幻影行底上界 40——debug/release 文本行
  度量不同，绝对常量跨构建不稳）。
- `r5414_no_in_flow_predecessor_stays_at_content_top`（has_in_flow_pred 门边界：
  无前驱 abspos 维持内容顶 0，不被 content extent 锚误移）。
- `r5414_inline_level_abspos_keeps_line_top_anchor`（inline-level abspos 维持
  R5166 行顶锚——is_block_level 门不因本修复越界）。

3/3 绿；r5411/r5412/r5413 常驻 8/8 绿（共享 walk 无回归）。

## 6. corpus 验证（全量 A/B 双窗，16594 案，BASE=同二进制 kill-switch OFF）

- 单案 ON ✓0.16%（修复前 ✗1.16%）；OFF 单案精确回 ✗1.16%。
- 窗1 OFF 716F / ON 715F；窗2 OFF 714F / ON 714F。
- **双窗均 FIXED = {abspos-007}**；mover 候选 5 案（窗1：insert-inline-end-001、
  table-anon-151、insert-inline-middle-002、table-anon-149；窗2：
  intrinsic-percent-replaced-dynamic-008）单案隔离双侧 3×3 **全绿**＝在册抖动族
  （guidelines #35；新名 intrinsic-percent-replaced-dynamic-008 入册）。
  **真态 FIXED={abspos-007} REGRESSED=∅，corpus 真态 712F→711F**（窗口原始计数
  含在册抖动族波动）。

## 7. 门禁

fmt ✓（zero-layout-engine/zero-engine 双 crate clean）；guarded-clippy 全工作区
EXIT 0；zero-layout-engine 全腿绿（lib 2394 ✓）+ r5411-r5414 常驻 11/11 ✓；
make test——workspace 腿 3724P/5F（js_dom_bridge CSSOM 对端账在册 5 案同名单）
+ renderer-bin 绿 + renderer-lib 371P 绿 + browser-bin 742P/1F＝
`default_actions_work_without_javascript`（在册对端碰头账，跨轮仍未自愈）+
browser-tests 742P/1F（同上同名单）+ quickjs workspace 腿 775P/1F＝
`image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight`
（隔离 3/3 绿＝并发采样抖动，network_loading 同类新名）+ quickjs renderer-lib 腿
368P/1F＝`page_scripts::tests::mutation_arm_does_not_apply_across_document_reset_window`
（**隔离 3/3 败＝稳态；stash abspos.rs 后 HEAD 纯净态同败＝HEAD 位元无关**，
非本轮引入，归对端碰头账新红腿）；product-smoke 主腿 form-input-perf-gate PASS +
struct 全 PASS、legacy struct-check failures: 0；bench-gate 定向
zero-layout-engine 见 §7.1（主腿 GATE PASS，OFF 配对腿落对端流重载窗未成对）。

## 8. 挂账（本轮新增）

1. `page_scripts::mutation_arm_does_not_apply_across_document_reset_window`
   quickjs renderer-lib 红腿：HEAD 位元无关（stash 实证），apps/renderer 面非本
   流域工作面，归对端碰头账，跨轮观察是否自愈或需碰头。
2. quickjs `image_batch_e2e` 并发采样抖动：隔离 3/3 绿，维持观察（network_loading
   同类）。
3. `intrinsic-percent-replaced-dynamic-008` 入在册抖动族（guidelines #35 名单续）。
4. 对端账维持：default_actions（跨轮未自愈）、paste_and_go、gpu_compositor、
   network_loading、js_dom_bridge CSSOM 5。
5. bench-gate 本轮配对双腿见 §7.1（对端流重载干扰，静窗补跑记录）。

## 7.1 bench-gate 配对记录（对端流重载干扰定谳）

- **主腿 ON3（静窗 verified：peer 进程 0、loadavg 1min 0.58）GATE PASS 25
  （NEW=0）全部在预算内**——block 3.577ms / deep_nesting 252.3µs / flex
  3.250ms / grid 316.8µs / incremental 3.209ms / wide_tree 183.8µs，与 R5413
  静窗值（block 3.547/3.611ms、flex 3.322/3.226ms、incremental 3.302/3.455ms、
  wide_tree 184.5/193.0µs）同带（−0.5%~+12% 噪声带内）。
- OFF 配对腿 4 次尝试（off/off3/off4/off5）均 GATE FAIL（4/4/5/10 指标超预算），
  且失败指标**跨腿剧烈波动、与极性无关**：deep_nesting 252(ON3)→792(off3)→269
  (off4)→472µs(off5)；block 3.52→3.52→11.0→10.8ms（off3→off5 三连 OFF 腿内部
  3.4× 波动）——同极性自噪定谳＝机器抖动非代码差异。
- **结构性零开销证据**：锚逻辑仅在 `oof_static_line_bottom=Some` 容器执行
  （bench 树无 abspos 容器不置位），六宏基准（block/flex/grid/deep_nesting/
  incremental/wide_tree 均纯无-abspos 树）对该改动**执行路径零增量**；OFF 腿在
  该改动不可能影响的基准（如 deep_nesting）上超预算，即证失败属环境非极性。
- R5412 先例同形：「ON1 腿 4 宏指标 3-4× 超预算＝前段编译负载余波，静窗重跑
  GATE PASS」。本轮 ON3 静窗主腿 PASS 已满足 bench-gate 对本轮改动的通过条件；
  OFF 配对双腿留挂账观察（§8.5），后续轮静窗补跑。

## 9. 工件

- 修复：`crates/layout-engine/src/engine/abspos.rs`（content extent 锚 + float
  子例外 + kill-switch）。
- 常驻测试：`crates/engine/tests/r5414_abspos_static_content_extent.rs`。
- 探针 probe_r5414a 用后即删；邻域 series /tmp/r5414_abspos_{series,off,on2}.log、
  /tmp/r5414_removeblock.log、/tmp/r5414_linebreaking.log、
  /tmp/r5414_staticinline_{on,off}.log；A/B 窗口 /tmp/r5414ab_w{1,2}_{o,n}.log；
  隔离记录 /tmp/r5414_bench_{on,off,on2}.log、/tmp/r5414_leg_*.log、
  /tmp/r5414_smoke_{main,legacy}.log、/tmp/r5414_maketest.log。
