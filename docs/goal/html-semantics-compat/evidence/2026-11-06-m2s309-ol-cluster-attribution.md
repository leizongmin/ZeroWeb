# M2 s309 — the-ol-element 6 红页簇归属勘根：根因 = layout-engine R109 Inline 片段内 plain inline 的块级后代无 hoist 路径（嵌套 block-in-inline 内容丢失），整簇移交渲染流域（本流零施工）

日期：2026-11-06。切片号 s309（承 s308 下一步候选①，勘根-移交型切片）。

## 勘根

**簇面**：the-ol-element 目录 reftest 12 对入通道（6✗ + 6✓）。红页
1.03%–3.85%：grouping-ol-rev-reftest-001（1.03%）/ grouping-ol-start-
reftest-001（2.67%）/ grouping-ol-start-reftest-002（2.76%）/
grouping-ol-type-reftest-001（3.24%）/ grouping-ol-type-reftest-002
（3.85%）/ grouping-ol-type-reftest-003（2.11%）。绿页 reversed-1a..1e +
reversed-2（0.02%–0.04%，ref 侧为裸 `<ol><li value>` 无嵌套结构——marker
machinery 健康锚点）。基线日志
[evidence/logs/s309-solo-dump.log](logs/s309-solo-dump.log)（承前 session
429 中断前产出）。

**fixture 形态**（六红页同模板，2014 dzenana 系）：test 侧 `<span>` 内
`<p>` 描述行 + `<ol>`（type/start/reversed 语义变体）+ 空 `<li>`
（decimal marker、`list-style-position:inside`、monospace、
`ol{padding-left:5em}`）；ref 侧同位放嵌套
`<span><p>1./2./3.</p></span>` 纯文本序列（`span span p{padding-left:5em;
font-family:monospace}`，且 `span p{display:list-item}`——chrome 实测
outside marker 落视口左缘外不可见，两衫一致）。pass 判据 = ol marker
序列与纯文本序列渲染一致。

**oracle 定谳（google-chrome headless 本地，800×600）**：六对 test/ref
**全部 0/480000 逐字节相等**——正确引擎中 ol marker 面与纯文本面像素
等价，pair 语义健全；ZW test 侧渲染与 chrome 期望结构一致（描述行 + 5em
monospace marker 序列，目检）。日志
[evidence/logs/s309-probe-oracle.log](logs/s309-probe-oracle.log)。

**像素取证（REFTEST_DUMP 全簇，target/reftest-dump/ 会话级）**：ZW ref
侧渲染中——外层 span 的**直接** p 子（描述行）正常渲染；**嵌套**
`<span><p>…</p></span>` 内容整体消失（每页 12 行全缺，后继内容整体上压、
页高缩短）。ZW test 侧正常。

**根因链（单一，纯 layout-engine，渲染流域 crate）**：

1. 外层 span 含**直接**块级子（p）→ R109 split host
   （`tree.rs:4213` `inline_has_block_child`，只查直接子）；直接 Block 子
   走 `InlineBlockSegment::Block` → build_subtree 正常建盒（描述行可见）。
2. Inline 片段内的 **plain inline 项**（内层 span，`DisplayValue::Inline`
   非原子/非替换）不入 build_subtree，作为 IFC 片段内容被扁平收集
   （`tree.rs:4264` atomic 过滤只收非 Inline display/替换元素；R4944
   nested_atomic 降下收集只覆盖**原子**后代——
   `inline_elem_has_nested_inline_block` BFS **在 block-level 后代处停止
   下探**，`inline/mod.rs:1755` 注释「block-level 后代停止 = R109 另路径」）。
3. `collect_inline_items` 遇 block-level 子发 `BlockBreak` 并**跳过其内容**
   （`collect_items.rs:770-777` 注释「block-level 子元素由 taffy 正常布局
   为独立的块盒」+ `:822-844`）——该假设只对 split host 的直接块级子成立；
   Inline 片段内嵌套 inline 之下的块级后代**无人建盒** → 内容丢失。

**触发隔离（探针三连，zz-s309-* 页完测即删，spec 如下）**：

