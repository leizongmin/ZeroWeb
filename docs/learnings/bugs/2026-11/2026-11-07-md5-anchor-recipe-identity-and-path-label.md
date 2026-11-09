---
date: 2026-11-07
modules: tests/wpt-runner, docs/goal/editing-compat
---

# 批脚本 md5 锚两陷阱：expect/got 配方必须同一函数；md5sum 行配方对路径标签敏感

## 问题描述

knife11 gates 批（2026-11-07）pre 态核验假 ABORT：`PRE_EXPECT`（HEAD blob 串接
内容 md5）与 `md5snap()`（`md5sum $SHIM4 | md5sum`）恒不等，守卫把正常 pre 态
判成 SHIM-DRIFT。同日核验侧另现伪漂移读数：同一四文件字节，裸文件名口径
（`cd /tmp && md5sum part01.js …`）与全相对路径口径（verdict 批 ：26
`md5sum crates/engine/src/js_dom_shim/part01.js …`）得出不同 md5。

## 根因分析

- `md5sum A B C | md5sum` 哈希的是 **md5sum 的输出行**，行内含文件路径串——
  路径标签（裸名 vs 全相对路径）参与摘要，同一字节两口径两值。锚值跨口径
  比对必然假阳性/假阴性。
- expect 与 got 用不同实现（blob 串接 vs 清单行再哈希）即使字节相同也恒不等。
- 后果链：假 ABORT 触发 stash push/pop 周期，未提交刀体在工作区两进两出，
  并发核验轮期间被误读为「两次不明抹除」，消耗三轮排查。

## 解决方案

- 守卫脚本的 expect 与 got **必须调用同一配方函数**（本例统一为
  `md5snap()`：清单行再 md5）；pre 态期望值对 HEAD blob 落临时文件后按同
  函数现算，不手拼内容摘要。
- 引用/传递锚值时**随值携带口径**（配方 + 路径标签 + 基线 commit）；换基
  传递定谳走「刀体 hunk 与原 blob 逐字节对账 + 基座增量限无关区」链路
  （见 `docs/goal/editing-compat/evidence/2026-11-07-knife11-anchor-rebase-399090bd.md`）。
- 读数异常先复核自身口径再怀疑字节：blob 级 `git diff <锚对象> -- <文件>`
  零差即字节恒等，md5 差异优先找标签差。
