# F7 delete-merge-div 两微机制刀（2026-11-09）

mt 残 21F（F4 收口后）之 F7 misc 12 首分治块：delete-merge-div 6（尾残 br 4 + 空壳并存 2），multitest `delete>inserttext(g)>inserttext(h)` 链族 flex/grid/plain/b 嵌套变体。ip-placeholder 4 / host-div-wrap 1 / trailing-empty-p 1 留分治后续刀。

## 根因链（两臂，均 part01f deleteSelection/块合并出口）

臂 1（:501 补 br 守卫）：「块暂无可见子」补 br 臂的可见性判定在**迭代中途**低估幸存壳占位——跨行内选区（`{abc` 起 span1、`}` 落 div 元素位）删净后 span1 空壳幸存于块中，flex/grid 布局下 div 直下仅剩空壳时 some(isVisible) 为 false，臂对 `parent_`（div 本体）补块级 br；尾部空 span 仍在 nodeList 的迭代轮二次触发双 br。br extraneous-invisible 不阻后续迭代，产出 `<span></span><br>` 尾残（typing 后 got `<span>gh</span><br></div>`，Chrome 期望壳本身即占位、不补 br）。g17 先例（blockquote>font 删净壳保留不补 br）的 `parent_ !== delKeepShell` 守卫只挡「壳=parent_ 直下」形，壳在块内更深一层（含块直子 span1）不挡。修法：守卫扩 `!_zwJusIsAncestorOf(blockOfParent, delKeepShell)`——幸存壳在块内任一层即块内占位，不再逐迭代补 br。

臂 2（:660 块合并 exit caret）：`_zwJusIsAncestorOf(endBlock, startBlock)` 块合并臂的出口 `collapse(startBlock, nodeLength(startBlock))` 落**末子空壳后的元素位**（探针实证 DIV#k@1 形），typing 在元素位以 armed tags 重建并列新壳（got `<div><b></b><b>gh</b></div>` 双壳，Chrome 期望壳内续写单壳 `<div><b>gh</b></div>`）。Chrome delta：块尾幸存空壳即 caret 载体。修法：`shellExitDesc` 四条件（delKeepShell 存在 ∧ parent==startBlock ∧ 为末子 ∧ nodeLength==0）折入 `(壳, 0)`；壳非末子/非空/delKeepShell 缺省走原语义零扰动。**纯 caret 出口问题，无需 part01h capture/consume**（开工前设计已被探针推翻简化）。

K2 护栏面（`abc{def` span2-caret 形，今日通过键）：折入后 typing 直接续写 span2 壳 vs 原元素位重建壳——终态 html 相同（delete.js:3144 纯 delete 期望 br 在 span2 内亦与折入方向一致）；同节点/无壳形 delKeepShell=null 两臂均不触。

## 刀体（commit 56a11928e；单 shim + fixture）

- **part01f.js +23/−4**：:501 守卫扩 `!_zwJusIsAncestorOf(blockOfParent, delKeepShell)` + :660 `shellExitDesc` 折入 `(壳,0)`。字节面史：builder WIP 版（md5 `6cf68599`）与 stash 窗口轮注释压缩版（md5 `a6b1839d`，blob dc014afad）语义逐条件等价（仅注释措辞），入库为压缩版；stash@{0}「f7-wip-baseline-window」（SHA `0c97821d`）为窗口轮 apply 残留，内容已等值入库，勿消费勿 drop。
- fixture `zz-probe-f7.html`（add -f 入库）：六刀形（D1 flex/D2 grid/V3 plain/V1 尾空壳——删除后中间态无 br + 终态单壳精确断言；D3/D4 跨块空壳并存终态单壳）+ 三护栏（K1 无壳 delKeepShell=null、K2 `abc{def` 壳内已然形、K3 同节点 `{abc}` 壳内 br 原语义）。r1 唯一红=探针自身标记提取器同节点双标记 bug（`replace(/[{}]/g)` 双删），逐标记串行剥离修复后 **9P/0F**。补充认证：zz-probe-f7dmd 12 案 72P/0F（b 深层壳变体 faithful + CASE603-614 直调，窗口轮产物）。
- md5 锚（批硬绑定八点）：part01=`f283e81b` part01c=`fb8aa0d8` part01d=`9fbab2af` **part01f=`a6b1839d`** part01h=`9592ebd3` part06=`e32619e9` bridge=`f7d65f32` fixture=`87f62c7d`；特征串 F4 全数不变 + f7ph（`blockOfParent, delKeepShell`）=1 + f7ex（`shellExitDesc`）=2。

