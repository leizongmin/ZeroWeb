---
date: 2026-10-09
modules: build-support,apps/browser
---

# target-disk-guard 三级全量清空会抹除多段链前序段的 release 产物

## 问题描述

合并树集成链（fmt → clippy → release 构建 zero-browser → make test → make reftest → WPT → 活体五面）跑到 reftest 段时，其前置 `target-disk-guard` 检测 `target/` 占用 70GB 超过 50GB 阈值：第一级删 `incremental/`（13GB）后仍超，按第三级策略**全量删除 `target/`**。两分钟前 release 段刚产出的 `target/release/zero-browser` 随之消失，链尾活体段启动浏览器时报 `env: '.../target/release/zero-browser': No such file or directory`（NO_CDP）失败，而链上各构建段退出码均为 0，表面看不出任何异常。

## 根因分析

- `scripts/target-disk-guard.sh` 挂在 `make test` / `make reftest` / `make browser-build` 等重型入口前置，磁盘超阈值时全量清空 `target/` 是设计行为（防止 partial 清理破坏 cargo 增量一致性）。单次 make 调用内这没问题；问题出在**多段长链**：前序段自建的二进制对后序段来说没有任何"仍存在"的保证。
- 该守卫触发时只在当前段的 log 里留几行 echo；链式脚本若不聚合检查，产物消失这一事实会被 rc=0 的各段掩盖，直到链尾实际消费者（浏览器启动）才暴露。
- `make test` 阶段（debug 测试 + 增量缓存）正是把 `target/` 推过 50GB 阈值的段——即"消耗磁盘的段"和"依赖前序产物的段"在同一条链上共存。

## 解决方案

- 链式脚本两类改法（任选或并用）：① 在依赖前序构建产物的段（如活体启动）前加产物存在性自检，缺失则重建再继续；② 把"可能触发守卫的段"（make test/reftest/build 家族）与"消费产物的段"显式分段，产物消费统一放在构建之后、且段间不隔重型 make 入口。
- 本轮实际处置（未改链脚本，事后补跑）：独立 log 重建 `make browser-build`（重建前确认 target/ 已被守卫清到 2.4GB，不会二次触发），重跑活体段，并在 chain-status.log 追加处置注记（原 run 内容未动）。
- 通用教训：rc=0 的构建段只证明"当时构建成功"，不证明"产物仍在"。跨段传递产物时，消费段的证据应以产物实存（或消费成功）为准；磁盘守卫这类会"顺手清场"的前置，等价于链上一次隐式 `cargo clean`。
