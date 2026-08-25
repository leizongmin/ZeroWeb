# R277 Evidence — text wrapper 单一对象源：复现域收窄（诊断轮，无代码 land）

**日期**: 2026-08-26
**切片**: M4——R277(a) R276 平行 wrapper 复现尝试
**改动面**: 无生产代码（诊断轮）

## 一、复现尝试与排除（三轮 probe，全部 clean 后删除）

在三个递进环境复现 R276 的「比较遍历 text = 完整 / childNodes = 修剪」分歧：

1. **顶层文档域**（setupRangeTests 直接建 paras）：`same=true`，
   deleteData 对 firstChild/childNodes/nodeValue 全可见——无分歧；
2. **单 iframe 域**（iframe src=Range-test-iframe.html + setupRangeTests）：
   同上无分歧；
3. **单轮克隆轮转**（mimic restoreIframe：referenceDoc 克隆 + doc strip +
   appendChild clone + setupRangeTests）：同上无分歧。

**结论**：分歧不在这些域——需要 **多轮克隆轮转累积**（真实 harness 对
每个 subtest i=0..53 都执行 restoreIframe，54 轮 referenceDoc 再克隆；R220
时代的「跨轮残留」线索同源）或 **双 iframe 交错轮转**（actual/expected
交替 restoreIframe 的全局 registry 串扰）才能触发。

## 二、R278 复现方向（下一步 probe 形态）

- probe 循环 N 轮 restoreIframe（克隆→strip→append→setupRangeTests），
  每轮后检查 `P#a.firstChild === P#a.childNodes[0]` 与 data 一致性，
  二分找到首分歧轮次 N*；
- 或双 iframe 交错版（actual/expected 交替），复刻 testDeleteContents 的
  restore 顺序；
- 复现后按 R276 的修复面落地（融合视图 text 子从 textEl/_zwMText 注册表
  取同一对象）。

## 三、验证

| 项 | R276 | R277（诊断轮） |
|---|---|---|
| Range-deleteContents | 115P/14F | 未跑（无代码变更，基线不变） |

## 四、R278 靶点

- **(a) 多轮克隆轮转复现 probe**（上述二分法）→ 复现后单一对象源修复。
- (b) 28,x / 49/50,x cursor-only；extract/clone 重聚类。
