# M2 切片 355：webview↔engine 交付排水门——execution-timing 051 F→P

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/execution-timing/051.html`（s354 暴露面首选候选：动态外链 classic src 装载序）
**结果**: 051 F→P；域 join det **F→P=1 / P→F=0 / T→P=0**；语料口径 the-script-element **13F→12F**（distinct-pair 320→321 键：300P/13F/7T → 302P/12F/7T；新增键 = the-noscript-element/non-html-noscript A 面缺席 B 面 Pass，非退面）；双态键 script-not-found-not-executed A/B 均 {1 Pass,1 Fail} 同集（join 配对伪影，0 净变）

## 勘根

[HTML prepare the script element](https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element)：动态插入的 src 脚本（testlib.addScript 形）取回是异步任务，parser 交付窗内插入让后于其后 parser 脚本。051 期望双 alternative：alt A = 流式模型（动态 fetch 先于 inline#2 物化，count 5）、alt B = 让后序（['inline script #1','end script #1','inline script #2','script tags in DOM: 6']，批模型可达）。引擎旧时序：动态 src 的 fetch promise 在**交付单元间**排空——count:6 落 inline#2 前（红态 got ['inline script #1','end script #1','script tags in DOM: 6','inline script #2']，s354-solo-battery.log），双 alternative 均不配。

上游双真值在册（s354 已核）：053 alt1/alt3 分别 sanction 动态先行/让后两序，051 altB = 让后序 + count 6。

## 施工（shim gate 单点 + webview 旗标两刀）

- **shim part03 `_zwPrepareSrcScriptGate`**：classic 臂 fetch 段包 `_zwArm96` 闭包、module 分支 owner/缓存直取两 setTimeout 臂包 `_msArm355`/`_msArm355a`——排水门活跃（`_zwDrainHold355()`：`__zwDeliveryDrainActive === true` 且非写流挂链面 `__zwDocWritePendingSlot < 0`）时 push `__zwDrainPendingSrcArms`、return true（记账语义不变）。charset 属性读取留登记时（prepare 时刻快照，s122 判据不随 flush 回溯）；async 排队分支判定同留登记时。写流挂链豁免 = s353 停车语义（写流 src 先于其后 markup），宿主让位（renderer）在既有门已 bail。
- **shim part03 导出区**：`__zwFlushDrainPendingSrc`——清旗标 + 逐臂发起（单臂 throw 上报后继续，不丢队列）。
- **webview.rs `run_page_scripts_impl`**：脚本执行 for 循环前置位 `__zwDeliveryDrainActive = true`；循环毕 execute flush。
- **FIFO 队位保序（probe 实证迭代）**：首版 module entry 登记时入队（交付窗内）而 classic entry flush 臂内入队 → 队序错位（probe 序 `module3,classic2,classic4`，execorder dynamic async-ordered P→F 同根因）。终版三臂统一**臂首入队**：classic entry 创建+入队提前到 fetch 发起前、module owner/缓存直取臂首 `_zwQueueEntry355(_msEntry250)`、settle/fail 时 `_zwQueueEntry355` 守卫幂等（waiter settle 随 owner 臂遍历序入队）。臂序 = 挂起登记序 = 插入序 → 队位 = 插入序，慢源 entry 不因 settle 晚失位（头阻塞语义完整）。

## Blast radius

- module src 同门（非仅 classic）：execorder dynamic async-ordered 的 module/classic FIFO 交错序依赖统一让后——module 分支若不挂起，其 settle drain 时 classic entry 尚未入队即抢跑（首域跑实证 P→F，probe 复现，终版同门修复）。
- 写流面零扰动：041（s352）/068（s353）在域跑 B 面 Pass（豁免臂生效）。
- 非 runner 车道（renderer 宿主让位）在 gate 既有门 return false，零变更；reftest/product 页面脚本均在 load 前执行完，flush 时点无观察面差异。
- 残 12F 逐项在册（s354 挂账清单）：052 批解析可见性长挂、143 upstream 复核、probe12/probe13 自建单列、fetch-src/alpha/base、json-module parse-error-location、custom-element-exception、import.defer 三案、remove-src-attr、script-not-found 双态。零新增红。

## 验证

- **红态**：s354-solo-battery.log（051 双 alternative 均不配，count:6 在 inline#2 前）。
- **probe**：最小复刻页（create_script async-ordered 三连 classic/module/classic）首版红态序 `module3,classic2,classic4` → 终版 Pass `classic2,module3,classic4`（probe 资产验证后删除，不入语料清单）。
- **域 join**（LC_ALL=C + (key,state) sort -u + 双态键 A/B state 集合核对）：A=s354-b **1109P/13F/8T→1109P/13F/7T** 账面（distinct-pair 320 键）vs B=s355-b **302P/12F/7T**（321 键，含 A 缺席 noscript 1 案）——F→P=1（051）、P→F=0、key 集合 A⊂B（仅新增 noscript）。
- **clippy** zero-webview + zero-wpt-runner `--all-targets` 净、`cargo fmt --all -- --check` 净。
- **make test**：见控制文档轮次账（既有 5F 为 R-t2qq adopted-stylesheets 等非本流在册集）。

## 挂账 / 下一步

- 052 批解析可见性：流式物化架构面长挂（不变）。
- 143 upstream 真值复核（s352 延续）；fetch-src/alpha/base、json-module parse-error-location、custom-element-exception、remove-src-attr、script-not-found 双态逐案勘根；import.defer 三案独立切片；probe12/probe13 自建单列；T=7 长尾在册。
- 排水门 module 内联面（`_zwScriptMaybeStartDynamic` module 臂 setTimeout 编译）未挂——交付窗内动态插入**内联** module 的求值时序独立面，语料无案面，挂账待语料扩展。

## 日志

- evidence/logs/s355-b-script-element.log（域跑 B 面，join 基线 s354-b-script-element.log）
