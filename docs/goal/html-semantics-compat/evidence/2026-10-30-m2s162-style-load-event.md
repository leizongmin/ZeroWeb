# M2 切片 162：style 元素 load 事件面三触发臂（update a style block 派发半）

- **日期**：2026-10-30
- **切片**：html-semantics M2 s162（候选③ style_load_event 入场，勘根扩面收口）
- **基线**：s161 终账 full4 `7093P/803F/91T`（(key,state) 对去重 8005 对 = 7090P/803F/91T/21NR）
- **改动**：shim 2 文件净 +100（part03.js +70 / part04.js +30），rust 零改动

## 入场与刀位

s162 候选③ `style_load_event.html`（1F）。勘根发现该面整体缺位，非单案：

- `style_load_event.html`：初始段 `loadCount 0≠1`（解析静态首派缺失）+ textContent 变更段 await listener 永挂（变更派发缺失）。
- `style-load-after-mutate.html`：createElement style → appendChild → 文本 → `await style.onload` 永挂 → 文件级 Timeout（动态插入臂缺失）。
- `style_events.html` / `style_load_async.html`：同族文件级 Timeout 形态。

Spec 语义（[update a style block](https://html.spec.whatwg.org/multipage/semantics.html#update-a-style-block)）：style sheet 就绪即向元素 fire load；inline 文本同步就绪，但更新走「queue a task」——task 异步使同 turn 后挂的 `onload`/`addEventListener` 可收。type eligible 门（s156/s160 精确匹配口径：type 存在且非空且非 ASCII ci `text/css` → 不创建 sheet → 不派）。连接门：更新仅在 shadow-including connected 元素运行。

## 施工（2 文件 +100，rust 零改动）

- **part03 共享 helper `_zwStyleBlockQueueUpdate(sel, handle, first)`**（+55）：三触发臂共享队列——identity key 双印记去重（首派臂 `_zwStyleLoadFired` 永久印记防多尾扫重复派；变更臂 `_zwStyleUpdatePending` 同 tick 去重 = spec 单 task 更新语义）+ `setTimeout(0)` task 尾派发 + fire 点现读 type 门与连接门（`_makeProxy().isConnected !== true` 早退）+ `_dispatchWithBubble` 元素站派发（`bubbles:false`）。
- **part03 解析静态臂**（+15）：`_zwStylesheetStaticScanUpto` 越界（尾扫）形对快照 style 元素全量首派——`_ord >= _zwScanScriptPrefix.length` 即脚本流末尾语义，零 script 页面首次调用即末尾；与 link 前缀游标无关。
- **part04 appendChild trap 双臂**（+26）：STYLE 父臂（子文本入 style）+ STYLE 子臂（style 本身插入文档，取 `child.__zwSelector || child.__zwHandle`）；handle-only 形经 `_zwHandleCreateTag` 即创建盖章链无漏（勘根确认）。
- **part04 textContent setter 臂**（+16）：写提交后入队（`_realTag === 'STYLE'` 门）。
- FIXME 注记：type/title 属性变更触发面、非 textContent 子变更（insertBefore/append() 入 style、CharacterData 写）未挂臂——语料无形态。

## 验证

- shim 拼接 `node --check` SYNTAX_OK。
- 定向：`style_load_event.html` **Fail→Pass**（初始 loadCount=1 + textContent 变更段收 listener）；`style-load-after-mutate.html` **文件级 Timeout→Pass**（`await style.onload` 收）。
- document-metadata 目录 join 对 s161 full4：**F2P 1 / P2F 0**，3 个新现子测全 Pass（style-load-after-mutate、style_events「If the style is loaded successfully…」、style_load_async「style load should be async」），2 文件级 Timeout→Pass；目录 `181P/31F/12T → 184P/30F/10T`；CSP 文件级 Timeout 为基线既有非回归。
- 全量 join（full1 对 full4，(key,state) 对）：**F2P 1（靶案 style_load_event textContent 段）/ P2F 0 / P2T 0 / T↔P 0**。LOST 77 全归因本轮环境截断与探针删除——71 forms + 2 interactive-elements + 1 scripting-1（select-event/customizable-select 交互时序域先例形态）+ `zz-s161-probe.html`（s161 探针已从树中删除，键自然消失）；其中 4 LOST Fail + 3 LOST Timeout 同为截断未报。GAINED 6：4 Pass（3 靶族 = 2 文件级 Timeout 恢复首报 + style_events 新现子测；1 focus-dynamic-type-change 环境恢复）+ 1 Fail + 1 Timeout（customizable-select optional / details name-attribute——在册 flaky 域键，基线被截断本轮报出）。raw 计数 `7090P/803F/91T/21NR → 7025P/799F/89T/21NR`（对去重键数）。
- 门禁决策（shim-only 轮惯例，s155/s156/s159 先例）：rust 零改动 → fmt/clippy 面零波及；本刀提交树态（HEAD + 我方两 shim hunk）与全量 run 编译树态逐字节同构（run 于 23:31 启动于我方最后编辑后、并发方 23:37 起才动树）——full1 7934 键零 P2F 即提交树的端到端行为门禁，强于补跑单测。共享树 make test 未跑通：T58 fmt 前置门在并发方 testharness.rs WIP（stylesheet.py 端点，rustfmt 违例）失败——非本刀文件不代修，其收口时自会过门。

## 归因与并发注记

- 本轮证据树态纯净：全量 run 于 23:31 启动（我方 shim 最后编辑后），并发会话的 css_wire.rs/testharness.rs（23:37）与 part03 per-sheet 枚举 hunk（23:44）均晚于 run 编译点，run 二进制只含本刀 hunks。
- **切片号撞车**：并发会话同号施工中（part03 `_zwSsExtSheetEntries` per-sheet 枚举口 + link media 不匹配推迟取回发起 hunk + css_wire/testharness WIP，注释亦标 s162，未落账）——本刀先落账保号 s162；其收口时应顺延重编号 s163 并同步代码注释。
- 本刀提交采用索引级 hunk 分割（`git apply --cached` 仅 stage 我方两 hunk + part04 全量），并发方 WIP 留在工作树零触碰。

## 挂账更新

- FIXME 新记：①type/title 属性变更触发面未建模；②非 textContent 子变更（insertBefore/append 入 style、CharacterData 写）未挂臂——语料无形态，暂不扩面。
- 承袭不变：动态 link query_all 不可见（并发方施工面）、外链槽位级联序、meta color-scheme 三重栈、selection-pointer 持久漂移、link-multiple 贴边、交互时序域全量 Timeout 环境截断等。

## 下一步 s163 候选

1. 并发会话施工面收口核对（per-sheet styleSheets 枚举 + media 推迟取回——落账时重编号 s163；含 stylesheet-media-change 断言 2 + media-match-block-script 设计面）。
2. meta color-scheme 三重栈续拆（used-scheme 传播 + 系统色动态解析）。
3. style 元素 load 面 FIXME 收尾（type/title 属性变更臂——语料无形态，低优先）。
4. shadow DOM 级联面（link-rel-attribute shadow 案）。
