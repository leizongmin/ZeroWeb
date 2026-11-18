# d54 跨容器 select-delete 摘空壳剪枝分形刀 A/B 证据（2026-11-18）

## 码面（part01.js +101/−27，git numstat）

1. **`_zwCeDeepEmpty` 深摘空判定**（joining-dl `[` 起始 selection 语料定谳）：
   子树无可见内容——折叠空白文本子与「自身深摘空」的元素子链（dl>dt、
   ul>li 空壳链）不计可见；br/img/hr/input 等可见空元素计可见（br 占位语义，
   与旧直子判空口径一致，防占位壳被剪）；depth>8 保守 false 防病态嵌套。
2. **d54 序修正（join 臂右剪枝）**：右链剪枝捕获点 `_sdRtPrune` 提前到 join
   变异前——join/take-line 摘壳后 `rightBlock.parentNode` 恒 null，旧尾置
   捕获使右链剪枝对 `list-item1[ + list-item2]` 形从未生效（dl2 空壳残留
   zz-probe-d54c 实证）。判空换深摘空（dt2 摘后 dl2 整剪；1_2 形 dt3 持位
   自止）。
3. **B2 分形（左侧无残余）**：旧码序 bug——`leftBlock.parentNode.
   removeChild` 后取 `.parentNode` 恒 null，左向上剪枝从未生效。按 ec 锚形
   两分（键入替换面维持旧单摘不入两分）：
   - ① ec 元素锚（`}list-item2` 形）→ 容器并界：摘空左块移除＋左向上深摘
     空剪枝（dl1 随 dt1 摘除，期望 `<dl><dt>list-item2</dt></dl>` 单容器）
     ＋右容器链中位深摘空 dt/dd/li 清理（`}list-item3` 形 dt2 摘除、dt3
     持位自止）。
   - ② ec 文本锚（`list-item2]` 形）→ 左摘空壳保位＋占位 br（Chrome 形
     `<dl><dt><br></dt></dl>`，dt1 不剪）；右深摘空尾链剪除（1_1 全摘形
     dl2 整剪；1_2 形 dl2 因 dt3 持位自止、仅 dt2 摘）。
   - ③ typingData 路径维持旧单摘行为（无链剪枝、无占位——插入随即填块）。
4. 左残余 join 臂剪枝循环同换 `_sdRtPrune`＋`_zwCeDeepEmpty`（序修正 2 的
   捕获点与判据同步）。

## 根因链（zz-probe-d54c 三 MISS 形定谳）

1. `<dl><dt>list-item1[</dt></dl><dl><dt>}list-item2</dt></dl>`（元素锚）：
   wip 残 `<dl><dt>list-item1list-item2</dt></dl><dl></dl>`——dl2 空壳。根因
   join 后 rightBlock 脱树，右链剪枝链首 detached 恒 null 止（序 bug）。
2. `<dl><dt>[list-item1</dt></dl><dl><dt>}list-item2</dt></dl>`（元素锚，
   左整体摘）：wip 残 `<dl></dl><dl><dt>list-item2</dt></dl>`——dl1 空壳。
   根因左剪枝序 bug（先 removeChild 后取 parentNode 恒 null，循环首轮即止）。
3. `<dl><dt>[list-item1</dt></dl><dl><dt>list-item2]</dt></dl>`（文本锚，
   全摘）：期望 `<dl><dt><br></dt></dl>`，wip 残双空壳——左保位占位面＋右
   尾链剪除面双缺。
4. 探针修后（_sdRtPrune 前置＋两分落地）三形全 IN-FORMS；caret join 形
   （`[]` 密位）不受影响（既有全摘形走 join 臂非 B2 分形）。

## 权威账（专项 A/B，base=/tmp/d54-base-runner＝HEAD 1d5044761 stash roundtrip）

| 腿 | base | wip | 共键 F2P | P2F | new/gone |
|---|---|---|---|---|---|
| joining-dl-elements | 572P/293F (865) | 712P/153F (865) | 140 | 0 | 0/0† |
| joining-dl-elem-another-list | 168P/128F (296) | 216P/80F (296) | 48 | 0 | 0/0 |
| deleting-in-list-items | 304P/960F (1264) | 504P/760F (1264) | 200 | 0 | 0/0 |
| gt-backspace | 81P/82F (163) | 恒等 | 0 | 0 | 0/0 |
| gt-forwarddelete | 100P/76F (176) | 恒等 | 0 | 0 | 0/0 |
| gt-non-collapsed | 110P/97F (207) | 恒等 | 0 | 0 | 0/0 |

† elements 首轮 wip gone=68（全 ？Delete 尾键）复跑（/tmp/d54-jdle-wip2.json）
全回归＝文件级超时吞键族带伪影；合并账 865 键 F2P=140（回归 68 键=9 F2P+
59 同败+0 P2F）。

