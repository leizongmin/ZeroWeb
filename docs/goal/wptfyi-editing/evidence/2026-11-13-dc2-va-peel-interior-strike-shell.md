# DC-2 · d14 批：strike S1 族内部界 peel + sub/sup 残段保面 + s 壳收编序列化

日期：2026-11-13。前置：d13 批（[sub/sup va 冲突壳](2026-11-13-dc2-sub-sup-va-shell.md)）。
基线：face 1 六文件 37F（bold 8 / italic 3 / underline 4 / strikethrough 16 /
subscript 3 / superscript 3），face 2 mt 断言 fail 2 + misc 22，哨兵电池
fail 名集 34，selection 全语料 35,668P/16,019F，钉测 execcommand 7/7。

## 结论先行

face 37F→22F（-15F，P2F=0）：strikethrough 16→7、subscript 3→0、superscript
3→0；bold/italic/underline 字节级恒等。四刀全落 `js_dom_shim/part01d.js`：

1. **S1 族内部界 peel（strike -8F）**：`<s>foo[b<span style=color>ar]ba
   </span>z</s>` + strike 解除——end 界落 color span 内部（ar|ba 间），旧
   composite 恒 -1 → end peel no-op → 整壳滞留 + 段尾 z 按壳值重裹。修：
   composite end 侧内部界解析（包裹整体随选区侧搬出，复合索引过槽位续爬升，
   逐层登记拖出段），peelOne 搬移后按壳 specified 值补裹拖出尾段。
2. **三门控（对账回归归零）**：首版解析+补偿无门致 face 37F→62F——bold
   数值壳嵌套形内部界误解析（P2F 3）、u/strike 壳 css-true 补偿序列化错
   （longhand 应 shorthand）、drag peel 端点回写偏移（qI-after ~16F）。
   门：解析限 underline/strikethrough 命令；补偿 longhand 限 `<s>` 壳 ×
   strike × css-true；drag 触发的 end peel 跳 `_zwVaPeelRestore` 登记。
3. **sub/sup 残段保面 retag（sub -3F、sup -3F）**：嵌套双 tag 壳 ADD 面语
   料十行统一规则 = 残段保自己原有的最深 va 壳面；strip 界剥后残段自然落
   外层壳，仅贴边 sandwich 全含形（选区两端同栖最内层壳）需 ADD 前 retag
   到最内层壳面（vaInT）。首版 retag 目标写反（vaOutT）+ 门过宽（误伤中间
   子级形/跨壳形各 2+1 形），face 对账暴露后按语料直读修正。
4. **s 壳收编序列化槽（strike S2c -1F）**：`fo[o<s>bar</s>b]az` + strike
   css-true——s 壳被 cv pass 摘除后 fresh wrap 收编落 shorthand，Chrome 录
   形 longhand（`text-decoration-line:line-through`，s 壳带 decl 合并形亦
   longhand）。修：cv 摘 `<s>` 壳时槽存壳 text-decoration 值面，forceValue
   尾臂 wrap 消费落 longhand 并壳值（canon 排序去重）。

## S1 族机理与修复形状

### 根因链（探针 JT dump 定谳）

end 界 (ar-tx, 2) 落 color span 内部 → `_zwVaEdgeComposite` 爬升到 span 层
idx=1 既非 0 非 plen → 恒 -1 → end peel no-op → `<s>` 壳滞留 → PD(b-tx)
pop 壳 clearValue 解包 + 子代重裹臂把未选中尾 z forceValue(line-through) →
fresh `<strike>z</strike>`（Chrome 录形 = 壳在 end 界切开、残段保原壳
`<s>z</s>`、拖出的 span 内未选中尾 ba 按壳值补裹）。

### 修复三件（peelOne end 分支）

- composite：内部界时包裹整体随选区侧，`idx = index(p)+1` 续爬升，逐层
  push `{host, idx}` 到模块级 `_zwVaPeelDragList`（-1 路径清单丢弃）。
