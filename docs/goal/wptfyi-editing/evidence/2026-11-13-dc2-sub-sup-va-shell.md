# DC-2 · d13 批：sub/sup vertical-align 冲突壳 + 语料红账重分类

日期：2026-11-13。前置：d12 批（[bold toggle-off 宇宙面定谳与桥层修复](2026-11-13-dc2-bold-toggleoff-universe-diag.md)）。
基线：face 1 六文件 57F（bold 8 / italic 3 / underline 4 / strikethrough 16 / subscript 13 / superscript 13），
face 2 mt fail 名集 2 + misc 22，selection 全语料 35,668P/16,019F。

## 结论先行

1. **d9 账目②瓦解**：bold 900 corpus css=false 双 F 与 d9 分诊的 cssFlag
   生命周期缺口（qS(stylewithcss) before=true 族）不是页内跨 case 状态
   污染，而是**语料切片割裂伪失败**——run 页 meta variant（?2001-3000
   等）把条目拦腰割断，注册期副作用无条件执行而 execCommand 闭包只在
   窗口内执行（subsetTest 跳过 test 创建，test() 同步立即执行），割裂
   条目的窗口内断言跑在未执行命令的 DOM 上 → got=输入原样。Chrome 在
   wpt.fyi 同红，harness 固有。
2. **sub/sup va 族 20F 修复落地**：sub/sup ADD 面对祖先 inline
   vertical-align decl 壳（任意值）按 Chrome removeInlineStyle 建模先清
   后裹——face 1 subscript 13F→3F、superscript 13F→3F（F2P=20/P2F=0），
   余五文件字节级恒等，face 2 / selection 恒等，钉测 7/7。
3. **六文件 37F 全谱重分类**（真值=Chrome 录音）：伪失败 ~17F（切片割
   裂：bold 6F = 条目 139/75/204 三割裂条目；italic 1F = 条目 131；
   underline 2F = 条目 65/133；strike 1-2F）+ 真差异 ~20F（sub/sup 嵌套
   双 tag 6F、b-in-i/b-in-s 块边界嵌套 bold 2F/italic 2F、multipara u 2F、
   strike 邻壳族 ~12F）。

## d9 账目②瓦解链

1. 900 corpus case（条目 139，tests 1999–2007，被 ？2001-3000 窗口起点
   割裂）got=输入原样，与 d12 隔离页 re-wrap 形不同 → 不是同一机制。
2. 断言数公式：每条目测试数 = ncmd + 2 + 6×查询键数（execCommand 闭包
   数 + 非编辑区检查 + innerHTML + 每查询键 6 断言）。条目 75 = 996–1011
   割裂于 1001；条目 204 = 2997–3012 割裂于 3001。
3. bisect 探针（`browserTests.slice(1-1,139)` 单窗口全跑）全绿 → 前序
   条目污染假设证伪；切片割裂机制由 harness 源码（subset-tests.js 跳过
   test 创建 + testharness.js test() 同步）+ 自定义 meta variant 探针
   （zz-probe-d13win-*，预测窗口名精确命中）实证。探针页已清理。
4. cssFlag 假说两轮证伪：无选区 styleWithCSS 驼峰探针 return false 是
   part06.js:5640 编辑上下文门（2026-11-05 deliberate，Chromium
   canEditRichly 建模）；带选区探针（zz-probe-d13swc2.html，留作本地
   哨兵）驼峰/全大写全部正常 → 无大小写缺陷，引擎 Chromium-parity。

## sub/sup va 冲突壳修复

### 形状（语料 data/subscript.js:399-417 / data/superscript.js:400-418）

- **F1 全包形**（12F）：`foo<span style=vertical-align:X>[bar]</span>baz`
  （X ∈ sub/super/top/bottom）+ 命令 → 期望 `foo<sub>bar</sub>baz`
  （摘壳换 tag；csstrue/cssfalse 期望同形——vendored 尾臂 effective 等值
  早退不落 style）。
- **F2 跨选区形**（8F）：`<sub>fo[o</sub><span style=vertical-align:X>
  b]ar</span>` → 期望 `<sub>foob</sub><span va>ar</span>`（被选段抽出
  裸落并入既有 sub；残段保壳保值）。
- tag 邻居同形（`<sub>fo[o</sub><sup>b]ar</sup>`）批前已绿——走
  specified tag face conflict 链 + sibCrit 合并，缺的是 va decl 面。

### 真值仲裁

