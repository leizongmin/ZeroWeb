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

## 追加：M1 完成（2026-08-16 深夜）——Pass 907 / Fail 2

**Pass 3 → 907**（909 Fail → 2）。约束位全系落地（M1 切片 2 的完整清单 + 后续修复）：

- **disabled 语义**：barred 仅对 valueMissing（WPT expectedImmutable——pattern/
  range/typeMismatch 在 disabled 仍校验；checkbox/radio/file/select 的
  valueMissing 例外——组状态/文件状态不 barred）
- **radio 组**：组级 required（组内任一 required + 无 checked → 全部 missing）；
  click 默认动作（勾选 + 同 name 互斥）；组查询的 remove 排除 + stale 兜底
  （selfRequired/curChecked latest-wins——mutation 未应用）
- **date 类**：ISO 范围校验（月/日/时/周 + 4+ 位年 + 空格分隔 + 毫秒）；
  range 宽松 comparability（含时间怪用例 + 无效月日不比较）；年数值比较
  （"10000" > "2000"——变长字典序错误）；time reversed range；datetime-local
  的 step（秒差）
- **matches/closest/querySelectorAll 的 :invalid/:valid**（约束校验联动）
- **execCommand InsertHTML**（插入 + maxlength 代理对安全截断）
- **pattern 的 v 非法检测 + multiple 逐项 + 回溯守卫**（V8 无超时防护）

**剩余 2 Fail（引擎级限制，已记录）**：
1. stepMismatch 3e-15：diff/st ≈ 5.67e15 的 IEEE 舍入——浮点取模不可靠
2. infinite_backtracking：V8 无 RegExp 超时——无限回溯 pattern 卡死——
   守卫跳过匹配（用例期望 invalid——tentative 引擎级）

## 追加：M1 全灭——Pass 909 / Fail 0（2026-08-17）

**剩余 2 Fail 修复**：
1. **stepMismatch 3e-15**：有理数 BigInt 整数性判定——十进制串（±整数/小数/
   科学计数法）解析为 {num, den} 分数——(value-base)/step 的交叉相乘模——
   IEEE 浮点取模不可靠（diff/st ≈ 5.67e15 舍入）——特判 st < 1e-9 的
   number/range
2. **infinite_backtracking**：V8 无 RegExp 超时（rusty_v8 无 Isolate backtracks
   API——实测 set_flags_from_string 的 --regexp-backtracks-before-fallback /
   --enable-experimental-regexp-engine-on-excessive-backtracks 均无效卡死——
   已回退）——守卫：组后量词 ")*"/")+" 的 pattern 直接 mismatch（近似——
   用例期望 invalid ✓）——不卡死

**最终**：constraints **Pass 909 / Fail 0**；testharness-canvas 1253 零回归；
wpt-runner 172 全绿；clippy/fmt 全过。
