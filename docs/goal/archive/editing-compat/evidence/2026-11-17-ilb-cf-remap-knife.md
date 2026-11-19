# ilb C+F 外迁族刀收口（ILB-DTP-REMAP，2026-11-17）

刀体 3312189d7（`fix(editing): ilb C+F 外迁族刀——dt/dd/pre 元素 caret 行断 br
落容器内不外迁`）＋门禁批 i7c0 ALL-DONE（head=3312189d7，bin
4d6f37ed7b69d43f9ae6ae5e074386b0，2026-11-17 01:09 完走）。ilb 33F census（
[2026-11-16-ilb-33f-census.md](2026-11-16-ilb-33f-census.md)）刀序 ① 兑现，
F2P 恰 6 键零新增红。

## 刀前基线（i7b0 fresh 复核腿）

census 数据源 i5p1（head=76918f601）早于 ihe1–ihe6 五刀，按协议现树 fresh 腿
复核：批 zw-i7b0（head=8fa693ee7，bin 3b49d237f1ac1f49fa540fe8abf47556）——
release runner 复建 47s＋ilb 腿 40s，**1658P/33F/0T**，33 键集合 vs i5p1
census 逐字节恒一（KEYSET-IDENTICAL）。ilb 面零位移定谳，刀序 ① 可落。

## 根因与修复面

`__zw_ce_line_break_mutate` → `_zwCeParagraphMutate` lineBreakOnly 块 caret
重映射臂谓词用共享窄表 `_zwCeIsBlockTagName`（div/p/h1-6/li），dt/dd/pre 不在
表内——dt/dd/pre 元素 caret（`<dl><dt>{}<br></dt>` 形）原走 (父, sc) 臂把 br
推出容器外（got `<dl><br><dt><br></dt>` 外迁形）。

修复：谓词扩 `lbTag === 'dt' | 'dd' | 'pre'`（part01.js 局部 `lbTag` 变量），
不扩共享窄表——ip 拆分边界语义不动（C4 刀注）。spec implementation.js :7383
insertlinebreak：br 插 active range start 所在元素内 childNodes[so] 前，
dt/dd/pre 同 div/p 为 br 合法子容器，落位不外迁。

影响面收口：`__zw_ce_line_break_mutate` 全调用点恰 2 处全在 part06 ilb 臂；
`__zw_ce_enter` 自带直插逻辑不经 `_zwCeParagraphMutate`，Enter 路径零影响。
文本 caret 案走 isText 臂，零 P2F 面。

## 钉测（part126，同提交常驻）

`test_ilb_dt_dd_pre_pins`：6 语料形锚 corpus idx 48/61-65 期望（compare
innerHTML 剥 marker 不锁 caret，setupDiv 同款）＋li 现役绿形控制键（证谓词
扩展对既有臂零扰动）。定向跑 1 passed（test-guard 包裹）。

## 门禁批 i7c0（刀后 ALL-DONE）

| 腿 | 门 | 结果 |
|---|---|---|
| L0 runner 复建 | bin 锚＋刀体特征串 | bin=4d6f37ed… ksig=1 head=3312189d7 |
| L1/L2 ilb 双腿 | 1664P/27F/0T | 1664P/27F/0T 双腿同数（106s/54s） |
| L3 键集 judge | 零新增红＋F2P 恰 6 刀键 | F2P-EXACT-6（6 白名单键同配方提取逐字节恒一） |
| L4 enginelib strict | 0F | **4077P/0F**（含 part126 新钉） |
| L5 mt 腿 | 残红恰 X37 双键恒一 | **10090P/2F/0T** 残红集 vs ihe6-mt 基线逐字节恒一，零新增红（P 留痕不作门——无 post-d46 mt 基线锚） |

mt P=10090 与 ihe6 基线同数：multitest.html 面不含 ilb 语料键计数位移，
6 刀键翻绿不在此面计数控域内；残红集恒一即零新增红充分。

## 尾巴

ilb 残 27F＝census 刀序 ②A 壳尾计数 5 → ③H+J 归一 5 → ④I flex/grid 4 →
⑤B+G+E 选区删除域 8 → ⑥D rawtext 2（+3 待归域 E1 等）。批工件
`.acceptance/dps/i7b0-*` / `i7c0-*`。
