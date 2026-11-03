# R5413 证据：scrollbar 槽位恒 0 + BFC containment 仅 auto 高——flexbox-overflow-horiz-001 ✗6.19%→✓0.00% ＋ contain-size-block-002 ✗12.60%→✓0.23%

日期：2026-11-04。驱动案：`css/css-flexbox/flexbox-overflow-horiz-001.html`
（4×70×70 float flex 容器：无声明/hidden/scroll/auto，bigItem flex:3 h:200px +
smallItem flex:1 mb:10px；ref 用 float/inline-block 75%/25% + calc(100%−10px)
模拟 scrollbar-less 几何，四容器几何必须一致）。series 基线仅 001 ✗6.19%，
002-005 ✓。

## 1. 归因（双断点，独立缺陷）

**探针 A**（probe_r5413a，test 页 BOX walk，修复前）：

```
d2 x=0   70×70 ox=Visible kids 53×200 + 17×60  ✓（ch 均未内缩）
d2 x=75  70×70 ox=Hidden  kids 53×200 + 17×60  ✓
d2 x=150 70×70 ox=Scroll  kids 41.2×200 + 13.8×45  ✗（15px 双轴死槽位）
d2 x=225 70×70 ox=Scroll  kids 53×200 + 17×60  ✓（声明 auto，R4265 门抑制）
```

**缺陷 ①：taffy scrollbar_width 槽位**（converter/mod.rs scrollbar_width 接线）。
R4265 门仅对「两轴均非声明 scroll」抑制槽位；声明 `overflow:scroll` 保留 15px，
taffy 对每个 Scroll 轴内缩（taffy-local compute/mod.rs:140、block.rs:398、grid
mod.rs:86 同源）→ scroll 容器内容区 55×55（items 41.2/13.8、teal 45）。ZW 不绘制
classic scrollbar（paint 侧仅右缘 overlay 指示器，消费零布局空间）→ 15px 预留是
无绘制的死槽位，ref（chromium overlay-scrollbar 口径）不预留。R5412 归档「ZW 无
classic scrollbar geometry」的 grep 只覆盖 scrollbar-gutter 属性代码，漏了本布局
槽位——归档前提在本轮修订。

**探针 B**（probe_r5413b，ref 页 BOX walk，缺陷 ① 修复后）：

```
d2 x=0  70×200  ✗（显式 height:70 被 float 子撑到 200）
d2 x=75 70×200  ✗
```

**缺陷 ②：BFC float containment 补扩无显式高守卫**（float_positioning.rs step 5
`use_bfc_float_containment` 臂）。CSS2 §10.6.7「高度含最低 float 外底边」公式限定
auto 高 BFC root；显式高容器 used height = 显式值（§10.5），float 仅溢出。同函数
非 BFC 收缩臂已有 R1277 ④ `ZW_FLOAT_RESPECT_HEIGHT` + declared_height_auto 守卫，
BFC 补扩臂对称缺失 → ref 页容器 70→200、页面整体下移 130px 与 test 页错位。
**案差主项是缺陷 ②**（仅修 ① 后 001 仍 ✗6.15%），① 修复后像素差几乎不动
（6.19%→6.15%）即证。

## 2. 修复

1. **converter/mod.rs**：`scrollbar_width` 恒 0（`scrollbar_layout_slot_suppressed()`
   OnceLock 门，kill-switch `ZW_SCROLLBAR_SLOT_SUPPRESS=0` 回退 R4265 口径：
   auto 15/thin 8、任一轴声明 scroll 保留）。R4265 的 auto 抑制判例对齐到全部声明值。
2. **float_positioning.rs**：`use_bfc_float_containment` 补扩臂加
   `contain_auto_only && !box_node.declared_height_auto` 跳过（kill-switch
   `ZW_BFC_CONTAIN_AUTO_ONLY=0` 回退无条件补扩）。auto 高 BFC root 补扩语义不变。

## 3. 探针 A/B（修复后）

| 形态 | 修复前 | 修复后 | 应 |
|---|---|---|---|
| test 页 scroll 容器 kids | 41.2×200 + 13.8×45 | 53×200 + 17×60 ✓ | 与 visible/hidden/auto 一致 |
| kill-switch OFF（①） | — | 41.2×200 + 13.8×45（精确回） | R4265 口径 |
| ref 页 float 容器 h | 200 | 70 ✓（两开关各验） | 70 |
| kill-switch OFF（②） | — | 200（精确回） | 无条件补扩口径 |

## 4. 常驻测试

- `crates/engine/tests/r5413_scrollbar_layout_slot.rs` 2 针：
  `r5413_scroll_container_keeps_scrollbarless_geometry` ★load-bearing（scroll 容器
  teal (61,8,17,60)，像素吸附）；`r5413_overflow_auto_geometry_unchanged`（R4265
  端点守卫）。2/2 绿。
- `crates/engine/tests/r5413_bfc_contain_auto_only.rs` 2 针：
  `r5413_definite_height_float_container_not_grown` ★load-bearing（显式高 float
  容器紫底 h=70）；`r5413_auto_height_float_container_still_contains_floats`
  （auto 高维持补扩 h=200）。2/2 绿。
