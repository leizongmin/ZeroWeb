# ip C4 刀收口——块边界拆分块级兄弟止步 + bare-br 行界双空块物化（corpus 14 键）

- 日期：2026-11-14；刀体 9d587f932（writer 轮落地后 rate-limit 脱离，后继轮核验收口）
- 普查复核：C4 族 17 键/10 fixtures（census 13 族档第 2 位）→ 宿主级 14 键出刀；dt 尾/blockquote 尾/li 尾 3 键归 list/容器链刀域不越界（预期仍 Fail）。

## 刀体（9d587f932）

统一机制 stop-at-block：拆分新块只收 caret 右侧行内连续段，块级后随兄弟（table/h1/p…）留宿主原位。

- `part01.js`（+81/−7）：`_zwCeBlockishForSplit` 帮手；① wrap 臂尾移止步/插入位修正；① 元素边界臂 bare-br 行界窄门——行首裸 br 物化双空块（新块 pad + br 行成块），行中 br 前行内壳存在不开门；stop-at-block + a2HeadEmpty guard 对齐。
- `part01f.js`（+21）：主拆分 host 级 caret 整选删面——extract 收窄首块级相含节点前，空新块 pad（嵌套违规 p>p/div>p 形修复）。
- `part116_c4_block_boundary_pins.rs`（+408）：9 钉测（表跨 dPS 双面/bare-br 尾·首侧·尾随块级/行中窄门 guard/整块选删双面），断言 corpus 实录 exp0 精确形；`js_dom_bridge_tests.rs` include 注册。

## 门禁链（批脚本谱系 `.acceptance/dps/batch-ipm3-c4-gate-v*.log`）

- v1 ABORT：锚头短哈希比较 bug（`9d587f932` ≠ `9d587f932f`），非行为红。
- v2 ABORT：L1 判读基线误用 ipm2-v3（pre-M3-A 99F）——F2P=40 含 26 UNEXPECTED 全为 M3-A 22 br 键 + C5 机械 C4 4 键误判；数据面本身全绿（F 99→59、P2F/lost/newF=0）。
- v3 ALL-DONE 2026-11-14T20:14:48 @9d587f932 bin ce5252d2（锚 comb-r1 post-M3-A 77F 基线 + expected-18 恰等门 + ensure_solo）。

## 四腿终判

| 腿 | 结果 |
|---|---|
| ip fresh | rows 7195 恒一，F 77→**59**，F2P=18（期望 14 目标 + 4 C5 搁置机械 C4＝`{<table>` 整选 ×2 + `{}<br><span display:block>` ×2，恰等缺=0 多=0），P2F=0/lost=0/newF=0，零 T |
| ab | 10092/0/0 严格 |
| mt | rows 10092 恒一，F 2→2，F2P/P2F/po_fail/lost=0，residF 恰 2（X37 维持） |
| enginelib | **3967P/0F**（前基线 7） |

## enginelib 名册 7→0 翻绿归因（非本刀清偿）

名册 7 红（img×2/svg settle/fetch_filtered/sw-iframe/stream/ws×2）翻绿系组合态入站 **1ef32fc4e**（html-semantics 流 s434「桥套件 7 红全树 bisect 归因清偿」）：一案引擎真 bug（SMIL 事件泵 `_scheduleTick` 无宿主定时器环境 `_defer` microtask 自链饿死 `_deferBudget(256)`，后续 deferred 回调含 img/svg load 派发永久丢弃，part09b 双探针守卫根修）+ 四测试前提对齐现行契约；bisect 账本见 [s434 evidence](../../html-semantics-compat/evidence/2026-11-14-m2s434-bridge-7red-bisect-attribution.md)。核验轮独立复跑：HEAD（含 s434+本刀）全 lib `cargo test -p zero-engine --lib` **3967P/0F** @21:19（有并发租户负载下），翻绿稳定非抖动；passed +18 对账闭合 = 9 本刀钉 + 2 s423 钉 + 7 名册翻绿。**后续 gate 名册基线更新 7→0**（新增红仍 ABORT，删除面归因后放行）。

## ip 残 59F 族图（census 13 族档对账闭合）

C4 越界 3（dt/blockquote 尾/li 尾→list/容器链刀域）＋ B 拆点空白/nbsp 归一 15 ＋ C2/F/I6 方向性壳携 16 ＋ C1 容器尾序反转 8 ＋ R1 qV-dPS 8 ＋ C5 2 ＋ C6 2 ＋ C3 2 ＋ I4 2 ＋ I5 1 = 59。

下一步＝**B 族刀**（拆点折叠空格按块边界规则归一 `&nbsp;`：`a[] b`→exp `a`/`&nbsp;b`、`a []b`→exp `a&nbsp;`/`b`、`abc[] `→exp 右块退化 `<div><br></div>`；census 注记与 white-space join 域同源不同位，动刀前先核 join 域回归面）。
