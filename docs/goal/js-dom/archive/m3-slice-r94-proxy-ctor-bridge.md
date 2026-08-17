# M3 切片 R94 — Proxy-ctor 桥（CE 用户 ctor 体执行）

**日期**: 2026-08-17
**Commit**: `d26f8a29`
**Milestone**: M3 Web Components 端到端（DC-2）
**前置**: R90（CE 升级三缺口修复，记「ctor 体不可重放」限制）、R93（CE 原型方法派发 + 探针取证）

## 问题

R90 记档的已知限制：custom element 的用户 class constructor 体无法在既有 proxy 上执行——
`B.call(el)` 抛 "cannot be invoked without 'new'"；`Reflect.construct` 新建 this 而非复用 el。
升级 = 原型挂接 + connectedCallback 承载初始化（imperative WC 模式），**lit/stencil 的
constructor 内初始化面不可达**——这是 M3「真实 WC 库端到端」的最后已知阻塞。

## 机制（node 探针全套实证）

**不是重放，是 this 注入**：derived class 的 `super()` 返回对象会成为整条 ctor 链的 this。
让 shim 的 polyfill HTMLElement 在 `_zwCeExisting` 已设时**返回既有元素**：

```
class MyEl extends HTMLElement { constructor(){ super(); this.state = 5; } }
_zwCeExisting = el; new MyEl();   // super() 返 el → this=el → this.state=5 落在 el 上
```

探针验证项（全部通过）：identity（inst === el）、chain-set-before-body（体内方法可达）、
多层派生链（B extends A extends hook）、顺序升级各自消费、ctor 内 `new.target` 是用户 ctor
（`customElements.getName(new.target)` 可用）、ctor 抛错后槽已消费不泄漏、嵌套 create（ctor
体内再造 CE 元素）互不干扰、普通 `new Other()` 不受影响。

## 实现

1. **part03 HTMLElement stub → ctor 桥 hook**（仅 polyfill 自建路径——native_dom 模式
   native HTMLElement 已在全局，走 native S5b upgrade slot（R3270 test 已证），本桥零交互）。
2. **`_ceRunCtor(ctor, el)`**：先 setPrototypeOf（体内方法访问经原型链可达）→ class ctor
   走 `new ctor()`（super() 注入 this），function ctor 走 `ctor.call(el)`。class/function
   判别用 `Function.prototype.toString` 的 class 语法探测（`/^\s*class[\s{]/`——语法关键字
   minifier 不可改名）；**否决 .call-抛错探测**：function ctor 体自身抛错会误判成 class 再经
   new 二次执行（双副作用）。异常 try/catch 吞（best-effort 升级不中断页面），finally 清槽。
3. **`_ceUpgradeNode`**（`customElements.upgrade` / 解析子树升级）同样执行 ctor 体
   （spec `custom-elements-upgrades` upgrade step 的 ctor 执行段）。
4. **part06 createElement CE 分支**：setPrototypeOf 换成 `_ceRunCtor`。

## 测试资产

`tests/integration/src/e2e_web_components.rs` 断言组 8 `wc_ctor_body_runs_on_element`：
① createElement 升级 ctor-ran/phase/instanceof ② ctor 内 expando 写入持久 ③
ctor→connectedCallback 顺序（`_seen = ['ctor','conn']`）④ function ctor（.call 注入）
⑤ ctor 抛错不中断（后续元素照常 + 无槽泄漏）⑥ 普通元素不受桥影响。

## 验证

- integration **775**（773 + 组 8）/ engine v8 **2188** / quickjs **1427** / webview v8 **601** 全绿
- WPT dom 四子目录 per-case 与 R93 **逐字节一致**（nodes 6654P/1568F、events 190P/138F、
  collections 48P/0F、traversal 1593P/11F）——零回归
- fmt 无 diff；clippy 零警告；pre-commit-guard PASS

## 对 M3 的意义

lit 的 constructor 内初始化面（attachShadow 准备、属性初始化）现在可达——**真实 lit 库
端到端 fixture 是 M3 剩余主项**（本切片解除了其最后已知阻塞）。

## 延后

- 真实 lit/stencil 库 e2e fixture（M3 收口主项）。
- 解析期元素（parser 建的 `<my-el>`）的升级时机：当前在 `customElements.upgrade` 调用 / 
  append 路径惰性升级；define 时全文档扫描升级的时机对齐待 lit 实测定。
