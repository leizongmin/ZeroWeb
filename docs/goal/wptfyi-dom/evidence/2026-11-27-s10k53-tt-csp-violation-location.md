# s10k53 刀㊺c：TT CSP 违规位置归因（S3 2F 收敛）

日期：2026-11-27 · 流域：wptfyi-dom（DC-2 切片 13 TT 席）· 基：HEAD `b9f4fe6c9`

## 目标与根因链

S3 族两页（`should-sink-type-mismatch-violation-be-blocked-by-csp-003` / `should-trusted-type-policy-creation-be-blocked-by-csp-005`）断言 violation 事件携带**触发脚本位置**（sourceFile＝外链 helper URL、lineNumber＝3/4、columnNumber≥5/7，CSP3 §create-violation-for-global）。根因链两段：

1. **引擎侧缺位置字段**：part14 `_zwTtViolation`（part14-trusted-sinks.js:308）只有 documentURI；part06 派发侧（part06.js:24216）`sourceFile = f.sourceFile || f.documentURI`、`lineNumber = f.lineNumber || 0` 缺省回落，事件恒为 documentURI/0/0。
2. **webview 侧 sourceURL 路径翻倍**：runner 投递 `data-inline`＝语料根相对裸路径（testharness.rs:11544），webview 消费者页相对解析 → `trusted-types/trusted-types/support/x.js` 翻倍（s367 先例同形）。strict lane 把该值作 `//# sourceURL` 注入，外链 helper 脚本栈帧 URL 随之翻倍 → 即使栈捕获成功 sourceFile 断言仍败。

## 刀面（三处，80 行级）

- **part14 `_zwTtViolation` 栈捕获**：同步窗 `new Error().stack`（deferred setTimeout(0) 任务栈是 timer 回调非违规点），自顶向下走栈：http 帧＝外链脚本（归其 URL/line/col）；行号 <5000 匿名帧＝页内联块 → 停在 null（派发侧回落 documentURI＝内联触发面正确归属）。**阈值走栈版**（终版）：shim 拼接偏移万行级（part01.js ~17k 行、TT 门整体 ≥ part12 段 ~174k 行）、页内联块 <1000 行，界线 5000；对 part01 <5000 行区域调用链有理论误判（误停→null 回落＝退化为刀前行为，不致错归）。v1「最深 http 帧」启发式被 tt6 臂证伪（见下）。
- **part06 透传**：捕获结果经 `sourceFile/lineNumber/columnNumber` 字段进派发 payload，缺省 undefined 时走既有 `|| documentURI`/`|| 0` 回落＝零漂移面。
- **webview.rs 两处 data-inline 归一**（s367 同款）：`strict_report_src` inlined_src 臂（~5306）与 CSP checkpoint（~5351）——含 `://` 或 `/` 前缀原样，否则补 `/` 变 root-absolute 再 `resolve_document_url`；页相对外链 src 原样（真 attr 形已按文档基正确）。

## 二进制谱系与臂账（TT 电池套内，guard 包装，空闲机）

| 臂 | run-bin md5 | 脸 | 说明 |
|---|---|---|---|
| tt5 基线（s10k52） | `3f37ab178503` | 2254P/13F/19T | HEAD b9f4fe6c9 pre-knife |
| tt6 刀面 v1 | `74753e50eff9` | 2255P/13F/19T | S3 两键 F→P，但 source-file-path `same-document script` P→F |
| **tt7 终判** | `e25ba2e88309` | **2256P/11F/19T** | F11/T19 与 tt5 基线**逐键 set-exact**（排序差为 runner 完成序） |

- tt6 回归发现：v1 启发式「最深 http 帧」跳过内联匿名帧后误抓 `csp-violations.js:38:13` helper 帧（helper `await fn()` 在栈内）。tt3/tt5 考古证实该页基线＝subtest1 1P＋页级 T，回归仅 subtest1。
- 阈值走栈修复后：solo7 四页验证——probe payload `"srcFile":"https://wpt.test/trusted-types/support/zz-s10k53-stack-ext.js","line":4,"col":23`（外链触发归外链脚本）；`currentScript` 外链脚本内为 null（佐证弃 currentScript 方案）；source-file-path 回 1P＋页级 T（2/3 子测 unreported 既有结构形）；003/005 保持 Pass。
- tt7 F11＝5 real（block-text ×2、getAttributeType-namespace、DOMParser-regression、SVGScript-internal-slot）＋6 zz 探针页；T19 与 tt5 逐页恒等。**S3 两键离场＝TT 真实池 7F→5F（散布 7＝S3 2✓＋S4 5）**。
- 判读依据：完整臂日志 `/tmp/s10k53/ver-tt7.log`（tt6 `ver-tt6.log`、solo `solo7.log`）。

## dom 跨套件零漂移（webview 归一化跨套件面证明）

dom 臂（`e25ba2e88309`，09:46-09:54Z 空闲机，TIME_LIMIT 2700）＝**57,463P/361F/12T** vs s10k51 同跑法锚 `ver-dom.log`（HEAD 4974ee5df 面，57,420P/362F/**12T**）：

- **F/T 集差恰一行**＝`dom/nodes/MutationObserver-attributes.html`（s10k51 面含 F，本臂 P——即已收账 MO 修复键，master.md 记录 41P/1F→42P/0F 逐字吻合）；T12 集逐页恒等。
- P 差 +43 全归因：OUT-T 唯一差异＝MO 键 41→42P（+1）；IN-T＝Range-mutations-replaceData 838→883（+45）/dataChange 882→879（−3）净 +42＝**Range-mutations 双页截断抖动已知噪声族**（±150 级带内，dom 腿逐文件判读基线在案）。
- 注：master.md 记录面行文「57,420P/362F/0T」的 0T 与 s10k51 ver-dom.log 实际 12T 不符，判读以 log 为锚；T12＝webkit-anim ×4＋incumbent ×2＋abort-signal-any＋replaceChildren ×2＋Range 双页＋cross-origin-named-access，全量单进程预算形。
- 判读依据：`/tmp/s10k53/ver-dom.log` vs `/tmp/s10k51/ver-dom.log`，逐页 P 计数 python 对账。

## 残面挂账（不在本刀域）

- source-file-path subtests 2/3（same-origin/cross-origin runtime script loads）pending＋页级 T＝结构性既有（tt3/tt5/tt7 三臂恒），非本刀回归。
- 探针两文件（zz-s10k53-stack-probe.html＋support/zz-s10k53-stack-ext.js）终判后已删（gitignored 语料）。
- S4 余量 5F、creation-001/002 页级 T（pipe= header 注入不支持）、nav 005/006.sub/csp-headers T 恒挂账不动。
- 累计头数 census 汇总挂账不变：MO +1＋nav +5＋本刀 +2 与 dom 复验 −321F 面尚未入 84,291 头数，全量 census 汇总挂下轮。

## fmt 门注记

`cargo fmt --all -- --check` 既有 diff 全在 `crates/style-system/src/if_function.rs`（css 流 ef1625b29 已提交面，非本轮文件，不顺手改）；本轮 .rs 面 webview.rs `rustfmt --check` 零 diff。
