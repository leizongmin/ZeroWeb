# ⑪ 轮 evidence：fix10 残差切片伪影审计定谳——156F = 135 真实 + 21 切片伪影，② B/D/E 位置基假设证否

日期：2026-11-05。对 ⑩ 轮六语料残差（五语料 31F + justifycenter 125F = 156F）逐行审计。判决：**135 行真实引擎缺口 + 21 行 variant 切片伪影**；控制面 ②「B/D/E 族 qV/qI-after 位置基（apply 后选区重锚落位 / `_zwValGetAllEffectivelyContained` containment 语义）」假设证否——伪影族无一涉及引擎行为，本轮零源码修改。

## 1. 切片机制定谳（本轮新发现，修正 ⑨/⑩ 轮「?9001-last 截断带」的部分口径）

run 壳页 variant 窗口（`?1-1000` 等）按**逐断言子测试**切（tests.js `runConformanceTest` 每个 row 一个 `subsetTest(test, cb, name)`，全局递增编号）：

1. 每片独立跑：片内每个 case 重做 setup（container.innerHTML 整换 + range 落位 + before 查询），但只执行窗内编号的断言；
2. case 的 16 断言（2 exec rc + 1 checks + 1 compare + 12 查询）编号连续、跨窗边界可截断——exec 子测试落窗外则 **exec 不执行**，窗内 after 读数反映未修改状态，对假定 exec 已跑的 Chrome 录音期望必红（切片伪影签名 A：straddle）；
3. 跨 case 状态泄漏（styleWithCSS mode 滞后一拍）在 Chrome 录音（顺序全文件跑）中存在，切片重启后丢失——qS(stylewithcss) before 录音 true 的行在片内无泄漏源 exec 时必红（切片伪影签名 B：leak-cut）。

伪影判定双门（audit v4 口径）：签名 A = 本片该 case 组无 execCommand 行 + 失败行为 POST 段（after/exec rc/checks/compare）；签名 B = qS-before expected true got false + 本片失败行之前无 `execCommand("stylewithcss", false, "true")` 行。复核：每伪影 case 在邻片 exec=2 时 fails=0 且两片行数拼回全案 16（或 23）。

## 2. 证据链（三重独立）

1. **邻片对照**（fix10-*.log 审计）：6 个签名 A 案全部「本片 exec=0 fails=N / 邻片 exec=2 fails=0」——fontsize B-false（?1001-2000 rows=7 exec=2 全绿 + ?2001-last rows=9 exec=0 fails=2）、fontsize/forecolor table qV-after（12+4）、fontname pre（11+5）/listing（15+1）、forecolor brown sw=true（9+7）。
2. **fs-win 变体窗复跑**（zz-fs-win.html = fontsize.html 拷贝 + 追加 `?1001-2130`/`?2051-2130`/`?2096-2130` meta，/tmp/qv-fix4/probe-vsel/）：`?1001-2130`（B 案全窗内）双变体全绿；`?2001-last` B-false 仅 9 行窗内（exec 段缺失）→ qI-after=true / qV-after="2"（未修改状态读数）红。窗内即绿、窗外 exec 即红，伪影直接钉死。
3. **探针钉测**（zz-probe-vsel.html，入库为常驻回归见证，16 Pass/0 Fail rc=0）：语料序复制（before 查询序 stylewithcss→fontsize、after 同序）下 exec 真跑时引擎读数逐案命中语料期望——B fixture `foo<font size=2>ba[r</font>b]az` 双变体 qI-after=false / qV-after="4"（语料红行期望值），rc true，qS-after=本案自设 mode。引擎无位置基缺陷。

签名 B 复核：5 个 qS-before 红行（forecolor ?2001-last ×1 + justifycenter ×4，全 expected true got false）各自片内失败行前无任何 sw=true exec PASS 行（fail@2044/1066/2093/5151/6179 处向前扫 ABSENT）。

## 3. 21 伪影清单（全部非引擎缺口）

