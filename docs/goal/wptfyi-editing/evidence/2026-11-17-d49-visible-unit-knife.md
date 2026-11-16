# d49 可见单元删除核 A/B 证据（2026-11-17）

## 刀面

part01.js +763/−39（工作区，基于 70b18f81e）：

1. BS/FD 塌缩界可见单元臂（d49 ①(b)）：walk 臂 img/hr 原子消费、行中 br
   消费+次行前导折叠空白吸收、行尾 br 探后并（先探后删镜像）、inline 壳端点
   叶消费+空壳剪枝、宿主缘 N 派发；元素锚臂 img 后退消费、可视空块整块摘除
   （原子块邻 no-op）、表格单元/caption 缘 N 派发、可视块行尾 br 消费+块界
   join（FD）。
2. walk 臂 ws 头/尾进入门：`   []def` / `abc[]   ` 形 [0..so)/[so..len) 全折
   叠空白 normal 域进入（preserve/pre-line 不入，语料 pre 形保持文内臂）。
3. E 族 pre-line `\n` 组：BS 摘末组（`abc\n []def` 界 (3,5)、双 `\n` 界
   (5,8)）+ 派发后重读镜像（zz-probe-d49e E3/E4 实证重读覆盖组端时组形退单
   字符）；FD 前向组臂（语料 FD caret 恒组首，后向组扩展零语料价值未做）。
4. B 族壳内整节点：gtr 双端升壳父界（`a<span>b[]</span>c` 界 (p,1)-(p,2)
   形）+ 叶尽空壳向上剪枝。
5. C 族 join 缝规范化（C-strip，JoinMutate）：跨容器块 join 且双侧 normal 域
   时摘 dst 深末叶尾随折叠空白 + 源行首文本前导折叠空白（FD/BS 双几何共用；
   jdwsp 左块 4,312P 反证面由 preserve/preLine 守卫挡）。
6. select-delete 镜像形排除（sc 在块内、ec 在宿主级时 ⑤ 核拒收降级，d43
   P2F 修复域细化）。

## 三轮修复（同会话闭环，均为专项 A/B 驱动）

| 轮 | 症状 | 根因 | 修法 | 实证 |
|---|---|---|---|---|
| 1 | BS 语料 3 测 P2F：`<div><p>abc</p>[]def</div>` 系（innerHTML 期望吸收 got 原样） | 新单元臂 catch-all（12988 NopDispatch+return）吞掉 `bn===sc && bp=块元素` 形——块形到不了 12993 块 join；base 该形直达块 join（引擎规范化 (div,1)→(t"def",0) 后 walk 臂吸收，bi=0） | 单元臂门加 `!_zwCeIsBlockEl(bp)`，块形归块 join（与 FD 镜像一致，FD 侧无 catch-all 天然无此病） | zz-probe-d49s 隔离页：base 吸收 bi=0、修前 wip 全静 bi=0 DOM 冻结、修后 `<div><p>abcdef</p></div>` caret (p,1) |
| 2 | FD 语料 1 测 P2F：`abc[]<span contenteditable="false">def</span>ghi` innerHTML 期望 no-op got 摘 'd' | 新 inline 壳臂无 ce=false 守卫——不可编辑壳被当可编辑壳消费（base 走 inline-return 引擎 no-op） | helper `_zwCeInCeFalseSubtree(n, stopAt)`（self-or-ancestor contenteditable=false 遍历，宿主止）加 BS/FD 单元臂门各一合取 | FD 专项 A/B 该测恢复 base 形；BS 镜像同门预防 |
| 3 | BS 语料 2 测 P2F：`<ol/ul><li>list-item1<br><br></li><li>[]list-item2</li>` gtr 期望 (li,1) got (li,2) | 本轮「尾随 br 串取最后一枚」改动引的是 P 形契约 (p1,2)，而 li 形语料契约 (li,1)——**语料 p/li gtr 起点契约分歧，单 picker 不可两全**；且 P 形在双侧同红（(p1,0)，join 缝消费未达），改动属净亏损 | 回退 `_tbBr` 取首枚（break 恢复） | li 双 br 隔离探针修后 gtr (li,1)-(...) 与 base 恒等；P 形 base=wip 同红维持 |

