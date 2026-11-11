# M2 切片 392（s392）——forms 树散单余红分诊 + img 域勘根（承 429 残件）+ 自建探针口径卫生（零源码改动）

日期：2026-11-11（承 s391 下一步：the-select-element 等域余红分诊；承接上一 session 01:32 429 限额中断残件）
域：html/semantics/forms 散单小域 + embedded-content/the-img-element + 全树口径卫生
性质：**分诊轮 + 口径卫生——勘根签名 + 归属判决，零源码改动**

## 承接残件恢复

上一 session（2026-11-11 01:24–01:27）在 429 中断前已开入 img 域勘根，残留：`/tmp/s392/dom-img.log`（img 域定向 sweep，20 键）、`probe.log`（zw392_probe 首版探针运行——currentSrc 断言写成相对路径而 runner 返回绝对 URL，探针 bug 未修正）、`zw392_probe.html`（已随口径卫生删除）。本轮完成该域勘根分诊（见下），探针断言 bug 不继承（施工轮重写）。

## 口径卫生（自建探针清理，s391 先例延续）

删除 25 个 gitignore 本地探针文件（全量备份 `/tmp/s392/probe-backup/`）：

| 文件组 | 数量 | sweep 键值 |
|---|---|---|
| the-select-element/customizable-select/ s324-probe-basic | 1 | 2P |
| 同目录 s325-probe-options-move + s325-probe2-corpus-replay | 2 | 13F + 9F2P = 22F2P |
| the-select-element/ s387-probe-contents-box | 1 | 1F |
| forms/ 根 s327-probe-identity / probe2-decompose / probe3-tree | 3 | 6F4P |
| grouping-content/the-ol-element/ zzz-probe-*（含 -ref） | 16 | 0 行（reftest-only，不入 testharness sweep） |
| media track-element zz-cueprobe | 1 | 0 行 |
| the-img-element zw392_probe（上轮残件） | 1 | 0 行（建于基线 sweep 之后） |

- **键级口径**：下次全树预期 **8199P/231F/37T = 8467 唯一键**（s391 基线 8504 −37 行：−29F−8P 全为本轮删除探针，零语料真实位移）。全树在册红 260F+37T → **231F+37T**。
- **the-select-element 域口径**：169F+2T（含探针 27 行）→ **146F+2T corpus 口径**——与 s386 非探针 59 文件清单完全一致（簇 A 特征门 53+根邻位 4、簇 C input-parsing 2+selectedcontent-mutations 8+特征邻位 2+通道上限 2）；forms 根目录 6F→0F。
- **教训钉**：scripting-1/the-script-element/module/ 的 probe8/9/10 七文件是 **s364（commit 42274e6e9）「行为钉转正入 git 跟踪——跨 clone 回归保护」常驻载体**，本轮误删即被 `git status` D 暴露，已 `git checkout` 恢复（solo 全绿确认）。后续探针清理前必须先 `git ls-files` 核对跟踪态——gitignore 目录内的个别文件可为强制跟踪的回归钉。

## img 域余红分诊（6 案 20 键，承上轮残件勘根）

| 案 | 键 | 失败签名（solo/dom-img.log） | 归属判决 |
|---|---|---|---|
| current-pixel-density/basic | 3F | width/naturalWidth IDL 全过（density/w 描述符/sizes 解析面在位）；红在 clientWidth：10000x/9e999…x/256w+sizes=0px 三臂产物 0.0256px 亚像素盒 ZW 取 1、上游取 0 | **布局亚像素取整面** → rendering-compat 挂账 |
| current-pixel-density/error | 6F | 首环 `assert_not_equals(expected_width, 0)`：broken img（404）的 alt 文本盒尺寸面缺席，ref width=0 连坐 7 案 | **broken-image alt-box 特性缺席** → rendering-compat 挂账（特性级，中等） |
| sizes/sizes-auto | 4F+1T | eager+width attr 臂全过；红在 style 几何臂（vertical-rl 以 block-size 为宽 / abspos left:50%;right:49% 约束宽）——sizes=auto 需布局几何馈给；file T 同族 lazy 相交面待深勘 | 几何半边 → rendering-compat 挂账；lazy 半边挂账待勘根 |
| svg-img-density-vs-zoom | 3F | page zoom 2x：rendered 尺寸未随 zoom 缩放（600 vs 300），naturalWidth 已对 | **page zoom 渲染面** → rendering-compat 挂账 |
| update-the-image-data/not-broken-while-load-task-scheduled | 2F | 未加载 img：width/height 属性→盒面 50px 缺席（clientWidth 0）；加载后臂未达 | **流内施工候选**（s394）——归属探针先行：engine attr→style 面还是 layout 未加载盒面 |

