# M2-s6 — corpus 第二批扩批 + eval 门禁（全绿 65→71，分母 415→445）

**日期**: 2026-09-25
**corpus**: `make testharness-csp`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-25-m2-s6-csp-final.json](2026-09-25-m2-s6-csp-final.json)
**前序**: [M2-s5](2026-09-25-m2-s5-csp-extensions.md)（65/415，28.9%）

## 结果

| 指标 | M2-s5 | M2-s6 | Δ |
|---|---|---|---|
| 执行用例 | 415 | **445**（+30：第二批扩批） | 分母扩张 |
| 全绿用例 | 65/415 | **71/445** | **+7 净 / −1（event-handler 案，见归因）** |
| subtests Pass | 165/570 = 28.9% | **169/604 = 28.0%** | +4（百分比被分母扩张稀释，绝对值 +4） |

新增全绿 7：unsafe-eval 簇 5（eval-blocked / eval-blocked-and-sends-report /
eval-allowed / function-constructor-blocked / function-constructor-allowed——**eval
门禁全簇翻绿**）+ script-src 1_4 / 1_4_2（eval 阻止语义正确化）。−1：
script-src-event-handler-on-inline-script（unsafe-hashes 放行的 onclick 处理器被
Function 代理误拦——shim 处理器求值与 eval 语义合流，归 M2-s7 attr 面同修）。

第二批新目录基线：inheritance 0/17（政策继承跨文档面）、sandbox 0/3（sandbox 指令
——iframe 面）、unsafe-eval 5/10、navigation/wasm-unsafe-eval 0 案过筛（worker/
导航执行面挂账）。

## 本切片落地（kill-switch 延续，default off）

1. **corpus 第二批扩批**（fetch 脚本 + `CSP_CORPUS_SUBDIRS` +5 目录）：inheritance/
   navigation/sandbox/unsafe-eval/wasm-unsafe-eval（90 html 落 wpt-data）。
2. **eval 门禁**（zero-security `eval_violation` 多政策并集 + webview shim 层）：
   v8 crate 无 codegen 写入面（仅 `is_code_generation_from_strings_allowed` 只读）——
   **shim 层 eval/Function Proxy 包装** + `__zwCspEvalBlocked` 原生回调（违例入
   connect 共享队列，run 尾 document 站派发，blockedURI="eval"、指令面无 -elem 子分）。
3. **per-script 作用域**（关键设计）：持久包装会折断 harness/shim 装配链（shim init
   自身经 new Function）——包装在**非 harness 脚本执行前后装/卸**（harness 豁免同款
   判定）；页面脚本执行机制 `script_run_classic_page` 改
   `(0,globalThis.__zwRealEval||eval)`——机制自身走包装安装时捕获的原生 eval，页面
   发起的 eval 仍被拦。

### 探针闭环（防复踩）

- 持久包装首跑 −34：机制自身 `(0,eval)` 被自家包装拦截 → 机制侧 `__zwRealEval` 优先
  + per-script 装卸两段修复。
- Function Proxy 前置安装折断 shim init（shim init 经 new Function）→ 后置到
  ensure_js_shim 之后。
- make test navigator_skip_waiting 一例红×2 → 隔离三连跑 ok/FAILED/ok 确证固有
  flake（R4740/R4743 已归因记录的同族，负载敏感 60s deadline）——全量复跑 exit 0。

## 剩余缺口（M2-s7+）

| 簇 | 形态 | 切片 |
|---|---|---|
| script-src-attr | onclick 处理器门禁（shim 求值走 __zwRealFunction + attr 语义检查）+ 修 −1 回退案 | M2-s7 |
| 运行时 img | createElement('img') src-set 钩子 | M2-s7 |
| inheritance/sandbox 簇 | 跨文档/iframe 面（inheritance 17 案、sandbox 3 案全红——重入条件 = 跨文档导航管道） | 记账 |
| eval 位置断言 | blockeduri-eval 15:13（eval 调用点定位） | M2-s7 |
