# M2 切片 32 — popover 有效性语义：`:popover-open` matches 通道与 hidePopover lenient 化（2026-10-23）

## 范围

s31 挂账序项「popover 有效性语义」：dialog-popover-closedby complex/simple
两语料的红因收口第一刀。complex 40F 统一红因（`:popover-open`
SyntaxError）消除、simple 文件级 Timeout 解体（+1 绿），残余红全部迁移
为可测的行为断言面并钉定下一切片缺口。

## 红因链（s31 账本 → 本切片破案）

- **complex 40F**：`assertStates` 用 `element.matches(':popover-open')`
  （DOM matches 路径），host 选择器引擎无该伪类 → SyntaxError →
  promise_test 未处理拒绝全红。`matches(':modal')` 已有 R4200 裸伪类
  JS 侧特判不在此列。
- **simple 1F**：`resetDialogOuterTest` 开头 `popover.hidePopover()` 对
  未 showing popover——shim R3071 过度抛出
  `InvalidStateError: not showing` → 未处理拒绝。
- **simple Timeout**：replica promise_test 不接体 rejection（未处理拒绝
  归因当前活动测试），其余 5 体在同点 reject 但未完成 → 挂死
  completion callback（`pending:5, tests:6` 实测吻合）。

## spec 钉定

hide popover 算法 step 2：「validityResult 非 true **且为 DOMException**
才抛」——check popover validity 对「有 popover 属性但显隐态不匹配」
（hidden / disconnected-not-showing）返回 false 布尔而非异常 →
hidePopover 静默 no-op（https://html.spec.whatwg.org/multipage/popover.html#dom-hidepopover）。
语料自带权威佐证：popover-utils `assertIsFunctionalPopover`
「Calling hidePopover on a hidden popover should not throw.」注释 +
上游 green（leading hidePopover 为静默 no-op）。仅非 popover 元素 →
NotSupportedError；showing 但已断连 → InvalidStateError。

## 落地三件（R5233）

1. **shim part01** `_zwHidePopover` lenient 化：attr 缺失
   NotSupportedError → 态不匹配静默返 → showing+断连
   InvalidStateError → 正常隐走 beforetoggle/toggle 事件流。`_zwTopLayer`
   注释块 `:popover-open defer` 子句随实现更新。
2. **shim part04** matches 陷阱 `:popover-open` 裸伪类特判（`:focus`/
   `:modal` 同款 JS 侧特判设计，R4200 紧随）：showing = `_zwTopLayer`
   成员 + 仍带 popover 属性（`_zwReadPopover`）+ connected
   （`_zwIsConnected`）。已知近似：`dialog[popover]` 经 showModal 入
   top-layer 但 popover 显隐态未 open 会误命中（本集群无交集面）。
3. **契约测试** part09 `test_popover_api_r3071` ④：未 showing
   hidePopover 期望翻转（InvalidStateError → 静默 no-op）+
   `:popover-open` matches 三态断言（show 前后随 showing 翻转）。

showPopover 对已 showing 仍抛 InvalidStateError（R3071 语义维持，
spec 现行为 lenient no-op——无语料红面，不扩切片）。

## 集群验证

- dialog 集群 82 files **411 subtest 321P/89F/1T**（数据
  [2026-10-23-m2s32-popover-open-semantics.json](2026-10-23-m2s32-popover-open-semantics.json)），
  同 key 406 对 s31 账本逐位 diff：回归 **0**、增益 **0**（既有绿
  零翻转）。
- **closedby-simple：文件级 Timeout 解体** → 6 子测收敛 5F+**1P**
  （Popover parent, dialog closedby=none child 全链绿：reset lenient
  复位 + clickOn + ESC + 二次 ESC 断言全走通）。体不再 reject，
  completion callback 正常回调。
- **closedby-complex：40F 红因迁移** SyntaxError → 统一行为断言
  （见下节缺口钉定）。
- 集群外复核：select-inside-top-layer / customizable-combobox-keyboard
  的 `matches(':popover-open')` 断言面随特判生效（此前同死
  SyntaxError）；残余红为 `:open` 伪类等既有缺口，与本切片无关。
  select-dialog-mode-focus 2F 维持（`:open` SyntaxError）。
- focus-after-close.html Timeout（s31 即有，`pending:1 tests:7`）既有
  挂账维持，与本切片无关。

## 探针破案（zz-probe-s32-nested，已删）

最小嵌套栈（dialogA[closedby=any] > popoverA[popover] > dialogB >
popoverB）逐步快照 + clickOn(unrelated)：

```
init → dA/pA/dB/pB 全开 ✓（setup 全绿）
click1 → dA=false pA=true dB=false pB=true
```

modal/modeless 同象。对照语料期望（一次 click 只关最内层链：dB+pB，
dA 保持 open）钉定两缺口：

- **缺口 ①popover light dismiss 未建**：auto popover 不随 outside
  click 关闭（R3071「light-dismiss / auto 关闭其他 popover defer」
  至今未回填）。
- **缺口 ②dialog light dismiss 链停**：一次 click 关闭**全部**
  closedby=any dialog 而非仅 topmost clicked dialog 链（spec
  light dismiss 一次 click 一链；s30 dialog-closedby-show-stacked
  通过因纯 dialog 叠栈几何覆盖未暴露）。

## 挂账（本轮钉定）

- **E popover light dismiss + 链停语义**（本流，下一切片主目标）：
  engine click 路径补 auto popover light dismiss（topmost-chain
  一次一链）+ dialog light dismiss 链停修正 + ESC 栈序（simple
  「Popover parent, dialog closedby=closerequest child」第二 ESC 断言
  面）。closedby-simple 5F / complex 40F 等此切片。
- A 强制同步布局冲刷、B abspos 收缩（渲染流）、D getComputedStyle
  路径 :modal 注入（低优）维持。
- focus-after-close.html Timeout 排查（低优，独立面）。

## 账本与门禁

- zero-engine lib **3429P/0F**；clippy `-D warnings` clean、fmt clean；
  零新增依赖。runner release 二进制重建（shim include_str! 进二进制）。
- 探针语料已删，集群干净复跑。
