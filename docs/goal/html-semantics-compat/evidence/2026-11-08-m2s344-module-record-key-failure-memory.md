# M2-S344：module 记录键恢复 + 失败记忆三面（入口键/求值-解析记账/中间错误记账）——module/ 簇 8 案 F→P

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s344 段）。
本切片收口 s343 挂账「module 簇 single-evaluation/evaluation-error/late-*-request 勘向」的
记录语义面：single-evaluation-1/2、slow-cycle、nested-imports、evaluation-error-1/3/4、
choice-of-error-3、compilation-error-1 共 8 案（子测试键）F→P，语料口径 36F→30F。spec 依据：
- module map：同解析 URL = 一份模块记录 = 恰一次求值；parse error 与 evaluation error
  均缓存于记录、后续加载复报**同对象**；链接（instantiation）失败**逐记录独立**不入缓存
  https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
- 中间模块记录同样持有其图内求值错误（choice-of-error-3 判据 log[1] === log[3]）
  https://tc39.es/ecma262/#sec-moduleevaluation
- 元素 load 属 fetch 完成路径而非求值——module map 去重只作用求值，每元素各自 fire
  https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element

## 定谳（本切片新增事实）

1. **内联投递外链 module 的记录键落序号面破恰一次语义**：s94 内联投递后单元以
   `PageScript::InlineModule` 到达 webview，模块缓存键 = `页面URL#!zw-inline-module-序号`
   ——同 src 两元素两键两份记录、模块体重求值（single-evaluation-1 实测体计数 2）。
   修复 = 内联投递单元（`script_csp_info[..].inlined_src` 在案）键恢复 `module_dep_base`
   （原 src 解析 URL）；真作者内联维持序号键（每元素独立记录，s94 定谳不变）。
2. **求值错误零记忆 → 复调重跑体抛新对象**：evaluation-error 族判据每元素一 error 事件
   共同对象——memoized 发射加失败表先行复抛 + 体 try/catch 记忆（原值入
   `__moduleFailures[key]` 后重抛）。
3. **编译失败记账的臂归属（返工定谳）**：compile Err 臂（Rust 变换失败 = 链接面
   「Module does not provide an export」）= **本入口键**缓存、禁跨键扫描——不同根共享
   失败依赖须各自独立失败对象（首版跨键扫描五案回归实证：error-type-2/3、
   choice-of-error-2、instantiation-error-3/5）；同根两元素复报同对象（instantiation-
   error-3 判据 log[0] === log[4]）。跨键扫描的归属臂是 **s102 臂**（包装级 parse）：
   compile 变换为词法面不做全量 parse，坏源码在 V8 parse 期炸包装（error-type-1
   syntaxerror.js 臂既有定谳）——parse error 缓存 + 依赖图扫描（syntaxerror-nested
   的失败源 = 依赖 syntaxerror.js 的记录错误，compilation-error-1 判据 5 元素 3 src
   全同源）。
4. **入口不在 registry → postorder 从入口不可达**：依赖求值序落字典序 leftovers——
   duplicated-imports-1/2 export-default 抢跑面。入口入册（prelude 跳过入口急切求值
   语义不变，R5182）。
5. **spec 中间模块错误缓存 = 祖先记账 + 入口预检短路**：choice-of-error-3 首元素在叶
   依赖 throw2?3d 抛 `{bar}` 中止 prelude，中间模块 3b 自身发射未达 → 只记叶键；第二
   元素（入口 3b）预检落空 → prelude 重跑图外 fresh 模块 throw?3c（log 翻倍 6≠5）。
   修复 = memoized 发射携带入口图 DFS 祖先链（module_graph_postorder 逐模块输出）+
   宿主包装 prelude 前预检（失败表命中即复抛零 prelude 零体；缓存命中跳 prelude——
   入口缓存 ⟺ 入口图后序全部完成）。
6. **成功去重单元元素 load 缺席**：体不再调用 → runner 体尾 load 尾片死代码——load 属
   元素 fetch 完成路径（nested-imports 判据 log=["1","2"] 实测少 "2"）。修复 = 执行前
   缓存探测（命中且非 async 面——s337 微任务臂承担 async、首评单元体尾承担）+ 无哨兵
   按序号派发；真作者内联不适用（load-error-events-inline 判据）。
7. **delayed-modulescript.py 语料端点缺失**（s227 先例：runner 端点臂）：取回 404 →
   error 代 load，nested-imports/slow-cycle/error-and-slow-dependency/import-subgraph-404
   四消费案文件级红与引擎 bug 不可区分。端点臂返回导出 + setTimeout 翻转体。
8. **B 侧域红 solo 绿 flake 一例**：modulepreload-referrerpolicy unsafe-url 子测试域红
   （referrer echo 空串）solo ×3 同 binary 全绿 → 并行负载时序 flake（s292 checkbox
   口径），非本切片回归。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| 记录键恢复 | webview `webview.rs` module_cache_key | InlineModule + inlined_src 在案 → `module_dep_base` 键（外链恢复解析 URL 记录键；真内联序号键不变）；ExternalModule 缺 base 维持 doc_url |
