# M2 s156 · LinkStyle sheet 创建门收口——type essence 门 + alternate 空 title 门

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s155（rel=preload fetch +
转换复用收口）

## 入场定刀位

s155 下一步候选①：LinkStyle sheet 属性面 1F（styling/LinkStyle.html「The
LinkStyle interface's sheet attribute must return null; the disabled attribute
must be false」——expected null but got object）。

## 勘根（探针一轮 + getter 定位）

1. **失败消息精确**：`style2.sheet` expected null but got object——style2 =
   `<link rel="alternate stylesheet" type="text/css" title="">`。同文件其余
   4 案 Pass（not-in-Document null 案、must-return-StyleSheet 案——style5/6/7
   sheet 对象存在、title/media 反射案）。
2. **getter 定位**：R3036 `element.sheet`（part04 set trap prop==='sheet'
   臂）——LINK 只有 rel 正则门（`/\bstylesheet\b/i`），无 spec 创建条件建模：
   ①type essence 门缺；②LINK alternate 空 title 门缺。
3. **探针钉态（zz-s156-probe，验后删）**：p1（rel=text）sheet=null ✓；
   p2（alternate+空 title）/p3（alternate 无 title）/p4（style type=
   "text/html"）sheet=OBJ 全应 null；p5/p7（stylesheet / style text/css）
   OBJ ✓；disabled 全 boolean false（通用反射已达标，无需 disabled 面）。
4. **spec 创建条件**：CSSOM create a CSS style sheet——type 属性存在且
   essence 非 text/css → abort（`TeXt/CsS` 小写归一创建、`text/cſs` 不创建
   ——JS toLowerCase 对 SHARP S 不归一，语义同向）；HTML the-link-element
   styling——rel 含 alternate 且 title 属性值为空串（缺失即空串）→ abort。
5. **位移面收敛**：语料 .sheet 读取点全查——the-style-element 族
   （mutations.window.js createElement 无 type、non-parser-created-doc 无
   type）gate 跳过不受影响；type 变体（TeXt/CsS、text/cſs）在 reftest 渲染
   面不读 .sheet。disabled 断言链 style1-4 已 boolean false。

## 施工（part04 sheet getter 净 +28）

两门前置于既有 LINK rel 门：

1. **type essence 门（STYLE/LINK 共用）**：type 属性非空（trim 后）时取
   essence（split(';')[0] trim lowercase）≠ 'text/css' → 返 null；type 缺省
   不设门（缺省即 text/css 语义）。
2. **LINK alternate 空 title 门**：rel 分词含 alternate（ascii 不敏感）且
   title 属性值空串（`|| ''` 归一缺失）→ 返 null；style6（alternate 非空
   title）照常创建。

## 验证

- **语法**：规范序拼接 node --check SYNTAX_OK 110429 行（s155 110401，净
  +28）。Rust 面零改动（clippy/fmt 零波及，未跑）。
- **靶面定向**：LinkStyle.html **5/5 全 Pass**、MAKE_EXIT=0。
- **全量案级 join（对 s155 /tmp/zw-s155-full.log，(key,state) 对集直接
  去重）**：**8082 对 = 8082 对**（7198P/797F/87T → 7199P/796F/87T）——
  **F2P 1（靶案）+ P2F 0 + other 位移 0 + LOST 0 + GAINED 0** 全域零位移
  （the-style-element 族、CSS-in-JS 面、s150-155 修复面全保持）。
- 探针语料（zz-s156-probe.html）验后已删（工作树零残留）。

## 挂账（本切片更新）

- **收口：LinkStyle sheet 属性面 1F**（styling/LinkStyle.html 5/5 全绿，
  「LinkStyle sheet 属性面」挂账销案）。FIXME 在册：sheet 创建未建模 fetch
  完成/加载中状态（getter 纯 attr 门——上游 sheet 在 load 完成前可 null，
  语料以 window load 后断言豁免此面，暂不建模）。
- **承袭不变**：link-rel-attribute 2F（sheet 应用/removal 渲染面——跨域
  边界待归因，见下一步）；link-rel-attribute-tokenization 1F；link-load-event
  document load 阻塞面 1F；stylesheet-media-change-no-fetch 1F；stylesheet-
  not-removed-until-next-stylesheet-loads 1F；meta long × corpus 通道放宽
  失效；details 族 lib flake；promise_test 串行启动面；innerHTML 身份分裂
  表示面；sel 重解析同槽同 href 印记失配边角；待决三项须用户拍板。

## 下一步（s157 候选）

①stylesheet-media-change-no-fetch 1F（media 变更不重取门——helper 同型
扩展面）；②link-load-event document load 阻塞面 1F；③link-rel-attribute
2F 归因轮（sheet 应用/removal 渲染面——getComputedStyle 绿色断言挂样式
应用管线，属渲染流域边界候选，先归因再定刀法；shadow DOM 案 getComputedStyle
返 '' 另根）。
