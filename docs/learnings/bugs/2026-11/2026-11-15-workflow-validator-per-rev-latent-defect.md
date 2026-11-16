---
date: 2026-11-15
modules: site_optimizer
---

# workflow validator 必须随 rev 持续跑——潜伏缺陷跨 rev 复制放大，delivery 回执引用与信封契约三缺陷连爆

## 问题描述

perf-mem run 的 v4 账本在 rev25 之后连续七个 rev（rev26-32）没有跑
`scripts/verify-workflow.mjs` 全链校验，只在阶段末（rev33，task done 迁移）首次
校验即 EXIT=2。探针直调取 REAL_ERROR=`Invalid evidence reference`，逐 ref 扫描
定位出三个潜伏缺陷同时引爆：

1. rev29 把 merge 回执的**内容对象**（`{confirmed, commit, …}`）直接绑进
   `task.delivery.merge`，而 `verify-delivery.mjs` L37 的契约是
   `evidence(root, delivery[key])`——该字段必须是 `{path, sha256}` 引用；
2. merge 回执文件缺 `schema_version`/`task_id`/`artifacts` 信封字段（L38
   `Invalid delivery receipt` 三要求），且缺五身份键 `repo/pr/base_branch/
   base_sha/head_sha`（L43 `matches()` 恒等比对）；
3. integration 回执文件缺同款信封。

## 根因分析

rev25 之后每次落账只跑了账本脚本内的**局部断言**（revision、字段形状、sha
现算），没有跑全链 validator。rev25 时 `delivery.merge` 还是 null，三缺陷全部
不在校验面上；此后每个 rev 从上一 rev 深拷贝继承，缺陷逐 rev 复制。直到
rev33 的 done 迁移第一次让 `merged/integrated` 谓词进入校验面才爆断。局部
断言验证的是「脚本写对了」，validator 验证的才是「状态符合契约」——两者不可
互替。

另一个放大因素：`delivery.merge` 一旦非 null 就被 transition 检查冻结
（verify-workflow L264-267 `Merged PR identity changed`），错误形状向前不可变，
没有任何后续 rev 能在不动校验器的前提下改对它。

## 解决方案

- 存证-回滚-重放（T65 先例）：被拒快照改名 `*.rejected-mergeenvelope` 保留，
  v1 回执文件原样保留作历史存证；信封合规的 v2 回执新落盘（事实字段与 v1
  恒一），同一 rev 号重放五 rev，`delivery.merge/integration` 改绑 v2 引用。
- 纪律固化：**每个 completed 类 rev 落账后立即跑一次 workflow validator**，
  EXIT=1（校验过未完成）即通过存证；不能攒到阶段末。
- 契约备忘：`delivery.review/merge/integration` 三字段都是 `{path, sha256}`
  引用；三份回执文件都必须带 `schema_version=1`、`task_id`、非空
  `artifacts[]`（逐项过 evidence()）；merge 回执额外携带五身份键。
- CLI 报错只有一句概括时，用探针直调导出函数（`verifyWorkflow`）取
  REAL_ERROR，再逐 ref 扫描定位到具体字段。
