# R370 — cross-realm MO 观察链三件（MutationObserver-cross-realm-callback-report-exception 收口）

**日期**: 2026-08-30
**切片**: 已知 Fail 集合 9→8——cross-realm MO 整文件转绿（R302 遗留「工厂 body
可观察 id」域的本轮收口；顺带落 R351 预留的 trap 顶部短路扩展）
**改动面**: `js_dom_shim/part03.js`（append/prepend 字符串转 Text + R370 body
独立派发 helper）+ `part01.js`（R302 勘误：函数回调印记读取）+
`part05.js`（R370 勘误：BoundFunction 的 win 捕获时机）+ R370 trap 短路扩展

## 1. 根因链（探针逐层剥）

WPT 流程：`mo = new frames[0].MutationObserver(new frames[1].Function(throw …))`
observe `frames[0].document.body` → `body.append("foo")` → 回调抛 → 须上报
**frame1** 的 onerror。探针序列定位三层断裂：

1. **`body.append("foo")` 对字符串静默丢弃**——iframe 子文档 body 的
   append/prepend（`_r117Install` 的 `_mk` 闭包）只处理 object 参数，字符串
   不转 Text → **mutation 根本没发生** → observer 回调永不触发
   （`onerrorCalls=[]` 的直接根因）。
2. 修 ① 后回调触发但异常上报到 **"top"** 而非 "frame1"——两层：
   - **R302 印记读取 gated `typeof === 'object'`**：函数回调（本用例形态）
     的 `_zwRealmWin` 永远读不到（part01）。
   - **R302 BoundFunction 的 `var win302 = win` 在 win 字面量求值期捕获
     `undefined`**（part05）——IIFE 在 `var win = {…}` 字面量内部执行，此刻
     `win` 尚未赋值 → 印记恒 undefined（falsy）→ flush 域反查 miss。旁证：
     R187 Proxy wrapper 的印记在调用时读 `win`（revocable 函数体内）故其
     用例一直正常——同一 bug 的两种摆放、两种命运。

## 2. 修复四件

1. **append/prepend 字符串转 Text**（part03 `_r117Install`）：spec
   convert-nodes-into-a-node——字符串经 `target.ownerDocument.createTextNode`
   转换后走 target.appendChild/insertBefore（detached/iframe 子文档域的
   ParentNode append/prepend 语义补全）。
2. **R370 body 独立派发**（part03 `_r370NotifyBodyChildList` + body.appendChild
   两分支接线）：detached/iframe body 的本地 mutation（R189 `__r189:BODY:<seq>`
   键注册的观察者）直接入列 + flush——R189 轻量元素同款模式；仅 R189 键命中
   者受影响，sel/handle/doc 站零变化。
3. **R302 勘误**（part01）：印记读取放宽 `typeof === 'object' || 'function'`
   ——函数回调的 `_zwRealmWin` 印记可读。
4. **R370 勘误**（part05 BoundFunction）：`win302` 字面量期捕获改**调用时读
   闭包 `win`**——字面量求值完成后 win 已赋值，BoundFunction 调用发生在页面
   脚本期，闭包读即为最终 win。
5. **R351 预留扩展**（part03 get trap 顶部）：`__zwIsText`/`__zwChildIndex`
   顶部短路——R260/R262/R263 adjust 扫描对 proxy 容器的这两键读此前付完整
   R98 原型走查（78µs/读）→ O(注册表 × ops) 尾批；proxy 从不 own 这两键
   （写点全在 plain wrapper），直返 undefined 行为等价。dataChange 单跑 declared
   395→454（+15%），文件级 Timeout 尾批仍在（R353 归因的游离树堆积域）。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 目标件 | `MutationObserver-cross-realm-callback-report-exception` 1F→Pass 整文件转绿 |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55495P/10F/14T——Fail 文件集合 9→8 零新增、Pass 净 +2**（Timeout 集合为并发噪声轮转族） |
| 哨兵套件 | MO 族 135P/3F（恰 +1P/-1F 即目标件）、Node-parentElement 12P、Node-appendChild 11P、inner-outer/takeRecords/children 全 Pass、Range-insertNode 1841P |
| engine 单测 | v8 **2498** / quickjs 1473 全绿（本轮无新单测——三件修复均由 WPT 目标件直接驱动，R302/R370 勘误的行为面即断言面） |
| webview / integration | 658P 全绿（SW flake 本轮未复现）；integration 784P |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

**过程教训**：① 对象字面量内 IIFE 捕获外层 `var` 的时机陷阱——字面量求值期
`win` 未赋值，捕获得到 undefined；凡需引用最终对象的 wrapper（BoundFunction
等）必须在**调用时**读闭包（R187 Proxy 的正确写法是现成反例教材）。② 函数
也是对象——`typeof x === 'object'` 的印记读取 gating 把函数回调排除在外，
R302 当年只验了对象形态回调。③ `append("foo")` 静默 no-op 是「字符串参数
不转 Text」的观测面——empty-calls 探针（fired 计数 + onerrorCalls 数组）把
「回调没跑」与「路由错」两层剥开。

## 4. 后续

- 已知 Fail 集合余 **8**（全部深结构/基建域定性）：MutationObserver-document
  3F（parse-time 基建）、remove-and-adopt-thcrash（window.open 无 popup
  通道）、querySelector-mixed-case / remove-next-sibling（R220 identity 双源
  域）、events 2F（onerror 跨 realm / Chromium 专有 pseudo）、ranges
  dataChange/replaceData 2F（文件级 Timeout 尾批——R353 游离树堆积域，本轮
  短路扩展 +15% 进度但不闭文件）。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛。
