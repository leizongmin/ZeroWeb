# ip M2 span 壳族三臂刀收口——gate v3 四腿全绿 + gate v1/v2 定谳与多执行体竞态记录

日期：2026-11-14；刀体 `0ee05d644`（已推 origin）；判读域重锚 doc-only `6c8457a36`（控制面 v1.103，刀域零交叠）；终批 `batch-ipm2-gate-v3.sh` ALL-DONE 08:18:34 @6c8457a36 bin `bf71873438b2cd3b2b1991d79874a056`。

## 1. 刀体（M2 三臂，part01f.js ipara + part113 探针 ⑬⑭⑮）

- **M2-A flex/grid item 块化**：container display ∈ flex/inline-flex/grid/inline-grid 且 caret 链全行内壳时 item 层拆分（container 不复制，半链构造与 M1 窄门同源），wired write 走 container 域 `innerHTML=` 单域替换契约，空侧壳链内垫 br（左不标记/右标记写后重打）。corpus 24 键（三 caret 位 × 四 display × dps 双面）。
- **M2-B extract 前导壳携 br 免垫**：`ipLeadSkipC` 跳过重建后旗标恒 false 臂裸垫第二 br；前导壳链 firstChild 脊下钻探 br 已携免垫。
- **M2-C 重建链根垫底下钻 innermost 空壳**：br 落最外层壳尾 → 入链内首 innermost。
- part113 insertparagraph 探针类 15/15 绿；TEMP-TRACE 全摘净；fmt/clippy 干净。

## 2. gate v1 ABORT 定谳（07:11:25，judge 白名单规格错，非刀体回归）

- 批 `batch-ipm2-gate-v1.sh`（锚 0ee05d644）L1 judge ABORT。数据面：rows 7195 恒一、F 206→99、F2P=107、P2F=0、lost=0——零回归。
- 拒因双柱：①judge 锚 `e1-ip.log` 206F 原基线，M1 已翻绿 49 键全数落白名单外（M1-CARRY 面未申报）；②18 键 M2-B/C 机制泛化外溢未申报。
- 18 键独立归因（vs M1 终态 `ipm1-v4-ip.log` 157F：F2P=58 = FLEX 24＋BR-TAIL 14＋SPAN-SHELL 2＋UNEXPECTED 18）：
  - **DBL-FMT 4**（`<p><b><i>foo[]</i></b></p>` 等双层 fmt 壳）：M2-C innermost 下钻为全壳族通用路径非 span 专属；v4 got `<b><i></i><br></b>` → exp `<b><i><br></i></b>`。
  - **BLK-SEL-DBLBR 14**（`<h3>[abc]</h3>`/`<div>[def]</div>`/`<p>[abc]</p>`/`<p>foo</p><h1>[bar]</h1><p>baz</p>` 整块选区族）：语料 exp 要求双空块各携恰一裸 br（insertparagraph.js :2117-2151 实录 `<p><br></p><p><br></p>`）；M2-B 脊下钻探 br 免垫消除旧 pad 无条件双 br（v4 got `<div><br><br></div>`）。
  - 翻绿全数真修复（Pass=got==exp），五族 got/exp 字节实证见 ACTIVE-BATCH 07:25 条。

## 3. 多执行体发射竞态记录（07:13-07:26，五席收敛）

- v2 命名批 bash（1451247）07:13:14 exec 时同名脚本已被并行轮两度重写，bash 开 fd 懒续读字节域错位（v1 横幅 + v2 重定向混合执行），07:16:57-07:18:45 间死亡；L1 腿链孤儿存活完走（922s）留下同二进制第二遍全量跑 `ipm2-v2-ip.log`。
- 本席（doc v1.103 写者轮）发射链 Write→Edit→syntax-check→fire 自洽，落在锁忙/竞态窗未成批；已修 judge 版 v2 脚本留档 `batch-ipm2-gate-v2.sh.quarantined-fxjudge`（base 157F + ENUM 18 + drift 断言，判读语义与 v3 等价，不发射）。
- 发射权按账本 flock 仲裁归 v3（1456954，核验方轮）；五席独立定谳映射收敛（UNEXPECTED 18 = BLK-SEL 14 + DBL-FMT 4），第四方产出 `ipm2-f2p58.txt`（58 键恰集）交叉。
- 教训：同名产物脚本在飞期禁改写；批内脚本身份用日志横幅 + 特征串双重核验；leg 链孤儿化后日志仍可作恒一样本（bin 恒一前提）。

## 4. gate v3 终判（四腿全绿）

| 腿 | 结果 |
|---|---|
| L1 ip | rows 7195 恒一，**F 206→99**，F2P=107 六族白名单零外溢（FLEX 24＋BR-TAIL 20＋SPAN-SHELL 28＋M1-CARRY 14＋DBL-FMT 4＋BLK-SEL-DBLBR 17），P2F=0，lost=0 |
| L2 ab | **10092 Pass / 0 Fail / 0 Timeout** 严格 + judge-dps VERDICT OK |
| L3 mt | rows 10092 恒一，F 2→2 残红恰 X37（subscript+formatblock qS/qV 双行），F2P=0 / P2F=0 / po_fail=0 / lost=0 零翻越 |
| L4 enginelib | 3935P / 7F，failed 名册 vs 基线逐名 **identical**（diff 空） |

- **跨跑恒一**：孤儿第二样本 `ipm2-v2-ip.log` vs v3 腿 `ipm2-v3-ip.log` 7195 键逐键状态 **drift=0**（同 bin bf718734 独立两遍全量）。
- 对账：206 − 107（F2P）= 99 ✓；157（M1 终态）− 58（M2 净增量）= 99 ✓。普查预期 34 键 → 实收 58（+24 flex 族恰中 + 18 机制泛化外溢）。
- ip 面通过率（compare 分面）：残 99F（含 qV 读面 8），下一普查定 M3 起刀面。

## 5. 门禁链谱系

v1（07:11 ABORT：judge 白名单规格错）→ v2（命名克隆撕裂读，未成批，孤儿腿留恒一样本）→ **v3（07:20:54 发射，08:18:34 ALL-DONE 四腿全绿，本收口）**。判读规格演变：e1 原基线窄白名单 → 157F 基线 + 18 键枚举（v2 档）→ 206F 基线 + 六族白名单全键覆盖预验证（v3 档，他轮审计 PASS）。
