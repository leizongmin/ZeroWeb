---
date: 2026-11-15
modules: tooling
---

# git diff pathspec 按 cwd 相对解析——子目录里跑仓库根路径 pathspec 静默伪空

## 问题描述

在运行目录（仓库根的子目录 `.acceptance/site-optimizer/<run>/`）里执行
`git diff <a> <b> -- crates/...` 比对合并树与候选树，输出恒空，据此得出
「main 携入全是 docs、合并树==候选树」的错误结论。连锁伪影一串：`--stat`
尾部截断被误读成全量、`ls-tree` 的 mode 列为空、name-only 能列出文件但单独
diff 该文件仍空——直到 `git diff --raw`（无 pathspec）与 blob sha 直接比对
才暴露真相：合并树相对候选树有 5 个代码文件 405+/15- 的真实差异。

## 根因分析

pathspec 参数按**当前工作目录**相对解析，不是按仓库根。cwd 在子目录时，
`crates/...` 解析成 `<run>/crates/...`——不存在的路径不报错，diff 静默返回空。
空输出被当成「无差异」而不是「路径没匹配上」，下游判读全部建立在伪空上。
`--stat` 只显示部分文件是同一原因（pathspec 过滤后只命中根目录下的 docs），
与 tail 截断叠加后伪影更难察觉。

## 解决方案

- 跨树比对一律 `cd` 仓库根再跑，或干脆不用 pathspec。
- 权威判据按优先级：blob sha 直接比对 > `git diff --raw`（无 pathspec）>
  tree-id 对树对象。`git diff <a> <b> -- <path>` 的空输出不能当「无差异」
  证据，先验证 pathspec 是否命中（`--name-only` 与 blob 比对互证）。
- 结论改变归因时（如「合并树==候选树」），必须有一条无 pathspec 的原始
  diff 作底账，防止伪空链式传播到判据复检决策。
