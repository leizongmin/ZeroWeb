# M2 s418 收口账——NotRun 5 勘根修复（import specifier 字面量内 `;` 截断）+ script-sandbox transform 面五守卫清除

- 日期: 2026-11-13；基线: HEAD 1c08a3a4a（s417 收口），驱动: s417 下一步「NotRun 5 css/json-module 外链模块基座 map 键错配勘根首位」。
- 在册口径: NotRun 5 = css-module/content-type-checking.html t4/t5（`text/css;boundary=something` / `text/css;foo=bar`）+ json-module/valid-content-type.html t5/t6/t7（`text/json;boundary=something` / `text/json;foo=bar` / `text/json;+json`）。

## 分型定谳：判别式 = specifier 内 raw `;`（map 键错配假设否证）

- s417 指针假设「外链模块基座 map 键错配」**否证**：runner fetch 面全链路对 `;` 处理正确——`parse_serve_with_content_type_query`（testharness.rs ~5753）只按 `&` 切 query、`ct` 值整体保留；`route_script_data_module_intent`（~6880）与 `mime_is_json` 均以 `mime.split(';').next()` 取 essence 判型，`;params` 不影响命中；路由表序 exact-named → referrer/accept → `__zw_mtype` marker（css/text）→ json_intent → load-error-events 全部正确响应 `;` 形 URL（探针直证，见下）。
- 判别式证据：同文件 %2B 编码变体（`text/json;%2Bjson`）NotRun 而 %2B 的 `+` 变体（`text/json;+json` 无 % 编码亦含 `;`… 注：t7 即 raw `;` 面）全 Pass 的对照组表明差异仅在 **specifier 字面量内是否含 raw `;`**——五个 NotRun 子测的 import URL query 全部携带 raw `;`，八个 Pass 子测（css t1-t3、json t1-t4/t8）均无。

## 根因（探针定谳，zzz-s418-probe.html 已删）

- 探针 v1（DUMP 注册滞后于 module block）：t5 CompileError 使页面执行中断、尾脚本不跑 → `tests:0` Timeout 无输出——改**前置 async_test DUMP 注册**（plain script 先于 module block）后定谳。
- 探针基线（修复前）：t1（plain ct）`t1-ok:{"test":true}`、t5（`;` ct）**无事件**；window error 有记录（"unclosed string literal"）但 `<script type=json onerror>` **元素 error 事件不派** → 页面 `onerror=unreached_func` 永不触发、async_test 永不完成 → runner 侧 NotRun。
- 根因位: **crates/script-sandbox/src/es_module.rs transform/emission 面五处 `split(';')` 前置守卫**（`extract_import_specifier_from_rest` / `extract_import_specifier` 字面量臂 / `transform_import` 副作用臂 / `static_dependency_specifiers` 的 head·clause_head 双处）——specifier 字面量 `"../serve-…?ct=text/json;boundary=something"` 先被按 `;` 腰斩成 `"../serve-…?ct=text/json`，`extract_string_literal` 收到未闭引号残段 → transform 期 `CompileError("unclosed string literal")`。
- 谱系: R3349（commit b586b97ca）已修 dynamic `import()` / 图收集器路径的 spec 提取（`extract_string_literal` 在匹配闭引号处停止、忽略其后剩余代码），但 transform 侧五个旧守卫漏网——`split_statements`（es_module.rs:2831）本身 string-aware（ScanCtx::Str 栈），语句切分不受字面量内 `;` 影响，守卫截断纯属多余且有害。
- 事件面语义: transform 期 CompileError 走 window error（script 源编译失败），与 fetch 失败走元素 error 事件分属两路——这就是「模块基座正常响应、页面却静默 NotRun」的机制闭环。

## 修复（es_module.rs 4 处 + 1 测试）

- 四处删除 `split(';')` 前置守卫，直接以 clause/rest 全文进 `extract_string_literal`（闭引号截停、`;`/with 子句/尾分号均为字面量后剩余代码）：
  1. `extract_import_specifier_from_rest`（~2272）
  2. `extract_import_specifier` 字面量臂（~1507）
  3. `transform_import` 副作用导入臂（~1947）
  4. `static_dependency_specifiers`（~456，head/clause_head 变量随之摘除；export 臂 `starts_with('*')` 恒假为**既有死逻辑**，按其语义忠实保留、不顺手修）