## 机制预测

- mt assertion/entry 级 F2P = 恰 6（delete-merge-div 两微机制族：尾残 br 4 + 空壳并存 2，含 b 嵌套变体）；P2F=0；残 21→15F。
- ilb/ip 共享 deleteSelection 臂（:501/:660）零漂移（域外键不触两臂条件）。
- calibrate 白名单 = 裸 `[["delete",""],["inserttext",…]]` 链签名 × 双页 = 12。

## 门禁批判读（batch-f7-delmerge-gate-v1.sh，08:51:19 发射 head=56a11928e，builder=169412 刀体认领轮）

批协调事件：刀体 b71667a55 提交后被并行流 pull --rebase 重放 **56a11928e**（并入 6b14662ba s359 JSON parse 定位域，script_gen.rs/webview.rs 不触八点锚，字节零变）并由监测轮先期 push origin；基线窗轮（zw-f7-baseline.sh）为取 F4 终态净树跑 delete/fd 语料基线，stash 收走 WIP 后 apply 回落——builder 判读字节等价后沿回落面提交。delete.html 语料腿 1800s 超时死（0 输出行，语料规模超腿帽——delete/fd 语料腿不入门禁，mt 为 delete 域唯一机制门）；fd 腿 09:02 rc=1（文件名 f7-fd-base.log 实为刀体字节面 post，判读不消费）。批发射由代跑轮在 builder 等待 fd 腿退出期间按「写者死亡代跑」纪律代发（builder 全程在线，追认有效——锁内串行、锚正确），builder 转判读位。终腿 ABORT 定性：binary sig 串 '壳在块内任一层' 在注释压缩版中跨行 grep -F 失效（核验表达式措辞 bug 非引擎面），sig 修正 + 补跑 5 串全 OK（见批日志补验段）。

锚复读 OK（八点 md5 + 特征串含 f7ph=1/f7ex=2）；mt 后八点 md5 复读门过；二进制身份（learning #28）：`⑤ F7① 扩面` / `块尾幸存空壳即 caret` / `块 caret 归一补臂` / `断点后 caret 下游归一` / `处方 c（knife11 v2 同构` 五串补验全在产物内（二进制=a6b1839d 源码字节面，mtime 08:37）。

| 腿 | 结果 | 判读 |
|---|---|---|
| probe-f7 | 9/0 rc=0 | 刀 fixture 全绿（六刀形中间态+终态 + 三护栏） |
| probe-f4ilb | 99/0 rc=0 | **deleteSelection 共享路径防回归主门绿** |
| probe-f1shell / f1b / f2 / f3 / f6b | 27/0 / 7/0 / 14/0 / 7/0 / 17/0 | 回归护栏五腿恒一 |
| probe-k11 | 224/17，rows 229 恒一 judge OK | 大探针零漂移 |
| ilb 语料 | vs f4ilb-ilb-post：F2P 0 P2F 0 post-only 0 lost 0 | 共享臂零漂移 |
| ip / ihtml | rows 7195 / 1565 恒一，F 206 / 119 恒一 OK | 零漂移 |
| exec×3 | rows 1806/1/6 恒一 OK×3 | 零漂移 |
| **mt judge** | **rows 10092 恒一，P 10071→10077 / F 21→15，F2P=6 P2F=0 post-only 0 lost 0 OK** | **机制预期恰值命中** |
| mt calibrate | entry 翻键 12=6 键×双页，裸 delete 链白名单 12/12，**白名单外 0**；flex/grid plain+b 嵌套×2 页逐键对账 | 域外零翻转 |
| iol / iul | rows 1761/2065 恒一，F 2/4 恒一 OK | 零漂移 |
| enginelib | 3819P/7F（13.41s），ROSTER OK(7-red)（3816→3819P = s359 并入测试面 +3，名册恒一） | 名册恒一 |
| battery 七腿 | 420/880/1104/552/3456/2607/1455 全 F=0 T=0 | ambient 零扰动 |
| binary sig 补验 | 5/5 OK（批终腿措辞 bug 修正后单独补跑） | 二进制身份闭合 |

## 残账与下一步

mt 残 15F：F7 misc 剩 6（ip-placeholder 4 + host-div-wrap 1 + trailing-empty-p 1 分治后续刀）+ F6b handoff 4（N1/N2/N4/N5 backlog）+ F6a 2 + F1' fontsize(4) 1 + X37 2（除名）。DC-4 降序下一刀目标：F7 misc 剩 6 三微机制归因（普查档对账）或 F6b handoff 4 语料形对齐，按下一轮普查定序。
