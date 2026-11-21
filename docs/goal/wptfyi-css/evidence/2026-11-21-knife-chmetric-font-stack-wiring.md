# 刀 CH-METRIC · testharness 字体栈接线（2026-11-21）

## 缘起与认领

css-values **line-break-ch-unit 97F**（97 fit 全 P + 97 wrap 全 F，
`expected 20 +/- 1 but got 10`）：页面 `font-family: monospace;
font-size: 10px`，`width: i ch`（i=3..99）盒内放 i/i+1 字符断行断言。
失败形态 = **1ch 从不等于真实 '0' advance**：

- style 侧 computed.rs:233 `LengthValue::Ch(v) => v * font_size *
  font_metrics.map_or(1.0, …)`——`font_metrics` 恒 None → 1em 兜底
  （该兜底注释自证是 Ahem-oriented 口径：语料字体 '0' advance = 1em）；
- layout 侧文本 advance 用内建默认字体（~0.6em）；
- 宽度 i×10px vs i+1 字 ~6(i+1)px 对所有 i≥3 恒不换行 → wrap 97F；
  fit 97P 是「不换行」的平凡通过。

认领链：calcsize 收敛笔（NOTES-knife-calcsize-grammar-claim 08:4x）
列本刀为下轮 recon 位；handoff-0845 记 fork B 声明「CH-METRIC 侦察」；
08:50 实测 fork B（session 4c42acbb）正工作 CALC-SIZE-CANON
（serialize_math/fold_decl_specified 域），CH-METRIC 实现位无主，
本线按协议认领（NOTES-knife-chmetric-claim 08:51）。

## 机制

**testharness 路径是全产品唯一零字体注入的渲染路径**。resolver 与
per-family 度量映射在 reftest harness（reftest.rs:806 resolver+metric
map+font_loader 三件套）与生产 renderer（runtime.rs:804-806）均有同款
配方接线（R1259/R3993/R4373/R4377 字体栈：Liberation Serif id0 /
Liberation Sans / Noto Sans Mono CJK SC monospace / Noto CJK 回退 /
Ahem），唯独 testharness runner（tests/wpt-runner testharness.rs，
WebView per-case 构造）从不调用 `set_font_resolver`/`set_font_metric_map`
——R4377 注释「monospace 文本宽 ~20% 与 chromium 分歧」修生产时
testharness 被遗漏。

链路（修复后端到端）：

1. `create_font_loader()`（reftest_fonts，进程级 OnceLock，首调 ~0.5s
   之后 duplicate 复用）→ `build_font_resolver()` + `build_line_metric_map()`
   （family → ex/cap/ch/ic/size_adjust aspect，ch_width = '0' advance
   比率，css-values-4 §5.1.2）；
2. `webview.set_font_resolver` → pipeline → layout 文本度量；
   `webview.set_font_metric_map` → style_system.font_relative_metrics；
3. compute_styles（lib.rs:3273 `font_relative_metrics_for(&style.font_family)`）
   → `FontRelativeContext.current` → computed.rs:233 ch 臂真实比率；
4. style 与 layout **同 loader 派生**（zeroweb-guidelines #3 度量单一
   owner）——宽度与文本 advance 同源，i 字恰合 / i+1 字换行两断言同时
   兑现。**只接 metric map 不接 resolver 是反向回归面**（宽度收半、
   advance 不变 → fit 97P2F），本刀两件套齐改。

face（3 文件，tests/wpt-runner）：

- `testharness.rs`：font 步接线两调用 + `#[used]` marker
  `CHMETRIC testharness-font-stack-wiring`；
- `reftest.rs`：`mod reftest_fonts` 可见性 pub(crate)；
- `reftest_fonts.rs`：`create_font_loader` pub(super)→pub(crate) +
  Linux 钉测 `base_loader_publishes_monospace_ch_metric`（resolver/
  metric map 同 face + ch_width ∈ (0.3, 0.8) 真实 advance 比率带）。

预期兑现核对：line-break-ch-unit wrap 97F→P（monospace ch≈0.5em，
宽度 5i px vs i+1 字 5(i+1)px 恒换行；fit 恰合不变）。

## blast radius（预注册不预设）

resolver/metric 接线改变 initial serif（id0 → Liberation Serif）与
全部 generic 族文本度量 + Ahem 面（'0'/'M' advance = 1em）真实解析
（css-fonts-4 #first-available-font）。电池七语料探针：values（靶）、
text（换行暴露面）、fonts（font 面）、sizing（内在文本尺寸暴露面）、
motion/typed-om/color（远哨，SET-IDENTICAL 为佳）。有位移按页归因，
**P2F>0 逐案归因后才可入账**。ride（他流 WIP 双臂同载）效应相消，
账面只认 b0→b1 差。全语料净效由 weekly 电池复验。

## 电池（unit zw-knife-chmetric-battery-<hhmm>）

