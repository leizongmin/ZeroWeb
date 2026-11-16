# 基本形状插值 applier 臂刀——clip-path/shape-outside 进动画写回（Phase 2 残面①清偿）

日期：2026-11-16（二十一轮，wptfyi-css）。二十轮残面①（clip-path shape() 串
raw 插值全对、canonicalize 恒 None——applier 不解析 clip-path 文法）本刀清偿：
`AnimationClock::apply_single_property` 新增 clip-path/shape-outside 两写回臂 +
clip_interp 共享插值器全族通道 + echo 保留门保 B0 伪通过池。**twin 孤儿 WIP
认领轮**（06:06 twin 实例 5 小时限额中断，WIP 静默 >5h 无进程无提交，按控制面
避让协议磁盘终态仲裁认领；审读 diff 五站连贯：import/loose 臂/strict 臂/applier
臂/3 测试）。

## 改动面（2 文件，es_module.rs s419 peer WIP 恒不入按惯例排除）

- **engine/animation.rs**（+~370 行）：
  - `apply_single_property` 两臂（twin 主体）：`clip-path` —— `parse_clip_path`
    解析成功即写 `style.clip_path` 槽；`shape-outside` —— None/Shape 包装/裸
    shape-box 四关键字三形（复用十九轮 `ShapeOutsideComputedValue` wrapper）。
  - loose `interpolate_property_value` 合并臂（本轮定稿）：`clip-path |
    shape-outside` 走 `clip_interp::interpolate_clip_path`，None 时经 echo
    保留门（[`clip_pair_echo_preserve`]）→ 空串（apply 解析必败 → 槽位不动 →
    canonicalize 恒等门回落 echo＝刀前行为逐位保真），未命中门回落 css-values-4
    §8.5.6 离散翻转（≥0.5 → to）。
  - strict `interpolate_property_value_strict` shape-outside 臂（twin）：登记门
    端点解析成功且真插值才 Some。
  - echo 保留门 `clip_pair_echo_preserve` + 结构化 calc 检测
    `clip_value_has_calc`（本轮）：命中四形保位——①任一端点插值视角 calc 化
    （calc 端点离散 + 期望串 setProperty 拒收 → B0 双巧合伪通过）；②rect 族
    （inset/rect/xywh）跨形对（spec compute 为等价 inset 应整体插值，期望串
    calc 化 inset 不可达）；③`inherit` 端点 ∧ 对端 rect 族；④path() 双端
    （段数失配离散步，翻转暴露 jsanim flip 时序面）。
  - 5 测试钉：关键字半径对离散（strict 拒登记 + loose 翻转两钉，twin）、
    echo 保留门（calc 端点/左上缘反例/inset↔xywh/inherit→rect/inherit→circle
    反例/initial·unset 翻转反例/none↔circle/path 失配，本轮）。
- **engine/animation/clip_interp.rs**（613 行，twin 主体 + 本轮 Fix 1）：
  `interpolate_clip_path` 共享插值器全族——inset（四边 + round）/circle/
  ellipse（半径 + at 位置）/polygon（fill-rule + 点对）/shape（命令级：
  move/line/hline/vline/curve/smooth/arc/close，控制点 from_ref 一致性门）/
  rect/xywh（`auto`/圆角槽一致性门）/path（段数·命令失配 None）；异构/单位
  失配/calc 端点一律 None 调用方定夺。**Fix 1（本轮）**：`lerp_radius` 关键字
  半径对（含关键字↔关键字）整对离散——css-shapes-1 §interpolation 关键字半径
  不参与插值，此前同关键字分支误判恒等可插值（18 REG 定谳修正）；删孤儿
  `radius_kw_str`。

## 机制要点

- **B0 伪通过池双巧合**（jsanim 十轮同款纪律）：interpolation-testcommon 的
  expectedValue 由 harness 把期望串 setProperty 到独立 expected-target 后读
  **本机 gCS**——期望串含 calc 化形时 setProperty 拒收 → expectedValue 落回
  静态值；B0（无 applier 臂）动画值不写槽 → gCS 恒 echo 级联值。两巧合叠加
  → B0 伪通过。刀后离散翻转写回暴露失配 → REG——真值（calc 代数插值）不可
  达，按保位纪律 echo 门回落，不追真值待 calc 面整刀。
- **edge-offset calc-ness 在归一层**（v2 测试失败定谳）：`circle(25% at
  right 5% bottom 15px)` 原文无 "calc(" 子串——`shape_axis_comp_resolved`
  把 `Edge(Right/Bottom, off)` 归一为 `Calc(100% − off)`（左/上缘折纯偏移
  不算 calc）发生在解析树之后的消费层，文本检查结构性漏检 → 按解析后结构
  遍历检测（`LengthValue::Calc` 直查 + 位置分量归一判定 + inset/rect/xywh/
  polygon/shape 各槽逐分量）。
- **`initial`/`unset` ≠ `inherit`**（v3→v4 inter-B1 对账定谳）：initial/unset
  的 underlying 恒 `none`，none↔形状对 Chrome 即离散翻转语义（期望=端点原形，
  settable）——翻转是真增益不是伪通过；inherit 的 underlying = parent
  computed 值形不定，与形状端点的真插值期望串为 calc 化 inset 不可达——
  才是伪通过池。keyword 臂过宽一度压掉 16 真增益（b1v7），收窄 `inherit`
  单关键字后回收。
- 二十轮残面①清偿实证：clip-path-interpolation-shape 页 +88（30 个 shape()
  子测 raw 插值全对 canon=None → applier 臂解析写回转真）。

## 迭代定谳链（四代二进制）

