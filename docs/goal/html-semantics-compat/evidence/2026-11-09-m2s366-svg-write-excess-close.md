# M2 切片 366：SVG foreign-content write 注入 `</script>` 早闭（143 F→P，域 1F→0F）

**日期**: 2026-11-09
**目标对**: 143/052 流式物化架构改造切片分解之切片 B——143 SVG 外层 script 内嵌套 script `document.write` 注入 `</script></script>`，外层早闭面（got 空/断言错位）
**结果**: **143 F→P**；域 join（A=s365-b 1132 pair 过滤域 vs B=s366-b 限制到公共 1132 键）唯一状态变化 = 143 子测试翻绿，零 P→F，T=5/NotRun=5 名单同构；域 the-script-element **1F→0F**（全域口径残 F 亦 0）。

## 勘根（承 s365 切片分解）

spec 流式：SVG foreign content 的 script 在 **DATA state** 记号化（s106 已探针实证），
嵌套 `<script>` 是真实子元素。143 外层 SVG script 含嵌套 script，后者
`document.write("<script>…#2…</script>" + "</script>")`（源码 `"<"+"script>"` 拼接，
规避字面 `</script`）——第二个 `</script>` 在 spec 流式下闭合**尚开启的外层**，
先于尾文本（L21–23，含故意 `assert_unreached():` 冒号笔误）记号化 → 外层以
**前缀文本**执行。本仓全量前置解析：外层 text content 含尾文本 → SyntaxError 红。
write 面注入的孤立 `</script>` 在 fragment parse 下无开启 script 可配对，对已解析
进 DOM 的外层无作用——缺的正是「write 尾带过剩 `</script>` ⇒ 外层即刻早闭、
取前缀」这一步。

## 施工（2 文件，均非 extract 侧——对 s365 草案的偏离）

s365 挂账草案为「extract 封装 {parent_idx, prefix_source} + 写面扫描 + 交付循环
`__zwParserConsumed` 跳过探测」。实施收敛为 **shim 侧自足**（extract.rs 零改动）：
前缀在 consume 时刻从活 DOM 现算，不跨面传递封装结构。

1. **part06.js write() 惰性安装块**新增两 helper：
   - `__zwDocWriteExcessScriptCloses(_exText)`：对 write 增量 tail 的 **raw string**
     线性扫描（lowercase、`<!--`/`-->` 注释区跳过、`<script`/`</script` 标签边界
     判定——下一字符为 `/`、`>` 或空白），深度计数，返回**过剩闭合数**（缓冲内
     无可配对开启的 `</script>`）。
   - `__zwParserConsumeExcess(_pcN)`：过剩闭合 >0 时，读 `_zwCurrentScriptIdx`
     （闭包变量，dispatch wrapper 置位），沿
     `getElementsByTagName('script')[idx]` **parentNode 链上溯最近 SCRIPT 祖先**
     （s106 重排语义：嵌套子先执行，consumed 记到外层）且其 `__zwSelector` 在
     `__zwParseScriptSelMap` 表内 → 解析期文档序 idx；逐过剩闭合记
     `globalThis.__zwParserConsumed[idx] = prefix.trim()`，prefix = 该外层**首个
     非 Text 子元素之前的 child Text 节点串联**（spec 早闭语义：body 截到前缀；
     空串也记账）。无祖先/无表/无 idx 时 no-op（fail-open，语义门非安全边界）。
   - write() 内 buffer 追加后快照 `var _dwpTailRaw = __zwDocWriteBuffer.slice(_dwpPreLen)`
     （先于 parking/重排，保 raw 序）；RunFace try 块内（`_dwbBody` 分支）消费钩子：
     excess>0 → consume（异常吞没，不阻断 write 主路）。
2. **webview.rs 交付循环**：
   - phase-begin 复位串追加 `globalThis.__zwParserConsumed = null`（持久沙箱跨
     文档存活，与 s365 ceiling/selmap 复位同点，先于 s349 stamp）。
   - code match 前 probe（仅 Inline）：`__zwParserConsumed[script_index]` 为
     string 则以之**替换**待执行源（`consumed_prefix.unwrap_or(c)`）。键一致性：
     consume 按 sel map 记**文档序 idx**，probe 按交付循环 `script_index`
     （同为文档序）。`execute_json` 返回值已经 `value_to_json_string` 编码——
     `null`→serde `None`，prefix 字符串→带引号 JSON→`Some(raw)`，单层编码即可。

