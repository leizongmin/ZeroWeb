# ip M1 刀收口——insertParagraph 拆分边界 inline 壳语义（壳尾重开判据 v1→v3.2 全链）

- 日期：2026-11-13 探针挂账 → 2026-11-14 00:44 刀体 → 04:28 gate v4 ALL-DONE 收口
- 刀面：`editing/run/insertparagraph.html` ip 语料 206F 中 div/p 容器子族 116 键（残面普查 r5450 最大子族）——insertParagraph 非 SLC wrap 臂对拆分边界行内壳（b/i/span/a 嵌套链）的壳链闭合/重开语义。
- 刀体提交链（均为 `crates/engine/src/js_dom_shim/part01f.js` + 探针 `crates/engine/src/js_dom_bridge_tests/part113_ip_inline_shell_split_probe.rs`）：
  1. `ba7e5dee2`（11-13 22:56）三形探针挂账——拆分边界 inline 壳语义红定谳。
  2. `0d6d41af5`（11-14 00:44）**M1 主体**：ipara 非 SLC wrap 臂壳链窄门直构（左块壳链闭合 + 右块重开/a 不重开 + 空壳丢弃 + 兄弟随侧归块 + br 垫底）+ wired-write 宿主落位（直构块序列化 innerHTML= 整域重排）；part113 三形探针转正常驻 3/3 绿。
  3. `d1d418daf`（02:04）**v3 三族 P2F 修复**：Fix1 gate 拆分位（caret 归一提升到壳本体时 indexOf(ipPathC)=-1 误落壳尾拆分→ipPathC===ipE 时拆分位取壳内 offset）；Fix2 gate 右壳剥 id（removeAttribute('id') 业务 attr 保留）；Fix3 gate `<a>` 恒重开+空 a 丢弃 + piece-3 存活判据 + 前导壳 guard。
  4. `65b06cbba`（02:32）**v3.1 重定谳**：corpus 三方对照修 v3 首版存活判据 14 键回归（12 带样式 span 壳尾 + 2 `<b><a>`，共同形=文本尾 caret 无后随时纯壳尾应携 br 入链首）。
  5. `8add42713`（03:40）**v3.2 合取定谳（终形）**：纯壳尾重开判据 = 结构∧文本双门合取——结构门 ipSeenAC（walk 已过 a 层的壳重开/未过不重开，修 P2F 双键）∧ 文本门 ipNbspC（a 链内紧邻前字符 \xa0 不重开，corpus `:1126` vs `:1082` 同构分叉实证；v3.1 单文本面/v3.2 首版单结构面各对一半）。收敛式 `visible && (survive || ((ipSeenAC || !ipHasAC) && (!ipHasAC || !ipNbspC)))`，注释两维真值表。part113 钉测扩至 12 形（m1r1-m1r4）全绿。

## 门禁批链（五射，前四射 ABORT 均有独立定谳非行为漂移遗留）

- **v1 ABORT**：轮初 pull --rebase 踩 chk_head 域，锚作废重跑（协议现象）。
- **v2 ABORT（真回归门）**：ip judge 门 rows 7195 恒一 F 206→187 / F2P=47（M1-shell 44+OTHER 3）/ **P2F=28** / lost=0。归因实验：pre-knife 刀父 `d2c3897da` detached 单干腿对 e1-ip 基线 rows/F/F2P/P2F/lost 逐位恒一（「基线→刀体间 19 笔 shim 提交他笔漂移」假说证伪）⇒ 28 P2F 全归刀体，定 A/B/C 三族（宿主块 a×b 双壳嵌套 18 / 壳首 caret+前导裸文本 4 / 带 attr 壳右块 id 未剥 4）。
- **v3 预检+摆动审计**：预检 P2F=0 / F2P=35（F 206→171）表面达标，但主核验三日志同键审计定谳 v2 意外绿面被收回 14 键——v3 首版 survivor 判据「无后随→裸」误伤 carry 面（corpus expect 壳携带形），基线相对门对此盲；gate v3 L1 复判 F 206→159 / F2P=49 / P2F=2——14 键全数回绿，但 a 链结构面新 P2F=2（`<a><b>foo[]</b></a>` expect 裸 `<p><br></p>`，v3.1 误携 `<p><b><br></b></p>`）。corpus 分叉定谳：壳尾携形取决于 a 相对位（a 内侧壳携外层 b、a 里侧壳裸）。
- **v3.2（终形）→ gate v4**。

## gate v4 终判（ALL-DONE 2026-11-14 04:28:36，锚 head=8add42713 bin=cc51d45ad9f36273a3eace653929be4b 四腿恒一）

- **L1 ip 腿**：rows 7195→7195，**F 206→157，F2P=49（M1-shell 46 + OTHER 3）/ P2F=0 / lost=0**，wall=771s——F2P 白名单逐字命中；与 v3.1 四日志同键审计 F2P 集恒一（49 键零换血=carry 面全保）；direct v3.1→v3.2 恰 F2P=2/P2F=0（仅 P2F-2 二键翻绿，外科手术级）；02:26 审计 14 键全绿零回退。
- **L2 ab 腿**：10092P/0F/0T 严格，wall=257s。
- **L3 mt 腿**：10090P/2F，rows 10092→10092，F2P=0 / P2F=0 / po_fail=0 / lost=0，wall=1762s——残红恰 X37 双行（subscript-qS-after + formatblock-qV-after，除名维持）。
- **L4 enginelib**：3932P/7F，名册 7 逐名恒一（r5275/r5277 img-svg settle + r2967 fetch body + fetch matrix + iframe SW + websocket ×2 外债档），零新增红。
- part113 insertparagraph 探针 12/12 绿（⑩ m1r2 复绿=03:08 盲区预测消费；⑫ m1r4 新钉绿）；judge 面盲区（⑩ 族内容形 judge 不可见）由钉测承载。
- 批脚本 `.acceptance/dps/batch-ipm1-gate-v4.sh`；日志 `.acceptance/dps/batch-ipm1-gate-v4.log`、`ipm1-v4-{ip,ab,mt,enginelib}.log`。多执行体协作链（施工/发射/主核验/收养四角色）判读回执见 `.acceptance/dps/ACTIVE-BATCH.md` 2026-11-14 00:44→04:28 各条；写者 session 04:25 rate-limit 终止，批脱离会话存活完走，L2-L4 批级判读由后继轮补记。

## 残面与下一步

- ip 语料残 **157F**：M1-shell 族残 ~70（116−46）+ span/ul/table/plaintext/h3/h1/b/blockquote/a/UNPARSED 宽散族——残面族普查（v4 ip 日志 157 键重聚类）定 M2 起刀面。
- 域决策挂账不动：exec1 474F（text-control execCommand tentative 面）认领前需域决策〔user_actions/shim 实现域〕。
