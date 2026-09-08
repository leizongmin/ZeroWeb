# K3 残余切片 A+B+C — 隐式提交 default button 规则 + insertAdjacentHTML 同步子视图 + apply 消退补偿

**日期**: 2026-09-07
**驱动用例**: WPT `html/semantics/forms/form-submission-0/implicit-submission.optional.html`
**commit**: 7545432d6

## 基线 → 结果

| 状态 | keyboard 套件 | implicit-submission.optional.html |
|---|---|---|
| 切片 A+B 前 | 15P | 0P/3F（populateForm 前置 TypeError 整簇死） |
| 切片 A+B 后 | 16P | 2P/1F |
| **切片 C 后（终态）** | **18P** | **3P/0F（全 Pass）** |

- ✅ subtest 1「Submit event with a submit button」：submitter 断言（`event.submitter ===
  form.submitButton`）+ SubmitEvent/bubbles/cancelable 全绿
- ✅ subtest 3「Submit event with disabled submit button」：unreached_func 守卫绿（disabled
  default button 隐式提交无动作，send_keys 正常 resolve）
- ❌→✅ subtest 2「Submit event with no submit button」：初判 listener 键错位，切片 C
  （apply 消退补偿清除）闭合——根因见下

## 落地面

### 切片 A（shim 同步子视图）

上轮记录的 implicit-submission 3F 根因为 `populateForm` 前置
`getElementsByName(frameName)[0].nextSibling` 断链——host mutation apply 前 JS 视图 stale。
本轮探针（`test_insert_adjacent_html_fusion_view_r3254_k3`）实证：getElementsByName 命中 1
（apply-pending 查询视图）、nextSibling=null、childNodes.length=0。修复（R293/R304 先例扩展）：

1. `insertAdjacentHTML` sel 路径同步视图三件补偿：插入父站定址（beforebegin/afterend=父）+
   `_zwChildBaseCache` 基底置空 + 解析顶层子挂 `_zwSelPendingParent` 槽 + parentNode 重指宿主
   容器 proxy（R136 同款）
2. 兄弟 getter sel→pending 身份归一：`_zwPendingParsedForSel`（tag+id+name+class 签名唯一
   匹配 pending 解析节点，pending 集空零开销）→ `__zw_parent` + `__zw_element_children`
   （apply-pending 视图）按 sel 定位索引取兄弟 sel proxy——返回 identity 与查询入口同族
3. FORM named access：`form.text` / `form.submitButton`（spec
   https://html.spec.whatwg.org/multipage/forms.html#dom-form-nameditem）——FORM gate +
   `_formControls` 名/id 首匹配

### 切片 B（engine/webview/page-runtime 隐式提交规则）

spec https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#implicit-submission：

1. `default_submit_button_selector`：表单 tree order 首个 submit button（`<input type=submit|
   image>` / `<button>` 非 button/reset）。**实现勘误**：逐节点判定直读 DOM 属性
   （`element_local_name` + `get_attribute`），不经字符串 selector 重解析——无 id 控件
   `stable_selector_for_node` 恒落 tag 回落（"input"），querySelector 恒命中首个同 tag 节点
   （首版 bug，subtest 1 由此不判定 submit button）
2. `enclosing_form_selector` 升级 `unique_selector_for_node`：无 id form 恒落 tag 回落
   `"form"`——多 form 文档 querySelector 恒命中首个（WPT 多 subtest 累积多 form 实证，
   subtest 2+ 隐式提交会命中错误表单）
3. `PlannedEvent.submitter` 字段 + `plan_submit`/`dispatch_planned_event` 全链透传
   （SubmitEvent.submitter 经 R2984 `detail.submitter` 通道；此前 submit 派发漏传 submitter
   ——event.submitter 恒 null）
4. webview Submit 分支 default button 三规则：click 路径（selector=按钮）→ submitter=按钮；
   Enter 隐式路径 → ①default button enabled→submitter=按钮 ②disabled→提交无动作（**返成功
   零 effects 而非 noop**——runner send_keys 对 noop(NotApplicable) 视为错误抛出，WPT
   unreached_func 守卫期望 send_keys 正常 resolve）③无按钮→submitter=None
5. `script_control_disabled_probe`：disabled 状态 live 反射探针

## 残余 1F 精确根因（subtest 2）→ **切片 C 已闭合**

初版根因假设（applied-view 与 live 再解析的序列化保真分歧）**被探针证伪**：
`test_serialization_form_roundtrip_r3254_k3_probe` 实证 engine 重放
（`apply_mutations_to_html`）serialize→re-parse 往返对连续 form 插入**完全保真**（两次 apply
后 2 form 各含 2 input）。

真实根因（页面级证据）：`document.body.children` 视图出现**双计副本**——
`IFRAME[path]; FORM[path]; IFRAME[path]; FORM[path]; SCRIPT; SCRIPT; IFRAME[]; FORM[];
IFRAME[]; FORM[]; INPUT[]; INPUT[]; INPUT[]`（前 6 项 = host applied view 带 path 选择器，
后 7 项 = **补偿 overlay 的无选择器副本**）。机制：insertAdjacentHTML 同步视图补偿把解析顶层
节点挂 `_zwSelPendingParent` 槽入 pending 桶；host apply（`__zw_apply_generation_bump`）后
这些节点已进 host 快照（基底含），但桶内补偿条目不清 → 融合视图 = 基底 + overlay 双计；
subtest 2 的 populateForm 命中的 `nextSibling` 是副本（无真 sel），listener 键与派发键错位。

**切片 C 修复**：`__zw_apply_generation_bump` 定点清除 sel-less 解析补偿节点（全局
`_zwPendingAdded` + `_zwPendingAddedById` + 所属父桶 + 槽位删除；handle 节点 identity 记账
不受影响）。**implicit-submission.optional.html 3/3 全 Pass**。

## 验证

- keyboard 套件（`make testharness-keyboard`）：15P→**18P**（implicit-submission 3/3 全
  Pass——切片 A+B 后 2/3，切片 C 补齐第 3 案；其余用例零回归，paged/scroll-padding 2T 为
  scrollIntoView rect 跨域既有项）
- 单测 +5 组：`test_insert_adjacent_html_fusion_view_r3254_k3`（populateForm 同形两轮序列 +
  named access + listener/dispatch 同 identity）、`test_default_submit_button_selector_r3254_k3`、
  `implicit_submission_default_button_rules_r3254_k3`（三规则静态表单）、
  `implicit_submission_inserted_form_r3254_k3`（插入式表单 e2e）、
  `test_serialization_form_roundtrip_r3254_k3_probe`（serialize→re-parse 保真回归守卫）
- engine 2647 / page-runtime 130 / webview 694 全绿；clippy `-D warnings` 零警告；fmt 干净
