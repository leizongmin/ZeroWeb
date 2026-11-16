# X37 双键归因普查复核 + assert_in_array 9 键收尾批量判读（读侧轮，2026-11-16）

零码面改动。两个读侧账本收口任务，数据源均为 ihe 系批日志（`.acceptance/dps/`）与
语料 pin 315976933（本 clone `tests/wpt-runner/wpt-data/editing/`）。

## 任务一：X37 双键归因普查（定刀序）

X37 = `subscript>formatblock>inserttext "foo[]bar"` 案的
`queryCommandState("subscript") after` 与 `queryCommandValue("formatblock") after`
双断言（[2026-11-08 knife11 普查](2026-11-08-knife11-v5fix2-mt-residual-census.md)
定谳签名 A 跨窗伪影除名）。本轮在 ihe6 批（bin 03bb9745 head 96fa8a10a）最新
log 上复核指纹与反证：

**指纹复核（ihe6-mt.log，mt 腿全 log 恰 2 Fail 全落本案）**：

- `?1001-2000` 窗：exec 三连（subscript/formatblock/inserttext return value）+
  `compare innerHTML` + `checks for modifications` + subscript qI/qS/qV before
  全 Pass——exec 在窗内真跑、树逐字合录音；
- `?2001-3000` 窗：本案断言块续行无 exec 行；Pass/Fail 分割线恰为「期望值是否
  编码 exec 后状态」——qV(subscript) after Pass（期望 "" 与未动树巧合一致）、
  qS(formatblock) after Pass（期望 false 与未动树一致）；两红行 qS(subscript)
  after（exp true got false）与 qV(formatblock) after（exp "div" got ""）均编码
  exec 后状态。签名 A 指纹逐条命中，与 2026-11-08 定谳逐位同形。

**ab 页反证（单发射全案顺序执行）**：i5p1-ab.log（2026-11-16 01:56，最新 ab 腿）
本案 23 子测试全 Pass（含两红行本体），c5p1-ab.log / rt9h-ab.log 同 23/23——
三批独立二进制反证链完整。

**判定**：artifact 除名维持，零引擎债，**不排刀**。mt 面（multitest 全分母
10092 断言）残红 2F 全数归因完毕：fixed 0 / artifact 2（X37 双键）/
handoff 0——DC-4 三分表 mt 面可收口（三分表即本节+上档，无未归因键）。

## 任务二：assert_in_array 9 键收尾批量判读

[2026-11-13 ihtml 机制普查](2026-11-13-ihtml-residual-mechanism-census.md)
散余桶：assert_in_array 归一化族 9 键（注释位 ×3 / 空 span ×3 / pre-line ws ×2 /
pre-CE-false ×1），修复优先级最低、建议收尾批量判读。本轮定谳全桶闭合。

**9 键精确定位**（inserthtml.js 数组期望案 = runner 侧 assert_in_array 唯一来源，
`tests.js` `runConformanceTest` compare innerHTML 臂 `Array.isArray(browserTest[2])`
分支；node eval 全 127 案扫描恰 9 案数组期望）：

| 语料 idx | 族 | setup / 载荷 | 翻绿批 |
|---|---|---|---|
| 103 | D 注释位 | `<p><br>{}</p>` + `<!--abc-->` | ihe2（D 族占位 br/元素边界刀） |
| 104 | D 注释位 | `<p><!--foo--><span><br></span>{}<!--bar--></p>` + `abc` | ihe2 |
| 107 | D 注释位 | `<p><span><!--foo--><br><!--bar--></span>{}</p>` + `<!--abc-->` | ihe2 |
| 123 | G pre-line ws | `<div style=white-space:pre-line>abc []\n</div>` + `X` | ihe1（G 族 ws-pre 壳刀；ihd1 基线 2 Fail 在档） |
| 124 | G pre-line ws | 同 setup + `X`,`Y` 双插 | ihe1 |
| 112 | E 空 span | `<div>a[]b</div>` + `<span></span>` | ihe6（E 空壳臂；ihe1–ihe5 FAIL 在档） |
| 113 | E 空 span | `<div>a[]c</div>` + `<span class=s1></span>b<span class=s2></span>` | ihe6 |
| 116 | F CE-false 邻接 | `<p><b><span contenteditable=false>abc</span>[Y]</b><i>def</i></p>` + `<i>Z</i>` | ihe6（F CE-false 邻接提升臂） |
| 111 | pre-CE-false | `<pre contenteditable="false"><span contenteditable>[1234]</span></pre>` + `<pre>abc</pre>` | ihe6（CEF pre 臂） |

**批量判读（ihe6-iht-a.log / ihe6-iht-b.log 双腿各 1565P/0F）**：

1. 9 键 compare innerHTML 行双腿全 Pass，每腿恰 1 行（按 tests.js 命名规则
   JSON 复配 test name 逐键定位，非子串近似）；
2. **无一 vacuous pass**：9 案接受形数组均**不含** NOOP 形（setup 去标记串逐案
   对照，如 idx 112 接受形 `a<span></span>b` vs NOOP `ab`、idx 123 接受形
   `abcX\n`/`abcX \n` vs NOOP `abc \n`、idx 111 接受形 `abc` 入壳 vs NOOP
   `1234` 残留）——Pass 即载荷真落位，2026-11-13 普查「多数亦为 NOOP 变体」
   的疑似空过风险**证伪**，翻绿全部实质；
3. 判读维 护：数组期望为 Chrome 录音逐位形（注释/空白/壳序多接受形），引擎落位
   均在数组内，无越形旁路。

**判定**：9 键全 fixed（族刀扫清：D 族刀 3 + G 族刀 2 + ihe6 三臂 4），零 artifact
零 handoff。散余桶闭合，inserthtml.html 面账目（census 0F）自洽。

## 对账影响

- mt 三分表：X37 ×2 artifact 维持（证据本轮刷新至 ihe6/i5p1 批），mt 面收口；
- assert_in_array 桶：闭合，不再列为下一步；
- 下一步簇序（按残键数降序、可动簇）：ilb 33F 面 → indent 27F 八微族 →
  outdent 残 4 → ip I4 2（域敏感缓）；exec1 474 域决策挂账维持；fb apply 侧
  ~32 族 + img 23F + fontSize T1 ×3 待接收在账。
