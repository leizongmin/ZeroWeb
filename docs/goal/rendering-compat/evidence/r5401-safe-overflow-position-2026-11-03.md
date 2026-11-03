# R5401：safe/unsafe overflow-position 前缀 + legacy -webkit-box align-self 抑制（2026-11-03）

## 目标案

WPT `css/css-flexbox/flexbox-safe-overflow-position-001.html`（✗1.40% → ✓0.00% 真通过-可信）。
谱系：css-align-3 §overflow-values——`<overflow-position>? <alignment-position>`，
`safe`/`unsafe` 前缀仅可与位置关键字组合（start/end/flex-start/flex-end/center）。
旧 `parse_alignment` 只认裸关键字，`align-content: safe flex-end` / `justify-content:
safe center` / `align-items: safe center` 全部整条拒收 → 回退初始值。

https://drafts.csswg.org/css-align-3/#overflow-values

## 归因

单缺陷：解析层整条拒收（无布局侧问题——taffy-local 既有 `SAFE_*` 常量与
`apply_alignment_fallback` / `resolve_self_alignment_safety` 消费面，converter 既往
从未传入）。

## 修复

- `parse_alignment`（css-parser `values/parse_misc.rs`）：`safe `/`unsafe ` 前缀分支——
  safe → `SafeStart/SafeEnd/SafeFlexStart/SafeFlexEnd/SafeCenter` 五变体；unsafe 与裸
  关键字同义（Unsafe 为默认，modifier 丢弃）；分布值/基线/初始值带前缀按消费方
  grammar 整条拒收（`safe stretch` / `safe space-between` / `safe baseline` 等非法）。
  keyword token 边界：`safestart`/`safe-start`/残段整条拒收；`safe center extra` 尾段
  非法；多空白合法。
- `AlignmentValue` 五个 Safe* 变体（css-parser `values/types.rs`）+ computed 序列化
  as specified（engine `js_dom_bridge/computed_style.rs` `alignment_str` /
  `align_content_to_css`）。
- converter（layout-engine `converter/mod.rs`）：`convert_alignment_to_align_items` /
  `_align_self` / `_justify_content` / `convert_align_content` 四入口把 Safe* 传入
  taffy `SAFE_*`（溢出时 taffy 回退 start 边；非溢出按原位置，不能预折算）。
- style-system：`AlignContentValue` 五变体 + `apply_advanced` 接受集（safe 保变体、
  unsafe 按 R1412 折算、`safe stretch` 拒）+ `is_place_longhand_value` align-content 臂
  同接受集。
- place-* 简写（style-system `shorthand/mod.rs` `expand_place`）：单值/双值两臂改
  `<align> <justify>?` 贪婪最长合法前缀切分（k 从 parts.len()-1 降序试探），无合法
  切分再试整值单值形——两侧长手值可多 token（css-align-3 `first/last baseline`、
  `safe/unsafe <position>`）；`place-content: safe flex-end center` 正确切分、单值
  `last baseline` / `safe center` 按规范展开双 longhand（旧实现对多 token 长手值整条
  拒收，属同域规范缺口，A/B 零扩散实证）。

## 第二缺陷（006 回归归因后补）：legacy -webkit-box 忽略 item align-self

### 现象

修复态家族 6/6 全绿后复核逐案 diff：006（`display:-webkit-box` +
`-webkit-box-align: end` + item `align-self: safe center`，assert「align-self:safe has
no effect on -webkit-box」）基线 0.00%（真通过-可信）→ 修复态 0.42%（近似通过），
双侧隔离 3×3 稳定非抖动。

### 根因

`display:-webkit-box` 水平语义在 cascade 被映射 flex（R3798）+ `-webkit-box-align`
映射 align-items；基线态 `align-self: safe center` 解析失败 → auto → 回落容器
align-items（与 chromium legacy box 语义巧合一致）；R5401 起该值可解析 → item 被改写
为 safe-center 溢出回退 start，脱离容器对齐——chromium legacy box 的 item 对齐只由
`-webkit-box-align` 控制，现代 `align-self` 不适用。

### 修复（胜者溯源标记，R3771 同款机制）

1. cascade（style-system `cascade.rs`）：R3798 水平映射分支记录映射虚拟声明 order 集；
   胜者解析后仅当 display 槽位胜者 order ∈ 映射集时置合成键
   `-webkit-box-flex-origin`——作者显式 `display:flex` 胜出（更高 order）时不置，
   元素是真 flex，现代 align-* 照常适用。
