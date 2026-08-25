# R247 Evidence — 引擎级复现证伪「iframe-window factory no-op」假设（诊断修正轮）

**日期**: 2026-08-25
**切片**: M4——R247(a) 差分定位（R246 结论修正；无代码 land）
**基线**: surround 1806P/34F 复核零漂移

## 一、R246 假设与证伪过程

R246 结论「host surround 在 iframe-window factory docEl 容器整体
no-op（deA=[HEAD,BODY]，P 缺失）」。R247 通过引擎级 r247_tmp_repro
（已清理）逐步复现 17,0 精确形态：

1. **顶层形态**（main doc `implementation.createHTMLDocument` +
   main-doc paras[0]）：`scDocEl=[P(1),BODY(2)]`，P 含 HEAD、BODY 完整、
   testDiv 清空、边界正确——**全链工作**。
2. **iframe-window 形态**（ifr contentDocument 建 paras[0]，
   `idoc.implementation.createHTMLDocument` 建 fd）：初次复现
   `de2=[P(1),BODY(0)]` 疑似 BODY 丢子；随后发现这是**复现构造 bug**——
   variant 2 从未把 paras append 到 fd2.body（fp1/fp2 属 R245 测试的另
   一个 fd）。
3. **精确序列隔离 fdZ**（iframe-delegated factory + text-child paras +
   body appends + createRange + setStart/setEnd）：`Zbl:2,2`——
   **完全正常**。append-only 探针 `Xbl:1`（顶层）/`Ybl:1`（iframe 委托）
   亦全部正常。

## 二、修正后的结论

- **R246 的「no-op」是 dump 伪影**：探针在 positionTests 阶段读
  `documentElement` getter，但 restoreIframe 每轮重建 doc 树——getter
  返回的新树与 walk 持有的 sc 旧引用是**不同对象**（R246 已记录的
  「对象域分裂」本身即伪影来源，而非 host 行为缺陷）。
- 引擎级 17,0/13,0 精确序列**全部通过**——真实分歧存在于 **harness
  全环境**（referenceDoc 逐轮克隆 + `window.eval(testRangeInput)` 的
  range 创建 + iframe common.js setupRangeTests 三者交织），无法在
  单测沙箱重现。

## 三、R248 方法论修正

放弃「engine 单测复现 17,x」路线，改用 **R222 式 win.run() 内联探针**：
把插桩注入 Range-test-iframe.html 的 `run()`（range 创建点），dump
sc/newParent 的**创建时对象形态**（factory vs 克隆 vs wrapper 域），
配合 domTests 后的双树 walk 差分——在 harness 真实环境内定位
sim/host 分歧的**对象域断点**。

## 四、验证

- r247_tmp_repro 清理后 part23.rs 与 main 零 diff（whitespace 已回退）。
- 基线复核：surround 1806P/34F 零漂移；engine 单测（r245 抽查）绿。
- 无代码 land → 无回归面。
