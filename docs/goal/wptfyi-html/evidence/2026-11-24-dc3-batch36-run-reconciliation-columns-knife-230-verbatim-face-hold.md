# DC-3 批三十六收账——columns 刀 +230 自报逐字精确兑现、face 109,426 持位零变化

2026-11-24。run 45304091946（01:18:45Z dispatch，HEAD ef0df1118＝批三十五收账提交
rebase 后，coverage-only `-f run_wptfyi_coverage=true`；03:35:23Z completed
success，历时 ~2h17m＝历史带（2h12-16m）上缘）。上一会话于 03:53Z 因 API 429
中断于下载完成点，本会话零损失接续收账。

## 窗码图

55 提取＝**36× ef0df1118（dispatch HEAD）＋18× 1a59ba257c（批三十六布防
docs，dispatch 后 90s 落）＋1× cd65c9f47（aggregate 唯一，03:35:02Z 起跑）**：
1a59ba257c 对 ef0df1118 互 diff 仅两 master.md＝docs-only 等码归并＝均匀同码窗；
cd65c9f47 唯 aggregate 捡到（批三十一同形，不跑套件＝零套件面影响）。

**窗内码刀（对批三十五套件 checkout c5c49ec0 全量携带，三刀）**：

- **43ca50c9d feat(css) columns 简写语法收编**（wptfyi-css 流，11-23 23:01:02Z
  落——批三十五窗内 dispatch 后 27m40s 落唯 aggregate 捡到，套件面未及；本窗
  全分片首次携带）＝自报 multicol F2P=230/P2F=0。
- **3fc122daa fix(style) 剥离 43ca50c9d 误卷入的 contain-intrinsic hunks**
  （11-23 23:47:35Z，构建面修复，与刀成对落库）。
- **d4bd1cec1 fix(engine) registries 刀②**（wptfyi-dom 流，00:34:21Z）＝批三十
  五预布兑现＝刀②归因窗。

## A/B（基线 /tmp/batch35/run35，subs 2,050,659 → run36 2,051,036）

套件面总 P **1,947,180→1,947,791（+611）**，组行闭合：

| 组 | ΔP | 归因 |
|---|---:|---|
| css-multicol | **+230** | **43ca50c9d(+3fc122daa) 自报逐字精确**：columns-interpolation 152→350（+198）/columns-computed 9→27（+18）/columns-valid 10→24（+14）＝+230；F 592→362 镜像 −230＝P2F=0；subs 恒 1,613。对端刀首入窗清单内全额兑现 |
| dom | +417 | 停摆形第 13 采样：Range-mutations-dataChange 919→1133（regs 920→1,134 +214）/replaceData 920→1123（regs +204）＝**regs+P 双升＝注册完成度形新变体**（页注册更全），dom/ranges 42,191→42,608 带上缘内（42,677）；刀域无关（窗内无 dom 码刀）＝timing 续证 |
| editing-run-er0 | +49 | multitest.html?9001-last 1035→1084（regs +1）＝editing/run 95,287→95,261 带内（−26）；停摆带同域 |
| editing-run-er2 | −75 | delete.html?7001-last 626→551（regs 692→596）＝**截断形第四读加深**带内，不定谳 |
| css-borders | −12 | corner-shape-interpolation 145→133＝wobble 尾巴带内（watch 值 145 本身即 wobble 位） |
| css-backgrounds | +1 | background-computed 24→25＝columns 刀对清单外正溢出候选（style apply 邻域）单窗不定谳 |
| html-rendering | −3 | svg-inline.html 408→405（regs 648→639 收缩）＝**rendering 面新观察形首现**，单窗不定谳；face 净零的唯一负构成 |
| html-semantics | +2 | onblur checkable-active 3→5＝**5P 回位**（水位震荡带例行） |
| media | +1 | track-cues-sorted-before-dispatch 0→1＝**交替带第十四演 F→P**（13 演后回 P，仅记不追） |
| web-components | +1 | DSD-opt-in 108→109＝108↔109 震荡对第六腿例行 |
| css-animations | 0 | event-dispatch regs 19→32 扩张 P 恒＝注册面 wobble 无 face |

