# M2-S348：模块包装级 parse 失败记录归属——choice-of-error-1 / compilation-error-2 双案 F→P

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（头部状态链 s348 段）。
本切片收口 s345/s347 承继挂账「parse 原子性双案（勘 compile-only API 可行性优先）」：
- choice-of-error-1（循环双根 1a↔1b，各带 query 区分的 syntaxerror 依赖 ?1c/?1d）：
  红态两根报告**同对象**（红勘 solo：`assert_not_equals: two different parse errors
  should be reported got disallowed value object "SyntaxError: Compile error:
  SyntaxError: Unexpected token '%'"`；前序断言 length==4 / 构造器 / onload 1、2 全
  过——唯对象恒等翻红）。
- compilation-error-2（5 元素 nested/nested/direct/nested/direct）：
  `assert_true: expected true got false`——`log.every(exn => exn === log[0])` 不成立，
  nested 首元素与后续直连元素各持一对象。域口径 the-script-element **23F→21F**，
  det F→P=2、P→F=∅、T=8 同集零位移。spec 依据：
- fetch a single module script（parse error 缓存于模块记录/module map）
  https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
- run a module script（失败报告面，s102/s255 既有）
  https://html.spec.whatwg.org/multipage/webappapis.html#run-a-module-script

## 定谳

1. **旧口径两处失真，同一根因**：s102 臂（execute Err=包装级 parse）新建记录恒落
   **入口键** + 复报走字典序 dep-scan。compilation-error-2：nested 首元素无既有记录
   可扫 → 新记录挂 nested 键；后续直连 syntaxerror.js 元素自身键无记录 → 复建第二
   对象（every 红）。choice-of-error-1：根 1a 失败记录挂 1a 键；根 1b 的图**循环包含
   1a** → dep-scan 撞见同对象复报（assert_not_equals 红）。字典序扫描无求值序概念，
   多失败图必然跨根串扰。
2. **spec 归属 = 失败模块自身记录**：parse error 属该模块的 module record（module
   map 失败记账）；多失败图各根按**各自 DFS 求值序**报告不同记录的错误（1a 序 =
   [?1d, 1b, ?1c, 1a] 首失败 ?1d；1b 序 = [?1c, 1a, ?1d, 1b] 首失败 ?1c——两对象
   天然互异）。归属序 = `module_graph_postorder`（s255 求值序既有机制）原样复用，
   首个 parse 失败键即记录属主，不再发明扫描。
3. **compile-only 面 = `Sandbox::parse_check`**：V8 后端实现（同 context、同
   classic `Script::compile` 编译面，无 run/watchdog/微任务/回调挂载——零副作用）。
   校验对象为**转换单元**（`compile_dependency_iife` 输出，与装配脚本同 classic
   语法面；raw registry 源含 import/export 不可 classic 编译）。QuickJS 后端默认
   EngineUnavailable → 调用方 trivial 编译 pre-flight 探测 → 整扫描跳过落旧口径
   （车道行为与 s344 基线逐位一致；FIXME 在 trait 默认实现注记）。
4. **parse 优先于既有记录**：spec 图内 parse 失败在求值前的 instantiate 阶段暴露，
   求值错误记录（s344 求值记账含祖先链）不得抢位——逐键先 parse-check，体合法而键
   带既有记录才作回落候选（求值序首键）；回落语义 = 旧 dep-scan 的有序版（非 parse
   失败面）。空源预存根与 build Err（解析/链接失败类）跳过，归旧臂。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| compile-only trait 面 | script-sandbox `lib.rs` `Sandbox::parse_check` | 默认 EngineUnavailable（调用方回落旧臂；QuickJS FIXME 注记）；V8 委托 [`V8Sandbox::parse_check`] |
