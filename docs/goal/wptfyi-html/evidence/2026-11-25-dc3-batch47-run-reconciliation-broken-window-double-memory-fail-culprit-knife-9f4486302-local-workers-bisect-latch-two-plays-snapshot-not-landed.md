# DC-3 批四十七收账——破窗（attempt-1 双分片内存败＋0 字节毒文件伪装回归→环比门闩锁两演拒写快照）＝本地 workers 二分定谳肇刀 9f4486302（对端 dom 流 k20 补丁 V8 堆滞留）＝rerun ×2 确定性同败＝gethtml 对照实验洗清截断簇＝早期 A/B 刀面正面 +598（TT +128/mixins +110/values +47/color +95 首演兑现）＝快照未落账 face 读数阻塞

2026-11-25。run 45595166328（10:27:31Z dispatch，HEAD
7e75a8c11＝批四十六收账快照提交 78a4588cf 后续笔，coverage-only
`-f run_wptfyi_coverage=true`，窗 42 commits 码刀 12）。上一会话
dispatch＋脚本族预布＋脱离启动后本会话接续收账＋**破窗全链归因**
（attempt-1：52 shard 中 **net 分片 V8 Fatal OOM 10:48Z**（job
139139362383，workers 域 "Ineffective mark-compacts near heap
limit"）＋**test262-builtins test-guard 顶格击杀 11:55Z**（job
139139362513，per-proc 超 6 GiB 帽）；**aggregate 12:25Z 环比门
exit 3**＝`third_party: 51907→28831（−23076，跌幅 44%）`拒写快照。
真因＝两失败分片上传 **0 字节毒 JSON 占位**（run47-a1 里
testharness-test262-builtins.json 与 testharness-workers.json 均 0
B）→聚合分子归零→把缺数据伪装成回归→门闩锁拒写。`rerun
--failed` attempt-2 12:31Z 起跑＝**×2 确定性同败**（net 12:51Z 同
V8 OOM；test262 13:38Z 击杀 6,325,176 KB>帽 6,291,456 KB 与首杀
6,335,116 KB 差 0.1%＝贴帽确定性；aggregate 13:38Z 同数字 exit 3
逐字重演）→13:38:37Z concluded failure，**快照未落，face/三口径
读数阻塞**。批四十六自愈形不可复演＝内存回归确定性失败 rerun 无
效。attempt-1 下载 167 文件中 165 健全（另 2 即毒文件）。）

## 在途 log 不可取课（批四十六课修正）

failarch 哨兵 10:58Z 在途归档三份失败 job log **只拿 119 B 占位**
（GitHub：run 在途时 log 不可取 "will be available when it is
complete"）→finalarch 哨兵等 RUN_DONE 标记后补归档三份真 log
（160/131/87 KB）。批四十六课「失败 log 须在 rerun 前提取」修正为
「**须在 run concluded 后、rerun 前提取**」：在途取不到，rerun 后
attempt-1 log 被 wipe。本窗三份 attempt-1 真 log＋三份 attempt-2
真 log 全存 `/tmp/batch47/failed-logs/`。

## 本地二分归因链（workers 套件 RSS 对照＝数小时替代 CI 数小时）

release 构建逐 commit checkout＋test-guard 包裹
（`--per-proc-mem 4 --total-mem 8 --time-limit 900`）＋
`/usr/bin/time -v` 测 peak RSS，workers 套件全量单跑：

| commit | 刀 | 产物 | peak RSS |
|---|---|---|---|
| b46bcfacb | 批四十六基线 | 完整 175 files 696P/18F | 4,459,744 KB |
| fe1ee9ed1 | k19b | 与 b46 **逐字节同** | 4,458,584 KB |
| 7c9cb581e | k20 | 175 files 696P/18F，唯一差异单子测 TO↔F 时序翻转＝计数级 SET-IDENTICAL | 4,434,996 KB |
| **9f4486302** | **k20 补丁＝肇刀** | **无 JSON exit 133 V8 Fatal OOM** | **4,554,756 KB** |
| 12924df1d | k20b | 无 JSON exit 133 | 4,531,600 KB |
| 7e75a8c11 | dispatch HEAD | 无 JSON exit 133 | 4,576,004 KB |

