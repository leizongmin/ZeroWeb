---
date: 2026-11-05
modules: tooling
---

# 日志失败计数未锚定 ^Fail 前缀，Pass 行同串误计

## 问题

验证语料行修复是否生效时用 `grep -c '签名.*compare innerHTML'` 计「失败数」，把计数 1 误读为仍红；实际该行是 `Pass … compare innerHTML`——同一 fixture+断言串同时出现在 Pass 与 Fail 行，修复成功后 Pass 行照样命中，计数恒 ≥1。

## 根因

testharness 日志行格式为 `Pass <file> :: <fixture> … <test name>` / `Fail <file> :: …`，fixture 与 test name 是 Pass/Fail 共享的公共后缀；不带行首等级锚定的签名必然双计。

## 解决

失败判定一律 `grep -c '^Fail '`（或 `grep '^Fail ' | grep <签名>`），通过判定一律 `^Pass `；交叉验证修复时同时给出 Pass/Fail 两个锚定计数。对照判读（修复前 Fail、修复后 Pass）才有证明力。
