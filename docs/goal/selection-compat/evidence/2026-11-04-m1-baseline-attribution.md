# M1 扩面基线 + 失败簇归因 + coverage 复测（2026-11-04）

## 运行方式

```
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 7200 -- \
    ./target/release/zero-wpt-runner testharness-selection --json
```

runner 通道 `testharness-selection`（本次起 `selection/` 根递归扫描，见
[语料扩面证据](2026-11-04-m1-corpus-expansion.md)）。corpus pin
`315976933870`（191 blob 哈希核对）+ 8 文件 pin 漂移 HEAD `43f66fcb72f7`。
墙钟 ~11 分钟。逐 subtest 原始结果：
[`2026-11-04-m1-baseline.json.gz`](2026-11-04-m1-baseline.json.gz)（516 KB，
675 案 / 51,757 subtests，后续轮次零回归对照锚）。

## 总账（DC-2）

- **35,637P / 16,116F / 4N（51,757 subtests，675 案），通过率 68.86%**
- 其中 editing 面（33 号域，随共享语料扩张入通道）：13,334P / 2,360F
- **selection 面：22,303P / 13,756F / 3N（36,062 subtests）**

| family | P | F | 通过率 |
|---|---:|---:|---:|
| selection/(root) | 22,142 | 13,342 | 62.40% |
| selection/contenteditable | 17 | 235 | 6.75% |
| selection/shadow-dom | 46 | 64 | 41.82% |
| selection/bidi | 0 | 92 | 0.00% |
| selection/caret | 12 | 18 | 40.00% |
| selection/anonymous | 10 | 3 | 76.92% |
| selection/textcontrols | 76 | 2 | 97.44% |
| editing（33 号域，通道内记账） | 13,334 | 2,360 | 85.10% |

## 一期锚面逐 subtest 对照（DC-2 零回归门）

锚面 = 一期 fetch 脚本 27 案（20 CASES + test-iframe + 3 textcontrols +
3 editing）。语料与 pin 逐哈希一致（编辑锚面 3 文件单独复核 MATCH）。

- **selection 锚面 25 案：subtest 计数与 wpt.fyi 三家 browser 基准逐案完全
  一致**（合计 2,815 = browser 2,814 + test-iframe；selectAllChildren
  2,242、deleteFromDocument 60、setBaseAndExtent 120 等无一漂移）。
- textcontrols 锚 3 案（66 subtests）全过，与一期一致。
- 一期终态失败 2F 现状：
  - `Document-open.html`（document.open 缺失）——**原样保留**（跨域挂账不变）。
  - `editing/event.html :: input event`——**原样保留**（editing 域）。
- **对一期锚新增回归 4F（全部归因 33 号流共享路径，挂账待转登记）**：
  1. `selection/deleteFromDocument-HTMLDetails.html :: abc{<details><summary>…`
     ——期望 `abc<details></details>jkl` 得 `abcjkl`（details 空壳未保留）。
     内容归因：`7ddf00c39`（F6 delete/forwardDelete 撤壳，part01f
     deleteSelection 忠实移植，10-30）——deleteFromDocument 走同一删除
     算法；一期终态（10-15）该案全过。
  2. `editing/event.html :: Command fontSize value ""/"quasit"/"15px"
     input event` ×3——期望 input 事件 1 次实得 0。内容归因：`90c992231`
     （style-with-value 簇实应用面，11-01）——fontSize 此前为「零变更臂」
     照派事件，实应用改造后 `__zw_ce_value_validate` 拒绝路径零事件，
     与 event.html 期望（无效值也派 input）冲突。
     两条均为**内容归因**（commit diff + 失败面吻合）；33 号流正在同域
     活跃改造，bisect 复验归 33 号侧复核，不在本 goal 硬解（run-rules §9
     碰头不硬解）。
- editing/event.html subtest 180 vs 一期 151（+29）：文件静态、语料一致；
  生成腿增加 = 编辑面执行更完整（33 号流改造的伴生效应），非回归。

## 失败簇归因（DC-3）

selection 面 13,756F 分列（互斥桶，簇合计与 selection 面 F 闭合）：

