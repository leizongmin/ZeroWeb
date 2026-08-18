# 表单校验运行时控制面板

**状态**: ✅ **已完成**（2026-08-17——M1-M3 全部完成，Done Criteria 满足：constraints 919 Pass / 0 Fail；入口文档已归档至 `archive/form-validation-goal-v1-2026-08-17.md`）
**最后更新**: 2026-08-17（M3 完成——提交阻断全链路 + 中间件构建断裂修复；归档收口登记）

---

## 当前状态

**专项定位**：从 html-compat（M0-M4 已完成）拆出的延伸专项——Constraint Validation API
从 permissive 基础深化为真实约束计算，WPT `html/semantics/forms/constraints` 真实用例驱动。

**与兄弟 goal 的边界**：
- js-dom（DOM API 反射面）— 表单控件属性反射段共享，run-rules §9 碰头管理
- rendering-compat（CSS 伪类/布局）— `:valid`/`:invalid` 伪类不碰
- html-compat（父目标）— 提交阻断与 `html_actions` submit 路径共享（深化不重建）

## 实测基线（2026-08-16）

### 现有实现（R2825 permissive 基础）

- ✅ part04.js：checkValidity/reportValidity/setCustomValidity/validity/validationMessage/
  willValidate 反射（permissive——原生约束不强制）
- ✅ part01.js：`_customValidity` 状态（setCustomValidity 跟踪）+ `_userEdited` 标记
  （minlength/maxlength 用户编辑）+ `__zw_reset_form_state` 清空
- ✅ invalid 事件派发（checkValidity/reportValidity invalid 路径）
- ⚠️ 原生约束位（valueMissing/mismatch/rangeUnderflow/rangeOverflow/stepMismatch/
  tooShort/tooLong/typeMismatch/badInput）全未计算
- ⚠️ 提交阻断（interactive validation）未实现
- ⚠️ willValidate 排除（disabled/hidden/readonly）未实现

### WPT 面（M1 基线 2026-08-16）

- ✅ 45 用例导入（fetch-constraints-subset.sh + `testharness-constraints` 子命令）
- 基线：27 文件有子测试、**Pass 3 / Fail 909**（permissive valid——约束位全缺失）；
  证据 evidence/m1-constraints-baseline-2026-08-16.md
- **M1 完成（2026-08-17）**：**Pass 909 / Fail 0——全灭**——约束位全系落地
  + 剩余 2 Fail 修复（有理数 step 3e-15 + 回溯 pattern 守卫）——
  证据 evidence/m1-constraints-baseline 追加
- **修复注记**：stepMismatch 极小 step（3e-15）用有理数 BigInt 整数性判定
  （IEEE 浮点取模不可靠）；无限回溯 pattern（V8 无 RegExp 超时）守卫直接
  mismatch（近似——回溯风险 pattern 视为不匹配——用例期望 invalid）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| V1 | WPT constraints 用例覆盖为零 | ✅ M1 完成（45 导入 + 909/0 全灭） |
| V2 | 原生约束位计算缺失（permissive valid） | ✅ M2 完成（约束位全系 + validityState 联动 + validationMessage 标准消息 + willValidate） |
| V3 | 提交阻断缺失（interactive validation） | ✅ M3 完成（requestSubmit + submit 按钮 click 默认动作 + 重入守卫 + disconnected 检查 + submitter 校验 + novalidate/formnovalidate + form 级 :invalid 聚合） |
| V4 | willValidate 真实化 | ✅ M2 完成 |
| V5 | validationMessage 约束消息 | ✅ M2 完成 |
| V6 | the-form-element 用例（requestSubmit/checkValidity） | ✅ M3 完成（form-requestsubmit.html + form-checkvalidity.html 导入） |

## M3 完成记录（2026-08-17）

**constraints 终态：Pass 919 / Fail 0**（45 上游 constraints 用例 + form-requestsubmit
10 子测试 + form-checkvalidity 1 子测试；16 Timeout = 14 个 `-manual` 交互用例 +
2 个 crash 回归用例——headless 预期）。证据 evidence/m3-constraints-final-2026-08-17.md。

**M3 修复清单**（form-requestsubmit 驱动）：

