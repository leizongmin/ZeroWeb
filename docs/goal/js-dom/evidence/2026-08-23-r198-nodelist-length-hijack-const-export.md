# R198 Evidence — NodeList length 劫持语义 + strict classic const/let 跨脚本导出（M4）

**日期**: 2026-08-23
**切片**: M4 轻量——NodeList-static-length-getter-tampered 六用例 0P/6F → 6P/6（全 100%）+ NodeList-Iterable own-keys 连带，全量 9758P/111F → 9765P/104F（净 +7P/-7F 零新增）
**改动面**: `script_gen.rs`（const/let 导出扫描）+ `part05.js`（`_zwMakeCollection` NodeList 路径 Proxy 化）+ `part03.js`（NodeList hasInstance 认 `__zwNL`）+ `fetch-dom-subset.sh`（support/ fixture）+ `part21.rs`（单测）

## 一、fixture 层根因（六用例整簇 "script fetch failed"）

`NodeList-static-length-getter-tampered-{1..3,-indexOf-{1..3}}.html` 引用
`support/NodeList-static-length-tampered.js`（`makeStaticNodeList` + 两个 `new Function`
索引器工厂）。`fetch_dir_html` 只列子目录**顶层** .html/.js 不递归 support/——六用例
在外链脚本 fetch 阶段整簇死。修：`fetch-dom-subset.sh` 补 `fetch_raw
"dom/nodes/support/NodeList-static-length-tampered.js"`（与 dom/events/resources 同款
显式抓取）。

## 二、strict classic 顶层 const/let 跨 `<script>` 不可见（script_gen）

间接 `(0,eval)` + 源内 `'use strict'` → eval 独立变量环境困住块级声明（R147 已解
function 形态）。扩展导出扫描到行首 `const NAME =` / `let NAME =`。

**两个坑（回归实证）**：
1. **等号前空白**——首版判 `after.starts_with('=')`，真实 WPT 源是
   `const indexOfNodeList = new Function(...)`（`=` 前有空格）全 miss。改
   `after.trim_start().starts_with('=')`。
2. **minified bundle 的 IIFE 内零缩进 const**——lit bundle 的
   `var ns = (function() {` 换行后是**零缩进**的 `const t=globalThis,...`；行首锚定
   无法区分真顶层与 IIFE 作用域。后缀在 eval 顶层执行时 IIFE 作用域名已消亡 → 裸
   `globalThis.t=t` 抛 ReferenceError **中止整个 eval**——lit e2e 六测试
   NO-REPORT/EXEC-ERR（`make test` 当场抓回，clean HEAD 对照确认归因）。修：每名
   `try{globalThis.NAME=NAME;}catch(_zw_ex){}` 独立包裹——作用域外名字静默跳过。
   R147 的 function 导出同款补包裹（lit 无行首 function 未暴露，防御同形态）。

## 三、静态 NodeList 的 length 劫持语义（part05 Proxy 承载）

真浏览器 NodeList 的 `length` 是**原型属性**（实例无 own length）——三形态劫持都
改变读取结果：

| 形态 | 用例 | 真浏览器行为 | Array 承载行为 |
|------|------|--------------|----------------|
| ① own `defineProperty(nodeList,'length',{get})` | tampered-1/-indexOf-1 | 实例无 own → 建 accessor 生效，循环读 10 → -1 | own + non-configurable → **抛 "Cannot redefine property"** |
| ② `setPrototypeOf(nodeList, {get length(){...}})` | tampered-2/-indexOf-2 | 原型 getter 生效 → -1 | own length 遮蔽 → 恒 50 |
| ③ `defineProperty(NodeList.prototype,'length',{get})` | tampered-3/-indexOf-3 | 原型篡改全局生效 → -1 | 产物不继承 NodeList.prototype → 恒 50 |

**方案**：`_zwMakeCollection(arr, false)` 升级为 **Proxy 承载，target 用普通对象**
（索引属性 0..n-1 落 target；无 own length → ① 无 invariant 冲突）。`length` 读取
按 spec 解析序：**expando（①）→ 原型链 getter（②③）→ 真实计数**。原型链接线
`target → NodeList.prototype → Array.prototype`（Array 原型方法经 receiver=proxy
调用，indexOf/map/for-of 全兼容）。

**迭代修正（两轮）**：
- 首版 Proxy target 仍是 Array——① 仍抛（proxy invariant：target 有 non-configurable
  属性时 trap 不能汇报不存在）。换 plain target 收口。
- `ownKeys` 初版直通 `getOwnPropertyNames(target)` 把内部印记
  （`__zwNL`/`__zwQSA`）泄进 `Object.keys`——NodeList-Iterable
  "responds to Object.keys correctly" 回归（got 多 2 键）。修：ownKeys 滤除
  `__zwNL`/`__zwQSA`/`__zwLiveNL`/`item`（length 本就不进 own）。

**消费面补偿**：`Array.isArray(product)` 由 true → false（Proxy 无法伪造）——
part03 的 NodeList `Symbol.hasInstance` 补认 target 的 `__zwNL` 印记（R159 的
instanceof NodeList 断言保持）；内部消费（索引循环 / prototype 方法调用 / for-of）
经 trap 语义不变（探针 12 步全 ok 实证：item/keys/forof/instanceof）。

## 四、A/B 与全量

- 全量 polyfill **9765P/104F** / native **9765P/104F**（per-file 零差异；±1P 边缘
  Timeout 不在 NodeList 族）
- fail 清单与 R197 基线 diff：**零新增**，fixed 7 文件（六 tampered + NodeList-Iterable）
- `make test`：唯一 1F = `update_permissions_follow_calling_worker_state_during_installation`
  ——clean HEAD 单跑同败，service-workers 流域（run-rules §10，R183/R192 同款先例）
- fmt / clippy v8 全 workspace + quickjs 矩阵（make test 内并行段）干净
- 单测：`test_classic_script_strict_const_let_globals_r198`（跨脚本可见 + IIFE 内
  const 不误发布 + sentinel 干净）；lit e2e 六测试全绿（lit.bundle.js 含 4 处行首
  IIFE 内 const——正是 try/catch 包裹的活回归样本）

## 五、commit

`0c2ba1f9a`（rebase 吸收并行 SW 流两提交后落位）

## 六、剩余 fail Top（R199 输入）

ParentNode-querySelector-escapes 4F / MutationObserver-document 3F /
Range-in-shadow-after-shadow-removed 2F / ParentNode-querySelector-scope 2F /
ParentNode-querySelector-case-insensitive 2F / ParentNode-querySelector-All 2F /
Node-constants 2F / MutationObserver-inner-outer 2F / Element-remove 2F /
Document-constructor 2F / Event-dispatch-redispatch 2F。
