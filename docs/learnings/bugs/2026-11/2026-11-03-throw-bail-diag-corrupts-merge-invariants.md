---
date: 2026-11-03
modules: engine, testing
---

# throw 式诊断 bail 在 DOM merge 路径会撕裂不变量

> 2026-11-04 修订：本条初版把「插桩版永不完页」「V8 看门狗不可达自旋」
> 当实证结论；构建行法证（编辑兼容 spin2 窗，evidence
> docs/goal/editing-compat/evidence/2026-11-03-spin2-residual-hang-m670-guard.md）
> 证明早期对照数据混用二进制代际，两条推论证据失效，已改写。撕裂不变量
> 与诊断形态纪律两条核心结论不受影响。

## 问题

gate28 spin2 窗诊断 delete.html?7001-last 挂死时，对 part01f.js 合并臂 17 个
循环插入「计数 4096 次 / 30s 墙钟双门 throw」诊断笔。初版记录称「插桩版页
永不完页、比无插桩版更糟」——2026-11-04 法证修订：该观察是 1500s test-guard
墙钟预算跨 16 页均摊击杀的误归因，不构成插桩改变行为或不可中断的证据；
且当时的「干净版」对照与插桩版实际共用同一二进制（up-to-date 链，见
[include_str! 二进制谱系](2026-11-04-include-str-shim-binary-lineage-ab.md)）。

## 根因

throw 在 move/merge 环中途 unwind，合并半途撕裂 DOM 不变量（endBlock 半搬
空 + startBlock 混入 plain 子树的序列化拷贝），后续宿主调用面对损坏树的
行为不可预期。初版「撕裂后进入 Rust 内部段自旋、V8 terminate_execution
打不到」的链条同样失去证据基础，降级为未证假说，不再作为结论引用。

另需区分两种「页完成」：诊断笔 abort 掉昂贵段后页快速完成（diag3 del 面
+206 行恢复）≠ 行为修复——该恢复带 118 Fail 后经干净二进制 solo 逐字节
恒等复现（COORD 四十二轮），内容是现役引擎真实基线态，但恢复本身是中止
昂贵合并的产物，与 m670 守卫是否有效无关（该二进制不含守卫）。

## 解决与避免

- DOM 多节点操作路径的诊断出账只用两种形态：页尾探针 test() 读
  globalThis 快照（corpus 页可加 TEMP-DIAG 探针，gitignored），或守卫环体
  内非 throw 记录（break + globalThis 记号）。
- throw 式 bail 只在「命令级原子路径」（抛出后被派发层整体吞掉、无半途状态）
  使用；merge/split/move 类多步路径禁用。
- 「插桩是否改变行为」的对比判据必须先固定二进制代际：Compiling 行验证 +
  二进制内容指纹（strings 查插桩 token），只做同代际 A/B；跨代数据先做
  谱系映射再引用，见
  [include_str! 二进制谱系](2026-11-04-include-str-shim-binary-lineage-ab.md)。
