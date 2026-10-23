# HTML Semantics 兼容 — html/semantics 语义面（主攻大域）

**版本**: v2.0
**日期**: 2026-10-23（v1.0 立项 2026-10-08）
**状态**: Active——M2 切片 43 收口（挂账收口：replaceWith self-removal 父桶记账——R5246 补 removed identity 入桶，合并视图残留终结，全域 REG 0 / GAIN 0），下一步按挂账序推进；**用户决策 2026-10-23**：parser form element pointer 4F **缓行**——与 declarative shadow DOM 打捆 parser 批次、M2 收口后尾段单独立项（[决策记录](html-semantics-compat/evidence/2026-10-23-user-decision-parser-form-pointer-defer.md)），流按其余挂账序推进
**执行模式**: WPT 驱动（上游 html/semantics corpus 为验收标尺）+ 语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——HTML 语义面）

> **▶ 当前状态（2026-10-23 M2 切片 43 收口）**：挂账「replaceWith
> self-removal 父桶记账」收口（s26 既有挂账，R371 模板残留）。**根因**：
> replaceWith sel 臂 remove 步只覆盖 apply 面（Remove/RemoveHandle
> mutation）与 sel/handle 字符串标记表（parentNode/isConnected 读），合并
> 视图子读（childNodes overlay / R322 归并查询）按**父桶 removed identity**
> 剔除——桶缺 removed 项 → 被替换者（快照 sel 节点）在合并视图残留至 apply
> 换代，视图 ≠ spec 终态。**落地（R5246 一刀）**：remove 步后经 canonical
> 记账入口 `_zwHCLiveInvalidate([], [被替换者 proxy], 父sel, null)` 直调
> （父桶 removed + 全局 pending-removed + live 集合失效一体；子树展平含
> template 内容子）；proxy identity 取基列表条目（`_r371Self`，与 overlay
> 剔除同一缓存对象）；记账先于 R371 rekey → dst 桶必存在，rekey 走合并分支
>（promote 分支保留护栏）。**验证**：契约
> test_dialog_replacewith_parent_bucket_accounting_r5246（part75 新文件，
> R1 corpus 同形全树视图退出 + R2 body 直子面）+ 既有
> r321_replacewith_fragment_flatten 断言更新（kids 前导宿主残留 DIV 消失，
> 视图即 spec 终态）；interactive-elements 全域 98 files 482 subtest
> **419P/61F/2T 对 s42 逐位 REG 0 / GAIN 0**（挂账本就「对扫描无害」，
> corpus 无观测面——切片性质同 s42：spec 正确性收口 + 契约钉住）；lib
> **3434P/0F**。**已知边界**：sel 臂 replaceWith 的 MO record 面（插入/
> 移除两段，R117 handle 臂已有）独立挂账不扩面。gates：clippy
> --workspace -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s43-replacewith-parent-bucket.md](html-semantics-compat/evidence/2026-10-23-m2s43-replacewith-parent-bucket.md)。
> **下一步**：按挂账序 summary-less details 内非激活后代翻转（s37 邻近
> 缺口，真浏览器会翻转）、focus-after-close Complex dialog usage focus
> 恢复深层（s39 剩余 1F）；parser form element pointer 4F **已缓行**
>（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆 parser
> 批次、M2 收口后尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 42 收口）**：挂账「非模态 close 不出
> 组栈」收口（s23 R4300 已知保守项）。**根因**：`_zwDialogCloseTheDialog4200`
> 的组栈出栈 splice 在模态臂内——modeless close()/requestClose()/ESC/
> light dismiss 全链残留 stale 项，show()/close() 循环重复项无限增长
> （读侧 purge 仅 ESC 扫描顺带清，行为面无观测差异）。**落地（R5245
> 一刀）**：splice 移出模态臂，模态/非模态一律同步出栈（spec close
> watcher 随关闭销毁、其组离栈）；show()/showModal() 无条件 push 与
> part06 读侧 purge 维持（markup open 直写面兜底不变）。**验证**：契约
> test_dialog_group_stack_hygiene_r5245（part74 新文件，R1–R4：三循环
> 重开 ESC 命中尾组 / 双 modeless 栈序 / 全关 no-op / 模态臂保持——栈
> 内部态无 JS 观测面，契约钉行为不变量）；interactive-elements 全域
> 98 files **419P/61F/2T 对 s41 逐位 REG 0 / GAIN 0**；lib **3433P/0F**。
> **已知边界**：markup removeAttribute('open') 直写面不经 close 核心，
> stale 项靠读侧 purge 兜底（R4300 既有设计维持）。gates：clippy/fmt
> 无 Rust 变化、零新增依赖。账本见
> [evidence/2026-10-23-m2s42-nonmodal-close-stack.md](html-semantics-compat/evidence/2026-10-23-m2s42-nonmodal-close-stack.md)。
> **下一步**：按挂账序 R371 模板残留、summary-less details 内非激活
> 后代翻转（s37）、focus-after-close Complex dialog usage focus 恢复
> 深层（s39 剩余 1F）；parser form element pointer 4F **已缓行**
> （2026-10-23 用户决策：与 declarative shadow DOM（H）打捆 parser
> 批次、M2 收口后尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **切片间隙宣告义务（2026-10-23 用户决策，配合
> [oracle 流解封协议](incremental-rendering-oracle/master.md)）**：每个
> shim 面切片 push 后，若下一挂账不触 engine 生产面（余下挂账均为 shim
> 面），在「当前状态」宣告切片间隙开放；开工下一切片前先查 oracle
> master 碰撞记录有无占用标记，有则等待或改排其他 shim 挂账。
>
> **▶ 上轮状态（2026-10-23 M2 切片 41 收口）**：挂账「ESC 组栈边缘乱序」
> 定性收口（非代码轮）。**探针钉定**（zz-probe-s41，验证后已删）：
> corner-cases「focusin calls showModal, modeless」ESC 案事件序 =
> show() 聚焦空 modeless dialog（subject fallback）→ close-the-dialog
> 焦点恢复尾步 anchor 门命中（焦点锚=subject 后代）→ prevFocused
> focusing steps 触 focusin → 处理器 re-entrant showModal 重开。**spec
> 对账**：恢复门（锚 shadow-including 后代 OR wasModal）、show()/
> showModal() 双记录 prevFocused、focusing steps「not focusable →
> nothing」与现行文本逐字一致——R4600/R5236 无漂移。**试错回退**
> （R5244 subject 非 focusable 门，未合入）：目标案 GAIN 1 但 REG 4
> （dialog-focus-shadow「show:」三臂 + focus-previous-outside——Chrome
> 断言面「show() 无 focus delegate 聚焦 dialog 自身」，whatwg/html#8199
> 只移除 TAB 序可聚焦），两面矛盾 Chrome 选边聚焦面。**Chrome-parity
> 同红证据**：crbug 417571176（父 376516550 closedby 特性）2026-04
> 仍 "still failing according to wpt.fyi"，上游已改一次 assertion 仍未
> 过；本项目按 Chrome 拉齐，修复后随 corpus 刷新自然转绿。**收口**：
> 挂账关闭（语料钉住 + crbug 引用入账）；interactive-elements 全域
> 98 files **419P/61F/2T 对 s40 逐位 REG 0**、lib 3432P/0F、零代码净
> 变化。账本见
> [evidence/2026-10-23-m2s41-esc-stacked-parity.md](html-semantics-compat/evidence/2026-10-23-m2s41-esc-stacked-parity.md)。
> **下一步**：按挂账序非模态 close 不出组栈、R371 模板残留、
> summary-less details 内非激活后代翻转（s37）、focus-after-close
> Complex dialog usage focus 恢复深层（s39 剩余 1F）；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）；
> declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 40 收口）**：挂账 custom command
> （`--` 前缀）CommandEvent 派发收口（s24 defer）。**spec 钉定**：现行
> button activation behavior——validity Custom 态对任意元素 true →
> fire command 事件（command=值本身大小写保留、source=invoker、
> cancelable=true）→ Custom 态 return 无内建行为；派发形状
> bubbles=false/composed=true/isTrusted=true/无 relatedTarget（上游
> button-event-dispatch 语料钉死）。**落地（R5243 两刀）**：①part05
> `_defineEventSubclass('CommandEvent','Event',[command,source])`
> （ToggleEvent 同厂）；②part01 `_zwInvokerCommandActivate` custom
> 分支构造 CommandEvent + trusted 印记（R312 通道）经
> `_dispatchWithBubble` 在 target 派发后 return——分派在既有调用方
> 门内（disabled/preventDefault 零新写），`cmdCustom` 原值与 `cmd`
> 小写拆分、内建关键字面不变。**集群**：interactive-elements 全域
> 98 files 482 subtest **419P/61F/2T 与 s39 逐位打平**；dialog 集群
> 82 files 对 s35 账本 **REG 0 / GAIN 5**（全为 s39 既有收益）；
> forms/the-button-element 18 files 54 subtest 对 s24 逐位一致
> **REG 0**（corpus 零 custom command 消费）。**已知边界**：现行
> spec「invalid → Unknown 不派事件」未跟进（维持 R4400 auto 映射，
> corpus 钉住，跟进涉 dialog/details 内建关键字重构独立切片）；
> popover 关键字 + oncommand IDL + button command/commandFor IDL
> 反射维持挂账。gates：lib **3432P/0F**（+1 契约 part73
> test_invoker_custom_command_event_r5243）、clippy/fmt clean、零
> 新增依赖。账本见
> [evidence/2026-10-23-m2s40-custom-command-event.md](html-semantics-compat/evidence/2026-10-23-m2s40-custom-command-event.md)。
> **下一步**：按挂账序 ESC 组栈边缘乱序、非模态 close 不出组栈、
> R371 模板残留、summary-less details 内非激活后代翻转（s37）、
> focus-after-close Complex dialog usage focus 恢复深层（s39 剩余
> 1F）；parser form element pointer 4F **已缓行**（2026-10-23 用户
> 决策：与 declarative shadow DOM（H）打捆 parser 批次、M2 收口后
> 尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 39 收口）**：挂账 focus-in-shadow
> 跨流收口（focus-after-close「moved to shadow root」案 +
> dialog-focus-shadow-double-nested）。**探针钉定**（zz-probe 五迭代，
> 验证后已删）四层根因：①sel 模板 content 子列表 per-read 重建，跨读
> mutation 落空（spec content getter 恒返同一 fragment）；②content
> 条目 parentNode 指模板 wrapper，replaceWith 落错列表（探针 D
> `pidIsView=false`）；③`_zwMEl.replaceWith` 只有 remove 半边，替换者
> 落空；④focus delegate 非 dialog 分支缺 get-the-focusable-area 递归，
> 嵌套 shadow host 被跳过后焦点落树序后一 focusable（探针 E
> `osraeTag=BUTTON`≠autofocus 锚）。**落地（R5241 四刀 + R5242 一刀 +
> 收尾失效）**：①part04 content 子列表首读物化缓存（缓存数组即活
> 树）；②parentNode/parentElement getter 槽位消费补 parentHandle 分支
> （focus chain 上行达 shadow 边界）；③`_tcParent = _tplContent`（条
> 目父 = 视图本体）；④part03 replaceWith 补插入半边（plain 父门控，
> host proxy 面零变化）；⑤part01 focus delegate 递归（dialog 分支
> seq-focusable 优先序不动）；⑥innerHTML= setter sel 模板缓存失效
> （R5196 lib 红面收口，视图身份不动）。**集群**：
> double-nested **2F → 2P 全绿**；focus-after-close **2F+1T → 1F+1T**；
> interactive-elements 全域 98 files 419P/61F/2T 对 s38 账本
> **REG 0 / GAIN 3**；forms 域 401 files 3902P/707F/20T/1U
> **REG 0 / GAIN 3**（GAIN 为 label 文件尾 3 案 s38 跑时截断未注册，
> 共同 48 案逐位一致）。**已知边界**：focus-after-close 余 1F
> （Complex dialog usage focus 恢复，既有）；content 缓存不回灌非
> innerHTML= 的宿主侧子树变化。gates：lib **3431P/0F**、clippy/fmt
> clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s39-focus-in-shadow.md](html-semantics-compat/evidence/2026-10-23-m2s39-focus-in-shadow.md)。
> **下一步**：按挂账序 custom command `--` 前缀 CommandEvent defer、
> ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留、summary-less
> details 内非激活后代翻转（s37）、focus-after-close Complex dialog
> usage focus 恢复深层（s39 剩余 1F）；parser form element pointer 4F
> **已缓行**（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆
> parser 批次、M2 收口后尾段单独立项，触发条件见
> [决策记录](html-semantics-compat/evidence/2026-10-23-user-decision-parser-form-pointer-defer.md)）。
>
> **▶ 上轮状态（2026-10-23 M2 切片 38 收口）**：挂账 appendChild move
> 后兄弟读 stale 收口。**探针钉定**（zz-probe 两迭代，验证后已删）：
> R334 appendChild/insertBefore wire 异步 drain，兄弟读无 pending 补偿
> （parentNode 有槽位补偿、兄弟没有）；两遍调用序列进一步钉出**共享视
> 图插位缺陷**——R182 重定位 pos 在旧位剔除前计算，剔除后未同步，
> cleanup `insertBefore(input, dialog)` 形态视图成 [dialog, input,
> script]（真树 [input, dialog, script]）；shadow 容器案露第三层——
> 上一子测试 stale parentSel 槽未覆写。**落地（R5240 三刀）**：①part04
> 兄弟 trap pending gate（sel 槽→融合视图定位 / 容器 ariaSync→
> registry 定位）；②part05 overlay 旧位剔除 pos 同步（**共享路径**，
> childNodes/children 全消费面受益）；③容器挂载路径 dual-form 槽覆写
> （R97 noWire 同款形状）。**集群**：focus-after-close **3F+1T →
> 2F+1T**（moved-in-between 案全绿）；interactive-elements 全域 98
> files 416P/64F/2T 对 s37 账本 **REG 0 / GAIN 1**；forms 域 4627
> subtest 逐位一致（REG 0）。**已知边界**：moved-to-shadow-root 案兄
> 弟断言过、红点推进到 `shadowRoot.activeElement`（focus-in-shadow 跨
> 流缺口，与 dialog-focus-previous-outside 同族，新挂账）。gates：lib
> **3430P/0F**（+1 为渲染流新增）、clippy/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s38-pending-move-sibling.md](html-semantics-compat/evidence/2026-10-23-m2s38-pending-move-sibling.md)。
> **下一步**：按挂账序 parser form element pointer 4F（用户决策门禁，
> 单独立项征询）、custom command `--` 前缀 CommandEvent defer、ESC 组
> 栈边缘乱序、非模态 close 不出组栈、R371 模板残留、summary-less
> details 内非激活后代翻转（s37）、focus-in-shadow
> shadowRoot.activeElement + focusing steps 跨 shadow 恢复（s38 新，与
> dialog-focus 族跨流缺口同族）；declarative shadow DOM（H 定性后）待
> parser/渲染流联合切片。
>
> **▶ 上上轮状态（2026-10-23 M2 切片 37 收口）**：挂账 anchor-with-inline
> 导航缺口收口。**探针钉定**（单案隔离；多案探针 onhashchange 竞争污
> 染作废）：裸 a 后代 click 也不导航——R112 post 段 nearest activation
> 无 act:a 分支（落 input-change 兜底）；svg 案露出第二根因——summary
> 豁免祖先 walk 不认 SVG ns a[href]（`_zwIsInteractiveContentEl` ns
> 门），summary 误翻转。**落地（R5239）**：①R112 post 补 act:a/act:area
> 分支执行 `_zwAnchorActivate`（receiver 自身让位防双导航）；②summary
> 消费面祖先 walk 补 A/AREA href 判定（不扩共享交互函数）。**集群**：
> anchor-with-inline-element **Timeout 全红 → 5P/0F**；A/B 对照 summary
> 域 REG 0 / **GAIN 5**（窄面 20P/0F/0T 全绿）、details/label 逐位一
> 致；interactive-elements 全域 98 files 415P/65F/2T（Timeout 2 文件均
> 既有挂账）、dialog 子集对 s35 REG 0 / GAIN 1；forms 4627 subtest 计
> 数逐位一致。**已知边界**：点击 summary-less details 内非激活后代不
> 翻转（同根邻近缺口，真浏览器会翻转，记待办）。gates：lib **3429P/
> 0F**、clippy/fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s37-anchor-inline-activation.md](html-semantics-compat/evidence/2026-10-23-m2s37-anchor-inline-activation.md)。
> **下一步**：按挂账序 parser form element pointer 4F、custom command
> `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不出组
> 栈、R371 模板残留、summary-less details 内非激活后代翻转（s37 新）；
> declarative shadow DOM（H 定性后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 36 收口）**：挂账 impl-doc wrapper
> requestClose 收口。**探针钉定**：dialog-requestclose 面 5 files 78P/1F，
> 唯一红正是挂账项；合成 doc（`_zwMEl` 产物）dialog 四方法全缺，R125
> 原型链已接 HTMLDialogElement.prototype 但 prototype 空占位。**落地
>（R5238）**：`HTMLDialogElement.prototype.requestClose`（R4100
> details.name 同款 plain 世界供给惯例）——spec「request to close」
> 步骤照抄：open 缺 return、ownerDocument ≠ 主文档（非 fully active）
> return；trap 世界走 part04 R4200 版先于原型命中，双世界不重叠。
> **集群**：requestclose 面 **78P/1F → 79P/0F**（EXIT 1→0）、dialog
> 集群 82 files 对 s35 账本 **REG 0 / GAIN 1**（351P/59F/1T）、forms 域
> 4627 subtest 对 s34 基线计数逐位一致。**已知边界**：detached doc
> dialog 的 close/show/showModal 三方法仍缺（挂账范围外，探针 3F
> 实证）。R5237 撞渲染流，改 R5238。gates：lib **3429P/0F**、clippy/
> fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s36-wrapper-requestclose.md](html-semantics-compat/evidence/2026-10-23-m2s36-wrapper-requestclose.md)。
> **下一步**：按挂账序 anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边
> 缘乱序、非模态 close 不出组栈、R371 模板残留；detached doc dialog
> close/show/showModal 三方法（新）；declarative shadow DOM（H 定性
> 后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 35 收口）**：挂账 E 余项 ESC 栈序
> 收口，**closedby 双语料面全清**（simple 6/6 + complex normal 面 s34
> 已绿）。**落地（R5236）**：`__zw_esc_dialog_cancel` 两段式——①
> `_zwTopLayer`（showModal+showPopover 共享插入序）反走，栈顶 popover
> 命中即 hidePopover、先碰 open dialog 切组栈；②R4300 组栈原逻辑不动
> （惰性补登记 + none 挡板 + 尾→头首个 open 组）。spec 钉定：close
> watcher 组栈，none dialog 组挡「再往下」不挡「更晚注册」——两段拼接
> 与 watcher 组栈语义在所有混合序一致。**集群**：82 files 411 subtest
> **350P/60F/1T**，同 key 对 s34 ON 基线 diff **REG 0 / GAIN 5**；回归
> 面 canceling 族 8P/0F、keyboard 77 键 A/B 逐位一致。设计迭代：首版
> 统一扫描破 part60 R2（lib test 抓住），两段式恢复。gates：zero-engine
> lib **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s35-esc-unified-toplayer.md](html-semantics-compat/evidence/2026-10-23-m2s35-esc-unified-toplayer.md)。
> **下一步**：按挂账序 impl-doc wrapper requestClose（s36 已收口）、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close
> 不出组栈、R371 模板残留；declarative shadow DOM（H 定性后）待
> parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 34 收口）**：挂账 F 帧驱动快照新鲜
> 化。**落地（R5235）**：Makefile `testharness-html-semantics` 入口默认
> `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a/R3254-KP4 既有开关，
> keyboard 套件同款入口模式；直接调 runner 维持 OFF 旧路径）。**对账三
> 面**：dialog filter 82 files 411 subtest **337P/73F/1T → 345P/65F/1T
> REG 0 / GAIN 8**（closedby-complex normal dialogB/popoverB——快照新
> 鲜后命中循环拿到真实盒）；testharness-html 合成面 7 files 14 subtest
> 逐字节一致；forms 域 401 files **4627 subtest 逐位一致**。**同日续勘
> 归因修正**：**挂账 G 移除**——spec 原文（#blocked-by-a-modal-dialog）
> 就是「栈顶模态 + flat tree 后代豁免」无祖先链豁免，gate 只判栈顶与
> spec 一致，三层栈探针是人造面（误关真因 = 快照滞后已修）；**挂账 H
> 重新定性**——最小探针实证 `<template shadowrootmode>` declarative
> shadow root 不支持（shadowRoot null、template 当活 DOM=R371 伪象），
> shadow slots 16F 归因 declarative shadow DOM 缺口（parser + shadow 树
> 渲染，后者归渲染流），修 light dismiss flat contains 是修在伪象上。
> **挂账 I（新）**：全量 testharness-html-semantics（2594 用例）>5400s
> 跑不完且 `--json` 缓冲到最后超时全丢——runner 增量输出或 make 分域入
> 口。gates：runner Makefile 一行无 Rust 变化、零新增依赖。账本见
> [evidence/2026-10-23-m2s34-raf-frame-driven-default.md](html-semantics-compat/evidence/2026-10-23-m2s34-raf-frame-driven-default.md)。
> **下一步**：E 余项 ESC 栈序（simple 5F——统一 top-layer 序覆盖
> popover 候选，涉 R4300 组栈改造）或按挂账序 impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留；declarative shadow DOM
> （H 定性后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 33 收口）**：light dismiss 两遍模型
>（挂账 E 第一刀）。**落地（R5234）**：`__zw_dialog_light_dismiss_check`
> 两遍重写——①popover pass（`_zwTopLayer` 反走自顶向下，stale/摘属性/
> 断连跳过，contains 即级联停，否则 `_zwHidePopover`）；②dialog pass
>（反文档序首个 open+any 一锤定音：backdrop 命中自身 → requestClose；
> contains → break 内层挡外层；否则 requestClose——一次 pointerdown 至
> 多关一个 dialog）；③shadow 守卫（targetSel 解析不到 → 保守跳过）。
> **集群**：82 files 411 subtest **337P/73F/1T**，同 key 411 对 s32 逐位
> diff **REG 0 / GAIN 16**（closedby-complex 的 dialogA/outside-all/
> popoverA 面）。**探针钉定零盒根因 = 快照滞后**：`waitForRender`（双
> rAF）在 shim OFF 模式是同步 stub 不驱动 render，测试体内 gBCR 永远读
> 上一拍快照；`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a 既有开关）
> 实验实证 closedby 46 子测全出 **25P/21F** 对 OFF 同 key **回归 0
> 增益 +8**（normal dialogB/popoverB 全绿）——未默认启用（全集群对账
> 独立切片，挂账 F）。**新挂账**：F 帧驱动快照新鲜化（runner）、G
> runner modal retarget 多模态规则（下层 modal 盒内点击误 retarget
> 栈顶）、H shadow 影域 light dismiss（shadow slots 16F，target 解析
> 穿 shadow）、E 余项 ESC 栈序维持（simple 5F）。gates：zero-engine lib
> **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s33-light-dismiss-two-pass.md](html-semantics-compat/evidence/2026-10-23-m2s33-light-dismiss-two-pass.md)。
> **下一步**：挂账 F（帧驱动快照新鲜化 + 全集群对账，解锁 normal
> dialogB/popoverB 8 案与 simple Timeout 面已实证）或按挂账序 G/H、
> impl-doc wrapper requestClose、anchor-with-inline 导航缺口、parser
> form element pointer 4F、custom command `--` 前缀 CommandEvent defer、
> ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 32 收口）**：popover 有效性语义。
> **红因链破案**：closedby-complex 40F 统一死于 `matches(':popover-open')`
> SyntaxError（host 引擎无该伪类 → promise_test 未处理拒绝全红）；
> closedby-simple 1F = `resetDialogOuterTest` 前置 `hidePopover()` 过度
> 抛出，文件级 Timeout = replica promise_test 不接体 rejection、其余 5
> 体同点 reject 挂死 completion callback（`pending:5 tests:6` 实测吻合）。
> **spec 钉定**：hide popover 算法仅在 validityResult 为 DOMException 时
> 抛——hidden/disconnected 态不匹配返 false 布尔 → hidePopover 静默
> no-op（popover-utils「should not throw」语料权威）。**落地三件**
>（R5233）：shim `_zwHidePopover` lenient 化（非 popover
> NotSupportedError / showing+断连 InvalidStateError / 其余静默）+
> matches 陷阱 `:popover-open` 裸伪类特判（:focus/:modal 同款 JS 侧特判）
> + part09 契约测试翻转与三态断言。**集群**：82 files 411 subtest
> **321P/89F/1T**，同 key 406 对 s31 逐位 diff **REG 0 / GAIN 0**；
> closedby-simple Timeout 解体 → 6 子测收敛 5F+1P；complex 40F 红因迁移
> 为统一行为缺口。**探针钉定两缺口**（一次 outside click → dA/dB 全关、
> pA/pB 不关）：**挂账 E（下一切片主目标）popover light dismiss 未建**
>（R3071 defer 至今）+ **dialog light dismiss 链停**（一次 click 应只关
> topmost clicked dialog 链）；simple 第二 ESC 断言面（ESC 栈序）同挂。
> focus-after-close.html Timeout 既有挂账维持。gates：zero-engine lib
> **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s32-popover-open-semantics.md](html-semantics-compat/evidence/2026-10-23-m2s32-popover-open-semantics.md)。
> **下一步**：挂账 E（popover light dismiss + 链停语义，engine click 路
> 径）或按挂账序挂账 A（强制同步布局冲刷）、impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 31 收口）**：dialog showModal 布局
> 域。**谜题破案**：s30「dialog[open] computed block 但 gBCR 全零」是探
> 针伪象——gBCR 读 renderer 上一帧 rect 快照（rect_bridge），open 同 tick
> 读拿到旧布局；等帧后全部有盒。探测环刷新顺序（flush→timers→刷新）钉
> 死 stale 窗口：abspos-dialog-layout 9 案中 6 案红因 = showModal 同 turn
> 测量双侧 stale（expected 300/got 0），**挂账 A（本流）：强制同步布局
> 冲刷**（gBCR/offset* 读前冲刷 + 重排，CSSOM view 语义）。**`:modal`
> 通道落地**：模态性只在 shim JS 栈、DOM 序列化不携带 → 镜像 :focus 注
> 入通道（R4241）建全链——dom `Document.modal_dialogs`/`is_modal_dialog`
> + pipeline `inject_modal_state`（fresh-parse 后注入）+ webview 转发 +
> runner 探测环轮询 `_zwDialogModalAllSels()`（shim 全量导出，栈序、嵌
> 套模态皆匹配）+ matcher `:modal` 臂——作者 `dialog:modal` 规则自此真
> 实匹配。**UA 定位注入实验矩阵**（回归 0 纪律定性）：规范照抄（inset
> 0 + margin auto）= 9 回归（布局对 inset 指定 + size auto 走满视口拉
> 伸，light-dismiss 盒外点击全族失效）；inset auto 近似 = 3 回归（出流
> 重叠、selection/editing 点击几何漂移）；不注入 = **0 回归 0 增益**
>（320P/85F/2T 与 s30 逐位打平）→ 终态不注入。**挂账 B（渲染流）：
> abspos 收缩/fit-content 语义补齐**，UA 规则随该切片按规范回填（A 形
> 态实验已端到端验证轮询链）；s30「ancestor-is-inert 红因 = UA
> position:absolute 缺失」修正为「注入受布局收缩缺口阻断」。挂账 D：
> getComputedStyle 路径 `:modal` 注入（低优）。gates：zero-dom lib
> **863P/0F**、zero-engine lib **3428P/0F**（+1）、zero-style-system lib
> **2446P/0F**（+1）、clippy -D warnings、fmt clean、零新增依赖；渲染/
> UA 样式净变化为零免 product-smoke。账本见
> [evidence/2026-10-23-m2s31-dialog-modal-channel.md](html-semantics-compat/evidence/2026-10-23-m2s31-dialog-modal-channel.md)。
> **下一步**：挂账 A（强制同步布局冲刷，解锁 abspos/non-modal 布局语
> 料同 turn 断言面）或按挂账序 popover 有效性语义（`:popover-open` 选择
> 器 + hidePopover not-showing 抛 + popover-simple 余 hang）、impl-doc
> wrapper requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 30 收口）**：rendering inert 域第一
> 刀——hit-test 切面收口。宿主指针命中（runner `resolve_pointer_target`
> 候选循环）不排除模态 inert 子树，点击模态外元素照常命中。修复三件：
> ① 命中候选循环接 inert 排除（跳过 `!_zwDialogModalFocusGateOk(e)`——
> s29 门语义与 hit-test inert 排除同源直接复用，门补 globalThis 导出供
> 页面域调用，缺省回落原行为）；② backdrop 命中回落：候选全排除且栈顶
> 模态存在 → target=栈顶模态 dialog（新增 part01 `_zwDialogModalTopSel`；
> ::backdrop 无布局盒的近似）——真 Chrome 仲裁（ancestor 语料独立探针）
> 实测「点击 inert 区域 target=HTML」（backdrop 被 display:none 不渲染、
> 穿透到根），回落读 styleSheets 文本扫 `::backdrop{...display:none}`
> 命中则 target=documentElement；③ light dismiss backdrop 臂
>（`__zw_dialog_light_dismiss_check` 加 px/py）：backdrop 回落 target=
> dialog 曾压制 dismiss——target=栈顶模态 dialog 本身且落点在 gBCR 外 =
> 点击 backdrop → 照发 close request（真 Chrome md:dialog 且关闭）。
> 账本：dialog 集群 82 files **320P/85F/2T**（s29 同集 317P/87F/3T），
> 同 key 回归 **0**、增益 2（blocks-mouse-events F→P、light-dismiss-
> touch F→P 连带）+ backdrop-receives-element-events **Timeout→Pass**
>（3T 基线之一收口，backdrop 命中面直接语料）。gates：engine lib
> **3427P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s30-modal-hit-test.md](html-semantics-compat/evidence/2026-10-23-m2s30-modal-hit-test.md)。
> **本轮钉定的相邻归因**：ancestor-is-inert 红因已换——dialog showModal
> 后 UA `position:absolute` 缺失（探针 pos=static、盒 (101,201,50x50)
> 落进父 div 内容区），归 abspos/layout 账（与「showModal 居中」同根），
> 本轮命中面修复已为该案铺平；inert-svg-hittest `:active` 断言引擎恒
> false（matcher 运行时伪类臂缺 active/hover），归交互态伪类账；
> selection 两案依赖指针拖选→Selection 链路，挂账；dialog-inert 依赖
> 居中布局，归 abspos 账。
> **下一步**：dialog UA showModal 布局（absolute + inset:0 + margin:auto
> 居中——abspos-dialog-layout ~10F 与 ancestor-is-inert/dialog-inert 同
> 根）或按挂账序 popover 有效性语义（`:popover-open` 选择器 +
> hidePopover not-showing 抛 + popover-simple 余 hang）、impl-doc
> wrapper requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 29 收口）**：模态焦点门接页面
> focus()——s28 门只覆盖宿主驱动路径（pre-focus 步、mousedown 默认动
> 作），页面直调 `el.focus()` 不在门内，inert-node-is-unfocusable 13F。
> 修复四件：① `part04 focus()` trap 接门（已聚焦 no-op 之后、失连臂之
> 前；失连元素门内跳过保 R114/T55 detached 契约）；② 门位在 R4003b
> label 前递**之后**——前递源（label 本就非 focusable area）不受模态
> inert 门，inert 语义作用在焦点落点，前递目标经递归 focus() 再入 trap
> 自行过门（inert-label-focus 断言面钉次序：门前置时该案 PASS→FAIL，
> 移位后归位）；③ showModal 栈推进移到 focusing steps 之前（嵌套模态
> 子树 focusing steps 过门）；④ R4600b inert 分支 focus fixup 落 body
> 走 `__zwModalGateBypass`（fixup 是 UA 级焦点回落非 focusing steps，
> Chromium 同款 activeElement===body）。门包含判定补 shadow-including：
> `proxy.contains` 只覆盖 light DOM，既有契约 R4600 R8（delegatesFocus
> 影子树内锚）暴露缺口——新增 part01 `_zwDialogShadowInclContains`
>（parentNode 上行遇 shadow root（`__zwHandle` 命中 `_shadowHandles`）
> 换 host 继续、frag 容器经 `_shadowHandleMeta` 跳 host，路数同 part06
> activeElement retarget，64 步 guard 防反链环）。契约
> `test_dialog_modal_focus_gate_s28` 增 G5/G6（页面 focus() 拒焦／失连
> 归解析节点所有权）。dialog 集群 82 files 407 subtest **317P/87F/3T**
>（s28 同集 303P/101F/3T），逐 subtest diff 同 key 回归 **0**、增益
> **14**（inert-node-is-unfocusable 13F 全清 15/15 +
> remove-dialog-should-unblock-document 连带 F→P）；gates：engine lib
> **3426P/0F**（含 G5/G6）。账本见
> [evidence/2026-10-23-m2s29-page-focus-gate.md](html-semantics-compat/evidence/2026-10-23-m2s29-page-focus-gate.md)。
>
> **▶ 上轮状态（2026-10-23 M2 切片 28 收口）**：corner-cases 余 2F 收口
> ——先建真 Chrome 基准仲裁（playwright-core + chromium-1243，CDP 真实
> 输入层；test_driver 桥接补 ButtonType/特殊键映射），语料基线 29/30
> 唯一失败＝esc-modeless。两案定性：esc-modeless 非 bug（真 Chrome 事件
> 流与引擎逐条一致，上游期望 closed 在 Chrome 自身即失败，记账 Chrome
> parity 非 gap）；click-modal 引擎分歧（真浏览器不可能聚焦模态外元素，
> runner pre-focus 步泄焦）→ 修复＝模态 inertness 焦点切面：`part01
> _zwDialogModalFocusGateOk`（栈顶 open+模态 dialog 子树外非 focusable
> area，stale 栈项跳过，stacked modal 仅栈顶生效）+ `part06
> _zwIsFocusableSel` 接入（宿主 pre-focus 步与 mousedown 默认动作共用
> 门）。契约 `test_dialog_modal_focus_gate_s28` G1–G4。集群 **303P/101F
> /3T**，回归 0 增益 1（corner-cases **29P/1F == 真 Chrome 29/30**）。
> 账本见
> [evidence/2026-10-23-m2s28-modal-focus-gate.md](html-semantics-compat/evidence/2026-10-23-m2s28-modal-focus-gate.md)。
>
> **▶ 上轮状态（2026-10-23 M2 切片 27 收口）**：corner-cases 毒化链根因
> 收口（R4620 批内 selector 时序）——根因在宿主共享 apply 路径而非调用方：
> JS 侧同 dispatch 的 op 引用批开始时的树态（spec insertBefore 的 ref 节点
> 对象不因前序 remove 失效），而 drain 顺序重放时前序 Remove 摘除 dialog →
> ref script 的 `:nth-child` 索引漂移（3→2）→ INS target 失配 warn+skip →
> dialog 永久 detached、同批 SetAttr 连锁失配（remove_child 已清 id_map）→
> 记录毁灭后 29 案 setup 全部 closedBy=none、ESC close request 找不到
> dialog（探针宿主 warn 实锤，R361 stash 走不到——target 先失败）。修复＝
> `apply_dom_mutations_full` 批前预解析 insert-adjacent 族四变体
> （Html/Text/Element/SelElement，同构失配语义）target 为 NodeId（不随树
> 态漂移）+ 四臂失配查表兜底（ref 被批内摘除时维持 skip，语义对齐 JS 抛错
> 面）。契约 `dom_bridge_tests::test_apply_ins_target_pre_remove_index_drift_r4620`
> （修复前红/修复后绿）。dialog 集群 82 files **302P/102F/3T**（s26 同集
> 272P/135F，+30P/−33F），逐 subtest 对 s26 账本 diff 同 key 回归 **0**、
> 同 key 增益 27（全落 corner-cases 毒化案）；corner-cases **1P/29F→28P/2F**，
> 3T 全为 s26 基线既有；focus-shadow 50P/0F、closedby 27/27、requestclose
> 75P 维持；九域全量留待下轮刷新（本轮证据＝集群逐 subtest diff）。gates：
> engine lib **3416P/0F**（含新契约）、clippy -D warnings、fmt clean、零新
> 增依赖。账本见
> [evidence/2026-10-23-m2s27-r4620.md](html-semantics-compat/evidence/2026-10-23-m2s27-r4620.md)。
> 余 2F＝`focusin calls showModal` 变体（focusin handler 内 re-entrant
> showModal 的 ESC/light-dismiss 判定边缘；clicking/modal 键 s25 同 key
> 既有）。
> **下一步**：corner-cases 余 2F（re-entrant showModal 边缘，同域小残差）
> 或按挂账序推进 inert 计算态 13F、abspos/layout ~10F（rendering 域）、
> popover 有效性语义（`:popover-open` 选择器 + hidePopover not-showing
> 抛 + popover-simple 余 hang）、impl-doc wrapper requestClose、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不
> 出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-22/23 M2 切片 26 收口）**：dialog focusing steps 家族
> 收口（R4600 + rollover 挂死修复 R4600c/d）——spec show/showModal step 的
> run the dialog focusing steps、
> close-the-dialog 的焦点恢复落地：`_zwDialogFocusingStepsR4600`（autofocus
> 委派 → focusable area 递归判定（tag 表 + disabled/hidden/inert 子树传播/
> closed dialog 排除/delegatesFocus）→ focus 派 focusin）+
> `_zwDialogRestoreFocusR4600`（previously focused 捕获与恢复）+
> activeElement JS-observable（part06 focus chain retargeting：shadow 树内
> 沿 parentNode 上行解析焦点锚）。R4600b：focus-shadow 余 4F 根因＝R371
> replaceWith 重键的 `_r371Next` 后继快照取到**换行空白文本节点**（nodeType
> 3 无 selector），元素级 QSA R322 归并锚点只认 sel 字符串 → 恒尾插 → 焦点
> 扫描先命中后随兄弟；修复＝快照经 `_childNodeList(父sel)` 元素扫描推进到
> 最近后随**元素**兄弟。rollover 会话 R4600c/d 修 s26 树全量暴露的 3 文件
> 确定性整文件挂死（dialog-autofocus 族，load+focusin 双到齐缺 focusin
> ——runner 预注入 autofocus 先于页脚本，focusing steps 事件相位派发时
> 页监听器未注册且对已聚焦元素不重派）：part06 load 分支补 spec
> flushing-autofocus-candidates（processed flag + 首个合格候选 defer
> task，已聚焦态补发事件相位）+ part01 focusable area 补 pending-removed
> 门（元素级 QSA 无 overlay，同步摘除后仍命中）。testdriver click/down
> focus 步骤经 `_zwFocusSel` 门。
> part66 `test_dialog_focusing_steps_r4600` R1–R13 新契约。九域对账
> 5070→**5116P**（68.32%，Timeout 173 持平、subtests 7488 不变）：
> R4600c/d 隔离对照 **+3P/0 回归**（dialog-autofocus 族 3 挂死文件转
> Pass）；余 +43P 为 R4600 主体（focus-shadow 50F→0、closedby 27/27
> 维持）；同 key 回归 27 全落 corner-cases 毒化链（下述，s26 树已
> 同态）。corner-cases **1P/29F**（s25 28P/2F）：R4600 使 focusin 真实
> 触发音量 handler（remove+reinsert 共享 dialog），test#0 通过但毒化共享
> dialog → 后续 setup 全部 closedBy=none/conn=false——**新暴露**非新引入
> （s25 缺口休眠因 focusing steps 无操作），毒组合＝[reinsert#1+show+drain+
> ESC#1+focus+reinsert#2]，E1-E8 单因子全过；三层缝隙移交下轮：①R334 重
> 插 wire 完整性 ②ESC 派发锚 activeElement 反查 miss ③`_zw_esc_dialog_cancel`
> QSA-miss 顺栈 purge。另发现未修：R371 rekey 时 dst 桶不存在 → 被替换
> template 不入父桶 removed 合并视图残留（对扫描无害，spec 违例候选下轮）。
> **下一步**：corner-cases 毒化链三层缝隙（R334 重插 wire 完整性/ESC 派发
> 锚点/组栈 purge）；余挂账 inert 计算态 13F、abspos/layout ~10F（rendering
> 域）、popover 有效性语义（`:popover-open` 选择器 + hidePopover not-showing
> 抛 + popover-simple 余 hang）、impl-doc wrapper requestClose、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不
> 出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-22 M2 切片 25）**：corner-cases hang 收口（R4500）
> ——真根因横切两层：①shim `close()` IDL 包装透传 helper 的 wasOpen
> boolean（spec void）；②语料 testharness `is_valid_cleanup_result` 对
> cleanup 返非 undefined **及** `on_error` 对 cleanup 抛错均 `tests.abort()`
> → 完成回调不达 → file Timeout 吞全文件。`t.add_cleanup(() =>
> dialog.close())` 返 false 一个值吞 27-29 案；s23「detach 窗口身份链」
> 归因作废（hang 案实为前一案 cleanup abort，探针五文件 DA/DB 对照 +
> partial-flush 形态学定位）。修复：part04 三处 close 包装 void 化 +
> part14 R3290 两处旧断言更正 + part60 `test_dialog_void_return_r4500`
> （四方法全生命周期 + 八方法 void 族审计无第二 offender）。九域
> 5016→**5070P**，同 key 回归 **0**、翻转 **0**；Timeout 180→173（7 文件
> 解除吞没：closedby 1P→27/27、corner-cases 1P→28P/2F、focus-shadow
> 50F 新上报等），+55F 为新上报真断言非回归（分母 +102 漂移，百分比口径
> 不可比）。余 1 hang：popover-closedby-simple（cleanup `hidePopover()`
> 抛 not-showing → abort 同族——病根 popover 有效性语义，popover 域挂
> 账）。corner-cases 余 2F＝closing steps 内 focus fixup 不派 focusin
> （focusing steps 家族）。
> **下一步**：focusing-steps 家族（focus fixup→closing steps 派 focusin、
> autofocus 委派、activeElement 语义——corner-cases 2F + focus-shadow
> 50F + focusing-steps/activeElement 原挂账 ~15F 同域合并攻）；余挂账
> inert 计算态（inert-node-is-unfocusable 13F 精化）、abspos/layout
> ~10F（rendering 域）、popover 有效性语义（`:popover-open` 选择器 +
> hidePopover not-showing 抛，popover-simple 余 hang）、impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element pointer
> 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、
> 非模态 close 不出组栈。


> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：继
> net-api（79.7% 收口）之后的下一个"整域收割"级主攻——html/semantics 覆盖
> forms/embedded-content/tabular-data 等真实站点核心语义面；form-validation 已
> 打过样（monthly 高亮项），全语义面从未系统圈过。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：表单校验/控件状态面有存量
> （form-validation + html-syntax M 系遗产）；各子域（forms 约束校验/控件状态/
> 提交语义、tabular-data 表格模型、embedded-content 媒体与 iframe 元素属性等）
> 通过率未知——M1 分域基线定形。
>
> **▶ 域边界（先例：R4114-N 深结构归档条款）**：本 goal 只做 **DOM/语义/API 面**
> （元素属性/IDL 接口/约束/状态机）；布局渲染差异（table 布局、multicol、控件
> 视觉）归 rendering-compat。已归档 html-compat 的余账关系 M1 盘点核对防重复
> 计账。

## Mission

以 WPT html/semantics corpus 分域为验收标准，按子域切片推进（建议序：forms
约束与控件状态 → tabular-data → document-metadata/grouping/text-level →
embedded-content → links/microdata），每域独立收口出数字。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **分域收口制**：一个子域一个数字，不设总通过率门禁（corpus 太大）
3. **轻量修复优先**：约束校验/IDL 面走 shim；触 DOM parser 核心的深结构改动
   单独立项征询

覆盖范围（十个初始子域，fetch 脚本 DIRS 同清单）：forms（最大，含深层子目录
按 web-animations 先例追加）/ embedded-content / document-metadata /
grouping-content / text-level-semantics / tabular-data / interactive-elements /
scripting-1 / links / microdata。

### 排除（明确不在范围内）

- **table/multicol 等布局渲染差异** — rendering-compat 域
- **iframe 浏览上下文树** — navigation-compat M3 门控域
- **媒体元素播放语义** — media 系已归档 goal 域（embedded-content 中的媒体
  元素属性面归本 goal，播放管线不碰）
- **ARIA/无障碍映射** — 未立项域，挂账

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | 元素 IDL 面/约束校验/状态机语义 | part 系 JS 语义 |
| dom/style-system | 属性解析与反射（不碰 parser 核心与布局） | 轻量优先 |
| WPT 资产 | html/semantics 十子域导入（分批） | fetch 脚本 + 账本回填 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 rendering-compat**：布局/绘制差异记账回流（table 布局是其活跃工作面），
  本 goal 不越界修渲染
- **与 navigation-compat M3**：embedded-content 的 iframe 面只做元素属性，
  frame 树语义挂其门控
- **与 css-parser 流（本会话 fuzz 修复侧）**：CSS 解析面零交集

---

## Done Criteria

- **DC-1** 十子域导入 + 分域基线 + csv planned 行回填（每域独立数据行）
- **DC-2** 逐域收口：forms / tabular-data / document-metadata / grouping /
  text-level / interactive-elements / scripting-1 / links / microdata /
  embedded-content 各自"分级通过率不再下行两个连续轮次 + 余案归因挂账"
- **DC-3** 回归测试账本纪律（imported-tests.txt）；make test 全绿、make reftest
  零回归、product-smoke 逐字节恒值

## 里程碑

M1 导入与分域基线（✅ 2026-10-15）→ M2 forms 域（最大杠杆，多切片）→
M3 其余九域依序推进 → M4 总收口归档。

## 轮次账

- **2026-10-23 M2 切片 43**：replaceWith self-removal 父桶记账收口（R5246
  一刀，s26 既有挂账「R371 rekey 时 dst 桶不存在，不记父桶 removed」）——
  sel 臂 remove 步补 `_zwHCLiveInvalidate` 直调（父桶 removed identity +
  全局 pending-removed + live 失效一体），合并视图残留终结、视图即 spec
  终态；记账先于 rekey，promote 分支留护栏。契约 part75
  test_dialog_replacewith_parent_bucket_accounting_r5246（R1/R2）+ part24
  r321 kids 断言更新（宿主残留 DIV 消失）。interactive-elements 全域
  98 files **419P/61F/2T 对 s42 逐位 REG 0 / GAIN 0**；lib **3434P/0F**
  （+1 契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s43-replacewith-parent-bucket.md](html-semantics-compat/evidence/2026-10-23-m2s43-replacewith-parent-bucket.md)。
- **2026-10-23 M2 切片 42**：非模态 close 不出组栈收口（R5245 一刀，
  s23 已知保守项）——close-the-dialog 组栈出栈移出模态臂，modeless
  show/close 循环 stale 项无限增长终结；契约 part74
  test_dialog_group_stack_hygiene_r5245（R1–R4 行为不变量：循环/栈序/
  全关 no-op/模态臂保持）。interactive-elements 全域 98 files
  **419P/61F/2T 对 s41 逐位 REG 0 / GAIN 0**；lib **3433P/0F**（+1
  契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s42-nonmodal-close-stack.md](html-semantics-compat/evidence/2026-10-23-m2s42-nonmodal-close-stack.md)。
- **2026-10-23 M2 切片 41**：挂账定性轮——「ESC 组栈边缘乱序」收口为
  Chrome-parity 同红（corner-cases「focusin calls showModal, modeless」
  ESC 案：crbug 417571176、父 376516550，2026-04 wpt.fyi 仍红）。探针
  钉定事件序 + spec 逐字对账（R4600 恢复门/R5236 组栈零漂移）；R5244
  假设门（subject 非 focusable no-op）GAIN 1 / REG 4（dialog-focus-shadow
  Chrome「show() 聚焦 dialog 自身」面）试错回退，零代码净变化。全域
  419P/61F/2T 对 s40 逐位 REG 0、lib 3432P/0F、零新增依赖。账本
  [evidence/2026-10-23-m2s41-esc-stacked-parity.md](html-semantics-compat/evidence/2026-10-23-m2s41-esc-stacked-parity.md)。
- **2026-10-23 M2 切片 40**：custom command（`--` 前缀）CommandEvent
  派发收口（R5243，s24 defer）——part05 CommandEvent 接口
  （`_defineEventSubclass`，command=''、source=null，无 relatedTarget）
  + part01 `_zwInvokerCommandActivate` custom 分支派发后 return
  （command=原值大小写保留、source=invoker、cancelable=true、
  bubbles=false/composed=true、R312 trusted 印记；分派在调用方
  disabled/preventDefault 门内零新写）。interactive-elements 全域
  98 files **419P/61F/2T 对 s39 打平**；dialog 集群对 s35 **REG 0 /
  GAIN 5**（s39 既有收益）；forms/the-button-element 对 s24 逐位一致。
  gates：engine lib **3432P/0F**（+1 契约 part73）、clippy/fmt 过、
  零新增依赖。账本
  [evidence/2026-10-23-m2s40-custom-command-event.md](html-semantics-compat/evidence/2026-10-23-m2s40-custom-command-event.md)。
  挂账：现行 spec invalid→Unknown 面（跟进涉内建关键字重构）、
  popover 关键字、oncommand IDL、command/commandFor IDL 反射、
  ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留、
  summary-less details 非激活后代翻转、focus-after-close Complex
  dialog usage 深层。
- **2026-10-23 M2 切片 27**：corner-cases 毒化链根因收口（R4620 批内
  selector 时序）——根因在宿主共享 apply 路径：drain 顺序重放时前序
  Remove 摘除移动语义节点 → 后续 INS ref 的 `:nth-child` 索引漂移 →
  target 失配 warn+skip → dialog 永久 detached + 同批 SetAttr 连锁失配
  （JS 侧 op 引用批开始时树态，spec insertBefore 的 ref 不因前序 remove
  失效）。修复＝`apply_dom_mutations_full` 批前预解析 insert-adjacent
  族四变体 target 为 NodeId + 四臂失配查表兜底；契约
  `test_apply_ins_target_pre_remove_index_drift_r4620`（前红后绿）。
  dialog 集群 82 files 272P→**302P**（−33F），同 key 回归 **0**、增益 27
  （corner-cases 1P/29F→**28P/2F**，余 2F＝re-entrant showModal 边缘）；
  3T 全为 s26 基线既有；九域全量留待下轮刷新。gates：engine lib
  3416P/0F、clippy/fmt 过、零新增依赖。账本
  [evidence/2026-10-23-m2s27-r4620.md](html-semantics-compat/evidence/2026-10-23-m2s27-r4620.md)。
  挂账：corner-cases 余 2F（re-entrant showModal）、inert 计算态 13F、
  abspos/layout ~10F（rendering 域）、popover 有效性语义、impl-doc
  wrapper requestClose、anchor-with-inline 导航缺口、parser form element
  pointer 4F、custom command `--` 前缀、ESC 组栈边缘乱序、非模态 close
  不出组栈、R371 模板残留。
- **2026-10-22/23 M2 切片 26**：dialog focusing steps 家族（R4600）+
  rollover 挂死修复（R4600c/d）——show/showModal 跑 spec dialog
  focusing steps（autofocus 委派 → focusable area 递归判定 → focus 派
  focusin）+ close-the-dialog 焦点恢复 + activeElement 焦点链
  retargeting + R4600b replaceWith 空白锚点（focus-shadow 50F→0）；
  rollover 会话修 3 文件确定性整文件挂死：part06 load 分支补 spec
  flushing-autofocus-candidates + part01 focusable area 补
  pending-removed 门。九域 5070→**5116P**（68.32%），Timeout 173 持平，
  R4600c/d 隔离对照 +3P/0 回归；同 key 回归 27 全落 corner-cases
  毒化链（R4600 激活 focusin → 毒化共享 dialog，新暴露非新引入）。
  make test 76 腿全绿、clippy/fmt 不涉（纯 .js）、零新增依赖。账本
  [evidence/2026-10-22-m2s26-r4600.md](html-semantics-compat/evidence/2026-10-22-m2s26-r4600.md)。
  挂账：corner-cases 毒化链三层缝隙（下一切片首选）、inert 计算态
  13F、abspos/layout ~10F（rendering 域）、popover 有效性语义、
  impl-doc wrapper requestClose、anchor-with-inline 导航缺口、parser
  form element pointer 4F、custom command `--` 前缀、ESC 组栈边缘
  乱序、非模态 close 不出组栈、R371 模板残留。
- **2026-10-22 M2 切片 25**：corner-cases hang 排查（R4500）——横切根因
  收口：shim `close()` void 违约（透传 wasOpen boolean）× 语料
  testharness cleanup abort 语义（返非 undefined / 抛错均
  `tests.abort()`）→ file Timeout 吞全文件；s23「detach 窗口身份链」
  归因作废。修复 part04 三处 close 包装 + part14 两处旧断言更正 +
  part60 void 族契约 `test_dialog_void_return_r4500`（R1-R3）。九域
  5016→5070P（同 key 回归 0、翻转 0），Timeout 180→173（closedby
  1P→27/27、corner-cases 1P→28P/2F 等 7 文件解除吞没），+55F 为新上报
  真断言非回归。余 popover-closedby-simple 1 hang（cleanup hidePopover
  抛 not-showing → abort 同族，popover 域挂账）。gates：engine lib
  3394P/0F、webview 785→786P/0F、page-runtime 136P/0F、integration lib
  787P/0F、clippy/fmt 过、零新增依赖；中途 stash→rebase→pop 并入远端
  共享面提交（floats + workers 呈现管线，同动 part04.js 无冲突）。证据
  见 [evidence/2026-10-22-m2s25-r4500.md](html-semantics-compat/evidence/2026-10-22-m2s25-r4500.md)。
- **2026-10-22 M2 切片 24**：ToggleEvent.source / commandfor 簇
  （R4400）——toggle-source 三文件 3/10→10/10，九域 5008→5016P（+8 回归
  0）。含横切修复：user_actions.rs plan 通道私有批回灌共享队列全史 +
  prevented request-close 丢弃 invoker source。证据见
  [evidence/2026-10-22-m2s24-r4400.md](html-semantics-compat/evidence/2026-10-22-m2s24-r4400.md)。
- **2026-10-22 M2 切片 23**：dialog closedby 簇（R4300，rollover 接续
  ——上会话 429 中断遗留实现+契约 WIP，本会话核验收口）——两段推进：
  ①runner 通道：inline_local_scripts 根路径按 wpt_root 内联（域外助资
  产 `/common/top-layer.js`、`/close-watcher/resources/helpers.js` 等
  可达，harness 本体先前已被 replace_script_source 消费无双注入）+
  send_keys uE00C Escape 臂（`__zw_dispatch_key_at_active` 焦点元素三
  形态派发：`__zwSelector`/`_proxyCache` 反查/wrapper 直派；未取消 →
  `__zw_esc_dialog_cancel` 默认动作）+ click 命令 light dismiss 前步 +
  goal 脚本 5 助资产 fetch；②产品语义：closedBy IDL（小写化 + auto 派
  生 + 反射）+ `_zwDialogClosedByState` + close watcher 组栈
  `_zwDialogModalStack`（show/showModal 压组、首个 open 组即停、组内
  none 挡下探、markup/IDL open 惰性补登记、stale 顺栈清出）+
  `__zw_esc_dialog_cancel` 重写（closedby 资格门 + 组栈序 +
  `_zwDialogRequestClose`）+ `__zw_dialog_light_dismiss_check`（任何
  open+any dialog 包含链外落点 → close request；`__zw_pointer_down_
  sequence` 头部钩与 runner click 共享）+ `__zw_reset_form_state` 清
  栈。part60 R1–R5 契约。九域 4980→5008P（68.02%），对 s22 逐 subtest
  diff 净 +28P、回归 1：interactive-elements 202→227P（canceling 族假
  Fail 转真断言：ESC 主断言/组栈 none 挡板 4P/light dismiss 9P/
  cancel-events 族新上报 4P/closedby 逐案 7P/non-modal-canceling 1P）
  + scripting-1 +3（SVG script scheduler 意外翻绿稳定，机制未定位观
  察账），余七域 0 变化；回归 1（bounds-clicking (1,1)：dialog UA
  abspos 布局/hit test 缺口，归 rendering 域）。zero-engine --lib
  3386P/0F、page-runtime 136P/0F、webview 785P/0F、integration
  html_compat 21P/0F、clippy（test-guard 20G 包裹）/fmt 过、零新增依
  赖。挂账：corner-cases ~28 案 file Timeout（focusin remove+reinsert
  → showModal → ESC close 不达，detach 窗口身份链）、ToggleEvent
  .source/commandfor 8F 真断言（下一切片）、ESC 组栈惰性补登记边缘乱
  序（保守）、非模态 close 不出栈（读侧 purge 兜）、bounds-clicking
  (1,1)（rendering 域）、余 s22 挂账维持。账本
  [evidence/2026-10-22-m2s23-r4300.md](html-semantics-compat/evidence/2026-10-22-m2s23-r4300.md)。
- **2026-10-22 M2 切片 22**：dialog 元素簇（R4200，跨午夜会话）——两段
  推进：①part01 dialog 语义主体重构：toggle 事件任务队列（R4009 同款
  per-element 去重合并、preOpen 调用方写前捕获）+ close 事件排队异步
  （spec queue an element task，trusted 通道，旧同步派发 onclose 早于后
  续脚本）+ show/showModal 请求态语义（modal 上 show 抛/非模态上
  showModal 抛，旧静默切态）+ beforetoggle 同步可取消 + 写后重查 +
  close-the-dialog 核心共用 + `_zwDialogRequestClose`（cancel 可取消 +
  重入 busy 守卫，part04 DIALOG tag 门分发）；②共享基建：part03
  Promise.withResolvers polyfill + `_realTag` sel 印章（detach 窗口）、
  part05 `_insertAdjacentVariadic` sel 项（prepend(parsedEl) 丢失）、
  part04 matches 裸属性 lw + addEventListener `{signal}` per-record +
  set trap open moAttr 尾钩、part01 show/showModal/close 调用点 MO
  record（aria helper 共用防双计）。part60 T1–T8 新契约 +
  part14 ④⑧/part59 W3 契约同步（close 排队断言）。九域 4887→4980P
  （67.54%），对 s21 逐 subtest diff 净 +93P、回归 0，全落
  interactive-elements（109→202P）：requestclose 0→75P、toggle-events
  0→12P、close/close-event-async/requestclose-2/-recurse/-closedby/
  submit-dialog-close-event 各 +1。zero-engine --lib 3378P/0F、
  page-runtime 136P/0F、webview 785P/0F、integration html_compat 21P/0F
  （包名 zero-integration-tests 口径补正）、clippy/fmt 过、零新增依赖。
  挂账：closedby 簇 ~85F runner testharness helper 缺口（下一切片）、
  inert 计算态 ~15F、focusing steps ~15F（wrong global realm 面）、
  abspos/layout ~10F（rendering 域）、impl-doc wrapper requestClose、
  MO 内部写仅覆盖 dialog open（保守面）、`_zwSelSeenTag` 幽灵 sel 误
  服务（保守已知）、其余 s21 挂账维持。
- **2026-10-21 M2 切片 21**：details name= 互斥簇（R4100）——spec
  exclusive accordions 双 prong（others 开写面关其他 open / given 插入与
  name 变更面自身让位）+ 组口径（同树根 + 同名非空 name + DETAILS）。
  核心结构发现：innerHTML-into-sel 解析产物 `_zwMEl` plain wrapper 与
  t8r 物化 identity 元素混合同树——统一 node-based 核心
  `__zwDetailsExclusivityNode4100`（childNodes DFS 组枚举、`_elKey`/对象
  身份双态）+ R4009 队列第 4 参 plainNode 通道 + 解析扫描顺序插入模拟
  （树序第一 open 胜出，CeAttach 整批一次扫描）+ 六挂接面 + 解析面三处。
  part01 `_REFLECTED_NAME_TAGS` details + part03 原型 name accessor。
  取舍：解析面排程 toggle（spec 字面）与 test 3 期望序互斥，取 spec 口
  径维持 test 3 F。part60 T1–T8 契约。九域 4873→4887P（66.29%），对 s20
  逐 subtest diff 净 +14P、回归 0，全落 interactive-elements（95→109P）：
  name-attribute 2P/14F+Timeout→15P/3F、fragment-insertion 1P/0F。
  zero-engine --lib 3374P/0F、page-runtime+webview 785P/0F、integration
  html_compat 21P/0F、clippy/fmt 过、零新增依赖。挂账：toggleEvent 尾部
  ontoggle（document-parse 面）、name-attribute test 3 期望序互斥观察、
  xhr/template setup error、dialog 元素簇（下一切片）。账本
  [evidence/2026-10-21-m2s21-r4100.md](html-semantics-compat/evidence/2026-10-21-m2s21-r4100.md)。
- **2026-10-21 M2 切片 20**：details/summary 深层面（R4009，rollover 接续
  ——上会话 429 中断遗留 WIP 本会话续推收口）——toggle 事件任务队列
  （part03：去重/合并/净零/同态不排程/detached；choke point part04 `open`
  IDL setter + attr 尾钩）+ summary 激活行为（part03 bubble 站钩 + part04
  post 分支，首个 summary 元素子 + 交互豁免含祖先面）+ ToggleEvent 接口
  （part05）+ 宿主空格路由（webview SUMMARY → plan_summary）。**宿主双
  toggle 缺陷重设计**：plan_summary followup toggle 与队列双派被集成测试
  证伪（`toggle:true,toggle:true`），移除 followup 发射权归队列，断言
  `click:false,toggle:true,click:true` 不变、事件形状转正。part60 T1–T9。
  九域 4855→4873P（66.09%），对 s19 逐 subtest diff 净 +18P、回归 0，
  全落 interactive-elements（77→95P）：toggleEvent 9P/0F、
  activation-behavior 9P/0F、anchor-without-link 2P/0F、
  details-keyboard-activation 1P/0F。zero-engine --lib 3358P/0F、
  page-runtime 136P/0F、webview 785P/0F、renderer activation 9P/0F、
  integration html_compat 21P/0F、clippy/fmt 过、零新增依赖。挂账：
  details name= 互斥簇 18F+Timeout（下一切片首选）、toggleEvent 尾部
  ontoggle 内容属性面、ToggleEvent.source/commandfor、anchor-with-inline
  synthetic click 导航缺口（导航域）、parser form pointer 4F、
  navigation-compat M3 项维持。
- **2026-10-21 M2 切片 19**：label 本体激活转发（R4008，rollover 双触发
  两会话同树协作合账，commit 88793f87d）——shim 交互豁免身份比较（bubble
  站挂钩 + R155 post 段，target 即 label 本体不豁免）+ 宿主面上移（renderer
  activate_label_at 既有 + webview user_actions plan 链递归 Activate，
  labelable 过滤防环/取消抑制/disabled noop）；shim 不承接宿主 generic
  Event（首发 shim finally 段转发被 renderer 双激活集成测试证伪后重设计，
  checkbox 双翻）。part60 W1–W6 重写 + integration 三面新测。九域
  4848→4855P（65.91%），对 s18 逐 subtest diff 净 +7P、回归 0：
  proxy-modifier tentative 4F 全翻绿（主面）、label-forwarded-click-pointer
  -properties 未上报→2P/0F（isTrusted trusted 面随宿主派发通道翻绿）、
  simulated-click-inert 翻绿。zero-engine --lib 3355P/0F、webview lib
  785P/0F、renderer activation OK、integration html_compat 21P/0F、
  clippy/fmt 过、零新增依赖。协作分工：对侧会话代码提交 + 全量 corpus，
  本会话 kill 冗余全量轮 + docs 合账 + 账本独立复算。挂账：parser form
  element pointer 表规则 4F 同根（深结构单独立项）、details 键盘激活 /
  summary toggle coalescing、navigation-compat M3 项维持。
- **2026-10-21 M2 切片 18**：label.form IDL（R4007，rollover 接续）——
  `.form` getter 分发区补 LABEL 分支，`_zwLabeledControl` 取控件后委托
  控件 `.form`（R4005 关联面全继承），label 自身 form 属性不参与。
  part60 契约 1 测（W1–W3）。九域 4838→4848P（65.82%，账本条目口径），
  对 s17 逐 subtest diff 净 +10P、回归 0：form.html label.form 8 案
  翻绿（7P/10F→15P/2F）、dialog autofocus 两案意外翻绿（焦点链联带）。
  zero-engine --lib 3325P/0F、make test 76 腿 22222P/0F 全绿、clippy/fmt
  过、零新增依赖。挂账：parser form element pointer 表规则 4F 同根
  （深结构单独立项）、proxy-modifier tentative 4F、details 键盘激活 /
  summary toggle coalescing、navigation-compat M3 项维持。
- **2026-10-21 M2 切片 17**：form owner 动态关联（R4005/R4006，rollover
  接续）——`.form` getter 重写（JS 视图 parentNode 三族统一上行 + form
  attr 控件所在树查找 + latest-wins 读 + OBJECT gate）、R52 消零 purge
  WeakRef 复活（identity 断修复、泄漏口径不变）、gEBI pending 窗口同 id
  树序取首 + `_r125Overridden` 改名门。part60 契约 1 测（W1–W4）。九域
  4774→4838P（65.90%），对 s16 逐 subtest diff 净 +64P、回归 0：
  form_attribute.html 43P/57F→100P/0F 满绿、association.window.js 翻绿、
  object.form/form-elements-filter +1+1、select-tab* 意外翻绿 2、
  load-error-events 上报展开 +2。zero-engine --lib 3310P/0F、make test
  全腿绿（GPU adapter 94P/2F 容差漂移结构不阻塞非本轮、归 rendering 流
  观察账）、clippy/fmt 过、零新增依赖。全量轮 5400s 限被兄弟流并行负载
  顶爆、10800s 放行记 evidence。挂账：label.form IDL（form.html 余 10F，
  下一切片）、form_owner_and_table parser 指针表规则 2F（深结构）、
  proxy-modifier 4F、details 键盘激活 / summary toggle coalescing、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 16**：label 激活行为面（R4004）——交互内容判定
  存在性走 has 句柄（get_attr_handle 缺失返空串致恒真误判）、R368 盖章
  属性回填、label 激活转发三路归一（bubble 站挂钩新设 + post/trap 迁移
  `_zwLabeledControl` for 感知 + 交互 target 豁免补齐 + 双闸互斥收敛
  自转发死循环与 post+trap 双转）+ plainParent hop/R11 媒体补链/DFS 控件
  查找/盖章子事件方法转 live。part60 契约 1 测。九域 4710→4774P，对 s15
  逐 subtest diff 净 +64P、回归 0：clicking-noninteractive-* 57F 全数
  翻绿、意外翻绿 7 案（label-inside-anchor/proxy-click Timeout→Pass、
  dialog 系 5 案含 inert-label-focus 观察账消账）。zero-engine --lib
  3289P/0F、make test 主矩阵绿 + browser bin 腿既有失败 stash 对照定谳
  非本轮、clippy/fmt 过、零新增依赖。make test 与 clippy 并行触发 guard
  总内存超限（8GB 预算下编译峰值 ~8.8GB，16GB 放行）记 evidence。挂账：
  form_attribute 动态关联 ~50F（下一切片）、proxy-modifier tentative 4F、
  details 键盘激活 / summary toggle coalescing（M1 起既有）、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 15**：labels IDL 簇 + label.focus() 前递
  （R4003/R4003b/R4003c，rollover 双触发两会话同树合记）——part03
  `_liveQueryCollection` 泛化（自定义快照 + NodeList 原型 target +
  `__zwNL` 印记透读）、part04 labeled control 算法 + labels 反查活集合
  （[SameObject] per-element-key 缓存居 part01 共享作用域）+ focus 前递
  + body/html offsetParent→null。part60 契约 3 测。九域 4683→4710P
  （64.00%），对 s14 逐 subtest diff 净 +27P、回归 0（labelable-elements
  +21、button/input-labels +1+1、forward-focus +4；inert-label-focus
  失败模式漂移 +0，run2 二进制未含 R4003c）。zero-engine --lib
  3273P/0F、make test 58 腿绿 + browser bin 腿 xvfb 复跑 665P/0F、
  clippy/fmt 过、零新增依赖。并发碰头信号（共享 target 指纹 E0425、
  run kill 史 ×4、06:43 一次死因未考证）记 evidence。挂账：
  clicking-noninteractive-* 57F（下一切片首选）、form_attribute 动态
  关联 ~50F、proxy-modifier 4F、inert-label-focus 负载超时观察账、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 14**：meter 实际值算法 + double setter（R4002）——
  part04 `_zwMeterActualValues`（六 IDL getter 缺省/钳制 + 浮点整串解析，
  前导 + 跳过为语料钉死行为；R5009 片 e 旧前缀解析修正）+ 六 setter
  WebIDL double 转换（非有限 TypeError）+ best representation 写属性 +
  METER 移出 value 纯反射族（part03 host-read 委托支/part05 tag-gate
  不变）。part60 契约 1 测（W1–W5）。meter 簇 14P/48F→62P/0F，forms
  3688→3736，九域 4662→4683P（63.48%），对 s13 逐 subtest diff 净 +21P。
  **跨流窗口 −27P 对照实验归因、meter 排除**：B2 提交树 16 变更文件复跑
  与 B1 全量账本持平（账本绑定提交树）；无 meter 对照重建原样复现；
  scripting-1 −24 主假设归 script-sandbox 模块缓存三部曲（8458c9223/
  a85c7ade4——安静机恒 Timeout 而 s13 负载下 18P，方向与边界漂移相悖）、
  interactive-elements −3 签名匹配 s7 runner 通道 realm 家族（Fail
  "wrong global"）与窗口候选 3ccfdc479/b2fde4965/368ce9158 并存两说，
  commit 级 bisect 因旧 commit v8 源码构建指纹受阻留属主流。本切片由
  rollover 双触发并发会话同树推进，记账已合并（见切片账本说明）。make
  test 59 腿 58 绿 + 1 既有无 display 环境腿（xvfb 同二进制 665P/0F）、
  clippy/fmt 过、零新增依赖。挂账：labels IDL ~30F（下一切片）、
  form_attribute 动态关联 ~50F（中深）、clicking-noninteractive-* 57F、
  module/dialog 跨流碰头两项（属主流核账）、navigation-compat M3 项维持。
- **2026-10-19 M2 切片 13**：selected autocomplete value 算法 + HTMLCollection
  named access（R4001）——part04 category 全表 + token 算法（INPUT/SELECT/
  TEXTAREA getter，FORM/其余保持枚举反射；field token 小写为 corpus+Chrome
  67/67 钉死行为）、part05 setter 缓存旁路、part03 `_liveQueryCollection`
  named property 补齐（`document.forms.X` 可达）、part03 `.form` 表补
  FIELDSET（named access 修复暴露 vacuous-pass 真值化）。part60 契约
  W1–W5；part15 R3204 input 缺省断言 'on'→''（spec 修正）。九域
  4581→4662P（63.20%），对 s12 逐 subtest diff 净 +81P、回归 0（−11 条
  上报漂移全为 runner 超时边界，P 零损失）。zero-engine --lib 3245 全过、
  make test 全腿绿、clippy/fmt 过、零新增依赖。挂账：meter 净化 44F
  （下一切片）、labels IDL ~30F、form_attribute 动态关联 ~50F（中深）、
  encoding 三簇/double-submit 归 navigation-compat M3 维持、R3204 类
  既有契约钉旧行为风险记入（动枚举反射分发区先跑 part10/part15）。
- **2026-10-19 M2 切片 12**：populateForm 同 turn 二次插入 sibling 断链
  修复（R4000）——`_zwPendingParsedForSel` 候选收集 `candCount > 1`
  提前 return 缺陷删除（签名盖章截断致第二批 pending 节点 hits 恒 0，
  兄弟导航断链；歧义保守语义由 hits 唯一性判定承担不变）。part59 契约
  1 测（W11）。解锁切片 11 flag 契约：form-submission-algorithm t2/t3
  Fail→Pass，九域 4579→4581P（62.10%），对 s11 逐 subtest diff 净
  +2P、回归 0（账本绑定 rebase 后合并树，对 rebase 前复跑 0 变化）。
  zero-engine --lib 3224 全过、clippy/fmt 过、browser bin 腿 :102
  665P/0F、quickjs 矩阵腿全绿、零新增依赖。挂账：
  form-submission 余 F 归 iframe 导航基建簇（navigation-compat M3）、
  既有小挂账维持、二批插入后 body.childNodes 融合视图丢 base 尾节点
  （corpus 无消费面，观察项）。
- **2026-10-19 M2 切片 11**：firing submission events flag + dialog
  returnValue WebIDL 化 + 宿主 click 坐标注入（R3999）——每 form 粒度
  `__zwFiringSubmissionEvents`（spec 步 5.1/5.2/5.4/5.7，入口早退 +
  窗口置位 + finally 清位，仅激活/requestSubmit 通道）；returnValue
  setter DOMString 转换（null→"null"，undefined→"" 兼容例外）；泛型
  click offsetX/offsetY 派发期注入（与 image selected coordinates 同
  口径，事件类保持泛型不改 R108 面；并行 regression-guard 流协作归并，
  账本排查记录 3）。part59 契约 3 测 + part14 期望修正。九域
  4577→4579P（62.07%），对 s10 逐 subtest diff 净 +2P、回归 0：
  dialog-return-value 翻绿、dialog-form-submission 8/8。zero-engine
  --lib 3193 全过、make test 全腿绿（browser 腿 :102 复跑 664P/0F）、
  clippy/fmt 过、零新增依赖。挂账：populateForm 同 turn 二次插入 sibling 断链
  （R3254-K3 面，挡 form-submission-algorithm t2/t3，flag 契约已就位）、
  既有小挂账维持。
- **2026-10-18 M2 切片 10**：form 提交 construct-entry-list 对齐 + dialog
  提交分支（R3910）——constructing entry list flag 共享（submit 入口
  早退先于 submit 事件、构造窗口置位/finally 清位）、formdata 事件共享
  条目存储派发（handler 增删改写提交条目）、action 解析按 spec 顺序后移
  （旧位序丢 formdata/dialog 面）；method=dialog 分支（无祖先 return、
  image 坐标、optional value、_zwDialogClose）；close-the-dialog 步 1
  无 open 早退（四处共享路径）；SubmitEvent/FormDataEvent 构造器
  WebIDL 化。part59 契约 6 测。forms 3602→3605P、dialog-element
  38→47P、九域 4565→4577P（62.05%），对 s9 账本逐 subtest diff 净
  +12P、回归 0。zero-engine --lib 3169 全过、make test 全腿绿
  （browser 腿 :102 复跑）、clippy/fmt 过、零新增依赖。挂账：
  form-double-submit-requestsubmit 36F→Timeout（iframe 导航基建，归
  navigation-compat M3）、submit-dialog-close-event 挂起维持、
  firing submission events flag（t2-t4）、image 坐标 offsetX 面（t5）、
  dialog-return-value returnValue setter String 强转 1F。
- **2026-10-18 M2 切片 9**：selectedness/selectedIndex 语义（R3909）——
  selectedness setting（受影响者胜 affKey + size 1 非 disabled 回落）、
  ask-for-a-reset 汇流钩（`_mo_notify` childList 触发、批插依序、innerHTML
  容器合流、Blink 重回落）、selected 属性变更步两半、selectedIndex setter
  （可全不选）、select.value setter 双形态真态写（旧 guard 双重排除丢写面
  根因修复）、form.reset 复位算法、select.remove(n)/add 祖先检查 + 元素代理
  add、select value-missing（placeholder 四条件）、Option() 构造器
  （属性只随 defaultSelected + clean 层）；读侧 selectedIndex/value/视图
  全改 JS 真态，pristine-only size 1 跳 disabled 回落。part59 契约 12 测。
  select-value-selectedOption +8、selected-index 13/13、select-validity
  6/6 等**净 +34P**；forms 3569→3602P、九域 4531→4565P（61.59%），对
  s8/s6 账本 diff 均零回归。zero-engine --lib 3138 全过、make test 全腿绿、
  clippy/fmt 过、零新增依赖。挂账：option.form 1F、text-setter 1F、
  replaceChild sel 形落尾缺口、t8g 墙钟钉 clippy 并行负载边缘越限观察账。
- **2026-10-18 M2 切片 8**：HTMLOptionsCollection/option list 基建
  （R3908/R3908b）——`_zwListOfOptions` 统一构造（嵌套 optgroup 排除、
  select/hr/datalist 整枝）、HTMLOptionsCollection 代理（length/索引
  setter、replace 取先删后插序规避 overlay identity miss）、
  `_zwIsSelectTag` 前缀判定六处、selectedOptions 代理三 trap + 视图归一、
  option.selected dirty selectedness（不反射；宿主激活脚本
  script_gen.rs 分流属性写）。part58 契约（W1–W9）。common 三文件
  15P、selectedOptions 簇 18P、script-mutation 5P 等**净 +54P**；forms
  3515→3569P、九域 4477→4531P（61.13%），对 s7/s6 账本 diff 均零回归。
  zero-engine --lib 3111 全过、make test 全腿绿（browser 腿 :102 复跑，
  s6 同款环境归因）、clippy/fmt 过、零新增依赖。挂账：ask-for-reset 3F、
  selected-index 7F、add/remove 4F、constructor dirtiness 3F、validity 7F、
  通用 replaceChild sel 形 oldChild 落尾缺口（独立排查）。
- **2026-10-18 M2 切片 7**：option 元素文本/反射语义（R3907）——
  `.text` getter「collect text value + strip and collapse」（script
  命名空间排除模型、子列表递归、NBSP 保留）、`.label`/`.value` 无 ns
  存在即采纳 + collapse 回落（R122 实例层判定）、`.text` setter
  textContent 委托；part03 助手 4 个（R3028 latest-wins 先探、64 层
  防环）+ part04 setter 支，part57 `test_option_semantics_r3907` 契约
  （W1–W7；part16 近 2000 行上限新切片文件）。option 族 36→101P
  （+65P），forms 3450→3515P、九域 4409→4477P（60.40%），对 s6 账本
  逐 subtest diff 零回归。zero-engine --lib 3107 全过、clippy/fmt 过、
  零新增依赖。**stash 对照实验**排除 dialog +3P 因果（runner 案级
  realm 身份非确定性，归 wpt-runner 通道挂账；d11 detached-document
  showModal 不抛为 `_zwIsConnected` 预存缺口）。挂账：option 余簇 7F
  异根（selectedness dirtiness 5F、form owner 1F、isConnected 面 1F）、
  HTMLOptionsCollection/option list 基建簇 ~45F+。
- **2026-10-17 M2 切片 6**：文本控件选区语义（R3906）——selection
  setter「set the selection range」算法 + value setter 选区折叠 +
  异步 select 事件（part06 `_zwScheduleSelectEvent`）+ textarea 换行
  归一修正 + attr 回落 latest-wins + defaultValue= 真脏态守卫 +
  type 变态复位 direction 归一（初版裸 'none' 满账本暴露 80F 同名
  回归后修正，作废首轮取账重跑）+ runner variant meta 展开（R329
  机制复用）。part16 `test_textfieldselection_r3906` 契约（W1–W9）。
  textfieldselection 784P/0F、type-change-state 462P/0F；select-event
  0→270P、selection-not-application variant 化 183+79P；forms
  2848→3450P、九域 3807→4409P（4409/7412 = 59.48%，+602P，分母
  +499），对 s5 账本逐 subtest diff 零回归。zero-engine --lib 3100
  全过、clippy/fmt 过、零新增依赖；make test 三个 browser shell
  间歇失败归因为环境面（X :100 catlink 桥死亡 + load 抖动，隔离与
  main CI 双证）。挂账：editing 管线 `_textSelection` 裸 'none'
  观察账；066.html 预存 hang。
- **2026-10-17 M2 切片 5**：select.showPicker() spec 检查序（R3905）——
  part03 方法分发区新支（disabled→InvalidStateError 先于
  being-rendered→NotSupportedError（display:none + cv:hidden 祖先）先于
  NotAllowedError 门、`_zwTransientActive` 同步消费、multiple/size 不抛），
  part16 R3905 契约单测（9 场景 + 抛错面激活不消费钉针）。show-picker
  六文件 0→7P、customizable-select/nested-options 联带 +1P，forms
  2840→2848P、九域 3799→3807P（零回归；分母 6913 不变）。
  `make test` 全绿、clippy/fmt 过、零新增依赖。教训：corpus 全量轮
  禁与 make test/clippy 并行（首轮负载噪声作废）。
- **2026-10-17 M2 切片 4**：valueAsNumber/valueAsDate setter 抛错面
  （R3904）——part04 两 setter 按 spec setter steps 重排（valueAsNumber
  ±Infinity→TypeError 先于适用性、不适用→InvalidStateError、NaN→''；
  valueAsDate WebIDL `Date?` 转换先行（undefined→null）、不适用
  （含 datetime-local）→InvalidStateError、null/undefined/Invalid Date
  →清空），part16 R3904 契约单测（步序钉针 + 三清空面）。typeerror
  两文件 8F + datetime-local-valueasdate 1F 翻绿、同族 invalidstateerr
  两文件联带 +2P，九域 3788→3799P（零回归；分母 6913 不变）。
  `make test` 全绿、guarded-clippy/fmt 过、零新增依赖。
- **2026-10-16 M2 切片 3**：input.showPicker() spec 检查序 + 瞬态激活消费
  （R3903）——part03 方法分发区新支（不可变检查先于激活门/消费、readonly
  适用 12 类型表、NotAllowedError 门、`_zwTransientActive` 同步消费、无
  focus），latest-wins 属性读三级回落，part16 契约单测。show-picker 三文件
  0→129P（user-gesture 63 / disabled-readonly 44 / does-not-focus 22），
  forms 2697→2829P、九域 3626→3788P（零回归；分母 6656→6913 系 img
  naturalWidth 文件完整上报）。`make test` 全绿、guarded-clippy/fmt 过
  （fmt 既有违规系并行会话遗留，见切片账本）、零新增依赖。
- **2026-10-16 M2 切片 2c**：stepUp/stepDown 全七类型 spec 步进算法 +
  valueAsNumber/valueAsDate IDL 全类型面（R3902c）——part03 `_parseHtmlDateValue`
  date/week/datetime-local 回读校验 + `_formatHtmlDateValue` week/
  datetime-local 新支 + stepUp/stepDown 分支重写（`_reflectedTypeEnum` 类型门、
  `_suDefaultStep`/`_suScale` 缺省表、`_suParseBound` 类型域解析、`_suKInt`
  相对容差近整判定、步进基与 spec 步 3/4/5/7/8/9/10；sel 路径 min/step 走
  `__zw_get_attr_lw` latest-wins 读）+ valueAsNumber getter 全类型，part04
  valueAsNumber setter date 族/month 扩面，part16 R3317 契约修正（方法恒
  暴露 + InvalidStateError + 离格 snap→6）。stepping 簇 +68P、联带 IDL 面
  +64P，九域 3494→3626P（零回归）。`make test` 全绿、guarded-clippy/fmt 过、
  零新增依赖。
- **2026-10-16 M2 切片 2b**：dirty checkedness/value + files IDL +
  `:active`/`:hover` + compound pending 回落（R3902b）——query.rs
  Active/Hover 识别臂，part01 dirty/files 两 map，part03 files/indeterminate
  `_realTag` 门 + 非 file 恒 null + checked 脏态优先，part04 setAttribute/
  removeAttribute('value') 已脏不覆写 + clone 传播 dirty value/checkedness/
  indeterminate + files setter（FileList 存引用/null no-op/否则 TypeError），
  part05 `.checked=` 置脏 + DataTransfer.files FileList 视图，part06 compound
  pending 回落（`[` 预门）。cloning-steps/untrusted-key-event/files/
  checkable-active-space 四簇 +125P，九域 3369→3494P（零回归）。
  `make test` 全绿、guarded-clippy/fmt 过、零新增依赖。
- **2026-10-16 M2 切片 2a**：focus/focus-visible 面（R3902）——part06
  focus-visible 模态旗标，part04 焦点派发接线 + matches() `:focus`/
  `:focus-visible` 分支（isConnected 连接性判定），part01 rAF 预算
  64→512（promise_test 串行链 double-rAF 断链根因）。focus
  -dynamic-type-change 双文件 0→120P、forms +121P、九域 3248→3369P
  （零回归）。`make test` 全绿、clippy/fmt 过、零新增依赖。
- **2026-10-15 M2 切片 1**：input value IDL 面（R3901）——part01 四值模式/
  净化/类型迁移助手，part03 getter 模式分派，part04 setter 分派 + setAttribute/
  cloneNode 接线，part05 `input.type=` setter，用户编辑通道
  （`__zw_text_input`/`__zw_text_delete`/execCommand/script_gen 三脚本）净化
  绕行旗标。the-input-element 528→951P、forms 1897→2320P、九域全量
  2715→3248P（+533，零回归）。中途 13 回归（textarea 缓存写误删）当日定位
  恢复（经验已沉淀 docs/learnings/bugs/2026-10/）。`make test` 全绿、
  clippy/fmt 过；csv 行回填全量数字。零新增依赖。
- **2026-10-15 M1**：fetch 脚本九域+叶子助资产（jsdelivr CDN 回落先落入
  lib.sh 共享路径）；runner 通道 `testharness-html-semantics`（九域顶层递归
  扫描 + skip 规则双侧同源）；基线 42.63%（2715/6369，1151 案），分域账与
  失败簇归因 evidence 落档；csv 数据行回填。挂账：span-limits /
  downloading-resources 两处挂死排除（FIXME）。`make test` 全绿、
  clippy/fmt 过、零新增依赖。

## 依赖约束备注

form-validation 遗产为 forms 域起点；html-syntax-compat（编号 40，已收口）的
parser 面资产为消费基础。