## 首版缺陷（JSON 双重编码，solo 跑抓获）

首版 probe 在 JS 侧再包一层 `JSON.stringify(...)`：`execute_json` 已对返回值
`value_to_json_string`，非 consumed 探针返回字符串 `"null"`（serde 反序列化为
`Some("null")`，truthy）→ **每个** classic inline script 的源都被替换成死字面
`"null"` → 全页死 → 143 呈 **Timeout**（testharness completion callback 未回调，
非目标 Fail）。修正：JS 侧传裸条件表达式，单层编码。**教训：`execute_json` 面
上 JS 再 stringify 即双重编码——消费端契约是「返回值已 JSON」**。

## 审计面（与 s365 门/s106 重排/026 写面正交性）

- s106 end-tag 重排 [0,1,2,3,5,4,6]：143 交付序嵌套 #5 先于外层 #4；consume 记
  外层 map idx=4，外层自身交付时 probe 命中换源——执行序无冲突（s365 验证过）。
- s365 ceiling/selmap：probe 仅读 idx 键表，不动门；consumed idx 交付时必然
  ≤ceiling（正在执行）；selmap 键表只读。026 写面：写创建 script 走 RunFace eval
  不经交付循环 probe，无涉；解析表键为 stamp 期文档序，写创建不占号。
- 前缀提取只读 child Text 节点，不改 DOM；consumed 表 phase-begin 清零不跨文档。

## 验证

- shim 拼装语法 node --check PASS（22 part 装配序）。
- solo 143 F→P（s366-solo-143.log，exit 0）。
- solo 哨兵 10/10 Pass（s366-solo-probes.log，exit 0）：137/138/139/140/142/149/
  052/026/041/068——SVG nested 族+write 面+s365 ceiling 面+inline-write 族。
- `cargo fmt --all -- --check` PASS；`make guarded-clippy` 全 workspace
  -D warnings PASS。
- 域 join：A=s365-b（1132 pair，FILTER=the-script-element）vs B=s366-b。**B 面
  调用缺 FILTER 位置参数 → 全语义面 sweep**（1229 文件/8417 pair：
  8078P/285F/43T/11NR）；限制到公共键集（1132，A-only=0）comm -3：
  **唯一变化 = `Fail …/143.html :: scheduler…` → `Pass`**，T=5/NotRun=5 同构，
  零 P→F。the-script-element 残 F=0（全域 grep 亦 0）。全 sweep 285F 分布
  customizable-select 126/dialog 41/select 族/img-density/embed-object 等既有
  未建面，scripting-1 零 F；无本流基线，作为首次全域快照留档备后续切片对照。
- make test：v8 workspace 腿完整 **9326P/7F**（与 s365/s363 基线总数逐名同构，
  零新增红——7 红为 s356-s358 起并行流组合态既有集：
  img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 /
  fetch_filtered_response_type_matrix / iframe_sw_messages_dispatch /
  response_body_readable_stream_r2967 / websocket_error_nonterminal /
  websocket_event_flow）。quickjs 腿未达（make 于 v8 腿既知 7 红 abort，
  s360 EXIT=2 同形态）——shim 为 feature 中性 JS（include_str 同文件），写面
  helper 与宿主 probe 均在 engine 上下文不激活（probe 仅 webview 交付循环），
  v8 腿 3821P 含全量 js_dom_bridge 测试已覆盖。

## 挂账 / 下一步

- SW module 静态 defer 求值语义；142 xlink:href write 注入面；T=5 长尾。
- 全语义面 sweep（8417 pair 快照）自此有档：后续切片可按公共键集法做 A/B。

## 日志

- evidence/logs/s366-solo-143.log / s366-solo-probes.log（solo 面）
- evidence/logs/s366-b-script-element.log（B 面，误全 sweep；join 基线
  s365-b-script-element.log）
