# DC-2 切片 8b（消毒刀 r263）：sanitizer 流消毒管路收口（2026-11-24）

## 本刀范围（纯 JS shim，2 文件，无 Rust 改动；核验方代跑尾链收口——原写者
## 04:17 终版编辑后静默，session b1533150 系按账本认领修复迭代+落库）

- `part13-sanitizer.js`（+62/−24 vs HEAD）：
  1. **⊆ 校验门控**：per-element removeAttributes ⊆ 全局的 TypeError 仅对
     用户显式非空全局 removeAttributes 强制——空数组是 remove 方向 canon 缺省
     合成形非用户意图，standalone per-element removeAttributes 合法
     （FIXME(spec-gap): 混合全局+条目 ⊆ 约束非现行 spec 步骤，待语料该形
     在席后复核）。**炸页归因**：旧版该 TypeError 在 basic-filtering 第 65
     测试处未捕获抛出杀整页 runner——attributes-per-element 段余 98 测试未
     报告；门控后页面复活＝+98 测试面世（+48P/+50F 新见挂账如实入册）。
  2. **template contents 子树同域消毒**：`n.content.childNodes` 也走
     filterList（mock 面 template 子在 content fragment 不在 childNodes——
     旧版整跳 content 致 per-element 属性方向模板内不生效 12 案）。
  3. **属性方向并集**：全局与 per-element 条目 attributes/removeAttributes
     同向并集生效（hitGRem/hitERem/hasAllow/hitAllow 四算——任一 allow 列表
     在席即 allow 向，remove 命中或全不命中才剥除；旧版仅全局 attributes
     在席时查 elEntry，纯 element 条目方向整面落空 24 案）。
  4. **qname 前缀 ns 派生桥（外来内容限定）**：SVG/MathML 元素上
     namespaceURI 缺失的 qname 前缀形（xlink:href）按
     adjust-foreign-attributes 表派生真 ns 参与方向匹配——HTML 内容不派生
     （spec：HTML 内容 xlink:href＝null-ns qname 形，{href,XLINK} 条目必须
     不命中剥除＝names #3 形）。桥仅方向块使用，on*/data-* 安全门仍用原始
     ans 判定不过桥。
- `part15-positional.js`（+42/−10 vs HEAD）：
  5. **config 快照语义**：流创建时 resolve 快照入 state（实例形态
     `new Sanitizer(san.get())` 深拷贝）——流期间调用方变异不影响已建流
     （「mutating the sanitizer while streaming」12 案）。
  6. **prepend 写序（两迭代收口）**：首批锚容器当前首子（r263 首版
     `lastIns?nextSibling:null` 丢回退＝anchor null 尾插，insertion-logic
     8 案 expected "A<span></span>" got "<span></span>A" 实证回退丢失），
     后续批插上一批末节点之后（双写案 got "21" 反转形修复保留）。
  7. `_p15Sanitize` 第四参快照实例接线（缺席＝单发管线即时解析不变）。

## 结果（对 s8 态基线：domparsing 1212P/323F/1T、san 360P/298F/1T）

- **domparsing 1212P/323F/1T → 1264P/271F/1T＝+52P/−52F，五页全限定、
  零未知漂移、零回归**（FILTER 预言 +52 精确兑现）：

| 页 | s8 | 8b 终态 |
|---|---|---|
| stream-html-sanitizer.html | 154P/50F | **204P/0F 全绿** |
| stream-html-sanitizer-safe.html | 28P/2F | **30P/0F 全绿** |
| stream-insertion-logic.html | 52P/0F | 52P/0F 持位（prepend 修复回绿） |
| stream-positional.html | 64P/18F | 64P/18F 持位逐位同形 |
| with-trusted-types-pass-trusted.html | 6P/4F | 6P/4F 持位 |

