---
date: 2026-11-08
modules: layout-engine,testing
---

# 自源 reftest 争议语义定谳：ref 页行为语义 = ZW 自身对应实现，先读 ref 页写法再对 ZW 同路径数值

## 问题描述

DC-14（self-source reftest corpus）的 test/ref 两页均由本引擎渲染。当某案失败、
且争议点落在「ref 页期望的语义究竟是什么」（如 justify 把哪些盒计入分配项、
ellipsis 槽位在哪、`text-align-last: justify` 的 gap 口径）时，逐像素比对与
截图判读都只能给出「差多少」，给不出「谁对」——争议语义没有规范文本可引时，
归因容易滑向猜测或把 ZW 自身行为误当 ref 权威。

## 根因分析

自源 reftest 的 ref 页虽然写法上模仿上游期望，但它**由 ZW 自己渲染**：ref 页
里使用的每个 CSS 特性（本例 `text-align-last: justify`）的渲染结果就是 ZW
对应实现代码路径的输出。因此「ref 页期望什么」不等于「规范要求什么」，而恰
好等于「ZW 对应实现的现有口径」。这不是缺陷而是可利用的性质：争议语义的正
确判定目标可以从「猜规范」收敛为「数值逐位对上 ZW 同一路径」——若 clamp 修复
臂的探针值与 ref 页对应实现路径的值逐位同值，即证明两臂口径一致，案定谳。

R5454（block-ellipsis-032 justify gap 口径差）实证：ref 行
`text-align-last: justify` 由 ZW normal justify 路径渲染，布局探针读出
ref runs remaining = 42.1、gap_count = 4（ellipsis 计入末项）＝clamp realign
臂 D 同值；旧实现 kept−1 = 3 口径每 gap 多分 D/12 ≈ 3.5px，正是 032 残差的
全部构成。ink forensics（截断痕迹判读）在此案给出过不可靠方向，探针数值定谳
一次收敛。

## 解决方案

争议语义定谳按固定三步：

1. **先读 ref 页怎么写**：确定 ref 页用了哪些特性、期望图像由哪条 ZW 代码
   路径产生（normal justify、`text-align-last`、fallback face 等）。
2. **再对 ZW 同路径数值**：用布局探针（临时插桩，用后撤）分别读 ref 侧对应
   实现路径与 test 侧修复臂的关键量（remaining/gap_count/位移表），逐位核对。
3. **逐位同值即定谳**：两臂口径一致 ⇒ 案绿有语义保证；数值差即缺口量，直接
   给出修法目标（如 gap_count 口径 +1）。

与既有判断的边界：

- 它**不**证明 ZW 行为符合规范（自源双侧同错的自洽绿仍存在，见 guideline #33
  与 border-radius 案例）；它证明的是「修复臂与 ZW 既有实现口径一致」。
- 需要规范权威时仍走 chromium oracle（headless 截图或 wpt.fyi），自源定谳只是
  把「两臂是否同口径」从判读问题变成算术问题。
- 配套纪律：pin 数学必须与臂同源口径（kept 集合、位移下标一致），跨口径断言
  会假红；盒旗标镜像落 store-gate return 之前（stored/Path B 共用信号）。