2. inheritance（style-system `inheritance.rs`）：合成键 → `ComputedStyle.
   legacy_webkit_box_flex = true`（主循环对该未知键 apply no-op，R3771 同构）。
3. layout（layout-engine `tree.rs` build_subtree）：父样式带标记 → 子项
   `taffy_style.align_self = None`（auto 回落容器 align-items）。converter 无父 display
   可判（R5154 同款父上下文改写先例），故在 build_subtree 处落点。
   kill-switch `ZW_WEBKIT_BOX_ALIGN_SELF_IGNORE=0`（default-on，OnceLock 单读）。
   FIXME：justify-self 同不适用于 legacy box，无驱动案暂不扩（同域语义）。

## 常驻测试

- css-parser `src/tests/r5401_safe_overflow_position_tests.rs` 6 针：safe 五变体解析 /
  大小写不敏感（SAFE CENTER / UNSAFE start）/ unsafe 折算裸值 / 非位置组合 11 形拒收 /
  token 边界（粘合形、尾段、多空白）/ 裸关键字行为不变锚。
- css-parser `tests/r5400…` 无关；`src/tests/mod.rs` 注册。
- style-system：`property/tests/ui.rs::test_apply_align_content_overflow_position`
  （apply 接受集 + 拒收集）；`cascade.rs::r5401_webkit_box_flex_origin_key_winner_traced`
  （映射胜出置键 / 作者 flex 胜出不置 / 无 box-pack-align 不置）；
  `inheritance.rs::r5401_webkit_box_flex_origin_sets_legacy_flag`（键 → 旗标）。
- engine `js_dom_bridge/computed_style.rs::r5401_safe_overflow_serialization_tests`
  （computed as specified + place 简写序列化同源）。
- layout-engine `tests/r5401_webkit_box_align_self.rs` 2 针：legacy 标记容器抑制
  （item 100px / 容器 90px align-items:end → dy=−10 ★load-bearing）+ 真 flex 对照守卫
  （无标记 → safe-center 溢出回退 start dy=0，R5401 解析 + taffy fallback 能力锚）；
  `tests/r5401_webkit_box_align_self_kill_switch.rs` 独立进程关断锚（R3858 口径，
  关断 → dy=0）。
- layout-engine `converter/tests/advanced.rs` 补 Safe* 传入 taffy 断言。

## 验证

- 家族 6 案（flexbox-safe-overflow-position-001..006）修复态 3× 全 ✓0.00% 真通过-可信
  （003/004 顺带近似→真通过）；基线（stash 后 HEAD 构建）001 ✗1.40% 单红、006 ✓0.00%
  ——A/B 双向归因。
- corpus 全量 A/B（双侧重跑二进制，BASE = stash 后 HEAD a9e898975^）：**BASE 727F →
  FIX 727F**，FIXED={flexbox-safe-overflow-position-001}，REGRESSED 候选
  {run-in-table-cell-between-003} 双侧隔离 3×3 全 ✓0.00% 洗清 = run-in 在册抖动族；
  真通过-可信 **10360 → 10363**（001 翻真通过 + 003/004 近似→真通过）；BASE 实测
  727F 与 R5400 账面 731F 差 4 案 = 对端流 engine 提交差 + 抖动族漂移（非本轮面）。
- css-parser lib 3020 / style-system lib 2477 / layout-engine 全腿绿（含 3 新集成针）。
- make reftest 本地集 **725/725**；fmt ✓ / guarded-clippy 全工作区 ✓。
- make test 腿级：workspace 腿 3720P/5F（js_dom_bridge CSSOM 序列化 5 案 = 在册对端账
  652a78e00 域，stash 实证干净 HEAD 同 5 案同红，修复侧仅多 1 新针 pass）；renderer
  bin/lib ✓；browser bin 741P/1F（default_actions = 在册对端账）；browser --tests
  740P/2F（default_actions + bookmark_context_menu_open_in_new_tab_loads_url——stash
  实证干净 HEAD 同红，对端 zero-browser 面新记账）；GPU adapter 腿 96P+1P；quickjs
  腿除 script_fetcher s249/s250（stash 实证干净 HEAD 同红 = 既有账）+
  mutation_arm（在册对端账）全绿。
- product-smoke 6/6 struct-check PASS；bench-gate 定向 zero-css-parser,zero-style-system,
  zero-layout-engine **GATE PASS 25 指标（NEW=15）**。