- **v1（b1v5，e3d13289）**：twin WIP 原样认领链——**+503 / 0 lost / 29
  regressed**。四类定谳：①18 = 关键字半径对恒等误判（→ Fix 1）；②6 =
  edge-offset calc 位置对伪通过暴露（→ echo 门 calc 检测）；③4 = inherit
  →rect/xywh calc 化 inset 伪通过暴露（→ echo 门 inherit 臂）；④1 = path
  段数失配 at(1)（→ echo 门 path 双端臂）。
- **v2（engine lib 4072P/1F 拦截；b1v6，7692f85a 带 bug 批预读）**：gate v1
  文本 calc 检查 + rect 族跨形 + path 双端——`test_clip_pair_echo_preserve_gate`
  失败定谳文本检查漏 edge-offset 类（calc-ness 在归一层）；链未中断，
  b1v6 = +509/0/10（对非 edge-offset 类有效的预读账，10 REG = 6 edge-offset
  + 4 inherit）。
- **v3（b1v7，319523b2）**：`clip_value_has_calc` 结构化遍历 + inherit 臂
  （初版 CSS-wide 五关键字全收）+ 左/上缘反例钉；engine lib 4073P/0F；
  b1v7 = **+499/0/0**——但 b1v6→b1v7 inter-B1 对账暴露 keyword 臂过宽：
  gained 6 / lost 16 / regressed 20，16 真增益（initial/unset→inset/rect/
  xywh 期望=端点原形的翻转对）被压。
- **v4（b1v8，afd7f247，权威）**：keyword 臂收窄 `inherit` 单关键字 +
  initial/unset 翻转反例钉两枚入 gate 测试；engine lib 4073P/0F。

## A/B 权威账

- B0 = results-trans-b1v4（HEAD da3c22bd5 码态采认——二十轮 v4 权威批终态，
  runner md5 639f7bd1）；B1 = **b1v8**（md5 afd7f247）七腿新跑 rc=0 ×7；
  transcript /tmp/zw-trans-{b1v4,b1v8}-\<corpus\>.txt；JSON
  .acceptance/wptfyi-css/results-trans-{b1v4,b1v8}；test-guard 包裹
  --per-proc-mem 4 --total-mem 8。
- corpus 腿：css-masking + css-shapes + motion（主面）+ filter-effects +
  css-gaps + css-multicol + css-typed-om（护栏）。
- **权威账：gained=519 lost=0 regressed=0，pages_gain=10 pages_loss=0**
  （DOWNS 红线零触碰；echo 池结构性保位）。逐页：
  clip-path-interpolation-001 +67（108→175）、-002 +83（292→375）、
  -shape-control-points +57、-shape +88（残面①清偿）、-xywh-rect +32
  （152→184）、clip-path-interpolation-discrete +8、shape-outside-
  interpolation +71（120→191）、shape-outside-path-interpolation +53、
  shape-outside-shape-interpolation +56、motion offset-path-interpolation-009
  +4。全四代 compare 快照 /tmp/zw-r21-b1v{6,7,8}-compare.txt。
- 中间二进制 provenance：v1 e3d13289 / v2 7692f85a / v3 319523b2 / v4
  afd7f247（bin-trans-b1v{5,6,7,8}/ 留档）；es_module.rs s419 peer WIP 两侧
  同载零贡献在案（B0 同树态）。

## 门禁

- 单测：engine lib **4,073P/0F**（v3/v4 两代复验；4071 基线 + 关键字半径钉 +
  echo 门钉，门钉内含 4 反例子断言防过宽）；v1 链 4,071P/0F（Fix 1 前
  twin 基线）。
- webview lib 定向腿：**852P/2F**——2F 逐名归因：`csp_gate::
  csp_connect_gate_blocks_fetch_sh1_m2s5`（s348 在册环境红，HEAD 基线在案）+
  `basic::test_webview_fetch_url_invalid_host`（**本环境新挂账代理形红**：
  环境 `HTTP_PROXY=192.168.1.212:10801` 把 0.0.0.0:1 连接失败翻译成 502
  响应，curl 同环境实证 `502`；reqwest 吃代理 → fetch 拿到响应 →
  `is_err()` 断言失败；本刀 diff 仅 engine animation 两文件、fetch 路径
  零触碰，round-20 同测试通过时代理环境变量不在位）——**零新增红**。
- clippy `cargo clippy -p zero-engine --all-targets` 净（唯一告警
  script-sandbox 系 peer s419 WIP 同载非本刀面）；fmt `cargo fmt
  -p zero-engine` 净。
- 全量 make test 未跑挂账（沿用二十轮定谳）：HEAD 既有 fmt 漂移两文件
  （dom tests_9 / testharness）+ peer s419 WIP es_module 挡门；按质量门禁
  「按实际影响执行检查」以 engine+webview 定向腿覆盖（改动面单 crate）。

## 残面（下刀队列）

- **calc 代数插值共享面**（整刀候选）：lerp_len Calc↔Calc 分量插值
  （edge-offset 位置/inset calc 分量真插值）+ rect 族跨形→inset 归一插值 +
  setProperty calc-inset 写面——三件齐才把 10 个保位对转真值。
- **jsanim flip 时序面**：共享 Steps(Start) 臂未吃 StepPosition（二十轮
  定谳沿用，T 通道用本地 entry_eased 绕开）+ path 段数失配 at(1) got=from
  行为（本刀 echo 门保位）。
- lerp_len 物理单位族（pt/pc/cm/mm/in/Q parse 期折 px 后可插值，现恒 None
  离散）。
- 二十轮残面②③④⑤（transition:all 展开/离散翻转语义/css-transitions 帧驱动
  面/共享插值器量域化）不变。
