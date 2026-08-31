# M3 R98 — lit 响应式更新链落地（define observedAttributes + CE accessor 派发）

**日期**: 2026-08-17
**Commit**: `d90be269d`
**Milestone**: M3（真实 SPA / Web Components 端到端验收）
**前置**: R97（lit 首渲染落地）

## 背景与目标

R97 打通 lit 首渲染后，响应式更新链是 Web Components「真实运行」的最后一块：
property set → requestUpdate → 二次 render → diff commit。探针实证该链断
（`el.name = 'Updated!'` 后 `updateComplete` 不变、shadow root 文本不更新）。

## 三重根因与修复

| # | 根因 | 修复 |
|---|------|------|
| 1 | `customElements.define` 不读 `ctor.observedAttributes`（spec define step 5 的 Get）——lit 静态 getter 内调 `finalize()` → `createProperty` 装 prototype accessor；不读则 accessor 从未安装（`getOwnPropertyDescriptor(GreetingEl.prototype,'name')` === undefined） | part03 define 注册后 `void ctor.observedAttributes`（异常吞） |
| 2 | set trap 不派发原型链 accessor setter（R93 只做了 get 镜像）——lit setter 内 `this.requestUpdate` 永不执行 | part04 set trap 顶部 CE-scoped accessor-setter 派发（8 层 own 命中，this=元素 proxy） |
| 3 | symbol-keyed 写丢失（lit fallback `this[s]=v` 以 Symbol 存值被 String() 后写 host attr）+ 首层 accessor getter 优先级（被 shim 反射属性分支先吞） | set trap symbol 直入 `_expando`；get trap 顶部 CE 首层原型 accessor getter 优先 |

## A/B 捕获的回归与收窄

初版 accessor 派发对**全部元素**生效，A/B 捕获两处回归：
- `Element-getElementsByTagNameNS('*','body')` 在普通 div 上破坏（方法读取路径变化）
- `HTMLCollection-supported-property-names` expando 1 例

**收窄**：派发条件加 CE 判定——首层原型的 constructor 须在 customElements
registry（`getName` 命中）。非 CE 元素零路径变化，两处回归归零。

## 验收

- **lit e2e 组 E（新，`lit_reactive_update_lands`）**：三段式断言
  - MID: `t1:Hello, ZeroWeb!|uc-changed:true|pending-at-set:true`（首渲染正确 + set 触发 requestUpdate）
  - POST: `t2:Hello, Updated!!|pending:false`（二次 render 文本 commit + 完成）
- **lit 全组 5/5 绿**（component_chain / template_content / first_render / render_direct / reactive_update）
- **WPT dom A/B**（clean-HEAD 二进制）：nodes 6673=base（per-case 一致 + name-validation 4F flake 消退）、collections 48=48（per-case 一致）、traversal 1595、events 236
- **单测**：engine v8 **2205** / quickjs **1431** / integration **780**；`make test` exit 0
- **质量**：fmt / 双矩阵 clippy 干净；pre-commit-guard PASS

## 教训

1. **get/set trap 的原型链派发必须 CE-scoped**：非 CE 元素的原型链虽是 shim
   自建（全 data property），过宽派发仍改变方法读取路径——A/B 是唯一安全网。
2. **响应式链分两段断言**：set 时点（同步观察 requestUpdate 触发）与
   post-drain（异步观察 commit 结果）是独立观察面，混读会误判。
3. **define 的 observedAttributes Get 是 lit finalize 的唯一触发点**——CE 桥
   的 spec 步骤覆盖度直接决定框架兼容性。

## M3 进度

- WC 端到端：customElements 五件套 + lifecycle + Shadow DOM + **真实 lit 库**
  （首渲染 R97 + **响应式更新 R98**）——DC-2 第二项实质达成
- 剩余：SPA 框架端到端（React/Vue 之一：hydration + 事件 + reconciliation）；
  lit 事件链（click → handler → state 更新 → 重渲染）作为 WC 交互面收尾候选
