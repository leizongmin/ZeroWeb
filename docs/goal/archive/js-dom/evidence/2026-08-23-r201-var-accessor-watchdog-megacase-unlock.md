# R201 Evidence — strict var accessor 导出 + V8 脚本看门狗：dom/ranges mega-case 家族解锁（M4）

**日期**: 2026-08-23
**切片**: M4——Range-mutations 族 12 文件 "xxxTests is not defined" 整族解锁（0 subtest → 真实混合结果）；**基线大迁移：9767P/100F → 30142P/24395F（polyfill）/ 30144P/24395F（native，fail 清单逐行 IDENTICAL ±2 边缘 Timeout）**；fail 文件集 diff **零新增**（新增 fail 全部落在既存失败文件内——解锁后从 1 行 fail 升级为真实 subtest 结果），5 文件转绿
**改动面**: `script_gen.rs`（var accessor 导出 + strict 判定）+ `webview.rs`（`script_timeout_ms` 看门狗接线）+ `testharness.rs`（runner 设 90s）+ `Makefile`（native 入口 TIME_LIMIT 透传）+ `part21.rs`（单测）

## 一、var accessor 转发导出（script_gen）

strict 间接 eval 同样困住顶层 `var`（R147 function / R198 const-let 的第三族）：
`Range-mutations.js` 的 `var insertDataTests = []` 12 张测试表 + `common.js` 的
七行 `var testDiv, paras,\n    foreignDoc, ...` 跨 `<script>` 不可见。

与 const/let 的**值快照**导出不同，var 有「声明后跨脚本再赋值」流——
`setupRangeTests` 在 harness 回调期赋值、后续脚本读赋值后的值。每名
`Object.defineProperty(globalThis, NAME, {get/set 闭包转发 eval 绑定})`——
get 读当前值、set 写回 eval 绑定（跨脚本赋值也落回）。

**实现细节**：多行 var 语句（行尾 `,` → 续行缩进裸声明符收集；末声明符剥 `;`——
首版漏尾名 `tailName` 由单测抓回）；带初始化首声明符单收首名（accessor 对其同样
成立）。

## 二、strict 判定门（lit 回归教训）

**非 strict 间接 eval 的 var 本就泄漏到 globalThis（数据属性）**——accessor 重定义
使 getter 内 `return NAME` 解析到全局属性 = accessor 自身 → **无限递归 Maximum
call stack**。lit e2e 的非 strict 页面脚本（`var log = []`）当场触发
（template_content_fragment_view NO-REPORT，探针 bisect 到 `var t =
document.createElement` 行）。**门**：源首非空行是 `'use strict'` /
`"use strict"`（带/不带分号四形态）才启用 var accessor 分支。lit e2e 六测试恢复
全绿（复测通过）。

## 三、V8 脚本看门狗（webview + runner）

解锁后 `Range-mutations-insertBefore` 在 JS 层**死循环**（common.js `indexOf` 的
`while (node != node.parentNode.childNodes[i]) i++`——childNodes 视图与
parentNode 失同步，R51b 家族的既存 latent bug；pre-mutation 视图探针全部一致
[paras/foreign/xml/detached 六形态]，自旋发生在测试中段 mutation 之后）。case 级
CASE_TIMEOUT 只在 `run_page_scripts` 返回后 tick——同步 JS 自旋卡死整套 runner
（3600s guard 整轮杀掉丢结果实证）。

修：`WebViewConfig.script_timeout_ms`（默认 0 = 不截断——生产语义不变）→
`ensure_sandbox` 经 `Sandbox::set_timeout_ms` 接 V8 `terminate_execution` 看门狗
（script-sandbox 既有 SEC-13 机制）。testharness runner 设 90s（>
CASE_TIMEOUT_LONG 60s 的 mega-case 正常脚本段）——自旋用例 90s 截断为
`Execution timeout: 90000ms` 单用例 Fail 收场。附带：Makefile
`testharness-dom-native` 硬编码 900s → TIME_LIMIT 透传（30min 套件曾被整轮杀掉）。

## 四、基线大迁移的语义（诚实记录）

- **+20k Pass**：12 个 mega-case 文件 + 既有 1 行 fail 的 Range/Element 大文件
  （Range-set/compareBoundaryPoints/isPointInRange/Element-matches 等原本也是
  "testPoints is not defined" 单行死）解锁后真实跑起数百 subtest/文件
- **+24k Fail**：真实暴露的缺口——两大簇：① **identity 域**（L2 统一域家族）：
  expected（eval 解析的 var 对象）vs got（Range 持有的另一 wrapper 路径对象）
  identity 分歧（Range-set 6767F / compareBoundaryPoints 9313F 的主形态
  `expected object "[object Object]" but got "__n2"`）；② foreign-doc
  CharacterData 方法面（insertData is not a function——foreignDoc.createTextNode
  产物无五方法）
- **fail 文件集 diff 零新增**（python 集合比对 + 逐文件核对；首版 diff 的粒度 bug
  已修正）——无任何之前全绿的文件变红；5 文件转绿（Node-compareDocumentPosition /
  Node-contains / Range-deleteContents / Range-extractContents / TreeWalker）
- **A/B 完美**：native fail 清单与 polyfill **逐行相同**（±2 边缘 Timeout 互换）
- `insertBefore` 自旋本身 = R202 输入（视图失同步定位）；watchdog 保证它不再
  阻塞套件

## 五、验证

- `make testharness-dom TIME_LIMIT=3600`：30142P/24395F/22T（~30min 完成）
- `make testharness-dom-native TIME_LIMIT=3600`：30144P/24395F/20T（fail 逐行同）
- `make test`：全绿除 `window_surface_present_smoke`（XOpenDisplayFailed——clean
  HEAD 复现同败，本 session 显示环境问题，run-rules §10）
- zero-engine 2338 单测全绿（含新 `test_classic_script_strict_var_accessor_export_r201`）；
  lit/vue e2e 全绿；fmt/clippy（v8 + quickjs 矩阵）干净

## 六、commit

`c4cbea15c`（rebase 吸收并行流后落位）
