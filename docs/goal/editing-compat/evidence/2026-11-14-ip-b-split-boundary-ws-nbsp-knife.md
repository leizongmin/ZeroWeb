# ip B 族刀收口——拆点块界空白/nbsp 归一（corpus 15 键）

- 日期：2026-11-14 施工／2026-11-15 收口；刀体 2de501560 + 修复链 b1b1e9b9e + 25f6a3943
- 普查复核：B 族 17 键（census 13 族档第 3 位）→ 拆点归一面 15 键出刀；`foo&nbsp;[]<br>` a/b 壳携 2 键归 C2/F/I6 方向性壳携刀域不越界（预期仍 Fail）。

## 刀体（2de501560 + 修复链两笔）

拆点块界空白归一：caret 两侧折叠空白串按块边界规则归一——行界折叠空格翻 `&nbsp;`、whitespace-only 整段退化摘除空块占位 br、单侧退化跳过对侧翻转、新块尾端折叠空白出块留宿主原层、行首空白不包块；white-space 显著面（pre/pre-line/pre-wrap）豁免。

刀体修复链两笔（v4 门禁真回归驱动，corpus 形与钉测形分叉定谳）：

- `b1b1e9b9e`：头侧全空退化判据改调用方侧扫描——v4 P2F=2（`<div>[] abc</div>`/`<div>[]  abc</div>` offset-0 空-头形尾侧 T 翻转误放行，B 钉面缺 offset-0 空-头形）首修；对钉测形（文本直挂宿主）有效、对 corpus 形（宿主>块div>文本）无效。
- `25f6a3943`：v5 复翻勘根——offset 0 拆分在 container 留空串文本左半残根（插桩实录 k0=3:"" containerKids=1），`bSideWsOnlyC` 节点级扫描走 `isWsOnly('')=false` 把全空头侧误判结构性；修复 = 空串文本不破 ws-only（空串全空等价空白），+4 行。

- `part01f.js`（+173）：spec 移植面 `__zw_ce_ipara_apply` 主拆分支钩子（IP-M1 piece2/3 后、双面垫底前）——div 文本位 caret 14 键族主战场；助手本地复制 `_zwIparaIsCollapsibleWsOnly`/`_zwIparaSplitBoundaryNormalize`/`_zwIparaWsDeclaredOf`/`_zwIparaWsSignificantOf`（IIFE 私有作用域不可达，同 `_zwIparaIsVoidTag` 惯例）。
- `part01.js`（+116/−2）：宿主级 ① wrap 臂同规则接线（CE-false 宿主级 caret 形，corpus 唯 1 键）——行界翻转 + 单侧退化 + 尾端出块 + 行首不包块；探针实证 div 键全走 spec 移植面、② sep 臂/m3a 臂不在 B 键路径，零改动。
- `part117_b_split_boundary_ws_pins.rs`（新增 15 钉）＋ `js_dom_bridge_tests.rs` include 注册：② 形 14 钉 + CE-false 宿主级 1 钉，断言 corpus exp 接受形（唯一形或 assert_in_array 落点形，`&nbsp;` 实体 ASCII 形）。

### 归一规则（逐键对 corpus exp 验证）

- 头侧尾端折叠空格（末字符）→ `&nbsp;`；尾侧行首折叠空格（首字符）→ `&nbsp;`、串内 `&nbsp;` → 空格（`a[]  b`→`a&nbsp;`/`&nbsp; b` swap 形实录）。
- 单侧 whitespace-only 整段退化：摘除该侧折叠空白文本、退化侧空块占位 br（`abc[] `→右块 `<div><br></div>`）；单侧退化跳过对侧翻转（`  []abc` 尾侧实文首字不翻）。
- 翻转门 = 双侧非退化 `(H !== '' || T !== '')`——H 空但头侧结构有内容（caret 文本位 0 形）时 T 侧翻转仍执行（key17 实录）。
- `headStructural`：头侧含元素时 ws-only 字符串头不判退化（保 key17 行首空格存留判定）。
- 新块尾端折叠空白文本出块留宿主原层（key17 exp 尾 ` ` 在新块 `</div>` 后实录）；行首空白只排除出 wrap 不重挂（见探针定谳）。

## 探针定谳（两笔）

1. **序列化器无罪**：根层尾 ws-only 文本节点 innerHTML 读回原样保留（setup=读回逐字节等，lastChild=3:" "）——尾空格消失不在序列化层。
2. **静默半挂（DC-4 Q6 s1 第三形）**：经 engine 块两跳摘挂（plain 宿主→engine 块 append→再摘）的原节点直插宿主，`insertBefore` 无异常但父指针已设而 view/childNodes 不动——节点对 childNodes/序列化全不可见（探针 nch=2 实录，尾空格凭空消失根因）。修复 = 重挂一律 `document.createTextNode(node.nodeValue)` 新建（fresh 节点物化先例：bNorm 文本、m3a head1）；行首侧节点从未摘挂，原位即挂载后首块之前，只从 wrap 集合排除、不重挂（重挂会双份空格实录）。沉淀 [learning](../../../learnings/bugs/2026-11/2026-11-14-engine-block-detour-node-remount-silent-half-mount.md)。

