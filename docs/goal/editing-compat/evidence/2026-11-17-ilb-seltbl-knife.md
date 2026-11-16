# ilb E2/E3 表格单元域刀收口（ILB-SELTBL，2026-11-17）＝ilb 普查十族全清收官

刀体 f7184d774（`fix(editing): ilb E2/E3 表格单元域刀——下钻排除表结构容器
＋td/th 入壳臂＋表节 caret br 非合法子守卫（ILB-SELTBL）`；docs push 前
rebase 改写自批运行树 a549fe075，crates/ 域 diff 恒等、bin 8ba4aa28 字节锚
为准）＋门禁批 i7h1
ALL-DONE（head=a549fe075 批时，bin 8ba4aa283105edc0062754698be0b3a3，
2026-11-17 07:19 完走）。F2P 恰 2 零新增红，**ilb 语料全清 1691P/0F/0T**
——insertlinebreak 普查十族（A-J）全部划除收官。

## 基线与组合态

刀⑥基线＝i7g0 门禁双腿（1689P/2F head=e429609ba，刀⑤ fresh，残键恰
E2/E3 在册，协议免重跑基线腿）。组合态新增维度：祖先链携入并行渲染流
ef6f2cf5a（刀 D rule-inset 全链，crates/engine 内 animation/computed_style/
css_style_declaration +1294 行＋shim part03 +126＋引擎测试 9 fn）——i7h0
L4 ABORT 归因即此（门期望 4109 未计并行流 9 fn，实测 4118；i7f2 先例同型
处理：属 docs rebase 携入祖先链测试非本流债，期望改锚重射 i7h1）。组合态
安全性由本批实证：ef6f2cf5a 的 shim 改动在树情况下 ilb 双腿 1691P/0F。

## 探针（四段取证，临时测试读数后删除不入提交）

E2/E3 逐形 setup→afterDel（`__zw_ce_delete_apply(false)`）→afterMut（接
`__zw_ce_line_break_mutate(host,null)`）→全路径 execCommand，四阶段 dump
range/DOM/caret：

- E2 `<td>[foo<td>bar]`（(fooText,0)-(barText,3) 跨单元文本选区）：
  afterDel 即恒等 spec 形——双 cell 文本逐摘＋占位 br 各一＋caret
  (td0,0)（占位 br append 不越 (td,0) 边界，DOM Range 插入移界规则 offset
  >index 才移）；afterMut got `<tr><br><br><td>`＝元素 caret (td0,0) 走
  (父,sc) 外迁臂把 br 推上 tr 层——唯一缺口＝td/th 不在 DTP-REMAP。
- E3 `<tbody data-start=0 data-end=1>`（harness data-start 语义＝
  (tbody,0)-(tbody,1) 元素端点，contained 仅 tr1）：afterDel 恒等 spec
  形——contained-nodes 表结构排除（spec :4689-4693）使 tr1 保留、双 cell
  占位 br、startBlock=tbody 非 br 合法父→null→collapseToStart 保持
  (tbody,0)；afterMut got `<table><br><br><tbody>`＝外迁＋caret (table,1)；
  全路径更糟——刀⑤ ILB-SELDRILL 对同节点元素容器下钻把 (tbody,0)-(tbody,1)
  改写为 (td0,0)-(td1,1)，caret 落 td 内。两缺口＝下钻未排除表结构＋无
  br-非合法子守卫 ②。
- 参考 E3p `<tr data-start=0 data-end=2>`（createlink 语料同款 tr 级形）：
  afterDel 同恒等（表结构排除面自产），佐证删除机制零改动结论。

## 根因与修复面（三组件，一提交；vendored delete 机制零改动）

1. **ILB-SELTBL 下钻排除**（part06 ilb 臂）：外层门增表结构容器判——
   thead/tbody/tfoot/tr/td/th 同节点元素端点不逐层下钻（spec
   implementation.js :4689-4693 contained-nodes 排除表），原选区直通
   delete_apply 自产面（tr 保留＋占位 br＋collapse (tbody,0) 探针实录）。
