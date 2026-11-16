# d46：TypingA 替换臂——非塌缩选区键入先删后插 + gtr 端点直取空白门

日期：2026-11-16。前置：master.md 下一步①(b) TypingA 替换臂 69F
（insertText 非塌缩选区先删后插缺失，`<p>{abc}</p>`+'a' 期望 `<p>a</p>` got
`<p>aabc</p>`）。本刀复用 d43 `_zwCeSelectDelete` 摘除核（新增第 4 参
typingData），摘除+join+插入单 beforeinput 承载；跨三个探测循环定位两处
shim 陷阱后收敛。探针/运行产物均在 /tmp（gitignored）。

## 落码（part01.js，+115/−9，唯一产品变更）

1. **`__zw_ce_insert` 替换拦截**：非塌缩选区 + 非 `\n` 键入改道
   `_zwCeSelectDelete(el, range, 'insertText', {data})`（真值返回即收束——
   beforeinput/摘除/join/插入/input 全在核内单事件承载）；拒收形回落塌缩
   插入旧径；'\n' 让位 insertParagraph 分流臂。
2. **`_zwCeSelectDelete` 第 4 参 typingData + `_sdFinish` 键入尾**：beforeinput
   携 data；键入尾 = 删除点插 data + caret 落插入段尾 + input(insertText)
   收束。元素锚插入位两侧**非空活文本优先拼接**（nodeValue splice 既有
   wire）；占位 br 门控 `if (!typingData)`（键入替换不建占位——插入随即填块）。
   join 臂传 `silent` 让位 caret 写回/input 派发，返回 `{node, offset}`
   （join 点首壳前）供键入尾消费。
3. **`_zwCeCanonGtrStart/End` 直取臂 + 空白门**：元素锚 child 即非空白文本时
   gtr 端点直取（(p,0) → (abc,0)，`<p>{abc}</p>` 语料定谳）；**纯空白文本子
   除外**（`div{  <ol>…  }` 装饰空白保持元素锚 (div,0)-(div,3)，d45b P 案
   回红对拍实证）。

## 探测定谳（三循环）

- **M1（zz-probe-d46a 微测）**：shim insertBefore trap 对**文本参考子失明**——
  `insertBefore(X, child[1])` 得 `abefX`（尾落）。中位元素插入换 nodeValue
  splice wire（d43 摘除截断同通道）。
- **ghost 分裂（ZW_SHAPED_ADVANCE_TRACE=1 console 通道，非塌缩全文件插桩）**：
  语料 initializeTest 的 innerHTML 重置在 shim/engine 间留**分裂投影**——摘除
  后 `p.childNodes` 含 parentNode 仍指 p 的空文本 ghost（trace：`<p>{<img>}</p>`
  摘除时 tks=[img, t'', t''] 而引擎视图空）。空 ghost 拼接的 nodeValue wire 在
  drain 静默丢弃 → 键入三案 `<p></p>` 回红。判据 = **锚数据非空**（parentNode
  检查不滤 ghost，空串任何视图比对恒真）；空锚落 insertBefore/append trap
  回落（trap 对文本参考子失明 → engine 侧 append 兜底，v1 旧径对该形实证落树）。
- **bi=2 双派**（zz-probe-d46b 交付对象身份比对）：同 beforeinput 对象两次
  交付（sameObj=true）——d41/d42 已记档的 sel 方言 cache 双键/身份 churn 病，
  **前置存在非本刀引入**（d45 同案异形红）；d42 A/B 判决归一刀修复净 −27，
  持位不在本刀。

## A/B 轨迹（基线 = d45b 终态二进制 1099P/2229F/3T，zz-probe 排除口径）

| 二进制 | 构成 | 非塌缩?TypingA | 全语料 | 判决 |
|---|---|---|---|---|
| v1（nc2） | 拦截+核+旧径收尾 | 19P | — | `{abc}` 系 +4 定谳核心可达 |
| v2（nc3/nc4） | + 两侧拼接（活锚守卫 v1 版） | 23P（**P2F=3**） | — | img 三案 ghost 丢失 |
| v3（nc5） | + 锚数据非空守卫 | **26P，P2F=0** | — | TypingA 面 +7 净收敛 |
| v4 全语料 | v3 二进制 | 26P | 1131P（**P2F=4**） | `div{  <ol>…  }` gtr 空白装饰形回红 |
| v5（singleChild 限制） | 直取限同容器单子 | 26P | 1098P（**P2F=37**） | joining-dl 跨元素直取需求实证，**弃** |
| **v6（空白门，终态）** | 直取门=非空白文本 | 26P | **1135P，F2P=36/P2F=0** | ✅ |

