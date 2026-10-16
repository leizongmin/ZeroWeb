# M2 切片 2a — focus/focus-visible 伪类 + rAF 预算（2026-10-16）

## 范围

- `:focus`/`:focus-visible` 伪类 JS 侧 `matches()` 判定（CSS Selectors L4 §13/§14
  https://www.w3.org/TR/selectors-4/#focus-pseudo ）——shim R3902。
- 脚本 `focus()` 的 focus-visible 模态近似：脚本聚焦落键盘式可见焦点
  （headless 无先验指针交互信号，恒匹配；失焦/宿主驱动失焦清旗标）。
- rAF 单脚本执行预算 64→512（part01 `_rafBudget`）：testharness 串行
  promise_test 链逐案 `await tick()`（double-rAF）全在首个脚本轮 microtask
  内，64 预算在 32 subtest 处静默丢回调（promise 永不 resolve → 文件级
  Timeout）——focus-dynamic-type-change-on-blur 双文件 33 号案起全挂的直接
  根因。512 仍有限（防 rAF(loop) 死循环守卫语义不变），覆盖 256 个
  double-rAF 案。

## 运行方式

- 通道：`make testharness-html-semantics TIME_LIMIT=9000 FILTER="--json"`
  （guard `--per-proc-mem 4 --total-mem 8 --time-limit 9000`，与 M2 切片 1
  同参数；全程无并发重载）。定向先行：`FILTER="focus-dynamic-type-change --json"`
  两文件 120P/0F 复现后才跑全量。
- 原始账本：[2026-10-16-m2s2-focus-visible.json](2026-10-16-m2s2-focus-visible.json)
  （九域全量 1226 案逐案逐 subtest）；回归判定按切片 1 账本
  [2026-10-15-m2-input-value-modes.json](2026-10-15-m2-input-value-modes.json)
  逐 subtest 精确 diff。

## 总账

| 面 | 切片 1 Pass | 本轮 Pass | Δ |
|---|---|---|---|
| focus-dynamic-type-change（40 subtest） | 0 | 40 | **+40** |
| focus-dynamic-type-change-on-blur（80 subtest） | 0 | 80 | **+80** |
| forms 域（4156 subtest） | 2320 | 2441 | **+121** |
| **九域全量（6656 subtest）** | **3248** | **3369** | **+121** |

**回归 0**：逐 subtest diff——基线 Pass → 本轮非 Pass = **0**；全量
+121P 全部落在 forms（其余八域逐域 Pass/总数逐字节不变）。提升构成：
focus 双文件 0→120P + time-focus-dynamic-value-change.html +1（`:focus`
匹配联带翻绿）。分母 +47 subtest（on-blur 文件 promise 链此前在 rAF 预算
处断链，后续 subtest 未注册；512 预算后注册完整）。

## 实现面（shim R3902）

- `part06.js`：`_zwFocusVisibleMod` 旗标（focus-visible 模态近似，脚本
  focus 置位、失焦清位）；`__zw_host_focus` 宿主驱动失焦同步清位（宿主
  驱动获焦无模态信号，保守沿用最近模态——注释挂账）。
- `part04.js`：焦点派发处旗标置位/清位；`matches()` 头部 `:focus`/
  `:focus-visible` 独立伪类分支——`_activeElKey === key && isConnected`
  （连接性以 isConnected 为准，不用闭包 sel 判 attached；spec `:focus`
  须在文档树内）。复合选择器与 closest/query 仍走 host（不命中，既有
  缺口不变）。
- `part01.js`：`_rafBudget` 64→512（两处：初始值 + `__zw_begin_script`
  重置）。

## 已知挂账

- focus-visible 的指针模态（mousedown 获焦不匹配）headless 无信号源，
  近似恒匹配；Tab 导航模态归宿主输入面（uievents-compat 域）。
- 本域剩余失败簇（下一切片候选，切片 1 挂账顺延 + 本轮新增归因）：
  untrusted-key-event 42F（`querySelector('input[type=X]')` 对
  createElement→IDL type=→append 产物不可见，探针已备）、cloning-steps
  45F（`_isTag` sel-only 门致 detached 元素 indeterminate/files IDL
  undefined + clone 不传播 dirty checkedness/indeterminate）、
  stepUp/stepDown 43F（现仅 number/range 有桩，spec 全类型须抛
  InvalidStateError + 走步进算法）、checkable-active-space 8F（JS 侧
  选择器白名单缺 `:active` 解析接受）、showPicker ~130F、files
  null-for-non-file 23F。
