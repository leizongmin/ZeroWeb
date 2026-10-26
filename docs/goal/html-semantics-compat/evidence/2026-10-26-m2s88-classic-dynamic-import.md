# M2-s88 施工轮：classic-script dynamic import（V8 原生宿主回调接三态契约）

日期：2026-10-26 ｜ 轮次：s88（施工，s86 切片序①续——classic 面）｜ 前序：[s87](2026-10-25-m2s87-dynamic-import-must-reject.md)

## 1. 入场红图（当拍，HEAD=a2b8e71e8 + FILTER=the-script-element/ 全子树基线复扫）

复扫 **138P/313F/34T**（与 s87 修复后扫描同态；域账面 139P/313F/34T 的
−1P 差为 `v8-code-cache.html` TLA 案页脚抛截断未上报，两轮同缺、对比
同口径）。classic 根 `Error: Not supported` 面当拍细分：

- **propagate-nonce-{inline,external}-classic** 2F：classic 脚本内
  `import("./../imports-a.js")` 裸 V8 无宿主回调 reject
  `Error: Not supported`。
- **string-compilation-classic** 1F+1T：eval 臂 `Error: Not supported`；
  setTimeout/Function/反射 handler/click 四臂 promise 永不落定
  （pending=4 完成回调超时）。
- **inline-event-handler** 1T：内联事件处理器字符串内 `import()` 不落定。
- **string-compilation-{base-url-*,nonce}-classic** 3T：import 解析不落定
  叠加 base URL/CSP 语义缺位。
- **css-module/import-css-module-dynamic** 1F：无 type 的 CSS module
  dynamic `import()` 须 reject TypeError，实际 `Error: Not supported`
  类型不符。
- **非 classic 同根证伪（s87 归因勘误）**：credentials 12F/1T 整族走
  iframe + HTTP cookie handler（fetch `set-cookie.py` → iframe
  appendChild → `fetch_tests_from_window`），与 classic `import()` 无关
  （详见 s87 账本 §4 勘误注）；delay-load-event 1F 须异步取回 +
  `?pipe=trickle` 时序建模，同步 resolve 仍错序。

## 2. 本轮一刀（共享路径）

- **`crates/script-sandbox/src/v8_runtime.rs`**：注册 V8 原生
  `set_host_import_module_dynamically_callback`——source 重写不可达的
  classic 上下文 `import()`（eval 字符串/setTimeout 字符串/Function
  构造器/内联事件处理器，tc39 ImportCall + HTML hostimportmodule-
  dynamically）共享一刃。回调内 `DynamicImportState`（isolate slot 持
  referrer+resolver），三态消费与 `__zw_load_module` 同款契约：
  resolver 返空 → 逐次新 `TypeError: Failed to fetch module: '<spec>'`；
  返 `resolved\x1f` 空码 → `TypeError: Failed to fetch module graph:
  '<resolved>'`；返 `resolved\x1fiife` → 按需安装 module prelude helpers
  （`__zw_dynamic_import` global 缺席时 `build_module_runtime_prelude`
  空注册表最小安装）+ TryCatch 内编译求值，命名空间 resolve / 原异常
  对象 reject。未注册 resolver 的嵌入方保持 V8 默认
  `Error: Not supported`（零行为外溢）。公开面
  `Sandbox::set_dynamic_import_resolver/referrer`（默认 no-op，QuickJS
  降级 classic 动态 import 不支持态不变）。
- **`crates/webview/src/webview.rs`**：`__zw_compile_module` 闭包体提升
  为 `compile_module_contract` 共享函数（R3094 递归 fetch + 三态标记
  原文不动），原生 resolver 注册同一契约；每 classic 脚本执行前刷
  referrer=文档 URL（classic 路径无 ScriptOrigin，宿主供给 base URL）。

FIXME 挂账：classic 上下文重复失败 import 的 `__moduleFailures` 同对象
记账未接（fetch 失败逐次新对象本就是 spec 判据，图断重复导入身份差）；
classic 上下文目标模块体 import.meta 引用无 `_importMeta` 注入。

## 3. 验证

- **单测**：v8_runtime 新增 classic 上下文原生动态 import 五钉——成功
  路径（resolver 收 (spec, referrer)、IIFE 求值命名空间落定、体执行）/
  eval 编译字符串触达 / 取回失败 TypeError 携 spec / 依赖图断 TypeError
  携已解析 URL / 无 resolver 保 V8 默认 `Error: Not supported`。v8
  **357/357**（352+5）、quickjs **255/255**、zero-webview **792/792**；
  fmt clean；clippy `-D warnings` script-sandbox/webview clean。
- **全子树复扫 bitwise**（make 目标 + FILTER，基线/修复后双扫）：
  the-script-element/ **138P/313F/34T → 144P/308F/33T**——diff 恰 6 行
  全红→绿零误伤：propagate-nonce-{inline,external}-classic、
  propagate-nonce-external-module、string-compilation-classic :: eval
  （4×F→P）+ inline-event-handler（T→P）+ **css-module/
  import-css-module-dynamic**（F→P，域外红利——无 type CSS module
  dynamic import 现按契约 reject TypeError）。fetch-error 16P /
  script-error 9P/2F / credentials 12F+1T 逐位不动（credentials 零移动
  反证其 iframe+cookie 根因）。域账自此按 **145P/308F/33T** 记（s87
  139P/313F/34T −5F−1T；账面含 v8-code-cache 惯例同 s87）。
- **运行方式**：make `testharness-html-semantics` + FILTER 透传（test-guard
  `--per-proc-mem 4 --total-mem 8 --time-limit 5400`），TLA 3 案页脚抛
  rc=1 截断两轮同态照记。

## 4. 挂账残面（本轮不动）

- **string-compilation-classic 余 4 臂**（setTimeout/Function/反射
  handler/click）与 nonce-classic/base-url-classic 3T：import 解析已
  通，挂点在 runner 计时器/Function Proxy/事件派发/base URL 建模——
  归 the-script-element 基面（s86 切片序③）。
- **nonce-classic/module 2 案**：CSP nonce 语义（合法 nonce 放行/无
  nonce 拒）引擎未建模——CSP 面。
- **delay-load-event 1F**：动态 import 取回须异步 + `?pipe=trickle`
  时序——runner 管线建模。
- **credentials 12F/1T**：cookie handler + iframe 多文档——M3 面
  （s87 勘误注，非 classic 根）。
- **QuickJS 后端**：classic 动态 import 维持不支持态（trait 默认
  no-op 降级，契约单测钉死 v8-only 面）。

## 5. 范围与记账

- 生产面两 crate 三文件（script-sandbox lib/v8_runtime、webview）+
  契约单测；runner 零改动；apps/browser（tab_js_worker）与 renderer
  宿主未动（未注册 resolver，行为不变）；语料不可改维持。
- `crates/script-sandbox/src/v8_runtime.rs` 本轮后 2929 行（预存超
  2000 行限，本轮前 HEAD 已 2600 行——拆分归基建轮，不随本刀扩scope）。