2. **ILB-TBLSCT 守卫 ②**（part01 mutate 入口 lineBreakOnly 臂，rawtext
   守卫同块扩臂）：start Element ∈ {table,thead,tbody,tfoot,tr} 行断
   return true——spec :7396-7401「start 节点为 Element 且 br 非合法子
   → return true」真身（此前 D 族 rawtext 守卫为窄表旁路实现）。E3 exp
   `<tbody>{}<tr><td><br></td>…`＝tbody 零 br＋caret 原位。td/th 不入表
   （br 合法子，走入壳臂）。
3. **DTP-REMAP 扩 td/th**（part01 元素 caret 入壳臂）：空 cell 占位 br
   落位后 caret (td0,0)，br 插 cell 内 childNodes[0]＝占位 br 前、caret
   塌 (td0,1) 两 br 之间（spec :7383 insert br 步；E2 exp
   `<td><br>{}<br></td>`＝新 br 占位 br 前的语料字节序）。

## 钉测（part131，同提交常驻）

`part131_ilb_seltbl_pins.rs` 4 钉：刀键 2（E2 跨单元 `<td>[foo<td>bar]`→
`<td><br><br></td><td><br></td>` 形＋E3 tbody data-start=0 data-end=1→
tbody 零 br tr 保留形；setupDiv 复刻 tests.js :5313-5325 data-start/
data-end 元素 attr 语义）＋表格域控制 2（E1 跨界 td 双 br 现役绿形＋E4
host 级 containment 现役绿形恒一）。定向 1 passed/0 failed；全库
4118P/0F（4118＝i7g0 面 4108＋并行流 ef6f2cf5a 引擎测试 9＋part131 钉 1）。

## 门禁批（i7h0 ABORT 归因＋i7h1 ALL-DONE）

i7h0（06:41 发射）：L0 bin 8ba4aa28 ksig=2＋L1/L2 ilb 双腿 1691P/0F/0T
（35s/37s）＋L3 F2P-EXACT-2 全绿——L4 ABORT enginelib 失配 rc=0 P=4118
vs 期望 4109。归因＝祖先链并行流 ef6f2cf5a 引擎测试 9 fn 未计入期望
（git grep #[test] e429609ba=4111 → head=4121；统计面勘合＋ef6f2cf5a
diff 12 文件 +1244 实证），非引擎债。i7h1（06:46 发射，改锚 4118 重射）：

| 腿 | 门 | 结果 |
|---|---|---|
| L0 runner 复建 | bin 锚＋刀体特征串 | bin=8ba4aa28… ksig=2 head=a549fe075 |
| L1/L2 ilb 双腿 | 1691P/0F/0T | 双腿同数（35s/35s；1689+2 翻清算术对齐） |
| L3 键集 judge | 零新增红＋F2P 恰 2 | F2P-EXACT-2（E2/E3 翻绿＝**ilb 语料全清**） |
| L4 enginelib strict | 0F | **4118P/0F**（4108＋并行流 9＋part131 钉 1） |
| L5 mt 腿 | 残红恰 X37 双键恒一 | **10090P/2F/0T** vs ihe6-mt 基线恒一（wall 1908s；SELTBL 下钻排除/td 入壳/表节守卫组合态首验通过） |

批工件 `.acceptance/dps/i7h0-*`（ABORT 留档）＋`.acceptance/dps/i7h1-*`
（发射器 md5 断言 1cecd3ee…，systemd unit zw-i7h1 pid=2124899）。

## 收官与后续

insertlinebreak 普查收官：十族 A-J 全部划除（i7c0..i7h1 七刀链＋刀⑤⑥
双收口），ilb 语料 1691P/0F/0T 零残红，mt 面 ilb 零残红（X37 双键＝
subscript/formatblock 域非本命令）。后续在账（exec1 474 域决策、fb apply
侧 ~32 族＋img 23F＋fontSize T1 ×3 待接收）按控制面排序推进。
