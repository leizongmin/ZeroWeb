---
date: 2026-11-21
modules: wpt-runner,rally
---

# wpt-runner suite 名＝wpt-data 目录名直映，cssom 系目录不带 css- 前缀

## 问题描述

knife 电池脚本按 `testharness-css-$c` 模板拼 suite 名传给 `zero-wpt-runner`。
`c=cssom` 生成 `testharness-css-cssom` → runner 打印 usage（`Unknown command`）
即退 rc=1，JSON 产物为 23,099 字节 usage 帮助文本；v2 修复时对 `cssom`
特例化却又对 `cssom-view` 重蹈（生成 `testharness-css-cssom-view`，正确名
`testharness-cssom-view`）。两例症状相同：腿 START/DONE 同秒、rc=1、
json=usage 文本，RECONCILE 装载即 JSON 解码崩。

## 根因分析

runner 套件名不是「testharness-css-＋短名」模板，而是
**`testharness-`＋wpt-data 下目录全名**的直映：
`css-text → testharness-css-text`、`css-typed-om → testharness-css-typed-om`，
但 `cssom → testharness-cssom`、`cssom-view → testharness-cssom-view`
（目录名本身不以 `css-` 开头）。模板假设只对带 `css-` 前缀的目录巧合成立。
合法名全表在 `tests/wpt-runner/src/main.rs` usage 文本与 dispatch match 中。

## 解决方案

电池脚本的 suite 名按「`testharness-`＋目录名」拼装，或直接引用 main.rs
dispatch 表；收账判读前先程序化核 JSON 首字节是否 `{`/`[`（usage 文本以
`Unknown command` 开头必非 JSON）。腿空跑的可靠签名＝START 与 DONE 同秒
＋rc=1＋json 体积≈usage 大小（23,099 字节）。 judge 对装载失败应显式
SKIP 并注因，不得静默当 SET-IDENTICAL 或计入门失败。
