# editing-compat — white-space join 簇实现落地 + 语料管线阻塞发现（2026-10-30）

承接 [自旋四案簇 A 修复后复测收口](2026-10-30-run-budget-unlock-rerun.md)（终账节）。
本轮完成 join-different-white-space-style-* 三案（60 变体 / 7,518F，两预算轮
稳定）的 CE 块界 join 实现 + 语义证明，并定位该簇自 M1 起 0P 的**语料管线
阻塞根因链**（非语义缺口）。

## 1. 实现（part01.js，`__zw_ce_delete` 前）

- `_zwCeIsBlockEl`：块级判定标签白名单（DIV/P/H1-6/BLOCKQUOTE/LI/UL/OL/TABLE
  族/SECTION/ARTICLE/…/PRE/FORM/FIELDSET/ADDRESS）。FIXME：style 系统未解析
  white-space/display，computed 值不可用；CSS display 解析落地后换计算样式判定。
- `_zwCeWsOf`：white-space 保留值来源——style attr 原始串扫描（
  `/white-space\s*:\s*([a-z-]+)/i`）→ class 名映射（语料 pre/preWrap/preLine/
  nowrap → pre/pre-wrap/pre-line/nowrap）→ 'normal'（host 级 run）。依据：
  white-space 属性全栈不支持（css-parser/style-system/shim style OM/
  getComputedStyle 均无），只能走 attr 原始串；`normalizeStyleAttributeValues`
  是 regex 级原始串归一，setAttribute 存原串可原样序列化。
- `_zwCeTakeLineFrom(parent, startNode)`：首行提取。行界=文本内 `\n`（head
  提取、tail 保留去符）/`<br>`（消费）/块级子（界即停；首内容块子递归解壳、
  空壳移除）；inline 容器克隆壳随提取迁出、提取尽移原壳。返回 `{nodes, ws}`。
- `_zwCeJoinApply(el, dst, srcParent, startNode, inputType)`：beforeinput
  (cancelable) → 首行提取 + `<span style="white-space:…">` 保留壳入 dst →
  caret 落壳前 → input。单 span 形态在语料 `expectedHTML` 多形态数组的接受
  集内（语料 `assert_in_array`）。
- `__zw_ce_delete` 新 ③ 臂（`collapsed && so === 0`）：链上找最近前邻——块 →
  该块接收；inline（caret 块为前 run 后的首块）→ 容器接收；run 中段/宿主起点/
  src 非容器末子 → 保守不并。原 flat 路径逐字保留。
- `__zw_ce_forward_delete` 新两臂：跨容器空界选区（select-boundary——sc 在文本
  尾、ec 在文本头；真跨块选区 defer）+ collapsed 文本终点。守卫同构。
- 三个链上 walk 循环均加 `if (!bc) return;` 链断守卫：分离/陈旧节点树（§3 resync
  场景实证）干净回退 flat no-op，不改变既有契约。

## 2. 语义验证：探针直调法

send_keys 管线被 §3 阻塞（键事件派发后 caret 已陈旧），语义验证改为**绕过
键盘管线直调**：临时探针页（已删；重建=普通 editing 语料页骨架 +
`utils.setupEditingHost(initialHTML)` + 同脚本上下文
`globalThis.__zw_ce_delete('div[contenteditable]')` 直调 + assert innerHTML）
在 testharness-editing-other 通道（ZW_SCRIPT_TIMEOUT_MS=300000）逐形态核对
实际输出与语料 expectedHTML：

| 形态 | initialHTML（setup 后） | 直调 | 实际输出 | 对照 |
|---|---|---|---|---|
| backspace block-join | `<div style="white-space:normal">abc</div><div style="white-space:nowrap">[]def</div>` | `__zw_ce_delete` | `<div style="white-space:normal">abc<span style="white-space:nowrap">def</span></div>` | paragraphs 案 test 1 期望形态 ✓ |
| forwarddelete select-boundary（br 消费） | `<div style="white-space:pre-wrap">abc[</div><div style="white-space:pre-line">]def<br>ghi</div>` | `__zw_ce_forward_delete` | `<div style="white-space:pre-wrap">abc<span style="white-space:pre-line">def</span></div><div style="white-space:pre-line">ghi</div>` | test 3 期望形态 ✓ |
| backspace `\n` 首行 | `<div style="white-space:normal">abc</div><div style="white-space:pre">[]def\nghi</div>` | `__zw_ce_delete` | `<div style="white-space:normal">abc<span style="white-space:pre">def</span></div><div style="white-space:pre">ghi</div>` | test 2 期望形态 ✓（skip:!preserveLineBreaks 族） |