## forms 树散单分诊（6 域 12 键）

| 案 | 键 | 失败签名（solo） | 归属判决 |
|---|---|---|---|
| the-output-element/output.html | 1F | `output.defaultValue="10"` 后 value 仍 "5"——defaultValue setter 未回写 textContent，value mode flag（default/value 二态）语义缺席 | **流内小刀 s393 候选**（part shim：flag 态 + 双 setter/getter 联动） |
| the-label-element/clicking-noninteractive-unlabelable-content.html | 2F | img（含 tabindex="" 变体）深嵌 label 下 span.click() → `Maximum call stack size exceeded`——label 激活转发对非 labelable 内容无终止条件，无限递归；first-child 位形触发、second-child 位形过 | **流内 bug 修复 s393 候选**（labeled control 决议/转发终止，共享路径勘根） |
| the-textarea-element/wrapping-transformation.window.js | 2F | wrap=hard：FormData API 值未按 cols 折行（期望 `ABCDEFGHIJ\nKLMNOPQRST\nUVWXYZ` 得原文；LF 归一臂过） | **流内中刀 s394 候选**（wrapping transformation：reset/entry-list 面 + cols 管线） |
| the-textarea-element/textarea-placeholder-lineheight.html | 1F | gBCR height 19.5→19 亚像素取整；gCS lineHeight 过 | **布局亚像素 gBCR 面** → rendering-compat 挂账（与 basic 案同族） |
| the-button-element/button-click-submits.html | 1T | **solo 12/12 全绿**；sweep 8-job 并行负载下 500ms 步进超时链翻文件级 T | **时序 flake（sweep 负载），非缺陷**——sweep 噪声键记录，不施工 |
| forms/form-submission-target/（6 案） | 6T | solo 复现 6T：reltester 真实表单提交导航 + `_blank` 开窗 + endpoint BroadcastChannel 跨 browsing-context 回报，ZW 无提交导航即挂 | **navigation-compat M3 门控域挂账**（Mission 排除条款：浏览上下文树）——6T 全归 |

## 下一步

- **s393 施工轮（双小刀）**：①output value mode flag（1F）；②label img 递归修复（2F）。
- **s394**：textarea wrap=hard（2F）+ img unloaded attr sizing 探针定归属（2F）。
- 挂账池新增：渲染流 4 面（basic 亚像素 / error alt-box / svg-zoom / lineheight + sizes-auto 几何半边）回流 rendering-compat 账；form-submission-target 6T 归 navigation M3；sizes-auto lazy 半边待勘根。
- A2+D 簇立项征询合并呈报维持待用户决策。

## 验证

- solos（日志 /tmp/s392/）：solo-output.log（1F 签名）、solo-label.log（2F stack overflow）、solo-ta-lh.log（19.5→19）、solo-ta-wrap.log（2F 未折行）、solo-button.log（**12/12 全绿**）、solo-notbroken.log（2F clientWidth 0）、solo-fst.log（6T 复现）。
- select 域删除探针后域 solo：**182P/146F/2T = 330 键，与预测精确一致**，probe 行 0（日志 /tmp/s392/solo-select-domain.log）。
- git：零源码改动，工作树干净（probe8/9/10 恢复后 status clean）。
