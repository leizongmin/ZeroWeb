# R312 Evidence — trusted 事件语义三件（Event-dispatch-redispatch 2F→0F 100%；events 域 579P/7F→581P/5F）

**日期**: 2026-08-27
**切片**: M4——R312(b) events 域尾部归因（redispatch 簇首件全解）
**改动面**: `part03.js`（`_makeEvent` trusted 内部口 + `__zwMakeTrustedEvent` 出口）+ `part05.js`（`_zwDispatchGuard` 脚本派发翻 false）+ `part06.js`（`__zw_dispatch_event` UA 置 trusted + 印记消费）+ `testharness.rs`（runner 注入 DCL/load 走 trusted 出口）+ `part24.rs`（+2 单测）

## 一、成果

| 套件 | 基线 | R312 | Δ |
|---|---|---|---|
| Event-dispatch-redispatch | 1P/2F | **3P/0F（100%）** | +2P/-2F（DCL + mouseup 两 redispatch 全解） |
| dom/events 全域 | 579P/7F | **581P/5F** | +2P/-2F |
| Event-dispatch 全族 | 216P/2F | 同 | 持平（handlers-changed + on-disabled-elements 两既存） |
| ParentNode-querySelector / Element-matches / MutationObserver | 基线 | 同 | 持平 |
| engine 单测 --lib | 2449 | **2451** | +2（r312 双形态断言） |
| make test | 1F 环境项 | 同 | 持平 |
| fmt / clippy | — | 干净 | — |

## 二、三件修复（isTrusted 全生命周期语义）

WPT Event-dispatch-redispatch 断言对「before redispatching trusted=true / after
redispatching trusted=false」要求三面语义：

1. **UA 合成事件置 trusted**（part06 `__zw_dispatch_event` 出口）：宿主经
   `script_gen` 派发的全部激活/输入事件（mouseup/click/keydown…）按 spec 是 UA
   合成——`isTrusted=true` + `_zwUaDispatch` 印记。runner 注入的 DCL/load
   （testharness.rs）经新增 `__zwMakeTrustedEvent` 出口（part03 `_makeEvent` 的
   `__zwTrusted` 内部口——页面脚本的 `new Event` 恒 false 不受影响，spec 保证
   只有 UA 能造 trusted 事件）。
2. **脚本派发翻 false**（part05 `_zwDispatchGuard`）：页面脚本 `dispatchEvent` 同一
   事件对象时按 legacy DOM3 语义置 `isTrusted=false`（全部 dispatch 入口经 guard
   单一落点）。
3. **UA 印记一次性**：印记在宿主 dispatch 完成（`__zw_dispatch_event` 尾）与 guard
   通过时消费——「首次（UA）dispatch 保持 trusted、再经脚本 dispatch 即翻 false」
   的精确 redispatch 语义。

**归因过程**：sandbox 复刻 UA mouseup→click 链 + click listener 内脚本
re-dispatch 的四态断言（firstMouseup=true/firstClick=true/afterRedispatch=false/
clickStill=true），三轮迭代（无条件翻 false 误伤 UA 首派 → 印记不消费使
redispatch 恒 trusted → 消费点放 `_dispatchWithBubble` 尾但该函数 10 个 return
路径漏覆盖 → 收敛到生产者侧消费）。

## 三、events 域剩余 5F（下轮候选）

- Event-dispatch-handlers-changed 1F（listener 内改 handlers 的迭代语义）
- Event-dispatch-on-disabled-elements 1F（click() on disabled 不派发）
- event-global-is-still-set-when-reporting-exception-onerror 1F（window.event 恢复）
- click-on-absolute-pseudo 1F（Chromium 专有，R144 记档不追）
- zz-r180-act-probe 1F（本地探针文件，非上游用例）

## 四、sweep 状态

R310 启动的后台全量 sweep 仍在跑（与 R312 的 cargo 构建多次交错，重启过）——
结果核对转 R313。

## 五、教训

trusted 语义是**跨三层的生命周期**（UA 置位/脚本翻转/印记一次性消费）——单层修复
必然在另一层翻车（本论三轮实证）；多 return 路径的函数尾部注入不可靠（10 个
return 只盖住部分路径），消费逻辑放**生产者单一出口**最稳。
