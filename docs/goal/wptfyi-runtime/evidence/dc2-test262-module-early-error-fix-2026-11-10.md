# DC-2 · test262 module 早期错误双通道预检 ＋ import.meta 词法重写修复

**2026-11-10** · commit `42ab0ee46`（`fix(script-sandbox): module goal parse 预检双通道＋import.meta 词法重写`）

## 根因

runner module 变体页（`testharness-test262` 子命令，`build_test262_wrapper`）对
`flags 含 module` 的案发 `<script type="module">` 变体页，体内仅 `import(<绝对路径>)`
动态导入真实用例源。negative phase=parse 的案依赖 V8 对源做 **module goal parse**
报 SyntaxError → import() promise reject → reporter 判 Pass。此前转换层无 goal
预检：parse 放行 → 正文执行 `$DONOTEVALUATE()` marker → 恒 Fail「Expected
SyntaxError but got Test262」。module-code 早期错误簇（dup-export/dup-import/
ambiguous 等 61 目标案）与 import.meta/syntax、assignmenttargettype 负例全在此桶。

第二缺陷：转换层 `str::replace("import.meta", "_importMeta")` 无视词法上下文，
把 runner 页 body **字符串字面量内**的 URL 路径段也改写（import() specifier
`…/language/import.meta/syntax/goal-module.js` → `_importMeta/syntax/…`）→
fetch 404 → import.meta/syntax 正向案假红。

## 机制（commit `42ab0ee46`）

1. **静态面**：`Sandbox::module_parse_check`（lib.rs，V8 feature）分发到
   `V8Sandbox::module_parse_check_v8`——原生 `script_compiler::compile_module`
   子通道，异常即 Err。webview 静态面 specs 提取前 precheck（webview.rs
   validate 块后），Err → `script_report_module_failure_cached` tag3 契约拒绝。
2. **动态面**：`HostCallback` enum 化——`Plain`（原形态）/ `Scoped(Arc<dyn Fn(&mut
   v8::PinScope, &[String]) -> String>)`。Scoped 变体在宿主回调窗内转借 V8
   HandleScope（v8 crate `CallbackScope` 同款「bootstrap scope inside callback」；
   `PinnedRef<TryCatch>` Deref 到 `PinScope`，`compile_module(try_catch,…)` 直接
   传参）。`__zw_compile_module` shim 注册走 `register_scoped_callback`，
   `compile_module_contract` 加 `parse_gate: Option<&mut PinScope>` 参数——resolved
   后对源跑 `module_parse_gate`（自由函数，v8_runtime.rs），命中 → tag3 契约
   `3\x1f<url>\x1f<SyntaxError msg>` reject，import() promise reject SyntaxError。
3. **豁免**：`has_unpublished_static_import`（es_module.rs）——`import defer` /
   `import source` 静态提案形尚无 goal 静态绑定语义（defer 求值延迟），预检放行
   交运行时分类；字节位 `b'i'` 前置守卫保证切片落 UTF-8 字符边界（«» 语料实踩
   panic 修复）；误判方向 = 误豁免（保守）。
4. **import.meta 词法重写**：`rewrite_import_meta_refs`（es_module.rs）镜像
   `rewrite_imports_impl` 的 `RewriteCtx` 状态机——仅代码态（栈空或模板 `${}`
   表达式区）且标识符边界成立时替换为 `_importMeta`；字符串/模板文本段/注释
   原样直通；转义对逐字符拷贝防切多字节。8 处 `replace("import.meta",…)` 调用点
   收编。已知限制同 `rewrite_imports_impl`：正则字面量内引号带入字符串态
   （t8j-r2 轻量取舍，漏替换降级 V8 parse 拒绝）。

## 对拍（release runner，test-guard 包裹）

| 面 | 修复前 | 修复后 | 净增 |
|---|---|---|---|
| module-code（module 变体） | 318P/262F/3T（v1 log） | 381P/230F/3T（v4） | **+63**（救回 45＋新案 18，零翻红） |
| import.meta 面（module 变体） | 7P/17F（DC-1 基线） | 24P/0F | **+17** 全回收 |
| expressions（module 变体） | 421F（DC-1 基线） | 320F（v3 log，10717P） | **+101** 救回，零翻红 |
| dynamic-import 面 | classic 701P/245F（DC-1 基线） | 797P/149F | 优于基线，无回归 |
| WPT import-meta HTML 面 | — | 4/4 Pass | 锁定 |

本轮同通道合计净增 **+181**。log：/tmp/round7-module-code.log（v1）、
/tmp/round7-module-code-v3.log（expressions 同轮）、/tmp/round7-module-code-v4.log、
/tmp/round7-importmeta.log、/tmp/round7-dynamic-import-v4.log、
/tmp/round7-wpt-importmeta2.log。

## 剩余与挂账

- module early-error 132 簇中 escaped-await IIFE 降级输入等**输入形态残余**
  未回收（runner 包装形态不表达的案，属探针结构性残余，非引擎差异）。
- classic 面负例已由 IIFE classic 编译面覆盖（compile_module_contract native
  resolver 调用点 parse_gate=None，s373 注释）。
- 模板/正则上下文为轻量词法扫描，非全量 JS 词法器——与既有
  `rewrite_imports_impl` 同款限制。
