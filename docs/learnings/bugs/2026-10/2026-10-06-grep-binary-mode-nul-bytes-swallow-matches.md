---
date: 2026-10-06
modules: diagnostics
---

# grep 遇日志内 NUL 字节静默转 binary 模式吞掉全部匹配行（打点"零命中"假象）

## 问题

E14 停摆诊断中，Rust/JS 两侧打点（`ZW-DIAG rust resolve begin`、shim `[console]`）确认已编进二进制（`strings` 可见）、代码路径确认执行（同函数后续日志正常落盘），但 `grep 打点串 renderer-diag.log | wc -l` 恒为 0——追成"打点悖论"空耗数小时，还差点重跑无谓的诊断轮。

## 根因

真站页面内容经 renderer 日志落盘时夹带**原始响应头字节**（如磁盘缓存命中行 `…iframe.html\x00vary=Accept-Encoding…`），日志文件出现 55 个 NUL 字节。GNU grep 检测到 NUL 即转 binary 模式：行输出被抑制（`grep | wc -l` = 0、管道下游全空），而 `grep -c` 仍返回真实计数——两者矛盾进一步加深误判。python（`errors='replace'` 读文件）不受影响，故同一文件分析脚本能看到全部打点。

## 解决方案

- 对可能含页面/网络原始字节的日志，一律 `grep -a`（`--text`）强制文本模式；验证用 `grep -a ... | wc -l`。
- 判别速查：`grep -c` 有数而 `grep | wc -l` 为零 → 立即怀疑 binary 模式，`python3 -c "data=open(f,'rb').read(); print(data.count(b'\x00'))"` 确认。

## 如何避免

- 「零命中」结论必须双工具交叉验证（python/awk 复核）后才可作为排查分支依据；本例中 python 分析器早已显示 236 条打点与 236 个臂严丝合缝，grep 假象本可当场戳穿。
- 诊断日志承载真站内容（console 打点、头回显）时，落盘侧可考虑转义非文本字节，但根防在检索习惯：`grep -a` 是这类日志的默认姿势。
