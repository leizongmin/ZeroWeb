# DC-3 · 批二十三：per-global 支撑闭包补档（object-association.js 全局缺档定谳与修复）

2026-11-23。批二十二合成 Fail 例行扫描唯一 html 面残留——
`the-navigator-object/per-global.window.js` 两 run 同形 1 synthetic Fail
`os error 2`（0P）——定谳与修复。无 runner 码笔。

## 定谳：隔离律第 8 例＝全局缺档变体

META `script=/common/object-association.js` 缺档。与前七例不同形：
**本地与 CI 同缺**（非分片隔离）——该文件从未被任何 goal 脚本拉取，
本地在册副本也不存在；`testIsPerWindow("navigator"/"clientInformation")`
两行即依赖它，页面从未实际运行，静态 `<script src>` 闭包扫描（批十五）
与运行期引用闭包扫描（批十六）都不可见——扫描输入为零（无失败页面、
无运行期错误）→ 只能由合成 Fail 例行扫描（批二十教训）抓出。

## 修复与验证

- 57 脚本（webappapis 语料 owner，在 syntax-dom 分片 fetch 清单）
  `fetch_raw "common/object-association.js"` ＋注释。
- `common/blank.html`（页面 iframe 目标，0 字节空页）由
  `fetch-wpt-html-testharness` 引导步全分片带入，不需补。
- 上游四路核对（批十七教训②）：jsdelivr/raw 字节相同、contents API
  2700B sha 669c17c07b1a、git trees 在册。
- 挪走重拉（批十七教训①）：exit=0、FAILED_FETCHES=0、与上游副本
  `cmp` 字节相同。
- 本地实跑（release runner 重建 HEAD c52ff813f，systemd scope 24G，
  3m43s）：**1 synthetic Fail → 6 注册/3P/3F**。navigator 三测全过
  （about:blank 导航恒等/iframe.remove 恒等/document.open 恒等）；
  clientInformation 三测 `assert_implements` 红＝engine 缺 navigator
  别名（js_dom_shim 面，新增刀位候选，~3S 上限）。

## CI 兑现与残留

+3P 随下轮 run 兑现（webappapis face 1,537→~1,540；页面为 3 iframe 案，
非墙钟型，CI 90s 页预算同形）。58 域同文件两用户
（the-location-interface/per-global、
history/the-history-interface/history-associated-with-document）目录不在
58 拉取面＝批五 browsers 余 102 键挂账既有项，扩面时沿用本补档。

[逐文件 JSON](2026-11-23-dc3-batch23-navigator-object-support-closure.json) ·
[批二十二证据](2026-11-23-dc3-batch22-run-reconciliation-three-fixes-and-rt-spillover.md) ·
[前置账](2026-11-22-dc3-goal-accounting-and-corpus-runway.md)
