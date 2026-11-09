# R5465 证据：viewport-CB abspos inset calc(%) ICB 基准求值——驱动案 vrl-007 ✗4.61%→✓4.27%（真通过带内），ref 臂 table 几何归因三面（表布局多片项目，另账）

日期：2026-11-08。基线 HEAD：`68684de21`（R5464）。驱动案：`sizing-orthog-htb-in-vrl-007`（R5464 挂账「body fit-content 域」）。

## 1. 归因勘误：R5464 body-fit-content 假设证伪，实病 = abspos inset calc(%) 塌缩

- R5464 残余假设「body 在 vrl 容器下 auto 物理宽应 fit-content（oracle 400）而 ZW fill 600」**证伪**：单测 harness（LayoutEngine::new(800,600)）与 runner LAYOUT_DUMP 双面实测 body w=400 定值、margin-box 右锚 html content 右缘 800（body abs [300,700]）、cb/ob/sb/sa 位全数对齐 r5464 oracle 值——test 臂几何无病。html w=600 shrink-to-fit 右锚亦正确。
- **ref 臂两实病**（LAYOUT_DUMP + 单测探针矩阵定界）：
  - **病 A（本轮修复）**：`table { position:absolute; left: calc(100% - 136px - 400px - 136px) }` 解析为 0 → 表贴视口左缘 x=0（应 128）。普通 div 同病（`left: calc(100% - 100px)` → 0，应 700）；`left: 128px` 与 `right: 0px` 均正确——**仅含 % calc 塌缩**。
  - 病 B（另账）：ref 臂列宽 21.7/457.9/24.4（应 52/400/52），详见 §4。

## 2. 修复：resolve_abspos_calc_inset（kill-switch `ZW_ABSPOS_CALC_INSET`）

- **机制**：`resolve_abspos_real_length` 对 Calc 走 `resolve_length` → `resolve_length_inner` Calc 臂构造 CalcContext `parent_length: None`（无 % 基准）→ `eval_calc_with_context` 百分比解析 None → 整式 `unwrap_or(0.0)` 塌缩。计算值阶段转换器对含 % calc 保留信号（style-system converter `Calc if calc_contains_percentage` 臂「由布局引擎处理」）——布局侧消费点缺 % 基准即为本病。
- **修复**：`abspos.rs` 新增 `resolve_abspos_calc_inset(value, font_size, basis, vw, vh)`——Calc 值以 `parent_length = basis`（水平 inset → CB 宽、垂直 → CB 高；CSS2 §10.1 viewport-CB → ICB 对应轴）构造 CalcContext 走 `eval_calc_with_context` 既有求值（复用 css-values 管线，零新求值器）；非 Calc 与 kill-switch=0 回退 `resolve_abspos_real_length` 旧口径。kill-switch 读 `LazyLock` 进程级单次（递归 pass 热路径不逐节点读 env，R5464 同款）。
- **落点（11.5 pass `adjust_absolute_pct_to_viewport` + R5376/R5377 方程 helper 的 right 解析共 14 点）**：auto-stretch 双 inset 臂、left/top Px 臂、right/bottom 反解臂、max-width 钳制臂、R5376/R5377 `right_px`。**margin 解析点不动**（% margin 相对 CB 既有口径维持，calc-% margin 与 % margin 同域另账 FIXME）；**nested-CB/fixed-stretch pass 的同名塌缩不动**（R4122 containment 域与 fixed stretch 的 calc-% inset 同病，见 §4 挂账——本轮只收 viewport-CB 域，blast radius 最小化）。
- 已知非域：abspos `width: calc(<percentage> ± …)` 走 converter/taffy 路径非本 pass（未探，挂账核实）。

## 3. 验证

- 常驻钉 6＋kill-switch 锚 1：`r5465_abspos_calc_inset.rs`（calc left 700/表 128/垂直 top 310 基准/right 反解 150/max-width 钳 410/Px 旧口径 120 守卫）＋`..._kill_switch.rs`（=0 回旧塌缩 −8，独立进程 R3858 教训）。
- 驱动案：vrl-007 **✗4.61% → ✓4.27%**（进 pass 带但 >strict，残余 = 病 B）；solo×3 稳定 ✓。
- 内测 reftest **725/725**；fmt＋clippy（-p zero-layout-engine --all-targets -D warnings）清零；layout-engine 全腿 **38 二进制 0F＋lib 2444P**。
- corpus kill-switch 双窗（16594，同法 R5464）：w1(ON) **691F** / w2(OFF) **690F**——互翻 3 案（tao-127 / insert-block-003 / border-collapse-dynamic-column-002）**双态 solo×3 全绿** = 全量窗噪声指纹家族（与 R5464 观察同族）；真修复 0、真回归 0，净零扰动。
- product-smoke 全腿 PASS（welcome/morning/article/320 结构门＋form-input-perf-gate）；bench-gate 定向 zero-layout-engine **GATE PASS 25 指标（NEW=0）**。