| 求值失败记忆 | script-sandbox `es_module.rs` memoized_dep_expr | 失败表先行复抛同对象 + 体 try/catch 原值入 `__moduleFailures` 重抛；入口体同步记忆化（TLA async 入口维持不记忆化 FIXME(R5182)） |
| 祖先失败记账 | 同上 + module_graph_postorder | postorder 逐模块携带 DFS 祖先键（入口→自身路径）；发射 catch 循环记入全祖先键（未记录时）；余量模块祖先恒空 |
| 入口预检短路 | engine `script_gen.rs` script_module_entry_precheck + webview 包装 | 失败表命中复抛零 prelude 零体；缓存命中跳 prelude（`{`…`}` 收口） |
| 链接失败记账 | webview compile Err 臂 | `script_report_module_failure_cached` 本入口键（同键复报同对象；`dep_keys=[自身]` 退化形） |
| parse 失败记账 + 依赖扫描 | webview s102 臂 + `script_report_module_failure_cached` 扩展 | execute Err（包装级 parse）→ `__moduleFailures` 记账 + 图注册键扫描（字典序定序）：依赖既有失败记录优先复报 |
| 缓存命中 load | webview 执行前探测 + post-execute 序号派发 | `script_module_cache_probe` 探测 + `script_dispatch_script_event_by_ordinal`（非 async 面 + 无哨兵 + 体正常完成） |
| 语料端点 | runner `testharness.rs` | delayed-modulescript.py 臂（`export const a=1` + setTimeout 翻转 `delayedLoaded`） |
| 单测 | script-sandbox 测试模块 | `test_memoized_entry_failure_memory_s344`（入口记忆化形态 + TLA 不记忆化）+ `test_prelude_dep_emission_memoized_and_source_order_s344`（依赖发射统一记忆化 + 源内声明序）+ `test_memoized_expr_replay_same_object_s344`（行为钉：体恰一次/复抛同对象/原值保真）+ `test_memoized_expr_records_ancestor_failures_s344r2`（祖先链发射 + 跨键同对象记账） |

## 验证

- solo（release binary，test-guard + RAF_FRAME_DRIVEN，31 filter 批次三轮 A/B）：
  目标翻绿 = single-evaluation 2P（原 2F）、slow-cycle 1P（原文件级 T）、nested-imports
  1P、choice-of-error-3 P、compilation-error-1 P、evaluation-error-1/3/4 P；回归归零 =
  error-type 3P、instantiation-error 7P/1T（首版跨键扫描五案红，扫描收窄后复原）；
  执行序零漂移 = execorder 8P/evaluation-order 7P/microtasks 11P/charset-01 12P/
  repeated-import 5P/mvs 2P/load-error-events 69P/1T（既有）全保持。
- 单测（test-guard 包裹）：script-sandbox **411P/0F**（+1 s344-r2）、zero-wpt-runner
  **294P/0F**。
- clippy（script-sandbox/engine/webview）净；rustfmt 净。
- 全域 join（`the-script-element` 子域 scoped、子测试键级、stable 集语义——
  `LC_ALL=C` 一刀切；管道级 grep 逐级 `-a`）：
  - 结果口径：1081P/38F/9T → **1088P/32F/8T**（+8P/−6F/−1T）。
  - det F→P=**8** 逐案吻合（上列 8 目标案）；det P→F=**2** = choice-of-error-1
    （s102 扫描归因面，见挂账③）+ modulepreload-referrerpolicy unsafe-url（flake，
    solo ×3 绿）；T −1 = slow-cycle 文件级 T 翻 P。
  - 语料口径（扣自建 probe12/13 恒红 2F）：**36F → 30F**。

## 挂账（承继 + 新记）

1. **live bindings 缺失**（transform 架构快照面）：duplicated-imports-1/2
   `set_foo(43); assert_equals(foo, 43)` 得 42（序面已修，活绑定残红）；
   late-namespace-request `m.foo` 二读 43、late-star-export-request undefined——需导出
   绑定 getter 闭包面（`Object.defineProperty(_exports, …)`），另记 import 声明提升面
   （late-namespace 源内 import 在体尾，发射在语句位 → m 未声明即读）。
2. **循环依赖 TDZ ReferenceError 面**：evaluation-error-2 ill-founded cycle（a↔b let
   互导）transform 读未初始化导出得 undefined 不抛（spec 抛 ReferenceError）——需导出
   绑定 TDZ 建模；缓存命中 load 面已修（got 5 项残红 7 项全靠 exn 三连）。
3. **包装级 parse 原子性（per-record 归因缺失）**：choice-of-error-1（两根不同坏记录
   ?1c/?1d 须各自错误对象）+ compilation-error-2（首元素为嵌套入口，失败应记入依赖
   syntaxerror.js 记录键）——坏依赖源内联致整单元 parse 失败，失败无法按模块记录归属；
   需沙箱 compile-only API（per-record parse 探测）或静态依赖改走宿主取回 eval 面
   （s87 三态记账既有）。
4. **内联 module 错误行号**：evaluation-error-5 判据 lineno 16 得 0——错误事件行列
   传播面（源位 → 错误对象注入）。
5. s341/s342/s343 既有挂账承继：iframe 子文档 currentScript、window.onerror window.event、
   credentials setTimeout 变体、engine fetch 完成驱动 ASAP 臂、renderer 产品路径
   deferred parity、execution-timing 041/056/063/068 文档写入面、fetch-src empty 双案。