| V8 parse_check | script-sandbox `v8_runtime.rs` | 编译丢弃产物（无 run）；单测 `parse_check_compiles_without_execution`（合法 Ok / 语法错 CompileError / 副作用零求值 / 空源 Ok） |
| 归属扫描 | script-sandbox `es_module.rs` `module_parse_failure_owner` | postorder 逐键：parse-check 失败即返属主键；既有记录回落候选；None=扫描不达。单测 `test_module_parse_failure_owner_postorder_s348`（三臂） |
| s102 臂接入 | webview `webview.rs` | registry/url 提升至元素循环域；失败路径 trivial 探测 → `__moduleFailures` 键集快照（execute_json）→ 归属扫描 → 按属主键记账报告（复用旧报告面缓存/新建语义）；None 落旧口径原样 |

## 验证

- solo：choice-of-error-1 **F→P**、compilation-error-2 **F→P**（`s348-solo-choice1.log`
  / `s348-solo-comp2.log`；红态证据 = s347 域跑 `s347-b-script-element.log` Fail 行
  + 本切片首勘 solo assert_not_equals 详情）。
- 域 join（the-script-element 子域、键级 stable 集语义——`LC_ALL=C`、`grep -a`、
  (key,state) `sort -u`、双态键剔除）：A = `s347-b-script-element.log` 23F；
  B = `s348-b-script-element.log` **21F**。det F→P=**2**（双案逐键吻合）；
  det P→F=**∅**；T=8 同集零位移；F 集差 = 恰去双案、无新增 F 键（comm 实证）。
  语料口径 **23F→21F**。
- 单测（test-guard 包裹）：script-sandbox **419P/0F**（含 2 个新增单测）；
  runner **298P/0F**（s347 基线持平）。
- webview **843P/2F**——`image_decoder::tests::proxy_mode_decodes_png_correctly`
  既有 env 红在册；`tests::csp_gate::csp_connect_gate_blocks_fetch_sh1_m2s5`
  **HEAD stash 同红实证**（fetch 拒绝语义 `rejected` ≠ 契约 `resolved:ok=false:status=0`
  ——新碰头账，非本切片引入，挂账 1）。
- clippy（zero-script-sandbox + zero-webview --all-targets）**0 告警**；rustfmt 净。
- 本切片无既有测试断言变更；`parse_check` 默认实现兜底全部既有 Sandbox 实现
  （engine 4 个测试 CaptureCb stub 经默认面零触碰，cargo check 全绿实证）。
- 归属扫描为**失败路径专用**（s102 臂 execute Err 后）——成功路径零新增成本，
  无热路径影响（#23 bench-gate 不适用）。

## 挂账（承继 + 更新）

1. **webview 新增 1 案环境依赖红**：`csp_connect_gate_blocks_fetch_sh1_m2s5`
   （HEAD stash A/B 同红；solo 与全套一致红）——与 image_decoder env 红同性质
   碰头账，归属 M3 收敛流（46号）；另仓内遗留 `stash@{0}`（s345 期 WIP on
   f9cb4db3a）在册只提及，不擅动。
2. **QuickJS parse_check 接线**（FIXME 在 trait 默认实现）：quickjs eval 无独立
   parse 面，接线前 QuickJS 车道 parse 失败面维持 s344 基线行为（双案仍红，V8
   为 runner 主车道）。
3. **跨元素极端序列的对象身份面**：归属扫描逐元素 walk 建账（首失败键先记），
   非「全图 fetch 期一次性建账」——第三元素直连此前未记键的非首失败模块时新建
   对象（spec 应复用 fetch 期记录错误），语料无钉面 FIXME（es_module 扫描 doc
   注记）。
4. s345/s346/s347 既有承继原样：外链模块求值错误行号与投递内联行号 0（语料无
   钉面）、动态内联行号未建模、engine 6 案 M3 回归修复（t8k websocket 对 +
   img/svg/fetch/iframe SW）、runner 定向 clippy 29 既有债、s346 join 键=原始 src
   FIXME、getter 安装失败回落、同名多星号首见 FIXME、域残余 **21F** 在册面。
5. 下一步候选：域残余 21F 逐案勘根（T=8 长尾在册：execution-timing 063/fetch-src
   empty 双案/load-error-events-3/credentials setTimeout 变体/code-cache-base-url/
   instantiation-error-8/script-crossorigin-network）；engine 6 案回归修复按 46号
   M3 收敛插队。
