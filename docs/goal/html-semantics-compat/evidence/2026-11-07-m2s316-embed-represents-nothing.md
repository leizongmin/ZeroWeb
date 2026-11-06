# M2 s316 — the-embed-element 簇 3 红页收口：embed represents-nothing paint 门（2/3 绿 + 1 跨流移交）

日期：2026-11-07。切片号 s316（承 s315 下一步候选①小簇排队，勘根→施工型切片）。

## 勘根/边界确认

- **簇红页形态**：the-embed-element 5 reftest 案 = 3 红 + 2 绿（hidden-attribute /
  represent-nothing-03 [video 祖先臂——video 子树布局层不建盒，结构满足 spec 的
  media 祖先腿] 维持）。
- **三红页两根因**（1 引擎 + 1 跨流）：
  1. embed-represent-nothing-01/02（match ✗2.08% ≈ embed 100×100 面积）——**引擎
     represents-nothing 门缺失**：`<embed>` 无 src+type（01）/ script 移除双 attr
     （02）→ 非 potentially active → represents nothing 不渲染；ZW 仍生成盒并绘
     作者红背景。探针实锤：无 attr 页 fill=(255,0,0,100,100)。
  2. embed-in-object-fallback-image（match ✗2.08% 反向——应有红无红）——**跨流
     layout-engine R57 一刀切**：`is_replaced_with_fallback`（tree.rs，R57 M3）对
     canvas/video/audio/iframe/**embed**/**object**/applet 无条件抑制 fallback 子
     参与布局——object 回退态（type=flash 不支持、无 data）须渲染子（HTML §4.8.4
     fallback content），ref 页 standalone embed 照常绘红 → test 侧红缺席。探针
     分流：object/video 吞全部子（object+div h=8 vs bare-div h=116），standalone
     embed 正常。
- **spec 判据**（§4.8.6 potentially active，全文核对）：src set 或 type set，且
  src 缺席或值非空串（src="" 即便带 type 仍 nothing）；media 祖先 / object 非回退
  祖先两腿在 ZW 由布局层子抑制结构满足。
- **同窗勘根副产物（object 簇 2 红定性，入 s317 候选）**：object-image/svg-only-
  for-print ✗1.04% = **runner print-reftest 分类面**——上游
  `tools/manifest/sourcefile.py` `name_is_print_reftest`：文件名
  `rsplit("-",1)` 尾 flag == "print" 或路径含 print 分量 → `-for-print.html` 文件
  **是上游 print-reftest**（print 媒体仿真跑，`@media not print` 臂失效、object 显
  图）；ZW runner 按普通 screen reftest 跑 → 结构性恒红。runner 已有 print 配置
  面（R2013 ReftestConfig print 档 + A4 宽）可承接。

## 施工

1. **engine paint 门**（painter/mod.rs 单谓词 + 双走查同步）：
   - `embed_represents_nothing(doc, node_id)`：embed 且 ¬(src set 或 type set 且
     src 缺席或非空) → true。谓词放 painter 自由 fn 区（widget_track_chrome_
     suppressed 同域先例）。
   - 主路径 `paint_node_inner` + 脏矩形路径 `paint_node_in_rect` 的 display:none
     守卫后各插整盒 return（R3768/R3769「跳过链两路径须同步加」先例）。
   - 布局盒保留（Chrome 无 layout object 的像素面近似——布局空间差异无语料驱动，
     三 driving 页不观测后续兄弟位移）。
2. **单测 1 案**（pipeline.rs）：四形态矩阵——无 attr 0 红绘 / src 非空+type 1 绘 /
   仅 type 1 绘 / src=""+type 0 绘（spec src 空串腿）。

## 语义边界（记录）

- 布局层不动：layout-engine/style-system 零触碰（工作面边界）；object fallback
  子渲染与 print-reftest 分类分别移交渲染流域与 runner 增强候选。
- embed void 元素无子树，整盒 return 不涉及子内容抑制面。
- fetch 失败臂（src 非空但资源不可用 → spec 亦 represents nothing）未实现：需
  fetch 状态观测，无语料驱动，FIXME 不设占位。
- perf-gate 豁免：谓词为逐 embed 元素 paint 前短分支（两 attr 查询），非热路径；
  承 s311-s315 先例。

## 验证

- 簇 solo reftest：**the-embed-element 3F→1F（4/5 绿）**——represent-nothing-01
  ✗2.08% → **✓0.00%**、represent-nothing-02 ✗2.08% → **✓0.00%**（02 动态移除
  重绘同门生效）、03/hidden 维持绿；fallback-image ✗2.08% 维持（跨流移交面零扰
  动）。日志 [logs/s316-embed-after.log](logs/s316-embed-after.log)（修前基线
  logs/s316-embed-initial.log：2P/3F）。
- 单测：engine lib s316 1 案全绿。
- clippy（engine）：零告警。
- 产品 smoke（paint 变更面）：make product-smoke-legacy struct 0 +
  make product-smoke PASS。
- make test（guarded-test，后台全量）终态 **80 腿 ok + engine lib 3783P/9F**：
  8F=已知键集（CSSOM 5 + websocket 2 + fetch 1），+1 新键
  test_iframe_service_worker_messages_dispatch_to_iframe_container——stash 探针
  （弃 s316 diff 后干净树同败）定谳非本切片，归因并行 noneditable 刀 96b49169a
  （rebase 落入晚于 s315 终态测量）。日志
  [logs/s316-make-test.log](logs/s316-make-test.log)。

## 跨流移交（记入待决）

1. **embed-in-object-fallback-image → 渲染流域**：layout-engine tree.rs R57
   `is_replaced_with_fallback` 一刀切抑制 object fallback 子——object 回退态须
   渲染子（HTML §4.8.4；canvas/video 现行为正确保留，仅 object 需按「是否展示
   回退」分派）。归因证据：探针 A/B/D/E + 代码锚 tree.rs:2806-2827 + spec。
2. **object-for-print 两案 → runner print-reftest 分类增强（s317 候选）**：
   `-for-print.html` 尾 flag 分类 + print 媒体仿真渲染（上游规则源码已核
   sourcefile.py:434-437/235-239）；ZW 已有 print 配置面可承接。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：
- s316-embed-initial.log（修前 2P/3F 基线）
- s316-embed-after.log（4/5 绿终态）
- s316-legacy-smoke.log / s316-smoke.log / s316-make-test.log（回归门禁）

代码锚：crates/engine/src/paint/painter/mod.rs（embed_represents_nothing + 双走
查守卫）；crates/engine/src/tests/pipeline.rs（s316 四形态矩阵）。

注记：本切片 diff 落 engine 2 文件；layout-engine/style-system/render-foundation
零触碰（跨流面只归因移交）。

## 下一步候选

1. object 簇收尾：runner print-reftest 分类增强（2 案 for-print 面）+ object 簇
   其余勘根；embed fallback-image 跨流移交跟踪。
2. （挂账承前）小簇排队 option 1/iframe 1/wbr 1 逐簇勘根；br 1 勘根前先 solo
   复核 editing-compat br 刀（fe760dc57）外溢现势；range-intrinsic-size 跨流移交
   跟踪。
