# 语料完备性更正与 DC-3 收口

日期：2026-10-29。触发：本轮语料续拉改走代理（直连 raw 持续超时；代理 10801
在线，raw 经代理 0.7s 200），拉取静默跑完后文件数不变（136 html）——追查
发现**上游 pin 本身只有这些文件**，多轮「语料卡点」为误记。

## 语料完备性实证（上游 trees API vs 本地，pin 315976933）

- 四族 `git/trees?recursive=1` 清单：japanese 127 / korean 66 / schinese 5 /
  tchinese 55 = **253 文件**，本地同 253（另有 1 个中断下载的 `.html.tmp`
  孤儿已清）——**missing = 0**。
- 29 个 `*-chars*.html`（各 ~852KB）**全部在位**；`<script>` 标签数 = 0——
  纯 span fixture（iframe 子文档，由 decode wrapper 经 iframe 消费，通道
  skip 规则如实）。`*_errors.html` fixture ×5 同（无脚本）。
- **「chars 大文件缺」为误记**：chars 文件自 30 号预置起即在本地，且被
  wrapper 全量消费（每 variant 对 chars 全量 span 断言，M1 记账 ~3–5 万次
  桥调用/variant）——M2 基线 360,431 子测试就是**全量 chars 数据**的断言面，
  不是部分语料。
- 六轮 FORCE=1 重试的 curl 报错是真实网络 degraded（jsdelivr 拒连、raw
  超时），但重拉的是**已在位文件**（FORCE=1 全量重下语义）——没有任何
  文件因网络缺失过。

## 136 html 面普查（与通道规则一一对应）

| 面 | 文件数 | 通道处置 |
|---|---:|---|
| decode wrapper（`-decode.html`/`-decode-*`，含 `-decode-errors` ×5） | 34 | 跑（M2 基线 385 case / 360,431 子测试 100%P） |
| chars fixture（`*chars*.html`，无脚本） | 29 | 不产 case——数据经 wrapper iframe 全量断言 |
| errors fixture（`*_errors.html`，无脚本） | 5 | 同上（`-decode-errors` wrapper 消费） |
| encode 面（form 46 + href 19 + `*-encoder.html` 2 + `big5-enc-ascii` 1） | 68 | 甄别 not-run（见 M3 甄别 evidence，归口非本 goal） |
| **合计** | **136** | |

## 分母对账（wpt.fyi 口径分解，DC-3 收口依据）

coverage-2026-10-27 分母 = wpt.fyi aligned stable 三家浏览器实报子测试
并集。**无脚本文档不产子测试**——29 chars fixture 在浏览器侧同样是 0 子
测试，「分母主体是 chars 大文件」同样不成立。四族分母 1,128,104 分解：

| 族 | wpt.fyi 分母 | ZW 实跑（decode 面） | encode 面差额（甄别 not-run） |
|---|---:|---:|---:|
| japanese | 447,723 | 114,208 | 333,515（35 文件，均 ~9.5k） |
| korean | 410,448 | 170,488 | 239,960（15 文件，均 ~16k） |
| tchinese | 268,916 | 75,735 | 193,181（15 文件，均 ~12.9k） |
| schinese | 1,017 | 0（无 decode wrapper，上游同） | 1,017（2 `*-encoder.html` inline href 面） |
| **合计** | **1,128,104** | **360,431（100%）** | **767,673** |

即：**可跑面（decode wrapper+errors）已 100% 全量收敛；分母余量全部是
encode 提交/序列化面**，已按划界甄别归口（2026-10-28 evidence），不属本
goal document 解码面。

## 锚点复验（2026-10-29，当前 HEAD）

M2 数字 transcription 勘误：M2 表 tchinese 行记 75,755，与其自身合计
360,431 矛盾（三族和应为 75,735）——本轮全通道重跑（38.9MB `--json`，
test-guard 同 envelope）实测 **75,735**，且与 M1 时代整跑 JSON
（`/tmp/ab-legacy-mb/m1/encoding-legacy-mb.json`，M2 基线数据源）逐案逐
子测试名集合恒等：385 case / 360,431 子测试 / 0 非 Pass，case 集与子测试
名 diff 双向为空。M2 以来全部 engine 变更（R4100 系 + M3 details 两面 +
并行流合入）对该通道零影响，恒等锚点成立。逐案紧凑 JSON 落
[evidence/2026-10-29-m2-full-baseline-summary.json](2026-10-29-m2-full-baseline-summary.json)。

## 结论与销账

- **DC-3 收口**：全量（pin 口径）分类通过率落账——可跑面 360,431/360,431
  = 100%；encode 面 68 文件甄别 not-run 归口；fixture 面不产 case（与
  上游浏览器口径一致）。wpt-suites.csv 数据行 note 同步更正。
- **「语料补全卡点」销账**：goal 头部/M2/M3 的网络卡点表述更正；fetch
  基建本身已通（代理路径可用，幂等重拉全绿），后续重拉无需再试直连。
- M2 evidence「chars 数据文件未到位」表述由本文件更正（原文保留不改写）。
- M3 甄别 evidence 语料盘点更正：encode 面合计 **68** 文件（原记 65，漏
  `*-encoder.html` ×2 与 `big5-enc-ascii.html`——`grep -l` 按文件名模式
  匹配漏掉非 `encode-href`/`encode-form` 命名的同面文件），结论不变。

## DC-4 待办（唯一剩余项）

coverage 复测 = `wptfyi_gap.py coverage --results <47 套件 --json 结果目录>`
全量重跑（2026-10-27 的结果目录未持久化，需重跑各通道，数小时量级）→
新日期报告。legacy-mb 行预期：分母 1,128,104 计入、分子 360,431（decode
面）、通过率如实（~31.9%，差额全部是甄别归口的 encode 面——face 归因
由本文件与 M3 甄别 evidence 承担）。

## DC-4 执行（2026-10-29 夜启动）

批量驱动仓库外与仓库同级的 `zw-coverage-dc4/run-dc4.sh`（运行目录，不入
git）：51 个 testharness 通道按 Makefile 各 target 的 env/guard envelope
逐通道顺序重跑（fetch 依赖跳过——语料已在盘），`--json` 落
该运行目录下 `results/<channel>.json`，
`status.tsv` 记 rc+字节数，`driver.log` 记起止。testharness-css-variables
按分片器同规则改为逐案 `--json` 合并（其 Makefile wrapper 不透传 JSON）。
首通道（testharness-html，7 case 小语料）已验证驱动机制正常。对账管道已
用前 3 通道结果干跑验证（临时副本，代理取 wpt.fyi aligned stable）：全量
口径 1,254,420 文件 / 2,266,562 子测试正常到账，工具返回的 legacy-mb 三族
分母行 447,723 / 410,448 / 268,916 与上表逐字一致；rc=1（有失败）通道的
JSON 同样完整可解析（runner 仅 rc=124 超时截断时 0 字节）——故收数门禁为
status.tsv 全行 rc=0 且 results/*.json 全部可解析（重试链负责补齐）。
下轮：
核对 status.tsv 全 rc=0 → `wptfyi_gap.py coverage --results <该目录> --out
<新日期报告>` → 核对 legacy-mb 行（分母 1,128,104 / 分子 360,431）与
wpt-suites.csv/hero 数字联动，DC-4 收口。
