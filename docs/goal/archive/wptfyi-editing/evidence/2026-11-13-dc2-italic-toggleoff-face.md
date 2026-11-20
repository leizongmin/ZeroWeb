# DC-2 · italic toggle-off 值面（d8 批，2026-11-13）

P2 主循环 ③/⑥：italic 残余 18F 收敛至 6F。face 1（六 format 文件 +
44 探针，含新增 zz-probe-d8diag 11 case）断言级 **F2P=12 / P2F=0 /
goneF=0**（italic 18F→6F；bold/underline/strikethrough/sub/sup 断言
名集恒等；newF 仅 d8diag 新探针 I3c 已知残红代表）。face 2 对跑见下
文数字节。

## 修复面

`part01d` 两臂（录形 = data/italic.js 原文）：

- **⑥ italic toggle-off peel 幸存壳重物化**（bold ⑥ 之 italic 镜像）：
  css-false 且壳 decl ≡italic 转 i 剥 decl（:655-657 首侧幸存壳
  `<i>fo</i>`、:665-667 尾侧幸存壳 `<i>ar</i>`；equivalentValues
  italic 恒字符串等值，oblique 非等值自然排除）；css-true tag face
  残料 span 化（:585-589/:595-599/:294-297 首/尾/中段切分幸存 i 恒
  `span style="font-style:italic"` 形；css-false 切分幸存 i 保 tag
  既有绿面 :590-594 等）。覆盖 I1a/b/c + I3a/I3b + I6（6F）。
- **③ 阻断载体资格两轴化**：资格判定前置至 forceValue 入口外裹臂前，
  拆 bcEligVa（入尾部下潜臂）/bcSkipWrapVa（额外跳过入口外裹）两轴。
  d8 扩列 cite/dfn/var 仅 newValue 'normal' 关值面入两轴（italic.js
  :218-235 `foo{<cite>bar</cite>}baz` 双 flag 录形 = face 保留 + 内嵌
  span font-style:normal，外裹裸 span 非录形；address 族同形但非
  allowed child，走既有 !allowedChild 下潜面）。d7 集合（类/身份壳 +
  命令 tag face）维持 d7 既有外裹-下潜-unwrap 时序不变——首版单轴
  （跳外裹并入 d7 集合）face 对跑产 `[foo<span class=notbold>bar</span>
  baz]` bold 双 flag 6 断言 P2F（`<b>foo</b><b><span class=notbold>bar
  </span>baz</b>` 拆组误形：d7 时序中外裹 b 存在期间 ⑦ 吸收臂/后续臂
  的中间树形态参与录形产出，跳过即逆形），两轴化后 bold 恒等。扩面
  仅限关值的旁证：ADD 面 cite/dfn/var 无独立录形（bold.js 零 cite
  用例、italic.js:435-452 ON 跨 cite = range 外裹形）。覆盖
  cite/dfn/var 关值全选 6F。

## 方法

同树前后双二进制对跑（base = d7b4-runner@a298e4ca2 前身 d7 收口态
face-b4.log；after = d8b2-runner@工作树，systemd scope 守卫构建，
`ZW_SCRIPT_TIMEOUT_MS=300000`），断言级（全子测试名）前后 diff。
face 1 = 六 format 文件 + 43 既有探针 + zz-probe-d8diag 11 case
（新增常驻判据：C1-C3 cite/dfn/var 关值 + I1a/b/c css-true 切分 +
I3a/b css-false decl 转 i + I6 address head + I5 collapsed qS 绿哨
兵 + I3c oblique 嵌套残红代表）。face 2 = multitest + misc 哨兵。
中间态 d8b1（单轴版）bold 6 断言 P2F 定谳两轴化必要性后弃用。

定点引擎钉测（test-guard 包裹，`cargo test -p zero-engine --lib
execcommand`）：7 passed / 0 failed / 0 ignored（nested_cross_b 已由
并行流 D1 刀解钉）。本批仅改 .js shim，Rust 面零变化。

## 数字

face 1：base 15,398/112 → after **15,410/101**：

| 簇 | base F | after F | 修复臂 |
|---|---:|---:|---|
| cite/dfn/var 关值全选（双 flag） | 6 | 0 | ③ 两轴扩列 |
| I1 css-true 切分幸存 i（首/尾/中段） | 3 | 0 | ⑥ css-true 臂 |
| I3 css-false decl ≡italic → i（首/尾） | 2 | 0 | ⑥ css-false 臂 |
| I6 address head css-true | 1 | 0 | ⑥ css-true 臂 |
| collapsed qS（跨 case 状态面） | 1 | 1 | 未动（隔离绿，见下） |
| 跨块 `<i><p>foo[<b>bar</b>}</p>` | 2 | 2 | 未动（后续批） |
| oblique 壳内嵌套（双 flag） | 2 | 2 | 未动（后续批，I3c 代表） |
| I3a qS after 读面 | 1 | 1 | 未动 |

- 断言级：**F2P=12 / P2F=0 / newF=1（d8diag I3c 新探针已知残红）/
  goneF=0**；六文件之外 43 探针逐名恒等（d7diag 20/20、d6diag 24/1
  恒态）。
- italic 18F→6F；bold 13F 恒等（notbold 首版回归已由两轴化消除）。

face 2（multitest + misc 哨兵）：base 9997P/2F/1T → after
10012P/2F/0T：multitest fail 集 2 行逐名恒等，P 差 +15 全落
`?9001-last` 慢页门杀进度差（base 该片 300s script 预算中途击杀
999P+Timeout，after 完片 1014P——承 d7 批非确定性窗口口径）；misc
423 断言逐名恒等。

定点钉测重跑实测 **7 passed / 0 failed / 0 ignored**（nested_cross_b
已由并行流 D1 刀解钉，d7 批账面 6/0/1 为旧态）。

## 残余与下一步

- italic 6F：collapsed qS ×1（探针隔离绿 + 语料红 = 跨 case 状态污
  染面，非单 case 缺陷）、跨块 i>p ×2、oblique 壳内嵌套 ×2（Chrome
  录形 = partially-selected decl 壳保壳内裹，非抽取——需嵌套臂）、
  I3a qS after ×1。
- bold 残余 13F（700/900 壳 partial、跨块 b>p、qS 杂项）、u/s 20F、
  sub/sup 26F 未动（master ② 清单）。
- d6diag 余红 1F（D19 strong-mid fresh b）留作 ② 簇代表。