**刀净 +388（F2P 388 / P2F 0）**。F2P 面全为跨容器选区残壳形：dil 四 variant
均匀（Delete,ul 57/Delete,ol 57/Backspace,ul 43/Backspace,ol 43，嵌套
ul/ol 形深摘空链剪枝直接清偿）；elements Delete 75+Backspace 65；jdla
Delete 28+Backspace 20。base 败因消息与三修复臂逐一对应（断言
assert_in_array/equals 摘录）：
`"<dl></dl><dl><dt>list-item2</dt></dl>" not in ["<dl><dt>list-item2</dt></dl>",…]`
（①左并界剪除）、`"<dl><dt>list-item1</dt></dl><dl></dl>" not in
["<dl><dt>list-item1</dt></dl>",…]`（②join 臂右链剪除）、
`expected "<dl><dt><br></dt></dl>" but got "<dl></dl><dl><dt></dt></dl>"`
（③文本锚保位占位）、`"<dl><dt>list-item1</dt><dt>list-item3</dt></dl>
<dl></dl>" not in […单容器形]`（④右容器链中位 dt2 清理）——wip 全数
IN-FORMS 翻 P。

## 哨兵

- input-events 全套件（完整轮 ~8min×2）：common 6339 **F2P=584/P2F=0**，
  gone=84（dil 文件级超时吞键族带伪影，专项 dil gone=0 为准）、new=1（base
  吞键伪影）。F2P 分解：dil 145＋elements 142＋jdla 48（与专项精确吻合）
  ＋zz-probe 面 249（b3s/d53g/d47b/d53k 等历轮探针同向翻转）。
- enginelib：**4165P/1F**（test-guard --compile-first --per-proc-mem 4
  --total-mem 8 --time-limit 1500）。1F =
  `js_dom_bridge::tests::test_get_computed_style_font_shorthand_r2761`
  （font 简写序列化：`font-stretch` computed=75% 过、`font` 简写串
  `condensed` 关键字 vs 期望 `75%`）——**stash roundtrip 定谳 HEAD 既有红
  非 d54 引入**（computed_style_property 为纯 Rust 参考实现，零 shim 参与；
  断言 2026-08-12 f4de464fd 即在）。归因：并行流 03:59–04:18 落窗（刀 P
  typed-om 门表扩/动画 transitionable 域）引入，font 简写域工作面不归本流，
  已记控制面卡点。测试数 4137→4166 = R18 part139 pin 文件 +29 落窗自增。
- selection corpus（53,316 共键全量）：base 49,025P → wip 49,025P
  **F2P=0/P2F=0/other-flip=0**；gone=new=1＝selection-nested-video.html
  同文件名漂移非码面（d53 同谱）。select-delete 臂对 selection 面零扰动
  （d43 先例 F2P=+10 本轮未现——d54 剪枝域全在 input-events CE 面）。
- editing/run（99k 键，base/wip 全量，8 分片 ×2 侧 2026-11-18 补齐）：
  common 91,635 键 **F2P=0/P2F=0 净零**，双侧 P/F 恒等（各 90,671P/964F）
  ——select-delete 臂剪枝对 execCommand 语料零扰动，与 selection 哨兵同向
  互证。单向缺键 base=170/wip=158（长文件 delete/forwarddelete/
  insertparagraph 等分片尾部枚举截断噪声，含 base 侧 7F+2T/wip 侧 1T，
  共键零位移结论不受影响）。

## 残留与归域

1. **r2761 HEAD 已知红**：enginelib 唯一失败，font 简写域（font-stretch
   关键字/百分比序列化不一致）。影响后续所有流 enginelib 哨兵基线（含
   `make test` 中止语义）。归并行流 font 简写刀面，本流不越界。
2. **③ generation-aware 域不动**：backspace/fd 主文件 got2/got0 交错探针
   定谳属派发计数域（d42/d47/d50 同域在册），本刀零涉入（两主文件专项
   A/B 恒等）。
3. **runner 文件级超时吞键族带**：suite 轮 dil gone=84 复现为伪影（专项
   全键集 gone=0 为准），与 d53 残留 3 同谱。
4. ①(b) 跨元素边界 caret join 与 ①(c) `{<hr>}` 核拒收仍挂（③ 域前置）；
   ④ B3 text-control 期望矩阵待排。

## 运行产物

- /tmp/d54-{joining-dl-elements,jdla,deleting-in-list-items}-base.json ＋
  同名 -wip.json；/tmp/d54-jdle-wip2.json（gone 复跑确认轮）
- /tmp/d54-get-target-ranges-{backspace,forwarddelete,
  non-collapsed-selection}-{base,wip}.json
- /tmp/d54-suite-{base,wip}.json（全套件完整轮）、/tmp/d54-enginelib.log
- /tmp/d54-selection-{base,wip}.json
- editing/run 回填腿：/tmp/d54-er-sh{1..8}-{base,wip}.json（8 分片）＋
  /tmp/d54-er2-status.txt；记账脚本 /tmp/d54-er-merge.py
  （common 91,635 键 F2P=0/P2F=0，双侧 90,671P/964F 恒等）
- 二进制：/tmp/d54-{base,wip}-runner（base=HEAD stash roundtrip）
- 探针：zz-probe-d54{a,b,c}.html（wpt-data 本地不入树；a=计数面、b=派发
  链诊断、c=dl selection 形 IN-FORMS 判定）