**d4bd1cec1 刀②归因窗第一读＝零 face 兑现**：custom-elements/registries 域零
键变化（自报探针 7P/2F 为桥内钉测形非 face 数）；按域双向扫溢出＝负向无新形
（dom/editing 变化全落停摆带内）。单窗不定谳，下窗续读。**6a8df8cee（刀②续，
自报 WC +20P/−20F、effective-global-registry 56F→48F）03:52:15Z 落＝aggregate
checkout 后＝批三十七全分片预布＝刀②续归因窗**。

## face 与快照

- **html face 109,426→109,426 持位零变化**（coverage 口径净 +38,330 不变，
  验收 ≥152,096 余 42,670）。构成＝semantics 10,901→10,904（+3）−rendering
  2,793→2,790（−3）＝净 0。
- 快照 coverage-2026-11-24 **同日覆写** 03:35:18Z 生成、03:35:19Z 落
  **9ed72ec31**（2 文件 36+/36−小差）；**manifest 恒 e00a7b14e8 零漂移**（批三十
  五换代后持位，无新漂移窗）；CI 分母恒 2,279,165、covered 74,189 恒；
  full_pass 63,640→63,643（+3）；zw_passes 1,928,825→1,929,436（**+611＝A/B
  套件面总和精确闭合**）；护栏未触发。
- 本地旧 cache（a3830cd8f7 分母代）face 行双证：html 109,426（den 160,124）、
  zw_passes 1,929,436、full_pass 63,643＝与 CI 逐字同值（分子与分母版本解耦
  维持）。

## 观察清单例行读

TT 1,482 持位四读；domparsing 1,264／sanitizer 414 新水位持位二读；selection
34,042（+44）三读持位＝timing 候选定谳强化（连续两窗持位后续第三窗仍持位）；
CE·SD idl-setters 225P/2,685F/2,910regs 第十二窗逐字持位（对端修复读数通道不
变，刀②续下窗进）；test262 51,928／csp 199＝**负向两形第十窗零回位**；dat
170 键零变化；dialog 1/1 持位；idbcursor 零变化；web-animations 526 持位；
per-global/webappapis 持位；**合成 Fail 扫描集恒 61=61**（零新增零消失，
HTMLTrackElement os error 2 ×4 为两侧同集持久形非新档）。

## 布防第六窗与新布防课

- waiter 全绿（poll29 03:40:24Z completed→RUN_DONE 自动写）；shas 全绿第三窗
  （收紧 checkout 锚，55 行验收过，三 SHA 对象存在性＋祖先链全过）。
- **新布防课＝批间脚本复制 exec 位丢失**：batch35→36 sed 复制未保 exec 位，
  retrydl.sh 落 `-rw-r--r--`＝autodl 主路径 03:42:06Z 调用报「权限不够」断
  （autodl.out 在档）；上一会话手动 `bash retrydl.sh` 补跑（03:45:38Z 起
  54 artifact 首轮全 ok）167/167 无损收齐（03:53:08Z flattened=167 done）。
  修复＝批间复制 `cp -p` 或复制后 `chmod +x`；验收形＝RUN_DONE 后还须核
  dl.log 完成形（flattened=167/done）与 autodl.out 无错误。本轮已就地 chmod。

## 下窗承接口（批三十七观察清单）

6a8df8cee 刀②续归因窗（自报 WC +20P/−20F 六页全限定＝registries 正向候选＋
负向照例行双扫）；45efa2584+cd65c9f47 columns B-cut 对归因窗（对端电池
F2P=13 超门翻转收账形，css-multicol 新水位 1,251 持位读）；05d32ab5c
perf(font) mmap 驻留首入窗（PR #321，字体载入路径 perf 改动＝timing 敏感带
读的归因干扰新变量，停摆带/track-cues 带读须加注）；columns 刀清单外 +1
（css-backgrounds）二读；css-multicol 1,251 新水位；domparsing/san 新水位
1,264/414 三读；selection +44 四读（定谳窗）；editing delete 截断形第五读；
dom/ranges 停摆带第 14 采样（新变体 regs+P 双升是否复现）；rendering
svg-inline −3 二读（新形定谳窗）；corner-shape/event-dispatch regs wobble
例行；CE·SD 第十三窗＋DSD 第七腿；track-cues 第十五演；onblur 震荡带例行；
负向两形第十一窗；TT 五读；合成 Fail 扫描照旧。
