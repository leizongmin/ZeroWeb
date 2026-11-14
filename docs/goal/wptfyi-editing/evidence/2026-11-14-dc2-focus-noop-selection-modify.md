# DC-2 · d19 批：focus no-op 族 + Selection.modify 简形实现

日期：2026-11-14。前置：d18 批（[Selection 三规范校验门](2026-11-14-dc2-selection-validation-gates.md)）。
基线：selection 全语料残余 2,806F 中 modify 域未实现（`Selection.modify` 整方法缺失
→ 语料 setup/移动面成片 TypeError/no-op 红）、editing/other typing-around-link 族
setup 首步 `window.focus()`/`body.focus()` 抛 TypeError → 整 promise_test
unhandled rejection；insertparagraph/delete-in-child-of-head tentative 族
`contentDocument.documentElement.setAttribute` / `contentWindow.focus` 同因全红。
本批四文件落 `crates/engine/src/js_dom_shim/`（part01b +9 / part03 +7 /
part05 +29 / part06 +228，纯新增无改写）。

## 修复内容（spec 依据）

1. **focus no-op 族**（HTML §7.3.2 focusing steps，
   https://html.spec.whatwg.org/multipage/interaction.html#dom-window-focus ）：
   headless 单窗面无用户可见聚焦转移可观测 → no-op 不抛，同
   alert/confirm/prompt/open/print/stop 既有 dismiss 语义（`||` 幂等守卫）：
   - 主窗 `globalThis.focus`（part01b，typing-around-link 变体
     `utils.window.focus()` setup 首步）；
   - 子窗对象 `focus`（part05，`iframeElement.contentWindow.focus()`）；
   - body 视图 `focus()`（part03，DesignMode `utils.document.body.focus()`）；
   - docEl `focus()`/`blur()`（part05）。
2. **docEl 属性写侧**（part05）：两条 docEl 路径（R292 提取只读
   get/hasAttribute、R177 合成无属性面）此前均无 setAttribute——
   insertparagraph-in-child-of-head 族
   `documentElement.setAttribute("contenteditable","")` TypeError。写侧入
   本地覆盖表，get/has 读侧先查覆盖再回落既有实现（不触碰底层存储，最小面）。
3. **Selection.modify(alter, direction, granularity) 简形**（part06；spec
   https://w3c.github.io/selection-api/#dom-selection-modify ，legacy
   WebKit/Gecko API）：沿文档字符边界格移动 focus。格模型 = body（退化
   documentElement）文档序枚举单元（每文本字符 + `<br>` 各一单元；SCRIPT/
   STYLE/TEMPLATE 不入格），边界 = 相邻单元空隙、跨节点同界去重合一
   （(text,len) ≡ (nextText,0)）；character 一步跨一单元，forward 落尾界
   prevText:len 形、backward 落前界 nextText:0 形（Chrome 录音对齐）；
   alter=extend 复用 extend 的 anchor 保持、alter=move 复用 collapse。
   编辑宿主 scope：focus 沿祖先链找 contenteditable(''/true) 根，宿主内
   contenteditable=false 子树按单个不可编辑单元跳格。word 粒度 = 跳空白后
   连续同类词段（四类：空白/拉丁数字/Hangul/其余符号；FIXME(uax29) 完整
   UAX#29 细类不在简形）；lineboundary = 最近块级祖先文本首/尾界（BR 切行；
   FIXME(visual-modify) 真语义需行盒几何）；line 粒度 no-op。direction
   left/right 视觉轴按 focus 前一字符双向类近似反转（FIXME(visual-modify)/
   FIXME(uax9)，混合双向段整段可视跳转不在近似面）。参数非法/空选区/无根/
   端界 → no-op（spec abort these steps 不抛）。

## 验证（同机同命令 before/after）

方法学同 d18：pre 二进制 = HEAD（df85ed2a6，/tmp/d19pre，09:09 构建）；post
= 最终树重建二进制（11:15 构建，与上一 session 09:28 快照二进制
/tmp/d19post 嵌入 shim 逐字节恒等——全量 include_str! 拼接串双二进制包含
同一 8,925,335 字节序列，尺寸差 18KB 为 codegen 布局噪声）；`--json` 断言级
ledger（按 (file, test name) 集合差，test-guard 4G/8G 包裹）。

