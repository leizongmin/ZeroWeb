# M1 证据：WPT constraints 基线建立（2026-08-16）

## 导入

- fetch-constraints-subset.sh：`html/semantics/forms/constraints` 顶层 45 用例 +
  support 资源（WPT_REV 3159769，与 canvas/html-compat 同 pin）
- runner：`testharness-constraints` 子命令（目录扫描顶层 .html——support 排除，
  仿 run_canvas_cases 模式）

## 基线（permissive valid——约束计算未实现）

| 指标 | 值 |
|------|-----|
| 文件（有子测试） | 27/45 |
| 子测试 Pass | **3** |
| 子测试 Fail | **909** |
| Pass 文件 | valueMissing-weekmonth（willValidate=false 消息空）、valueMissing（同）、inputwillvalidate（required 存在） |

**Fail 聚类**（子测试计数）：

| 用例 | Fail | 约束位 |
|------|------|--------|
| checkValidity / reportValidity | 130×2 | 全约束位（pattern/missing/tooLong 等） |
| validity-patternMismatch | 85 | pattern |
| validity-valueMissing | 78 | required |
| willValidate | 73 | willValidate 排除 |
| validity-tooShort / tooLong | 63×2 | minlength/maxlength |
| validity-rangeOverflow / rangeUnderflow | 49 / 47 | min/max |
| validity-valid | 35 | valid 联动 |
| validity-stepMismatch | 28 | step |
| 其余（badInput/typeMismatch/customError/weekmonth） | — | 混合 |

**根因**：R2825 的 permissive 基础（part01/04.js——`_validityState` 恒 valid、
checkValidity 恒 true）——原生约束位全未计算。Fail 即约束计算的验收清单。

## 验证

- wpt-runner 172 全绿（新增 constraints runner 无回归）
- clippy 零警告；fmt 无 diff