## 4. 病 B 归因（ref 臂列宽 21.7/457.9/24.4 ≠ 52/400/52——下一轮驱动，三片式表布局项目）

chromium oracle headless 探针（.xht XML + title 注入；`/tmp/r5465-oracle/` 探针组）+ ZW 单测探针矩阵定界：

- **oracle 事实**（ref 形态，htb p 变体）：侧列 = **max(specified 52, min-content 57.75) = 57.75**（specified 52 < 词 min-content → min-content 胜）；data 列 = available − Σ 侧列 = **388.5**（吸收余量，max-content 1995.4 不封顶不参与）；侧列仅指定或 data 仅指定均得同一结果。`display:block` td 与实 cell 列宽**逐位一致**（匿名 cell fixup 等价）。全 auto 列 = 64.31/373.11/66.58（min-content + 按 (max−min) 分配余量带）。真实 ref（p vertical-rl，行宽 ~20 < 52）→ 侧列 = 52、data = 400 列和恰等表 content 504。
- **ZW 缺口三面**：
  1. **R364b floor 语义**：`cell_used_width` 显式宽臂 `base.max(floor_intrinsic)` 的 floor 语境（`for_explicit_floor=true`）95% 臂失败后（块级子被拉伸至 cell 宽不满足 <95%）回落 `box_content_max_width` = **max-content** 而非函数注释自称的 min-content——内容 max-content 越过指定宽（htb p 侧列 124.8 > 52；对照探针 8 无块级子则 52/400/52 逐位正确）。R4028 停用的 DOM 臂防 c5501 撑破，fallback 却同病未防。
  2. **正交子轴向测宽**：htb cell 内 vertical-rl p 的固有宽测得 124.8（文本 run advance）而非块轴行宽 ~20（探针 7c 单列表 auto 宽 124.8；chromium 应 ~20）。波及 floor/R4028/扩展全域测量。
  3. **R4298 AUTO_SHRINK 无 floor 钳制**：总宽 > available 纯比例压缩，specified/floor 列不冻结（chrome 语义：min-content 冻结、slack 列吸收赤字；ZW 侧列 52→47.6 连带压缩）。
- 修至逐位（52/400/52）须三片齐动：floor 改 min-content（`block_min_content_width` R5036 已有）＋正交块轴测宽＋shrink 冻结臂——表布局全域 blast radius，按 R5162/R5434 先例独立成轮（oracle 探针组与本节即开工归因）。

## 5. 挂账（未了结债）

- **病 B 三片式表布局项目**（§4，新主账，下一轮驱动案）。
- nested-CB/fixed-stretch pass 的 calc-% inset 同病塌缩（R4122 containment 域 771-840/598-608 等解析点）——viewport-CB 修复同构扩展，案驱动再收。
- abspos `width/height: calc(<percentage>…)` converter/taffy 路径未核实（本 pass Percent 臂之外）。
- calc-% margin 与 % margin 同域（本 pass margin 解析点维持塌缩 0 旧口径）。
- **维持**：R5461 abs AR 臂 frame 双计 FIXME；%/关键字 CB、显式宽正交子、inline 元素子、% margin resolve 各 FIXME；c5525-fltmrgn-000 ✗3.92%；R5447 collapse ½border；multicol-under-vertical-rl-scroll runner 域；vlr definite 容器宽重解兄弟列重叠 FIXME；㉗-㉘/R5432 销账面/deep intrinsic 48 变体矩阵/**待用户决策清单**①-⑤（无新增）；对端 js_dom_bridge 红腿观察；deep_nesting 基线漂移观察；既有账。

## 6. 取证命令存根

```bash
# 驱动案 solo（ON/OFF 态）
make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-007
ZW_ABSPOS_CALC_INSET=0 make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-007
# chromium oracle（.xht 须 xmlns + meta 自闭 + title 先置；--dump-dom 注入取值）
google-chrome --headless=new --dump-dom file:///tmp/r5465-oracle/ref-shape.xht | grep -oE '<title>[^<]*</title>'
# corpus 双窗（剥 ✗ % 后 comm 互翻；-13 = OFF 独红修复候选，-23 = ON 独红回归候选）
make reftest-upstream 2>&1 | grep '^  ✗' | sed 's/ ([0-9.]*%)$//' | sort > /tmp/w_on.txt
ZW_ABSPOS_CALC_INSET=0 make reftest-upstream 2>&1 | grep '^  ✗' | sed 's/ ([0-9.]*%)$//' | sort > /tmp/w_off.txt
comm -13 /tmp/w_on.txt /tmp/w_off.txt; comm -23 /tmp/w_on.txt /tmp/w_off.txt
```
