# ihtml G 族 ws-pre 壳刀门禁批收口（ihe1）

**日期**: 2026-11-16
**刀体**: 8ed0e22bd（part06 `_zwInsertHtmlShellResolve` 增 ws-pre 壳臂三规则 + part124 G 形钉 8 枚）
**批**: ihe1 单批一次 ALL-DONE（发射 11:07:24，终态 11:42:58，writer session a82c7d91 于 11:44 429 限额中断，本 evidence 由延续轮按账本协议代跑尾链：判读对原始日志独立复核）
**census**: inserthtml 全页 1524P/**41F** → **1534P/31F**（F2P=10，零新增红）——ihd1 残 41 中 G 族 ws-pre 全 10 键清偿

## 根因（刀前已定谳，见[设计档](2026-11-16-gfamily-wspre-shell-rootcause-knife-design.md)）

harness 宿主为空 contenteditable 包装（include/tests.js），fixture 根 styled div（`white-space:pre/pre-line`）成 host 直子壳 → 内联臂 `sc.parentNode!==host` 全量路由壳域解析 → div ∉ SPLICE 落溶解臂 → 全含门对 collapsed 选区恒 false → null → 整命令 no-op，载荷整丢结构不变（十键实录根因，普查档 2026-11-16-ihc1-residual-49f-census G 族）。

## 刀体内容

`crates/engine/src/js_dom_shim/part06.js` +53：SPLICE 后新分支 ws-pre 壳臂——styled div 壳 × ws:pre/pre-line × 坍缩文本 caret × host 唯一子 × 壳单文本子 × 素净载荷（无 `<`）→ 三规则落位（全隐形前缀整段替换 / pre-line 前置 [ \t] 回退 / pre 保持拼接）+ 原标签字节重建；旁路维持：非 sole-child / 多子壳 / 非 div / 非 ws-pre 族 / 非坍缩 → return null（既有 no-op 不变，零外溢）。`part124_ihtml_pins.rs` +8 G 形钉（四形 × pre/pre-line × 单/双插代表形）。

## ihe1 判读（ALL-DONE 11:42:58 @bin 15053b4d，延续轮原始日志复核 2026-11-16 13:2x）

| 腿 | 结果 | 判定 |
|---|---|---|
| L1 iht A（乘数 2，wall 69s） | rows 1565 恒一，**1534P/31F/0T** | F2P=10 恰 G 族（`white-space:pre` 5 + `pre-line` 5，`abc[]` / `abc []` / `[]` × 单/双插 compare innerHTML 行，comm 对 ihd1-iht-a.log 基线双证）＋零新增红 |
| L2 iht B（wall 11s） | 同上恒一（红集逐键恒等） | 双腿确定性成立 |
| L3 mt（wall 1697s） | **10090P/2F/0T** | 残红恰 subscript+formatblock X37 双键零暴露——mt 语料无 ws-pre 壳坍缩形 |
| L4 enginelib strict（16.2s） | **4060P/0F** | 组合态现树实测（ihd1 时点 4060 持位＋part124 G 钉 8 枚入既有测试函数） |

腿 rc=2 为 make 底层对语料残余红的常态退出码，门=计数＋judge 双证（editing-gate-rc 纪律）。锚定 v2 全程抗性（LAUNCH_HEAD=8ed0e22bd engine 域冻结＋bin 字节 15053b4d 四腿恒一＋docs 轮并行提交不误伤）。

## 残 31 红族谱（41−10 对账闭合，ihd1 族谱基数）

| 族 | 键数 | 处置 |
|---|---|---|
| `{}` 元素边界坍缩选区（占位 br 消费/注释壳） | 12 | **下一刀**（ilb 占位 br 消费先例外推） |
| ws-pre 尾换行 caret 载荷丢失 | 0 | **G 族 10 键本刀清偿** |
| span color 壳＋行内载荷 | 0 | ihd1 已清偿 |
| rawtext 壳（xmp/script） | 4 | rawtext 域维持 |
| C 族全段选区隐形载荷块壳丢失 | 4 | 块臂旁路门核对后续刀 |
| A 族断载荷/退化壳吞并 | 5 | 攒批搭车 |
| E 空行内壳载荷丢弃 2 / F CE-false 邻接 2 / CE=false pre 链 1 / multi-root 1 等 | 6 | 挂账维持（ihc1 census 档案全谱） |

计数 12+4+4+5+2+2+1+1=31 ✓（以 ihc1 census 案号对账，ihd1/ihe1 两刀零新增红保恒等基数）。

## 下一步

- **D 族占位 br 消费刀**（12 键，`{}` 坍缩选区 × 占位 br/注释壳——ilb 占位 br 消费先例外推）；
- multi-root 载荷扩展 1 键搭车；C 族 4 键刀前先核块臂旁路门；
- assert_in_array 9 键收尾批量判读、exec1 474 域决策挂账维持。

## 批脚本与协调

`.acceptance/dps/ihe1-tail.sh`（发射 11:07 冻结）。日志链 `.acceptance/dps/ihe1-{build,iht-a,iht-b,mt,lib,tail}.log`。残 49 全集普查见 [ihc1 残 49F census](2026-11-16-ihc1-residual-49f-census.md)；根因与刀设计见 [G 族根因与刀设计](2026-11-16-gfamily-wspre-shell-rootcause-knife-design.md)。writer session 死亡与尾链代跑按 `.acceptance/dps/COORD.md` 账本协议执行。
