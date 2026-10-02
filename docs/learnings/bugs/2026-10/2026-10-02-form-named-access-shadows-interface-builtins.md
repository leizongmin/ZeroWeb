---
date: 2026-10-02
modules: zero-engine
---

# FORM named access 遮蔽接口内建方法：`<button id=reset>` 使 form.reset 静默失效

## 问题描述

表单交互 fixture 双失败：`apply_reset_on_click` 返回 false，宿主 reset 脚本静默 no-op。`git stash` 对照实证 HEAD 同样失败——HEAD 既有 bug，非修复栈回归（此前会话「HEAD 通过」的记录有误，对照验证必须留存命令与输出）。

## 根因分析

shim R3254-K3 的 FORM named access 分支在 FORM 方法 gate 之前返回命名控件：`<button id="reset" type="reset">` 入 form 后，`form.reset` 命中 named access 返回 button proxy（typeof 'object'）→ 宿主 reset 脚本的 `typeof f.reset==='function'` guard 恒假 → 重置整体失效。WebIDL 语义：named properties 对象位于接口原型对象**之下**（https://webidl.spec.whatwg.org/#idl-named-properties），命名控件不得遮蔽接口内建成员；HTMLFormElement 的 namedItem/item 及 reset/requestSubmit/submit 等方法永远优先。

修复：named access 条件排除 `item`/`namedItem`（已在前置分支）与 `reset`/`requestSubmit`/`submit` 三方法名。

## 解决方案

- 实现 WebIDL named properties 时，先列接口成员全集做豁免表；named getter 只兜底「无匹配成员」的键。
- 遮蔽类 bug 的症状是「guard 恒假静默 no-op」——宿主脚本对 DOM API 做 typeof guard 时，shadowing 让 API 变成意外类型，故障面比抛异常更隐蔽。
- 「HEAD 通过」这类对照结论必须当場留证（命令+输出）；记忆复述不可靠，本次复查发现先前结论错误。
