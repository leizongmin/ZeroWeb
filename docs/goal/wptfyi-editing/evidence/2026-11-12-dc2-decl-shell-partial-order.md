# DC-2 · decl 面壳多值偏序（d5decl 批，2026-11-12）

P2 apply 主循环 ①：u/s decl 面壳（inline `text-decoration` 替换 tag face）
的 apply/removal 分族清账——元素载体折叠批残余 65F 中的 ①簇（多值偏序
+ 分解转形）落地。六 format 文件 + 哨兵探针断言级 **净增 +70 零回归
零漂移**（F2P=70 / P2F=0 / newF=0）。

## 修复面

`part01d` 六臂 + `part01c` 一臂（仅 underline/strikethrough 命令面，
余命令零影响；录形 = data/underline.js、data/strikethrough.js 原文）：

- **gSpecified 长手回退**：strike/underline 的 style 声明面短手
  `text-decoration` 缺位时回退读长手 `text-decoration-line`（Chrome 同一
  逻辑属性；本引擎溶解换形产物即长手形，:492/:532），含命令 token 才认，
  否则 null 掩蔽 tag 面。`part01c` computed 长手回退同形补臂。
- **清值偏序删除**：removal 面对含命令 token 的 shorthand 声明仅剥该
  token 重写（junk 丢弃、canon 位序 [underline, overline, line-through]；
  underline.js:725-761 junk 族、:479-483 实证）；剥空照旧整删（:831
  recheck null → :2114 attributeless 解壳链不变）。他 prop 维持整删。
- **⑦ 门 decl 壳 ADD 分族**（data/underline.js:514-577、
  strikethrough.js:474-563 直译）：仅 decl 面（decl 空 = tag face 走既有
  绿面）——(a) 命令 token = 壳 tag face 外裹保壳（u+underline css-true
  span(newValue) / css-false u；:516/:521/:560/:565）；(b) 溶解合并
  shorthand span(decl∪命令 token)（s+underline css-true :540、u+strike
  css-true :556）；css-false 对位外裹保壳（strike :561、u :545）；(d)
  s/strike + strike css-true 溶解换形 **longhand**
  span(text-decoration-line:…)（:492/:532）、css-false 壳转形 + 内裹
  strike（surviving=[underline] 换 u 剥 decl :497、余者换 span
  longhand :537）。换形壳 attr 直写单声明（`_zwVaWriteTdDeclOnly`，
  规避 SetStyleProp 叠加并出 shorthand+longhand 双声明错形）。
- **s/strike decl 壳部分选区切分抽裹**：ADD 面切分（:500-507 三段形）
  ——既有面因 spec null 恒内裹不切分；门 = strike ADD、命令 token = 壳
  tag face、壳带不含该 token 的 inline decl；u 壳对位形恒内裹不入。
- **removal 重物化元素形**：peel 克隆/残余 el 携 decl 或裸 tag face 按
  Chrome 分解转形（css-true 换形 span(残值|[removed]) :504-513/:520-528
  /:648-656；css-false 残值非空保 tag 改 decl、残值空换 tag(removed)
  剥 decl）；css-true bare s 壳恒保 tag（:271-280/:655-663）、u/strike
  裸壳移除区贯穿元素尾的 start 克隆保 tag（`_zwVaPeelKeepTag` 登记，
  underline.js:658-662、strike.js:710-718；u 出口持线值时仍换形 :713-717）。
- **span decl 壳切刻面**：removal 面部分选区、壳 decl 剥命令 token 后
  有 canon 残值时不切分壳——decl 原位重写为残值 + 选区外首尾段以移除
  token 形内裹（underline.js:484-493、strikethrough.js:479-487；attr
  直写规避 CSSOM `k: v` 空格形泄漏，规范形与既有 canonicalize 链一致）。

## 方法

rebase 后同树前后二进制对跑（base/after 双二进制均在 pull 15 commit 后
的同一树重建，systemd user service 包裹，`ZW_SCRIPT_TIMEOUT_MS=300000`），
断言级（全子测试名）前后 diff：base = d5b-runner（HEAD 无本刀）、after
= d5a-runner（本刀）。面 1 六 format 文件 + 10 探针（新增 d5decl 6 case
判据 + d5diag 20 case 矩阵）；面 2 multitest + misc 哨兵（同基线双跑）。

## 数字

面 1：base 11,796/231 → after **11,866/161**（复核 base 与 pull 前
dc2us4-runner 重跑逐名恒等）：

| 文件 | base P/F | after P/F | ΔP |
|---|---|---|---:|
| underline | 2121/26 | 2143/4 | +22 |
| strikethrough | 2108/39 | 2131/16 | +23 |
| bold/italic/sub/sup | 恒等 | 恒等 | 0 |
| zz-probe-d5decl | 0/6 | 6/0 | +6（四形逐案翻转） |
| zz-probe-d5diag | 1/19 | 20/0 | +19（矩阵清零） |
| tstyle/mtmini/p2diag 既有诊断 | 1F/1F/5F | 恒等 | 0 |

- 断言级：**F2P=70 / P2F=0 / newF=0**，无任何共享断言位移；rebase 组合
  态对 bold/italic/sub/sup + 既有探针零漂移（并行流 maxlength 刀
  无交互实证）。

面 2（multitest + misc 哨兵）：base d5base2 10,462/24/1T → after
d5after2 **10,466/24/1T**：misc 401/22 恒等、multitest F=2/T=1 恒名，
fail 集逐名 diff 空（25 行恒等）。multitest P 差 +4 全落 `?9001-last`
慢页门杀进度差（base 门杀前 1,063 P vs after 1,067 P，300s script 门
非确定性，承 carrier-fold 批口径）；`?9001-last` 之外断言名集 9,423 行
逐名恒等，共享断言零漂移。

## 残余与下一步

- u/s 残余 20F（strike 16 / underline 4）两簇：② 部分选区 styled-child
  分解再裹 ×10（`<s>foo[b<span style=…>ar]ba</span>z</s>` 族 + 跨段
  `<p>[foo</p><p> <span>bar</span>…` 双命令各 1）；③ 零变化/读面 ×10
  （`fo[o<s>b]ar</s>` 卷位差 ×3、`<span style=text-decoration:
  line-through>` toggle-off ×1、bare 壳 queryCommandState after ×4、
  styleWithCSS before ×2）。
- bold 84F / italic 24F / sub+sup 26F 面未动（master ② 清单）。