- r5411/r5412 常驻 4/4 绿（修复面共享 walk，无回归）。

## 5. corpus 验证（全量 A/B 双窗，16594 案，BASE=同二进制双 kill-switch OFF）

- 单案 ON ✓0.00%（修复前 ✗6.19%）；series flexbox-overflow-horiz-00 全组 5/5 ✓。
- 窗1 OFF 719F / ON 716F：FIXED 候选 {001, contain-size-block-002,
  table-anon-130/146/148}、REGRESSED 候选 {insert-inline-middle-003,
  table-anon-154}。
- 窗2 OFF 718F / ON 720F：FIXED 候选 {001, contain-size-block-002,
  table-anon-149}、REGRESSED 候选 {border-collapse-dynamic-column-002,
  table-anon-130/148/151/154}。
- 交叉定谳：**双窗均 FIXED = {flexbox-overflow-horiz-001, contain-size-block-002}**。
  contain-size-block-002 单案 3×3：ON ✓3/3 / OFF ✗3/3＝稳定真修复；单开关归因
  `ZW_BFC_CONTAIN_AUTO_ONLY`（bfc-only-OFF ✓0.23%、slot-only-OFF ✗12.60%）。
  REGRESSED 候选 7 案（table-anon-154 双窗均现）单案隔离 ON/OFF 双侧 3×3 **全绿**
  ＝在册抖动族（guidelines #35，含新名 border-collapse-dynamic-column-002）。
  **真态 FIXED={001（双缺陷复合）, contain-size-block-002} REGRESSED=∅，
  corpus 真态 714F→712F**（窗口原始计数 OFF 719/718、ON 716/720 含在册抖动族波动）。

## 6. 门禁

fmt ✓；guarded-clippy 全工作区 EXIT 0（quickjs 组合 clippy 同绿）；zero-layout-engine
全腿绿（lib 2394 ✓ 9 腿 0 failed）+ r5411/r5412/r5413 常驻 8/8 ✓；make test——
workspace 腿 3724P/5F（js_dom_bridge CSSOM 对端账在册 5 案同名单）+ renderer-bin
绿 + browser-bin 2F＝`default_actions_work_without_javascript`（跨轮 drift 红腿
R5412 stash 实证位元无关，本轮起仍未自愈，维持对端碰头账）+ browser-tests 3F＝
default_actions（同上）＋`paste_and_go_blurs_and_clears_preedit`（**HEAD 位元无关
stash 实证**：stash 布局三文件后 HEAD 同败 1/1＝环境/预输入态漂移，归对端碰头账）
＋`gpu_compositor_path_dispatches_input_events_to_form_controls`（单测隔离 3 跑
2 绿 1 败＝环境抖动非语义）＋quickjs workspace 腿 775P/1F＝
`network_loading::same_origin_image_burst_reaches_per_origin_concurrency_limit`
（peak=7>6；单测隔离 3 跑全绿＝并发峰值采样抖动）；product-smoke 主腿
form-input-perf-gate PASS + struct 全 PASS、legacy struct-check failures: 0；
bench-gate 定向 zero-layout-engine：ON1 GATE PASS 25（NEW=0）＋同窗 kill-switch
OFF 配对腿 GATE PASS 25（NEW=0）——ON/OFF 静窗值 block 3.547/3.611ms、flex
3.322/3.226ms、incremental 3.302/3.455ms、wide_tree 193.0/184.5µs（ON −1.8%~+4.6%
噪声带内，R5410/R5411 配对口径）。

## 7. 挂账（本轮新增）

1. **R5412 scrollbar 族归档前提修订**：content-height-with-scrollbars /
   cross-axis-scrollbar 两案本轮起在 scrollbar-less 几何下仍 ✗（双窗双态均在
   fail 名单，属稳态非抖动）——R5412 归档判据「ZW 无 classic scrollbar geometry」
   已失效（槽位恒 0 后成立的是 scrollbar-less 几何），残余失败需重归因，留后续
   轮驱动。
2. **`paste_and_go_blurs_and_clears_preedit` / `gpu_compositor_path_dispatches…`
   browser 腿红腿**：前者 HEAD 位元无关（碰头账），后者环境抖动；两者均非本轮
   引入，跨轮观察是否自愈或需对端碰头。
3. quickjs 腿 `network_loading::same_origin_image_burst` 并发峰值采样抖动：
   隔离 3/3 绿，维持观察（非语义）。

## 8. 工件

- 修复：`crates/layout-engine/src/converter/mod.rs`（scrollbar_width 恒 0 + 门）、
  `crates/layout-engine/src/float_positioning.rs`（BFC 补扩 auto-only 守卫）。
- 常驻测试：`crates/engine/tests/r5413_scrollbar_layout_slot.rs`、
  `crates/engine/tests/r5413_bfc_contain_auto_only.rs`。
- 探针 probe_r5413a/probe_r5413b 用后即删；A/B 窗口记录
  /tmp/r5413_ab_w{1,2}_{on,off}.log、隔离记录 /tmp/r5413_*。