- **sanitizer-api 360P/298F/1T → 414P/342F/1T＝+54P/−44F**（六页构成：
  basic-filtering +48P〔炸页复活，50F 新见挂账＝attributes-per-element 段
  首次入扫〕、get +1〔8P/1F→9P/0F 全绿〕、sethtml-tree +3、names +2
  〔13P/7F→15P/5F：#7/#9/xlink-inside-svg 三修，#2/#5/mixed×3 残〕；
  F 净 +44＝复活新见 50F − 修复 6F，非回归）。
- **trusted-types 1538P/552F/64T 逐位恒等**（桥 v2 重建后复跑）。

## 修复迭代记录（三轮链验证，全量腿判读字节绑定）

1. **chain3**（写者终版字节）：domparsing 1250P/285F——sanitizer/safe +52
   兑现同时 insertion-logic 8F 回归＋positional +6F＝prepend 首批锚回退
   丢失（写者「双写案 21」修复吞掉旧形 `firstIns||firstChild` 回退臂）。
2. **chain4**（prepend 修复）：insertion-logic 52P/0F 回绿、positional
   64P/18F 与 s8 逐位同形、domparsing 1264P/271F 终形——+6F 回归全数归因
   单一修复点，模板 4F 经代码路径分析定谳 s8 既有挂账非本刀回归
   （s8 挂账「template content 路由 ×8」吻合）。
3. **chain5/6**（names 桥两版）：v1 无差别前缀派生修复 #7/#9/xlink-svg
   但反噬 #3（HTML 内容错误重挂 XLink ns）→ v2 外来内容限定门——#3 回
   Pass、#5 回 s8-Fail 形，names 15P/5F 终态＝对 s8 严格改善零回归。

## 验证

- 全量 domparsing/san/TT 三腿（test-guard 包裹 1800s 窗，chain6 终判绑定
  part13 `a0bc4794…`＋part15 `0be544ca…`＋binary 05:02:33）。
- engine lib **4252P/0F**（chain3，兄弟轮 registries WIP 入树前二进制）；
  v2 复跑 4251P/1F——唯一失败 `js_dom_bridge::tests::
  test_expando_enumeration_r3046`（for...in 多出 `customElementRegistry`
  键）**归属 s9-registries 兄弟轮在飞 WIP**（part01/03/05/06 四文件全命中
  该键、本刀两文件零命中），非本刀提交内容。
- fmt 零 diff、clippy-engine 0（chain3）。
- 混合二进制判读佐证：树内另有 registries WIP（part01/03/05/06），本刀
  判读页（stream 五页+san 七页+TT）与其编辑域（customElementRegistry/
  attachShadow option/createElement option）不相交；domparsing 全腿
  chain4→chain6 逐位恒等交叉佐证零外溢。

## 残余挂账更新

- ~~stream-html-sanitizer 50F~~ **清零**（元素配置管路 ×24、模板内容 ×12、
  config 快照 ×12、foster rWC ×2 全数兑现）。
- stream-positional 18F 持位（replaceWith 双参 ×10〔got "AAB" 占位复写形〕
  ＋template content 路由 ×8——engine 代管父，协议刀域）。
- pass-trusted 4F 持位（before/after 臂 processed options 未透传）。
- stream-html-with-declarative-shadow-root 18F、stream TT sanitizer 8F、
  stream-custom-element-sanitizer 1T 持位。
- sanitizer-api 新见挂账：basic-filtering attributes-per-element 段 50F
  （复活面世，素配置深项）；names #2/#5/mixed×3 残（mock 属性面
  namespaceURI/qname 保真深项——namespaced 匹配完整修复需 parse 面工程）。
- 协议刀域不变：insert_adjacent_html 36F、createContextualFragment 11F、
  stream 路由尾——`InsertAdjacentHtmlOnHandle`（zero-protocol +
  apps/renderer 两端同步；s9-protocol-knife-claim.md 已认领）。
- XML 域深项挂账不变（DOMParser xml 族 ~60F、XMLSerializer 30F、
  xml-serialization 17F、innerhtml-mxss 30F）。

A/B 与迭代链日志存 /tmp/r263-*（tmpfs 易失）；本档 tally 与逐文件 diff
记录即判读主证，s8 基线逐文件表见切片 8 证据文。
