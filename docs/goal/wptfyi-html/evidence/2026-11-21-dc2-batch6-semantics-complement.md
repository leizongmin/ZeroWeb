# DC-2 · 批六：html/semantics 补簇整批接入（语料 + 通道 + variant 解析修复 + 记账）

2026-11-21。批六落地：30 目录语料拉取（pin 315976933 git trees 子树显式枚举，
`GOAL_PULL_ANY_JS=1`，.headers sidecar 全量）+ html-syntax 通道加收 23 计分目录 +
`.https` 命名面簇内解锁 + 未引号 variant 解析修复。release runner 实测净
**+1,955 Pass**（html-syntax 65,314 → 67,269，801 → 1,049 键），共享键零回归
（801 键键集与逐键 Pass/Fail/Timeout/Unsupported/PreconditionFailed 与批五账
逐字一致；解析器修复前后对照零漂移）。

## 变更（三文件）

1. `tests/wpt-runner/scripts/goals/59-wptfyi-html-semantics-complement.sh`——
   30 目录显式拉取（10 顶层簇 + 20 子目录；fetch_dir_html 只递归一层，
   resources/support 助资产只拉不扫；幂等标记防限流）。links 簇不拉不扫——
   html-semantics 套件第 9 扫描根已认领（fetch 25 五叶 + 该 goal 账本 links
   119P/20F/4T 在账），通道若收则周跑聚合金重复计账（首版误收 11 簇，2026-11-21
   复核剔除）。
2. `Makefile`——`fetch-wpt-html-semantics-complement` target，挂入
   `testharness-html-syntax` 依赖（fetch 幂等，套件跑前自动补拉）。
3. `tests/wpt-runner/src/testharness.rs`——`HTML_SYNTAX_CORPUS_SUBDIRS` 加收
   23 计分目录（9 顶层 + 14 计分子目录；selectors 顶层无独立计分键不列）；
   `html_syntax_case_skipped` `.https.html` 命名排除对 `html/semantics/` 簇解锁
   （runner 恒 `https://wpt.test`、isSecureContext 恒真，批四/批五同前提）；
   `case_variants` 未引号属性值解析修复（止于空白或 `>`——HTML Standard
   §13.2.5.6 attribute value (unquoted) state）；补簇内 `.https` 解锁与 idlharness
   命名门优先的单元断言。

## 首跑 → 修复 → 重跑

首跑（旧解析器）溢出 39 键：**19 键**未引号 variant `>` 错形（interestfor 10 +
popovers popover-focus-2 3 + the-button-element invalid-element-types 6——分母
同名无 `>` 键，根因 `<meta name=variant content=?method=hover>` 标签紧贴值结束，
旧空白切片把 `>` 带入键形）+ 3 键生成形（.window.js 源键，分母 .window.html 直映，
本就执行）+ 17 键真溢出（漂移/快照新增，见下）。`case_variants` 修复后重跑：
19 键全部归位对齐分母；共享键 801 键修复前后逐键零漂移（解析修复不影响既有键）。

## 结果（release runner，test-guard 包裹，全量套件）

| 面 | 变更前 | 变更后 |
|---|---:|---:|
| html-syntax 键 | 801 | 1,049（+248 补簇执行 + 溢出） |
| html-syntax Pass | 65,314 | 67,269（**+1,955**） |
| Fail | 6,766 | 10,384（+3,618） |
| Timeout | 28→121（批五后） | 138（+17） |
| Unsupported | 7 | 10（+3） |
| NotRun | 0 | 2（geolocation get-current-position 2 案） |
| PreconditionFailed | 1 | 1 |

- 通道分母（summary-chrome 4b26db8978）**428 键 6,444 子测试**（chrome 全过
  6,041）：popovers 127 键 3,918 最大单簇（chrome 过 3,902）+ the-button-element
  812 + selectors 413 + permission-element 382（131 键）+ disabled-elements 329
  + interestfor 294（57 键）+ menu 187 + sections 107 + edits/obsolete 2。
- 执行 **231 键**（den 侧 5,890 子测试），203 键子测试数与分母逐字对齐；
  28 键不齐挂账：部分执行中断 23（页早停，如 interestfor-css-properties
  1/82）+ chrome 零子测试 variant 记录 4 + 超跑 1（menu/tentative/checkable
  17/15）。
- 执行 + 溢出 248 键中 ZeroWeb 过 1,955——通道大头挂在 popovers（见战役图）。

## 溢出 17 键 28 子测试（执行超出分母，诚实记账不截留）

- **更名漂移 16 键**——pin 仍 `.tentative`、wpt.fyi 快照已稳定化去 tentative：
  permission-element camera 4 + microphone 4 + usermedia 8（分母侧镜像 16 键
  28 子测试挂未执行账）。
- **快照新增 1 键**——menu/tentative/menu-dialog-mode-focus.html（快照无此键）。

## 余面挂账（197 键未执行，554 子测试，逐键归因）

- **非 testharness 形态 110 键 0 子测试**——reftest/crash/optional 自述式页
  （分母零子测试，不影响子测试口径：interestfor pseudo-element 外观族、
  menu 元素默认样式族、popovers reftest 面、obsolete/crashtests 等）。
- **pin 缺失 56 键 212 子测试**——快照↔pin 漂移：interestfor
  command-invoker/pseudo-element 动态族、popovers declarative 新面等 pin
  315976933 无此文件（git trees 枚举为凭，随周跑另报）。
- **iframe 内容门 12 键 155 子测试**——popovers light-dismiss 族 8 键
  （popover-light-dismiss 33 子测试最大单面）+ install iframe/sandbox
  createElement 形 2 + button/input-type-popovertarget 等。与 master 卡点
  iframe 子文档管道同根因（批四 107 + 批五 39 + 本批 12 同门）。
- **idlharness 命名门 3 键 159 子测试**——interestfor 44 + menu tentative
  72 + geolocation idlharness.window 43。
- **更名漂移镜像 16 键 28 子测试**（见溢出侧）。

## Fail 战役图（3,618 Fail + 17 Timeout 集中度）

- **popovers 2,744 F+T across 66 键**——绝对大头（invoke/light-dismiss/
  anchor-position 位面）；后续刀位首选。
- the-button-element/command-and-commandfor 318 across 23 键（invokeaction
  语义面）
- interestfor 112 across 28 键 + selectors case-sensitivity 92 across 1 键
  （values.window 生成形）+ pseudo-classes 87 across 27 键
- disabled-elements 76 across 6 键 + sections headingoffset 69 across 2 键
  + menu tentative 59 across 18 键
- permission-element 61 across 38 键（install 18 / geolocation 16 / usermedia
  14 / 顶层 13）

[逐文件 JSON](2026-11-21-dc2-batch6-semantics-complement.json) ·
[批五证据](2026-11-21-dc2-batch5-browsers-window.md) ·
[批四证据](2026-11-20-dc2-batch4-webappapis-channel.md)
