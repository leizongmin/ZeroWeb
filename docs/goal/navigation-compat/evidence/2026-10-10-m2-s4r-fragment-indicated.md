# M2-S4R — 片段指示元素匹配语义 + 锚 record push 例外 + 跨源 canIntercept 面（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹）
**全量运行日志**: [2026-10-10-m2-s4r-full-corpus.txt](2026-10-10-m2-s4r-full-corpus.txt)（终跑；首跑揪 traverse 滚顶回归后修入）；per-subtest 精确 diff 对 [2026-10-09-m2-s4p-full-corpus.txt](2026-10-09-m2-s4p-full-corpus.txt)（S4Q 终态）
**前序**: [2026-10-10-m2-s4q-navtype.md](2026-10-10-m2-s4q-navtype.md)

## 切片内容（三独立面）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **片段指示元素 decoded 匹配 + 无匹配滚顶**：`_scrollToAnchorForHash` 增 decoded 面（percent-decode + UTF-8 lossy 解码，`TextDecoder('utf-8', {ignoreBOM: true})`——缺省会剥前导 BOM，恰为本面断言点）；raw ID → decoded ID → raw name → decoded name 四级查找；**无指示元素 → 滚回文档开头**（traverse 路径 `_noTopFallback` 门——不得覆写 entry 恢复滚动位） | `part02.js` | [find a potential indicated element](https://html.spec.whatwg.org/multipage/browsing-the-web.html#find-a-potential-indicated-element) + scroll to the beginning；WPT fragment-and-encoding/‑2「%EF%BB%BF→U+FEFF 命中（BOM 不剥）」「%FF→U+FFFD 不命中」「%E2%99%A1%FF 不命中」「%C2→U+FFFD 命中」「无匹配 hash → scrollY 0」 |
| **锚激活 record 侧 push 例外**：hash-setter 的 CCE record 分派由载入态单一谓词改为与 navigate 事件同谓词（`_s4qNavType === 'push'`）——锚激活恒 push 也适用 record 侧（S4Q 只盖事件侧，record 侧载入前 replace 使 `e.from` detach / entries 不增长） | `part02.js` | Following Hyperlink 传 push（不随载入态）；WPT currententrychange-event/anchor-click「载入前 a.click() → e.from 在位 + index +1」 |
| **href-setter 跨源 canIntercept + intercept() SecurityError**：`_setLocationPart` 的 navigate 事件增 `canIntercept` 同源判定（锚路径 S4H 已同面）；`intercept()` 对 `canIntercept === false` 抛 **SecurityError**（原 InvalidStateError 系误记） | `part02.js` + `part05.js` | spec「can have its URL rewritten to app URL」+ intercept() 步骤；WPT intercept-cross-origin「location.href = 跨源 → canIntercept false + intercept() SecurityError」 |

## 数字

| corpus 域 | S4Q 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api（记账链延续） | 234/255 = 91.8% | **236/255 = 92.5%** | +2 |
| 全量 | 369 P / 476 = 77.5% | **377 P / 476 = 79.2%** | +8 |

**8 翻 Fail → Pass，零回归**（per-subtest 精确 diff，F/T 集合恰删 8 行零新增）：
fragment-and-encoding ×3 + fragment-and-encoding-2 ×3 + currententrychange-event/anchor-click +
intercept-cross-origin。

**首跑揪回归 1 案并收口**：scroll-restoration-navigation-samedoc（traverse 到无匹配 hash entry
被新滚顶回退覆写 entry 恢复位 555→0）——`_noTopFallback` 门后终跑复绿。

## 挂账（本轮确认）

- scroll-to-fragid 余项（scroll-to-top/TOP ×2、scroll-to-anchor-name、scroll-position 竖排 ×3、
  003、focus-changes、scroll-frag-percent-encoded）：滚锚几何/书写模式面，属渲染域
  （scrollIntoView 精确位 + vertical-rl/ll），记账回流，不属本 goal 语义面。

## 质量门禁

- `make test`：全绿 **20,359 P / 0 F**（R3061 hash 滚锚 pin 随 S4R 语义更新——无匹配元素
  改钉滚回开头 scrollY=0，单测更新后定向复验 + 全量复跑全绿）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning（含 part06.rs pin
  更新）。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
