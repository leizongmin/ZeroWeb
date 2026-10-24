# M2 切片 51（s51）——R5256 dialog open 属性移除面关闭步 + detached 文档 dialog 三方法；A/H declarative shadow 联合切片分诊落档

日期：2026-10-24。基线 s50（428P/54F/1T，483 子测试）→ 本轮 **431P/51F/1T（483 子测试），REG 0 / GAIN 3**。

## 刀前分诊：A 族 16F 与 H 前置 4F 同根——declarative shadow 提升缺失（挂账改记）

`dialog-popover-closedby-complex.html` 16F 全部是「shadow DOM slots」变体（normal nesting 同结构 20 子测试全 Pass）。探针实证链：

- runner 页面视图里 `<template shadowrootmode=open>` 元素**消失**、内容平铺进 light DOM（host.children=[DIALOG,DIV]、qsa('template')=0；part03:7639 既有注记「template contents 内联（parser 占位，M2 收口项）」互证）——slot 归属信息被销毁，popoverB 与 dialogB 成兄弟，light dismiss 包含链断裂 → 误关。
- `host.shadowRoot` 返 null（声明式提升未实现，与 s48 H 前置分诊一致）；shim `_attachShadow` 是 JS 虚拟树（fragment 注册表），不下传 dom crate（`attach_shadow` 生产零接线）；布局引擎 shadow 遍历（tree.rs:3623 slot 解析）是无米之炊的前瞻代码。命令式 attachShadow 探针：shadow 内容 rect=0,0,0,0、elementFromPoint null——**shadow 几何面未通**。
- clickOn 坐标驱动（getBoundingClientRect + Actions pointerMove），shadow slots 提升后无几何 → 点击落 (0,0) 全员误关，现存 4 个 shadow-slots Pass 子测试会 REG——**shim 虚拟单边提升不可落地**。
- web-components 流（2026-09-11 归档收口）明示「Shadow DOM 渲染级 composed tree 属深结构，等用户点名后协调 rendering-compat」。

**结论**：A 族 16F + H 前置 4F（focus-previous-outside 3F + form-submit-dialog-shadow 1F，纯 JS 语义无几何依赖、shim 虚拟提升可翻但其前置相同）统一记「declarative shadow parser 提升 + shadow 几何渲染流联合切片」，渲染流半边维持用户决策门禁（与 parser form pointer 4F 缓行决策 2026-10-23 同域）。本轮转红名单中本流可独立收口的 dialog shim 面。

## 刀一：open 属性移除面关闭步（part01 helper + part04 两钩子）

`dialog-close-via-attribute.tentative.html` 2F：removeAttribute('open') 于 open dialog 未派 close。shim 旧注记「直接 removeAttribute 不派 close——real browser 一致」已被 2025 规范变更（the-dialog-element attribute change steps → dialog closing algorithm）推翻。

- **`_zwDialogAttrClosedR5256`**（part01，close 家族并列）：清模态态/top-layer/组栈 + close/toggle 事件排队（与 close 核心同队列）。差异面：无 cancel（非 request-close）、无 beforetoggle（corpus 无钉、最小 delta）、焦点**不恢复**（tentative 断言 previously focused 不聚焦）、returnValue 不变。
- **removeAttribute 尾钩**（part04）：pre-snapshot（属性在场读 wasOpen + watcher 门）→ 真移除后调 helper；IDL falsy 臂同款接通（`open = false` 直写 = 属性移除面）。
- **close watcher 门**（首版 REG 修正）：首版全域对账 `dialog-open.html :: On setting it to false, the close event should not be fired` REG（2F→3F 边缘）——属性直开（`d.open = true` 未经 show()/showModal()）无 close watcher，移除仅隐藏不关闭。门实现为 pre-snapshot 双信号（模态态 `_zwDialogModal` / R4300 组栈成员）+ wasOpen，两调用点均在删态前取样；show()/close() 循环生命周期与 watcher 销毁语义对齐（开即压栈、关/移除即 splice）。

## 刀二：detached 文档 dialog 三方法（part03）

`dialog-active-document.html` 1F：`createHTMLDocument().createElement('dialog')` 产物无 showModal（TypeError 非 DOMException）。detached createElement 的 `_zwMEl` 产物接三方法：showModal() 抛 InvalidStateError（文档永非 fully active——无 browsing context）、show() 静默返（spec not-fully-active → return）、close() 状态清理 no-op（FIXME: wrapper 无派发链，open 态 close 事件队列面未建，语料无钉）。

## 契约（part82 `dialog_attr_removal_closing_r5256`）

(a) 模态移除：立即非 :modal、close 异步恰一次、cancel 零次、toggle open→closed；(b) IDL falsy 同面；(c) closed 再移除零事件（wasOpen 门）；(d) 同任务 showModal+移除 toggle 合并单发（R4200 去重）；(e) detached showModal 抛 InvalidStateError + matches('[open]') false + show/close 不抛；(f) 属性直开移除零 close（watcher 门）。时序模拟：记录式 timer stub + 手动 fire（part81 同款）。

## 验证

- close-via-attribute.tentative **2F → 2P**；active-document **1F → 1P**；dialog-open.html 维持 3P（watcher 门修正后）
- engine lib：**3447P/0F**（+契约 part82）；fmt --check 干净；clippy -p zero-engine 0 告警
- 全域 interactive-elements：**431P/51F/1T（483 子测试）对 s50 逐位 REG 0 / GAIN 3**（python 逐位 diff，98 files 一致）
- release runner 含 R5256 锚点（grep 实证 5 处 `_zwDialogAttrClosedR5256`）

## 余账

- **declarative shadow 联合切片**（原 A 族 16F + H 前置 4F）：parser 提升 + shadow 几何渲染流半边——待用户点名协调 rendering-compat 后联合立项
- F 族 name-attribute 3F、C 族渲染流 20F、pointer-capture 指针管线深水维持挂账
- detached wrapper 的 close 事件队列面（无钉 FIXME）随 declarative shadow/包装态底座后续
