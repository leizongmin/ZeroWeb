# R313 Evidence — disabled 表单元素的 click() 门 + handle 布尔 falsy 真移除（on-disabled-elements 1F→0F）

**日期**: 2026-08-27
**切片**: M4——R313(a) events 剩余 5F 逐个归因（首件全解 + 两件深结构备档）
**改动面**: `part04.js`（click() 的 disabled 门）+ `part05.js`（布尔 falsy 的 handle 真移除）+ `part24.rs`（+1 单测）

## 一、成果

| 套件 | 基线 | R313 | Δ |
|---|---|---|---|
| Event-dispatch-on-disabled-elements | 3P/1F | **4P/0F（subtest 100%）** | +1P/-1F |
| dom/events 全域 | 581P/5F | **582P/4F** | +1P/-1F |
| ParentNode-querySelector / Element-matches / Node-properties / MutationObserver | 基线 | 同 | 持平 |
| vue e2e | 3P | 3P | 持平（布尔修复涉表单面，Vue 表单交互无冲突） |
| engine 单测 --lib | 2451 | **2452** | +1（r313 六态断言） |
| make test | 1F 环境项 | 同 | 持平 |
| fmt / clippy | — | 干净 | — |

## 二、两件修复

1. **click() 的 disabled 门**（part04 click trap）：spec HTML §activation——form-associated
   element 是 disabled 的 → 激活行为返 undefined（跳过派发）。可禁用族 =
   button/input/select/textarea/fieldset/optgroup/option。属性存在即禁用（handle
   latest-wins / sel `__zw_has_attr_lw` 同 disabled getter 口径）。**dispatchEvent 直发
   不受门影响**（spec：dispatchEvent 无激活行为——单测断言 directDispatchWhileDisabled）。
2. **handle 布尔 falsy 真移除**（part05 set trap 的 hidden/checked/disabled/selected
   分支）：旧注释「handle falsy：无 remove-handle 变体 → 不设」——但
   `__zw_remove_attr_handle` **已注册**（callbacks.rs:1001，R3039/40 反射表分支在用）。
   `.disabled = false` 后属性残留使门与 getter 恒真（WPT re-enable 后 `.click() must
   dispatch` 断言必败的根因）。补 handle remove 分支（与反射表同款）。

## 三、events 剩 4F 域界定（2 件深结构备档 + 2 件记档）

- **Event-dispatch-handlers-changed 1F**（17 vs 16 target 阶段计数）：spec
  `dom-event-dispatch` 的「target 阶段 capture/bubble 两份 listener 内拷贝各跑一次」
  ——dispatch 循环的 target 阶段双调用语义深改（涉全部派发路径），**深结构备档**。
- **event-global-onerror 1F**：`window.event` 全局恢复 + 跨 realm onerror 链
  ——event-global 族深结构（与 R302 cross-realm 同域），**备档**。
- **click-on-absolute-pseudo 1F**：Chromium 专有（R144 记档不追）。
- **zz-r180-act-probe 1F**：本地探针文件（非上游用例）。
- 另：on-disabled-elements 尾部 Timeout（CSS transitions promise_test）——headless
  无 transition 引擎面，**渲染域**（非 events 语义）。

## 四、教训

「无变体」类注释会过期——`__zw_remove_attr_handle` 在 R3039/40 已注册但布尔四件套
分支未同步（注释停留在早期无回调状态）；修共享机制后应 grep 同语义旧分支。
