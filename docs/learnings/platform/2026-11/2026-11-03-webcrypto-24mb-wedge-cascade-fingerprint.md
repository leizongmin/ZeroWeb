---
date: 2026-11-03
modules: tests/wpt-runner, zero-engine
---

# webcrypto 全量账 24MB 案楔死级联的判别指纹与串行复跑协议

## 问题

webcrypto 全量锚账复跑分母塌缩：82,065 → 64,304 keys，T 4→0，166/360 案塌缩
成单条案级 Fail。塌缩案隔日单案复跑全绿——非确定性，与代码改动无关。

## 根因

runner 是**单进程内嵌引擎**：全语料共用一个 QuickJS 运行时。KT/TurboSHAKE
24MB 案（挂账 heisenbug 家族，rquickjs 桥层大串参数拷贝 / GC 时序敏感）在本轮
把运行时打楔——此后所有页面的 shim 编译全部失败，案级 Fail 直到结尾。d1s3
干净轮同案只是子测试超时（T=4）；本轮并发流（另一 clone 的验收电池）在场时
发作升级为楔死。

## 判别指纹（零回归分析三步）

1. **总账形态**：分母塌缩 + T 清零 ≠ 切片翻绿形态（翻绿是 P/F 互换、分母不变）。
2. **塌缩连续段**：塌缩案在 runner 顺序里连续（本轮 idx 194→359），起点 =
   24MB 案（`modern-algos/digest/turboshake.*.worker.js`）。
3. **案级 Fail message**：塌缩案单 Fail 的 message 同文
   `DOM shim init: Compile error: SyntaxError: Invalid or unexpected token`
   ——运行时楔死后编译全挂的指纹，不是内容语法错误（内容错会从第 0 案全塌）。

## 解决

锚账落盘必须**串行安静窗口复跑**（无并发构建/测试流），退出码与分母逐位对
上一轮锚（360 案 / 82,065 keys）才有效；塌缩轮作废不落锚不记 FIXED。
`scripts/webcrypto-anchor-diff.py` 的 DRIFT 报告（lost=N gained=1 连续案）是
首检哨兵。

## 避免方式

全量锚账跑前查 `uptime`/并发 runner 进程；跑后先核分母再跑锚对照；遇楔死
指纹直接作废重跑，不对塌缩账做任何归因分析（省时且避免误记 FIXED 靶面）。
