---
date: 2026-08-17
modules: zero-style-system
---

# 递归继承应借用 owned 父样式

## 问题描述

样式 DFS 为元素生成 owned `ComputedStyle` 后先写入结果 map，再从 map 深 clone 一份作为子元素的 `parent_style`。frame-pointer profile 显示 `ComputedStyle::clone` self 占 medium 全帧约0.8%，其内部字符串、长度和背景列表搬运继续放大 `memmove` 与 allocator 开销。

## 根因分析

结果 map 同时承担最终输出和递归期间的父样式存储。递归需要继续修改 map，Rust 无法在持有 map value 借用时插入子结果，因此旧代码用深 clone 解除借用冲突。实际计算出的 owned style 在插入前已经具备完整继承语义，可以直接作为子递归的只读输入。

直接把大型 `ComputedStyle` 留在每层递归栈并不安全。该初版通过样式单测和性能 A/B，但完整 `make test` 的深嵌套 HTML 用例触发 stack overflow。优化递归拷贝时必须同时审计栈帧尺寸，而不是只看 CPU。

## 解决方案

默认把当前 owned style 放入 `Box<ComputedStyle>`，子递归借用 Box 中的值，子树完成后再移入结果 map。Box 让递归栈只保留指针，同时消除父样式深 clone。`ZW_STYLE_DELAY_PARENT_INSERT=0` 恢复旧的先插入、再 clone 路径。

通用规则：递归过程已有 owned 大对象时，优先延迟提交并借用该对象；若递归深度由外部输入决定，大对象必须放在堆上，且要用深嵌套测试验证栈安全。

## 验证

固定 CPU Criterion 的 `compute_styles_200_elements` 从 `24.318ms` 降至 `22.542ms`，变化 `-6.72%`，95% CI `[-10.70%,-2.12%]`，`p=0.00`。最终 frame-pointer profile 中 `ComputedStyle::clone` self 从0.80%降至0.26%，归一化 clone 样本约降64%。深嵌套复现、style-system `2176/2176`、reftest `687/687`、产品、完整测试和性能绝对门均通过。
