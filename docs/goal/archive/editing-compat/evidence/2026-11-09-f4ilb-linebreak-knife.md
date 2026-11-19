# F4 ilb 非 collapsed 行断三臂刀（2026-11-09）

mt 残 30F（[普查档](2026-11-09-f1shell-post-knife-30f-census.md) ⑤ 车道）：F4 ilb-first 全族 9（img 幻影 4 + 文本 dup 3 + `<s></s>` 空壳 2），multitest.js 案号 530/532/534/536/540/546/548/554/556。

## 根因链（三臂）

spec insertlinebreak action 首步 `deleteSelection({stripWrappers: false})`（vendored implementation.js :7387）在引擎分派臂缺失——part06 ilb 臂直达 `__zw_ce_line_break_mutate`，跨节点被选内容（img / inline 壳 / 裸文本）全数残留；typing 落壳外裸文本位成 dec/cdec 重复（probe 直调 CASE540 实证被选 "c" 存活）。同节点非 collapsed 面（单文本 `foo[bar]baz` 形）line_break_at 的 boundaryText 头尾切片本就忠实（knife9 语料 1641P 基），delete 前导的 whitespace canonicalize 反有字节漂移风险——刀体收窄为跨节点才走前导。

臂 2/3 在 `_zwCeParagraphMutate` 边界归一链上：① 块元素 caret 臂缺失——deleteSelection 落位 `(div, k)` 形（mt 536/548/556 族）上爬越宿主直子块，br 逸出宿主（got `<br>de<div>…`）；② `_zwCeLineBreakAt` 断点后 caret 落裸元素位，typed 段不进邻接 inline 壳（Blink VisiblePosition AfterNode(br) deep-equivalent = 下游语义归一入 inline 链首文本位；mt 530/546 族 `<b><i>def</i></b>` 形）。语料考古定标：Chrome collapsed 壳内 br 录音（insertlinebreak.js）→ mt br-outside 结果全由 delete+typing 管线解释，无需 hoist 逻辑。

## 刀体（commit 97729759d，14e4f7a11 patch 重放；两 shim + fixture）

- **part06.js ilb 臂重写**：collapsed / 同节点 / 无 delete_apply 三条件让位原径（`__zw_ce_line_break_mutate(host2, rng2)`）；跨节点面处方 c（knife11 v2 ip 臂同构）：`__zw_ce_delete_apply(false)` + null range live 重读 `__zw_ce_line_break_mutate(host2, null)`。
- **part01.js 补臂 1**（边界归一链 ③ 位，lineBreakOnly 门）：块元素 caret（`sc !== host && _zwIsBlockTagName(sc.tagName)`）重映射 `boundaryParent=sc, boundaryNode=childNodes[so], boundaryOff=so`——br 插块内子 offset 位不出块。`_zwCeIsBlockTagName` 表 div/p/h1-h6/li（td/tr 不在→wss 表格 caret 不重映射；⓪/①/② ip 臂 boundary 语义不动）。
- **part01.js 补臂 2**（`_zwCeLineBreakAt` 断点后）：`childNodes[idx3+1]` 为元素时沿 inline 链下钻（深度 16 门、块界不下钻）首文本位重设 caret；文本首子 / 断器位保持原 caret。
- fixture `zz-probe-f4ilb.html`（add -f 入库）：9 目标案（multitest ilb-first 全族原形）+ 2 collapsed 护栏（bold+ilb、bold+ilb+typing carrier）。faithful 相 runConformanceTest = 断言权威；直调相 per-step trace 定位 ilb 后 caret 与树中间态，CASE530 直调选区寻址伪影（img 跨界 + 文本内端 offset，直调 addRange 后 caret 溢宿主树落探针自身 script 文本；mt harness 同案绿）trace 不判断言面。
- md5 锚（批硬绑定八点）：part01=`f283e81b` part01c=`fb8aa0d8` part01d=`9fbab2af` part01f=`f5e149d2` part01h=`9592ebd3` part06=`e32619e9` bridge=`f7d65f32` fixture=`0fe3b405`。

