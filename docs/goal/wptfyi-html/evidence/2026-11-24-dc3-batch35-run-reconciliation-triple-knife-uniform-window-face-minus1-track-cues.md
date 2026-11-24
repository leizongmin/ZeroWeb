# DC-3 · 批三十五：run 45280793303 收账——三码刀均匀窗（72957ce1e＋587250e6e＋7f8df21ce 全分片携带）＝三刀归因窗兑现（domparsing +170／sanitizer +80／web-components +90 精确）＝face −1 全落 track-cues 交替带＝停摆形第 12 采样交替互补＝css 负腿三读收口 timing 定谳＝布防全绿第五窗

2026-11-24。run 45280793303（11-23 22:33:22Z dispatch，00:31:44Z completed
success，历时 ~1h58m＝回到历史 2h2m 带内；快照 coverage-2026-11-24 新日首写
＝00:31:38Z 生成、00:31:40Z 落 3842330c7，manifest 063eb7cae2→**e00a7b14e8
换代＝零漂移第七窗破（第八窗漂移，另报不入账）**）对批三十四 run
45253935330 同分片逐键 A/B。下载布防**全绿第五窗**：waiter 补 cd 重挂后
poll1 `queued` 通过（布防新课第三条落地）→ autodl 首轮 163/167（4 件
fail：css-v2-face-d／editing-run-6／catch-all-9／form-encode-zh）→
retrydl 第二轮补齐 flattened 167/167（00:41:05Z）＝韧形下载第二演。
前 session 429 断线（00:49Z）发生在下载＋shas＋A/B 之后、收账落笔之前，
本轮断点续收零损失。

## 窗口码图＝三码刀均匀窗（批三十四预布双刀归因读到期＋新识别刀①）

55 提取＝**41×3897181f4（dispatch HEAD）＋13×c5c49ec0（批三十五布防
docs，dispatch 后 4m20s 落）＋1×2f4a05be5（aggregate 唯一，00:28:01Z 落）**：
- c5c49ec0 对 3897181f4 互 diff 全 docs（3 文件 84+/41−）＝13 docs-ahead
  分片按等码性归并＝套件分片同码窗；
- **三码刀全在 dispatch HEAD 祖先链＝全分片携带＝均匀码窗**：
  ① 72957ce1e（DC-2 切片 8 stream 残余 JS 面，domparsing +118P 自报）；
  ② 587250e6e（切片 8b sanitizer 消毒管路，+52P 自报）；
  ③ **7f8df21ce（切片 9 registries 刀①，scoped CE registries 基础面，
  WC +90P 自报）＝新识别**（落批三十四 aggregate checkout 后、批三十四
  收账提交前，批三十四窗码图未及记录）；
- dispatch 后 main 落 **43ca50c9d**（css columns 刀，wptfyi-css 流，
  23:01:02Z＝dispatch 后 27m40s＝全部分片起跑后）**唯 aggregate 捡到**
  （2f4a05be5 祖先链含，套件产物零触碰）＝批三十一同形；aggregate 只聚合
  不跑套件＝快照 face 不受影响（css 面 A/B 无 columns 动静与此一致）；
- run 完成后落 d4bd1cec1（00:34:21Z，registries 刀②）＝**批三十六窗
  全分片携带预布＝刀②归因窗**。

## 目标级账＝face −1（全落 flake 带）

html 面 109,427 → **109,426（−1）＝净 +38,330/基线 71,096，验收
≥152,096 余 42,670**。−1 唯一构成（coverage 口径）：html/semantics 组
10,902→10,901＝**track-cues-sorted-before-dispatch Pass→Fail＝交替带
第十三演**（b32–b34 三连 P 后破，带形仅记不追）。dialog-overlay 1/1
持位零变化键（震荡带例行）；onblur 3P 持位两连（零变化键，水位带例行）。
三刀域全在 html/ 路径外（domparsing/／sanitizer-api/／custom-elements/）
＝face 不受三刀影响＝口径自洽。

full_pass 文件 63,642→63,640（−2）；covered 74,189 恒；分母 2,279,168→
2,279,165（−3）＝manifest 换代漂移另报；本地旧 cache 复现 face 行
**109,426 精确双证**（zw_passes 1,928,825 精确同、本地冻结分母 2,278,580）
＋护栏未触发（rc=0）。

## 三刀归因读数（批三十四预布兑现）

- **72957ce1e → domparsing 1,094→1,264（+170，subs 恒 1,536）**＝F→P
  翻转面 5 键：stream-html-sanitizer 84→204（+120）／stream-insertion-logic
  28→52（+24）／stream-positional 46→64（+18）／stream-trusted-types-pass
  0→6（+6）／stream-sanitizer-safe 28→30（+2）。CI 高于自报 +118＝环境差
  如实记（批三十三 domparsing 自报 +297 vs CI +923 同向先例）；
- **587250e6e → sanitizer-api 334→414（+80，subs +98）**＝7 键：
  basic-filtering 44→98（+54，**regs 64→162 注册扩张 +98 全落此键**）／
  sethtml-tree-construction 63→71／config 47→53／parseHTML 2→6／
  modifiers 5→9／get 7→9／names 13→15；
