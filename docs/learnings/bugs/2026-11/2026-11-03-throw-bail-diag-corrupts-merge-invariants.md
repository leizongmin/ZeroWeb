---
date: 2026-11-03
modules: engine, testing
---

# throw 式诊断 bail 在 DOM merge 路径会撕裂不变量并致不可中断自旋

## 问题

gate28 spin2 窗诊断 delete.html?7001-last 挂死时，对 part01f.js 合并臂 17 个
循环插入「计数 4096 次 / 30s 墙钟双门 throw」诊断笔：插桩版页永不完页，被
1500s test-guard 击杀——比无插桩版（JS 级自旋，300s V8 看门狗可中断出
Timeout 行）更糟。

## 根因

throw 在 move/merge 环中途 unwind，合并半途撕裂 DOM 不变量（endBlock 半搬
空 + startBlock 混入 plain 子树的序列化拷贝）。后续宿主调用对该损坏树进入
Rust 内部段的自旋，V8 terminate_execution 打不到宿主内部，任何 JS 侧预算
都失效。同一挂死点换非 throw 形态（页尾探针读 globalThis 快照 / 进展守卫
break）后页正常完成。

## 解决与避免

- DOM 多节点操作路径的诊断出账只用两种形态：页尾探针 test() 读
  globalThis 快照（corpus 页可加 TEMP-DIAG 探针，gitignored），或守卫环体
  内非 throw 记录（break + globalThis 记号）。
- throw 式 bail 只在「命令级原子路径」（抛出后被派发层整体吞掉、无半途状态）
  使用；merge/split/move 类多步路径禁用。
- 判断插桩是否改变行为：对比插桩版与干净版在同切片的 Timeout 形态与
  完成性（本例：插桩版永不完页 vs 干净版 300s 截断出账，即行为改变信号）。
