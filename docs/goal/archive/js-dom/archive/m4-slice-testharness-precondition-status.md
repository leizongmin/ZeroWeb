# M4 Slice R20 — testharness PRECONDITION_FAILED/NOTRUN 中性 status 精确化

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R20
**前置**: R14/R17（createEvent alias 全覆盖 + non-createable 抛 NotSupportedError，遗留 6F TouchEvent）

## 问题

`dom/nodes/Document-createEvent.https.html` 剩 6F 全是 TouchEvent，失败信息 `'expose legacy touch event APIs'`。根因：上游用例 `supportsTouchEvents()` 调 `assert_implements_optional('ontouchstart' in document, "'expose legacy touch event APIs'")`——ZeroWeb 未暴露 legacy touch API（`'ontouchstart' in document` = false），触发 `assert_implements_optional` 抛 `OptionalFeatureUnsupportedError` → subtest status = **PRECONDITION_FAILED (4)**。

但 `map_harness_results` 的映射 `0=>Pass, 2=>Timeout, _=>Fail` 把 status `4`（PRECONDITION_FAILED）和 `3`（NOTRUN）**误计为 Fail**。spec/上游 WPT dashboard 把 PRECONDITION_FAILED 视为**中性状态**（precondition 失败 = optional feature 不支持，非实现缺陷，既不计 pass 也不计 fail）。runner 的 status 映射 bug 拖低了 dom/nodes 通过率（6 个 optional TouchEvent 被误计 fail）。

## 修复

testharness runner status 映射精确化（`tests/wpt-runner/src/testharness.rs`），属 runner testharness 域工作面，零碰撞：

1. **`HarnessStatus` enum 新增中性变体**：`NotRun`（上游 `NOTRUN=3`，脚本错误/超时致 test() 未执行）、`PreconditionFailed`（上游 `PRECONDITION_FAILED=4`，`assert_implements`/`assert_implements_optional` 失败）。文档注释说明中性语义（通过率统计不计入 fail）。
2. **`map_harness_results` 精确映射**：`0=>Pass, 2=>Timeout, 3=>NotRun, 4=>PreconditionFailed, _=>Fail`。未知编码（5+）保守回落 Fail（暴露异常）。
3. **单测** `maps_notrun_and_precondition_failed_as_neutral_r20`：验证 6 种 status 编码（0-4 + 未知 9）精确映射。

**注**：`cmd_*_dom` 的 `failed = any(status != Pass)` 退出码判定**不变**——中性状态仍触发 exit 1（作为「还有测试未通过」的 rally 推进信号，基线不必全绿）；通过率统计（pass/(pass+fail)，中性从分母排除）才是准确化点。

## 验证

- **单测**：wpt-runner v8 `maps_notrun_and_precondition_failed_as_neutral_r20` pass。
- **clippy 双矩阵**：v8 + quickjs wpt-runner 零警告；fmt clean。
- **WPT dom/nodes 全量双路径**（完整 JSON 入 evidence，WPT 标准口径 pass/(pass+fail)，中性从分母排除）：

  | 路径 | R19（旧口径 pass/total） | R20（WPT 标准口径 pass/(pass+fail)） |
  |---|---|---|
  | Polyfill | 55.55%（2501P/2001F） | **55.63%**（2501P/1995F + 6 中性 PreconditionFailed） |
  | Native | 54.91%（2472P/2030F） | **54.98%**（2472P/2024F + 6 中性） |
  | 双路径差 | 0.64pp | 0.65pp |

  6 个 TouchEvent 从 Fail 变中性 PreconditionFailed（双路径各 -6 fail），pass 数不变（55.55%→55.63% 是分母排除中性的口径变化，非新 pass）。net≥0，无回归。

## 决策记录

- **为何选 runner status 精确化而非暴露 ontouchstart**：`assert_implements_optional` 的语义是「optional feature」——浏览器**可选**支持 legacy touch API。ZeroWeb 把 optional skip 误计 fail 是 runner bug，正确修复是让 status 映射对齐上游（中性而非 fail）。强行暴露 `ontouchstart`（让 `'ontouchstart' in document`=true）会把 optional 变 mandatory，可能暴露更多 TouchEvent 实现缺口（不一定全 pass），且语义上错（ZeroWeb 无真实触摸设备，不该声称支持）。
- **NOTRUN(3) 一并修**：虽当前 dom/nodes 用例无 NOTRUN 失败，但 `_ => Fail` 对 status 3 同样误计。精确映射 NOTRUN→中性是预防性正确化（canvas/html-interaction 用例可能涉及）。保守回落 Fail 仅留给真正未知编码（5+）。
- **退出码判定不改**：`failed = any(status != Pass)` 保持中性状态触发 exit 1——这是「rally 仍有未通过测试」的信号（基线推进期正确），通过率统计口径才是准确化点。两者职责分离。

## 残留（非本切片）

- createEvent 用例 6F → 现已中性（不计 fail），Element-classlist.html 仍全量 100%
- 其他失败聚类见 master.md「剩余聚类」（iframe.contentDocument / querySelector-mixed-case / canvas proxy instanceof / polyfill appendChild 闭环）
- 后续可选：若 ZeroWeb 决定支持 legacy touch API（暴露 ontouchstart），Touch 6 中性可转真断言——属产品决策，非本目标
