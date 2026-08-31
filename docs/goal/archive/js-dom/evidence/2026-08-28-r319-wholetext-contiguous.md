# R319 Evidence — wholeText contiguous 语义（Text-wholeText 全文件转绿）+ getElementsByClassName live 二次尝试负结果定性

**日期**: 2026-08-28
**切片**: M4——R319(a) 备档面假设复核（R313「假设会过期」教训的系统性应用）
**改动面**: `part04.js`（wholeText contiguous 重写）+ `part03.js`（_zwMText 同款）+ `part24.rs`（r319 分段回归测试）

## 一、R319(a) 负结果：getElementsByClassName live 的桶归因重试（无代码 land）

R318 定性「pending 表无容器归因是结构性缺口」——本轮证伪该归因的一半：**归因存在**
（`_zwPendingByParent` 的 `'_h:'+handle` 桶，R51c 记账已带容器），桶展开 + 类名过滤 +
scope 上行 matches 三层全上后 Element-getElementsByClassName 3P 全绿，但
**Event-dispatch-single-activation 仍 61P/43F**。bisect 定位：关闭构建期桶并入（仅留
matches）57P 仍破——**matches 本身**（mutation 期维护路径）是破坏源；全量回退后 132P
恢复。机制未完全定位（matches 闭包在 activation 链中的副作用域），但两次独立实现
（全表类名 / 桶归因 + scope 上行）都在同一用例面翻红 → **定性为 liveSpec.matches 与
activation 激活链的深层交互，超出轻量修复面，维持备档**（该 1F）。

## 二、正面修复：wholeText 的 contiguous 语义（假设复核命中真缺口）

`Text-wholeText`（WPT）此前归档为未归因 Fail——本轮按 R313 教训复核，逐段断言探针
两轮拆出**两个独立缺陷**：

1. **非 contiguous 拼接**：spec `dom-text-wholetext` 是 this 所在「逻辑相邻 Text 序列」
   的联接（向前/向后延伸直到非 Text 节点）；旧版全子树 Text 拼接——`insertBefore(<a>, t3)`
   隔断后 t1.wholeText 期望 "ab" 得 "abc"。修：定位 self 位次 + 双向延伸。
2. **向后延伸区间初值缺陷**：`_wtHi` 初始 0 非 idx——自身非首子且无后邻时拼区间
   [lo..0] 塌缩（t2.wholeText 期望 "ab" 得 "a"；trace `idx1/2;o0="a"` 单步实证）。
   修：`_wtLo = _wtHi = _wtIdx`。

`_zwMText`（part03 解析域）同款 contiguous 修（其 lo/hi 初值本就正确，仅拼接范围同款
收紧）。探针分段断言 `w0=a|w1=ab,ab|w2=abc,abc,abc|w3=ab,ab,c` 全对齐 WPT 期望。

## 三、A/B

| 套件 | R318 基线 | R319 | Δ |
|---|---|---|---|
| Text-wholeText | 0P/1F | **1P/0F** | +1 |
| CharacterData 157P / Text-splitText 6P / Node-textContent 81P / MO-characterData 21P / Range-mutations-splitText 116P | 全绿 | 全绿 | 持平 |
| **全量 dom sweep** | 54140P/59F/21T | **54140P/58F/22T** | **Fail set 恰 -1（Text-wholeText）零新增**；+1 Timeout 单跑 Pass（并发噪声）|
| Event-dispatch-single-activation-behavior | 132P | 132P | 零回归（回退验证链确认）|
| engine --lib（v8/quickjs）| 2456/1460 | **2457**/1460 | +1（r319 分段回归）|
| fmt / clippy（v8 guarded + quickjs）| — | 干净/0 | — |

## 四、教训

1. **「结构性缺口」归因也要复核**：R318 的「pending 表无容器归因」一半错误——归因
   存在（分桶），真缺口在 liveSpec.matches 与 activation 链的交互。负结果的表述要精确
   到「哪个具体假设被证伪」，否则下轮会在错误前提上重试。
2. **分段断言 vs 终态断言**：wholeText 缺陷 2 只在逐段断言形态暴露（append 后立即读），
   终态统一读探针全绿——探针复刻必须对齐用例的断言节奏。
3. **区间延伸的初值**：双向延伸循环的区间变量初值必须是种子位（idx）而非 0——初值 0
   在「无后邻」形态把区间塌缩到错误边界。