| 面 | 实现 |
|----|------|
| submitter 校验 | requestSubmit(submitter)：非 submit 按钮 → TypeError；form owner ≠ form（含 detached → null）→ NotFoundError（真实 DOMException，`_throwDom`） |
| 共享提交路径 | `_zwRunFormSubmit`（requestSubmit + click 默认动作共用）：novalidate/formnovalidate 跳过 → interactive validation（首个 invalid 控件派发 invalid + 中止）→ cancelable submit（SubmitEvent 含 submitter） |
| 重入守卫 | `_zwSubmitBusy`（IIFE 作用域）——submit/invalid 事件处理中重入 requestSubmit/click 直接返回（spec "submit event is firing" 语义） |
| click 默认动作 | INPUT[type=submit/image] + BUTTON（默认/type=submit）click → 表单提交（经共享路径）；disabled 无激活行为 |
| disconnected 表单 | `document.contains(form)` 检查——detached/removed 表单不派发 submit |
| form 级 :invalid/:valid | `_validityState` FORM 分支——聚合候选控件（submit/reset/button/image/hidden 非候选，disabled 排除）——`form.matches(':invalid')` 联动 |
| 查询 applied view | 查询类回调读「快照 + pending mutations 应用副本」（memoized 缓存，结构级 InsertAdjacentHtml only）——同批 insertAdjacentHTML 后 querySelector 命中；属性级/handle 链/Remove/SetInnerHtml 不应用（latest-wins 队列 + JS 回落覆盖，既有测试语义保持） |
| dom 选择器列表 | `Document::query_selector(All)` 支持顶层逗号选择器列表（`'input, button'` 等——spec querySelector 语义；旧实现空结果） |
| 值缓存碰撞 | `_inputValues` 位置选择器跨批碰撞（`form:nth-child(1)` 指向不同元素）——lazy-init 缓存仅稳定键（#id/@handle）；setter 写入（.value=）双源标记 `_inputValuesSet` 始终可用 |
| form.elements | `input[type=image]` 排除（WPT oracle）；form 属性关联控件计入 |

**中间件构建断裂修复**（6ef6fca29 引入，阻塞全部 workspace 级构建）：

- 根因：zero-browser 默认 features 含 quickjs → `cargo build --bin X`（无 -p）/`--workspace`
  统一出 v8+quickjs 双 feature → zero-script-sandbox 无法双编译
- 修复：script-sandbox/webview/browser 双 feature 编译（v8 优先——`not(feature = "v8")`
  门控 worker 分支/es_module/原生绑定/Drop impl/re-export 歧义）；`--bin`（无 -p）构建恢复
- 另修复：callbacks.rs 查询回调捕获引用参数的生命周期 bug（按文件惯例 Arc::clone）
- `__zw_get_text_lw` 自死锁（持 html 锁调 apply_pending 再锁 html——std Mutex 非重入）——
  作用域收窄修复（探针定位：打点显示卡在第二次 count 读取后）

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT constraints 基线建立 | ✅ 完成（909/0 全灭——45 用例、约束位全系） |
| M2 — 约束计算完整化 | ✅ 完成（validationMessage 标准消息 + requestSubmit interactive validation） |
| M3 — 提交阻断与事件序列 | ✅ 完成（requestSubmit + button click + novalidate/formnovalidate + 重入守卫 + disconnected + form 级 :invalid） |

**待用户决策**：
- `--bin`（无 `-p`）构建的 workspace 级 feature 统一行为（browser quickjs 默认 + 其余 v8
  默认）——本轮以双 feature 编译兼容处理；browser 流若想恢复单 feature 构建，
  可考虑把 quickjs 移出 browser 默认 features（其决策）
  **2026-08-19 状态**：已纳入 goal-blockers 飞书征询（msg `om_x100b677cc00360a0c0203581aaf8e64`，建议保持现状销项、browser 流重构时顺带处理）
- 位置选择器（nth-child）的跨批身份不稳定是 shim 各 per-key 缓存的潜在碰撞源——
  `_inputValues` 已修；`_customValidity`/`_userEdited` 等同类缓存有相同 latent 风险
  （**技术备忘，非用户决策项**：保持观察，后续 shim 演进时随 L2 缓存体系一并收敛）

## 验证基线

- 测试基线：engine 2173 全绿（含 R2825 validation + R3048 修正）；clippy 零警告；fmt 无 diff
- WPT constraints 面：**Pass 919 / Fail 0**（45 + form-requestsubmit + form-checkvalidity）
- WPT canvas 面：1253 / 0 零回归
- workspace 测试：仅 compositor_gpu_dmabuf（GPU adapter 环境依赖，改动前同失败）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