probe 迭代五轮：r1 直调相暴露三臂缺口（块 caret 逸出 + typed 裸落）；r2 补两臂后 8/9；r3 发现 CASE530 直调伪影（faithful 同案绿）改 traceOnly；r5 收窄 scope（同节点让位原径）后 **99P/0F**（11 权威案 + 直调 10 断言 + summary 恒绿）。

## 机制预测

- mt assertion/entry 级 F2P = 恰 9（ilb-first 全族 9 键；同节点语料让位原径不翻）；残 30→21F。
- ilb 单命令语料域内翻转 = 跨节点残键 17（ilb-mid 链中 24 条全裸文本尾中的跨节点形）；P2F=0。
- 翻键窗口：entry 级 [9,18]（双页折叠）；calibrate 白名单 = `[["insertlinebreak",""],...]` 签名 × 双页。

## 门禁批判读（batch-f4ilb-gate-v2.sh，06:50:38 发射 head=14e4f7a11 → 07:49:54 ALL-DONE，59min）

批协调事件：v1 批 06:44:30 发射（head=d3299255e，rebase 前锚）前 8 腿绿后，开工 pull --rebase 并入并行流 6335a60af（s357 script 加载语义，正交域）——恰改 part01.js（八点锚之一），批中树漂移 + mt 后 md5 复读门必 ABORT + 后续腿构建成混合二进制，按红线停批整跑（kill 进程组 + detached runner 残留补杀）；v1 结果作废。v2 重锚 14e4f7a11 重发射（part01 锚更新 f283e81b，其余七点实测不变；特征串门 9 项新树精确匹配）。

锚复读 OK（md5 八点 + 特征串：06 del=9 ilb=6 | 01 blk=9 drill=2 lba=3 con=4 walk_px=5 post_insert=5 | 01f cap=2）；二进制身份（learning #28）：`块 caret 归一补臂` / `断点后 caret 下游归一` / `处方 c（knife11 v2 同构` 三串 grep 全在产物内；mt 后八点 md5 复读门过。

| 腿 | 结果 | 判读 |
|---|---|---|
| probe-f4ilb | 99/0 rc=0 | 刀 fixture 全绿 |
| probe-f1shell / f1b / f2 / f3 / f6b | 27/0 / 7/0 / 14/0 / 7/0 / 17/0 | 回归护栏五腿恒一 |
| probe-k11 | 224/17，rows 229 恒一 judge OK | 大探针零漂移 |
| ilb 语料 | vs knife9-ilb-post：rows 1691 恒一，P 1641→1658 / F 50→33，F2P 17 P2F 0 post-only 0 lost 0 | 域内翻转 17 跨节点残键；同节点零漂移 |
| ip / ihtml | rows 7195 / 1565 恒一，F 206 / 119 恒一 OK | 零漂移 |
| exec×3 | rows 1806/1/6 恒一 OK×3 | 零漂移 |
| **mt judge** | **rows 10092 恒一，P 10062→10071 / F 30→21，F2P=9 P2F=0 post-only 0 lost 0 OK** | **机制预期恰值命中** |
| mt calibrate | entry 翻键 18=9 键×双页，ilb 链白名单 18/18，**白名单外 0** | 域外零翻转 |
| iol / iul | rows 1761/2065 恒一，F 2/4 恒一 OK | 零漂移 |
| enginelib | 3816P/7F（13.18s，rc=101 计数惯例），ROSTER OK(7-red 含 r2967 外部债) | 名册恒一 |
| battery 七腿 | 420/880/1104/552/3456/2607/1455 全 F=0 T=0 | ambient 零扰动 |

## 残账与下一步

mt 残 21F：F7 misc 12（delete-merge-div 6 = 尾残 br 4 + 空壳并存 2 两微机制，归因材料见 ACTIVE-BATCH 2026-11-09 06:38 条；ip-placeholder 4 / host-div-wrap 1 / trailing-empty-p 1 分治）+ F6b handoff 4（N1/N2/N4/N5 backlog）+ F6a 2 + F1' fontsize(4) 1 + X37 2（除名）。DC-4 降序下一刀目标 = ⑥ **F7 misc 12 分治**。
