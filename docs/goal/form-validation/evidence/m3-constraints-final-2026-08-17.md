# M3 证据：提交阻断与事件序列全链路（2026-08-17）

## 终态

constraints 套件（45 上游 `html/semantics/forms/constraints` 用例 +
`html/semantics/forms/the-form-element/form-requestsubmit.html`（10 子测试）+
`form-checkvalidity.html`（1 子测试））：

| 指标 | 值 |
|------|-----|
| 子测试 Pass | **919** |
| 子测试 Fail | **0** |
| Timeout | 16（14 个 `-manual` 交互用例 + 2 个 crash 回归用例——headless 预期） |

## form-requestsubmit 子测试结果（M3 驱动用例）

| 子测试 | 结果 |
|--------|------|
| Passing an element which is not a submit button should throw | ✅ TypeError |
| Passing a submit button not owned by the context object should throw | ✅ NotFoundError（DOMException code=8；含 detached submitter） |
| requestSubmit() should accept button[type=submit], input[type=submit], and input[type=image] | ✅ |
| requestSubmit() should trigger interactive form validation | ✅ invalid 事件派发 |
| requestSubmit() doesn't run form submission reentrantly | ✅ 重入守卫（requestSubmit+requestSubmit / requestSubmit+click / click+requestSubmit 均 1 次） |
| requestSubmit() doesn't run interactive validation reentrantly | ✅ |
| requestSubmit() for a disconnected form should not submit the form | ✅ |
| The constructed FormData object should not contain an entry for the submit button | ✅ |
| Using requestSubmit on a disabled button should trigger submit but not be visible in FormData | ✅ |
| The value of the submitter should be appended, and form* attributes handled | ✅（form.matches(':invalid') 聚合 + formnovalidate 跳过） |

## 关键决策

1. **applied view 范围 = InsertAdjacentHtml only**：查询类回调读「快照 + pending
   结构 mutation 应用副本」。属性级（SetAttr/RemoveAttr）由 latest-wins 队列
   （`__zw_has_attr_lw`）覆盖；handle 链（createElement/appendChild）由 shim 本地
   registry 回落（身份 `===` 须同一 proxy）；Remove/SetInnerHtml 不应用（被移除/
   替换元素的 proxy 属性读取回落快照——R3029/R47/replace_child_e2e 语义保持）。
2. **值缓存双源标记**：`_inputValues` 的 lazy-init 仅稳定键（#id/@handle）；setter
   写入（.value=）经 `_inputValuesSet` 标记始终可用（SetFormValue 在 applied view
   no-op——JS-set 值只能从缓存读回）。
3. **dom 选择器列表**：`Document::query_selector(All)` 顶层逗号拆分（spec 语义）——
   修复 `form.querySelectorAll('input, button')` 空结果（旧实现把列表当单选择器解析失败）。
4. **中间件构建断裂**：zero-browser 默认 quickjs + workspace 统一出双 feature →
   script-sandbox 等无法双编译（R3399 的 8 参 worker 签名 vs quickjs 5 参调用点等）。
   双 feature 编译以 v8 优先（`cfg(all(quickjs, not(v8)))` 门控）修复。
5. **R3048 测试修正**：requestSubmit(submitter) 的 detached submitter 按 spec 抛
   NotFoundError（WPT form-requestsubmit 用例）——测试改用 form 内 sel-based submit
   按钮（快照加 `<button id='sb' type='submit'>`）。

## 验证

- engine 2173 全绿（含 R2825 validation + R3048）
- testharness-canvas 1253 / 0 零回归
- `cargo clippy --workspace --all-targets -- -D warnings` 零警告；fmt 无 diff
- workspace 测试仅 compositor_gpu_dmabuf 失败（GPU adapter 环境依赖，改动前同）