| # | 簇 | F | 归因 | 处置 |
|---|---|---:|---|---|
| C1 | Selection API 边界点校验族（addRange 5,104 / collapse 5,772 / extend 2,342 / stringifier 10 / root 散案余量） | 13,277 | spec 边界点有效性校验缺失：非法/畸形 Range（detached XmlText、comment、display:none、inverted 形态）应拒绝或不动 anchor/focus，shim 直接应用 | **超限挂账**（≫300）→ 待用户决策表 D2 立项建议 |
| C2 | `selection.modify()` 缺失（bidi 92 + contenteditable modify.tentative 36 + root modify-* 62） | 190 | API 未实现（`selection.modify is not a function`；非标准 API，tentative 面） | **≤300 且零新引擎概念** → M2 就地收敛候选（下一轮实现） |
| C3 | contenteditable 其余（initial-selection-on-focus ?div/?span 149F 等） | 153 | 焦点初选区/宿主边界语义 | 细拆后小簇随 M2 复核；大部分与 C1/C2 同根 |
| C4 | shadow-dom（getComposedRanges tentative 未实现 + cross-shadow setBaseAndExtent TypeError + 0 权重 reftest 页空跑） | 64 | tentative API + composed range 机制 = 新引擎概念域 | 挂账 |
| C5 | 拖选/键盘/布局命中（canvas-drag、drag-*、direction-on-click、move-by-word、caret-position-moveup 等） | 49 | runner Actions 坐标→文本偏移命中 + 真布局 rect（一期 anchor-removal 同族，现已部分修复） | runner 基建通道挂账 |
| C6 | caret 键盘移动（move-around-generated-content 族） | 18 | 键盘 caret 默认动作 + ::pseudo/生成内容 caret 位（`zw-pseudo` 形态期望） | runner 基建 + engine caret 管线挂账 |
| C7 | anonymous/details（summary 选择语义） | 3 | 匿名节点概念缺失 | 挂账（engine 概念域） |
| C8 | deleteFromDocument-HTMLDetails details 空壳 | 1 | **33 号流回归**（见锚面对照§） | 挂账待转登记 33 号 |
| — | textcontrols 锚面残 1F（focus.html，布局命中） | 1 | 同 C5 机理，桶属锚面族 | 随 C5 挂账 |

editing 面 2,360F 归 33 号域（本 goal 不展开）；fontSize ×3 回归见上。

## coverage 复测（DC-4）

工具 `wptfyi_gap.py coverage --results <本基线 --json> --cache
tests/wpt-runner/wptfyi-cache`（wpt.fyi aligned stable @ 三家并集，与
coverage-2026-10-31 快照同口径）：

- **selection universe：194 文件 / 34,421 子测试 → 覆盖 158 文件；
  未覆盖 = 36 文件 / 112 子测试**，分桶：
  - `idlharness.window.html`（112 子测试）——WebIDLParser/.idl build 资产
    不在语料，runner 记账排除（clipboard/timing/net-api/compression 四
    goal 先例）；
  - crashtests 35 文件（0 子测试权重）——skip 规则（uievents 先例）；
  - **plain/query-variant 未覆盖 = 0**——可执行面清零成立。
- 全部 23 个 `?query` variant universe 项由 variant meta 展开覆盖
  （含 pin 漂移 splitText 6 变体）；pin 漂移 8 文件全部映射入分子。
- 分母经 max(browser, ZW) 修正为 36,186（ZW 侧多跑出的生成腿入分母），
  selection 通过率 22,302/36,186 = **61.63%**（立项时 8.09%）。
- DC-4 结论：**未覆盖子测试清零路径成立**——残余 112 为 idlharness 记账
  排除（语料边界，非引擎缺口）；残余未覆盖文件均为 0 权重
  （crashtests）或记账排除（idlharness）。

## 与一期基线的一致性

一期终态 2968P/2F（2970 subtests）中 selection 锚面在本轮复现：
2,815 subtests 计数一致、Document-open 原样、textcontrols 全过；差异仅
上述 4F 回归（33 号流共享路径，已归因挂账）与 editing 面 +29 生成腿。
一期两处已修复案（toString rendered text、anchor-removal 拖选）保持通过
——**一期修复无回退**。
