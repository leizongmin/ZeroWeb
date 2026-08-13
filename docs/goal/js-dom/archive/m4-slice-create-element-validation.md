# 归档：M4 切片 3 — createElement 非法标签名校验抛 InvalidCharacterError（双路径）

**日期**: 2026-08-13
**轮次**: R3
**Milestone**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**切片**: M4 切片 3（R2 DOMException 基建之后，createElement 抛出点扩面）
**基线**: `04a96ee4`（R2 land 后）

## 切片目标

R1 基线聚类：`Document-createElement.html` 非法标签名（`1foo`/`<foo`/`fo o`/`-foo` 等）应抛 `InvalidCharacterError` DOMException（spec `dom-document-createelement` validate），当前静默回落 `div`（native）/直接建（polyfill）。复用 R2 落地的 `throw_dom_exception` 基建扩面。

## 实现产物

### spec Name production 校验（双路径共享逻辑）
- **native** `dom_bindings/mod.rs`：`is_valid_qualified_name(name)` + `is_name_start_char` + `is_name_char`——首字符须 name-start（ASCII 字母/`_`/`:`/非 ASCII），后续须 name-char（name-start 或数字/`-`/`.`）。对齐 WPT `Document-createElement.html` valid/invalid 列表。
- **polyfill** `js_dom_shim/part01b.js`：`_zwIsValidQualifiedName` + `_zwIsNameStartChar` + `_zwIsNameChar`——JS 镜像同样逻辑（A/B 等价）。

### createElement 抛 InvalidCharacterError（双路径）
- **native** `factories.rs native_create_element_invoke`：校验失败 → `throw_dom_exception("InvalidCharacterError", ...)`（此前空标签回落 div）。关键：`createElement(undefined)`/`(null)` 经 ToString → "undefined"/"null"（首字符字母）合法通过（WPT valid 列表含之）。
- **polyfill** `part06.js createElement`：校验失败 → `throw new DOMException(msg, 'InvalidCharacterError')`（调 `__zw_create_element` 前）。

### A/B 门 createElement 异常路径扩展
**`tests_ab_compare.rs`**：`ab_create_element_invalid_name_throws_dom_exception`——5 个非法标签（`1foo`/`<foo`/`foo>`/`fo o`/`-foo`）两路径抛同 InvalidCharacterError + 合法标签防误伤。

### native 单测（tests_dom_api.rs）
- `native_document_create_element_invalid_name_throws`：10 个 WPT invalid 标签全抛 InvalidCharacterError
- `native_document_create_element_valid_name_passes`：合法标签（含 `:`/`undefined` ToString）不抛

## 验证证据

| 矩阵 | 命令 | 结果 |
|------|------|------|
| zero-engine v8 lib | `cargo test -p zero-engine --features v8 --lib` | ✅ 2068 passed（+3：createElement valid/invalid 测试） |
| zero-wpt-runner | `cargo test -p zero-wpt-runner` | ✅ 167 passed |
| clippy v8 + quickjs | `cargo clippy -p zero-engine ...` | ✅ 双矩阵零警告 |
| **testharness-dom 全量** | `make testharness-dom` | dom/nodes 56.08% → **56.45%（+0.37pp，0 回归）** |
| Document-createElement.html HTML 上下文 invalid | 单用例筛选 | `1foo`/`fo o`/`}foo`/`<foo`/`foo>`/`<foo>` 等 **HTML 上下文全转 Pass** |

**核心结论**: createElement HTML 上下文 invalid 标签校验修复净正（+0.37pp）。整体提升幅度小于 R2（classList）因 createElement invalid 仅占 ~10 个 HTML subtest，且大头在 XML/XHTML iframe 上下文（`dummy.xml`/`dummy.xhtml` 在 headless 不加载——独立缺口，非本切片范围）。

## 关键决策

1. **valid 列表反推 spec 规则**：WebFetch 被网络限制，从 WPT valid/invalid 列表（`undefined`/`null` valid、`1foo`/`<foo` invalid）反推 Name production 规则——`createElement(undefined)` ToString 成 "undefined" 合法，故校验在 `String(tag)` 后做。
2. **不破坏 lenient 回落**：既有空标签回落 div 注释为 "best-effort"，无测试依赖；改抛异常是 spec 合规修正，无回归（valid 测试守护）。
3. **校验 helper 双路径共享逻辑**：native Rust + polyfill JS 镜像同一规则，A/B 门验证两路径行为一致。

## 下一步（M4 切片 4 候选）

按剩余 ROI：① appendChild/insertBefore 闭环（HierarchyRequestError）② createProcessingInstruction（44）③ 扩 dom/events。