## 钉测进展（六轮迭代）

15F（探针接线错臂）→13P/2F（spec 移植面重接后：⑭ 语料保真 3 空格误写、⑮ 门滞）→18P/1F（⑭ 修正 + 翻转门放宽 `(H !== '' || T !== '')`）→**19/19**（行首侧只排除不重挂 + 尾侧 fresh 重挂）。v4 门禁 P2F=2 后补 offset-0 空-头形双钉 ⑯⑰ → 全 ip 钉面 **46/46**（终态）。旁证：C4 钉 9/9（wrap 臂共用面无回归）。

## ilb 域暴露核查（零暴露定谳）

wrap 臂 else 侧 = `!lineBreakOnly && …` 否侧，静态可达 insertLineBreak。ilb 语料实证**零暴露**：CE-false 同字面案（` <span>A</span>[] ; …`）现行 got 为 br 插入形（`<br> ; …` 无块创建）——linebreak 在到达 wrap 臂前已被 br 插入臂拦截返回，wrap 臂对 lineBreakOnly 不可达；div 包裹 ws 案走 div 内部路径不经接线。批后跑 ilb 核验腿（judge vs e1-ilb.log 33F 基线）作硬证据。

## 门禁（gate 链 v1→v6，终判 batch-ipm3-b-gate-v6 ALL-DONE 五腿全绿）

- v1 @21104a331 bin fc571f789：无终态标记（sibling 换锚接替）。
- v2 @a869c9a9e：ABORT-DOMAIN-DRIFT（锚被 rebase 击穿，零腿数据；knife 基体已推 origin 2de501560）。
- v3 @2de501560：受控终止（在飞脚本被 sibling 误改写的字节偏移风险，零腿数据，换新文件名重射）。
- v4 @2de501560 bin e2f304e58：ABORT ip judge 门——F2P=15 恰等（缺=0 多=0）但 **P2F=2 真回归**（`<div>[] abc</div>`/`<div>[]  abc</div>` offset-0 空-头形尾侧 T 翻转误放行；刀体缺陷非判读缺陷非伪影，verifier 记账认定接线作者轮取刀体修复权）。
- v5 @b1b1e9b9e bin 2960f37915：ABORT P2F=2 复翻（同两键，F2P 缺 2）→ 插桩勘根 = 空串文本残根（修复 `25f6a3943`，见刀体节）。
- **v6 ALL-DONE 2026-11-15T01:05:31 @67ba9cc63（pull --rebase 后重放为 25f6a3943）bin fb8868f3b03e418f4790382f6794e67e ksig 9/9，五腿全绿**：
  - L1 ip fresh：rows 7195 恒一，**F 59→44 / F2P=15 恰等（缺=0 多=0）/ P2F=0 / lost=0 / newF=0 / residF=44 期望44**；census B 17 键中 2 壳携键（`foo&nbsp;[]<br>` a/b）按预期留 C2/F/I6 刀域残面。
  - L2 ab：rc=0 **10092/0/0** 严格，wall=240s。
  - L3 mt：rows 10092 恒一，F 2→2（X37 旧账除名维持），F2P=0/P2F=0/po_fail=0/lost=0。
  - L4 enginelib：**3984 passed / 0 failed**（名册基线 0，零新增）。
  - L5 ilb 核验腿：rows 1691 恒一，F 33→33 逐位恒等（F2P=0/P2F=0/lost=0/newF=0）——**wrap 臂 else 侧静态可达面零暴露硬证据落账**。
- **换基传递闭合（v6 判读树 67ba9cc63 → 推送树 25f6a3943）**：pull --rebase 并入 s440（ae91fbe94，part04 摘挂章重算加 CE 域门）+ weekly cron/docs——刀体 crates/ 域 diff md5 `7c40f78eb5` 双侧相等逐字节恒一；s440 CE 宿主内逐位恒等 M3-A 行为（commit 明示「editing ip A/B 族 37 键保持」，editing 语料全在 CE 宿主内零暴露，他流自带门禁验证全域双跑恰 1 键 F→P）；docs-only 提交零代码暴露。VERDICT 传递随附。

## ip 残 59F 族图推进

B 族 15 键收割后 ip 残 59→44F：C2/F/I6 方向性壳携 16＋C1 容器尾序反转 8＋R1 qV-dPS 8＋C5 2＋C6 2＋C3 2＋I4 2＋I5 1＝44；C4 越界 3（dt/blockquote 尾/li 尾）维持挂账。下一步＝**C2/F/I6 方向性壳携刀**（样式壳携全链/链接壳弃，census 13 族档第 4 位）。