- **7f8df21ce → web-components 17,577→17,667（+90，subs 恒）＝自报逐字
  精确兑现**＝15 键全 custom-elements/registries/*（主 0→N 形：scoped-
  registry-initialize +26／ShadowRoot-init +11／effective-global-registry
  +10／initialize-upgrades +8／Element-customElementRegistry +6／
  Document-createElement +5／createElementNS +5／customelementregistry-
  attribute +5／initialize +4 等）＝刀① 在已注册面上 F→P 翻转；
- domparsing/san 新水位基线重排 **1,264/414**（残 F+TO 272/343）。

## A/B 套件面总账＝+425 两口径各自闭合

套件面总 P **1,946,755→1,947,180（+425）**；coverage 分子 zw_passes
1,928,572→1,928,825（**+253**）。闭合：aligned 清单内 **+317**＋清单外
**+108**（domparsing tentative 5 键 +170／editing/run 查询键 −63／
idbcursor −1／sanitizer-names +2）＝A/B 口径；分子 +253 与 in-manifest
+317 差 −64＝子测试名级交集差（basic-filtering 注册扩张 98 超出 aligned
名单部分不入分子）＝批三十三两口径各自闭合同形，如实记。

13 套件 43 变化键，组行：**domparsing +170／sanitizer +80／web-components
+90／dom +87（Range-mutations dataChange +53＋replaceData +34，注册恢复）／
selection +44／css +19（animations +18＋writing-modes +3＋backgrounds −1＋
values −1＋forms ±1 镜像净 0）／editing −63（er0 −57＋er2 −6）／indexeddb
−1／media −1＝+425 精确闭合**。

## 停摆形第 12 采样＝交替互补形续（双带内，刀域无关）

- **dom/ranges 42,104→42,191（+87，带内 42,022~42,677）**＝b34 −424
  注册截断的恢复形（regs 42,395→42,482 同步）；
- **editing/run 95,350→95,287（−63，带内 95,130~95,442）**＝注册截断形
  （regs 96,533→96,470 同步；multitest 1092→1035/1036、delete 699→692）。
  三窗交替互补（b33 dom +403/editing 持→b34 dom −424/editing +36→b35
  dom +87/editing −63）＝timing 形续证；三刀域（stream/sanitizer/
  registries）与 Range-mutations/multitest 域无关＝非码归因。
- **editing delete 第三读**：632→626＝带内注册截断伴生（regs −7 同步）
  ＝非码回吐，a2295242d caret 域正向定谳维持；批三十四预布「先验
  aggregate-only 形」兑现＝本窗三刀全分片携带，delete 变化与注册同步
  ＝timing 形读数。

## css 负腿三读＝回位兑现定谳收口

Element-getAnimations 1→17 **精确回位**＋event-order regs 1→4 精确回位
＝b33 两形完整回位＝**timing 形定谳，be78d24cf 嫌疑卸除**；event-dispatch
regs 17→19 再扩（+2P）＋corner-shape 零变化持 145＝wobble 尾巴续（例行）；
css color 第七窗零键变化（3,753 持位）；css-forms checkbox/radio ±1
精确镜像（同名子测试 Pass↔Fail）＝timing。

## 观察清单

- **selection +44 单窗不定谳**：plaintext-only/nested 0→44（regs 56 恒，
  F→P 翻转）＝三刀域无关＝timing 带候选，二读 b36；selection regs 定谳键
  零变化＝12 持位。
- **idbcursor 交替带二读**：1→0（Pass→None＋None→Timeout）＝b34 满额
  回吐，仅记。
- **负向两形第九窗零回位**：test262 三套件＋csp 零键变化（对端未修）。
- **TT 四读**：trusted-types 1,482 持位（零键变化）。
- **CE·SD 第十一窗持位**：idl-setters 2910 regs 225P/2,685F 全等；
  DSD-opt-in 零变化键＝108 持位。
- **dat 170 键 shapes 全同持位**（TO 148/Fail 19/Fail-Pass 3）＝write-
  driver 110 TO 域持位；per-global 6 regs 3P/3F 持位；webappapis 1,540
  持位；web-animations 526 持位（零变化键）。
- 合成 Fail **103=103**（19 套件同值）；variant 三键 DISTINCT 通过；
  interestfor 30 键逐字同。

## 布防

全绿第五窗：waiter cd 课落地（批三十三课只补了 shas.sh 的缺口补齐）；
shas 55 行验收 ✓；A/B `> file` 重定向 ✓；autodl 首轮 4 fail → retrydl
第二轮补齐 167/167＝韧形下载第二演。批三十六起复制验收加 waiter/autodl
自带 cd 检查（布防新课第三条）。运行时长 ~1h58m 回到历史带内。

[逐文件 JSON](2026-11-24-dc3-batch35-run-reconciliation-triple-knife-uniform-window-face-minus1-track-cues.json) ·
[批三十四证据](2026-11-24-dc3-batch34-run-reconciliation-zero-code-window-face-minus3-dialog-break-shape.md)
