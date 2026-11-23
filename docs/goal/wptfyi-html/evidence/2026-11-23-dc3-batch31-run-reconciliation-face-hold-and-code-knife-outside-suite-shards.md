# DC-3 · 批三十一：run 45169842737 收账——face 持位零变化（109,425）＝窗内真码刀 a3932b528 落在全部套件分片起跑之后（仅 aggregate 捡到＝套件面零码归因维持）＋css 带精确镜像回位＋editing 上缘外穿＋dom 下缘再穿＋egress 503 逐件韧形下载课＋cache-key 误抓第三演

2026-11-23。run 45169842737（11:57:47Z dispatch，~13:55Z 完成
13:57:53Z 读数 success，快照 coverage-2026-11-23 同日覆写＝
13:57:11Z 生成落 840aacea5，manifest 恒 063eb7cae2 零漂移第四窗）
对批三十 run 45145268804 同分片逐键 A/B。布防全绿一窗：等待窗
200×120s 未击穿、干净版脚本子目录名 run31 正确（批二十九/三十两课
落地首演）、waiter poll25 正常捕获（网络抖动只伤 gh api jobs 查询
一次 502 重试即愈）。下载窗遇新形：`gh run download` 整包式先在
zw-results-test262-language 连接重置、重试再遇 **HTTP 503 egress
over the account limit**（账号级带宽滚动窗，多流并行下载所致）＝
整包式无断点续传、一次失败全弃。改逐 artifact 韧形下载器（已提取
跳过＋45s 退避＋8 轮循环）后 167/167 收齐（首轮 54/55，次轮补齐
test262-language）。

## 分片 checkout 逐 job 实测＝双 SHA 窗、码刀在套件分片之外

**55 提取＝44×b8b630f7e（43 直取＋1 补提取）＋8×e4bdd0ecd＋
2×5ce4b517c（后两者 docs-only 领先，祖先链互验全过＝码图同
b8b630f7e）＋1×79ceeb3b8d（aggregate 分片）＋1 误抓 b8a594ea5**
（fallback grep 又抓 Cargo.lock cache key＝批二十八课**第三演**，
对象存在性检查当场捕获，对 job log 定向 grep `git log -1` 上下文
补提取真身 b8b630f7e）。**窗内真码刀 a3932b528**（script-sandbox
TT 跳板 rquickjs 0.7 API 编译适配，13:35:32Z 落树）：54 套件分片
最晚起跑 13:35:31Z 全部早于它＝**码刀在全部套件分片之外**；唯
aggregate 分片 13:56:55Z 起跑捡到（checkout=79ceeb3b8d）——
aggregate 只聚合产物不跑套件＝**套件面零码归因维持**（码变区间
b8b630f7e..79ceeb3b8d 含 a3932b528＋其余全 docs/chore(perf) 数据
笔）。**批三十二预布**：下窗全部分片将携带 a3932b528 码图＝TT 面
（tt 套件/test262 ③③b collateral/CE·SD）首个码归因读数到期。

## 目标级账＝face 持位零变化

html 面 **109,425 → 109,425（0）＝净 +38,329/基线 71,096，验收
≥152,096 余 42,671**。testharness-html.json 不在变化套件清单＝face
零变化；track-cues 第九演＝media 套件零变化＝b30 Fail 持位（带形
F P F P F F P F **F**＝两连 F，仅记不追）。

## A/B 总账与组行闭合

总 P **+298**（1,945,556→1,945,854；两侧 167 artifacts、subs
2,050,283→2,050,571）；12 变化套件 14 变化键。组行五项闭合：
**dom −118、shadow-dom −1、web-animations +2、css +157、editing
+258＝+298 精确闭合**＝zw_passes 1,928,302→1,928,600；分母恒
2,279,168/html 160,190（零漂移第四窗）；覆盖文件 74,189 恒；
full_pass 文件 63,636→63,639（+3）；本地旧 cache 复现 face 行
109,425 同值双证。

## 停摆形第八演＝双边外穿双向记带

- **editing/run 95,184 → 95,442（+258，上缘外穿 +89）**：
  multitest ?9001-last 926→1,092（regs 927→1,092 全回满、TO 1→0）
  ＋delete ?7001-last 632→724（regs 699→822、F +31）——批二十八
  dom 上穿单窗尖峰先例照读＝实测带暂记 95,130~95,442。八窗序列
  b24 +96→b25 −220→b26 +223→b27 +5→b28 −125→b29 −20→b30 −29→
  **b31 +258**。
- **dom/ranges 42,140 → 42,022（−118，下缘再穿）**：dataChange
  873→835、replaceData 915→835（同两键第九演）＝实测带暂记
  42,022~42,677。八窗序列 b24 −350→b25 +120→b26 −247→b27 +198→
  b28 +306→b29 −441→b30 −96→**b31 −118**。零码窗连震＝timing 形
  第四互证；dom/ 邻账归 wptfyi-dom。

## css 带精确镜像回位（批三十 −140 的整带镜像）

color-computed-hsl **+136**（regs 3,618→3,753＝与批三十 −136 同键
逐字节镜像对）、borders corner-shape-interpolation **+18**（批三十
−16 镜像）、writing-modes +1、values +1、animations event-dispatch
+1（regs 16→18）、transitions 零（regs 30→17 收缩）、forms 双键
P 互换净 0——css 组行 **+157**＝批三十 −140 带的整带镜像回位。
四窗读数 b28 −34→b29 +34→b30 −136→b31 +136＝对称震荡定谳强化
（同键镜像对第二演）。shadow-dom −1＝DSD-opt-in 109→108（regs
119→116）＝批三十 +1 同键镜像对（timing 带）；web-animations +2
（DocumentTimeline/constructor 2→4，批三十 −3 带续震荡）。

## 负向两形第五窗＝对端修复兑现监测第二读零回位

test262 **51,907 持位**、csp **199 持位**（两套件均不在变化清单）＝
五窗连零回位（对端未落行为修复）；本窗窗内的 a3932b528 为 TT 域
（③③b 主嫌域）**编译适配**（非行为修复）且未入任何套件分片码图＝
其对 test262 collateral 面的读数**顺延批三十二**（下窗首次携带）。
**CE·SD 第七窗持位**（changed entries = 0）；**DP①② 第四演持位**
（domparsing/sanitizer 零键变化）；tt 1,482 持位。

## 观察清单全持位

onblur checkable-active-onblur-with-click **3P/4F 五连持位**
（b27→b31＝回稳判定候选升级：五连 3P）；webappapis 1,540/207
files/subs 2,540 逐字同；per-global 6 注册 3P/3F names 全同；dat
170 键 shapes 148TO/19F/3 逐字节同；interestfor 30 键逐字同；
variant 三键 DISTINCT regs/P/nameset 569d8f/d83306/a823af 全同＋
同值检测互等；合成 Fail 扫描 **103=103 逐套件同**。

[逐文件 JSON](2026-11-23-dc3-batch31-run-reconciliation-face-hold-and-code-knife-outside-suite-shards.json) ·
[批三十证据](2026-11-23-dc3-batch30-run-reconciliation-zero-code-window-and-autodl-dirname-lesson.md)
