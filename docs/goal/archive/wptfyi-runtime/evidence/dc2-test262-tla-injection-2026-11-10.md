# DC-2 · test262 TLA 判定单调不减注入（扫描器恒保持＋V8 双腿探针仲裁）

**2026-11-10** · commit `59091ef2e`（`fix(script-sandbox): TLA 判定单调不减注入`）
＋ commit `5b8080ae2`（`fix(script-sandbox): TLA 深度门只认大括号`，s375 续轮）

## 根因

转换层 is_async 判定的文本扫描器（`module_source_has_top_level_await`）深度门把
裸块 `{` 统计为嵌套表达式——块语句不引入函数边界，块内 await 仍是 TLA。
test262 `top-level-await/syntax/block-await-expr-*` 20 层裸嵌套
`{ { … await 1; … } }` 簇恒 SyntaxError「await is only valid in async functions」
（31 案），if 块/嵌套块同族。TLA 面 111P/145F（module 变体）。

## 机制（commit `741b328ce`）

1. **探针**：`module_tla_probe`（v8_runtime 自由函数，回调窗形态）＋
   `V8Sandbox::module_tla_check_v8`（自持 isolate 形态）——async-function-body 腿
   与 classic-function-body 腿双编译仲裁，TLA ⇔ async 腿过而 classic 腿拒，
   零启发式。async 腿拒返 `None`（含顶层 import/export 声明的源函数体包裹必拒
   ——module-only 语法，s374-r2）。
2. **注入**：es_module `TLA_OVERRIDES` thread_local 表（`set_tla_overrides`/
   `clear_tla_overrides`）——`module_tla_at(key, source)` 查表命中取探针值、未命中
   退扫描器；4 个判定点（deferred 入口 1548/dep Fresh 1877/dep Visited 1889/依赖图
   入口 1799）统一收编。webview `compile_module_contract(_deferred)` Ok 臂在
   parse_gate 同窗（reborrow）内对依赖图全源预判定后 set，转换段结束 clear；
   `ModuleRegistry::iter` 新增遍历 API。deferred 通道 parse_gate 槽位同款扩展。
3. **单调不减设计（s374-r3）**：扫描器命中源**恒保持 true**（v1 行为，零翻红面）；
   仅扫描器未命中源交探针仲裁。教训两轮：
   - r1 纯探针替换：export/import 源错判非 TLA → **47 案翻红**回滚；
   - r2 双腿全量仲裁：`await <模板串/正则>` 的 classic 腿 sloppy 语义误合法
     （tagged template 调用/除法链）→ **4 案翻红**回滚；
   - r3 扫描器 true 恒保持＋探针只仲裁 false 面 → 零翻红。

## 对拍（release runner，test-guard 包裹）

| 面 | 修复前 | r3 后 | s375 后 | 累计净增 |
|---|---|---|---|---|
| TLA 面（top-level-await） | 111P/145F | 208P/48F | 238P/15F | **+127，翻红 0** |
| module-code 面（含 TLA 子目录，614 键） | 381P/230F | 478P/136F | 508P/103F | +127，翻红 0 |
| dynamic-import 面 | 797P/149F | 797P/149F | 797P/149F | 逐案零漂移 |
| import.meta 面 | 24P/0F | 24P/0F | 24P/0F | 逐案一致 |

净增账 75,307 → 75,404（r3）→ **75,434**（s375）/ 76,000。log：/tmp/round8-tla.log
（v1）、/tmp/round8-tla-v2.log（r1 翻红 47）、/tmp/round8-tla-v3.log（r2 翻红 4）、
/tmp/round8-tla-v4.log（r3 定稿）、/tmp/round9-tla-v5.log（s375）、
/tmp/round8-module-code-v5.log、/tmp/round9-module-code-v5.log、
/tmp/round9-dynamic-import-v5.log、/tmp/round9-importmeta-test262-v5.log。

## s375 续轮：深度门只认大括号（commit `5b8080ae2`，+30）

上轮挂账「for-await-of 形态 19」勘误：从 v4 log 提取 `missing )` 簇名单，
实为 `syntax/export-class-decl-await-expr-*` 18 案（`export class C extends
fn(await 1) {}`——类 extends 子句调用实参含 await）＋ `module-import-unwrapped`
1 案（`assert.sameValue(await mod, …)`），即文档化欠检测残面「括号内 await」
＋探针 None（import/export 声明使 async 腿必拒）的组合——非 for-await-of。

根因：扫描器深度门把 `(`/`[` 计入深度，但括号不引入函数作用域（函数体/类体/
方法体全为大括号）；module 保留字语义下括号内 await 只能为 await 表达式，
不存在「标识符 await」误判面。sync IIFE 内 `await` 退化为标识符，
`fn(await 1)` 解析为实参 `await` 后跟 `1` → 「missing ) after argument list」。
修复：深度门只计 `{`；单测翻转钉现状断言并补命中/排除面。

## 残余 15F 定性（挂账）

- **template-literal 3**（block/if-block/try 前缀，`await is not defined`）：
  r2 差分残面——块内 `await \`tpl\`` classic 腿 sloppy 误合法（tagged
  template），探针判 false，运行期报错。
- **语义/时序 12**：defer 传染/cycle/ticks 族（dfs-invariant、pending-async-dep-
  from-cycle、top-level-ticks 等）＋ module-import-rejection/resolution ＋
  flattening-order（import-defer 求值序，实际案源在 `language/import/` 域）——
  V8 模块图求值时序面，与 s361 deferred 传染簇同源。

## 已知限制

- 探针 classic 腿 Sloppy 语义与 module 语义的差分不闭合（`await <模板串/正则>`
  等形态）——r3 设计把它划入扫描器恒保持面，探针只服务裸块/if 块漏检面。
- 无窗路径（native resolver/classic 面与 Plain 形态回调）维持扫描器现状。
