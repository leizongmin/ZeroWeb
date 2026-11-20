# d63 · join-white-space 回归回收刀：D3 caret 重映射域 fresh 查询通道

**日期** 2026-11-20 · **目标** [wptfyi-editing](../master.md) · **基线** d62 判决轮（141,068，距线 5,116）
**码面** `crates/engine/src/js_dom_bridge/css_wire.rs` +18 · `crates/engine/src/js_dom_bridge/callbacks.rs` +15 · `crates/engine/src/js_dom_shim/part01.js` +30/−1（首稿记 +9/+31/−1 系注释扩写前口径，落树时以 numstat 勘正）
**性质** d62 定谳的唯一达线路径（d57 刀 C-view 粘性 canonical 对 join-white-space 五文件族 −5,192）的回收刀；方向＝master ③ 的收窄变体：**重映射域 fresh、身份域粘性**，非全量代际分桶。

## 背景

d62 bisect 定谳：d56-clean 3,456/3,456 全绿 → d57-final 749/3,456，selection 套件 −5,192 全落 join-white-space 五文件（−2,707/−1,296/−1,127/−65/−20），失败形＝join 后 innerHTML 读回 pre 态原样未 join（d47 判负指纹逐族吻合）。d57 刀 C-view（css_wire.rs `STICKY_QUERY` 粘性 canonical memo）服务 wrapper 身份稳定（input-events d57–d59 +903 增益域），但其查询落点在 shim D3 世界同步的重映射路径上不再是「当前视图活子树」。

## 诊断链（上会话 429 中断前完成，本会话核账）

1. **单页复现**（`zz-probe-d63-jws` / `zz-probe-d63-jws-b`，gitignored wpt-data editing/other/）：`<div style="white-space:normal">abc</div>` + `<div style="white-space:pre">def</div>` caret backspace join 形。base（d61 终版）页 `EXPECT-JOIN-FORM` Fail——join 后 innerHTML 仍为两 div 原样；探针 `CARET/AFTER-RIGHT` 各步读数正常，毒在重锚产物家族：`scParent` 为粘性缓存壳（setup 代 sel 家族），其 `childNodes` 投影不含 world-sync 后新文本节点，join walk 的 `bc === el` 上行永不命中、前邻找不到即链断静默返回（join 不发生，非 join 错形）。
2. **二分定谳（两刀方向各一负样本）**：
   - **谓词 fresh**（`_zwCeCaretAnchorStale` 也走 fresh）：caret `n` 即活粘性壳时，fresh 落异家族壳 `c !== n` 恒判陈旧 → 活壳被重映射进 fresh 家族、与宿主侧粘性 el/页面壳身份断开——ie 全套 −911（P2F），单页 `zz-probe-d47b` 复现。**谓词必须留粘性**（「同家族壳=活」正是该谓词语义）。
   - **map 臂 fresh**（仅 `_zwCeMapStaleContainer` 走 fresh）：join-white-space 族全回收，ie 全套保位（见下 A/B）。重映射的前提（落点=当前视图活子树）只有 fresh 满足；谓词域（身份判活）只有粘性满足。两域要求相反，一刀分开两通道。

## 刀面（最小三处）

1. `css_wire.rs`：`query_match_selector_doc_fresh(doc, selector)`——跳过粘性 memo 直接 `find_by_selector` + canonical 重算，形态与 [`query_match_selector_doc`] 逐值等价。
2. `callbacks.rs`：注册 `__zw_query_match_fresh` 回调（页面可见 querySelector 仍走 `__zw_query_match` 粘性域零变化）。
3. `part01.js`：`_zwCeFreshQueryWrap(sel)`——命中走 fresh 回调 + `_zwQueryWrapIdentity` 包裹；**miss（sel 已不在引擎当前视图）回落 `document.querySelector(sel)` 原粘性通道**，保 null 语义与 overlay/pending 臂逐值不变（`_zwQueryWrapIdentity('')` 会落空选择器鬼壳，不可作 miss 返回值；宿主缺回调时整体回落原查询）。仅 `_zwCeMapStaleContainer` 消费；谓词 `_zwCeCaretAnchorStale` 留粘性并在原位注记二分依据。

