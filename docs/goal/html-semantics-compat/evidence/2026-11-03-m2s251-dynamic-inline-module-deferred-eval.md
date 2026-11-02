# M2 s251 — 动态内联 module 延迟求值（5 键 N→P，第十个连续完美 A/B）+ s250 执行上下文归因修正

日期：2026-11-03。切片号 s251（承 s250）。根：s250 下一步候选
#1「动态内联 module host 侧执行通道（沙盒重入 API 设计）」——
勘根**推翻前提**，收口不需要任何沙盒 API，一个求值时序修复替
代整案。

## 入场定刀位

s250 残余挂账：`inline, 200, not parser-inserted` ×3（text/json/
css-module 三页）NotRun，归因「严格页面 var 在 shim eval 上下文
不可见（8 轮探针定谳）」，修复方向预设 host 侧执行编译体。本切
按候选先勘 v8 沙盒重入面（host_callback_invoke 持有活动 scope、
register_callback 线程局部槽位制、resolve_async_callback host 侧
execute 先例），尚未设计 API 即转入先验证前提。

## 勘根（前提推翻 + 机制定位）

- **探针定谳可见性全通（推翻 s250 归因）**：最小探针页（动态插
  入内联 type=module，体 = 静态 `import "./text-module/file.txt"
  with { type: "text" };` + `onExecute(pv)`，pv 为前导脚本严格
  `var`）三测全 Pass——体求值、页面 var 解析、onExecute 完整
  done 链全通。可见性机制：页面 classic 脚本经
  `script_run_classic_page` 间接 eval 执行，strict 顶层 `var` 由
  `scan_top_level_decl_exports` 导出 **globalThis accessor**
  （getter/setter 闭包持有 eval 局部绑定）——任何后续 eval 经全
  属性查找即可达。s250「replay 作用域不可见」结论作废（当时 8
  探针未覆盖「同脚本声明 + 同脚本插入」形态）。
- **复现真根因（求值时机竞态）**：同脚本形态探针（`var pv_same
  = event_test(...)` 与 `appendChild` 同一 `<script>` 内）→
  NotRun 复现。机制：s109 内联 module 臂在 `appendChild` 调用内
  **同步**编译+eval 模块体，此时插入脚本自身的 eval 未结束——
  accessor 导出后缀（eval 源尾拼接）**尚未执行**，`pv_same` 还
  不是 global 属性 → 体引用 ReferenceError → catch 上报 →
  onExecute 不达 → 测试 pending。spec 语义：动态插入模块的
  evaluate 排在插入脚本完成之后（microtask），真实浏览器同脚
  var 引用无此竞态。
- **探针语料路径坑（两踩，与 s250 同型）**：探针页放
  `the-script-element/` 根时 `../resources/` 上溯出域到
  `scripting-1/resources/`（helpers 404 → event_test 未定义假
  象）；`./file.txt` 相对探针页位置出域（404 → 编译失败 error
  事件假象）。探针页须与目标族同目录放置，或显式修正相对路径
  （`./resources/`、`./text-module/file.txt`）。

## 施工（shim 1 处）

- **engine part03.js s109 内联 module 分派臂**：编译+四态状态机
  +求值整体挪入 `setTimeout(0)`（src 路径 rev3 同型；spec
  evaluate 后置的 macro-task 近似）。保持插入时刻同步的部分：体
  文本融合读 + 空体早退门（145 形空脚本 textContent= 臂依赖）+
  run-once 记账（spec already-started prepare 置位属同步段）。
  404 臂（依赖图取回失败 → error）既有延迟事件派发嵌套保持——
  派发 tick 不变（编译与派发同落 tick 1），已翻绿臂零时序漂移。
  延迟闭包捕获 `_miText250`/`_miEl250`（插入时刻快照，后续文本
  变更不回溯）；`_miBase250` 按钮内捕获（与 src 臂 rev3 一致）。
  classic 内联臂同步 eval 不动（spec classic 同步执行语义，124
  形期望序依赖）。

## 验证

- solo（FILTER=scripting-1/the-script-element，RAF frame-driven
  + test-guard，evidence/logs/s251-solo.log）：**984P/79F/25T/
  5N**（对 s250-solo 979P/79F/25T/10N：ΔP=+5、ΔN=−5，F/T 零
  位移）。键级 join（last-wins + LC_ALL=C）：5 NotRun→Pass 零
  回归——目标 3 键（text/json/css-module `inline, 200, not
  parser-inserted`）+ module/load-error-events-inline.html 2 键
  （`src, 200, not parser-inserted, no defer, async` /
  `no defer, no async, no non-blocking`——名为 src 实为动态内联
  （体 appendChild 注入无 src），同一竞态同刀收口）。
- 全树 A/B（evidence/logs/s251-ab.log vs s250-ab.log，make
  testharness-html-semantics 无 FILTER）：行级 7781P/451F/77T/
  11N（对 s250 行级 7757P/458F/78T/21N；行级差含同键重复行伪
  位移，两日志各 8320 行）。键级 join **8317/8317 对，LOST=
  GAINED=5（同键 NotRun→Pass 换形），P→F 0，F/T 零位移**——
  第十个连续完美 A/B。
- 拼接 shim `node --check` 干净（单 part 文件本就非独立可解析
  ——part 边界在对象字面量内部，按 js_dom_bridge.rs 装配序七段
  拼接后校验）；Rust 零改动（fmt/clippy/单测不适用；make 目标
  自动重建 release runner 内嵌新 shim，探针行为已证新 shim 生
  效）。
- 四页终态：text-module/json-module/css-module 各 8/8 P，
  module/ 两页 16/16 P——load-error-events 全家族（5 页 48 臂）
  首次全绿。

## 归因修正与挂账更新

- s250 残余挂账「动态内联 module 成功体执行上下文」**收口**；
  其中「shim 侧任何 eval 与 window 属性通道均不可见」的机制描
  述作废（实为 accessor 通道全通 + 同步 eval 时序竞态），历史档
  不改写，以本档为准。
- s250 挂账「prelude `__zw_load_module` 直接 eval 同型潜伏面」
  假说弱化：动态 import 体经 promise 异步求值，触发脚本此时已
  完成，同型竞态不适用——归入候选 #2（module/dynamic-import
  基面 ~10F 勘根）一并核。
- 其余历史挂账延续：charset ×6 Timeout、http-accept-header ×2、
  CSS module 基 URL/复用面、referrer-policies ×3、s97
  insertBefore inline-module 面、动态 src module specifier 解析
  失败 window TypeError 报告面（shim tag 1 空码臂 FIXME）、
  in-order 队列 FIXME（async=false 动态模块按 asap）；customiz
  able-select 177F 挂起待用户拍板（GB-20261031 征询在途）。

## 下一步候选

1. module/dynamic-import 基面 ~10F 勘根（动态 import 与 script
   元素面共享底层后自然暴露的下一层缺口；含 prelude 潜伏面复
   核）。
2. 动态 src module specifier 解析失败 window TypeError 报告面 +
   in-order 队列 FIXME（shim 侧两处动态面余项，小刀位）。
