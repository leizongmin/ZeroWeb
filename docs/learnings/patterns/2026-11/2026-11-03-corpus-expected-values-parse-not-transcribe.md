---
date: 2026-11-03
modules: tests/wpt-runner
---

# 密码学语料期望值程序化解析取数，不手抄长 hex 串

## 问题

KMAC 仲裁脚本手打 6 条期望 hex（128/256 字符/条）进 heredoc，S4/S6 两条与实现
输出「前 32 hex 一致但整体判否」——对确定性散列这逻辑上不可能，疑似期望串
本身被抄错。

## 根因

手抄长 hex 串的静默笔误（多字/漏字/换行断点抄错），比对失败被误读为实现差异。
16 字节前缀一致 + 后段发散只在「期望串被插入/删除字符」时可能出现。

## 解决

正则直接从语料文件抽数，跳过人工转写：

```python
sigs = re.findall(r'signature:\s*new Uint8Array\(\[([^\]]+)\]\)', src)
vectors = [bytes(int(t.strip(), 0) for t in s.replace('\n', ' ').split(',') if t.strip()).hex()
           for s in sigs]
```

要点：JS 数组字面量允许尾逗号——split 后必须过滤空 token（`if t.strip()`），
否则 `int('')` 抛 ValueError。

## 避免方式

凡涉及逐字节比对的期望值（NIST KAT、WPT vectors、DER fixture），一律从源文件
程序化解析；手抄仅用于人读展示。比对脚本报告失败时先验证期望串来源
（解析自语料 vs 手打），再怀疑实现。
