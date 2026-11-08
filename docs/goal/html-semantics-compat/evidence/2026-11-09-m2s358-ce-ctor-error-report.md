# M2 切片 358：CE ctor 异常上报对齐——custom-element-exception F→P

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/module/custom-element-exception.html`（s357 挂账残 9F 之一）
**结果**: 单案 F→P；域 join det **F→P=1 / P→F=0 / T 双向 ∅**；语料口径 the-script-element **9F→8F、5T 不变**（321 键）；残 8F 全部在册零新增红。

## 勘根定谳

1. **测试面**：module 内定义 XThrower（class ctor `super()` 后 `throw new Error("custom element error")`）→ define → createElement → window load 时断言 `log.length==1`、`log[0] instanceof Error`、`message=="custom element error"`（log = window error listener 收集 `ev.error`；红态 `assert_equals: expected 0 but got 1` = log.length 0，异常未达 window error 通道）。
2. **spec 链核对**（curl custom-elements.html 原文）：升级算法 catch 段尾「Set element's custom element definition to null. Empty element's custom element reaction queue. **Rethrow the exception.**」；drain 面（§4.13.6 Custom element reactions）「upgrade reaction → Upgrade element... If this throws an exception, catch it, and **report it** for ... associated realm's global object」——即升级异常**不向页面传播**（createElement 正常返回）但须 report 到 window（WebIDL report the exception = ErrorEvent.error 承载异常对象）。
3. **根因定位**：引擎升级原语 `_ceRunCtor`（part03 13136）class 臂 `try { new ctor(); } catch (_eNew) {}`、function 臂 `try { ctor.call(el); } catch (_eCall) {}`——**两处静默吞掉，无上报**。全部升级面（createElement part03/part06、innerHTML R365/R366、walk `_ceUpgradeNode`、clone `_ceUpgradeElIfRegistered`）单点收口本函数——单点修全覆盖。
4. **无重试面已等价**：`_ceUpgraded` WeakMap 章门**先于** ctor 体盖章（13141）——ctor 抛错不产生重试窗，spec「definition 置 null」步的引擎面内等价已存在；「清空 reaction queue」步引擎无 reaction queue 面，无对应缺口。

## 施工（JS shim 单刀，Rust 零改动）

`_ceRunCtor` 两处 catch 臂接 `_zwReportListenerError`（part03 16495，window error 上报既有通道——ErrorEvent + listener 派发 + R114 onerror 直调，`.error` 承载异常对象恰合测试断言）：

1. **class 臂**：catch 先捕获到局部变量，`finally` 复位 `_zwCeExisting` **之后**再上报——上报 listener 内同步再建 CE 时不消费过期 this 注入印记（catch 先于 finally 执行，若在 catch 内直调则印记仍在场）。
2. **function 臂**：catch 内直调上报（该臂无 `_zwCeExisting` 参与，无序问题）。

吞后继续形态保留（createElement 照常返回 el、`_ceFireInitialAttrChanges` 照走）——非 throw 路径控制流逐字节不变（新增代码全在 catch 臂内）。

## Blast radius / 回归定谳

- **corpus 全域普查**：`customElements.define` + `throw` 同页仅 1 页 = 本切片目标（另 2 页 dialog/fieldset CE 页 ctor 不 throw，"throw" 命中在非 ctor 代码）——新派 error 事件的可能面仅此 1 页，join 证实恰 F→P=1。
- **engine 单测普查**：js_dom_bridge 测试 CE define+throw 面 **0**——make test 侧无新 error 事件源。
- **walk 面探针预期修正**：probe16 首版 C 相位（define 前创建 detached 元素 → define）红 `expected 1 but got 0`——spec upgrade candidates 限**文档树内**元素（detached createElement 产物不被 define 升级，保持普通元素直到连接）——引擎行为正确，探针改「连接后 define」命中 walk 面后全绿。非引擎缺陷。
- 上报是 fire-and-forget（`_zwReportListenerError` 内部自兜底 + 调用点 try 包裹）——升级流程控制流零变更。

## 验证

- **红态**：solo 断言 `expected 0 but got 1`（s357-b 域账 Fail 行）。
- **绿态 solo**：custom-element-exception Pass；probe16（自建，跑后已删）五相位全 Pass——A class 臂 createElement 恰一次、B function 臂恰一次、C 连接 walk 面恰一次、D innerHTML 面恰一次、F listener 内再建 CE 成功（`_zwCeExisting` 复位序判据）+ E 零误报。
- **域 join**（LC_ALL=C + (key,state) sort -u）：A=s357-b 323 键 vs B=s358-b 321 键——A-only 恰 4（custom-element-exception Fail + probe14/15 已删探针 3 行）、B-only 恰 2（custom-element-exception Pass + probe16 Pass）；**P→F=0、T→F=0、零意外位移**；残 8F = probe12/13 自建单列 2 + 143/052 长挂 2 + json-module parse-error-location + import.defer 三案。
- **shim 22 part 拼装 node --check SYNTAX-OK**（纯 JS 无 Rust 改动，clippy/fmt 不适用面）。
- **make test**：3816P/7F EXIT=2（失败集与 s356/s357 基线 7F 逐名同构 = 并行流组合态既有集，本流不入侵）。

## 挂账 / 下一步

- json-module parse-error-location、import.defer 三案（basic-deferred-evaluation/cyclic-tla-defer-eval/tla-deferred-evaluation tentative，独立切片）在册；143/052 流式物化长挂；probe12/13 自建单列；T=5 长尾（load-error-events-3、code-cache-base-url、dynamic-imports-credentials-setTimeout、instantiation-error-8、script-crossorigin-network）。
- s97 insertBefore 臂与 R387 内联臂 prepare 缺口（s357 在册）——无 corpus 钉面不扩刀。
- 语料扩展出 CE 异常相关钉面（web-components 通道 custom-elements 域）再收多 realm 上报面（R187 realmWin 臂）——本切片主 realm 单点已覆盖。

## 日志

- evidence/logs/s358-b-script-element.log（域跑 B 面，join 基线 s357-b-script-element.log；probe14/15 行已随探针删除消失、probe16 行为探针噪声跑后删除）
