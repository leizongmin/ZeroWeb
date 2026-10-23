# 用户决策：parser form element pointer 4F 缓行

**日期**：2026-10-23（用户采纳建议裁定）
**对象**：挂账序首位「parser form element pointer 表规则 4F」——form-control-infrastructure/form.html
`label-in-table-with-control` / `label-in-table-for` 2F + form_owner_and_table.html / _3 各 1F，
同根（s17 起「深结构单独立项征询」口径维持至本决策）。

## 决策内容

**缓行，本轮不立项。**该项从「用户决策门禁（待征询）」转为「已缓行（打捆待触发）」；
流按其余挂账序继续推进，不再在每轮「下一步」首位重复征询。

## 理由（入档备查）

1. **增益 0.05%**：4 subtest / 九域 corpus 7488 subtest；涉及 table 内嵌 `<form>` 遗留写法，
   真实站点罕见。
2. **成本最高类改动**：根因在树构建器 form element pointer（table 内 form 插入即弹出栈、
   指针保留，后续 form-associated 元素按指针而非 DOM 祖先关联）——最终 DOM 树无结构痕迹，
   shim 树上行查找（R4005 路数）不可达，必须动 parser 核心（或跨层随 snapshot 传解析期状态）。
   parser 是每次页面加载必经路径，改动需全量重验（make test 76 腿 + reftest 712 + 全 corpus），
   回归面为「所有解析」。
3. **机会成本**：dialog 集群余 ~65F 与 ESC 组栈边缘、custom command 等挂账均为 shim 面轻修复；
   单独清本项也不满足 oracle 流解封条件（挂账须清空）。

## 打捆处置（触发条件与形态）

与**挂账 H（declarative shadow DOM，`<template shadowrootmode>`，s34 定性 parser + shadow 树
渲染联合缺口）**打捆为一个 **parser 批次**，M2 收口后尾段**单独立项**征询——一次全量重验摊
两个深结构改动。

触发条件（三条同时满足再立项）：
1. shim 面挂账吃干净（ESC 组栈边缘 / custom command `--` / 非模态 close / R371 模板残留 /
   summary-less details 翻转 / focus-after-close 深层等）；
2. dialog 集群收口（interactive-elements 域分级通过率两轮不下行）；
3. rendering-compat 协调好 parser 占用窗口（run-rules §9 碰撞声明）。

关联证据：[m2s17-r4005](2026-10-21-m2s17-r4005.md) §余 2F / [m2s18-r4007](2026-10-21-m2s18-r4007.md)
§挂账更新（4F 同根定谳）。
