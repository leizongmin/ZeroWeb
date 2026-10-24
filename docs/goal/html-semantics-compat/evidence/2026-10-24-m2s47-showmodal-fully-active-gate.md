# M2 切片 47 — showModal fully-active 门——sel/handle 收养章 + 主文档 append 族对称删章（R5250）（2026-10-24）

## E 族重分诊（本切片主产出之一）

s46 分诊将「dialog API 语义 5F」列为可 shim 收口候选。逐项核证后收窄为 1F：

- **close-via-attribute 2F → 挂账（Chrome-parity 同红）**。现行 HTML spec（2026-09
  版，本切片拉取核对）dialog attribute change steps 只跑 cleanup/setup steps，且
  明文「The `close` event will not be fired」；WPT 测试链接的 whatwg/html#5802
  （"Dialog should be better-behaved on misuse"）至今 open，测试 2024-05 被移为
  tentative 即提案未落地。实现该行为反而背离真实浏览器。
- **dialog-active-document 1F → 挂账（接口面缺口）**。`createHTMLDocument()`
  产物文档的 `createElement('dialog')` 是 R3018 `_zwMEl` 本地节点（无接口 IDL
  接线），`showModal` 方法不存在——`TypeError: dialog.showModal is not a
  function`。修复属 detached-doc 元素模型接口面工程，非 dialog 切片可收。
- **dialog-open-movebefore-setup 1F → 改判 A/B 族（特性栈未实现）**。依赖
  Element.moveBefore（shim 无）+ closedby=closerequest + userActivation 离散组 +
  ESC 栈顶关闭，与 light-dismiss 特性栈同底座。
- **dialog-showModal detached-throw 1F → 本切片收口（GAIN 1）**。

## 根因（showModal「not attached to any pages」）

spec showModal 步 2/3：未连接抛；node document 非 fully active 抛。连接门
（R3290 既有）对第一断言面工作（探针证实 detached showModal 正确抛
InvalidStateError）。第二断言面：detached dialog 经 `doc.body.appendChild(dg)`
移入 `implementation.createHTMLDocument()` 文档后——

1. M3 扩批 XLIII 的 `_zwUnmarkRemoved`（part03 body appendChild 通用路径，
   「新父即脱离 removed 态」有意行为）清掉移除标记；
2. host 快照滞后，`__zw_contains('html', sel)` 恒返 1（探针实证移除后仍 1）；
3. 连接门双信号均判连接 → showModal 误开。

「node document ≠ active document」在 JS 侧的唯一信号是收养章：R191/R327/
R330 家族已对 handle 形态落 `__zwAdoptDocByHandle`，plain/sel 形态只在代理对象
上盖 `__zwAdoptDoc191/330`（showModal 持 `(key, sel, handle)` 三元组不可达）。

## 修复（R5250 四刀）

- **part03 `_zwUnmarkAdoptDocSel`**：sel 键控收养章清除 helper（`_zwUnmarkRemoved`
  旁，append 族挂点共用）。
- **part03 `_r191adoptBody`**（detached/iframe doc body appendChild 通用路径）：
  proxy 携 `__zwSelector` 时落 `globalThis.__zwAdoptDocBySel[sel] = doc` 章。
- **part01 `_zwDialogShowModal`**：连接门后加 fully-active 门——owner doc 经
  `__zwAdoptDocByHandle`（handle 形态）或 `__zwAdoptDocBySel`（sel 形态）读取，
  非空且 ≠ `globalThis.document` → 抛 `InvalidStateError`（spec 步 3，
  #dom-dialog-showmodal）。
- **part04 主文档 append 族五处对称删章**（重插主文档恢复 fully active）：
  R334 appendChild 两分支（6237 sel 子重插 XLV 挂点、6270 主臂）、R334b 带位
  变体（7378）、R86 append 循环（6559）、R87 insertBefore（7908）。

首版只补 R334/R334b/R87 三处，契约 (c) 回插臂红——探针定位 `document.body.
appendChild(sel 子)` 实际走 XLIV/XLV 分支（6237）与 R86 循环（6559），补齐后
契约全绿。教训：**unmark 挂点必须全量枚举，按「移除标记被清但删章未跑」的
表现反查遗漏分支**。

## 验证

- dialog-showModal 单案 10 Pass/1 Fail：目标子测试「When the document is not
  attached to any pages, showModal() should throw.」**Fail → Pass**；唯一余
  Fail 为「most recently opened is rendered on top」（elementFromPoint 渲染流
  面，C 族既有挂账，与本切片无关）。
- interactive-elements 全域 98 files 482 subtest **424P/56F/2T，对 s46 逐位
  REG 0 / GAIN 1**（GAIN 即目标子测试）。
- zero-engine lib **3443P/0F**（3442 + 契约 part79）。
- 契约翻转力实证：R5250 门条件临时短路 → 钉 (b) 红（收养态误开复活）→ 还原。
- fmt clean；clippy --workspace -D warnings clean（test-guard 包裹）。
- release runner 重建并核符号（`_r5250OwnDoc`×4、`__zwAdoptDocBySel`×14）。

## 契约（part79）

`show_modal_fully_active_gate_r5250`：(a) detached showModal 抛
InvalidStateError（既有连接门防回归）；(b) 收养进 createHTMLDocument 文档后
showModal 抛（fully-active 新门——连接门此态已失效）+ (b2) 抛后不写 open 属性；
(c) 回插主文档 showModal 成功开（删章恢复 fully active，不误伤）；(d) close
正常关（状态机无残留）。

## 已知边界

- iframe 文档元素同样被 R191/R327 盖收养章 → fully-active 门对其会抛。spec 中
  rendered iframe 文档是 fully active（FIXME: 浏览上下文树 active 判定；corpus
  无 iframe 内 showModal 面，暂按 not-active 收窄）。
- handle 形态收养章（`__zwAdoptDocByHandle`）主文档重插无对称清除（既有机制
  面貌，ownerDocument trap 同样残留）；sel 形态经本切片五处删章已闭环。
- 文档级/docEl 级 appendChild 的收养路径（R191/R330 doc 级）未落 sel 章——
  corpus 无对应 dialog 面，按最小实现不扩。

## 门禁

零新增依赖；corpus runner release 二进制含 R5249+R5250 符号。数据
[2026-10-24-m2s47-showmodal-fully-active-gate.json](2026-10-24-m2s47-showmodal-fully-active-gate.json)。
