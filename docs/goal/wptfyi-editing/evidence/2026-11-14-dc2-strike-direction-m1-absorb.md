# DC-2 · d16 批：strike REMOVE 方向规则（S2e）+ u/s css-true 裸 span 吸收（M1）

日期：2026-11-14。
前置：d15 批（[strike S2a 并拢门 + P5 td span retag 端点重锚](2026-11-13-dc2-strike-s2a-p5-retag.md)）。
基线：face 1 六文件 20F（bold 8 / italic 3 / underline 4 / strike 5 /
sub 0 / sup 0），哨兵 34 名集 + 2 页级 Timeout（fncorpus 两窗，d15 已有），
钉测 7/7。

## 结论先行

face 20F→17F（**F2P=2 / P2F=0**）：strike M1（:33 css-true）与 underline
M1 双胞胎（:33 同构）转绿；bold/italic/sub/sup 断言级 fail 名集逐文件
恒等。S2e（strike :640）innerHTML 转绿但同行 qS-after 翻红（净零，
读面家族归属见下）。两刀全落 `js_dom_shim/part01d.js`：

1. **strike REMOVE 方向规则（toggle dispatcher strike 臂）**：方向判据
   按选区起点位置本地计算（`hitDir(ctxAtDir(start))` 祖先链
   line-through 面命中 ⟺ REMOVE），不用 fq.state（vendored 全 contained
   普查）。**门限 strikethrough**：underline 同形录形相反
   （underline.js:640 ADD 壳吸收）且 u 基线无方向红。ctxAtDir 文本
   尾边界前移规范化（兄弟最深首叶），**off=0 头边界取父链**（上轮草稿
   前移下一兄弟为误，位置语义在父链）。
2. **u/s css-true 裸 span 吸收臂（forceValue pass）**：ADD 面全含
   span（`_zwJusTag === 'span'`、无 text-decoration decl、spec 无命令
   值）且 run 单成员时 `_zwVaSetStyleProp(node, 'text-decoration', …)`
   并值不外裹。run 连接判定沿兄弟链双向扫空白（空白链是连接体），
   **判据 = 链上首个非空白节点即连接（存在性判定）**——成员判定被 :20
   双 span 行打回（Range 半开界：span1 起缘等界不在集内、span2 尾缘
   在，首版成员判定让 span2 误吸收产分体式双 decl P2F）。

## 方向规则的语料核对（上轮草稿两处误读纠正）

- **槽位解码**：state 数组槽位 = `[indetermB, stateB, valueB, indetermA,
  stateA, valueA]`（tests.js:5549 注释）。S2e :640 = `[true,false,"",false,
  true,""]` → **stateBefore=false 而方向 REMOVE**（方向与 qS 读数分途，
  上轮按 [stateB,stateA,…] 误读成 true）。
- **条款 2（起点前命中 ⟹ REMOVE）无语料证据，撤**：草稿引用的
  :527/:581 "两壳间裸文本"行是把**期望输出串**误读为输入行（d14 教训
  同型复发）；真输入 `foo<u style=line-through>b[a]r</u>baz`（:526，
  stateB=true → REMOVE ✓ 条款 1 即覆盖）与
  `<p style=line-through>foo[bar]baz</p>`（:576，同）。语料中全部
  "两壳间裸文本"字符串均为 REMOVE 行的期望输出，无输入行。
- **全含 REMOVE 行一致性**：起点恒命中 → 条款 1 与 fq.state 臂同向
  （:331/:336/:670/:675 及 :526 u-style/:576 p-style 录形核对），
  无翻向面。

## S2e qS-after 翻红（读面家族归属）

方向规则落地后 S2e compare innerHTML 绿（REMOVE 管线起界侧拆分产出
精确 `foo<s>ba</s>rbaz`），同行 `queryCommandState("strikethrough")
after` 翻红（期望 true，got false）——净零。与既有 qS 读面三红
（strike :336 qS-after、:685 qS-after、:690 qS-swc-before）同族：
exec 后选区位移映射 + Chrome 塌缩/邻接 qS 语义（R3254-M3 起点链近似
与 Chrome 实际的分层差异），两条读数互为镜像（:336 我 true/Chrome
false；S2e 我 false/Chrome true）是同族证据。归属读面家族挂账，
exec 面语料真值（innerHTML）已达成。

## 冷路径分叉发现（探针案撤下记录）

S2e :640 行**孤立探针跑产 `rbbaz` 双 b**（end 侧 splitText 重绑 +
move 补偿族，与 d15 P5 位移槽脱钩同家族的镜像形态：d15 = 孤立绿/
序列红，本轮 = 孤立红/序列绿）。face 序列行为为语料权威（Chrome
录音即在序列上下文），探针不守该行——:670 尾边界案孤立绿守方向门
+ REMOVE 管线，:640 行由 face 每批守。

## 账目

- face 1（六文件断言级，base=d15）：20F→17F，F2P=2、P2F=0。strike
  5F→4F（余 4F 全 qS 读面族：S2e qS-after、:336 qS-after、:685
  qS-after、:690 qS-swc-before）；underline 4F→3F（余 2 qS +
  u-qS 134）；bold 8F / italic 3F / sub 0F / sup 0F 名集与 d15
  逐文件 diff 恒等。
- 哨兵电池（40 页 zz-probe-*）：`^Fail` 名集 34=34 与 d15 恒等；
  页级 Timeout 2 条（fncorpus ?1001-2000 / ?2001-last，tests:0 挂起
  形）与 d15 恒等——非本批引入（d15 日志已存在，上轮 34 名集口径
  只计 `^Fail` 断言行）。挂起窗属 fontname 域待查小桶。
- 钉测：test-guard 包裹 `cargo test -p zero-engine --lib execcommand`
  （独立 CARGO_TARGET_DIR=/tmp/d13tgt）7 passed / 0 failed。
- face 2：misc 22F 形态与既有常设红账一致（quasit + swcss 畸形值
  读面）；mt 慢页在并行流编译负载下守卫墙钟内截断，fail 名集对
  d13 基线共窗核对（结果补记于 master.md 批条目）。
- 常驻探针：新增 `zz-probe-d16strike.html` 两 case（:670 尾边界
  REMOVE + M1 吸收 style 归一形），2/2 绿。
- selection 全语料未重跑：两刀 + 读面翻红均在 execCommand
  strike/underline 管线内，Selection API 语料不经 execCommand；
  d15 全 selection 面名集级 P2F=0 基线维持。

## 残余账目（strike 4F / underline 3F）

- qS 读面族 4 条（strike）：S2e qS-after（本批归属）+ :336/:685
  qS-after + :690 qS-swc-before。根因层级 = exec 后选区位移映射 +
  Chrome 塌缩 qS 语义，修复面在选区映射补偿/qS 读面分层，独立批。
- underline 3F：`foo<u>[bar]</u>baz` qS-after（css-true/css-false
  两变体）+ u-qS 条目 134。
- bold 8F / italic 3F：d13 分类维持（b-in-i 块边界嵌套、cssFlag qS
  族切片伪失败等）。
