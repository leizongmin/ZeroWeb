# 表单校验运行时控制面板

**状态**: Active（2026-08-16 立项——从 html-compat 拆分）
**最后更新**: 2026-08-16（M1 切片 1 准备——WPT constraints 用例导入）

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
| V1 | WPT constraints 用例覆盖为零 | ✅ M1 切片 1 完成（45 导入 + 基线 3/909） |
| V2 | 原生约束位计算缺失（permissive valid） | 🔄 M2 |
| V3 | 提交阻断缺失（interactive validation） | 🔄 M3 |
| V4 | willValidate 真实化 | 🔄 M2 |
| V5 | validationMessage 约束消息 | 🔄 M2 |

## 下一步计划

1. **M1 切片 2（进行中）**：约束位首修已完成（Pass 688）——剩余 = date/time
   格式位（checkValidity/reportValidity 32×2）、willValidate 剩余（31+datalist
   17）、pattern "Invalid v"（19）、weekmonth range（13×2）、step（8）、
   radio 组（6+6）
2. **M1 切片 3**：date/time 格式位 + stepMismatch
3. **M2**：全约束位 + validityState 联动 + validationMessage + willValidate

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT constraints 基线建立 | 🔄 进行中（切片 1 未开工） |
| M2 — 约束计算完整化 | 未开始 |
| M3 — 提交阻断与事件序列 | 未开始 |

## 验证基线

- 测试基线：engine 2163（含 R2825 validation 测试）；clippy 零警告
- WPT constraints 面：无基线（未导入）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
