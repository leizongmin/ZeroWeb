---
date: 2026-11-12
modules: systemd-run, bash
---

# systemd-run scope 的 bash -c 吃不到调用方未 export 变量——单引号双层屏蔽，落盘走错路径快速假成功

## 问题描述

守卫编排习惯写法 `RUN=... && systemd-run --user --scope ... bash -c '...$RUN...'`
下，脚本内 `$RUN` 为空：重定向与 mkdir 落到错误路径（如 `/evidence/...`），
payload 未跑就快速退出、无报错栈，看起来像"秒完成的成功"。

## 根因

两层屏蔽叠加：

1. `bash -c '...'` 单引号在**调用方**不求值，字符串原样交给 scope 内 bash；
2. systemd-run 经 systemd 管理器 exec，只继承**已 export** 的环境——调用方
   仅赋值未 export 的变量不跨边界，scope 内展开为空串。

## 解决

- 编排脚本内一律**硬编码绝对路径**（本仓运行目录路径固定，无需参数化），
  或改用 `systemd-run --env=KEY=VAL` 显式注入；
- 排查信号：scope 单元结束极快 + 日志里出现带空路径段的 `No such file`；
  自检办法：payload 首行 `echo "$VAR" >&2` 打印边界内可见性。
