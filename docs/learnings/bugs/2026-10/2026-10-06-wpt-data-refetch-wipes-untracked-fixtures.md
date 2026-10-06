---
date: 2026-10-06
modules: wpt-runner,build-support
---

# make import-wpt 连锁 fetch-wpt-data 重建目录，静默抹掉未入 tag 的 media fixture

## 问题

`make test`（workspace 全量）在零媒体相关改动下失败：
`crates/media/src/tests.rs::webm_av_opus_track_decode_chain` 报
`No such file or directory`，读不到 `tests/wpt-runner/wpt-data/media/movie_5.webm`。
当天早些时候同一命令还是 exit 0。

## 根因

1. `Makefile` 的 `import-wpt` 目标依赖 `fetch-wpt-data`；后者的守卫条件是
   `if [ -f "$(WPT_DATA_DIR)/reftest-manifest.json" ]`——**manifest 缺失即
   `rm -rf` 整个 wpt-data 目录并重clone默认 tag（v1.10）**。
2. `wpt-data` 目录本身不被主仓 git 跟踪（clone 后 `.git` 也被删），各 tag 子集
   只含 reftest/资源白名单；`media/*.webm` 等 fixture 由**独立入口**
   `make fetch-wpt-media`（`fetch-media-subset.sh` 白名单）事后补进同一目录。
3. 于是当旧 wpt-data 目录恰好没有根级 manifest（旧版 fetch 格式或部分初始化）
   时，一次普通的 `make import-wpt` 会静默触发 rm -rf + 重 clone：
   reftest 资产回到 v1.10，而补装的 media fixture 全部丢失——远端 tag 从未含
   `media/`（v1.9/v1.10 均 404），没有任何机制把它带回来。
4. 失败面与改动面完全无关（布局修复 → 媒体测试红），极易误判为回归。

## 解决

- 恢复：`make fetch-wpt-media`（正规入口，白名单含 movie_5.webm；网络慢时
  movie_300 等大文件可能超时，cargo test 只依赖 movie_5，可后续重跑补齐）。
- 重跑 `make test` 确认 exit 0。

## 如何避免

- 在依赖 wpt-data 的 Make 目标（import-wpt/reftest/testharness-*）前，若本机
  曾手工补装过 fixture（fetch-wpt-media / fetch-wpt-* 家族），先确认
  `reftest-manifest.json` 存在再触发 fetch-wpt-data；缺失时先走
  `make reftest`（其自身依赖 fetch-wpt-data）完成一次带 manifest 的完整初始化，
  再补装 media。
- 结构性改进候选（未实施）：fetch-wpt-data 的守卫从「manifest 存在与否」改为
  「目录存在与否」，或把 media 白名单并入 wpt-data tag 子集，使 rm -rf 重建
  不再丢资产。
