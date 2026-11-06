# 2026-11-07 fbq 腿 P2F=7 / lost=30 核验方案号级归因（legs3，head=64bcf8491+树 +38 预检）

核验方独立判读 legs3 门禁数据（`batch-fbq-legs.log`：fb F2P=794/P2F=7/lost=30）。对主 evidence 文档两处归因提出修正，并给出第四刀的三行真债案号。

## 修正一：lost 30 = 分片覆盖缺口，非插桩伪影

- legs3 跑的是干净 committed 版（64bcf8491，无 FBQ-STAGE throw），lost 仍 30 →「插桩伪影」假说被证伪。
- 真因：**语料 `data/formatblock.js` 共 363 案，8 分片只盖 [0,360)，且 b7 有 off-by-one（SLICE_B=359）**——未覆盖案 = **359、360、361、362** 四案（`/tmp` 核验脚本核算）。四案 assert 行合计恰 30。
- 案 360-362 正是 nested contenteditable=false 混合形（display:block span 链 / brace 形），与「lost 全集中在 ?5001-last 族」观察吻合——位置吻合但机制是缺口不是 throw。
- 修复实证（03:05-03:10 补跑）：`zz-fbq-b8.html`（SLICE_A=359, SLICE_B=363）补跑 29P+1F 恰 30 行——1F = 案 362 compare innerHTML（pre 同红 F2F 非新债）；配套 zz-fbq-lost.html 切片页（6 案 verbatim）54 行全落账（51P+3F，3F = ret-F2F 恒红族挂账）证条目本体健康。wl-rerun post2 log 行源页全为 zz-fbq-b*.html → 其 lost 30 同源覆盖缺口，FBQ-STAGE 归因一并作废。
- 9 分片重采（batch-fbq-final.sh）或单页跑法（batch-fbq-legs.sh 03:08 改，首选）均覆盖洞清零。

## 修正二：4 行 dps qV-before P2F = 分片窗界泄漏伪影（非引擎回归、非插桩）

案号级铁证（node 解析 `data/formatblock.js` 案索引）：

| P2F 行 | 案号 | 分片窗位 | 机制 |
|---|---|---|---|
| `<p>[foobar]</p>`+dd qV(dps) before | **180** | b4 **窗位 0**（SLICE_A=180，fresh 进程首案） | dps 状态携带链断 |
| `<dl><dt>foo<dd>[bar]</dl>`+div 同行 | **45** | b1 **窗位 0** | 同上 |
| `<p>[foo<p>bar]`+dd 同行 | **229** | b5 窗位 4——但前邻 225-228 全无 dps exec（article/aside/blockquote/body 案） | 携带链在窗界后至首 dps 案前必断 |
| `<pre>[foo]<br>bar</pre>`+p 同行 | **318** | b7 窗位 3——前邻 315-317（address+fb h1）无 dps exec | 同上 |

机制：qV(dps)-before 断言执行于案 setup 后、execs 前（runner 行序＝打印序非执行序），读的是**跨案携带状态**（录音会话泄漏链，[editing-corpus 窗界状态泄漏]既定先例）。pre 基线跑法（行区间窗 ?1-1000）与 post 分片跑法（案区间窗）窗界不同 → 同案在 pre 链完整、post 链断裂 → P2F。对照片证：同族 dps-p 变体案（181/230/46/319）全绿——它们吃窗内前一案的 dps exec 携带，值恰与录音吻合（案 180 的 dps-div exec → 案 181 起点携带 "div"）。与 nq7 轮「qVb 窗界伪影非引擎可修」同类。

处置建议（03:15 更新）：不修引擎；**单页跑法（行区间窗）与 pre 窗界同构 → 4 行伪影自动消失**，无需名册过滤。batch-fbq-legs.sh fb 腿已改单页首选、9 分片回退（回退路径仍需滤这 4 行）。

## 真债 3 行（第三刀实装收口，legs4 终判）

| 行 | 案号 | 现象 | 实装机制（02:44 工作树四笔，与下方原提案不同） |
|---|---|---|---|
| `<div><ol><li>[foo]</ol></div>` + fb div compare ×2（dps 两变体） | 340/341（b7） | 树仍变更，Chrome exp 零变更 | 同 tag skip 祖先链爬升**列表透明**（ol/ul/li 穿过不判、弃 prohibited veto），首个非列表元素 tag==v 剔除 → nodeList 空 → return true 零变异 |
| `<p>[foo<p>bar]` + fb `<p>` compare ×1 | 267（b5 窗位 42） | 树变更，Chrome exp 原树不变（双 p 已同名） | SLC 解包臂 `wrap(sublist)` 裸 text 成员父=host 触发 ViaHtml innerHTML 整段重写 → 后续成员游离复制倒序；`_zwJusWrap` 第 4 参 `plainSkipViaHtml`（仅 fb 传 true）走 plain wrap，解包重锚净零变更 |

