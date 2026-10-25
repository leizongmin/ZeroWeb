# M2-s87 施工轮：dynamic-import must-reject 族收口（错误类型/对象身份/MIME 三根一刀）

日期：2026-10-25 ｜ 轮次：s87（施工，s86 切片序①）｜ 前序：[s86](2026-10-25-m2s86-scripting1-triage.md)

## 1. 入场红图（当拍，HEAD=1a724948a + FILTER=the-script-element/module/dynamic-import）

子树当拍 **6P/49F/11T**（485 行全量面另见 §4；两轮扫描均在 TLA
`*.tentative.html` 案页脚抛（`SyntaxError: Unexpected identifier 'defer'`，
0 declared tests → runner rc=1）截断，`v8-code-cache.html` 未及上报——
runner「整案抛且零声明即致命」行为先于本轮存在，账目照记不掩盖）。

must-reject 24F 三根（s86 记「单根」经当拍细分）：

1. **错误类型根（18F）**：`__zw_load_module` 对宿主返空一律
   `throw new Error('Module not found: …')` → 404/500/跨源/裸 specifier
   全 reject 成普通 `Error`。HTML spec fetch-a-single-module-script：取回
   失败须 reject **TypeError**（WPT `promise_rejects_js(t, TypeError, …)`）。
2. **对象身份根（4F）**：parse/求值期失败不缓存——重复 import 每次新对象。
   WPT script-error 判据（对齐 spec module map 失败记账）：parse error、
   求值 error、**依赖解析失败**复抛**同对象**；fetch 期失败（404/MIME/跨源/
   顶层裸 specifier）逐次**新对象**（fetch-error 全族 `assert_not_equals`）。
3. **MIME 根（2F）**：`?pipe=header(Content-Type,text/plain)` wptserve 响应头
   模板未建模——errorhandling-wrongMimetype.js 按扩展名 .js 误判 scripty，
   体含 `export` 经 classic 化 transform 在 eval 抛 SyntaxError（须拒未拒）。

## 2. 本轮一刀（三面）

- **`crates/script-sandbox/src/es_module.rs`**：prelude `__zw_load_module`
  失败三态——①宿主返空 → 逐次新 `TypeError`（不入缓存）；②宿主返
  `resolved\x1f` 空码标记（依赖图断）→ `TypeError` 记入 `__moduleFailures`
  复抛同对象；③取回成功但 parse/求值抛错 → 原错误对象入缓存复抛。
  `__moduleFailures` 并入 `MODULE_CACHE_RESET_SCRIPT` 与两处 init。
  未发标记的宿主（renderer/tab）②退化为①，行为不劣化（契约向后兼容）。
- **`crates/webview/src/webview.rs`**：`collect_module_deps_recursive` 返
  `ModuleGraphFailure` 三分类（TopFetch/DepResolution/DepFetch——裸
  specifier 无 import map 即解析失败，spec resolve-a-module-specifier）；
  `__zw_compile_module` 失败两态回传：fetch 期失败返空、依赖解析失败返
  `resolved\x1f` 空码标记。
- **`tests/wpt-runner/src/testharness.rs`**：`wpt_data_script_fetcher` MIME
  派生序新增最高优先级 ①——`?pipe=header(Content-Type,X)` 模板（其余
  pipe 指令如 trickle 照旧忽略）。

## 3. 验证

- **单测**：es_module 新增 `test_dynamic_import_failure_identity_semantics`
  （四态钉：取回失败 TypeError×新对象 / 图断 TypeError×同对象 / parse
  SyntaxError×同对象 / 求值 Error×同对象+体恰跑一次）+ 既有
  `test_dynamic_import_failure_rejects_not_throws` 升 TypeError 契约；
  v8 352/352、quickjs 8/8（dynamic_import 过滤）；zero-webview 792/792。
- **子树复跑 bitwise**（make 目标 + FILTER）：dynamic-import 子树
  **6P/49F/11T → 28P/27F/11T**；the-script-element 全子树基线复扫
  （stash 后 HEAD 基线 **116P/335F/34T**）vs 修复后 **138P/313F/34T**——
  diff 恰 22 行全 Fail→Pass、全落 dynamic-import/ 目标族，零翻转零误伤，
  T 逐位不变。域账自此按 **139P/313F/34T** 记（s86 117P/335F/34T −22F）。
- fmt clean；clippy `-D warnings` 三 crate（script-sandbox/webview/
  wpt-runner）clean（含 video_registry map_entry 代修，见 §5）。

## 4. 挂账残面（本轮不动）

- **instantiation error 2F**：期望 SyntaxError×新对象（link 期失败不入
  map）；transform 管线编译期即败（`Module not found`→今 TypeError×同/新
  对象随路径）——构造函数仍不符。归 module 基面切片（s86 切片序④，
  instantiation-error 8F 族同源）。
- **credentials 12F**：根非 credentials mode 语义而更前置——classic 脚本
  动态 `import()` 未重写，裸 V8 无宿主回调 reject `Error: Not supported`
  （delay-load-event/propagate-nonce-classic/string-compilation-classic 同
  根）。转 classic-script dynamic import 重写 + `check-cookie.py`/
  `set-cookie.py` handler 建模 + 跨源 cookie jar——下一刀单独成轮。
  **勘误（2026-10-26 s88 当拍实证）**：上句「同根」归因有误——
  dynamic-imports-credentials.sub.html（含 setTimeout 变体）整族走
  iframe + HTTP cookie handler（fetch `set-cookie.py` → Promise.all →
  iframe appendChild → `fetch_tests_from_window`），12F/1T 根因是
  cookie-handler 未建模（M3 多文档面），classic `import()` 重写对其
  **零移动**（s88 复扫 bitwise 不变证）；classic 根实际落在
  propagate-nonce-classic/string-compilation-classic/inline-event-handler。
- **TLA 3 案**（basic/cyclic/tla-deferred `.tentative.html`）：模块体
  top-level await transform 后 eval 语法错——归 module 基面；runner
  「整案抛零声明 → rc=1」报告语义留 FIXME。
- **shared-dependency / string-compilation / nonce / code-cache 等族**：
  非 import 失败面，s86 切片序②③领地。

## 5. 范围与记账

- 生产面三文件（script-sandbox/webview/runner）+ 契约单测；`apps/browser`
  （tab_js_worker）与 renderer 宿主未动——返空契约降级路径行为不变，
  空码标记升级随其面后续切片。
- 共享面碰头记录：`crates/webview/src/video_registry.rs:262`
  `clippy::map_entry`（`9c6652fa3` bg02-media 引入）本流曾代修（entry API
  等价改写）单独成提交；rebase 时实证渲染流已同刻自修（`9d50e4a91`，
  同款 `entry(key).or_insert_with` 改写 + part90 useless_concat）——保留
  上游版，本流重复提交 rebase skip 移除，零净损。
- 语料不可改维持。
