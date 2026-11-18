# ilb B+G+E 选区删除域刀收口（ILB-SELDEL，2026-11-17）

刀体 e429609ba（`fix(editing): ilb B+G+E 选区删除域刀——同节点元素容器
下钻＋后删 caret 入壳＋块界补断细化＋rawtext no-op 守卫（ILB-SELDEL）`）＋
门禁批 i7g0 ALL-DONE（head=e429609ba，bin
6010e608bcebe96c428b709d05152e97，2026-11-17 06:18 完走，**首发零 ABORT**）。
census 刀序 ⑤主体＋⑥一次兑现 11 键（B6＋G1＋E4＋E1＋D2），F2P 恰 11 零
新增红，ilb 残 2F（E2/E3 表格域）。

## 基线

刀⑤基线＝i7f2 门禁双腿（1678P/13F head=b6e2ddc05，刀④ fresh，协议免重跑
基线腿）。组合态注意：现树含并行编辑流 d48 join 壳吸收刀（70b18f81e）及
其后继引擎测试（lib 面 4107），mt 对 ihe6 残红恒一门实证组合态（漂移即
ABORT-ATTRIBUTE）。

## 探针（三段取证，临时测试读数后删除不入提交）

B/G/E/D 10 形逐形三段：setup range 形 → `__zw_ce_delete_apply(false)` 单独
跑完 DOM＋caret → 接 `__zw_ce_line_break_mutate(host,null)` 终态 → 全路径
execCommand 对照。关键读数：

- B1 `{bar}`（(span,0)-(span,1) 元素容器同节点）：afterDel 即对齐
  （bar 删、span 留、caret 塌 span@0）——缺的是路由与入壳插入。
- B2 `{<span>bar</span>}`（(p,1)-(p,2)）：前导默认 stripWrappers=true 把壳
  剥净（`<p>foobaz</p>`）——需下钻归一走文本切片径。
- B3/B4/B5：前导删完 caret 均 (span,0)，mutate 元素 caret 臂外迁——缺入壳。
- G/E4：host 级 afterDel 完美（h1/table 整摘＋占位 br 对齐）——缺路由
  （同节点元素容器被旧门让位原径零删除）。
- E1：删除＋插入均对，只缺块界补断（`fo<br><table>` vs exp `fo<br><br>
  <table>`）。
- D1/D2：collapsed 插入未挡——缺 rawtext no-op 守卫。

## 根因与修复面（四组件，一提交）

1. **ILB-SELDRILL**（part06 ilb 臂）：同节点元素容器非 collapsed 选区逐层
   下钻到内容子切片（`{bar}`→(span,0)-(span,1)→(barText,0)-(barText,3)，
   B2 两层），下钻产物走既有 text 切片径（`[bar]` 1641P 基）零新增机制；
   host 级元素容器不下钻——containment 整元素摘除（G/E4 afterDel 实录），
   旧门 `startContainer === endContainer` 直通改「text 容器才直通」。
2. **后删元素 caret 入壳**（part01 DTP-REMAP 扩臂）：`lineBreakOnly &&
   !isText && sc !== host` 增 `_zwCeIlbInlineClimb(sc)`——inline 壳
   （span/b/i/… 白名单，void/replacement 不入）元素 caret br 插壳内
   childNodes[so] 非 (父,sc) 外迁（corpus `<span><br>{}</span>` 形实录；
   语料普查绿键零 `{` inline 形暴露）。
3. **ILB-BLKPAD**（`_zwCeBrVisibleAfter` 细化）：后随**块界元素**（非
   inline-climbable：table/p/h1-6/显式非 inline display）不填 br 行盒补
   第二 br（E1/G 实录）；替换型空元素白名单（img/br/hr/…）仍占行盒不垫
   （`{}<br><h1>` 现役绿 3 键恒一——pad 面语料普查 4 绿键全 br-后随形零
   翻转暴露）。
4. **ILB-RAWTEXT 守卫**（part01 mutate 入口，仅 lineBreakOnly 臂）：
   rawtext/text-only 容器（xmp/script/style/textarea/title/iframe/noembed/
   noframes/plaintext/noscript 窄表）行断 no-op——spec implementation.js
   :7396-7409 br 非合法子 return true（D 族 2 键 exp `<xmp>foobar</xmp>`
   零变更）。ip 域 lineBreakOnly=false 不入守卫零接触。

## 钉测（part130，同提交常驻）

`part130_ilb_seldel_pins.rs` 16 钉：刀键 11（B 族 6＝543/548/573/578/588/
583swcss 双态＋G 378＋E4 33＋E1 8＋D 族 253/261 rawtext）＋现役绿形控制 5
（:538 text 容器切片／:553 跨节点前导／:358 br-后随既有 br 不垫／`foo[]`
块尾补断／:168 h1 内同形）。定向 1 passed/0 failed；全库 4108P/0F
（4107＋part130 钉 1；4107＝i7f2 面 4100＋并行流 7）。

## 门禁批（i7g0，首发零 ABORT）

ALL-DONE 2026-11-17T06:18:22 head=e429609ba bin=6010e608…：

| 腿 | 门 | 结果 |
|---|---|---|
| L0 runner 复建 | bin 锚＋刀体特征串 | bin=6010e608… ksig=1 head=e429609ba |
| L1/L2 ilb 双腿 | 1689P/2F/0T | 双腿同数（40s/40s；1678+11 翻绿算术对齐） |
| L3 键集 judge | 零新增红＋F2P 恰 11 | F2P-EXACT-11（白名单=i7f2 残 13 扣 E2/E3 定义域残键，同配方 grep -Fv 提取恒一） |
| L4 enginelib strict | 0F | **4108P/0F**（含 part130 新钉） |
| L5 mt 腿 | 残红恰 X37 双键恒一 | **10090P/2F/0T** vs ihe6-mt 基线恒一（wall 1745s；SELDEL 下钻/块界补断/rawtext 守卫组合态首验通过） |

批工件 `.acceptance/dps/i7g0-*`（发射器 md5 断言 dffa815e…，systemd unit
zw-i7g0 pid=1965595）。

## 尾巴

ilb 残 2F＝E2（`<table><tr><td>[foo<td>bar]<tr>…` 跨单元选区：got td1 整摘
br 落 tr 层）＋E3（tbody data-start/data-end 选区：exp 双单元占位 br＋
caret 落 tbody 界 no-op 插入）——**表格单元域刀**（下一刀）：delete 前导
contained-nodes tr/td/tbody 排除面＋ emptied-cell 占位 br 双单元填充＋
guard ②（tbody 界 br 非合法父 return true，E3 exp 探针已证 no-op 形）。
census 刀序 ⑤⑥余量仅此 2 键；exec1 474 域决策、fb apply 侧 ~32 族＋img
23F＋fontSize T1 ×3 待接收照旧在账。
