---
date: 2026-11-24
modules: 构建与测试
---

# pipefail 下 `strings | grep -q` 存在性检查恒假阴性

## 问题描述

wptfyi-css kc5r81c 电池发射脚本带 `set -o pipefail`，用
`strings <130MB bin> | grep -q '<指纹>'` 做 shim 装载 provenance 硬门。构建确实
重编了 zero-engine（include_str 依赖触发），bin 实际已含新 shim 指纹，但门连续
两轮报"指纹缺失"，触发不必要的 touch + 全量重编并按缺失路径退出（exit 9），
电池发射被延迟约 3 分钟。事后用 `strings | grep -c` 全量读计数复核＝1，bin 无恙。

## 根因分析

`grep -q` 在首个匹配处立即退出并关闭管道读端；上游 `strings` 继续写入时收到
SIGPIPE，以非零码（141）终止。`set -o pipefail` 使整条管道的退出码取任一非零
成员，于是"命中"（grep -q 找到）被上游的 SIGPIPE 非零覆盖成"失败"。脚本只看
管道整体退出码，将命中误读为缺失。匹配位置越靠管道头部、文件越大，上游被
SIGPIPE 杀死越必然。

## 解决方案

门脚本里的存在性判断不用 `grep -q` 挂长输出管道，改为全量读的计数判：

```bash
FP_BIN=$(strings "$BIN" | grep -c 'fingerprint')
if [ "${FP_BIN:-0}" -lt 1 ] 2>/dev/null; then ...; fi
```

或对个别管道临时 `set +o pipefail`。等价陷阱：`grep -c` 无命中时打印 `0` 且
退出码为 1，`$(grep -c x f || echo 0)` 会产生双行值毒化整数比较——取值后直接
数值判，不在替换里加 `|| echo`。
