# Console M2 — IDL/格式化/状态语义收敛 + BiDi 捕获通道（2026-10-14）

## 结果

**全 corpus 28/28 = 100%**（基线 8/28 = 28.6% → 一轮收口，无下行轮次）。
JSON 全量：[2026-10-14-m2-semantics.json](2026-10-14-m2-semantics.json)。复现：
`make testharness-console`。

| 切片 | 面 | 子测试 | 修复 |
|---|---|---|---|
| M2 切片 1 | WebIDL namespace IDL（is-a-namespace + class-string） | 5 → 8/8 | 空原型中间层 + `Symbol.toStringTag` |
| M2 切片 1 | count/time label 转换（label-conversion） | 10 → 10/10 | label ToString + 异常再抛 |
| M2 切片 2 | BiDi log.entryAdded 面（count/countReset/timing ×2/number-format） | 5 Unsupported → 5/5 | 捕获通道 + Formatter + 状态记账 |

全绿面：11/11 案（含 5 个 BiDi 观测案 + 3 个历史遗留方法不存在案 + 参数容错
2 案）。

## 语义实现（crates/engine/src/js_dom_shim/part02.js）

1. **Namespace IDL**：console 对象改为 `Object.create(Object.create(Object.prototype))`
   ——自有成员直挂、空原型中间层（`getOwnPropertyNames(proto) === 0`）、
   `Symbol.toStringTag = "console"`（{writable:false, enumerable:false,
   configurable:true}，delete/defineProperty 可操作）。
   https://webidl.spec.whatwg.org/#es-namespaces
2. **count map / timer table / group stack**：count 计数输出 `"label: N"`
   （level=count）；countReset 重置/missing 告警；time 记时 + already-exists 告警；
   timeLog/timeEnd 输出 `"label: <ms> ms"`（timeEnd 移除条目；timeLog 追加额外
   data）。label 经 WebIDL `optional DOMString = "default"`——缺参/undefined →
   "default"，其余 ToString **不带守卫**（toString 抛出原样传播——
   console-label-conversion 口径）。状态查用 `hasOwnProperty`（`'constructor'`
   label 原型链污染防护）。
   https://console.spec.whatwg.org/#counting §1.2 / #timing §1.4
3. **Formatter**：`%s` String；`%d/%i` Symbol→NaN 否则 parseInt(current,10)；
   `%f` 同型 parseFloat；`%o/%O` 序列化串（optimally useful formatting 实现定义
   ——对象走 JSON）；`%c` 消费不产出（spec TODO）；`%%` 转义**不消耗参数**；
   首参非 string 不格式化；转换异常不向页面抛。text 面 = 格式化后单行；
   valuesJson（CDP S11）面 = 未格式化原参（与 Chrome consoleAPICalled args 一致）。
   https://console.spec.whatwg.org/#formatter §2.2
4. **BiDi log.entryAdded 页内捕获通道**：shim `globalThis.__zwConsoleBidiListeners`
   + `_zwConsoleEmit` 逐条目派发 `{type:"console", method, text, args}`（text
   格式化、args 为 value-only remoteObject 列表）；runner TESTDRIVER_STUB 增
   `bidi.log.entry_added.{subscribe,on}`（on 注册进 shim 注册表，返回 remover；
   subscribe 返 resolve(unsubscribe)——无真 BiDi session 语义）+
   `unsupported_testdriver_dependencies` 的 `bidi` 白名单项。
   https://w3c.github.io/webdriver-bidi/#log-entryAdded
   **devtools 共用面不变**：`__zw_console_log(level,text,argsJson)` 宿主回调契约
   原样（level 新增 count/timeLog/timeEnd/group 等值，headless CDP 层
   `other => other` 透传，零协议改动）。

## 验证

- WPT yardstick：`make testharness-console` 28/28（本轮两次收敛——切片 1 后
  18/18 可执行面全绿、BiDi 5 案 Unsupported；切片 2 后全量 28/28）。
- 单测钉：`test_console_host_bridge_r3256` 扩段——count 计账/`default` 缺省/
  label 异常传播/countReset 重置语义 + Formatter 四断言（%d/%f/%s、%O JSON、
  Symbol→NaN、%% 不耗参）+ BiDi 条目形态（listener 抛错不中断派发；text 格式化 /
  args 原参）。
- 门禁：`make test` 71 套件全绿（exit 0）；`cargo fmt` / `cargo clippy
  --all-targets -D warnings`（zero-engine + zero-wpt-runner）干净。
- 过程注记：BiDi 通道首跑抓出 formatter 对 Arguments 对象误用 `.slice`
  （TypeError）——emit 入口统一 `Array.prototype.slice.call(arguments)` 后修复。

## 待办（M3）

- DC-3 收口核对：`make reftest` 零回归（本轮未跑——console 面无渲染交集，M3
  收口时统一跑）+ product-smoke welcome 恒值。
- wpt-suites.csv M3 终态行（100% 定格）+ imported-tests 账本面（testharness
  corpus goal 无 imported-tests.txt 条目——cookies 先例，fetch 通道 + evidence
  即资产记录）。
