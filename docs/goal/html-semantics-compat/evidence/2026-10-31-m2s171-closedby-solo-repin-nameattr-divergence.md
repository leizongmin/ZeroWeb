# M2-S171 closedby 漂移域 solo 冷却钉态复跑 + name-attribute 跨群独立性分叉勘根（s167 挂账销账 + 新挂账）

**日期**: 2026-10-31
**刀位**: s167 挂账「closedby/details 漂移域 solo 冷却复跑钉态」——s170 入场时负载窗不成立（load 92）推迟，本轮 load 1min 5.58 solo 冷却态成立。纯复跑 + 勘根轮，零代码改动。

## closedby 漂移域钉态（s167 挂账销账）

`make testharness-html-semantics TIME_LIMIT=9000 FILTER=interactive-elements` → **438P/45F/0T**（raw `/tmp/zw-s171-closedby-solo.log`，runner = b100c742a 组合态 09:27 构建）。Timeout 0 无截断。逐键对比（(key,state) 对 sort -u）：

- **closedby complex+simple 两文件 46 键与 s166 full1 基线逐键完全一致**（diff 空），30P/16F——s167 全量 join 的 P2F 13 键中 closedby 面 11 键（complex ×8 + simple ×3）全部 Fail→Pass 回绿。s167 重载态 19P/27F 的 11 键漂移定谳为环境时序抖动（hit-test 落点），solo 冷却态回归 s166 基线，域钉态 = 30P/16F。
- auto-expand 单键 s166 vs solo **IDENTICAL**。
- raw 对比：`/tmp/zw-s166-full1.log`、`/tmp/zw-s167-closedby.log`（s167 定向态 19P/27F 实为 complex+simple 两文件全案）。

s167 挂账「solo 冷却复跑钉态」**resolved**。

## name-attribute 分叉勘根（新挂账，未定谳）

P2F 13 键余下 name-attribute 1 键 solo 态**保持 Fail 不回绿**，与 closedby 性质分离：

- 失败案：`name-attribute.html :: handling of name attribute changes`——`assert_array_equals states before first mutation expected [1,0,1] got [1,0,0]`。语义：`name="a"` 群（e0 open/e1）+ `name="b"` 群（e2 open）经 innerHTML 注入，**跨 name 群独立性**——e2 属 b 群唯一成员应保持 open，实得 closed（b 群被 a 群互斥误伤）。同目录其余 name 群案（parser 互斥、exclusivity enforcement、insertion into group、未连接互斥等）全绿，details 目录 41P/2F（另一 F = closed-details-layout-apis 布局域既有）。
- **时间线**：s161 full4 (10-30 23:12) P → s162 full1 (00:17) P → s164 full1 (01:08) P → s166 full1 (03:39) P → **s167 stash 二分 (05:06, HEAD a897d536b) F 起**，s171 solo 全域 + 定向×2（同 b100c742a 二进制）三连 F 稳定。非单跑抖动，是 s166 full1 二进制与 s167 重建二进制之间的持续态分叉。
- **c1f8e9b63 排除**：03:00 进的并发流「撞车仲裁终验」是窗口内最大嫌疑（engine part01 重锚+回灌 stamping 旁路 + part04 门），但 diff 全部在 CE caret 世界同步路径（`__zwCeWorldSyncing` 置位/复位 try/finally 无泄漏面、门恒 falsy 时 stamping 行为与旧版逐字节一致），与 details name 群零交集。
- **嫌疑面收窄**：①s166 full1 (03:39) 构建树与 a897d536b (03:54) 提交树若仅差提交化则内容等价，P→F 需另有根因；②release 重建非字节同构（代码布局/内嵌偏移时序漂移）——但同二进制三连稳定 F 又非随机抖动形态；③解析注入路径属性挂载/互斥扫描时序敏感（`__zwDetailsScan4100` 解析面 given-prong 顺序插入模拟本身按 name 分群逻辑正确，误关路径未定位）。
- **下一步**：worktree 检出 s166 树（a897d536b）重建 runner 二分（worktree 前置见 memory：rusty_v8 档案 + wpt-data 语料手拷）定谳「s166 树已 F」还是「重建二分引入」；前者继续 bisect s164↔s166 间 main 线提交，后者查解析注入时序根修。

## 验证面注记

本轮零代码改动，无 lib/语料回归面。复跑 raw 落盘 `/tmp/zw-s171-closedby-solo.log`、`/tmp/zw-s171-details-dir.log`、`/tmp/zw-s171-nameattr-solo.log`。
