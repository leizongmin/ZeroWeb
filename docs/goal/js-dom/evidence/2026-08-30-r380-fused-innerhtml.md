# R380 — sel 域融合 innerHTML + 克隆纯文本 registry 子（pending-apply RFC pa3 前置，remove-next-sibling 1F→0F）

**日期**: 2026-08-30
**切片**: pending-apply RFC pa3 前置两件（`crates/engine/src/js_dom_shim/part04.js`）
**轮次来源**: 上一 session R380 半成品（429 中断遗留未验证 WIP）+ 本轮死循环 hunk 证伪移除

---

## 1. 修复两件（part04.js）

### ① sel 域融合 innerHTML（innerHTML getter sel 分支）

- **根因**（R379 pa1 §3.3 探针实证的 Fail 实际形态）：`container.innerHTML` 直读
  `__zw_get_inner_html(sel)` host 快照——replaceWith/removeChild 等同 turn mutation
  已 enqueue 但 host 未 apply，读串是 apply 滞后旧树。
- **修复**：pending 桶（`_zwPendingByParent.get(sel)`）非空时改从 `_childNodeList(sel, null)`
  融合视图序列化（R51/R309/R322 overlay 补偿链 + R379 pa2 移除标记语义——标记中子
  不出现在融合视图）。序列化：text 转义（`_zwMEscapeText`）/ comment / 元素递归
  `outerHTML`。桶空时保持 host 直读零变化（热路径不受影响）。序列化异常回落 host 快照。

### ② innerHTML setter 克隆路径纯文本 registry 子

- **根因**：R151 的 registry 同步填充只处理 markup 形态（`ih.indexOf('<') >= 0`），
  纯文本 innerHTML（script 源码 / `<span>New </span>` 的 "New " 文本）落 else 分支
  **清空** registry → 克隆 script 的 `_handleChildren[scriptH]` 恒空 → R377 插入期
  脚本钩子源码收集失败 no-op（R379 pa1 探针实证 `r377-kids:(empty)`）。
- **修复**：非 markup 非空 = 恰一个 text 子——`__zw_create_text(ih)` + `_textHandles`
  印记 + `_handleChildren[nh] = [_wrapHandle(tn)]`（与 appendChild 的 text 子同形态）。
  空串保持清空语义。

## 2. 证伪记录：fragment 展开路径「registry 文本子随迁」（实验移除）

上一 session WIP 的第三 hunk（part05 `_insertAdjacentVariadic` R321 分支把
`_handleChildren[fragmentHandle]` 搬给首个克隆子 handle）在 `make test` 全量单测中
**100% CPU 死循环**（`test_fragment_flatten_all_insertion_paths_e2e` 挂起，clean HEAD
0.05s 通过）。

- **根因**：fragment registry（`createDocumentFragment` 产物）只存**顶层子**数组；
  把顶层数组 `[a, b]` 搬给首个克隆子 a 的 handle 键 → `_handleChildren[aH] = [a, b]`
  **自环** → `_ceApplyConn`/`_zwHCCollectSubtree` 的 DFS 无环检测死循环。
- **语义错误**：script 的 text 子根本不在 fragment 顶层 registry 中（`_zwMEl
  appendChild` 的后代自记账已覆盖该面）——随迁前提不成立。
- **处置**：hunk 整体移除（非 disable）。教训：registry 结构性搬移必须先画清
  「谁在哪个键下」——顶层数组 ≠ 每子的后代数组；DFS 消费面（`_ceApplyConn`、
  `_zwHCCollectSubtree`、`_mo_notify` 上行）对环零防御。

## 3. 效果

- **driving 用例** `dom/nodes/remove-next-sibling-during-replace-with`：**1F→0F**
  （自 R328 立项起 52 轮的持久 Fail；R377/R379 两轮审计的终端收口）。断言串
  `<span>New </span><span>content</span>` 全等（融合序列化直接产出）。
- 全量 dom sweep（polyfill，TIME_LIMIT=2400）：**55808P / 11F**——真实 Fail 文件集
  12→11（该文件转绿），其余 11 项与 R379 备档集合逐文件恒等（MutationObserver-document
  3F parse-time 架构域 / remove-and-adopt-thcrash window.open / click-on-absolute-pseudo
  Chromium 专有 / ranges dataChange+replaceData 2F 游离树堆积 / historical 3F stale /
  window-extends 2F EventTarget 继承域转档）。
- 新单测 `r380_fused_innerhtml_and_text_registry_children`（part25.rs 新段）：锁定
  ①克隆 script 有 registry 源码 ②replaceWith 同步标记 ③融合 innerHTML 与 WPT 期望
  串全等 ④**死循环回归不复发**（replaceWith(fragment) + script.remove() 序列完成）。

## 4. 验证（landing 门）

| 门 | 结果 |
|----|------|
| driving 用例（双路径） | polyfill Pass / native（ZW_NATIVE_DOM=1）Pass |
| 全量 dom sweep（polyfill） | 55808P / 11F（-1F，集合恒等收窄） |
| 全量 dom sweep（native） | 55806P / 12F / 16T——Fail 12 项 = polyfill 11 项 + abort/reason-constructor 1F（iframe DOMException per-realm，clean HEAD 同败预存）；Timeout 轮转族 16 项经 Node-parentNode A/B 复核 clean HEAD 恒等（R355 先例的环境噪声族） |
| engine v8 / quickjs 单测 | 2504P / 2507P 全绿（含新 part25 段） |
| clippy v8 / quickjs | 双矩阵零警告 |
| fmt | 无 diff |
| 已知挂起回归 | test_fragment_flatten_all_insertion_paths_e2e 复测 0.08s 通过（hunk 移除后） |

## 5. 教训

1. **实验性 registry 搬移 hunk 在全量单测门必现形**——上一 session 中断前未跑
   `make test` 的 WIP 带病过夜，接手轮第一步是全量单测而非直接跑定向 WPT。
2. DFS 消费面（_ceApplyConn/_zwHCCollectSubtree）对 registry 环零防御，任何写入
   `_handleChildren` 的新代码都必须自查「数组内容是否含自身/祖先」。
3. 单测断言须与上游用例**同构**（本轮首版误假设 script 执行移除 b——WPT 用例实际
   在脚本执行前显式 `container.querySelector('script').remove()`，b 存活是期望行为）。