- 补偿：搬移后对每拖出段的 `children[idx..)` 补裹——文本 + css-true + s 壳
  → 手搓 longhand span（`_zwVaWriteTdDeclOnly`）；余者 `_zwVaForceValue`
  （shorthand span / `<strike>`/`<u>` tag 由其分流）。空白跳过、spec null
  不补偿。
- 语料实证锚：S1a-h 形（color/id/font-size span、i-in-s，双 css）全绿；
  strike face F2P=8 名集 = 语料 `<strike>foo[b<span>…]` / `<s>foo[b<i>…]`
  / `<s>foo[b<span font-size>…]` 八变体。

### 三门的必要性（face 对账实证）

- **命令门**：`<b>foo<span style="font-weight:normal">bar<b>[baz]</b>quz
  </span>qoz</b>` + bold——span children=[bar-tx, inner-b, quz-tx]，E 爬升
  span 层 idx=2/plen=3 内部 → 无门时解析误触发 → 拖出 quz 补偿 forceValue
  重裹 → b-outer 溶解，逆录形（Chrome bold.js B 族外层壳整体保留不切）。
- **序列化门**：`<u>` 壳 underline css-true / `<strike>` 壳 strike css-true
  的补偿期望 shorthand span（underline.js:220、strikethrough.js:373）；
  longhand 仅 `<s>` 壳（strikethrough.js:311）。首版 u/strike 壳 css-true
  innerHTML 形对 decl 错（P2F 6）。
- **端点门**：drag peel 的选区尾界已随 movePreserving 骑在原边界叶（Chrome
  录形选区尾停最后选中叶）；peelRestore 回写把端点修进补偿壳 → qIndeterm-
  after 读数偏移 ~16 案。

## sub/sup 残段保面规则

### 语料十行统一规则（data/subscript.js / data/superscript.js 嵌套族直读）

残段（未选中叶）保**自己原有的最深 va 壳面**；与命令面同面者经 ADD wrap
sibCrit 并拢成单壳。N5 `foo<sub><sup>b[a]r</sup></sub>baz` + sup：残段最深
壳 sup=命令面 → `foo<sup>b[a]r</sup>baz` 单壳；N6 `foo<sup><sub>b[a]r</sub>
</sup>baz` + sup：残段 sub → `foo<sub>b</sub><sup>a</sup><sub>r</sub>baz`
三分裂。strip 界剥后残段自然落外层壳（探针 N1-N6 got 实证 vaOutT 落位），
故仅 sandwich 形需 retag。

### retag 门（首版两误的修正）

1. 目标 vaOutT→vaInT：首版按探针期望（自拟，N5/N6 恰好互换）写成 vaOutT，
   语料直读后纠正。
2. 门收窄：首版仅判 vaInT 非空，face 对账 P2F 三族——中间子级形
   `foo<sub>b<sup>[a]</sup>r</sub>baz`（start 爬升链 sub 层非贴边）、跨壳
   形 `<sup>fo[o</sup><sub>b]ar</sub>`（end 链无 vaInT 面）残段面本就正确
   不该 retag。终版门 = start 链逐层贴边 + end 链逐层贴边且链上遇见 vaInT
   同面壳。修正后 sup face 5F→0F（含 d13 既有 3F 全 F2P）。

### Fix 1（remat sub/sup skip，sub 3F→0F）

`_zwVaRematerializeCarries` general 臂对 sub/sup 命令 continue——flag-form
重物化对 sub/sup 无 span 样式形（Chrome 录形嵌套族 css-true 亦保 tag 载
体），误入即 span 化 + prop 链缺 sub/sup 臂落 `background-color:superscript`
junk 壳（N 族 6F 根因，d13 账）。

## s 壳收编序列化槽

流（探针 JT dump）：`fo[o<s>bar</s>b]az` + strike css-true → postCv=
`foobarbaz`（s 壳 contained 被 cv 摘）→ 三裸文本各自 FV wrap shorthand
span → 并拢单 span shorthand。语料期望单 span **longhand**。s 壳被摘后
DC-2 ① decl 壳分族臂无从见壳，故 cv pass 摘壳时槽存壳值面（`_zwVaSdCarry`
，setSelectionValue 轮次内生命周期，cv 循环前清零），FV 尾臂 wrap 消费：
prop 换 `text-decoration-line`、值 = 壳面 ∪ line-through canon 排序去重。

