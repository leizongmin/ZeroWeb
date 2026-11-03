# R5400：qualified rule 规范恢复 + 嵌套 @规则 extent 修复（2026-11-03）

案面：`css/CSS2/fonts/font-family-invalid-characters-003.xht`（✗99.96% 隔离 /
✗1.00% 全量，1.00% 档唯一窄修候选——R4452 五桶分类在案）。CSS 2.1
syndata.html#parsing-errors：规则 6 `#div3 { font-family: test}` 提前闭块后遗留
孤儿段 `foo, Ahem; } body { background: red; }`；assert 全页白底，TEST 页整页红。

## 归因（chromium oracle 探针链 + 临时 parse 探针，均用后即删）

- **ZW 全页红根因**：旧 `skip_malformed_qualified_rule`（CSS 2.1 §4.2 malformed
  statement 语义）在 `;` / 游离 `}` 处**重同步**——孤儿段被丢弃后，规则 7
  `body { background: red; }` 逃逸为顶层合法规则 → 全页红底。
- **chromium 语义**（CSS Syntax L3 §5.3.2 consume-a-qualified-rule）：前奏只跑到
  `{` / EOF；`;` 与游离 `}` 都是前奏 component value，**不触发重同步**；规则连同
  第一个 `{...}` 块整体丢弃，块后重同步。headless computed-style 四探针实证：
  `garbage; body{red}` body 不应用；`junk1 junk2 } body{red}` 同；`foo, Ahem; bar
  {blue} body{red}` body 应用（一个块后重同步）；`;;; #x{red} ;;;` 规则不应用。
- **第一轮修复回归**（4 案 @supports 块内族）：`#test1 { background: green };`
  型游离 `;` 在块体内被规范恢复吞到包块 `}` 级联吞块 → at-supports-font-format-001
  / font-tech-001 / selector-detecting / css-nesting/supports-rule 全红。修复：
  `in_block: bool` 线程化——块体内当前深度 `}` 是包块边界（不消耗即返回），顶层
  `}` 按前奏吞掉（chromium P3 实证）。前三案洗清。
- **supports-rule 仍红 ✗4.17%**（= 2 个 100×100 方块 test-3/test-4 红，PNG 采样
  定位；kill-switch 运行时关断回 ✓ 定谳为本改动路径）。**前缀二分 + 恢复入口
  trace + 解析器实例归因**（临时探针，用后即删）：`.test { > .test-2 {…}
  @supports (selector(> .test-2)) { > .test-2 {…} } }` 后随 `.test-3` 整条消失，
  顶层规则解析落在**游离 `}`** 上。
- **根因（嵌套 @规则 extent 泄漏，先于本改动存在）**：
  `consume_nested_at_rule` 块体用 `skip_simple_block` 消耗 extent——但该函数的
  `{` 开括号在**调用内**消耗（深度从 1 起算），此处开括号已消耗（深度从 0 起算），
  内容含嵌套规则时**首个内部 `}` 归零提前返回**：at-rule 自身闭 `}` 泄漏到外层
  流 → `.test` 的块闭误吞 @supports 的 `}` → `.test` 自己的 `}` 泄漏到顶层。
  旧顶层恢复对游离 `}` 静默丢弃（返回不消耗 + 进度守卫强吃），**掩盖了该泄漏**；
  规范恢复把游离 `}` 当前奏吞掉（chromium P3 正确语义）后级联吞掉 `.test-3` 与
  后随 @supports。body 切片同时失衡（含 `{` 丢 `}`），下游子解析靠 EOF 容错。

## 修复（两个单点，css-parser/parser/mod.rs）

1. **`skip_malformed_qualified_rule_spec(in_block)`**（新函数；样式规则 None 回退
   专用，at-rule None 回退与嵌套规则回退保持旧恢复——R3757/font-face 在案边界
   不动）：前奏跑到 `{`（`(`/`[` 整块消耗保护内部 `{}`），`;` / 顶层游离 `}` 按
   前奏吞掉，一个 `{...}` 块（含）后重同步；`in_block` 时包块 `}` 不消耗即返回。
   kill-switch `ZW_CSS_QRULE_SWALLOW=0`（default-on）回退旧 `;`/`}` 重同步语义。
2. **嵌套 @规则块体消耗**（`consume_nested_at_rule` LBrace 臂）：改按 `{}` 计深
   循环到匹配 `}`（EOF 破），body 切片排除两端括号——extent 全消费 + body 平衡。

## 常驻测试

- `css-parser/src/tests/r5400_qualified_rule_recovery_tests.rs` 6 针：`;` 垃圾
  吞后随规则 / 驱动案孤儿段整吞（`#d{font-family: test}` 存活断言）/ 一块后
  重同步 / 游离 `}` 不重同步 / 干净样式表零触碰 / **嵌套 @规则 extent 不泄漏
  闭括号**（supports-rule 回归 pin，5 规则全存活断言）。
- `css-parser/tests/r5400_qrule_kill_switch.rs`（包根独立进程锚，R3858/R5391
  口径）：关断后 `;;; div { color: red; }` 恢复旧 `;` 重同步语义（div 存活）。
- `coverage_round6.rs::test_stylesheet_with_garbage_tokens` 判例翻转：
  `;;; div { color: red; } ;;;` → 0 规则（chromium oracle 实证，R5394 判例翻转
  先例，注记在案）。

## 验证

- 新针 6/6 + 关断锚 1/1；css-parser 全腿 **3014 绿**。
- kill-switch 双向：关断 `;;;` 锚转旧语义（1 规则）；开断 5 WPT 案全绿（下）。
- 案面五案：font-family-invalid-characters-003 ✗99.96% → **✓0.01%**；
  supports-rule ✗4.17% → **✓0.00%**；at-supports-font-format-001 / font-tech-001
  / selector-detecting → **各 ✓0.00%**。
- corpus 全量 A/B（双侧重跑二进制：BASE = stash 修复后 HEAD 0c898dc6a 构建）：
  **BASE 731F → FIX 731F**，FIXED={font-family-invalid-characters-003}；全量窗口
  mover（cv-099 + tao-127/130/148/153）双侧各 3×3 隔离全 ✓0.00% 洗清＝在册并发
  抖动族（cv-099 首见，同族并发负载型），真态 **REGRESSED=∅**。
- 本地集 **725/725**；fmt ✓ / `make guarded-clippy` 全工作区 `-D warnings` ✓
  （含顺手修 layout-engine/float_positioning.rs:4214 文档行首 `+` 触发的
  doc-lazy-continuation——对端流已提交代码被新 clippy 破门禁，纯注释重排零语义，
  单独 commit 可溯）/ make test **除 HEAD 既有对端账**（js_dom_bridge CSSOM 5 案，
  stash 后干净树复跑同败实证）零新增失败 / product-smoke 全腿 PASS / bench-gate
  定向 zero-css-parser **GATE PASS 19 指标（NEW=7，parse_ms 5.79 vs 基线 5.61
  预算内）**。

## 挂账与下一步

- R5399 挂账全维持：floats-031 深结构、img/replaced UA display 翻转、R5397
  negative-clearance/mc-104、floats-clear-multicol 7 案、R5242/R5392/R5391/R5390/
  R5389；对端账（js_dom_bridge CSSOM 5 案/image_batch_e2e/mutation_arm/
  default_actions/gpu_compositor/paste_and_go）维持。
- **下一步 R5401**：①731F 真态账本续扫择轻（避开对端面与 floats-031 深结构）
  ②backdrop-root 深评（维持）③color-6（维持）。