vendored implementation.js 复现嵌套形（= 我们批前 got）：specifiedValue
对 sub/sup 仅认 tag face（:2223-2245）、clearValue 因 specified null 早退
（:2483）、forceValue 尾臂 span 弹出+递归后文本裹 `<sub>` 落壳内
（:2895-2939）。语料期望与 spec 算法相悖 → 语料按 Chrome 录音，修复按
Chrome removeInlineStyle（冲突 prop 直删 + 段抽取）建模。

### 改动（crates/engine/src/js_dom_shim/part01d.js，三处）

1. **Arm A（F2）** `_zwVaSplitConflictingInline` 链走补门：sub/sup 且
   newValue 非 null 且 specified null 时，祖先有 inline vertical-align
   decl 即入 conflict 链（peel 抽取）。end 侧裸移由
   `_zwVaCarrierResidualStyling` 既有门保证（sub/sup 不在克隆五命令集，
   恒裸移）；start 侧克隆壳剥 va decl（被覆盖段不继承冲突面，新面由
   wrap 供给）。抽取后 forceValue 的 sibCrit（specified 等值壳复用）自然
   并入邻接 `<sub>/<sup>`。
2. **Arm B（F1）** contained element clearValue pass 补臂：sub/sup ADD
   面、specified null 且有 inline va decl 的全含壳——剥 vertical-align
   decl（style 直写重写，`k:v` 规范形）；style 摘空且属性全空 → 解包
   （kids 升格），残余 decl 壳保壳。解包后 forceValue pass 重算集顶层
   裹 tag。
3. toggle-off（newValue null）面两臂均不动，维持 tag-only 既有行为。

### 验证

| 通道 | 结果 |
|---|---|
| face 1 断言级对拍 | subscript 13F→3F、superscript 13F→3F（F2P=20 全落 va 族 innerHTML 行，P2F=0/newF=0/goneP=0）；bold 8F/3040P、italic 3F/2070P、underline 4F/2143P、strikethrough 16F/2131P 字节级恒等 |
| face 2 | mt fail 名集 2=2、misc 22=22 恒等（mt 行数 7,865 vs 基线 9,832 为 ？9001-last 慢页截断窗口差，fail 名不受影响，承 d7-d12 口径） |
| 哨兵电池（45 页） | 唯一 Fail→Pass 翻转 = zz-probe-d13vashell 7 案（d12 二进制红/新二进制绿）；余页恒等，f6b/f6bwin 慢页截断 0F |
| selection 全语料 | 35,668P/16,019F 字节级恒等，fail 名集 16,019=16,019 零漂 |
| 钉测 | `cargo test -p zero-engine --lib execcommand` 7/7（test-guard + 独立 CARGO_TARGET_DIR） |
| 新哨兵 | zz-probe-d13vashell.html：F1×4（sub/super/top/bottom）+ F2×3 行为断言，已入库 |

## 残余账目（六文件 37F）

- **subscript/superscript 6F**：嵌套双 tag（`foo<sup><sup>b[a]r</sup>
  </sup>baz` 族，csstrue）——Chrome 做同 face 冗余外层坍缩 + 内层切分，
  我方产出 `<span style=background-color:rgb(0,0,0)>` junk 壳（css 化
  转换面把非本命令 tag 壳误入 style 化链）——独立缺陷族，下批候选。
- **bold 8F** = 6F 切片伪失败（条目 139/75/204）+ 2F 真差异（条目 82/83
  `<b><p>foo[<i>bar</i>}</p>…` b-in-i 块边界嵌套）。
- **italic 3F** = 1F 伪（条目 131）+ 2F 真（条目 105/106 同族）。
- **underline 4F** = 2F 伪（条目 65/133）+ 2F 真（multipara
  `<p>[foo</p><p> <span>bar</span> </p><p>baz]</p>` + 条目 134 u-qS）。
- **strikethrough 16F** = 1-2F 伪 + ~14F 真（b-in-s ×2、color/font-size
  邻壳、line-through 条目 137 等——精确分解下批入档）。

## 支线与处置

- 探针页清理：zz-probe-d13bisect/d13win-*/d13swc 已删；d13swc2（swc
  大小写 parity）留本地不入库；d13vashell 入库为哨兵。
- 构建/运行全程独立 CARGO_TARGET_DIR（/tmp/d13tgt）+ systemd scope，
  与并行流 D1 的 workspace test+clippy 共享 build 锁零碰撞（承 d12 碰撞
  记录同法分流）。
- 经验沉淀：[语料红账判读](../../../learnings/bugs/2026-11/2026-11-13-wpt-editing-corpus-truth-sources.md)。