| 语料 | 行数 | 明细 |
|---|---|---|
| fontsize | 3 | table qV-after ×1（?1001-2000 straddle）；B-false qI/qV-after ×2（?2001-last straddle） |
| fontname | 3 | pre qI/qV-after ×2（?1001-2000）；listing qV-after ×1（?2001-last）——**fontname 残差 3F 全伪影，真实 0** |
| forecolor | 4 | table qV-after ×1；brown sw=true qI/qV-after ×2（?2001-last straddle）；brown sw=false qS-before ×1（?2001-last leak-cut） |
| hilitecolor | 0 | 全文件跑（壳页无切片），无伪影 |
| justifycenter | 11 | after 段 straddle ×7（?2001-3000 ×3、?3001-4000 ×2、?4001-5000 ×2）；qS-before leak-cut ×4（?1001-2000、?2001-3000、?5001-6000、?6001-last） |
| insertunorderedlist | 2 | dps-div straddle qInd+qState after 双行（?1001-2000；exec #990-991 落 ?1-1000 窗）——2026-11-06 eb1 轮补审定谳（三重门 + fs-win `?985-1010` 双行 PASS，见 [evidence](2026-11-06-empty-body-eb1.md)）；iol/iul 语料首次纳入本清单 |

## 4. 135 真实残差重记账（② 解散 → 族谱收敛）

| 族 | 行数 | 明细 |
|---|---|---|
| **载体形/prop-carry（③ 主刀）** | 13 | fontsize teal `<span style="font-size:32px; background-color:rgb(0, 128, 128)">` `[a]bc`/`a[b]c` compare ×2；forecolor 嵌套/跨元素 compare ×7（blue/brown 嵌套 sw=false ×1、font 形 blue/brown sw=true ×1、rgb/brown sw=false ×1、rgb/purple `[a]` sw=true/false ×2、brown/#0000ff 相邻 sw=true/false ×2）；hilitecolor 全选区载体形 ×4（`<font size=6>[foo]</font>` / `<span style=font-size:xx-large>[foo]</span>` 双形 ×2 mode） |
| **读数面（④）** | 8 | forecolor purple qI/qV-before ×4 + rgb/purple qV-before ×2（exec=Y 组，真读数缺口）；fontsize br qI-after ×2（`<p>[foo<p><br><p>bar]` fontsize=4，?1-1000 全案在窗） |
| **dps/jc 族（justifycenter 真实）** | 114 | 125F − 11 伪影 |
| 合计 | **135** | 五语料 22 + justifycenter 114 |

③ 载体形 got/expected 实证对（fontsize teal `[a]bc`，?2001-last，exec=Y）：got `<font size="5">a</font><span style="font-size:32px; background-color:rgb(0, 128, 128)">bc</span>`——新 `<font>` 壳 prop-strip 丢背景 carry；expected 双形 `<span style="background-color:rgb(0, 128, 128)"><font size="5">a</font></span>…` 或 `<font size="5"><span style="background-color:rgb(0, 128, 128)">a</span></font>…`。`[abc]` 全选 / `ab[c]` 尾段 / 跨段案全绿 → 缺口仅在**部分选中含样式壳的中间/头段切分**：tag 模式 wrap 需把被切分 span 其余 props 带入新壳（`_zwVaSplitConflictingInline` peel / `_zwVaRematerializeCarries` 的 tag-mode 面）。

## 5. 工件

- 审计判读：/tmp/qv-fix4/audit-straddle-v4.py（转义安全 fixture 解析 + 每变体分组 + 逐行签名 A/B 分类 + 邻片复核）；fix10-{fontsize,fontname,forecolor,hilitecolor,justifycenter}.log
- 变体窗复跑：/tmp/qv-fix4/probe-vsel/fs-win.log + zz-fs-win.html（语料拷贝不入库）；探针 r4-probe-witness.log（16 Pass rc=0）
- 探针入库：tests/wpt-runner/wpt-data/editing/run/zz-probe-vsel.html（v2 删 dump 强红断言后收编：qI/qV/swS/rc 钉测 16 断言全绿常驻）
