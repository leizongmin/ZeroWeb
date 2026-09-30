---
date: 2026-09-30
modules: tests/wpt-runner,crates/media
---

# `rm -rf wpt-data` 重新克隆会静默丢掉 git-ignored 的 media 子集，`make test` 才暴露

## 问题描述

`make fetch-wpt-data` 的实现是 `rm -rf tests/wpt-runner/wpt-data` + 全新克隆。之后
`make test` 在 `zero-media` 失败：`webm_av_opus_track_decode_chain` 报
`Os NotFound`（`tests/wpt-runner/wpt-data/media/movie_5.webm` 不存在），而
reftest 等其他套件全部正常——wpt-data 目录看起来"存在且完整"。

## 根因分析

- `wpt-data/` 内 tracked 的部分（css/fonts/images/quirks/reftest-manifest.json）
  由克隆恢复；`media/` 子集是 **git-ignored** 的，由独立的
  `make fetch-wpt-media`（`tests/wpt-runner/scripts/fetch-media-subset.sh`）另行
  下载。
- Makefile 的存在性判断只看 `reftest-manifest.json`（tracked 文件），克隆后判定
  "已存在"，不会提示 media 子集缺失。
- 于是任何一次 wpt-data 重新克隆（刷新 ref、换分支验证）都会静默清空 media 测试
  素材，直到跑全量 `cargo test` 到 zero-media 才暴露，浪费一轮全量测试往返。

## 解决方案

wpt-data 重新克隆后补跑一次：`make fetch-wpt-media`。
验证：`ls tests/wpt-runner/wpt-data/media/movie_5.webm` 存在即可。

## 如何避免

- wpt-data 任何形式的删除/重克隆后，把 `make fetch-wpt-media` 作为固定收尾步骤。
- 排查口诀：`zero-media` 测试报 fixture NotFound 时，先查
  `wpt-data/media/` 目录是否存在，再怀疑代码。