三种几何族（块界前邻 join / select-boundary 空界 / 首行提取行界）逐字命中
语料期望；`white-space` 值来源（style attr 原始串）与保留壳形态（单 span）正确。

## 3. 语料管线阻塞（0P 根因链，既有缺陷）

新 runner 下 join 案仍 0P/3,456F，且所有 Fail 的 actual = initialHTML **逐字
（含 `[]` 标记）**——剥离从未生效。探针定位（探针页同 §2 骨架，18 探针）：

- **D1 解析 DOM 标记剥离不持久化**：`setupEditingHost` 的 marker strip 对
  innerHTML= 产物的文本节点写 `.data`，但该节点是 part05 sel 域**纯快照**——
  探针证：无 `__zwSelector`/`__zwChildIndex`/`__zwWriteChildText`/`__zwHandle`，
  父 div 同样无 `__zwSelector`，`data` 为 own expando 字段（非 defineProperty
  访问器）。R48 写路径（`__zw_set_child_text`，要求 parentSel）与 R3034 handle
  路径（要求 `_textHandles[handle]`）均不满足 → 只写本地字段，引擎 doc 保持
  原串（part04.js R2823 注释自证「parsed DOM 文本节点为 _wrapNodeEntry 静态
  快照无 handle」）。JS 视图读回干净（innerHTML getter 读本地快照），引擎侧
  仍是带标记原串。
- **D2 引擎动作触发视图重建**：send_keys 命令处理（dispatch_action 动作管线）
  触发 fused-view 重同步——JS 视图自引擎 doc 重建 → 标记回滚 + proxy 身份
  更换。判据三件：①跨脚本执行不回滚（strip 后仅读，下个 promise_test 仍干净
  ——普通 execute_script 不触发）；②仅 send_keys 后回滚（backspace/可打印键
  同）；③页内直调 `__zw_host_focus` 不回滚（focus 单独无效应）。
- **D3 caret/listener 陈旧**：resync 后 live Selection `_ranges[0]` 仍持旧
  proxy → `__zw_ce_delete` caret walk 走分离树（无守卫时 TypeError，现 `!bc`
  守卫回退 no-op）；resync 前注册的宿主 listener 零命中，而 document 层捕获
  证实事件确实派到新 proxy（`fi@DIV|kd@DIV:Backspace`）——键管线本身通，
  死在身份/选区陈旧。

结论：join 语义实现就绪；语料回归须 **D1（解析子树 selector 寻址/CharacterData
写路径）+ D2（resync 语义与选区重映射）** 落地后重测。影响面非 join 簇特有：
**全部 marker+send_keys 编辑语料**（editing/other 键盘驱动族）同链受阻，
M2-r1 other 面 11,159F 中 setup 相关 Fail 需按此重归类；execCommand 族
（run 面 90,489 行现行账）自管 DOM 变更，不受影响，账不变。

## 4. 既有红对照（本轮不改）

- `uievents/textInput/delete.html`：本轮抽查出现 page script SyntaxError
  （`Unexpected token '{'` at :3:38，declared tests: 0）——外部
  `support/no-textInput.sub.js` 查询参数 eval 装载面问题；同二进制探针页正常
  跑通，与 shim 改动无因果。挂账不展开。
- 历史对照：uievents 尾簇 32 的 CE ForwardDelete 案（同文件 caret@1 in-text
  形态）此前已在 keyboard/uievents 通道走通——本轮 join 臂不触碰 so!==0 的
  flat 路径。

## 5. 验证与回归

- shim 全量拼接 `node --check`（bridge 顺序 part01→part08b）语法通过。
- 探针 16/17/18 三形态逐字命中（加 `!bc` 守卫后复跑一致）。
- `make test`（DISPLAY=:20）：**3607P/9F**——9 个失败与已知红集逐一对应
  （fetch/Request/Response/iframe-service-worker 域，与本次 shim 改动无
  交集），零新增；3601→3607 的 Pass 增量为并行流新增回归钉（R5341 钉等），
  非本改动产物。
- join 三案语料账：D1/D2 落地前维持 0P 预期，缺口账面不变。

## 6. 复验入口

- 探针重建：§2 直调法（临时页跑完即删，勿留语料目录）。
- 单案通道 `make testharness-editing-other-one FILTER=join-different
  TIME_LIMIT=600`（ZW_SCRIPT_TIMEOUT_MS=300000 须显式）。
- D1/D2 修复切入：part05.js sel 域快照节点构造（`__zwSelector`/`__zwChildIndex`
  补齐 + `__zwWriteChildText` 写路径）+ resync 处（engine 动作管线 fused-view
  重同步点）选区重映射。