## 权威账（专项单文件 A/B，base=/tmp/d49-base-tgt（=70b18f81e），final=本刀）

| 文件 | base | final | 净 | F2P | P2F |
|---|---|---|---|---|---|
| input-events-get-target-ranges-backspace.tentative | 44P/163 | 78P/163 | +34 | 34 | 0 |
| input-events-get-target-ranges-forwarddelete.tentative | 55P/176 | 96P/176 | +41 | 41 | 0 |
| input-events-get-target-ranges-joining-dl-elements | 735P | 751P（共键 F2P=0/P2F=0） | 0（差为 base 腿截断键） | 0 | 0 |

**净 +75（F2P 75 / P2F 0）**。套件级 1398→1518P（+120）含 zz-probe 本地
探针文件自增（d49e +30、b3r +34、d46 系 +12、d49p +3）与并行腿截断噪声
（joining-dl base 腿 0 键、b3s/d49e 计数漂）——不作账。**方法论：ledger 级
A/B 用专项单文件双腿，套件级只作粗核。**

## 哨兵

- enginelib：**4099P/0F**（cargo test -p zero-engine --lib，test-guard
  --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 1500）
- html-semantics：共键 F2P=0/P2F=0（base 5453 键/wip 5450 键，尾差为截断）
- selection（contenteditable 过滤）：214P 恒等，F2P=0/P2F=0
- 全 selection 套件 >520s 被守卫截——CE 键改动与 Selection API 无交，过滤腿
  足够；本轮未跑全量

## 停车场（本轮新增证据，未修）

1. **文件内跨测引擎池 rebake 毒**（探针自造形，语料不受影响）：同文件先
   FD elem-join（C4 b/b）后 BS ws-seam join（C6）时，C6 的 gtr 混陈旧锚
   `(p,0)-(#text "   def",3)`、post 树复现旧池节点（zz-probe-d49r：C6s
   PREINIT 全新 p#8/p#10 → POST `p#2{t#12}`——p#2 为 C4/C6r 时代旧对象）、
   C-strip 被跳（`<p>abc   def</p>` 不摘）。宿主/祖先 style+class 全 null
   （CHAIN dump）→ `_stampWs` 残留假设证伪；`_zwCeWsOf` 无缓存 → 毒必在
   引擎侧（innerHTML 换代后旧池对象被 key 默认动作 rebake 复插）。语料 BS
   文件无 FD key、FD 文件无后续 BS ws-seam 形 → 零语料命中。解域归 master
   ③ generation-aware 归一刀。探针组：d49r（属性链+树身份）、d49s（J1
   隔离）、d49q（双测毒序 bisect）。
2. **pre 双 br p 形**：`<p>abc<br><br></p><p>[]def</p>` 契约 gtr (p1,2)+
   content `abc<br>def`，base/wip 同红 gtr (p1,0)+双 br 全留——join 缝 br
   消费未达 p 形（li 形 d44 缝消费达一枚）。base 同红非本刀回归。
3. 既有红维持（前轮已停）：N3 elem-anchor img nobi、I1 hr 臂 nobi、C5 族
   i/b join 引擎不可见插入。

## 运行产物

- /tmp/d49-bs-{base,final}.json、/tmp/d49-fd-{base,final}.json（专项双腿）
- /tmp/d49-suite-{base,wip}.json（粗核，噪声见上）
- /tmp/d49-jdl-{base,final}.json（joining-dl 共键核）
- /tmp/d49-selce-{base,wip}.json（selection-CE 过滤腿）
- zz-probe-d49{r,s,t}.html（毒/隔离/li 双 br 探针，wpt-data 本地不入树）
