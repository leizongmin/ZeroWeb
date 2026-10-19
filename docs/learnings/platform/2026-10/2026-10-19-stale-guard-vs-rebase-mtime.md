---
date: 2026-10-19
modules: engine,browser
---

# rebase/pull 翻新源文件 mtime 会触发 cdp 测试 stale binary 守卫

## 问题描述

`apps/browser/tests/cdp_focus_governance.rs` 等 spawn `target/debug/zero-renderer`
的集成测试带新鲜度守卫：二进制 mtime 早于 `crates/engine/src/js_dom_shim/`
下最新源文件 mtime 即 panic「stale … — rebuild zero-renderer」。共享树并行
开发时，另一流 `git pull --rebase` 会把未改动的 shim 文件 checkout 重写
（mtime 整体翻新到 rebase 时刻），使刚构建的 debug 二进制瞬间「变旧」，
`cargo test -p zero-browser` 大面积假失败（同 suite 内先跑的案过了、后跑的
案撞守卫，失败数在 3F/6F 间漂移，看似行为回归）。

## 根因分析

守卫以 mtime 为新鲜度依据；git checkout/rebase 无条件重写工作区文件 mtime，
与二进制构建时刻解耦。二进制内容可能完全一致（rebase 前后 shim 内容不变），
但 mtime 序被翻转。多 agent 共享树 + 各自跑 make test 时必然间歇踩中。

## 解决方案

撞到该 panic 时先查两侧 mtime（`stat target/debug/zero-renderer` vs
`find crates/engine/src/js_dom_shim -newer <bin>`），确认是 mtime 翻转后
直接重建 standalone 二进制再重跑该腿（`cargo build -p zero-renderer
-p zero-compositor -p zero-image-decoder`），不要按行为回归排查。
运行侧教训与切片账本「stale binary 假回归」（evidence m2s11 排查记录 1）
同根：依赖二进制的测试前，必须确认构建产物新鲜。