原提案（apply 级 ol/ul 子树早退、按块同名整体 no-op）未采用：列表透明爬升覆盖同一案形且改动局部于既有 skip 环，ViaHtml 游离为根因级修复（提案机制只压症状不解 ViaHtml 对多成员循环的破坏）。配套两笔：`_zwFbqVisBlock` 去 prohibited veto（qV 读面含 li 块误返 ""）保留；wrap 位允许性预检**撤销**（核验方 B2-vs-NEW 对账：host 位 dd/dt 20 行 Chrome-matching 回归 / 0 行收益，`fbq-thirddelta-dddt-regression-verifier-alert.md`）。legs5 单页终判判读三行归零为验收线。另：legs3 二进制为部分快照（build 窗口落在编辑之间），其 fb/mt 行级数字对任何树状态非有效测量。

## mt 腿死亡定谳与分片方案

wl-rerun mt 全量单进程死亡实 Observ（~12min、rc=2、0 行＝缓冲输出全丢）；legs3 mt（02:46:40 批的 02:50 起跑腿，含第二刀 qV 重写 + 第三刀前二进制）03:15 时点 25min 仍存活、远超死亡窗，文档先前「legs3 同型死亡」为投影非实 Observ，待 ALL-DONE 定谳。若 legs3 mt 完整出账 → legs4 mt 维持单进程（与 batch-nq7-mt-post.log 基线同构）；若同型死亡 → 落 `batch-fbq-final.sh` 方案：multitest.html 按自带 variant 窗 10 fresh 进程（各 ~1K 行 ≈71 案，低于 c120 存活线）+ multitest-ab 独立进程（全 a-b 字母命令，无四命令 apply 面）；judge key（`page?variant :: case`）与基线逐字同型。

## 混合二进制污染史（本批判读前科，防复发）

- wl-rerun b0/b1 分片跑在 02:20:57 throw 版二进制（Fail 88/91 vs b2/b3 干净版 10/4 实锤）；legs3 全分片同二进制（build 02:46:40 晚于第三笔 02:46:25）无此问题。
- `batch-fbq-final.sh` 内置 shim md5 快照守卫：批中源码变化即 ABORT MIXED。

## 追记（04:20 核验方）——final2 终判元组与 P2F=13 终分解

final2（03:28:49 build＝HEAD 64bcf8491 逐字干净版，预检已撤销）ALL-DONE 04:07:35：

| 腿 | 元组 | VERDICT |
|---|---|---|
| fb 9 分片（含 b8） | rows 5046→5046 **lost=0**，F2P=**820**/P2F=13，F 993→186 | REVIEW |
| indent / hr / img | F2P=119/114/53，P2F=0，lost=0（hr 全绿 1348P/0F） | OK×3 |
| mt 全程单进程 | 键级 10092 恒一，F2P=**166**/P2F=70，lost=0 | REVIEW |
| enginelib | 九红名册恒一 | OK |

**mt 干净版 vs legs3 预检版 95 键 F2P 差＝预检附带损伤独立实锤**（同机制：host/p/岛 wrap 位拒绝真换块案形），与 fb 语料 dd/dt 20 行同源。

**P2F=13 终分解**（regress 文件行级）：

- **6 行窗口/携带伪影**：4 行 qV(dps)-before（案 180/45/229/318，前记）＋**2 行 qV(fb)-before table 形（案 342/343，b7 窗位 27/28）**——qV-before 经 override/storedRange 惰性机制间接携带跨案状态，窗位值随前邻案遗留状态翻转（legs3 预检版该两行绿＝预检改变了前邻案遗留 override 态，镜像互证）。非引擎可修，窗界伪影名册同录。
- **1 行真债**：案 267 double-p compare（`<p>[foo<p>bar]`+fb p）——owner delta-③（part01g wrap 调用 plainSkipViaHtml=true，final2 build 后落树）靶向，legs5 验证。
- **6 行 CE 岛回退代价**（`<div CE=false><p CE=true>[foo]` + div、span+p、`<p CE=false><span CE=true>` + p 的 compare+qV-after）：**警示档「预检 0 行收益」断言有误**——预检拒绝路径在 CE 岛形真实可达（成员爬升停在岛内 formattable 容器 → allowedChild 拒 → 树不变 → compare 绿），legs3 该 6 行绿即证。净核算：撤销回收 dd/dt +20（fb）/+95 键（mt），代价 CE 岛 -6 行——**净 +14 仍支持撤销决策**，但终判账应如实记 6 行回退代价，不得按「0 收益」口径。后续可作独立小刀（仅 host 位 dd/dt 豁免式拒绝，不动岛形）收复。

**经验**：qV-*-before 断言行全部是携带敏感面（dps 直接携带 / fb 经 override 机制间接携带），窗界伪影计数不跨二进制稳定——终判 P2F 红线判读必须配伪影名册逐行核案号窗位，机械红线会在伪影上误报。
