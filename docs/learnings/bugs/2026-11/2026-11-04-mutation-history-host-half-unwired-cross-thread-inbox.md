---
date: 2026-11-04
modules: engine
---

# timer 回调丢 data-\* 属性——读侧回落机制与宿主半边分离交付，跨线程接线是结构必需

## 问题描述

CDP evaluate + awaitPromise 泵 timer 的 runner 形态下，createElement 句柄元素同批 `dataset.seq = '0'` 即时可见；首批 evaluate 结束（宿主 drain+apply）后，timer 回调内 `dataset.seq`/`getAttribute('data-seq')` 翻转 `undefined`/`null`，而 JS 对象身份与子树结构保持。dom-ops 基准 `mut.replace.pool` 语义 FAIL（Chrome PASS）。

## 根因分析

handle 元素属性读链是两层设计：当前 mutation 批 latest-wins + `MUTATION_HISTORY` 线程本地历史回落（R100/slice48）。**读侧半边已就绪、宿主半边（apply 前 `append_mutation_history`）生产从未接线**——源码注释明记「三面同债，残余在册」，但债拖延的结构原因不是遗漏一行：renderer automation apply 在 **renderer 主线程** drain，读回调在 **js_worker 线程**执行，主线程直呼 `append_mutation_history` 写的是错误线程的 thread_local。半接线机制跨线程失效，直呼补线是错误修复。

判别关键：fresh-evaluate 臂——promise 落定后另起一次 evaluate 读同一属性返正常值，证明 renderer `cached_html` 完好、丢失仅在 await 循环内的 worker 视图，一步排除 renderer apply 路径。「同 API 新旧两次读结果分叉」先对数据源，与同页面 gBCR 缺陷1（2026-11-04）同一排查范式。

## 解决方案

收件箱桥：worker 启动时发布本 worker 的收件箱 `Arc<Mutex<Vec<DomMutation>>>`（thread_local 槽，每 worker 一份保多 tab 隔离），宿主 apply 前锁内 append，读回调回落前整批搬空进本线程历史（`sync_mutation_history_inbox`，锁不嵌套）。生产装配序 drain → append → apply 与引擎测试钉①（part46/48）镜像；文档换代在 worker Reset 臂同点清历史+收件箱。未发布宿主（webview 路径、测试）恒 None 零行为变化。

## 如何避免

- 读侧回落/兜底机制与写入侧（宿主半边）分离交付时，回落是死代码——接线完成的判据是「生产调用方存在且在同线程/经桥可达」，不是「函数已导出」；申报残余要写明**为什么接不了**（本例：跨线程），否则后来者会当一行漏调用直呼补线（错线程静默无效）。
- thread_local 只对同线程可见：凡「A 线程写、B 线程读」的共享状态，用进程级槽（gcs drain record 先例）或 per-worker 共享 Arc（本例收件箱）桥接；桥的内容按 worker 隔离（多 tab 同进程 handle 名可碰撞）。
- 半接线机制既存的引擎测试（直接 thread_local append）全绿不代表生产可用——测试装配镜像的是「同线程角色」，跨线程路径必须有独立端到端钉（真实 worker 两 execute 跨 drain）。
