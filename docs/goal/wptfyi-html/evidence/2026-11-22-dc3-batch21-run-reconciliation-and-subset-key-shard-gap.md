# DC-3 · 批二十一：批十八 run 45017894086 收账＋subset-tests-by-key 分片缺档定谳与补档

2026-11-22。run 45017894086（4fb8af586，批十八语料＋批十九 dom.idl 修复）
五 html 分片产物逐键 A/B（批十七 run 45008050513 同分片产物）；两处语料
缺档修复随文落码，随下轮 run 兑现。

## 批十八 run 逐通道收账（45008050513 → 45017894086）

| 通道 | 批十七 | 批十八 run | Δ | 判定 |
|---|---:|---:|---:|---|
| html/dom idlharness 三查询键 | 0P×3（各 1 setup Fail） | 1,493P×3（各 6,407 注册） | +4,479 | dom.idl 兑现＝但三键全页未过滤（本轮主项，见下） |
| html/semantics scripting-1 14 键 | 0（语料未在 CI） | 230P | +230 | 批十八兑现（−4＝non-object 键 4 json 缺档，第 6 例已修） |
| html/semantics 其余 | 7,773 | 7,774 | +1 | checkable-active-onblur-with-click 3→4（flaky 族回位） |
| html/webappapis | 1,537 | 1,537 | 0 | 持位 |
| html/syntax | 2,790 | 2,790 | 0 | 持位（dat 修复 83c293e8d 未在此 run，下轮兑现） |
| html/infrastructure | 674 | 674 | 0 | 持位零漂移（messagechannel 115 持） |
| html/rendering | 2,795 | 2,795 | 0 | 持位零漂移 |
| html/speculative | 43P/83TO | 43P/83TO | 0 | 持位 |
| import-maps | 37 | 37 | 0 | 持位 |
| html/the-xhtml-syntax | 25,606 | 25,606 | 0 | 持位 |
| shadow-dom（CE·SD 邻账） | 11,620 | 11,686 | +66 | 跨流正向位移：0df33127f RT 切片 reference-target 四键（property-reflection-idl-setters/imperative-setup 0→30×2、basics 0→3、idl-reflection 3→5）＋opt-in +1，入 dom 账本邻账 |

目标级账（aggregate 快照 c7725c4de，13:23Z 落 main）：html 面
107,392 → **112,102（净 +41,006，验收 ≥152,096 余 39,994）**＝dom
59,634→64,113（+4,479）＋semantics 10,464→10,695（+231＝230+1）＋其余
十一组（browsers/canvas/editing/infra/rendering/syntax/xhtml/webappapis
等）逐组持位；全量 1,918,680→1,923,901（+5,221，含对端流面另账）。
CE·SD 组持位（546/627）——shadow-dom 套件 +66P 的 reference-target 四键
均 tentative 形，不入 wpt.fyi aligned face（邻账不虚计）。

### dom 三查询键 +4,479：兑现但含口径虚增（本轮主项）

三键各注册 6,407 子测试 / 1,493P，**三键子测试集逐字节相同**——
`?include=(Document|Window)` 键含 HTMLAnchorElement 全套断言＝include/exclude
过滤未生效，每键都跑了全页（真过滤形＝三键不相交，合计 ~5,822 注册
/~1,477P，见本地探针）。face 兑现 +4,479 中 ~+3,002 为两把变体键的
全页重复计入。

**根因＝分片语料隔离律第 7 例**：页面过滤助手
`<script src=/common/subset-tests-by-key.js>`（读 location.search 收敛
idl_test 注册集）只有 62 脚本（infra 通道，批十四补拉）在 fetch 清单——
syntax-dom 分片 fetch 步（40/57/58/59/61/dom 六目标）恒缺 → CI 页面无
过滤助手 → idl_test 照常全页跑（不报错、不 TO，静默虚增）。
本地在盘（62 脚本批十四本地实跑带入）＝本地三键过滤生效，批十六本地
定向测量 5,822/1,397P（923/48/426）与当前 HEAD 探针复现值
（5,822 注册/1,477P＝48/1,003/426，键间交集仅 setup 2 项）同源同形——
「本地真 CI 假」第 3 演（第 4 例 dom.idl、第 5 例 html5lib dat 同族）。
本地超 CI 的 585 注册差＝本地 30s 案预算截断（CI syntax-dom 90s 页预算
跑完全页未截断）。

**修复**：40 脚本（html/dom 语料 owner）补拉
`fetch_raw "common/subset-tests-by-key.js"`（+7 行含注释）。验证按批十七
教训①口径：`bash -n` 通过；挪走本地文件后实跑 `make fetch-wpt-html-syntax`
exit=0、文件重拉回且与原文件 `cmp` 字节级相同。
**下轮 run 兑现后 face 回吐 ~−3,002**（三键回归真过滤形）——记账纠偏，
非增益；同轮 non-object 修复 +4、html5lib dat +118。

### non-object 键 −4：4 支撑 json 缺档（隔离律第 6 例）

`json-module/non-object.any.js` CI 1P/4F vs 本地 5P/0F：4 个 Fail 全为
`TypeError: Failed to fetch module './null.json'` 等——源文件自身静态
import 的 4 个支撑 json（null/true/false/string.json）不在 59 脚本
fetch_raw 清单（批十八「json-module 5 json」= bom×3＋array＋module，漏
源文件 import 闭包；支撑闭包枚举口径勘误）。修复：59 脚本 +8 行
（4 fetch_raw＋注释）。验证同教训①口径（挪走重拉 byte-identical）。

### 邻居观察（不入本流账）

- dom/ranges 两键 +394P（Range-mutations-dataChange 738→930、
  replaceData 944→1146，批十七 Timeout 截断形回位扩形）＝对端渲染流
  engine DC-2 切片 2（ec4ef28dc）正向位移；切片 3（5e217bdba，
  dataChange 188F→0F）在本 run 分片 checkout 之后落 main，下轮 run
  兑现，归 wptfyi-dom 账本。
- shadow-dom +66P＝0df33127f RT 切片（同上跨流），CE·SD 邻账。
- editing-run、css 面随全量另账。

## 下轮 run 承载清单

dat 补拉（83c293e8d）＋non-object 4 json＋subset-tests-by-key.js；预期
html 面 112,102 − 3,002（过滤纠偏）+ 4（non-object）+ 118（dat）≈
**109,222**（±90s 页预算下过滤形截断余量）。engine 切片 2/3/4 对 dom/
面的正向位移（RT 三矩阵、CharacterData 边界、sanitizer/trusted-types
通道 +70P）随 wptfyi-dom 账本另计，不入 html 面。

[逐文件 JSON](2026-11-22-dc3-batch21-run-reconciliation-and-subset-key-shard-gap.json) ·
[批十九证据](2026-11-22-dc3-batch19-batch17-run-reconciliation-and-domidl-shard-gap.md) ·
[批十八证据](2026-11-22-dc3-batch18-scripting1-module-anyjs-corpus.md) ·
[前置账](2026-11-22-dc3-goal-accounting-and-corpus-runway.md)
