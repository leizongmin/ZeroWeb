# M2 切片 360：import.defer 语法面——transform 重写 + 延迟命名空间门面

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/module/dynamic-import/` 三 tentative 页（s359 挂账残 7F 之 3）：basic-deferred-evaluation / tla-deferred-evaluation / cyclic-tla-defer-eval
**结果**: basic+tla 双案 F→P；cyclic 单案残红为 tc39 proposal-defer-import-eval issue #83 已知循环 TLA 死锁面（bounded Fail 非 hang，出范围记账）；域 join 见下「验证」。

## 勘根定谳

1. **语法面**：tc39 proposal-defer-import-eval——动态形式 `import.defer(spec)` 返回 promise for deferred namespace，静态形式 `import defer * as ns from`；语义 = 依赖图急切求值 + 入口体延迟到命名空间属性访问。
2. **真根因**：bundled rusty_v8 150.2.0 默认关 `--js-import-defer` 旗 → `import.defer` parse 即 SyntaxError「Unexpected identifier 'defer'」→ 三页文件级 Fail。
3. **路线取舍**：Route A（V8 原生旗）放弃——旗默认态不可控 + HostImportModuleDynamicallyCallback 双相宿主回调 API 不确定；选 **Route B（transform-layer 源码重写，全控）**：Module 档 `import.defer(` → `__zw_import_defer.call({referrer, credentials}, `，ClassicDeferOnly 档 → `__zwClassicImportDefer(`（classic `import(` 不重写，走 V8 原生宿主回调 s88 面）。
4. **资源面**：三页引用 resources/ 五件（side-effect-module / module-with-tla-dep / tla-module / cyclic-tla-dep-{a,b}）fetch 白名单漏收新目录 → 404；按 pin 315976933 取件补齐（update-wpt-data.sh 头部挂账）。

## 施工

1. **es_module.rs scanner 双档重构**：`rewrite_dynamic_imports`（Module 档，原签名不变）+ 新 `pub fn rewrite_classic_deferred_imports`（ClassicDeferOnly 档，无 `import.defer` 短路原样返回）；两档共享 `rewrite_imports_impl(source, mode)`，`import.defer(` 识别位（栈空 + TemplateExpr 两处）按 mode 分派重写，eager `import(` 臂加 Module 档守卫（classic 档不动 eager 面）。
2. **es_module.rs deferred split emission**：`build_module_script` 拆 `build_module_script_inner(deferred: bool)`——deferred 时 prologue 后发 `var __zwBodyRan = false;`、export getter 局部名顶层 `var` 预声明、提升 import 原样内联进 `__zwDepsPromise` async fn（enclosing_async=true 归一 TLA await）+ 补 namespace 绑定 getter、体语句剥声明关键词进 `__zwRunBody`（`if (__zwBodyRan) return; __zwBodyRan = true;` 恰一次守卫），收尾 `return { exports: _exports, depsPromise, runBody }`。配套 `compile_dependency_iife_deferred`（dep 镜像）、`namespace_binding_getters`（整行紧凑形态匹配）、`assignment_from_decl`；MODULE_CACHE_RESET_SCRIPT 加 defer 双表复位；单测三个（rewrite 双档 + emission shape）。
3. **part01.js defer runtime**：`__zwDeferRecords`/`__zwDeferFetchKeys` 记账表 + `__zw_load_module_deferred`（fk 缓存 → 五参 `__zw_compile_module(spec, parent, type, cred, 1)` → 四态契约同 eager 镜像：sep===1 charCodeAt(0)===51 → SyntaxError、空串 → 顶层 TypeError、`resolved\u{1f}` → 图 TypeError、`3\u{1f}resolved\u{1f}msg` → 链接逐次新 SyntaxError）+ window-scope 包裹 `__zwRealEval` 豁免 eval + `__zw_make_defer_facade` + `__zw_import_defer`（this.referrer/this.credentials）+ `__zwClassicImportDefer`（parent=`__zwSwapImportReferrer()`、cred=`__zwActiveScriptCred` 委托）。分隔符一律 `\u001f` 转义字面量（裸 0x1f 控制字节已修）。
4. **facade 触发语义**（红态三修正后的定稿）：**仅导出名触发延迟求值**——get trap `if (!(name in record.exports)) return undefined;` 先于 runBody；has/ownKeys 只读导出面不触发。依据：spec 延迟命名空间属性访问触发的是导出绑定求值，非导出属性/内部探查（**await 归一化 thenable 检查读 `.then`**）不触发；导出 getter 已在记录 IIFE 首装（`in` 判定不读 getter 值）。红态实证：无守卫 get 臂在测试首断言前被 promise resolution 的 Get("then") 命中（probe18 stack-dump：`at Object.get (<anonymous>:12311:41)`）→ 体提前跑 → flag 断言 `expected undefined but got true`。
5. **webview.rs**：`__zw_compile_module` 回调第 5 参 defer 标记（`args.get(4)=="1"`，v8_runtime host_callback_invoke args 全部 to_string 数字 1 → "1"）→ `compile_module_contract_deferred` 分支（镜像 eager：import map apply → fetch → compile_dependency_iife_deferred → 同契约形态）；classic lane 跑前 `rewrite_classic_deferred_imports`（strict/non-strict 两臂）。

## 红态轨迹（solo 全程）

1. 重写前：SyntaxError「Unexpected identifier 'defer'」三页齐红。
2. 漏发 `var __zwBodyRan`：ReferenceError。
3. 体序倒置（eager 路径重构引入）：4 既有测试红 `ReferenceError: e is not defined`——修 = eager 统一 body 缓冲（getters → hoisted → body）。
4. `expected undefined but got true`（basic+tla 同签名）：probe18 两轮诊断——record 形态正确（runBody=function/depsPromise=object）→ stack-dump 钉死 Proxy get trap 在 thenable 探查时盲触发 → facade 守卫定稿（施工 4）→ **solo 双绿**。

## Blast radius / 回归定谳

- **eager 面零位移**：rewrite_dynamic_imports Module 档行为不变（新识别位仅在 `import.defer(` 前缀命中时分派）；build_module_script eager 路径仅走缓冲重组（发射序列 getters→hoisted→body 语义不变），sandbox 128 单测含既有 emission 断言全绿。
- **classic 面零位移**：rewrite_classic_deferred_imports 对无 `import.defer` 的源短路原样返回——存量 classic 脚本字节级不变；有 `import.defer` 的 classic 源重写仅命中该前缀。
- **defer 契约与 eager 同镜像**（fetch 失败/链接失败/图失败/求值抛），part01 loader 分支判定与 eager loader 逐臂同构。
- **已知未建模（FIXME 在册）**：mixed eager/defer 同 URL 记录共享（同 URL 先 eager 后 defer 时记录覆盖）；cyclic TLA 死锁面（issue #83）；静态 `import defer *` 声明形式被 strip_import_keyword 误分类（cyclic 页内 dep 链触发，两断言残红）。
- 域 join / clippy / make test 见下。

## 验证

- **solo**：basic-deferred-evaluation + tla-deferred-evaluation 双 Pass；cyclic-tla-defer-eval bounded Fail（`assert_true: module B should have started evaluating ... got false`，非 Timeout 非 hang）。
- **域 join**（A=s359-b 320 键 vs B=s360-b，LC_ALL=C + 按文件名 join）：A-only 恰 1（the-noscript-element/non-html-noscript.html——B 跑偶发缺结果行，非状态翻转，单案重跑 Pass 归因 runner 偶发缺行后补回 join）；B-only 0。**det F→P=2（basic+tla 双目标案）/ P→F=0 / T 双向 ∅，零意外位移**；语料口径 the-script-element **7F→5F、5T 不变**（B 面残 F=143/052 长挂 2 + probe12/13 自建 2 + cyclic 出范围 1；残 T=5 与 s359 同名单）。
- **clippy + fmt**：zero-script-sandbox + zero-webview --all-targets 净（`build_module_script_inner` 8 参按仓库惯例 `#[allow(clippy::too_many_arguments)]` + 理由注释，同 webview.rs 8508 s338 先例）；cargo fmt 全树净（MODULE_CACHE_RESET_SCRIPT 常量拼接与 webview.rs 常量段两处重排，字符串内容零变化）。fmt/allow 后 Rust 面零语义改动——域跑 B 面 release runner 为标注前编译，join 结论不受影响；sandbox 单测 423 全绿（含 s360 三新单测）复跑确认。
- **make test**：3819P/7F EXIT=2，失败集与 s356-s358 基线（并行流组合态既有集）逐名同构（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / test_fetch_filtered_response_type_matrix / test_iframe_service_worker_messages_dispatch_to_iframe_container / test_response_body_readable_stream_r2967 / test_t8k_websocket_error_nonterminal / test_t8k_websocket_event_flow_open_message_close），本流不入侵零新增红。
- **probe 清理**：probe18-defer.html + resources/probe-target.js 跑后已删。

## 挂账 / 下一步

- cyclic 死锁面 + 静态 `import defer *` 声明形式（独立切片候选，tc39 issue #83 语义决策先行）；mixed eager/defer 同 URL 记录共享语义；TLA 入口体延迟属性访问挂起语义。
- 非 webview 宿主（renderer/tab/SW）import.defer 面。
- 存量在册：143/052 流式物化长挂；probe12/13 自建单列；T=5 长尾。

## 日志

- evidence/logs/s360-b-script-element.log（域跑 B 面，join 基线 s359-b-script-element.log）