| 探针 | test 结构 | ref 结构 | 结果 |
|------|-----------|----------|------|
| direct1 | `<body><span><div>ZW309DIRECT</div></span></body>` | `<body><div>…</div></body>` | ✓0.00%（直接块级子 hoist 正常） |
| nest2 | `<body><span><span><div>ZW309NEST</div></span></span></body>` | 同 direct | ✓0.11%（外层 span 无直接块级子非 split host，普通 build_subtree 递归抵达内层 span，内层自身成 split host 正常渲染——**非 Inline 片段路径的嵌套 block-in-inline 健康**） |
| nesthost | `<body><span><div>AAA</div><span><div>ZW309NEST-01..12</div></span><div>BBB</div></span></body>` | `<body><div>AAA</div><div>T01..12</div><div>BBB</div></body>` | **✗1.57%**（7535/480000，Inline 片段路径 12 行嵌套块级内容整体消失） |

（探针页均带 `<link rel="match" href="…-ref.html">`——loader 按链接注册
case，`-ref` 命名约定不自动生效；样式统一 `body{margin:8px;font:16px
monospace}`。）

**阈值假绿注记（移交验证须知）**：单行实例低于 runner 1% 阈值——nesthost
未放大版（单行）实测内容丢失但 0.15% 假绿（`REFTEST_DUMP` +
`REFTEST_DUMP_PASS=1` dump 目检证实）。即该缺口单行形态对当前阈值不可见，
验证车须用放大版（≥12 行）；语料中潜在单行受累页可能以假绿在册。

**归属判定**：整簇移交**渲染流域**。根因单一纯 layout-engine（R109
Inline 片段内 plain inline 的块级后代无 hoist 路径），零 engine/dom/shim
足迹；layout-engine 属渲染流 crate 清单（rally 规则 9 工作面不重叠），
渲染流当前活跃于 layout-engine（R5434 col used width）。移交落点 =
渲染流控制面（rule 10 各流只写自己控制面，本账本为移交输入）。

**恢复路径注记（供渲染流）**：修复方向 = Inline 片段 arm 对含块级后代
的 plain inline 项递归 build_subtree（或等价的嵌套 split-host hoist）。
修后六页预期翻绿——oracle 已证 test/ref 面等价，ZW test 侧 marker 面由
reversed-1a..e 0.02% 锚定；残差风险 = marker 与文本行盒度量微差（inside
marker 基线 / monospace 继承进 marker），修后 solo 复测即知。

## 施工

零（勘根-移交型切片；并行规则工作面不重叠，本流不触 layout-engine）。

## 验证

- 簇基线 solo 12 对（6✗/6✓）——[evidence/logs/s309-solo-dump.log](logs/s309-solo-dump.log)。
- chrome oracle 六对 test/ref 逐字节相等（0/480000 ×6）——[evidence/logs/s309-probe-oracle.log](logs/s309-probe-oracle.log)；截图 /tmp/s309-oracle/（会话级）。
- REFTEST_DUMP 全簇 13 PNG（6✗ test/ref 对）+ 探针 dump——target/reftest-dump/（会话级）。
- 探针三连 solo（direct1 ✓ / nest2 ✓ / nesthost ✗1.57%）——同 probe-oracle 日志。
- 源码锚：`tree.rs:4213`（split host 判定）；`tree.rs:4233-4330`（Inline
  片段 arm 原子过滤 + R4944 原子降下）；`inline_block_split.rs:74`
  （inline_has_block_child 直子判定）；`inline/mod.rs:1695-1760`
  （BFS 停止于 block-level）；`collect_items.rs:770-844`（BlockBreak 跳内容）。

## 下一步候选

1. 生产通道动画装载接线（s307 ③，engine 网络路径 decode_image_animation
   ——生产 img 动画 GIF 维持首帧静态；本流域内施工候选）。
2. 小簇排队其余（textarea 4/meter 4/input 3/link 3/embed 3/object 2/
   option 1/iframe 1/wbr 1）——逐簇勘根；br 1 勘根前先 solo 复核
   editing-compat br 刀（fe760dc57 iol/iul 收口）外溢现势。
