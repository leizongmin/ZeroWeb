---
date: 2026-10-04
modules: zero-media,tests/wpt-runner,Makefile
---

# fresh worktree 跑 make test / 构建前必须先拉 wpt-data 素材与 rusty_v8 archive

## 问题描述

在新 `git worktree`（detached，干净检出）里有两类确定性缺件：

**构建面**：`cargo build -p zero-browser`（含 v8 依赖）直接 101——`failed to run custom build command for v8 v150.2.0`，build.rs:937 panic `NotFound`。v8 build script 找的是**worktree 内相对路径** `.cargo/rusty_v8/archive`（日志 stdout 明示 `static lib URL: <worktree>/.cargo/rusty_v8/archive`），gitignored，fresh worktree 必缺。

**测试面**：跑 `make test`，zero-media 两测确定性失败（复现 2/2）：

- `tests::webm_av_opus_track_decode_chain` — `crates/media/src/tests.rs:455:32` panic：`Os { code: 2, kind: NotFound }`
- `tests::webm_colour_identity_full_range_passthrough` — `crates/media/src/tests.rs:129:32` 同型

其余 webm/vp9 测试通过，仅这两个缺文件。

## 根因分析

两个测试经 `decode::workspace_path("tests/wpt-runner/wpt-data/...")` 读 WPT 素材（`media/movie_5.webm`、`css/css-sizing/aspect-ratio/support/2x2-green.webm`）。`tests/wpt-runner/wpt-data/` 是 **gitignored 的外部拉取物**：

- `make fetch-wpt-data` — 从 pinned 分支克隆 `zeroweb-wpt-data` 仓库（含 reftest manifest、imported 资源闭合检查）
- `make fetch-wpt-media` — `tests/wpt-runner/scripts/fetch-media-subset.sh`（103 files + track 目录 @WPT 31597693…），`movie_5.webm` 在这一层

主工作区早拉过所以一直在；fresh worktree 两层都缺 → 读文件即 NotFound。易误判为 flake（同 PR 交付轮 run1 失败 run2 绿的先例让人往偶发上想）——判别要点：**同命令重跑同腿同 panic 位置 = 确定性缺件，不是 flake**。

## 解决方案

fresh worktree 上按序：

```bash
# 构建前置：rusty_v8 静态库符号链接（与既有 worktree 同构）
mkdir -p .cargo/rusty_v8
ln -s ~/.cache/zero-web/rusty_v8/v150.2.0/librusty_v8_release_x86_64-unknown-linux-gnu.a.gz \
      .cargo/rusty_v8/archive

# 测试前置：WPT 素材两层
make fetch-wpt-data    # pinned 克隆（有 reftest-manifest.json 则跳过）
make fetch-wpt-media   # 媒体子集（幂等，报 WPT pin）
make test
```

校验素材身份可对比 sha256（如 movie_5.webm = b1d79ce4…，与既有工作树逐位同）。

## 如何避免

- worktree 建好后的首次构建/`make test` 前，固定先补 rusty_v8 符号链接 + 两个 fetch target（或把该前置写进 run-rules/任务书模板）。rusty_v8 符号链接的专门分析见 [2026-09-30-worktree-rusty-v8-archive-symlink.md](../2026-09/2026-09-30-worktree-rusty-v8-archive-symlink.md)。
- v8 构建失败时先读 build script 的 stdout：它直接打印期待的 archive 路径，一眼定位缺件。
- 失败分诊时先区分「同位置复现的 NotFound」（缺件）与「跨跑位置漂移的失败」（flake/顺序依赖）。
