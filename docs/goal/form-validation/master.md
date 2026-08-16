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

### WPT 面

- `html/semantics/forms/constraints` 未导入（wpt-data 无 forms 目录）——零基线

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| V1 | WPT constraints 用例覆盖为零 | 🔄 M1 切片 1（导入 + 基线报告） |
| V2 | 原生约束位计算缺失（permissive valid） | 🔄 M2 |
| V3 | 提交阻断缺失（interactive validation） | 🔄 M3 |
| V4 | willValidate 真实化 | 🔄 M2 |
| V5 | validationMessage 约束消息 | 🔄 M2 |

## 下一步计划

1. **M1 切片 1**：WPT `html/semantics/forms/constraints` 导入 + 分类通过率报告
   （零源码改动，纯资产——fetch 脚本扩展 + 基线测量）
2. **M1 切片 2**：失败聚类分析 → 首个轻量修复队列（约束位计算——required/pattern
   起步）
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
