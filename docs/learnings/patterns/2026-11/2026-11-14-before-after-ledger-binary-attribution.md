---
date: 2026-11-14
modules: wpt-runner, js-dom-shim
---

# before/after ledger 的二进制归属核验：include_str! 拼接串子串比对

## 问题

wptfyi-editing d19 批复跑时，上一 session 因 API 限流中断，留下疑问：
`--json` ledger 是哪个二进制产的——后段 driver（post-driver2.sh）在最终
构建（09:28）前发射，ledger 完成时间（09:41）与构建时间交叉，无法从
mtime 断定它跑的是 09:19 中间态还是 09:28 终态二进制。若用中间态账目
写证据，F2P 会偏差（实测差 4F，全落 word 落点形域）。

## 根因

无人值守批的后台 driver 链（wait pid → 顺序跑多腿）与增量构建交错时，
"哪条腿用了哪个二进制"没有留痕；mtime 只能给出区间不能给出绑定。

## 解决

三层核验，全部只读、秒级：

1. **内容绑定核验**：`js_dom_bridge.rs` 用 `include_str!` 按固定顺序拼接
   shim 各 part（part01.js…part11-computed-color.js）。把当前树的全部
   part 按 `include_str!` 出现顺序 `cat` 成单串，对该串做
   `buf in open(binary,'rb').read()` 子串判定：
   - `True` → 该二进制的嵌入 JS 与当前树逐字节一致（排除 mtime 干扰）；
   - `False` → 二进制含旧 JS，tree ≠ 该二进制。
2. **行为钉测**：用小 pattern（如 `testharness-selection "modify"`）
   复跑定向 probe，与快照产物逐断言比对（`mismatches=0` 即终态复现）。
3. **裁决规则**：只有"启动时刻晚于最后一次源码编辑"的 ledger 才可作
   终账；存疑腿用终态二进制重跑（全语料 ~15-20 分钟）。跨腿对比出现
   位移时，先核文件清单差（并行流会向 wpt-data 加页）与超时键漂移
   （同页 Timeout 在 test 级/harness completion 级产生不同键名，集合差
   会误报 new-fail/vanished，需逐名核对 status 语义）。

d19 实测：终树拼接串（8,925,335 字节）同时包含于重建二进制与 09:28
快照二进制（字节级一致，18KB 尺寸差为 codegen 布局噪声），而 pre
（HEAD）二进制不含——归属当场定谳；selfull2 复跑证实其为中间态账
（差 4F 全落 09:27 末次编辑域），以终态重跑账为准。

## 避免方式

- 批内先落全部 JS 编辑、再构建一次终态二进制并 `cp` 留快照，之后才
  发射任何 ledger 腿；driver 顺序 = 构建 → 快照 → 各腿。
- 每条腿开头在输出文件名或 driver 日志里记 `sha256sum <binary>`，
  事后归属不用反推。
