# M2-S339：iframe 域动态 import 模块体 window 身份桥——credentials 簇 12F→10P/2F（承 s338）

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s339 段）。
本切片收口 s338 挂账的根因修复：iframe 子文档内注册的测试体调用动态 import()，
模块体求值落 isolate 全局作用域，`window.<id>` 写错落主窗、iframe 续延读形参恒
undefined（s338 定谳根因链 C2——合并 Function 形参遮蔽 vs 全局作用域模块求值）。
spec：模块的 settings object/realm 归属 importing document。
https://html.spec.whatwg.org/multipage/webappapis.html#module-scripts

## 定谳（本切片新增事实，承 s338 C2）

1. **s261 预内联投递剥 src**：runner 投递给 iframe 子文档的外链脚本是
   `<script data-zw-inlined data-inline="resources/testharness.js">原源码</script>`
   形态——R206 提取循环的 `\bsrc\s*=` 判据不命中，全部落 inline 分支，
   harness 段按 src URL 的锚点检测恒 -1。修法：inline 分支加 `data-inline`
   属性读取作 partSrcs 槽值（extract.rs `inlined_src`/webview.rs 既有双认，
   shim 通道此前未对齐——已沉淀 learning）。
2. **注册 API 裸赋值落宿主全局**：子文档 harness 的 `promise_test` 是 harness
   IIFE 内部绑定，合并 Function 作用域无同名局部——注入包装的裸赋值经作用域链
   落宿主全局（setter 陷阱 + 栈捕获定谳）。与 s147 注记「嵌套 test() 落宿主
   harness」同一承重契约路径；配 pre-call 快照 + fn.call finally 恢复（注册在
   fn 同步窗内完成、体已被包装，恢复不撤已注册面；同路径影子页裸赋值落影子
   局部绑定，恢复为同值回写 no-op）。
3. **桥位窗语义**（两求值臂构建期判定 `globalThis.__zwActiveIframeWin`）：
   native settle 臂同步于 import() 调用窗（corpus 面全走此臂）；prelude eval
   臂经 s337 双跳、构建落在体返回后的微任务（桥位已清）→ 恒原样面（仅同步
   直调形态可达，语义保守）。桥位缺席逐字求值——worker/裸 sandbox 裸 `window`
   仍 ReferenceError，零漂移。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| shim 注入 | `js_dom_shim/part01.js` | `_zwRunIframeScripts`：partSrcs 槽（src/data-inline/''三态）+ `/testharness/` 锚点检测 + harness 段后 splice 注入段（test/async_test/promise_test 包装：体执行窗 try/finally 换入换出 `__zwActiveIframeWin/Doc`，嵌套安全）+ fn.call pre-call 快照/finally 恢复宿主全局三注册名 |
| 模块求值包裹 | `script-sandbox/es_module.rs` | `MODULE_EVAL_WINDOW_SCOPE_OPEN/CLOSE` 常量（形参遮蔽 IIFE，内容无双引号/反斜杠）；prelude eval 臂桥位在场（构建期三目）才夹包裹 |
| native settle 臂 | `script-sandbox/v8_runtime.rs` | `settle_dynamic_import`：Rust 侧读 `__zwActiveIframeWin`（undefined/null 之外为在场），条件推包裹源/原样源 |

不采用（备选否决）：无条件形参遮蔽（裸 `window` 从 ReferenceError 变
globalThis——worker/裸 sandbox 语义面漂移，不可接受）；var 局部绑定承载包装
（hoisted undefined 遮蔽 pre-harness 捕获，`_zwPT` 恒 undefined）；恢复省略
（宿主全局包装跨页泄漏——后续主页面注册体继承 iframe 桥位窗）。

## 验证

- **契约**：sandbox `test_dynamic_import_window_scope_bridge_prelude_s339`
  （同步直调形态：window 写落桥位/globalThis 落主全局/命名空间落缓存；桥位
  清空第二 spec：裸 window ReferenceError 零半写——单 execute 全量，缺省
  persistent_context=false 跨 execute 不持久）+ `test_native_dynamic_import_
  window_scope_bridge_s339`（native 臂：桥位 import 同窗形态全钉 + 逐字臂
  ReferenceError reject 构造器钉）。sandbox lib **406P/0F**（404+2）、runner
  bins **291P/0F**、webview lib **844P/1F**（红 = `proxy_mode_decodes_png_
  correctly` 环境红在案）、engine lib **3811P/6F**（已知六红精确）。
- **clippy/fmt**：`-D warnings`（script-sandbox+engine）全绿；rustfmt 全绿。
- **solo 面**：credentials 簇 **12F → 10P/2F**——2 红恰为 use-credentials
  cross 双腿（Gap-B：iframe 子文档 script 的 crossorigin 不入
  `extract_script_credentials`，s340 候选）；setTimeout 变体文件 file Timeout
  承基线（字符串求值窗仍全局——C3 族，独立面）。
- **the-script-element 域 A/B**：`logs/s339-script-element-after.log`
  **1068P/44F/11T** vs `s338-script-element-after.log` 1058P/54F/11T——全行
  comm（state+case+subtest，LC_ALL=C sort -u）**1122 = 1122**，双向差恰为
  credentials 簇 10 子测 Fail→Pass（**零 P→F**，其余全域字节等价）。

## 挂账

- **s340 候选（Gap-B）**：iframe 子文档 script 元素 `crossorigin` 属性不入
  `extract_script_credentials`（该遍历只走主文档 HTML；子文档 HTML 由 shim JS
  regex 路径解析）——2 个 use-credentials cross 子测（预期 found）不可达。
  修法方向：shim 侧子文档 script 提取循环保留 crossorigin 属性并穿宿主编译
  契约（`__zw_compile_module` 第 4 参已有通道）。
- setTimeout 变体（`window.resolve` 字符串求值窗全局——C3 同族）+ `window.
  onerror` per-fire `window.event` 语义偏差（承 s336/s337/s338）。

## 证据索引

- `logs/s339-credentials-solo.log`——credentials 簇 solo 终态（10P/2F +
  setTimeout 文件 Timeout）
- `logs/s339-script-element-after.log`——the-script-element 域全域终态（A/B
  vs s338）
- `docs/learnings/bugs/2026-11/2026-11-08-iframe-child-doc-inlined-scripts-
  data-inline.md`——data-inline 检测盲区 + setter 陷阱排查法沉淀
