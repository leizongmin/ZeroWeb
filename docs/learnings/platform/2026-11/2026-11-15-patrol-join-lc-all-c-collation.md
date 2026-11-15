---
date: 2026-11-15
modules: wpt-runner,engineering-practice
---

# 巡查 join 管线必须全链 LC_ALL=C

## 问题描述

html-semantics 守成巡查的 norm/canon/join 管线（`/tmp/s48x` 沉淀脚本）在 zh_CN.UTF-8 会话里跑 `comm` 时报「文件没有正确排序」，对上一轮 stable key 集做 GONE/NEW 对账失败；`sort -c` 同时报第 11 行乱序。同版脚本在早前会话能跑通——差异只在 shell locale。

## 根因分析

两个叠加因素：

1. `LC_ALL=C cut ... | sort` 这种写法里 locale 前缀只作用于管道第一段，`sort`/`comm` 仍继承 shell locale（zh_CN.UTF-8）。zh_CN 排序规则会忽略部分标点（如 ` :: `），导致同一文件在 `sort` 和 `comm` 两步之间被判定为不同顺序；跨轮对账时新旧产物各自在不同 locale 下排序，直接 `comm` 必炸。
2. `join.sh` 末行用 `grep -Fxf stable.keys l1u` 生成 stable canon，但 `stable.keys` 是纯 key 行、`l1u` 是 `key\tstatus` 行，整行精确匹配永远为空集——s486 轮的 stable canon 是事后单独生成的，脚本本身从未产出过该文件。

## 解决方案

- 整条管线（含脚本内部）统一 `export LC_ALL=C`，或在脚本内每个 `sort`/`comm`/`join` 调用前显式加前缀；跨轮对账前对上一轮产物先 `LC_ALL=C sort -u` 重新规范化，不假设其排序 locale。
- stable canon 改用 awk 按 key 集过滤：`LC_ALL=C awk -F'\t' 'NR==FNR{k[$1];next} ($1 in k)' stable.keys l1.canon`。
- 对照实验时先 `echo $LC_ALL` 确认当前会话 locale 再起跑；rendering 流的对账命令一直显式带 `LC_ALL=C`，两个流管线应对齐这一惯例。
