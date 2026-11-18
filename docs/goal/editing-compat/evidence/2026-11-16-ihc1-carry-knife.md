# ihtml 携带族 override carry 刀门禁批收口（ihc1）

**日期**: 2026-11-16
**刀体**: 1477a5657（发射锚 8fb4e955d，在飞窗内五次 pull --rebase 重放 b48956330→6c37909ea→7894ff141→8d08b7f86→1477a5657 推送定锚不可变，crates/engine＋tests 域 diff 恒空、part06.js blob md5 3a0ea2ec 恒等传递闭合；订正轨迹 b48956330→8d08b7f86→1477a5657 见 COORD 09:45 备忘条目）
**批**: ihc1 单批 ALL-DONE（08:31:44 发射，09:37:29 终判）
**census**: inserthtml 1508P/57F → **1516P/49F**（F2P=8 恰携带族，零新增红）

## 刀体内容

`crates/engine/src/js_dom_shim/part06.js` 新增 `_zwIhtmlCarryOverrideOpen`：劈裂壳 font color attr / span·font style color 声明取色，swcss true → `<span style="color:rgb(...)">` 形、false → `<font color="#hex">` 形（`_zwVaNormalizeCssColor`/`_zwVaParseSimpleColor` 同 forecolor 序列化口径）；色值不可解析、face/size attr 或 font-family/font-size/font style 声明同壳 → null 维持旁路（face/size 携带域与混合壳语义未实证，后续刀）。BLOCK_T 臂素净载荷 × carry 壳接管：包壳在载荷块根内部包住内联内容（语料 inserthtml 8 键实录形 `<p><span style=color…>abc</span></p>`）；carry 域限内容模型为行内的块根（p/div/h1-h6/blockquote）；li/dt/dd/list/table/hr 载荷域维持旁路；payload 自带 style 或壳仅 font-variant 类非 carry 样式放行走通用劈裂（smallcaps 形现役绿路径不变）。

`crates/engine/src/js_dom_bridge_tests/part124_ihtml_block_pins.rs` 旁路锁定钉换 carry 钉四形（`ihb-bypass-font-carry`/`ihb-bypass-span-carry` → `ihb-carry-font-false`/`ihb-carry-span-false`/`ihb-carry-font-true`/`ihb-carry-span-true`，+2 净增 JS 钉），四形锁语料实录期望串；CE=false pre 旁路钉与控制三形现役绿路径恒一。

## ihc1 判读（ALL-DONE 09:37:29）

- **L0**：release runner 复建 bin=`cec0d8bde4ba1b6bdd59fc17848c26a2`（字节锚全腿恒一），刀体特征串 ksig 2/2。
- **L1/L2 iht A/B 双腿**（1565 行，乘数 2，wall 11s/63s）：**1516P/49F/0T，F2P=8 恰携带族＋零新增红**。F2P 集合独立复核（comm 对 ihb3 基线）：交集 49、新增红 0、消失恰 8 键——font color 壳 ×4＋span color 壳 ×4，全 `<p>abc` 载荷 × compare innerHTML 行，swcss true/false × dPS div/p 四象限。
- **L3 mt**（wall 1864s）：**10090P/2F/0T**，残红恰 subscript+formatblock X37 双键，零暴露（multitest inserthtml 仅行内载荷，块臂/携带臂全旁路）。
- **L4 enginelib strict**：**4059P/0F**（16.49s）。
- 锚定纪律：发射前现树复验（pins 绿＋enginelib 4059P/0F＋iht 单腿 1516P/49F F2P=8 comm 双证）；L2 起跑前 ensure_solo 等 ZW-2 渲染流租户（shapeout-b1v4 批）清空；chk_dom 对 rebase 重放（8fb4e955d→1477a5657 五次，engine/Makefile 域 diff 恒空）未误触熔断。

## writer 死亡与尾链代跑

writer CC session 于批在飞期间结束（harness task stopped），批进程 setsid 脱离存活跑完全程；刀作者延续轮按 COORD 2026-11-16 08:33 条目回执代跑尾链（判读复核→本 evidence→控制面 v1.115→commit→push）。

## 残 49 红族谱（57−8 对账闭合）

| 族 | 键数 | 处置 |
|---|---|---|
| span color 壳＋行内 `ab<b>c</b>d` 载荷 | 8 | 内联臂携带域——后续携带刀 |
| CE=false pre 链 | 1 | 轴二 pre 壳臂现役 got 锁定（part124 ⑲ 钉） |
| multi-root div 载荷＋`{}` 选区 | 1 | 单根门旁路——multi-root 扩展挂账 |
| inline 载荷其他（text/空/ws/comment/b/i/script/xmp 等） | 39 | 轴二/assert/ws 族维持 |

## 下一步

- **span color 壳行内载荷携带刀**（8 键：`ab<b>c</b>d` 行内载荷入 font/span color 壳，内联臂携带域）；
- multi-root 载荷扩展（1 键，可搭车后续块臂轮）；
- assert_in_array 9 键收尾批量判读、exec1 474 域决策挂账维持。

## 批脚本

`.acceptance/dps/ihc1-tail.sh`（md5 b10165790a56839368a4145fae546ea8，chmod 400 冻结）。协调全录 `.acceptance/dps/COORD.md` 2026-11-16 08:33 条目。
