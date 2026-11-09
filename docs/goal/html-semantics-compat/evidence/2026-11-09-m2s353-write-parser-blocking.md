# M2 切片 353：写流解析阻塞停车——execution-timing 068 F→P

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/execution-timing/068.html`（s352 基线 14F 中单 F，同 chunk script 后 markup 提前物化）
**结果**: 068 F→P；域 join det F→P=1 / P→F=∅ / new-F=∅ / removed-F=∅（同翻转）/ T=8 同集双向 ∅ / 双态键 0；语料口径 **14F→13F，1108P→1109P**（distinct-pair，1130 键两侧）

## 上游语义（勘根）

[HTML pending parsing-blocking script](https://html.spec.whatwg.org/multipage/scripting.html#pending-parsing-blocking-script)：parser-inserted src 外链（无 defer/async）准备期间 **parser 停在该元素，同 chunk 其后标记不解析**——script 求值完成后 parser 恢复。068：`document.write('<script src=find-foo.js>…</script>' + '<div id=foo></div>')`，find-foo.js 断言 `getElementById('foo')` 为 **null**（div 须在 script 求值后才物化）。引擎旧模型 `body.innerHTML = base` + `insertAdjacentHTML('beforeend', 全量缓冲)` 一次物化 → div 先于 script 求值存在 → `YES` 红态（evidence/logs/s353-solo-068-red.log：`expected "found #foo element: NO" but got "YES"`）。

## 施工（engine shim 单面，part06.js）

写流「已物化前缀」模型：`__zwDocWriteVisLen` = 缓冲已物化前缀长度；停车段在主链排水后 walk-end 续流物化。

1. **阻塞扫描器** `__zwDocWriteFindBlockingClose(text, from)`：缓冲内首个 src 阻塞外链（无 defer/async）闭合标签后偏移。近似解析器——注释区跳过；script 文本整体跳至闭合（写入串按惯例 `\/` 转义闭合，语料零违反）；引号属性值内引号配对跳过。FIXME(spec-gap)：属性值内 `<script` 字面、未转义闭合的病态形状不作 token 级识别（语料 0 形；误检劣化为「延后物化」自愈于排水续流）。
2. **write 停车决策**（隐式流专用，显式 open/close 早退不触）三支真值表：
   - **挂链 fetch 在途、无求值窗 → 尾停**：visLen 不动，写入随停车段后流（009 多写序：页面顺序写的 include-2 不得越过先写的 include-3——流位置在全部既有写入之后）；
   - **求值窗内嵌套写入 + 停车段在途 → 插入点重排**：执行链脚本的写入紧贴 visLen，既有停车段后移——保持「已物化槽位 = 最终槽位列表前缀」不变式（槽位号按物化序分配，停车段物化时新槽位恒尾插）；
   - **空闲 / 求值窗且停车段空 → 扫描定界**：本次写入含阻塞外链 → 边界 = 其闭合后；无 → 全量放行（既有口径）。
3. **重放切片**：`insertAdjacentHTML('beforeend', buffer.slice(0, visLen))`（越界夹挤防御）。
4. **walk-end 续流** `__zwDocWriteVisExtend()`：非 throwaway 面走查完毕且挂链/嵌套全排空时，从 visLen 扫下一阻塞外链、物化 `[visLen, 新边界)` 段（有阻塞则新边界 = 其闭合后，其后继续停车）→ 续跑面。水位线先行落定（续流面从旧物化数续走新物化槽位）。段耗账推进不受解析异常回退（保活性优先）。
5. **复位三处**（open/首写/close）：visLen 归零；首写 init 先行于停车决策（决策后置会被 init 覆写——验证轮实证后重排）。

## 设计不变式与中间缺陷（验证轮排除）

- **插入点重排仅限求值窗**：首轮实现把「停车段在途」的所有写入一律重排 → 009/010/011/012 四案逆序红（页面顺序写的 include-2 被挪到先写的 include-3 前，域跑 `expected "external script before doc write" but got "external script #2"`）→ 分支加 `!_dwpEval` 门，四案转绿。插入点语义只对执行链脚本成立。
- **首写 init 与停车决策顺序**：首轮决策在前、首写 visLen=0 覆写在后 → 首写停车失效（靠 walk-end 续流自愈，功能对但多一跳+注释失真）→ init 块移至决策前。
- 复用 s352 单一 `__zwDocWritePendingSlot` 串行器与 copy-based 执行链（副本脱离树，全量重放破坏不了在途 fetch）。

## Blast radius 普查

- 行为变化仅「src 阻塞外链 + 其后标记同 chunk」形；全语料该形 13 文件（execution-timing 11 + history/008 + resource-timing/initiator-type）。
- 形状普查逐族定界：src-only 写（007/008/009-012/041 等）边界=写尾零差异；defer（096）/async（097）扫描器排除；xlink:href（142）无 src 不阻塞；纯内联写（005/006/026/027/141/143/149）不停车；explicit 流（102）不经停车路径；link/iframe 写（107/108）无 script。
- 外溢双点实证：resource-timing/initiator-type-for-script solo exit 0；history/008 Timeout 与基线同态（跨域 script 不可达既有面）。

## 验证

- **红态**：修前 solo 068 `expected "found #foo element: NO" but got "YES"`（evidence/logs/s353-solo-068-red.log）。
- **solo 六哨全 Pass exit 0**（终版 binary：009/010/011/012 回归哨 + 068 目标 + 041 嵌套链哨；evidence/logs/s353-solo-probes.log）。041 全 11 元素序手工 trace 与期望吻合（嵌套 write 经插入点重排 + 排水续流落位）。
- **域 join**（LC_ALL=C + `(key,state)` sort -u + 双态键 0）：A=s352-b **1108P/14F/8T** vs B=s353-b **1109P/13F/8T**——det F→P=1 恰 068；new-F/P→F=∅；T=8 同集双向 ∅。
- **shim 拼装语法**：21 part 构建序拼接 `node --check` SYNTAX-OK。
- **clippy** zero-engine `--all-targets -D warnings` 净 + `cargo fmt --all -- --check` 净（纯 JS shim，无 Rust 改动双证兜底）。

## 残余 13F（B 基线在册）

execution-timing 056/063/143/probe12-defer-slow（自建 probe 单列口径）、fetch-src/alpha/base、json-module/parse-error-location.tentative、module/custom-element-exception、module/dynamic-import/{basic-deferred-evaluation,cyclic-tla-defer-eval,tla-deferred-evaluation}.tentative（import.defer JS 语言特性面独立切片）、module/probe13-async-srcfast（自建 probe 单列口径）、remove-src-attr-prepare-a-script、script-not-found-not-executed。T=8 长尾在册。

## 挂账 / 下一步

- execution-timing 056 mutation 触发 script prepare 面（空 `<script></script>` + appendChild 文本同步执行）——下一候选勘根。
- 143 upstream 真值复核（s352 挂账延续：wpt.fyi 四次取空超时）。
- 063 cloneNode 执行中脚本、fetch-src/alpha/base（src 解析 base URL 面）、json-module parse-error-location、custom-element-exception、remove-src-attr、script-not-found 逐案勘根；import.defer 三案独立切片。
- 停车面 FIXME 两项（token 级病态形状、跨开放元素边界切片树形）语料 0 形在册。

## 日志

- evidence/logs/s353-solo-068-red.log（修前红态）
- evidence/logs/s353-solo-probes.log（六哨 solo 全 Pass）
- evidence/logs/s353-b-script-element.log（域跑 B 面）
- s352 基线 A 面：evidence/logs/s352-b-script-element.log（既有）
