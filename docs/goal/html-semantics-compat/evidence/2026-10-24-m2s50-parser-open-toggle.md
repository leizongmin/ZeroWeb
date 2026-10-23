# M2 切片 50（s50）——R5254 parser 初始 open details toggle 补偿两段式，toggleEvent.html 文件级 Timeout 收口

日期：2026-10-24。基线 s49（426P/54F/2T，482 子测试）→ 本轮 **428P/54F/1T（483 子测试），REG 0 / GAIN 2**。

## 卡点（s49 挂账分诊）

`toggleEvent.html` 文件级 Timeout：11 个 async_test 中 9 Pass，`pending: 2` 永挂——

- **t9**（`Setting open=true on an opened 'details' element should not fire`）：依赖 `details9` 的 inline `ontoggle="window.details9TogglePromiseResolver()"` 在 parser 声明 `open` 时派一次初始 toggle resolve promise——promise 不 resolve 则 t9 永不 done。
- **末案**（`Setting open from the parser fires a toggle event`）：`DOMParser().parseFromString('<details open>')` 产物的 `ontoggle` 等初始 toggle。

根因：toggle 队列（R4009 `_zwToggleQueueNotify4009`）只在写路径入队（IDL setter / setAttribute/removeAttribute 尾钩）；parser 初始 `open` 由 host 解析直建、不经任何 JS 写路径 → 无事件。innerHTML / createElement-HTML / 包装世界解析面已各自接 `__zwDetailsScan4100(…, true)`（R4100），**唯主文档初始 HTML 与 DOMParser 快照面缺口**。

## 刀一：主文档两段式补偿（part03 + callbacks.rs + part01 reset）

单段直扫两难（两版都实证过）：
- **注册期直扫**：task 派发早于页面脚本——契约面 microtask 立即 drain（handler 未赋白派）；runner 面 arm 到真线程 timer、drain 到达序随机（t9 仍挂）。
- **DCL 直扫**：时序对了但分不清 parser open 与运行时 open——`activation-behavior.html` coalesced 案 toggle 计数 2≠1 回归实证（toggle-tester 初始 closed、页面脚本点击后 open，DCL 时被误判「初始 open」补派一次）。

**两段式**（spec 锚点：attribute change steps 随 parser 运行 → queue details toggle event task）：
1. **注册期标记**（`__zwDetailsInitOpenMark5254`，callbacks.rs 注册尾调）：页面脚本前文档是 parser 初始态——collect `details`，open 者身份 key（`_elKey`）记入 `__zwDetailsInitOpen5254`。
2. **DCL 派发**（`__zwDetailsInitOpenFire5254`，DOMContentLoaded capture listener）：解析完成点页面脚本必已跑完——重新 collect，只对「初始 open 且当前仍 open」`_zwToggleQueueNotify4009(sel, hnd, false)` 入队（走既有 toggle 队列：trusted ToggleEvent / per-element 去重 / setTimeout task 全复用）；标记集一次性消费。
3. **跨导航**：`__zw_reset_form_state` 尾清集 + 重标记（同 sandbox 复用，新文档重新标初始态；DCL listener 持续有效）。

## 刀二：DOMParser 快照面（part02）

快照架构（querySelector 每次重解析产新对象）死穴：扫描快照 ≠ 用户 querySelector 手持快照，实例属性 handler 不可达（首版直调实证计数 0）。修法：
- **parseFromString 返回前**扫 `details[open]`：命中快照打 `open` IDL getter（boolean 反射 `_attrs`，断言面 `e.target.open`）+ setTimeout task。
- **`_zwParseEl.ontoggle` IDL accessor**：handler 按身份键（`_zwEvKey`，id 或 sig 串——同源重解析稳定）登记全局表 `_zwParsedToggleH`；task 经身份键寻址派发（构造 ToggleEvent，target=扫描快照）。

## 契约（part81 `parser_open_details_toggle_r5254`）

钉 (a) 主文档初始 `<details open>` 恰好一次 toggle（closed→open + target.open 真值）；(b) 初始 closed 不派（翻转：标记/派发链被撤 → 计数 0）；(c) DOMParser 产物收到 toggle + target.open 真值；(c2) 无 open details 解析产物零 task。时序模拟：契约赋 `__zw_setTimeout` 记录式 stub + 手动 `__zw_fire`（runner/browser 宿主为真 timer）+ 手动派 DCL（宿主生命周期等价）。

## 验证

- toggleEvent.html：**Timeout → 11 子测试全 Pass**（t9 + 末案翻绿）
- activation-behavior.html：coalesced 案 DCL 直扫版 REG（2≠1）→ 两段式回绿
- engine lib：3446P/0F（+契约 part81）；fmt --check 干净；clippy -p zero-engine 0 告警
- 全域 interactive-elements：**428P/54F/1T（483 子测试）对 s49 逐位 REG 0 / GAIN 2**（fail 集逐位一致，仅 toggleEvent Timeout 消失；子测试总数 482→483 为 Timeout 不再吞 2 案）
- release runner 含 R5254 锚点串（grep 实证 4 处）

## 余账

- F 族 name-attribute 3F（mutation events / XHR 解析 / template 附着底座）维持挂账
- DOMParser 快照架构限制（只读、无 live 视图、on* 仅 ontoggle 接事件面）——live 文档面属底座 follow-up
- A 族 closedby×slot 16F / H 前置 declarative shadow 联合切片维持挂账序
