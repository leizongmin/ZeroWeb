# outdent 实应用刀收口（knife v1.2，2026-11-11）

## 目的与刀体

census v1 权威数：outdent.html 2377P/**173F**/0T = editing/run 最大残簇；归因定谳 **171 compare 行全 got==输入树（outdent 命令整体 no-op）**——引擎无 outdent 实应用臂（part06 旧注在案）。刀体 b49bfe19e（v1 发射前已推送）：part01g `outdentNode`（vendored implementation.js :5520-5692 忠实移植，helper 全复用）+ `__zw_ce_outdent_apply` action（:8209-8311）+ part06 apply 臂/preservesOverrides 接线 + 常驻探针 zz-probe-outdent.html（语料真条目 16 形）。

## v1/v1.1 两发 ABORT 定谳（判读规格错与探针规格错，非行为漂移）

- **v1 首射（12:30:34）**：锚门拦截锚规格自误——S1（`__zw_ce_outdent_apply` 定义行）期望 ≥2 实 1、S3（`cmd === 'outdent'`）期望 1 实 2（cmdPresOv 行＋apply 臂）。教训四度实证（tq v4 同族）：锚写死前先 grep 现树实值。v1.1 修锚（S1=1/S3=2）同门重射。
- **v1.1 L0 探针红（12:33:08，P=0 F=1 `PICKS-FOUND`）两根因**：
  1. **探针页缺 `/common/subset-tests.js` include**——corpus `runConformanceTest` 经 `subsetTest(test, fn, name)` 注册子测试，脚本缺载即首个条目 ReferenceError 杀断 IIFE，**16 语料条目从未执行**（v1.1 失败日志仅 1 行＝PICKS-FOUND 本体佐证）。若两 pick 恰好全中，L0 将以 P=1 空转假绿放行——`P>0` 形门对探针页是假绿陷阱，精确计数/键集门才是有效门（F2P=171 主门为二级防线，未走到）。
  2. **两 pick 语料失配**：webkit-indent-blockquote 族条目全带 `stylewithcss`+`defaultparagraphseparator` 前导 setup（无裸 outdent 条目）；`<ol><li><h1>[foo]</h1></ol>` 语料 setup 为裸 outdent（探针误写 DD）。

## 探针修复（38427da6e，已推送）

补 `/common/subset-tests.js` include＋两 pick 对齐语料实值。preflight（odt-probe-preflight2.log）实测 **P=215 F=7**，7 残红定性均刀域外：5×`qV("defaultparagraphseparator") before`（录音会话状态链在探针序下的伪影，同 census dPS 挂账族）＋2×compare innerHTML（见下节 CSSOM 定谳）。

## CSSOM style 写回 no-op 定谳（诊断探针 zz-diag-normstyle，已删不入库）

两 compare 残红形状：expected `<div style="color:rgb(0, 0, 255)">`（语料录制时管线预归一）vs got `<div style="color: blue">`（字面量原样）。harness `normalizeSerializedStyle`（tests.js :5499，exec 后同步跑）应把 got 归一到 rgb 形；其颜色臂依赖 CSSOM 写回（`node.style.color = newColor` 须回写 style attr）。诊断探针五级断言：①`qSA("[style]")` 命中 ✓ ②`style.color` 读 "blue" ✓ ③`normalizeColor` → rgb ✓ ④**CSSOM 写回 getAttribute 不变 ✗**（`style.color=""`＋`=rgb` 后 attr 仍 `color: blue`）⑤全管线 ✗。定谳：**引擎 CSSOM 属性 setter 不回写 style attribute**，属引擎读/序列化面独立缺口（boolface 面旧绿系引擎自身输出已 rgb 形，未证 harness 归一管线）。残红 8 行全此族。

## v1.2 门值定谳与批程

批脚本 batch-outdent-knife-v1.2.sh @**38427da6e**（探针修复 commit，已推送；rebase 并入他流 431d229e5 anchor 族七属性批——touch part01c/part03/part06，刀体特征串锚复读 S1/S2/S3=1/2/2 恒一，editing 语料无 anchor 声明）。门值全部由 preflight 实测定谳后嵌入（精确计数＋恰集键清单，弃 `P>0` 形）：

- **L0 probe：P=215 F=7 恰集**（批跑 13:35 ✓ bin c1b81788）
- **L1 outdent 页键 judge vs census（ind-c1）：F2P=161 / P2F=0 / lost=0 / 残红恰 12**（批跑 ✓ rows 2550→2550）
- **L2-L6 全绿，ALL-DONE 14:26:49 @38427da6e**：L2 indent 1302/27 严格零漂移；L3 mt **20182/2 恰值＋judge 四零**（F2P/P2F/post-only/lost 全 0）VERDICT OK；L4 singles×5 恰值（iol 1759/iul 2061/ip 6989/ilb 1658/ihtml 1446）；L5 exec×3 VERDICT OK；L6 enginelib **3864P/7F** failed==7 名册（3864 vs tq 代 3846＝+18 系 431d229e5 anchor 批新增测试，门只锁 failed 名册）。二进制 md5 全腿恒一 c1b81788（batch-outdent-knife-v1.2.log）。

**判读方法教训（FILTER 子串页拉入）**：`FILTER=outdent.html` 子串匹配会同时拉入 `zz-probe-outdent.html`（批 L1 日志 P=2753=语料 2538＋探针 215）。v1.1 版 judge 键剥页名，probe 行覆盖语料行造成 **P2F=5＋new-row 1 判读伪象**（preflight 首次 judge 即现）；v1.2 改页键判读（`run/outdent.html?` 过滤）后 P2F=0 定谳。后续加探针页须：①judge 按页键过滤；②或探针命名避开语料页名子串。

## 残红 12 行三族分类（173 = 161 F2P ＋ 12 残红，对账闭合）

| 族 | 数 | 形 | 定性 |
|---|---|---|---|
| CSSOM style 写回 | 8 | bq `style="color: blue"` ×4（dPS div/p × 形）、`<ol style=color:blue>` ×2、`<li style=color:blue>` ×2 | 引擎 CSSOM setter 不回写 attr（已定谳），独立挂账刀候选 |
| dPS-qV-before | 2 | `<blockquote><ol>…` / `<ol><li>[foo]<ol>…`（dPS div 形） | census 既有 dPS 读面挂账，刀中性 |
| ws-bq spec-vs-Blink | 2 | `<blockquote> <p>[foo]</p> </blockquote>` / `<blockquote><p>[foo]</p> </blockquote>`（裸 outdent） | got＝尾随空白被 walk indent 阶段包成空 `<blockquote> </blockquote>`；expected（Blink 原生录音）＝ws 移出 bq 全除。参考实现 indentNodes 无 ws-only 特例＝书面 spec 与 Blink 录音分歧，收口需刻意 Blink 兼容偏离臂，独立刀候选 |

## 判读与下一簇次序

1. outdent 173F 簇收口定谳：**实应用刀收 161**（73.4%→96.6% 通过率：2018→2179P），残 12 全数三族归类零未归因。
2. 下一刀候选序：**CSSOM style 写回刀**（残 8＋indent 页 style 族复用面，修后 probe/oracle 面广）> ws-bq Blink 偏离臂（残 2，小而清晰）> indent 27F 八微族（census 在档）> dPS 读面挂账维持不追。
3. 语料/产物：odt-probe.log / odt-outdent-post.log / odt-outdent-fails.txt / odt-indent-post.log / odt-mt-post.log / odt-{iol,iul,ip,ilb,ihtml,exec1..3,enginelib}.log（.acceptance/dps/）。