脚本 `.acceptance/wptfyi-css/knife-chmetric-battery.sh`（calcsize 配方
逐字改参）：flock `/tmp/knife-chmetric-battery.lock` + RESULTS-NOT-FRESH
双守卫；编译门 `cargo check -p zero-wpt-runner`（依赖链全量）；b0 =
剥 face 构建 / b1 = face 构建；FM/M2 门 + S1/S2/S3 ride census；七语料
× 双臂 RECONCILE（case 键级 gained/lost + ASYM 行）。

## 结果（收账 2026-11-21 10:0x）

- 电池 unit `zw-knife-chmetric-battery-0929`：B0 BUILD START 09:29:15、
  B1 09:30:19、14 legs 09:30:34–09:50:36、RECONCILE + GATE-DONE
  09:50:36、EXIT rc=0（`knife-chmetric-battery-0929.log`）。
- 门：FACE-MARKER b0=0 b1=1（log:160）；copied bins md5 b0
  `f53bd8c7643a51af…` / b1 `942c1df0f7aefd…`（M2 双臂异 bin 成立）；
  S1/S2/S3 ride census——SKILL.md `f12bbf53…` 与 es_module.rs
  `8c77f2ad…` 三窗字节同；master.md `c427137f→d0247405` 仅 S1→S2
  变更且 S2==S3 稳定（docs 不入 bin 输入）→ 二进制干净，记 census
  注记非毒化。
- 逐语料（case 键级，断言数）：
  - values **+1/−0**：`rch-invalidation.html` P 0→1；
  - text **+86/−4**：43 页 `baspglwj-*` F2P 共 +86 断言；
    121/124 两页 P2F 各 −2；
  - fonts/sizing/motion/typed-om/color 五哨兵全零（SET-IDENTICAL）。
  - 净 **+83**（+87/−4），无 ASYM 行。
- P2F 归因（4 断言 / 2 页）：`baspglwj-121/124` 含 U+202F NARROW
  NO-BREAK SPACE（GL 类，css-text-3 §line-break-details 不可断行）。
  b0 假度量使 50ch 盒不溢出 →「无需断行」平凡通过；b1 真度量后文本
  溢出，breaker 在 GL 处误断 → **既有 GL 断行缺口被揭显**，非接线
  回归，挂账断行器刀（+4 恢复靶）。
- F2P 机制：真度量 → 50ch 盒真溢出 → 可断行空格处正确断行，
  offsetHeight 期望兑现（+86 全数翻 P）。
- 主靶未翻定界（line-break-ch-unit 97F 持平）：页面 inline 声明
  `style.width = i+'ch'`。三探针（v1 接线 / v2 scale / v3 同文档同
  声明二分）定界三径（10:0x 修正笔：径 1/2 定界收窄，见
  NOTES-knife-chmetric-claim 10:0x 仲裁节）：
  1. layout 消费径·静态声明（stylesheet 与 markup inline style 属性）
     **已通**：bisect 探针静态臂 3ch@10px ow=15/oh=20 翻面；
     monospace '0' advance=0.5em 精确（inline-block 10 字形串
     offsetWidth 直测 10px→50 / 100px→500，scale-invariant；
     10ch@100px 静态臂 offsetWidth=500 同证）；
  2. **JS DOM-mutation 径不通（真因）**：`createElement`+
     `style.width='3ch'` 与载入后 `setAttribute('style',…)` 臂恒
     ow=30/oh=10（ch 塌 1em）——element/class 选择器、写入 API
     形态、50 兄弟干扰臂均排除；mutation 后 restyle 的样式解析丢
     metric map（`resolve_length` metrics=None → 1em 兜底），
     真页恰走动态创建路径 → 97F；
  3. gCS 读回径**不通**：gCS width 双臂恒 30px——桥层
     `js_dom_bridge/computed_style.rs:2656` ch 臂 `ch_width:
     Some(font_size_px)` 静态 1em 兜底，不消费
     font_relative_metrics（与 layout 消费径无关，静态臂布局已
     翻面，红鲱鱼另域）。
  → 本刀边界（runner 注入面）内零回归；JS mutation 径挂
  CH-METRIC-2（engine 域），gCS 径另挂一刀。
- 与 CALC-SIZE-RESOLVED（peer 619e194c7 认领，calc-size used/resolved
  value 域）文件面与失败面均不相交。
- 钉测：`base_loader_publishes_monospace_ch_metric`（reftest_fonts.rs
  Linux 臂）经 test-guard `--compile-first --per-proc-mem 4
  --total-mem 8 --time-limit 900` 定向执行
  （`chmetric-pin-test-<HHMM>.log`）：1 passed 0 failed（rc=0）。

## 域界与挂账

- 本刀只接 testharness 注入面；ch/ex 兜底值本身（computed.rs map_or）
  不动（Ahem-oriented 口径仍服务 map 未收录 family）；canvas 度量
  （registry loader）不受影响（canvas 文本另栈）；paint 侧
  `set_font_loader` 不动（testharness 断言不消费像素）。
- 挂账维持：CALC-SIZE-INTERP（MOTION-3 fork A 认领）、CANON
  （fork B 4c42acbb 在飞）、INTRINSIC ~209F、calc-interpolation 117F、
  if-cycle 38F。
