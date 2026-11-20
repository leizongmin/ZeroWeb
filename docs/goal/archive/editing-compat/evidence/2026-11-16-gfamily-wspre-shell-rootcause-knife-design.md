# G 族 ws-pre 壳 caret 载荷丢失根因 + 刀设计（读侧分析轮，2026-11-16）

普查档（2026-11-16-ihc1-residual-49f-census.md）G 族 ×10 的根因定谳与下一刀设计。纯读侧分析，零码面改动；ihd1 在飞窗内落档，供刀后实施轮取用。

## 根因链（四步全实证）

语料形（editing/data/inserthtml.js :618-664）：`<div style=white-space:pre>abc[]\n</div>` + `inserthtml "X"` → exp `abcX\n`，got `abc\n`（载荷整丢、结构不变）。

1. **宿主非 fixture 根**：`runConformanceTest`（include/tests.js :5556）先设 `test-container.innerHTML = "<div contenteditable></div>"`，fixture 作为该包装的 innerHTML——editing host = 空包装，fixture 根 `<div style=white-space:pre>` 是 host 直子壳，caret 文本节点在壳内。
2. **路由进壳域解析**：`sc.parentNode !== host`（part06.js :3365）→ `_zwInsertHtmlShellResolve`。反证：`foo[]bar` 形（裸文本 caret，parentNode===host）不进解析臂，全绿。
3. **div ∉ SPLICE**：拼接壳白名单仅 xmp/script/plaintext/listing/style/textarea/pre（:3600），`div` 落入 styled 行内壳溶解臂。
4. **坍缩选区恒不合形**：溶解臂要求元素壳 span 全含于选区 `[ws,we)`（:3638 `ws <= sp.s && sp.e <= we`），collapsed range（ws==we）对非空壳恒 false → `return null`（:3640）→ 内联臂 `if (!sr) return`（:3367）→ **整命令 no-op**。八形 got 全部为结构不变，精确吻合。

块臂不涉：payload `X` 无 `<`，:3923 即旁路。轴二元素端点臂不涉：sc 是文本节点非元素。

## 语料目标语义（10 键，含双插）

| 形 | ws | 命令 | exp | 备注 |
|---|---|---|---|---|
| `abc[]\n` | pre / pre-line | X；X,Y | `abcX\n`；`abcXY\n` | 换行保留（corpus 注 1：preformatted linefeed 不删） |
| `abc []\n` | pre | X；X,Y | `abc X\n`；`abc XY\n` | 空格在 pre 下可见，原位拼接 |
| `abc []\n` | pre-line | X；X,Y | in_array [`abcX\n`, `abcX \n`]；[`abcXY\n`, `abcXY \n`] | 空格在 pre-line 下隐形，允许删或回退（corpus 注 2） |
| `[]\n` | pre / pre-line | X | `X`（无尾换行） | 全隐形前缀：块尾隐形空白被插入消费 |

## 刀设计（part06 `_zwInsertHtmlShellResolve` 增 ws-pre 壳臂）

位置：SPLICE 臂之后、溶解臂之前（:3629/:3630 之间）新分支。门（全合取，保守贴语料形）：

- `range.collapsed`（G 族全坍缩形；非坍缩维持溶解臂既有域）；
- `shellTag === 'div'` 且 shell 是 host 唯一子（kids.length===1，八形实录同构）；
- shell 恰一子且为文本节点；
- shell style white-space ∈ {pre, pre-line}（getAttribute('style') 串解析）。

产物（复用 `_zwIhtmlTopSpans` 取原 open/close 标签字节，绕开裸标签重建 round-trip 失真）：

- `rebuilt = html.slice(0, tSpans[0].innerStart) + newContent + html.slice(tSpans[0].innerEnd)`，round-trip 门＝`html.slice(innerStart, innerEnd) === shell.innerHTML`；
- 返回 `{html: rebuilt, want: 壳基 + 载荷落位}`，走内联臂既有 caret remap（shellMode 路径 wantStart+fragTextLen 实证可达，双插 seq 推演闭合）。

newContent 三规则（按序判）：

1. **全隐形前缀**：caret 前缀无可见字符 → `newContent = frag`（`[]\n`→`X`，双 ws 值同）；
2. **pre-line 前置不可见空白回退**：caret 前紧邻 `[ \t]+` 且该 run 至行尾（下一个 `\n` 或串尾）前无可见字符 → 落位回退到 run 起点，拼接（`abc []\n` pre-line → `abcX \n`，在 in_array 内）；
3. **pre 保持拼接**：其余原位拼接（`abc[]\n`→`abcX\n`；`abc []\n` pre → `abc X\n`）。

双插 seq 推演（第一插后 caret remap 落位 → 第二插门重判）：pre `abc[]`→`abcXY\n` ✓、pre `abc []`→`abc XY\n` ✓、pre-line `abc []`→`abcXY \n`（在 in_array 内）✓。

旁路维持：非 sole-child / 多子壳 / 非 div / 非 ws-pre 族 / 非坍缩 → return null（既有 no-op 不变，零外溢）。预期门禁：iht 双腿 F2P=10 恰 G 族、零新增红（49→31F）；part124 新增 G 形钉 8-10 枚锁语料实录（四形 × pre/pre-line × 单/双插代表形）。

## 实施约束

- ihd1 在飞窗禁码面写（L4 lib 腿现树编译，混合二进制风险）；待批 ALL-DONE 且判读收口后由下一实施轮出刀。
- 与轴二元素端点臂（c19712be9）零交叠：彼改元素端点归一（:3501-3514），本臂插 :3629 后，hunk 不相交。
- D 族 `{}` 形（×12）另案：其 sc 为元素端点形者已归轴二臂域，刀后复普查定残量，勿在本刀夹带。
