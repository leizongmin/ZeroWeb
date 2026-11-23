# DC-3 · 批三十：run 45145268804 收账——零码刀窗全带内（face −1＝track-cues 交替带第八演）＋负向两形第四窗零回位＝对端修复兑现监测通道首读持位＋布防课：批间脚本子目录名残留伪回归识别

2026-11-23。run 45145268804（09:14:42Z dispatch，~11:15Z 完成，快照
coverage-2026-11-23 同日覆写＝11:16:43Z 生成落 fc0231be0，manifest
恒 063eb7cae2 零漂移第三窗）对批二十九 run 45124287002 同分片逐键
A/B。布防事故二如实记：批间 sed 复制 autodl.sh 只替换了 /tmp/batch29
→ /tmp/batch30 路径前缀，**字面量子目录名 run29-raw/run29 残留**＝
批三十产物下载落 /tmp/batch30/run29/；A/B 首跑 new dir（run30/）不
存在 → 全套件 subs=0 伪回归 −1,945,824（识别：全 0 模式 P/F/TO 三
值全零＝缺档形非真回归）；目录改名后重跑恢复（167 artifacts 完整，
两侧 total subs 150,582 同量级互证）。**课：批间 sed 复制须同时替换
子目录名字面量；A/B 首跑前先验证 new dir 存在且 subs 总量与基线同
量级**。分片 checkout 逐 job 实测：**55 分片+gap 全提取＝40×
da82f2226＋14×7e113546e＋1×5289372d5**（后两者 docs-only，三 SHA
祖先链互验全过＝码图同 da82f2226）。窗口码变：**零码刀**（3fa4c2d76..
da82f2226 5 commits 全 docs/chore(wpt)；fc0231be0 快照数据写回非码）
＝本窗全部变化零码归因。waiter 本窗遇 API 网络抖动（poll25-26 连接
错误），RUN_DONE 由主 Agent 手动补写、autodl 幂等承接，无产物损失。

## 目标级账

html 面 109,426 → **109,425（−1）＝净 +38,329/基线 71,096，验收
≥152,096 余 42,671**。−1 唯一构成＝track-cues-sorted-before-dispatch
Pass→Fail＝交替带第八演（七演 F P F P F F P → 八演 …P **F**＝带内
震荡续，不逐轮追判定维持；批二十九键级验键先例续用）。

## 负向两形第四窗＝对端修复兑现监测通道首读

- **test262 51,907 持位、csp 199 持位**：判读窗（批二十九）已收口
  为码形主导证据，本窗起读数转为**对端修复兑现监测**——③③b
  081d40b4a／③③a+③⑤ sink 域主嫌证据链已移交对端，后续窗对端
  修复落码后回位即兑现读数。本窗首读＝零回位（对端未修）。
- **CE·SD 第六窗持位**（property-reflection-idl-setters 225P/2,685F
  等五键族 changed entries = 0）。
- **DP①② 第三演持位**（domparsing/wc/sanitizer 三组行零键变化＝
  批二十八 +49 组行读数第三窗稳定）；tt 套件 1,482 持位。

## 停摆形第七演＝带内

- **dom/ranges 42,236 → 42,140（−96）**：dataChange 923→873、
  replaceData 961→915——下缘 42,173 轻微外穿 −33＝实测带范围暂记
  42,140~42,677（批二十八上穿 + 批三十下穿双侧扩缘，中值回归带内）。
  七窗序列 b24 −350/+96 → b25 +120/−220 → b26 −247/+223 →
  b27 +198/+5 → b28 +306/−125 → b29 −441/−20 → b30 −96/−29＝交替
  震荡续（零码窗连震＝timing 形三度互证）；dom/ 邻账归 wptfyi-dom。
- **editing/run 95,213 → 95,184（−29 带内 95,130~95,353）**：
  multitest ?9001-last 1,057→992（TO +1）／forwarddelete 350→406
  （regs 416→491 回满）／delete 586→632（regs 636→690）。

## flake 带与邻账小项

- onblur checkable-active-onblur-with-click **3P/4F 四连持位**
  （b27→b30）＝水位下修形稳定（5P→四连 3P）；
- **shadow-dom +1**＝DSD-opt-in 108→109P（regs 116→119 回批二十七
  水位）＝XHR document.open regs 带第三演回位（批二十八 −3 回撤的
  镜像恢复，timing 带）；
- css 组行 −140＝震荡带（color **−136** regs 11,338→11,203 页面
  注册收缩＝批二十九 +34 后大跌、三窗 ±34/−136 扩幅震荡 watch；
  animations +16/borders −16 镜像、writing-modes −3、values −1、
  transitions regs 回落 14）＝带内不深挖；
- web-animations −3、IndexedDB 零变化。

## 观察清单全持位

webappapis 1,540/207 files/subs 2,540 逐字同；per-global 6 注册
3P/3F names 全同；dat 170 键 shapes 148TO/19F/3 逐字节同（write TO
恒定续持）；interestfor 键族逐字同；variant 三键 DISTINCT regs/P/
nameset 569d8f/d83306/a823af 全同；合成 Fail 扫描 **103=103 零漂移**
（新旧两侧逐套件同）。

## 快照闭合

组差合计 **−268**＝zw_passes 1,928,570 → 1,928,302 逐项闭合
（html −1、dom −96、editing −29、css −140、shadow-dom +1、
web-animations −3）；覆盖文件 74,189 恒；全过文件 63,640→63,636
（−4＝track-cues 退出＋css/web-animations 域伴动，文件级口径照记）；
分母恒 2,279,168／html 160,190（零漂移第三窗）；本地旧 cache 复现
face 行 109,425 同值双证（zw_passes 1,928,302 同值）。环比护栏未
触发（−268 带内震荡）。

[逐文件 JSON](2026-11-23-dc3-batch30-run-reconciliation-zero-code-window-and-autodl-dirname-lesson.json) ·
[批二十九证据](2026-11-23-dc3-batch29-run-reconciliation-zero-code-window-and-negative-third-window.md)
