# M2 s318 — runner reftest-wait TestRendered 派发：the-option-element 簇 1F→1/1 全绿 + css 动态案 2 红搭车转绿

日期：2026-11-07。切片号 s318（承 s317 下一步候选①小簇排队：option 1 勘根先行）。

## 勘根/边界确认

- **簇红页形态**：the-option-element reftest 簇仅 1 案
  dynamic-content-change-rendering ✗0.11%（510/480000 px，max ch=255；页带 fuzzy
  `maxDifference=0-3;totalPixels=0-20`——即便生效 510>20 也不过，非容差面）。
- **页面结构**：test = 空 `<option>` + script 追加 `<span>` → `TestRendered`
  listener 中置 `textContent='foo'/'bar'` 并摘 reftest-wait class；ref = 静态
  `<option>foo</option>`。考语义 = HTML §4.10.10 concept-option-label（option
  后代文本）+ 首帧后动态变更重绘。
- **根因（runner harness 面）**：`TestRendered` 全仓零命中——runner 从不派发该
  事件，页面 listener 永不触发 → textContent 不执行 → test 侧 select 空标签，
  ref 侧 'foo'/'bar' 字形 ≈510 px 差。layout-dump 探针：select w=30 各带 anon
  空盒（首帧态正确；dump 不含 TestRendered 后状态，html 仍挂 reftest-wait
  class——listener 未跑的旁证）。
- **上游契约核源**：wptrunner 对带 reftest-wait 的页面在首次渲染后于
  documentElement 派发 `TestRendered`（页面借此做首帧后变更再摘 class 允许截图；
  https://web-platform-tests.org/writing-tests/reftests.html#controlling-when-
  comparison-occurs）；语料 common/reftest-wait.js 在库（takeScreenshot =
  classList.remove）。
- **静态后代文本面排除**：option_label_text → `doc.text_content` 递归收集（span
  后代文本在内）；paint_select_value（R1679）显式绘 selected option 标签——
  静态结构面无缺口，纯派发面缺失。

## 施工

reftest_scripts.rs `apply_scripted_dom_mutations`：load 派发后增 TestRendered
派发（单行 execute，镜像 `'load'` 先例形态）——guard `documentElement.classList.
contains('reftest-wait')`（上游仅 reftest-wait 流程触达；无监听页 no-op；ref 页
同流程上游同样派发，uses-label-dynamic-ref 在名单佐证）；mutation 随下方
recorded 一次应用。多轮重入（listener 摘后重挂 reftest-wait 等下一轮）未模拟
——FIXME 无语料驱动，单轮覆盖既有全部 42 案。

## 爆炸半径

全语料 grep `TestRendered` 42 页：css 流域 38（overflow-alignment 20、
backgrounds 4、contain 5、transforms 2、ui 3、selectors 1）+ html/semantics 4
（option 1 + select 3，其中 1 为 ref）。reftest-wait 无 listener 页 no-op；全量
before/after 归因闭环（下方验证）。

## 验证

- **option 簇 solo ✗0.11% → ✓0.00%**（1/1 全绿，rc=0）。日志
  [logs/s318-option-initial.log](logs/s318-option-initial.log) →
  [logs/s318-option-after.log](logs/s318-option-after.log)。
- **css 动态案 2 红搭车转绿（确定性修复，证据链 = clean-tree stash 探针 solo 红
  / with-change solo + 全量 ×3 绿）**：
  - transform-3d-scales-different-x-y-dynamic-001 ✗2.64% → ✓0.00%；
  - content-visibility dynamic-change-paint-fully-obscuring-child-001
    ✗2.08% → ✓0.00%。探针 logs/s318-probe-*.log、终态
    logs/s318-after-solo-*.log。
- **全量翻转矩阵**：before [logs/s318-upstream-before.log](logs/s318-upstream-before.log)
  15987P/1033F → after 15994P/1026F（+3 确定性修复；run2 15996/1024、run3
  15995/1025——flake 噪声带 ±6）。
- **时序 flake 6 案定性（s292 同二进制跨跑协议）**：table-anonymous-objects-128/
  148/151、aspect-ratio-intrinsic-size-002、position-relative-004、
  insert-block-in-blocks-n-inlines-end-001——双二进制 solo 全绿（clean-tree 探针
  logs/s318-probe-t128.log 等 4 份），全量 8-job 并行双向翻转（insert-block
  ✓✗✓✓、151 ✗✓✓✗）；5 案无 reftest-wait（dispatch guard 惰性证明），insert-block
  带 reftest-wait 但 `className=""` 摘 class、无 listener（派发 no-op）。不记账
  为修复或回归。
- select 簇其余 TestRendered 页（reset-algorithm-rendering、uses-label-dynamic）
  before/after 恒 ✓（派发无实效，零扰动）。
- DC-14 内部集 725/725 零扰动（before/after 同值）。日志
  [logs/s318-dc14-before.log](logs/s318-dc14-before.log) /
  [logs/s318-dc14-after.log](logs/s318-dc14-after.log)。
- runner clippy 零告警（logs/s318-clippy.log）。
- make test（guarded-test，后台全量）终态 **80 腿 ok + engine lib 3783P/9F**
  （与 s316/s317 终态同键集——本切片 diff 仅 reftest_scripts.rs，runner/引擎零
  新失败）。日志 [logs/s318-mt.log](logs/s318-mt.log)。

## 语义边界（记录）

- 派发时机 = load 派发后脚本窗口末（上游「真首帧渲染后」的串行近似——harness
  无 live 渲染循环）；TestRendered 事件对象 = 泛型 Event（bubbles/cancelable
  true），既有页面仅消费 `e.target`（removeAttribute）。
- runner harness 变更跨流共享：css 流域 2 案搭车转绿为 harness 修复红利（css
  引擎面零触碰）；perf-gate 豁免（每 reftest-wait 页一次 sandbox execute 短分支，
  非引擎热路径；承 s311-s317 先例）。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：s318-option-{initial,
after}、s318-upstream-{before,after,run2,run3}、s318-probe-*（clean-tree solo
5 份）、s318-after-solo-*（2 份）、s318-dc14-{before,after}、s318-clippy、
s318-mt（全量回归门禁）。

代码锚：tests/wpt-runner/src/reftest/reftest_scripts.rs（TestRendered 派发单点，
load 派发后）。

注记：本切片 diff 落 runner 1 文件；engine/dom/webview 零触碰。回归面 = 语料
case 本体（全量 reftest 每 25s 跑一遍，先例 s316 动态案同款）。

## 下一步候选

1. 小簇排队承前：wbr 1 / iframe 1 逐簇勘根；br 1 勘根前先 solo 复核
   editing-compat br 刀（fe760dc57）外溢现势。
2. （挂账承前）embed fallback-image 与 print pagination 7 键跨流移交跟踪；
   range-intrinsic-size 移交渲染流域跟踪；全量并行时序 flake 6 案（本切片定性）
   如需收口归渲染/runner 共享面另行立项。
