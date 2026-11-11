---
date: 2026-11-11
modules: engine
---

# css-color none 保形回归：raw 影子保存条件误排 + 基线巧合通过

## 问题描述

r2320 color 批（part11-computed-color none 保形序列化按输入语法跟随：裸数字保裸、% 输入保 %）后，WPT `color-computed-hsl/hwb` 8 案回归：`hsl(120 80% none)` computed 期望 `hsl(120 80% none)`，实产 `hsl(120 80 none)`（% 丢失）。cc8 基线这 8 案全过——批量对账净增为正，若只看总数不逐页归因就会漏掉。

## 根因分析

链路三层叠加，深层在写面影子：

1. **写面归一剥 %**（part03 `_zwNormalizeDeclValue`）：`hsl(120 80% none)` 落库存为 `hsl(120 80 none)`——% 语法在 declared 文本里已不可逆。
2. **raw 影子保存条件排除该群**（part03 `_styleProxy` setProp）：条件 `pri.value !== core && /none/i.test(pri.value) && !/none/i.test(core)` 的最后一项要求归一后**不含** none 才存原值；而 % 剥除形 core 仍含 none 字面 → 不存影子。注释本意是「消解 none 的形才需要影子」，漏了「none 保形但语法变形」的形。
3. **computed 读裸 decl**：part01c `_zwPlainDecl` 影子 miss → 取归一后文本 → part11 序列化按输入语法（现在正确地）输出裸数字。

**cc8 为什么过**：旧 serial 无视输入语法一律给非 hue 分量补 `%`，把收到的裸 decl `hsl(120 80 none)` 强折回 `hsl(120 80% none)`——对这 8 案恰好等于期望值。即基线通过是两个错误（影子漏存 + serial 强加 %）相互抵消的巧合，不是行为正确。修掉一个就露馅。

## 解决方案

放宽影子保存条件：归一变形且原值含 none 即存（去掉 `!/none/i.test(core)`），computed 的 none 保形只能从原值恢复分量语法。修复后 8 案全数恢复，hsl/hwb 页逐页回到 cc8 水平。

### 二层根因：raw 通道暴露 mix 插值空间别名不对称

影子修复落地后对账，`color-computed-color-mix-function` 反掉 13 案（`color-mix(in xyz, color(xyz none …), …)` 组：none→0 全权重归一，期望 none 传播/逐通道 carry）。探针三路对照（raw / 归一 core / 手写 xyz-d65）定谳：declared 归一已把 `xyz` 规范成 `xyz-d65`（side 解析处 alias 折叠），mix 解析的插值空间不折叠——`_zwCCMixSide` 按 space 全等匹配 own 形，raw 输入匹配失败走 rgb 回退丢 none 旗标。cc8 喂归一 core 恰好绕开该缺陷。修复：mix 解析层同一处折叠别名（与 side 解析先例同规），输出侧死三元式随改清理。

## 避免方式

- **基线通过 ≠ 行为正确**：A/B 对账里「旧实现靠错误行为撞中期望」的案，新实现如实化后反而翻红。逐页归因看丢案断言消息（expected vs got 的形态能直接指向产出层），别只看净增总数。
- **保形序列化的输入必须是原值**：写面归一不可逆变形（剥 %、消解 none）时，computed 面依赖影子/原文通道，保存条件要覆盖「语法变形」不止「语义变形」。
- **别名/归一化折叠必须单点同规**：同一规范形（`xyz`≡`xyz-d65`）在相邻解析函数里一处折叠一处不折叠，全等比较就会按输入形态分歧——raw 通道（影子/原文）把另一形态送进来才暴露。新解析入口先查同文件既有折叠点。
- 直调管线探针（`__zwComputedColor(raw)`）与 gCS 通道探针（`getComputedStyle().color`）分开做——本案直调全对、通道全错，一步定位到影子层。
