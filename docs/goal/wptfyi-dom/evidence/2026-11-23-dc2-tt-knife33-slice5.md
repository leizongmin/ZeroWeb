# DC-2 切片 5 刀③③a 证据：script sink timer 面（2026-11-23）

## 定谳前置（③a 回退的 11F 机制收口）

③a 恒装 wrap 砸 lib 11F 的机制本轮核清：**worker 域在 part05 以"影子全局"
实现**（data:/blob:/外链 worker 脚本经主文档 `new Function` 壳编译 + 影子
`var self/postMessage/onmessage/importScripts`）——wrap 替换主文档全局
`eval`/`Function`/`setTimeout` 三个共享内建时，worker 壳编译归因位移
（p7 断言 helper.js:2 → helper-main.js:0 正是 `new Function` 壳编译单元）、
影子 string timer 的编译/可见性链断裂。结论：**全局 wrap 一律不装**，TT
检查必须挂在页面 sink 调用时点的 shim 内部。

## 交付面（part01.js 两处，TT 门控）

`setTimeout`/`setInterval` shim 入口（`var handle = _timerId++` 后、s90
string 编译分支前）：enforced 门内 `typeof fn === 'string' || typeof fn
=== 'object'`（TT 实例与 null 都是 object）过 `__zwTtSinkValue('TrustedScript',
fn, 'Window setTimeout'/'Window setInterval')`——管道产出 string 正好落入
既有 s90 编译分支（注册期 `new Function` + s91/s92/s382 捕获槽全保）。非
enforced 页门短路零行为变化。函数参不进门（透传）。

- sink 名 ground truth：语料 `support/block-string-assignment-to-
  DOMWindowTimers-setTimeout-setInterval.js` default policy 回调第 3 参
  断言 `Window setTimeout`/`Window setInterval`（worker 域对应
  WorkerGlobalScope 前缀——影子通道不在 runner 通道，挂账）。
- null 语义：管道通用 String 化（"null"）→ 无政策 TypeError / 有政策
  createScript("null")——语料双臂 ground truth。
- 页级 Timeout 根因（③b② 持位形态）：TT 实例臂 `setTimeout(script)` 在
  无门 shim 里 typeof 非 function/string → no-op → async_test 的 done()
  永不回调 → 页 Timeout。装门后 fire 执行数据串，Timeout 解除。

## 探针（zz-knife33-probe.html，临时件不入库）

enforced 页 10/10 全绿（第二轮；首轮 p07 Fail 为探针自身 bug——policy 名
未用 'default'，③a p21 同款教训，引擎行为正确）：TT 实例注册+fire（含
interval 连发+clear）、plain string/null 双 API TypeError、default policy
sink 名精确、函数参透传（含 fire）。

## 语料对账

- **目标页 block-string-assignment-to-Window-setTimeout-setInterval：
  2P/5F+1TO → 9P/0F 无 TO**——5F 全翻（string/null 双 API throws +
  string via default policy），1TO 解除，2 条假过转真过（sink 名传递/null
  default policy——此前管道未挂时 string 原样执行/no-op 恰好不抛，装门后
  createScript 管道与 sink 断言真正执行）。
- **姊妹页 Window-setTimeout-setInterval：2P+1TO → 6P 无 TO（+4P/-1TO）**
  （assigned via policy / via default policy ×2 API 新翻——基线 TT 实例
  no-op 挂死 async_test 致页 Timeout；"shouldn't go through default
  policy"×2 基线即 P，持位）。
- **TT 语料全量对账：1397P/668F/66T → 1420P/665F/64T**。逐行 comm 对账
  零未知漂移：真收益 **+9P/-5F/-2T**（上述两页），其余 delta 全闭合到
  探针件（zz-knife33-probe 10P 入扫 + ③b② 后建的 probe8/9 2P/2F——
  纯文本片段 2F 为预存 shadow 缺口，与挂账一致）。
- reporting-for-Window-setTimeout-setInterval：Timeout 持位（`pending:4,
  tests:4`——violation 事件面 SecurityPolicyViolationEvent 预存挂账，与本
  刀零交涉：门只改 enforced string 臂，reporting 页 harness 挂点在事件
  回调）。
- ③b① 回归面：set-attributes-require-trusted-types-default-policy /
  no-default-policy 两页 **264P/0F 各页持位**；no-require 页 **202P/0F**
  持位（非 enforced 零变化实证）。

## 回归网

- zero-engine lib unittests **4252P/0F**——比③b② 基线 4249P/3F 多 3P
  且 3 个预存 F（offset_path_m2/min_max_single_arg/numeric_special）转绿
  ＝并行流近期修复的环境面变化（pull 进位的 css-parser MATHFN v3 等与
  CSS 流 WIP 在途态），非本刀效应；本刀对比口径：零新增失败 ✓。
- TT 语料全量两轮对账（见上）✓。
- san 整套件恒等观察：**323P/328F/1T ＝ ③b② 记录值精确恒等** ✓
  （sanitizer 语料与 timer TT 门 enforced 门零交涉实证）。

## 挂账

- **③③b eval/Function 域（机制调研已完成，刀位可行域收窄）**：
  - **直接 eval 臂不可达**——QuickJS 直接 eval（`eval(x)` 标识符直调）是
    编译器特判（字节码层特殊调用序列），覆盖全局 eval 属性拦不到；需 patch
    rquickjs 依赖的 quickjs C 源（[patch.crates-io] 接 fork），超微刀范畴，
    独立评估。
  - **Function 构造器域可收（③③b 主体）**——`new Function` 无编译器特判
    （纯全局属性）：rquickjs 宿主函数 `Function::new` 覆盖全局 Function，
    参数形态须 `Rest<rquickjs::Value>`（现 `Coerced<String>` 在宿主侧已
    ToString 毁 TT 品牌，quickjs_runtime.rs:504 实证）+ Rust 侧调 JS 品牌判
    helper + 保存原内建调用。归因保持预期可行（宿主函数不推 JS 帧）。
  - **间接 eval 可收同型**（`eval?.()` 属性访问调用走全局属性）。
  - 域内页：eval-csp-tt-*（间接臂 2F+直接臂 2P 假过不可收）、
    no-default-policy（间接臂 2F）、no-csp-no-tt（2F）、
    Window-block-eval-function-constructor（Function/AsyncFunction 臂）、
    reporting-for-eval/function-constructor（TO）等。
- worker 影子域 script sink（DedicatedWorker/SharedWorker-setTimeout-
  setInterval 页）：影子 setTimeout 的 TT 门（sink 名 WorkerGlobalScope
  前缀）——影子通道页面在 runner 零输出，随 worker 通道刀位走。
- report-only violation 事件面（SecurityPolicyViolationEvent）：reporting
  页族 Timeout 预存挂账（③b② 同记录）。