单调排除：k20 (7c9cb581e) CLEAN ⇒ 其前全部码刀（k111/k19/k19b/
k112/k113/mem_diag 观测仪）clean；k20b OOM 先于 k114 ⇒ 肇刀窗收敛
至 [9f4486302, 12924df1d]，k20p 直接 OOM **定谳肇刀＝9f4486302**
（k20 补丁：动态 meta CSP 指纹失效，对端 wptfyi-dom 流 engine
域；diff 3 files 128+/11−＝part165 测试 +81＋js_dom_shim/part04.js
＋part14-trusted-sinks.js）。机制候选＝**RSS 同量级但 mark-compact
失效＝对象强滞留形**（非裸内存增长）：动态 meta CSP 指纹重扫路径
（首调用缓存改活动重扫）滞留＋R522c 委派序换。CI 佐证＝attempt-2
test262 击杀 6,325,176 KB 与首杀 6,335,116 KB 差 0.1% 贴帽确定
性；本地六探针 RSS 全落 4.43–4.58 GB 档。**修复归对端流，本流不
硬修 engine 域**（并行双流工作面纪律）；已飞书照会。

## 截断簇洗清（本地对照实验）

早期 A/B 负面簇（web-components −2,502/er 系 −3,690/dom −357/
html-rendering −142）险些误判刀回归。**gethtml.html 本机对照**＝
b46bcfacb 本地同样尾部挂起（3,390 subs TO=1 截断形），HEAD/k20/
k19b/k19/k111 六探针全挂或截断＝**本地时序伪影**非刀回归；CI
attempt-1 gethtml 1,788 截断＝宿主内存压力窗产物（同窗三 OOM 三
证）。截断簇批四十八满窗重跑预期自然回带。

## 早期 A/B 读数（attempt-1 残缺聚合 167 套件 vs 批四十六快照 78a4588cf）

总 P 1,946,828→1,916,940（**−29,888**）＝毒文件对
−23,775（test262-builtins −23,079＋workers −696）＋余 165 套件
−6,113。余内＝**正面 15 套件 +598** 对冲**负面 9 套件 −6,711**
（截断簇）。**刀面正面兑现**：

- **trusted-types P 1,671→1,799（+128）F 347→211**＝k20 系五刀叠
  加面，013/014/015 归零候选兑现（布防预判命中）
- **css-mixins P 158→268（+110）**＝k113 解析期框架兑现（at-
  function-parsing 38→86 领衔 15 键；k114 dispatch 后落树未跑）
- **css-values P 6,670→6,717（+47）**＝k111/k112 NET 逐字
- **css-color +95**（color-computed-hsl 3,568→3,663）＝镜像回摆
- css-animations +16（getAnimations 1→17 回位领衔）＋selection +8
  ＋css-borders +5（corner-shape-interpolation 146→151）＋er2
  +182（截断带内反向，复验）

毒文件甄别注记＝test262-builtins/workers 两行 P=0 为 0 字节占位
假读数，非真回归；**本窗一切读数均为早期口径，批四十八满窗复
验**。

## 新形态课

1. **毒文件形态**：失败分片上传 0 字节 JSON 占位→聚合分子归零→
   环比门把缺数据伪装成 −44% 回归拒写（third_party 51,907→28,831
   假回归）。毒甄别＝精确文件名字节数检查。
2. **内存回归 rerun 无效**：确定性贴帽失败 ×2 同败，批四十六瞬态
   自愈形不可复演；判据＝击杀值与帽差 <1%。
3. **RSS 滞留指纹**：peak RSS 同量级（4.5 GB 档）＋mark-compact
   失效＋无产物 exit 133＝滞留形回归，非裸增长。
4. **本地对照法**：单套件 RSS 对照可数小时完成 CI 数小时归因；尾
   部挂起先跑旧基线对照防误判本机伪影。
5. **SHA 程序化**：手打转录多次伪差（b46bcfacb 错一字符→
   checkout 静默失败→跑旧二进制），一切 SHA/路径用 git 命令提
   取。

## 批四十八布防

- **触发**：对端修复 9f4486302 滞留回归后重 dispatch（本流不动
  engine 域）；dispatch 前 workers 套件本地 RSS 复测确认修复。
- **窗**：预计含 k114 57222a907（@function 定义文法，本窗
  dispatch 后落树未跑）；A/B 基线照旧取批四十六 78a4588cf。
- **复验**：TT/mixins/values/color 刀面满窗复验＋gethtml/er/
  ranges/svg-inline 截断簇回带候选＋k20 系 013/014/015 归零
  对账。
- **台账**：face/三口径读数待快照落账后补记本窗行。