对照面（语料直读确证门不误伤）：u 壳/strike 壳 css-true 恒 shorthand
（`foo<u style=overline>[bar]</u>` = `text-decoration:overline line-through`
；strike 壳 REMOVE 族 shorthand）；underline 命令不消费槽；REMOVE 面无
wrap 不消费。s 壳带 decl 两行（`foo<s style=text-decoration:underline>
[bar]</s>`）批前已绿（DC-2 ① 臂路径），槽刀两路殊途同归不扰。

## 方法学教训

探针 famCase 的期望串两处自拟错误（N5/N6 superscript 互换）致首版 retag
方向写反、探针假绿掩盖语料真值——**探针期望必须逐条从 data/*.js 直读，
不得由 got 反推**。face 六文件断言级对拍是回归归因的最终仲裁（本轮三处
门控缺口全部由 P2F 名集暴露）。

## 账目

- face 1（六文件断言级，base=d13）：37F→22F，F2P=15、P2F=0。明细：
  strikethrough 2131P/16F→2140P/7F（S1 八变体 + S2c）、subscript
  1238P/3F→1241P/0F、superscript 1270P/3F→1273P/0F（F2P 三条 =
  `<sub><sub>` swcT/N5 swcT/N6 swcT 嵌套族）、bold/italic/underline
  恒等。摘面包屑后干净版重跑六文件与带面包屑版 fail 名集逐文件 diff
  恒等（strike 仅 S2c 转绿差异）。
- face 2：mt 断言 fail 名集 2=2 恒等（mt-g 全页 test-guard 950s 守卫跑，
  d13 基线单元无墙钟限跑 ~15 分钟不可复刻；d14 多出 `?9001-last` 窗口级
  Execution timeout 为 d13 未达窗口的伪失败，非断言、不可比）；misc
  22=22 恒等。
- 哨兵电池（47 页 zz-probe-*）：fail 名集 34=34，P2F=0/F2P=0。
- 钉测：test-guard 包裹 `cargo test -p zero-engine --lib execcommand`
  7 passed / 0 failed（独立 CARGO_TARGET_DIR；冷 target 编译期内存峰值
  5.2G+，守卫 total-mem 5G + CARGO_BUILD_JOBS=2 双杀两次后 2 作业并行过
  ——后续冷 target 钉测沿用此参数）。
- selection 面（部分语料名集级）：`selection/` 全部 86 页 + `editing/other`
  12 快页 + event.html 逐页分段跑（全量模式 950s 守卫超时、runner 输出尾
  flush 缓冲全丢后改逐页分段）——fail 名集对拍 **P2F=0**；editing/other
  余 ~47 慢页（delete/forwarddelete mutation 面，单页 ~110s）未跑全，名集
  覆盖差非回归：本轮四刀全在 format 命令 va 管线，delete/forwarddelete 不
  经 format_apply。补全留下一批（或沿用 d12 行为态近似口径）。
  分段跑法注意：runner `testharness-selection <页名>` 为路径子串过滤，case
  路径前缀 `selection/` 或 `editing/other/`；全量单次跑 ~15 分钟超常规守卫
  墙钟，需分段。

## 残余账目与下一批方向

- strike 7F：S2a/S2b/S2e（ADD 面邻壳不吸收 + range-start face 方向规则
  ——sibCrit merge 对 u/s tag 形按 tag 名判同的收窄面，语料 S2a
  `fo<strike>o</strike><s>bar</s>` fresh wrap 不并入邻 `<s>`）、M1
  （multipara 裸 span 溶解）、`<strike>…` qI-after、`foo<s>[bar]</s>`
  qI-after/qI-before 两条（切片伪失败族待复核）。
- bold 8F / italic 3F / underline 4F：d13 分类维持（真差异 b-in-i 块边
  界嵌套、cssFlag qS 族切片伪失败等）。
- B1-B4（b-in-i 块边界 toggle-off per-p 拆壳）与 M1/M2 同属 multipara/
  跨块族，可并入下一批评估。
