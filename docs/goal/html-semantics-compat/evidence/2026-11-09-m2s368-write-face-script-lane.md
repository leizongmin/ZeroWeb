# M2 切片 368：写面 SCRIPT 编译车道——load-error-events-3 T→P（域 T 4→3）

**日期**: 2026-11-09
**目标对**: 挂账「classic 脚本 SCRIPT 编译切片」勘根 + 施工——s367 定谳的 GDI 提升缺口（write 面 mid-body eval 对同脚本 var ReferenceError）
**结果**: **混合路由设计定谳**（含 write 的 classic 脚本走 SCRIPT 直编译车道，eval wrapper 车道不动）；一刀落地（webview.rs）后 load-error-events-3 **文件级 Timeout → 子测 Pass**，域 the-script-element **T 4→3**、公共键 1136 零状态变化、零 P→F。

## 勘根定谳（s367 四条排除路径后的结构性修复设计）

1. **sandbox 现状**：`v8_runtime.rs` `execute`（1144 行）本体就是 `Script::compile(code, None)` +
   `run`——SCRIPT 直编译能力**已存在**，无需新 sandbox API；"ScriptOrigin 缺席" 注记（122/777）
   的命名面由 `//# sourceURL=` 注释承担（既有机制，V8 源末行取名），本刀沿用。
2. **parity 全查**：currentScript 设/清——`script_set_current_script`/`script_clear_current_script`
   宿主侧 helper 已存在（R3258 即为「不走 wrapper 的 classic 执行路径」预备），清沿既有无条件
   clear，设为本刀新增 pre-execute；strictness——间接 eval 的 strict 本就取自源内 directive
   （拼接保序），Script 编译原生同语义，`script_directive_is_strict` 扫描机对本车道自然退役；
   sentinel——wrapper 专设 `__zw_pgerr__` 复位+尾读，直编译车道**必须跳过尾读**（不复位即读到
   上一个 wrapper 脚本残留值 = 假 first_strict_error，见施工 ③）；错误面——直编译抛错走
   `execution Err → combined_err → first_strict_error`（文本形状 t5 同契约：消息头+帧行），
   wrapper 的 reportError 透传 + error 后 load 派发本车道暂缺（见挂账）。
3. **混合路由 vs 全车道迁移**：全 strict 车道迁移（ retire wrapper）会把错误序（s335
   `[error, body-μt, load]`）依赖的 in-execute catch 面、跨脚本词法重声明早错面、event.error
   原值透传面一次性全开——blast radius 不可控；混合路由把刀口收在 write 面族（静态探测，
   the-script-element 域 33 页 / semantics 全树 36 页），漏检 fail-open 回落既有车道零回归。
   全迁移作为后续独立挂账（设计勘根已完备，见挂账）。
4. **slow.py 仿真臂不需要**：`/common/slow.py` 404 即 link error 结算，无监听方，判据只要求
   onExecute 触达——绿色 solo 首跑实证（修复一刀即 Pass，未触任何 runner 侧 slow 端点仿真）。

## 施工（webview.rs 单文件三处 + 一 helper）

1. `classic_source_has_document_write(code)`：子串 `document.write` 探测（write/writeln 两形；
   动态构造写调用不命中 = fail-open）。
2. classic 臂三叉：`strict && 命中` → **SCRIPT 直编译车道**（classic_code + 源真末尾
   `//# sourceURL=<src>`，换行先行防行注释吞后缀）；`strict` → 既有 s335 wrapper（零漂移）；
   非 strict → 既有 browser 车道前置串（零漂移）。
3. 直编译车道 pre-execute `script_set_current_script(idx)`（full 构建后、`sandbox.execute(&full)`
   前）；sentinel 尾读加 `!classic_script_lane` 门控（勘根 2 的假错面）。
4. fmt PASS、clippy --all-targets 净。

## 验证

- **solo**：`make testharness-html-semantics FILTER=load-error-events-3` →
  `Pass ... :: no src, parser-inserted, has style sheets blocking scripts, script nesting level == 2`
  （红态证据 = s367-b 域跑文件级 Timeout + s367 probe-e `sameVar-REFERR`/`laterVar-REFERR`）。
- **域 join**（A=/tmp j367c = s367-b 1137 对 vs B=本刀域跑 1137 对；LC_ALL=C 全键 join，
  按键态稳定分类防 s139 叉乘；sort -k2 只取路径字段的 join 会在同路径多子测键上叉乘——
  本轮复踩一次，全键 join 归零）：
  - 公共键 1136：**1128 Pass|Pass + 5 NotRun|NotRun + 3 Timeout|Timeout，零状态变化**；
  - A-only 1 = load-error-events-3 文件级 Timeout 键（旧形态 `path :: path`）；B-only 1 =
    子测 Pass 键（新形态）——合即该页 **T→1P**（item 38 键形人工合并判读）；
  - 残 T=3 与 s367 定谳逐名同构（credentials-setTimeout / instantiation-error-8 /
    script-crossorigin-network，基建面挂账）；域残 F=0 保持；零 P→F。
- make test：v8 workspace 腿完整 **9329P/7F**（7 红既有集逐名同构，零新增红——img_runtime_
  deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_sw_messages_dispatch / response_body_readable_stream_r2967 / websocket_error_
  nonterminal / websocket_event_flow）；P 较 s367（9326）+3 属并行流组合态计数漂移（js_dom_
  bridge 腿 3821，本刀仅 webview 面不在该漂移源）；quickjs 腿未达（make 于 v8 腿既知 7 红
  abort EXIT=2 同形态）。改动面在 webview 交付循环 runner 车道激活（write 探测命中形），
  engine lib 单测无此形态覆盖。

## 挂账 / 下一步

- **全 strict 车道迁移**（后续独立切片，设计已勘定）：retire eval wrapper——需要 sandbox
  in-execute catch 面（保 s335 `[error, body-μt, load]` 序，`__zw_e` 原值挂全局供 catch_js）、
  quickjs 腿平行实现、reportError 原值透传迁移、跨脚本词法重声明早错的语料面 A/B。
- 直编译车道错误面增量：reportError 透传 + 抛错后元素 load 派发本车道暂缺（corpus 域内
  write 族无此判据，域 join 零漂移实证；全迁移切片统一补齐）。
- 基建面 T=3 维持 s102/s124 挂账；probe8/9/10 转正决策维持。

## 日志

- /tmp/s368-solo-le3.log（绿色 solo）、/tmp/s368-b-script-element.log（域 B 面）、
  /tmp/s368-make-test.log（全量回归）——tmp 工件，入库载体验证面见本文数字。
