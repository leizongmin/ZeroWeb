# M2 s253 — CSP eval 门禁互斥模块编译通道（1 键 F→P）

日期：2026-11-03。切片号 s253（承 s252 候选 #1）。s252 原候选为
propagate-nonce ×1 + delay-load ×1 双键；勘根后**收窄为 nonce 单键**
——delay-load 根因定谳为动态 import 同步取回冻结回合，修复需异步通
道（契约第五态 + 完成泵），独立切片量级，改挂账（见挂账更新）。

## 勘根（三探针实证三通道分流）

solo 过滤 `dynamic-import/propagate-nonce` 四页（同 CSP
`script-src 'nonce-correct'` 无 unsafe-eval），webview 侧三处临时
eprintln 探针（包装安装点 idx/active/module/nonce + eval-blocked 回
调 + v8 原生 resolver）取证（evidence/logs/s253-probe.log）：

- **propagate-nonce-inline-module（Fail 键）**：内联 module idx=1
  `active=true`——包装在场。转换后模块体 `import()` 重写为 shim 通道
  （`__zw_dynamic_import` → prelude `__zw_load_module`），其编译 eval
  （`eval('(' + code + ')')`）在触发脚本执行窗内同步运行 → 被自家
  eval 包装拦截 → `Unhandled rejection "EvalError: call to eval()
  blocked by CSP"`。
- **propagate-nonce-external-module（既 Pass——机制查明为「错进错
  出」）**：runner 外链脚本内联投递统一打 `data-zw-harness`
  （testharness.rs 投递形态，s92 注记在案）→ 引擎侧
  `script_csp_info[1].zw_harness=true` → **包装根本不装**（探针
  `active=false`）——页脚本整体豁免 eval 门禁而非豁免正确。
- **propagate-nonce-{inline,external}-classic（既 Pass）**：classic
  体不转换，真 `import()` 走 v8 原生通道（HostImportModuleDynamically
  → Rust 侧 `v8::Script::compile`）——JS 包装不可见，探针 resolve 命
  中且无 eval-blocked。
- **spec 口径**：module evaluation 不是 eval()——CSP `script-src` 管
  模块 fetch（nonce 检查已在宿主 resolver 完成，s92 通道），不管求值
  机制；eval 门禁针对的是页面字符串编译 API（eval/Function）。包装拦
  截内部模块求值属误伤。
  https://w3c.github.io/webappsec-csp/#can-compile-strings

## 施工（prelude 一行窄豁免）

`build_module_runtime_prelude` 的 `__zw_load_module` 编译 eval 换为
包装感知形：

```js
(typeof __zwRealEval === 'function' ? __zwRealEval : eval)('(' + code + ')')
```

- `__zwRealEval` = webview M2-s6 包装安装点保存的真实 eval。包装在场
  → 经真实 eval 求值（仅此编译调用豁免）；未装包装（renderer/tab 宿
  主、无 CSP 页）→ 回落 `eval` 同形，零行为变化。
- **窄豁免三层语义**：①模块体求值照常落地；②模块体内嵌套 eval()/
  Function 仍解析到 `globalThis.eval`（包装）照拦——页面级 eval 判据
  （string-compilation-* 族）不受扰；③直呼→间接调用对括号包裹 IIFE
  表达式语义等价（无 var 泄漏面，s2081 注记同形）。
- s250 动态内联 module 臂（part03.js eval 通道）同暴露面未动——不在
  本切目标路径，挂账记同类。

## 验证

- **红态实证**：新钉 `test_dynamic_import_eval_gate_exemption_s253`
  （script-sandbox——复刻 webview 包装 eval 半边 + `__zwCspEvalBlocked`
  JS 桩，三层断言：模块编译 eval 豁免 / 模块体内嵌套 eval 照拦 / 页
  面 eval 照拦）。旧 prelude 行下红：
  `rejected:EvalError:EvalError: call to eval() blocked by CSP`（与
  WPT 页失败签名同形）；修复行绿。首轮测试自身时序 bug（`__zw_report`
  同步跑在微任务检查点前，import `.then` 未落定）——按 s87 形挂进
  promise 链尾后修正。
- 单测：script-sandbox **380 全绿**（379 + 新钉）+ webview **835 全
  绿**（prelude 波及面零回归）。fmt/clippy 干净（warning 全在
  video_registry 既有面）。
- solo（FILTER=dynamic-import/propagate-nonce，evidence/logs/
  s253-solo-nonce.log）：四页全 Pass——目标键 F→P，三既 Pass 页维持。
- 全树 A/B（evidence/logs/s253-ab.log vs s252-ab.log）：行级
  **7782P/447F/77T 双侧同数**（目标 F→P 与漂移族 P→F 一对冲）；键级
  join **8306/8306 对全闭合，迁移恰 2 键**——目标键
  propagate-nonce-inline-module **F→P** + checkable-active-onblur-
  with-click 1 子测试 **P→F**（同跑同文件 3F+4P 混态自证，
  s238/s239/s245/s246/s248 在册 checkbox `:active` 双向漂移族——本轮
  P→F 方向，先例销账记基线态，通道无涉：prelude eval 现场与 CSS
  `:active` 交互面无通道）。

## 挂账更新

- **delay-load ×1 根因定谳（勘根收口改挂账）**：`__zw_load_module`
  的宿主 resolver 链在脚本执行窗内**同步阻塞取回**（
  script_source_fetcher 同步闭包）——slow-module `?pipe=trickle(d2)`
  冻结 JS 回合 2 秒，'slow' 在脚本执行内推入，window load 派发（脚本
  循环后）与 1s step_timeout（回合后宏任务）全排在断言之后（got
  `["import start","slow"]`，缺 load 与 loading 两环）。修复需动态
  import 异步通道：契约 pending 态 + 完成泵（frame loop 挂钩）或 v8
  原生通道扩展——中大型刀位，独立切片。
- **data-zw-harness 投递标记保真面（新挂账）**：runner 外链脚本内联
  投递统一打 `data-zw-harness` → CSP 页所有外链页脚本整体豁免 eval
  门禁（本切 external-module 页 Pass 的真实机制）。翻案此标记（投递
  形态区分 runner 基建 vs 页面脚本）会波及全部 CSP 页外链脚本面，需
  独立评估；本切修复后该页经豁免通道同样 Pass，无即时行为差。
- s250 动态内联 module 臂 eval 通道同暴露面（CSP 页动态插入内联
  module）记同类待勘。

## 下一步候选

1. importmap ×4（`_importMeta.resolve` 特性面——import map 解析已在
   `resolve_registered_specifier` 就位，补 JS API 面）。
2. instantiation-error-6/7 星号导出歧义检测（transform 管线内
   export * 链歧义归并 + blame 定位，中刀位）。
3. 动态 import 异步通道（delay-load 根因切片；顺带覆盖 trickle 类
   慢模块时序族）。
