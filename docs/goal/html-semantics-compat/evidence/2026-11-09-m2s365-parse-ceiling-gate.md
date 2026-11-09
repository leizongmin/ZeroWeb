# M2 切片 365：script 枚举交付水位线门（052 F→P，域 2F→1F）

**日期**: 2026-11-09
**目标对**: 143/052 流式物化架构改造切片分解之切片 A——052 外链脚本执行时 `getElementsByTagName('script')` 超前物化（got 5 expected 4）
**结果**: **052 F→P**；域 join 唯一状态变化 = 052 子测试翻绿，零 P→F，T=5/NotRun=5 名单同构；残 F 仅 143（切片 B/s366 范围）。

## 勘根（承 s364 重勘）

spec 流式解析：parser-inserted 阻塞脚本于自身 end tag 执行，tokenizer 停在该点，
其后标记尚未物化进 DOM（#script-parser-executing / #prepare-the-script-element）。
本仓为全量前置解析 + 批量交付架构：052 外链脚本（文档序 #3）执行时内联 #2（#4）
已在 DOM。**不经 parser 核心改造的等效面 = 枚举水位线**：交付循环逐阻塞脚本前推
`__zwParseCeiling`（= 当前脚本文档序号），script 标签枚举面裁到已交付前缀。

**切片分解**（Mission 约束 #3 轻量优先，避免 parser 核心重写）：
- 切片 A / 本轮 = 052 枚举门（getElementsByTagName/NS + document.scripts）。
- 切片 B / s366 = 143 write 注入 `</script>` 早闭面（extract 侧封装 + 写面扫描，设计已勘定）。

## 施工（3 文件）

1. **webview.rs**（宿主侧，交付循环）：
   - phase-begin 复位 `__zwParseCeiling=-1; __zwParseScriptSelMap=null`（持久沙箱
     跨文档存活，R5182-D1 同源）——**必须先于 s349 stamp 调用**（首版顺序颠倒把
     stamp 新建的键表清回 null，门 fail-open 退化为全量视图，solo 052 仍红 5；
     修正后过）。
   - 循环头逐阻塞脚本 `__zwParseCeiling = max(ceil, script_index)`（shim 侧单调
     ——s106 end-tag 序执行时祖先脚本文档序小于已执行子脚本，ceiling 不回退）；
     进入 deferred 尾与循环毕清 -1（the-end 相位全文档可见；纯阻塞页由循环毕
     清水恢复全量，后续排水/事件派发不受门限）。
2. **part06.js `__zw_stamp_parser_scripts`**：宿主 stamp 入口（阶段起点，ceiling
   未激活=全量视图、写创建脚本尚不存在）枚举全体 script 建 **sel→文档序号表
   `__zwParseScriptSelMap`**（键 = 解析期宿主身份 `__zwSelector`，跨包装实例稳定）。
3. **part05.js `_zwGateDeliveredScripts`**：script 枚举门，挂
   `_zwFilterByTagNameNS` 返回口（doc 级 getElementsByTagName/NS、元素级子树枚举
   `_zwGetByTagNameSubtree`、live collection 谓词 `_zwLiveMatchesFor` 单点汇集）+
   **part06.js `_zwDocHtmlNsColl`**（document.scripts qSA 快照闭包，缓存键加
   ceiling 维度——ceiling 变化不改宿主视图戳）。判据三分：
   - 无 sel 或 sel 不在表内 → **放行**（动态产物即时可见；write 面回放元素经
     fragment parse 同样带 fragment 域 sel，但不在解析表内——026 面）；
   - 表内序号 ≤ ceiling → 已交付放行；
   - 表内序号 > ceiling → 解析产物未交付，**裁剪**（052 面）。

## 首版设计缺陷（域 join 抓获，域跑回归价值实证）

首版判据「有 sel 即查门、前缀集外即裁」：026.html P→F——write 创建的 script 带
fragment 域 sel，不在前缀集 → 被裁 → `scripts[scripts.length-1]` 命中 inline#1、
src 设错对象，external script 缺席（got 长度 3）。改三分判据（表外 sel 放行）
后 026 复绿。**教训：sel 有无 ≠ 解析/动态判别**；动态产物可携带宿主身份（fragment
parse map 分配），判别必须以「阶段起点解析表」为准。

## 审计面（门兼容性）

- s95 currentScript（自索引 ≤ ceiling ✓）；s349 stamp（激活前建表 ✓）；
  part06 派发面（current-script 事件枚举 idx==ceiling ✓）；s152 interleave 扫描
  与写面体遍历（不过滤核 ✓）；testharness get_script_url（index 0 恒已交付 ✓）。
- 不门近似（语料无钉面，在册）：getElementById/querySelector 选择器面、树遍历、
  `'*'` 快道。门缺失键表时 fail-open（语义保真门，非安全边界）。

## 验证

- shim 拼装语法 node --check（part01..06 装配序）PASS。
- solo 052 F→P（s365-solo-052.log）；solo 026 复绿（s365-solo-026.log）。
- 域 join（A=s364-b 1132 pair vs B=s365-b 1132 pair，LC_ALL=C 全行 (state,key)
  sort -u）：A-only={Fail 052}、B-only={Pass 052}、其余零位移；残 F=143、T=5、
  NotRun=5 名单与 s364 同构。域 the-script-element **2F→1F**。
- `cargo fmt --all -- --check` PASS；`make guarded-clippy` 全 workspace
  -D warnings PASS（47.5s）。
- make test：v8 workspace 腿完整 **9326P/7F**（与 s363 基线总数逐名同构，零新增红
  ——7 红为 s356-s358 起并行流组合态既有集：img_runtime_deferred_settle_r5275 /
  svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_sw_messages_dispatch / response_body_readable_stream_r2967 /
  websocket_error_nonterminal / websocket_event_flow）。本轮独立 stash A/B 双跑
  同红双树逐名同构（s329/s352 同法第三度实锤非本流引入）；quickjs 腿未达（make
  于 v8 腿既知 7 红 abort，s360 EXIT=2 同形态）——shim 为 feature 中性 JS
  （include_str 同文件），门在 engine 上下文恒不激活（ceiling 仅 webview 交付
  循环置位），v8 腿 3821P 含全量 js_dom_bridge 测试已覆盖。

## 挂账 / 下一步

- 切片 B / s366：143 SVG foreign-content write 注入 `</script>` 早闭面
  （extract 封装 {parent_idx, prefix_source} + 写面 raw-string script 平衡扫描 +
  交付循环 `__zwParserConsumed` 跳过探测；执行序 [#1,#2,#3] 对 s106 重排
  [0,1,2,3,5,4,6] 验证过无冲突）。
- SW module 静态 defer 求值语义；142 xlink:href write 注入面；T=5 长尾。

## 日志

- evidence/logs/s365-solo-052.log / s365-solo-026.log（solo 面）
- evidence/logs/s365-b-script-element.log（域 B 面，join 基线 s364-b-script-element.log）