**selection 全语料（666→667 文件）pre 47,149P/2,806F → post 47,320P/2,638F：
F2P=168、P2F=0、真 new-fail=0**：

| 族 | F2P | 明细 |
|---|---|---|
| Selection.modify | 143 | bidi modify.tentative 36 + move/extend-by-character 22+22、contenteditable modify.tentative 15 + non-editable-span 16 + inline-boundary 18、顶层 modify.tentative 7、word 残面（korean forward 1、symbol backward/forward 2、generated-content 1、trailing-inline-block 1、extend-backward-on-input 1）等 |
| focus no-op 族 | 22 | typing-around-link collapsed 3+3、non-collapsed 4×4（ContentEditable/child=b/parent=b/parent=b&child=i 四变体） |
| 本批探针 | 3 | zz-probe-d19focus（临时不入库） |

名义 new-fail=1/vanished=1 = 同一页 `selection/selection-nested-video.html`
（媒体页，10-27 入库）三账全 Timeout：pre/selfull2 超在 test 级、selfull3
超在 harness completion 级——键名随超时阶段漂移，非行为翻转（该页在并行流
workspace 全量编译同机重载下 chronic 页级超时，d18 账内同红）。

- **上一 session selfull2 账（F2P=164）勘误**：selfull2 于 09:27 最后一次
  part06 编辑（modify word backward 落点形修正）前发射，实测其与 selfull3
  差 4F 全落 word 落点形域（korean forward、symbol backward/forward、
  generated-content）——以 selfull3（最终二进制 11:46 完跑）为终账。
- 钉测复核（17 文件 206 断言，modify + move-by-word 两 pattern）：最终树
  重建二进制 vs 上一 session 快照产物逐断言恒等（mismatches=0）。
- 常驻探针 zz-probe-d19focus 3/3 绿、zz-probe-d19word（临时不入库）。
- 定向 probe 残面（终态）：bidi modify.tentative 36/36、contenteditable
  modify.tentative 15/15、non-editable-span 16/16、顶层 modify.tentative 7/7
  全绿；残余 57F = 鼠标驱动 modify（2/7×2+3/7，事件集成面）、line-flex/
  grid 0/2×3（FIXME(visual-modify) 行盒几何）、inline-boundary 18/36
  （视觉边界）、bidi character 22/28×2（RTL 视觉轴近似）、word 细分
  （korean 2/4、symbol 2/6、generated-content 1/2）。

## 零回归腿

- **sentinels**（FILTER=zz-probe 54 文件）：P2F=0、F2F=0、new-fail=0；
  P 2855→2883（+28 组成漂移全 Pass，并行流加页）、F 43=43。
- **editing face 六文件**：ledger 逐名恒等（bold 9F、italic 3F、underline
  3F、strike 4F、sub/sup 0F），与 d17 终账一致——零位移。
- **input-events 全语料**（23 文件）：逐名恒等 210P/907F——零位移。
- **引擎 lib 钉测**（test-guard + 独立 CARGO_TARGET_DIR=/tmp/d19tgt）：
  1586 passed / 7 failed，7F 为 d18 定谳的 main 既有欠账集（img/svg settle
  + fetch/iframe-SW/response-stream/t8k-websocket 族），与本批零交集。
  钉测于 09:22 跑（part06 末次编辑 09:27 前）；09:27 后 JS 差量仅 modify
  word 落点形，grep 证 Rust lib 测试无 `.modify(` 调用面——钉测结论覆盖
  终树。

## 账目

- 断言级：selection 全语料 F2P=168 / P2F=0 / 真 new-fail=0（vanished=1 =
  nested-video 超时键漂移）；focus 族 F2P=22；face/sentinel/input 零回归。
- 全语料 P：47,149→47,320（+171，含探针页自增 3+1 断言面）；较 11-12 CI
  快照（d18 后口径 47,147P）+173。
- 残余 modify 域 ~57F 属视觉/鼠标/细词类近似面（FIXME 已标），后续按簇
  评估是否进入精化批；三目录预估缺口进一步收窄，正式落账待下周 DC-3 快照。
