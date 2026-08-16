# M1-M3 里程碑归档（2026-08-17）

> 归档区域：只追加不修改。M1/M2/M3 的详细过程与证据（只读快照）——
> 运行时状态与后续决策见 `master.md`（控制面板）与 `evidence/`（验证证据）。

## M1 — WPT constraints 基线建立（2026-08-16 ~ 2026-08-17）

**目标**：导入 `html/semantics/forms/constraints` 用例，跑通 testharness，记录通过率基线。

- 45 用例导入（fetch-constraints-subset.sh + `testharness-constraints` 子命令）
- 基线：Pass 3 / Fail 909（permissive valid——约束位全缺失）
- 终态：**Pass 909 / Fail 0 全灭**——约束位全系落地（valueMissing/patternMismatch/
  rangeUnderflow/rangeOverflow/stepMismatch/tooShort/tooLong/typeMismatch/customError/
  radio 组/willValidate/disabled 语义/date 类/matches:invalid 联动）
- 修复注记：stepMismatch 极小 step（3e-15）有理数 BigInt 整数性判定；无限回溯
  pattern 守卫直接 mismatch（V8 无 RegExp 超时）

证据：`evidence/m1-constraints-baseline-2026-08-16.md`

## M2 — 约束计算完整化（2026-08-17）

**目标**：全约束位真实计算 + validityState 联动 + validationMessage + willValidate。

- validationMessage：各约束位 Chromium 标准消息（valueMissing/typeMismatch
  email+url/patternMismatch/rangeOverflow+min/rangeUnderflow+max/stepMismatch）
- requestSubmit：interactive validation（spec §4.10.5.4）——invalid 控件第一个
  派发 invalid + 中止；novalidate/formnovalidate 跳过；valid 派发 submit
- 单测 +1；constraints 909/0 零回归；testharness-canvas 1253 零回归

## M3 — 提交阻断与事件序列（2026-08-17）

**目标**：interactive validation 全链路——submit 阻断 + invalid 聚焦 + novalidate 跳过。

- WPT `form-requestsubmit.html`（10 子测试）+ `form-checkvalidity.html` 导入
- 终态：**Pass 919 / Fail 0**（45 + 2 额外文件；16 Timeout = 14 个 -manual + 2 个 crash 回归，headless 预期）
- 修复清单：
  - requestSubmit(submitter) TypeError/NotFoundError 校验（含 detached）
  - `_zwRunFormSubmit` 共享提交路径（requestSubmit + submit 按钮 click 默认动作）
  - 重入守卫 `_zwSubmitBusy`；disconnected 表单不提交；form 级 :invalid/:valid 聚合
  - 查询 applied view（快照 + pending InsertAdjacentHtml 应用副本 + memoized 缓存）
  - dom crate 顶层逗号选择器列表支持；`_inputValues` 位置选择器跨批碰撞修复
  - form.elements 排除 input[type=image]（WPT oracle）
- 中间件构建断裂修复（6ef6fca29 引入——zero-browser quickjs 默认 → workspace
  双 feature → script-sandbox 无法双编译；js-dom R84 同步修复，本归档与其一致）

证据：`evidence/m3-constraints-final-2026-08-17.{md,json}`
