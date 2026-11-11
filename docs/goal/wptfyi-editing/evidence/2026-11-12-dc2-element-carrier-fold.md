# DC-2 · effectively-contained 元素载体折叠（u/s/ins/del，2026-11-12）

P2 apply 主循环 effectively-contained 元素臂 ⑤：u/s/ins/del 载体三形
（strike-wrap-tag 残余 ① + ⑤ 保壳族）Chrome 录形清账断言级 **+10 零回归**。

## 修复面

`part01d _zwVaForceValue` 三臂（仅 underline/strikethrough 命令面，余命令零影响）：

- **span/ins/del 门扩**：ins/del 元素载体入 P3 span 落点门——css-true
  保壳原位并 tag face（`foo<ins>[bar]</ins>` +strike =
  `<ins style="text-decoration:underline line-through">`，strike.js:389；
  `foo<del>` +underline 同形 underline.js:419，内容语义壳不溶解）；css-false
  tag face 走下潜内容侧（`<ins><strike>[bar]</strike></ins>`、
  `<del><u>[bar]</u></del>` strike.js:389/underline.js:424）。bare ins +
  underline 无变化由 F6b 等值门早退、del + strike 零变化由 removal 不解
  del 保，均不入本门。
- **css-true bare u/s/strike 折叠溶解**：全含壳（tag face、无 inline/author
  decl）自身溶解、tag 值并入新裹 inline 面短手（`foo<u>[bar]</u>` +strike
  = `<span style="text-decoration:underline line-through">[bar]</span>`，
  strike.js:182、underline.js:241 `foo<s>` +underline 同形；canonical 位序
  [underline, overline, line-through]）。decl 面载体不入本门。端点序
  insertBefore→搬移→removeChild：par 层 insert +1 陈旧恰被同位 removeChild
  抵消，壳内端点随 `_zwJusMovePreservingRanges` 搬移。
- **⑦ own face 补 tag face 臂**：`__zwPlainComputed` 无 text-decoration 臂
  致 bare u/s/strike ownFacePre 恒空、css-false 整壳外裹——补 tag face 后
  ownGenuine 成立下潜子级（`foo<u>` +strike css-false =
  `<u><strike>[bar]</strike></u>` nest-inside 录形 strike.js:182、
  underline.js:246 `foo<s><u>` 同形）。仅 u/s 命令面补臂。

## 方法

同树前后二进制对跑（test-guard/systemd scope 包裹，`ZW_SCRIPT_TIMEOUT_MS
=300000`），断言级（全子测试名）前后 diff：base = dc2us3-runner（strike
wrap tag 批入账树，face 11,785/216 + us3mt 10,485/24/1T），after =
dc2us4-runner（本刀）。面 1 `dc2us-face-run.sh`（六 format 文件 + 8 探
针）、面 2 `dc2us-mt-run.sh`（multitest + misc 哨兵）。

## 数字

面 1：us3 11,785/216 → us4 **11,795/206**：

| 文件 | base P/F | after P/F | ΔP |
|---|---|---|---:|
| strikethrough | 2104/43 | 2108/39 | +4 |
| underline | 2115/32 | 2121/26 | +6 |
| bold/italic/sub/sup | 恒等 | 恒等 | 0 |
| 探针 8（tstyle 1F、mtmini 1F、p2diag 5F 既有诊断） | 恒等 | 恒等 | 0 |

- 断言级：**F2P=10 / P2F=0 / newF=0**，10 翻转逐名恒等，全落目标簇：
  ins/del 载体 ×4（strike `foo<ins>` css-false/true、underline `foo<del>`
  css-false/true）+ css-true bare 壳折叠 ×6（strike `foo<u>`、underline
  `foo<s>`、`foo<strike>` 各 css-true）。

面 2（multitest + misc 哨兵）：us4mt **10,230/24/1T**：misc 401/22 恒
等、multitest F=2/T=1 恒等名，断言级 **newF=0 / 丢失=0**（fail 集逐名
diff 空）。multitest P 差全落 `?9001-last` 慢页门杀进度差（us3mt 门杀前
1,086 P vs us4mt 831 P，300s script 门非确定性；?9001-last 之外丢失
=0），共享断言零漂移。

## 残余与下一步

- u/s 残余 65F（strike 39 / underline 26）三簇：① decl 面壳多值偏序
  （`foo<s style="text-decoration: overline">` +underline 族，长手折叠
  语义 ~19 例）；② 部分选区 styled-child 分解再裹（`<u>foo[b<span>ar]
  </span>z</u>` 族）；③ 零变化簇残余（`foo<s>[bar]</s>` +strike ×2 卷
  位差）。
- **①簇录形实取**（2026-11-12 探针
  `zz-probe-d5decl.html` 常驻判据 6 case，期望 = data/underline.js:538+
  /data/strikethrough.js 原文）：decl 壳（inline text-decoration 替换
  tag face）apply 族当前恒下潜内裹，Chrome 分四形——
  (a) wrap-out 保壳：u-decl + 命令 css-true/css-false 双 flag（`<span
  style="underline"><u style="overline">` / `<u><u style="overline">`，
  新裹在 decl 壳外、decl 原样）；(b) 溶解合并：s-decl + underline
  css-true、u-overline + strike css-true（`<span style="underline
  overline">` 单壳，tag 消失）；(c) 全溶解零产物：u-lineThru + strike
  css-true 全含（`foo[bar]baz`，inline decl 抵 tag face 后 toggle-off
  残壳不值钱）；(d) 分解转形：u-lineThru 部分选区 +strike（`b</u>…` 段
  转 `<span style="line-through">`，选中段裸化）。落点 = ⑦
  residualDecls 下潜门 decl 壳分族 + removal 分解面元素形，与 ② 簇
  （styled-child 分解）同机器。
- bold 84F / italic 24F / sub+sup 26F 面未动（master ② 清单）。
