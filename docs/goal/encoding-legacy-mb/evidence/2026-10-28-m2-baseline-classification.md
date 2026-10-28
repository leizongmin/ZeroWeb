# M2：legacy-mb 基线分类（部分语料先行）+ .headers 两形态复核

日期：2026-10-28。目标域：`encoding-legacy-mb` M2/DC-3。**口径：部分语料**
（136 html，chars 大文件缺，网络 degraded 卡点维持）——全量语料补全后按同法
增量复测收口 DC-3，本文件为基线数据源与分类方法锚点。

## 数据源

- JSON：`/tmp/ab-legacy-mb/m1/encoding-legacy-mb.json`（M1 对账链 HEAD 臂
  整跑产物，2026-10-28 15:48–16:23，通道 `zero-wpt-runner
  testharness-encoding-legacy-mb` 全量无 FILTER，exit 0，源码 906b99540 +
  docs f59bd9eb2）。该 JSON 同时是 M1 零回归对账的 after 臂产物。
- 语料盘点：`tests/wpt-runner/scripts/goals/31-encoding-legacy-mb.sh`
  `goals_inventory`（136 html：euc-jp 18 / iso-2022-jp 16 / shift_jis 33 /
  euc-kr 37 / gbk 1 / gb18030 1 / big5 30）。

## 基线分类（385 case / 360,431 子测试 / 360,431P / 0F / 0 超时）

通道 per-dir skip 规则下可跑面 = decode wrapper 族 + decode-errors 族；
chars 数据文件（`.headers` sidecar 的主要携带者）未到位不产 case；
encode-form 面按划界排除；helper/skip 面按规则排除。

| 族 | case 文件 | case（含 ?N variant） | 子测试 | 通过率 |
|---|---:|---:|---:|---:|
| japanese/euc-jp | 4 | 43 | 40,388 | 100% |
| japanese/iso-2022-jp | 3 | 17 | 14,666 | 100% |
| japanese/shift_jis | 9 | 65 | 59,154 | 100% |
| korean/euc-kr | 11 | 181 | 170,488 | 100% |
| tchinese/big5 | 7 | 79 | 75,755 | 100% |
| schinese/gbk · gb18030 | 0（仅 encoder，出界） | 0 | 0 | — |
| **合计** | **34** | **385** | **360,431** | **100%** |

形态拆分：decode-wrapper 380 case / 360,386 子测试；decode-errors 5 case /
45 子测试（euc-jp 11 + iso-2022-jp 8 + shift_jis 10 + euc-kr 8 + big5 8，
全部 Pass——错误字节序的替换字符/致命错误语义与 encoding_rs 同表）。

要点：

- **五族 wrapper/errors 面全绿**（360,431/360,431）。与 30 号 JS API 面
  「解码表 ~100%」预期一致；文档面 `<meta charset>` 声明 → prescan →
  encoding_rs 解码 → span 文本断言整链成立，含 iso-2022-jp 转义状态机
  （7,329×2 wrapper 变体全过）。
- euc-kr 11 个 case 文件 ×18 variant 是最大族（170,488 子测试）——17 个
  label 别名（cseuckr/iso-ir-149/ks_c_5601-1987/windows-949 等）全数识别，
  label 表面无缺口。
- big5 `-decode-extra` 3 variant（hkscs 扩展区之外的特殊映射）同全过。

## 与全量分母的关系

coverage-2026-10-27 对账口径全量分母 1,127,087 子测试（日/韩/繁三族）+
schinese 1,017。本基线 360,431 来自 34 个 wrapper/errors 文件——chars 大
文件（上游各 ~700KB/文件、分母主体）未到位，**数字不是全量口径**，语料
补全后直接增量（新文件产 case 计入分母，既有 34 文件结果应恒等，可作
复测锚点）。

## .headers 两形态占比复核（DC-3 第三项，销账）

全量 98 个 sidecar 逐一比对（CT charset vs 本体前 1024 字节 prescan
`<meta charset>`，与 `decode_document_bytes` 窗口一致）：

- **hdr==meta：98/98**——全部 sidecar 的 `Content-Type: text/html;
  charset=<X>` 与本体 `<meta charset="<X">` 声明逐字一致。
- 「有 headers 无 meta」：**0/98**。
- 「头/meta 声明不一致」：**0/98**。

结论：file 通道不消费 `.headers` sidecar 对当前全部 sidecar 化语料**行为
中性**（meta 声明即权威，与预检 14 案抽样结论一致，现升格为全量实证）。
挂账销账；若后续语料补全带入新 sidecar，按同法增量复核（脚本逻辑记录于
本节，99 行 python，比对规则如上）。

## wpt-suites.csv 回填

新增数据行（2026-10-28，encoding-legacy-mb，subtests 口径
360431/360431，status=active，evidence 指向本文件），note 注明部分语料
口径与全量分母关系。

## 后续（语料补全后）

1. 全量语料落位 → 通道重跑（恒等锚点：本文件 34 文件基线）→ 全量分类
   复测收口 DC-3。
2. coverage 复测（DC-4）：legacy-mb 四族分母计入、通过率如实回填
   `docs/compat/wptfyi/coverage-2026-10-27.md` 复核。
3. M3：失败簇（当前 0）或 form-encode 面甄别 + `_zwMBuildDeepEntry`
   details 裁决态挂账收口。

## 更正（2026-10-29）

本文件上方「chars 数据文件未到位不产 case」「chars 大文件未到位，数字
不是全量口径」为误记：29 个 chars 文件自预置起即在本地（上游 trees API
对账 missing=0），它们是无脚本 span fixture，本来就不产独立 case——其
数据经 decode wrapper iframe 全量断言，**本基线 360,431 即全量口径**。
详见 [2026-10-29-corpus-completeness-dc3.md](2026-10-29-corpus-completeness-dc3.md)。
「后续」第 1 项随之完成（DC-3 收口），第 2 项（DC-4 coverage 复测）为
唯一剩余项。原文保留不改写。

### 勘误二（2026-10-29，同日更正追记）

上表 tchinese 行 75,755 为 transcription 笔误——与自身合计 360,431 矛盾
（114,208+170,488+x=360,431 ⇒ x=75,735）。2026-10-29 当前 HEAD 全通道
重跑实测 75,735，并与 M1 时代整跑 JSON 逐案恒等（0 diff）。详见
[2026-10-29-corpus-completeness-dc3.md](2026-10-29-corpus-completeness-dc3.md)。
