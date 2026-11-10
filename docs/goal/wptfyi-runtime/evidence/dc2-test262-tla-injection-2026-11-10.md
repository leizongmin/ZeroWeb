# DC-2 · test262 TLA 判定单调不减注入（扫描器恒保持＋V8 双腿探针仲裁）

**2026-11-10** · commit `59091ef2e`（`fix(script-sandbox): TLA 判定单调不减注入`）

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

| 面 | 修复前 | 修复后 | 净增 |
|---|---|---|---|
| TLA 面（top-level-await） | 111P/145F | 208P/48F | **+97，翻红 0** |
| module-code 面（含 TLA 子目录，614 键） | 381P/230F | 478P/136F | +97，翻红 0 |
| dynamic-import 面 | 797P/149F | 797P/149F | 逐案零漂移 |
| import.meta 面 | 24P/0F | 24P/0F | 保持 |

净增账 75,307 → **75,404** / 76,000。log：/tmp/round8-tla.log（v1）、
/tmp/round8-tla-v2.log（r1 翻红 47）、/tmp/round8-tla-v3.log（r2 翻红 4）、
/tmp/round8-tla-v4.log（r3 定稿）、/tmp/round8-module-code-v5.log、
/tmp/round8-dynamic-import-v5.log、/tmp/round8-importmeta-v5.log。

## 残余 48F 定性（挂账）

- **for-await-of 形态 19**：`missing ) after argument list`——扫描器 true 已
  async 包装，语法/重写面缺陷（`for await (` 形态处理），下轮定性。
- **export＋块 await 混合 10**：`await is not defined`——探针 None（export 系）
  ＋扫描器漏检（块内 await）的组合面，r3 单调设计不覆盖，需转换层剥
  module-only 语法后探针（后续切片）。
- **语义/时序 ~19**：unreachable/SameValue/hung 等——求值时序与依赖图面。

## 已知限制

- 探针 classic 腿 Sloppy 语义与 module 语义的差分不闭合（`await <模板串/正则>`
  等形态）——r3 设计把它划入扫描器恒保持面，探针只服务裸块/if 块漏检面。
- 无窗路径（native resolver/classic 面与 Plain 形态回调）维持扫描器现状。
