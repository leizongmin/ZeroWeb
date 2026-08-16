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

## 追加：M1 切片 2——约束位首修（2026-08-16 晚）

**Pass 3 → 688 / Fail 909 → 221**。修复清单：

| 约束位/面 | 实现 |
|-----------|------|
| pre_check 基础 | _makeProxy target 预置约束校验属性（webview 页面脚本路径的 `in` 不调 has trap——实测；get 仍走 trap 实时计算） |
| valueMissing | required + 值缺失：text 类/checkbox/radio/select/file/date/number（ISO 格式 + isFinite） |
| patternMismatch | anchored（^(?:pattern)$——spec 完全匹配）+ u flag（Unicode 特性） |
| typeMismatch | email/url 格式（近似正则 + multiple 逗号分割） |
| rangeUnderflow/Overflow | min/max 数值比较（number/range） |
| willValidate | disabled/readonly（text 类）/type ∈ {hidden,button,reset} 排除 |
| form.checkValidity | 遍历控件（proxy querySelectorAll + 本地 _zwMEl 树 _collectControls） |
| disabled 语义 | barred 仅对 valueMissing（TEXT/date expectedImmutable false）；checkbox/radio 组状态例外；pattern/range 等 disabled 仍校验（expectedImmutable 缺省） |

**剩余 221 Fail 聚类**：date/time 格式位（checkValidity/reportValidity 32×2——
badInput 面）、willValidate 剩余（31+datalist 17）、pattern "Invalid v"（19——
v flag 正则）、weekmonth range（13×2——日期比较）、stepMismatch（8）、
radio 组（6+6）、valid 联动（12+6）。
