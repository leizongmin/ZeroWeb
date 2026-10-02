---
date: 2026-10-02
modules: zero-engine, zero-renderer
---

# 动态外链脚本双路径并存：shim 页面 fetch 与宿主取回双执行/双事件

## 问题描述

动态插入 `<script src>`（appendChild）测试双失败：error 用例 `__dynErr=0`（事件丢失），成功用例 `__okLoad=2`（load 事件触发两次）。

## 根因分析

shim R387b 在插入期用页面 `fetch()`（cors 语义）取回脚本源并 eval；后提交的 R-baidu8 改造把动态外链脚本下放到宿主通路（runtime `execute_new_dynamic_scripts`/`tick_dynamic_scripts`，no-cors 脚本语义 + 元素事件派发）。两路径并存：宿主取回派 load 后 shim 的页面 fetch 又跑一次 → 双执行/双事件。雪上加霜：无 fetch handler 时 resolve `ok=false/status=0`，R387b 的 guard `if (!_r.ok && _r.status) throw` 把它误判为成功 → 误派 load。

修复：`globalThis.__zwHostDynamicScripts` 旗标——renderer worker bootstrap 与 ResetDocumentState 重装 shim 后置位；R387b 分支读旗标让位宿主通路。browser 单进程路径（tab_scripts 无宿主 loader）不置旗标，R387b 照常。旗标随 context 重建重置是关键：shim 重装后旗标丢失会让双路径复活。

## 解决方案

- 同一语义（动态脚本取回）只允许一条实现路径；引入新路径时用旗标/特性检测让旧路径显式让位，并在 context 重建点重置旗标。
- 嵌入式架构（browser 单进程 vs renderer 多进程）下 shim 是共享层——改共享层行为要枚举全部嵌入方，用运行时旗标区分而不是硬切换。
- guard 条件要覆盖「失败但字段为零」形态：`!ok && status` 在 status=0 时放行失败结果，误判为成功。
