# M2 s319 — 小簇三连勘根（wbr/iframe/br）：全部落跨流或能力面，移交 2 + 结构性 1，零本流施工面

日期：2026-11-07。切片号 s319（承 s318 下一步候选①：wbr 1 / iframe 1 / br 1 逐簇勘根）。

## 勘根结论（三簇）

### 1. the-wbr-element：wbr-element.html ✗18.09% → 跨流移交 layout-engine break_lines

- 页面契约：`p{max-width:300px; font:15px/1 Ahem}` + ~570 字符无空格串以 ~50 个
  `<wbr>` 连接；ref 硬编码 `<br>` 对齐 Chrome 断行位（HTML §12.3 wbr = 断行
  机会非强制）。
- layout-dump 探针：**p h=15px 单行未断**——整串溢出 300px；wbr 盒零宽存在
  （R1682 零宽可见性已修）。根因 = R1682 显式留白：「断行机会语义（长词在 wbr
  处可断）是 line-breaker 增强，本 slice 只修零宽可见性」。
- 移交面：crates/layout-engine/src/inline/break_lines.rs（渲染流域）——wbr 须向
  breaker 投递软断行机会标记 + 相邻 TextRun 无空格拼接串的机会面。日志
  [logs/s319-wbr-initial.log](logs/s319-wbr-initial.log)。

### 2. the-iframe-element：iframe_sandbox_iframe_pdf_viewer.html ✗0.00% → 结构性能力缺位

- 案型 = **mismatch**（rel=mismatch → 须不匹配才过）：test `<iframe sandbox
  src=…/sandbox.pdf>`（sandbox 阻断 plugin）vs support ref 裸 `<iframe
  src="sandbox.pdf">`（Chrome 渲染 PDF 查看器）。✗0.00% = 双侧渲染全等。
- 勘根：ZW 全仓零 PDF 处理面（engine/webview grep 空）——双侧 iframe 均空盒
  → 全等 → mismatch 结构性不可过。缺位面 = PDF 查看器产品能力（非布局/绘制
  流当前 face，非本流可施工）。定性：结构性已知红，簇记 4/5 绿 + 1 能力缺位。
  日志 [logs/s319-iframe-initial.log](logs/s319-iframe-initial.log)。

### 3. the-br-element：br-bidi-in-inline-ancestors.html ✗2.78% → 跨流移交 layout-engine para_bidi

- 案型 = bidi 面（UAX#9）：br 作 bidi 段界 + 内联祖先（bdo/span dir 嵌套）在段
  界关闭并重开 embedding level（HTML §the-br-element + css-writing-modes
  unicode-bidi）；ref 硬编码 LRO/PDF 控制码序。
- ZW bidi 现状（para_bidi.rs R5121/R5337 谱系，段落级 UBA 默认开）显式边界：
  含 `bidi_override`（=bdo）/plaintext/isolate 的 run **整体回退** per-run 路径
  （本页满页 bdo/span dir，正中回退臂）；元素级 unicode-bidi:embed 未映射显式
  level（UAX#9 X5-X8 挂账）；br 段界（P2/P3 段切分 + BD5 关闭重开）未实现。
  2.78% 小差 = Hebrew 字形正常、重排局部差（非 tofu 面）。
- 移交面：crates/layout-engine/src/inline/para_bidi.rs（渲染流域）——X5-X8
  显式 level 映射 + 段界语义双缺口。
- **editing-compat br 刀（fe760dc57）外溢复核 = 阴性**：该刀为 execCommand
  列表 enable 行域面（part01e `_zwListEnableLineScope`），与本 bidi reftest
  零交集。日志 [logs/s319-br-initial.log](logs/s319-br-initial.log)。

## 语料卫生挂账（共享面，不擅动）

`grouping-content/the-ol-element/zzz-probe-copy.html` / `zzz-probe-copy2.html`
（✗6.99%/5.60%）：git 未跟踪本地探针残留（wpt-data gitignore 无归属可查；
嵌套 list-item display 探针形态，非本流产物——本流探针全走 /tmp 与 engine
测试）。按共享面纪律不删，归渲染流域（list-marker 面）处置；2 案现污染本域
计数（95 红中含此 2）。

## 域内现存红页盘点（新队列依据，s318 after 全量基线）

html/semantics 95 红（含 zzz 探针 2）分布：interactive-elements ~30
（**the-dialog-element top-layer/backdrop 族为主体**——top-layer-parent-{filter,
mask,opacity,overflow-* ,transform} 8×100% = 祖先效果对 top-layer 对话框失效
面）、forms ~55（customizable-select ~30 tentative 高级面 + 其余散红）、
text-level-semantics ~15、grouping-content 6 真红（ol start/type/rev 列表标记
值）+2 探针、embedded-content 4（本切片定性 2 + svg-img-with-external-
stylesheet 1 + srcdoc-print 1）。

## 施工

无——三簇根因全落他流 crate（layout-engine ×2）或产品能力缺位（×1），按工作
面纪律不施工（s314 range-intrinsic-size / s316 fallback-image 移交先例）。

## 验证

- 三簇 solo 基线日志 3 份（logs/s319-{wbr,iframe,br}-initial.log）。
- layout-dump 探针 1 份（wbr 单行溢出定性）。
- br 刀外溢复核阴性（commit fe760dc57 --stat 核对面：part01e execCommand 列表域）。
- 本切片 diff 零代码触碰（纯勘根/移交账目切片）——无测试门禁新增面，全量基线
  承 s318 终态（80 腿 ok + 3783P/9F；15994P/1026F）。

## 跨流移交（记入待决）

1. **wbr 断行机会 → 渲染流域**：break_lines.rs 增 wbr 软断行机会标记（R1682
   留白收口）；证据 = layout-dump p h=15px 单行 + R1682 注记 + HTML §12.3。
2. **br-bidi 段界语义 → 渲染流域**：para_bidi.rs X5-X8 元素级显式 level +
   br 段切分/关闭重开（P2/P3/BD5）；证据 = para_bidi.rs 回退臂边界注记 + 案型
   满页 bdo/span dir。

## 下一步候选

1. **the-dialog-element top-layer 族勘根（新主力候选）**：~30 红簇，8×100% 的
   top-layer-parent-{filter,mask,opacity,overflow-*,transform} 形态 = 祖先效果
   链对 top layer 对话框应失效面（engine paint 祖先效果链 + top layer 定位面，
   需勘根分派 paint/layout 归属）。
2. ol start/type/rev 列表标记值族（grouping-content 6 红，标记值 = 标记面归属
   待勘根）；svg-img-with-external-stylesheet / srcdoc-print 两散红勘根。
3. （挂账承前）embed fallback-image（R57）、print pagination 7 键、range-
   intrinsic-size、wbr 断行机会（本切片）、br-bidi 段界（本切片）跨流移交跟踪；
   zzz-probe 语料卫生归渲染流域；iframe pdf = 结构性已知红不立项。