v5→v6 判据定谳：直取门按**子文本是否纯空白**（`/^\s*$/`）而非选区形状——
deleting-in-list `div{…}` 空白装饰子保持元素锚、joining-dl `<dd>[list-item1`
(dd,0)→(text,0) 跨元素直取保留，两族并存与 d44 克隆/持活分流同构（期望表
异相族不可一刀切）。

逐文件 F2P（+36 全落 gtr/替换簇，零面回退）：

| 文件 | F2P |
|---|---|
| non-collapsed?TypingA | +26（69F→43F；`{abc}`/`{abc}<br>`/img 三案/`{ abc }`/跨 p join 系/caret host/`abc[<img>]`/`abc{<br>}`/li 系 gtr 面） |
| non-collapsed?Backspace/Delete | +3/+3（`{abc}` 系 gtr 直取面） |
| deleting-in-list-items ×4 | +1×4（`<li>{list-item1}</li>` 单子全选 gtr 直取 (li,0)→(text,0)） |

## 哨兵（v6 终态二进制 /tmp/d46-tgt/release/zero-wpt-runner）

- html-semantics 全语料（950 文件）：`/tmp/d46-sem.json` **P 恒等 5378**，
  F2P=0/P2F=0（F/T/NotRun 计数同 d45 窗）✅
- selection 全语料（738 文件，probe 排除）：`/tmp/d46-sel.json` 48,914P →
  **48,942P 净 +27，F2P=32/P2F=4**——F2P：dwuf Typing 面 +24
  （delete-without-unwrapping-first-line-of-child-block 八变体的 Typing 变体，
  master 挂账「dwuf +22 名义红」首次成批翻绿——替换臂+gtr 直取覆盖 Typing
  形）、typing-around-link **Direct typing** 变体 +4（非塌缩键入走替换核，
  `[z]abc`+键入期望 `XYabc` 内链接形）、insertHTML white-space 四域 +4。
  **P2F=4**：typing-around-link Backspace/Delete/execCommand(delete/fd) 变体
  （`Inserting "XY" after deleting first character of a link`）——两阶段流
  （phase-1 键删 z + phase-2 键入 XY）got `<p><a>abc</a></p>` XY 丢失。phase-1
  臂（arm ② 文内 splice）与 phase-2 代码 d45/d46 逐字节同，直接键入变体改走
  替换核后同文档残留态演变分岔——**initializeTest 分裂投影 chaos 族**（img
  三案同根，d42 A/B 前例：追身份残留净负），接受记档不追。

## 守卫与门禁

- 全部运行：`./target/test-guard --per-proc-mem 4 --total-mem 8
  --time-limit {600,1500,2700} -- <runner> testharness-{input-events,
  html-semantics,selection} --json`；runner 构建
  `systemd-run --user --scope -p MemoryMax=24G -p MemorySwapMax=0`。
- 零 Rust 变更（仅 part01.js，fmt/clippy 不消费）；探针 console 通道
  `ZW_SHAPED_ADVANCE_TRACE=1`（runner 仅该 env 下 init tracing subscriber，
  console.log → tracing::info → stdout——本 session 记档）。

## 遗留（持位）

- TypingA 残余 43F：bi=2 身份双派族（d42 A/B 挂起）+ `{<hr>}` 纯元素选区
  核拒收形（bi=0）+ `<ul><li>def</li><li>]ghi` TypeError null.firstChild
  （语料级状态依赖，probe 单测不可复现）+ table 跨 cell join bi=3 族。
- 非塌缩 Backspace/Delete 44F：跨元素 caret join（同 d44 遗留 backspace 主
  文件 243F 族）+ 嵌套深锚 FIXME(nested-list-deep)。
- selection 4 P2F（typing-around-link 键删+键入两阶段变体）：分裂投影 chaos
  族，与 generation-aware 归一刀（master 下一步③）同根，不单独追。
- C3 空壳剪枝/C4 行捐赠/C5 空白边界：Blink 深语义（probe 定谳期望表与
  Chrome 行为分离，暂不追）。
- initializeTest 分裂投影病（ghost/双派同根）：generation-aware 归一刀
  （master 下一步③挂起中）——本刀非空锚守卫是该病的键入面局部免疫。