## A/B（fix2 = rebased 树 fffb6c0e4 + 本刀 vs head2 = d4179058a clean；battery v3，test-guard 包裹，`--json` 键级对账）

| 腿 | head2 | fix2 | 判决 |
|---|---|---|---|
| selection `join-different-white-space` 过滤腿（**本轮新增哨兵腿**） | 2,376P/5,142F | **7,518P/0F（rc=0）** | F2P=5,142 **P2F=0，回归全回收** |
| selection 全语料 | 43,862P/9,490F | 49,128P/4,272F | F2P=5,235 / P2F=5（见归因）/ other-flip=0 |
| input-events 全套件 | 4,571P/2,106F | 4,614P/2,063F | F2P=45 / P2F=2 / 净 +43，**d57–d59 +903 增益全保** |
| dom 哨兵 | 57,147P/681F/18T | 同 | 共键 57,761 F2P=0/P2F=0/other=0（new=12/gone=12＝风暴页截断伪影族，d58 同款记档） |
| uievents 哨兵 | — | — | 逐字节恒等（416,587B） |
| selce 哨兵 | 311P/119F | 同 | 逐字节恒等（103,184B） |
| zz-probe-d47b（ie −911 复现页） | 205P/111F | 同 | 0/0（谓词留粘性直接兑现） |
| enginelib（rebased 树 phase2） | — | **4,231P/0F** | rc=0（+1 测试＝并行流入树） |
| clippy / fmt | — | rc=0 / clean | — |

selection F2P=5,235 逐文件：style-paragraphs +2,707、style-left-paragraph-and-right-line +1,308、style-left-line-and-right-paragraph +1,127、join-pre-and-other-block +65、delete-without-unwrapping-first-line-of-child-block +24、本地探针页 +4（zz-probe-d63-jws/-b/d63-fd/lrlp）。五真实文件合计 +5,231，对 d62 账 −5,192 全量回收（d62 数字系两代 coverage 快照差含截断噪声，本 A/B 为同机同语料直算，以本账为准）。input-events F2P=45 逐文件：getTargetRanges-backspace/forwarddelete 各 +7、non-collapsed +3（corpus 面 +17）、探针页 +28。

## P2F 归因（两簇，均披露入账）

1. **selection corpus P2F=5**：`insertparagraph-in-non-splittable-element.html` 的 iframe/meter/progress/select/table 五形——期望 `<div><br></div><div><iframe…></div>`（原壳留占位 br、元素搬入新块），got 元素复制形（旧壳视图仍列元素 + 新块一份）。**刀致**（battery v1 fix 预 rebase 同 P2F=5 实证，非 rebase/miss-fix 引入）。机理与回收面同根：拆分裂变家族把「单家族投影内搬移」的视图一致性需求拉进重映射域——fresh 壳入拆分 walk 后旧侧壳投影不前进，序列化两侧同现。join 族要 fresh 投影、拆分搬移族要单家族视图一致，两需求在 map 臂对撞——与 d47 别名刀两族相反要求同构，归 **master ③ 查询视图代际分桶** 域挂账。
2. **input-events 探针 P2F=2**：`zz-probe-d56j`/`zz-probe-d56jt6` T6 `<div>abc<ul><li>[]def</li></ul>ghi</div>` caret backspace innerHTML 各 1 键（d56b ③ 诊断页族，非 corpus）。

## DC-3 投影

141,068（d62 快照）+ 5,226（selection corpus：+5,231/−5）+ 17（input-events corpus）+ 10（d61 刀 F 未入跑）≈ **146,321 ≥ 146,184**，余量 ~137。CI coverage 不含本地探针页（gitignored），corpus 口径如上。落树后 dispatch `weekly.yml run_wptfyi_coverage` 复判。

## 附注

- 上会话 429 中断点：battery v1（pre-rebase knife）跑至 sel-full-fix 后进程被杀；本会话续 d63b2/d63b3（battery v3 = miss-fix + rebased 树终版）。fix 与 fix2 的 jws/ie 产物逐字节恒等，miss-fix 与 rebase 对 CE 域零扰动。
- 谓词 fresh 负样本（ie −911）与 d47b 单页复现探针配方保留在 gitignored wpt-data（`zz-probe-d47b*`），③ 域重设计时的反证面。