- 回归测试 `test_import_specifier_with_semicolon_in_literal`：`from "…;…"` / 裸字面量 / `v from "…;…"` 三形断言 spec 含 raw `;` 完整保留。
- 修复后全 crate `grep split(';')` 零残余。

## 验证

- 单元: `cargo test -p zero-script-sandbox` 443/443（含新测试）。
- 探针端到端（修复后 runner 重跑探针）: `DUMP=["t1-ok:{\"test\":true}","t5-ok:{\"test\":true}"]`——`;` ct import 载入并执行成功，JSON module synth 正确。
- solo 双驱动文件: content-type-checking **5P**（t4/t5 `;boundary`/`;foo=bar` 翻 Pass）、valid-content-type **8P**（t5/t6/t7 翻 Pass），原 Pass 子测零扰动。
- sweep ×2 join 对 s417 双基线 fixpairs-1/2（(key,state) 语义、\x1f 分隔）:
  - **my2 vs b2 = 0 预外 state-diff + 恰 5 new**（driving `;` 子测 NotRun→Pass 成行，全部 Pass）——配对轮全净，唯 driving diff。
  - my1 vs b1: 同 5 new driving；3 state-diff 逐一归因——checkable following-label 子测 Fail→Pass（s417 在册 flake 族，四轮矩阵已立同二进制双向翻转）；select-home-end ×2 文件级 Timeout→Fail（簇 D 族内 T↔F 漂移红集不变，s415/s417 在册）。另 m1 两文件级 Timeout（select-event / focus-dynamic-type-change-on-blur）为**负载 flake**：四轮状态矩阵 b1 全绿/b2 全绿/m1 文件级 T 部分完成（270→176、80→113 行）/m2 同二进制全绿回翻 + solo 双绿（270/80 行对 b2 逐一 stable 零差）；机制排除——select-event 全无 module 面（`import` 命中仅为注释词），focus-dynamic 虽为非引号 `type=module` 但 spec `"./input-types.js"` 无 `;`（新旧代码恒等），且 module 载入破坏形态应为子测成批 Fail（inputTypes undefined）而非跑完 113 子测后文件级 T。
- make test 全量: 155 腿 154 绿 + 仅 engine lib 在册红腿 3911P/**7F** 失败清单逐名
  IDENTICAL 零位移（对 s417 收口轮同名单 diff 空：img_runtime_deferred_settle_r5275
  / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_service_worker_messages / response_body_readable_stream_r2967 /
  t8k_websocket_error_nonterminal / t8k_websocket_event_flow）；clippy 腿
  （渲染流 13 crate -D warnings）零告警。过程注记：本轮 make test 期间并行
  clone（ZeroWeb-2 渲染流 rally）构建风暴 load 237-265（16 核），compile 阶段
  饥饿拖长至 ~2h（test-guard --compile-first 语义 compile 不限时、测试阶段
  900s 限内完成），测试结果数字与清单不受影响（compiling 期间无测试运行，
  无时序污染面）。
- fmt/clippy: rustfmt --check 干净；`cargo clippy -p zero-script-sandbox --all-targets -- -D warnings` 干净（test-guard 包裹）。

## 口径

- 在册红 212F+15T 不变（本轮修复面为 NotRun→Pass，不动 F/T 账）；**NotRun 5 → 0** 清账。
- 附带修复：控制文档承前链两处断头——415 头（s417 提交 rewrite 脚本 end_marker
  截断机制吃掉头部起点，HEAD 已损在案）+ 416 头（本轮同款脚本同缺陷）；两处均按
  原文前缀（`> **▶ 承前（…切片 N 收口）——`）恢复，diff hunk 仅 2（Active 块替换 +
  415 头修复），417→416→415→414 链完整。后续轮换头脚本须把 end_marker 截断点
  改为整行保留。
- 下一步: 挂账池承 s416/s417（渲染流 6 面 / sizes-auto F5 / 簇 D select 立项征询 / A2+D 簇 / checkable parent-label 2F / SVG `<image>` 载入链 canvas 13-hang 面）。
