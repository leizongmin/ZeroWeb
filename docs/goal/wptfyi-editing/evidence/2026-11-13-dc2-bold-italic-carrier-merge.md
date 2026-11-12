# DC-2 · bold/italic 载体吸收与边界挖入（d7 批，2026-11-13）

P2 主循环 ③：bold 残余 42F 三簇（notbold 类指定值读面、跨 merge 邻接、
数值壳 partial/b#purple）+ italic var/dfn 归一清账。face 1（六 format
文件 + 42 哨兵探针，含新增 d7diag 20 case）断言级 **净增 +55 零回归**
（F2P=55 / P2F=0 / newF=0 / 断言名集漂移空；bold 42F→13F、italic
24F→18F）。face 2（multitest + misc）fail/timeout 恒名恒数，P 差 +66
全落 `?9001-last` 慢页门杀进度差（300s script 预算非确定性窗口，承
carrier-fold/d6 批口径）。

本批由上一 session 落码（rate-limit 中断未验证），本 session 对跑
验证并纠偏一处：初版 ④ 挖入门含 u/s/sub/sup gCS 信号臂，face 对跑
两轮（signal 两形）均产 `foo<s>[bar]</s>baz` +underline css-true
`<s><span style=underline>` 保壳内裹逆录形——实际录形 = decl 合并
`text-decoration:underline line-through`、s 溶解（P2F 7 case），且
shim gCS 对 UA 表读数不可靠无法辨信号；该臂弃用，④ 收窄至
bold/italic bare 载体面（zero F2P 依赖 u/s/sub/sup 分支，弃用零损）。

## 修复面

`part01c` 一臂 + `part01d` 四臂（录形 = data/bold.js、data/italic.js
原文）：

- **resolved prop 读序重排**（part01c `_zwFmtResolvedProp`）：inline →
  作者表（`_zwPlainAuthorDecl`，复用 ③'-c 作者快照匹配机具）→ host
  gCS → UA 元素表。继承属性下 own 声明恒胜继承值，host gCS 读数混入
  父链继承（`.notbold` 载体在 `<b>` 内 host 读 bold 继承致 state 读
  true 误判，bold.js notbold 簇 qS before/after 面实证）；作者臂补
  `<link>/<style>` 类选择器声明盲区。
- **阻断载体递归臂 + 独占载体 unwrap**（part01d ③，bold/italic）：
  contained 非简单可改性且无本命令 specified 的类/身份壳/他命令 tag
  face 不被外裹，forceValue 下潜内容侧原位保留（`[foo<span class=
  notbold>bar</span>baz]` = `<b>foo<span class="notbold"><b>bar</b>
  </span>baz</b>`、`<b><span class=notbold>[foo]</span></b>` 递归后
  外层 b 单子摘除升位）。
- **边界挖入门**（part01d ④，bold/italic bare 载体）：端点落元素内且
  range 全部内容落元素之内时元素不作整裹单元（abc 族分型 :1045-1080
  ——端点在 i 内只裹文本 `abc<i><b>def</b></i>ghi`，端点在元素层紧贴
  两缘裹整 i `abc<b><i>def</i></b>ghi`）；spec 维度带指定值/自有
  inline 声明不摘；挖入收窄以 bp 序裁越界形（双 span 跨选保既有并拢
  绿面）。u/s/sub/sup 不扩面（见前言纠偏）。
- **toggle-off css-true 残料 span 化 + b#purple 属性提升**（part01d
  ⑥）：css-false 切分幸存 b 保 tag 既有面不动；css-true 切分幸存 b
  换形 span+font-weight:bold（:918-933 首/尾侧载体）；两翼载体同
  tag、属性集同含身份属性、同父相邻 → 属性提升外裹 span（:433-442
  `<b id=purple>bar [baz] qoz</b>` 双 flag 录形直译；style 面仅命令
  换形 decl 可入，700/900 decl 壳排除）。
