# ip 残面 M2 普查（gate v4 ipm1-v4-ip.log 157 键重聚类）

- 日期：2026-11-14
- 数据源：`.acceptance/dps/ipm1-v4-ip.log`（gate v4 L1 腿，@8add42713 bin cc51d45a，M1 收口后权威残面）
- 方法：Fail 键逐条解析（json raw_decode 命令序列 + 场景引串 caret 位 inline 壳栈走查），按 face × 断言 × caret 壳链签名三轴聚类。

## face × 断言总表（157）

| face | compare innerHTML | qV(dps) 读面 |
|---|---|---|
| dps-div | 86 | 4（before） |
| dps-p | 59 | 1（after） |
| plain ip | 4 | — |
| swcss-false | — | 1 before + 1 after |
| swcss-true | — | 1 before |

- compare 149 + qV 读面 8；qV 读面 8 键与 r5450 基线同数（独立小子族，dPS 读面挂账维持不追）。

## compare 149 按 caret 壳链签名

| 签名 | 键数 | 形态 |
|---|---|---|
| bare@text | 51 | 裸文本 caret：ws 环绕 ~17（`abc[] `/`a&nbsp;[] b`/` [] abc`）、整块选区 ~12（`<div>[abc]</div>`/`<h3>[abc]</h3>`/`abc</div><div>[def]</div>`）、list/dl caret 3（`ab[]cd</li></ol>`/`</dd></dl>`）、script rawtext 2、余散 |
| NO-CARET | 17 | `{}` 空 caret/范围形（`{<table>…</table>}` 整表选区、`<p>foo</p>{}<br>`、`<dl>{}<br>` 嵌套） |
| span 壳族 | 34 | span@text 16 + span@shell-tail 8 + span+span@shell-tail 4 + span@br-tail 4 + span+span@br-tail 2 |
| br-tail 裸/a 族 | 22 | bare@br-tail 16 + a@br-tail 2 + a+b@br-tail 2 + b+a@br-tail 2 |
| b/i/a 壳残 | 10 | b@text 2 + b+i@text 2 + i+b@text 2 + b+a@text 2 + b+i@shell-tail 2 + i+b@shell-tail 2 |
| bare@block-boundary | 7 | 表跨界（`fo[o<table><tr><td>b]ar`）5 + `<blockquote><p>foo[]<p>bar` 2 |
| bare@shell-tail | 6 | 表 td 出口 3 + `<pre>foo&#10;[]` 1 + `<blockquote>foo[]` 2 |

- flex-span 子族：`display:flex`/`inline-flex` 容器 × span × 三 caret 位 = **6 场景 × 2 dps 面 = 12 键**（span@text 16 中的主体）。

## M2 候选排序（DC-4 择大续刀）

1. **span 壳族 34**（最大连簇；M1 壳链窄门未覆盖的 span 形；flex-span 12 可同门顺带或独立微刀——flex 容器块语义新面）
2. **bare@br-tail+a 族 22**（caret 落 br 后语义，机制单一）
3. **ws 环绕 caret ~17**（拆分位空白规范化面）
4. **整块选区 ~12**（`[abc]` 全选拆分语义，v4 F2P[OTHER] 3 键同族互证）
5. NO-CARET {} 17 / 壳残 b-i-a 10 / 表跨界 7+6 / script rawtext 2 依次靠后。

施工轮注意：签名聚类只到形不分因，M2 开刀前先对 span 壳族 got/exp 逐形 diff 定机制（M1 wrap 臂窄门扩形 vs 独立新臂），预检批沿用 gate v4 红线（ip F 157 恒一、F2P 白名单 M2 域、P2F=0 硬门）。
