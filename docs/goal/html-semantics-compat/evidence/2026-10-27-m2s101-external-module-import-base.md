# M2 切片 101：外链模块 import 收集基 + 内联模块 spec 重写 + module 图拼写别名

日期：2026-10-27。范围：`crates/webview/src/webview.rs`（ExternalModule 依赖收集基 + module 图钻石依赖补注册）+ `tests/wpt-runner/src/testharness.rs`（内联投递模块体静态 import spec 重写 root-absolute）。上轮：[M2 切片 100](2026-10-26-m2s100-load-event-delivery-fidelity.md)。

## 1. 入场定谳（两层根因）

- 挂账面：nomodule-set-on-external-module-script 1F——「An external module script with nomodule content attribute must run」整页 abort：`page script threw (declared tests: 0): module compile: Module ./cocoa-module.js does not provide an export named default`。
- 根因①（webview）：外链模块静态 import 的收集基 = **页面 URL**；spec 语义 import 相对 **requesting module 自身 URL** 解析（HostResolveImportedModule referrer 为模块自身）。模块体在 `resources/` 子目录时其相对 import 解析到页目录 → 取回 404 → 空存根 → 假编译错（strict 整页 abort）。`<script type="module" src="./resources/exports-cocoa.js">` 内 `import Cocoa from './cocoa-module.js'` 实证。
- 根因②（runner）：runner `inline_local_scripts` 把**外链 module 也内联**（s94 保 type），engine 走 InlineModule 臂——根因①的 ExternalModule 修复对 runner 页面不触发；内联投递后模块原始 src 的解析基准整体丢失。
- **方案取舍**：A) runner 跳过 type="module" 内联（走 ExternalModule 臂）——波及面宽：parser module 臂 load/error 投递依赖 s100 内联投递尾，外置即丢（module/load-error-events 2P 回退）；B) data 属性透传原 src → engine extractor → InlineModule 携 origin——跨 crate 形状变更。取 **E1）内联投递时重写模块体相对 import spec 为 root-absolute**（投递形态适配器职责，同 `.sub.js` 替换/查询串剥离先例），波及面最窄。

## 2. 接线（三刀）

- ①webview ExternalModule 收集基：`module_dep_base` = `resolve_document_url(doc_url, src)`（仅 ExternalModule 臂；InlineModule 维持页面 URL——真内联模块 spec 语义如此），`collect_module_deps_recursive` 首跳以模块自身 URL 为 parent。
- ②runner 内联模块 spec 重写：`rewrite_inline_module_import_specs`——仅 `./`/`../` 相对 spec（已 root-absolute/裸 specifier/含 scheme 不动；裸 specifier 须维持 s87 解析失败语义），以**模块自身 wpt_root 相对目录**为基归一为 root-absolute，lexer 感知提取（`extract_static_module_import_specifiers`）定位后引号包裹整体替换；查询串保留（fetch 通道消费）。动态 import() 走 `__zw_compile_module` 运行时链，不在本重写面（module 尾片挂账）。
- ③webview module 图拼写别名（②连带暴露的共享路径缺陷，旧文档明记「已知限制」）：`collect_module_deps_recursive` 增 `fetched` URL→源缓存——循环防护命中时按**当前拼写补注册**同一源。spec 模型 module map 按 URL 记账、同一模块唯一；本引擎 registry 按导入方体内原文拼写查表（transform_import），旧版仅注册首拼写，第二拼写 lookup 落空 → compile 失败（module/imports.html imports-self-inner 自导入实证：顶层重写拼写先注册，内层体 `./` 拼写循环命中被跳过）。钻石依赖/自导入/跨拼写引用同面受益。

## 3. 验证

- 单测：zero-webview lib **802/802**（+2：收集基钉 `external_module_dep_base_is_module_url_s101`（dep 仅在模块相对位置可达，红→绿）+ 拼写别名钉 `module_graph_spelling_alias_registers_current_spec_s101`）；runner bin **229/229**（+1：helper 四态——相对双引号/`../` 上溯/root-absolute 与裸与 scheme 不动/端到端 module 标签重写而 classic 体不动）。
- 定向：FILTER=nomodule-set-on-external-module → **1F→1P**（两轮：仅①时同错确认根因②、①+②+③ 全绿）；FILTER=module/imports → **5 Pass 0 Fail**（中途态 5P→1F 假红由③修复归位，红态证据在 /tmp/s101-full.log 旧轮）。
- 全子树终扫（scripting-1 口径 578 行）：**326P/220F/14T + 18 NotRun**；diff vs s100 基线（/tmp/n100s.txt 归一）**恰 1 页 1 行**：nomodule-set-on-external-module-script Fail→Pass，**零 P→非P**，零其他行移动（/tmp/s101-rows2.txt）。
- fmt clean；clippy `-D warnings`（webview + wpt-runner，ulimit 24G + timeout shell 兜底，s91 口径）；reftest/product-smoke 未跑（无 render/layout 面）。

## 4. 范围与挂账

- 触碰面：webview.rs（module_dep_base + fetched 缓存/补注册 + 2 单测）、testharness.rs（重写 helper + 调用点 + 1 单测）；shim 零变更；语料不可改维持。
- 挂账承袭：module 尾片（.py 执行器前置——text-module 1F、css-module/json-module 同族、动态 import() 相对模块解析基同属此列）；module 编译错误 error 事件面（nomodule-set-on-external-module-script 同页另一形态——本切片修的「must run」臂之外，compile 失败须 error 事件而非整页 abort 的 strict 行为仍挂账）；116 frameset 文档语义、063 Timeout、charset 12F、svg-script-fragment-clone 3F、FIXME 组、接口对象基建面。
- 附带发现（未处理，非本流引入）：testharness.rs 既有已提交的 `/tmp/zw-hb*.txt` 调试插桩两行（非本会话所加，来源待查）。
