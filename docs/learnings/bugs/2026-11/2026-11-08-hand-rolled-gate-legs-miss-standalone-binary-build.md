---
date: 2026-11-08
modules: engine, webdriver, testing
---

# 手搓 make test 镜像腿缺前置 standalone binary 构建——cargo fail-fast 放大成整腿早停

## 问题描述

2026-11-08 shadow-dom M3 追加轮门禁链（legs9）的 workspace v8 腿
（`cargo test --workspace --exclude zero-browser --exclude zero-renderer`）
只跑出 42P/11F 即以 rc=101 收场（同树 R3b 基线 3814P/6F）。11 个失败全部
集中在 `apps/webdriver/tests/http_session.rs`，首案 panic
「zero-webdriver 未就绪」，余 10 案全是同一把 spawn 锁的 PoisonError 级联。

## 根因分析

两层叠加：

1. **工件缺失**：`zero-webdriver` 启动时绑定 HTTP 端口后立刻做 init，
   `resolve_renderer_binary()`（session.rs）找不到 `target/debug/zero-renderer`
   → 进程在 ~35ms 内退出。测试首案每 100ms 探测一次 TCP、共 5s——35ms 的
   存活窗口恰好错过全部探测点 →「未就绪」→ 锁中毒级联。
   当天多轮 feature 翻转链（quickjs/renderer/browser/reftest/smoke）加上
   engine shim 源码在树编辑弄脏 renderer bin，把该工件清掉了。
2. **cargo fail-fast 放大**：`cargo test`（非 `--no-fail-fast`）在第一个
   失败的测试二进制后直接中止后续二进制。`apps/webdriver` 按字母序排在
   workspace 测试目标最前 → 11F 让整条 workspace 腿跑了 8 个二进制就停，
   余下 ~99% 用例根本没执行。

规范入口 `make test` 在测试前显式
`cargo build -p zero-renderer -p zero-compositor -p zero-image-decoder -p zero-media-host`
（Makefile 注释：「刷新 standalone binaries」），所以从未踩中；手搓镜像腿
照抄了 cargo test 命令却漏了这一步前置构建。

## 解决方案

- 判读顺序：workspace 腿 P/F 数远低于基线 + 失败集中在单一二进制 +
  其余全是 PoisonError → 先查「首败二进制为何失败、工件是否存在」，
  不要按失败面归因行为回归。
- 复现/证伪路径：直接 spawn `target/debug/zero-webdriver --port <p>` 观察
  init 日志（监听→退出仅 35ms），即可把 TCP 超时类失败钉到 init 失败。
- 手搓 `make test` 镜像腿时必须带上 standalone binaries 前置构建；或者
  干脆跑规范 `make test` 入口（fmt 前置 + 构建前置都在里面）。事后恢复：
  任一 v8 default 模式的 `-p zero-renderer --bin` 腿构建即自动补回工件。
- 该形态还有一个副作用要会认：同一个缺失也会让 browser tests 的
  mtime/陈旧二进制守卫类红与真行为红混在一起，判读先做 provenance
  （工件 mtime vs 源码 mtime）。
