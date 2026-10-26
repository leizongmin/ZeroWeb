# M2 切片 105：import attributes 静态校验——属性键 SyntaxError 先行 + type 值 TypeError

日期：2026-10-27。范围：`crates/script-sandbox/src/import_attributes.rs`（新模块）+ `es_module.rs`（5 扫描件提 `pub(crate)`）+ `lib.rs`（注册）+ `crates/webview/src/webview.rs`（is_module 臂接线 + 2 回归钉）。上轮：[M2 切片 104](2026-10-27-m2s104-frameset-document-body.md)。

## 1. 入场定谳

- s105 探针三候选：①execution-timing 50F 异质（scheduler 排序长尾 + SVG integration-point 解析器面 + 138/139/143 疑似变换自伤），非单根因切片，否；②063 = runner 脚本记账深坑（cloneNode deep 重复执行 + pending=1），否；③**import attributes 静态校验** 3 Fail 页，最小高信，定刀。
- 判据（语料原文核对）：invalid-type-attribute-error（4 外链模块 `type:` = notARealType/""/js/javascript → window error `ev.error.constructor` 全 TypeError，`onerror=unreachable` 即元素须 fire load 不 fire error）；unsupported-attribute（1 外链模块未知 key → SyntaxError）；invalid-import-errors-order（3 内联模块：`import "INVALID" with { unknown: "foo" }` / `import "./valid" with { unknown, type: "unknown" }` / `import "./valid" with { unknown }` + `import "INVALID";` → 全 SyntaxError——属性键错误先于同句/后句 specifier 解析错误与 type 值错误）。
- type 值合法性 = HTML module type map：不注册 "js"/"javascript"（JS 模块以无属性为默认形态），语料四值全 TypeError 与此一致；合法面 {"json","css","text"}（数据模块取回/MIME 面归 css/json/text-module 切片，本轮不触）。
- 现成机制复用：管线已容忍 with 子句（extractor `extract_import_specifier_from_rest` 取首个 `;` 前字符串字面量，empty-attributes-clause.html 本已 Pass）；报告面 s102 `script_report_module_failure(msg, ordinal, error_ctor)`（window 报实例 + 元素 fire load）与体跳过 s103 口径（continue 连 prelude 预热一并跳）全现成。

## 2. 接线

- 新模块 `import_attributes.rs`：`validate_static_import_attributes(source)` 逐静态 import 语句（复用 es_module 的 `split_statements`/`strip_import_keyword`/`import_clause_shape_ok`/`split_from_clause`/`extract_string_literal`，提 `pub(crate)`）——定位 specifier 字面量后探测 `with {…}` 子句：任一属性键 ≠ `type` → SyntaxError（键校验全量先于 type 值校验）；type 值 ∉ map → TypeError。块未闭合/值非字符串字面量 → SyntaxError（提案语法静态语义）。 FIXME: `export … from` 属性子句与动态 `import()` with 选项未覆盖（语料未及）。
- webview is_module 臂顶部（图收集前）接线：违例 → `script_report_module_failure` 报对应构造器实例 + `continue`（体不执行含 prelude，s103 同口径）。校验先于图收集保证属性键 SyntaxError 胜过裸 specifier 的 DepResolution TypeError。合法属性零行为变化（仍走图收集 → 数据模块 MIME 面维持现状）。
- 放新文件不进 es_module.rs：后者已 3470 行超 2000 行准则，新代码按职责落新单元。
- **module type map 首版漏 "text" 扰动实录**：首版 map 仅 {"json","css"}，一轮全扫 text-module 域 5 页翻转（integrity/load-error-events P→非P、module.html F→NotRun、http-accept-header F→T）——语料 text-module 族 32 处 `type: "text"`（grep 统计）被误判 TypeError。收录 "text" 后定向 FILTER=text-module 与 s104 基线零 diff，扰动完全愈合。教训：module type map 的合法面以**全语料属性值分布统计**定界，不凭 spec 印象。
- 测试坑：webview 回归钉首版用 `import BAD with {…}` 形态（default binding 无 from 子句——非法 JS，`import_clause_shape_ok` 正确拒绝致无报告），改裸字符串 `import "BAD" with {…}`（corpus 同形）后过。

## 3. 验证

- 单测：script-sandbox lib **372/372**（+7：四值 TypeError/未知 key/三序判据/合法面含 text/散文与动态形态防误扫/压缩形态/块语法错）；zero-webview lib **809/809**（+2 回归钉：非法 type 值 TypeError+体跳过+元素 load；未知 key 胜 bare specifier 的 TypeError 序判据）。fmt clean；clippy 双 crate `-D warnings` clean（补结构体字段文档 2 处）。
- 定向：**FILTER=import-attributes 4/4 全 Pass**（3 目标页 F→P + empty-attributes-clause 保持 P）；修复后 **FILTER=text-module 与 s104 基线零 diff**。
- 全子树终扫（scripting-1 口径 593 行）：**352P/189F/37T + 15 NotRun**（s104：349P/192F/37T+15N）；行级 diff **恰 3 行**（3 目标页 F→P），**零 P→非P**（comm 校验）、T/N 计数与基线持平、零旁损。
- 渲染面核对：本轮 JS 静态校验 + webview 编译臂，无布局/绘制面，reftest/product-smoke 未跑。

## 4. 挂账

- import attributes 余项：数据模块全量支持（fetcher MIME 门 testharness.rs:4904-4964 + extractor 类型捕获 + registry 合成源 + rewriter，跨 script-sandbox/webview/runner 大切片）；`export … from` 属性子句与动态 `import()` with 选项；依赖模块（dep-level）的属性校验。
- 承袭挂账不变：063 Timeout、charset 族、svg-script-fragment-clone 3F、FIXME 组、接口对象基建面、execution-timing scheduler 族 50F、testharness.rs /tmp/zw-hb*.txt 插桩来源待查。