- **载体吸收臂**（part01d ⑦ `_zwVaAbsorbCarrierSiblings`，
  merge-through-carrier，:813-847 M 族直译）：face 并值/新裹落位后
  邻接阻断载体（specified 空 + 独子链贯通至命令 tag face）整链吸入
  并摘链内冗余 face（`<i><b>foo</b></i>[bar]<b>baz</b>` =
  `<b><i>foo</i>barbaz</b>`、font-b 载体链同形）；链尾非命令 face
  （裸 `<i>baz</i>`）语义门不吸；css-true 吸收形先并/换形 tag 单壳。
  接入早并落位面与新裹落位面两处。

## 方法

同树前后双二进制对跑（base = d7a-runner@HEAD 33236320b——本轮开工
rebase 入渲染流 R5508/5509、css-values g23、嵌套簇 B1 根因刀等 17
commit 组合态；after = d7b3-runner@工作树含纠偏，systemd scope 守卫
构建，`ZW_SCRIPT_TIMEOUT_MS=300000`），断言级（全子测试名）前后
diff。face 1 = 六 format 文件 + 42 探针（既有 41 + 新增
zz-probe-d7diag 20 case 常驻判据；zz-probe-d7trace 为本批开发期
trace 临时探针，验后移除不入册）。face 2 = multitest + misc 哨兵。

定点引擎钉测（test-guard 包裹，`cargo test -p zero-engine --lib
execcommand`）：6 passed / 0 failed / 1 ignored（nested_cross_b 既有
ignore 恒态，B1 解钉 italic_after_prior_cmd 恒绿）。本批仅改 .js
shim，Rust 面零变化，clippy 无新增检查对象。

## 数字

face 1：base 15,343/167 → after **15,398/112**：

| 文件 | base P/F | after P/F | ΔP |
|---|---|---|---:|
| bold | 3006/84→42（d6 后） | 3035/13 | +29 |
| italic | 2049/24 | 2055/18 | +6 |
| underline | 2143/4 | 恒等 | 0 |
| strikethrough | 2131/16 | 恒等 | 0 |
| subscript / superscript | 1228/13、1260/13 | 恒等 | 0 |
| zz-probe-d7diag（新增） | 5/15 | 20/0 | +15 |
| zz-probe-d6diag | 19/6 | 24/1 | +5 |
| 其余 38 探针 + tstyle/mtmini/p2diag 诊断 | 既有恒态 | 恒等 | 0 |

- 断言级：**F2P=55 / P2F=0 / newF=0 / goneF=0**，断言名集逐名恒等。
- bold +29 簇：notbold 类壳 qS 读面 + innerHTML（作者臂+③）、M 族
  merge-through-carrier（⑦）、purple 双 flag（⑥）、700 partial
  innerHTML（④ spec 门）。
- italic +6 簇：cite/dfn/var 双 flag innerHTML（⑦ 链尾 tag face 归
  一吸收——master ① italic var/dfn 归一清账）。
- d7diag 20 case = abc 族 6 + M 族 7 + purple/700/900/残料 7，全绿。

face 2（multitest + misc 哨兵）：base 10,398/24/1T → after
10,464/24/1T：

- misc 401/22 断言级恒等（423 行逐名 diff 空）。
- multitest fail 集 24 行逐名恒等、Timeout 1 恒名；P 差 +66 全落
  `?9001-last` 慢页门杀进度差（drift 行 100% 含 ?9001-last，其余
  断言名集恒等）。

## 残余与下一步

- bold 残余 13F：① 700/900 壳 partial qS 读面 + css-true 形 ×6；
  ② 跨块 `<b><p>foo[<i>bar</i>}</p>` 载体 ×2；③ qS 杂项（collapsed
  `<b>foo[]bar</b>`、`foo[<span style=bold>` 指定值读面、100/200 邻
  接 qS）×5。
- italic 残余 18F：css-true i 边界残料 span 化（bold ⑥ 之 italic 臂
  未落）×3、var/dfn/cite css-true 形 ×3、oblique/邻接壳 ×…（清单见
  transcript）；u/s 20F、sub/sup 26F 面未动（master ② 清单）。
- d6diag 余红 1F（D19 strong-mid fresh b）留作 ② 簇代表。
