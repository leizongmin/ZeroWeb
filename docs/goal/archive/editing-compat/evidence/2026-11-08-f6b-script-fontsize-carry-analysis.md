# F6b sub/sup×fontsize carry 刀前分析（2026-11-08，③ F6b 开刀轮）

## 活体普查（f1b-mt-post.log，F1'-B 后二进制，main 域）

F6b 簇 14 键（knife11 普查口径：sub/sup+delete/fwddelete 4 + sub×fontsize 互序 4 + delete 族 sub/fontsize 嵌套 6）。**X37 两键（subscript>formatblock qS/qV）已定谳窗界伪影除名，不在本刀范围**。live 实测期望/现状逐键：

### C 族：sub×fontsize 同案互序（4 键，?8001-9000）

| 序 | 期望（语料） | 引擎 got |
|---|---|---|
| sub→fontsize(2)→type | `foo<sub>a[]</sub>bar`（**无 font**） | `foo<font size="2"><sub>a</sub></font>bar` |
| fontsize(2)→sub→type | 同上 | 同上 |
| sub→fontsize(3)→type | `foo<font size="3"><sub>a[]</sub></font>bar` | `foo<sub>a</sub>bar`（font 丢） |
| fontsize(3)→sub→type | 同上 | 同上 |

### A 族：sub/sup+delete/fwddelete+type（4 键，?1001-2000 / ?2001-3000）

| 序 | 期望 | 引擎 got |
|---|---|---|
| sub→delete→type | `fo<sub><font size="3">a[]</font></sub>bar` | `fo<sub>a</sub>bar`（font 丢） |
| sub→fwddelete→type | `foo<sub><font size="3">a[]</font></sub>ar` | 同构丢 font |
| sup→delete→type | `fo<sup><font size="3">a[]</font></sup>bar` | 同构丢 font |
| sup→fwddelete→type | 同构 | 同构 |

### N 族：delete+type 在 sub/fontsize 嵌套树内（6 键，?8001-9000 / ?9001-last）

| setup | 期望 | 引擎 got |
|---|---|---|
| `foo<font size=2><sub>[bar]</sub></font>baz` | `foo<font size="1"><sub>a[]</sub></font>baz`（**size 2→1**） | `foo<font size="2"><sub>a</sub></font>baz` |
| `foo<sub><font size=3>[bar]</font></sub>baz` | `foo<font size="3"><sub>a[]</sub></font>baz`（**换序 font 外**） | `foo<sub>a</sub>baz`（font 丢） |
| `foo<font size=3><sub>[bar]</sub></font>baz` | `foo<sub>a[]</sub>baz`（**font 丢**） | `foo<font size="3"><sub>a</sub></font>baz` |
| 尾选区三形（[bar</sub></font>baz]） | 同上三形对应尾版 | 同构 |

## 机制考古

1. **语料=手编 spec 期望，非 Chrome 录音**。multitest.js 案例块带编辑意图注释（`// <font> should be outer-most element`，line 2065/2179/2185）。multitest 页面加载即 `styleWithCSS=true`（工具页头注释披露录音生成上下文的顺序状态携带），per-command queryCommand 期望值内嵌。
2. **spec 参考实现 oracle 实测**（include/implementation.js `myExecCommand`，headless Chrome 驱动，2026-11-08）：A 族 3/3 与语料逐字节吻合（deletion typing style 机制实锤：delete 后 typing style 显式携带 fontSize 16px，typing 组合 `<sub><font size=3>a</font></sub>`）。**C 族/N1/N2 与参考实现有分歧**：参考实现 C 族一律 `<sub><font size=N>`（font 内）、N1/N2 也是 font 内；语料要求 font 外（editorial 规则）+ C1/C2 的 size-2 丢弃。语料是绑定目标。
3. **size 等价跳过规则**（从 N1 size 2→1 反推）：spec fontSize px↔legacy 桶映射 13px→size 1、16px→size 3；`<sub>` 内 effective = smaller(16px)≈13.3px → 与指定 13px 同桶 → 等价跳过 wrap（C1/C2 无 font 的机理）；16px ≠ 13.3px → wrap（C3/C4）。N1：删除位 effective 13px（font size=2 上下文）→ typing style 记 13px → 重发 size 1。
4. **A vs N 嵌套序矛盾**（语料自身）：A 族（裸文本+空壳删除）font 内；N 族（嵌套树内删除）font 外。判别特征=删除位是否在既有壳内。引擎需按上下文分派两种序。
5. 引擎现状 systematic 反转：N3（应丢）引擎发、N1（应换 size=1+换序）引擎保 size=2、C 族两序都错——删位 typing style 记录源与组合序双缺陷。

## 引擎代码路径锚点（2026-11-08 续轮定位）

- **C 通道（指定值）**：fontsize collapsed 臂 `_zwVaSetValueOverride`（part01d :1677，`_zwVaSetSelectionValue` 无 contained formattable 分支）→ inserttext 核头快照（part01 :9733 `__zw_ce_value_override_snapshot`）→ 插入后重放 `__zw_ce_value_restore_overrides`（part01 :10095-10099）→ 临时 range 走 setSelectionValue 包 font。现状偏差：重放无桶等价跳过（size2 多包）且有默认值跳过面（size3=16px 被当缺省丢弃）；font 落位在 armed sub 壳外（重放整段包裹）。
- **A 通道（删除携带）**：删除捕获在 part01f `__zw_ce_capture_typing_chain`（:1169）+ delete 入口 F2 link-carry（:1222 注释块，仅 link 上下文捕 color/face/size）；typing 组合 `_zwCeTypingInsert`（part01 :9581）carry font 落 tags **内层**——与 A 族期望形（`<sub><font size=3>` font 在 sub 内）同构，缺口仅是 sub/sup 上下文的 fontSize 捕获（现有捕获门=link 上下文，不含裸 sub 壳）。
- **N 通道（嵌套树）**：同删除 strip-wrappers/keep-shell（part01f `_zwDelKeepShellOf`）+ 同 C 重放路径；需语料形三面：幸存 font 壳改 attr 重映射（N1 size2→1）、font 壳换序到 sub 外（N2/N4-N6）、font 壳溶解（N3）。
- **sub/sup arming**：collapsed 臂 part06 :4916-4961（`__zwTypingTags` ± 互斥 :4945）；bool 读面 fallback :2788。
- **桶映射静态表**：part01d :180-199 `_zwVaLooselyEq` sizeMap（font[size] 1-7 级联读数表，host UA 计算口径 10/13/16/18/24/32/48）——C 等价比较与 N 重映射共用此表的候选基座。
- 机制模型（自洽核）：删除记录=caret 位**未缩放** effective px→桶（N1: 13px→桶1→size1 ✓、N2: 16px→桶3→size3 ✓）；打字重放比较= caret 位**含 sub 缩放** effective→桶（C1: 指定 13px 桶1 ≡ 13.3px 缩放桶1 → 跳 ✓、C3: 16px 桶3 ≠ 桶1 → 包 size3 ✓）。序规则：指定值/嵌套树通道 font 外（语料 editorial 注释），typing-carry 通道 font 内（A 族语料形=oracle 实证）。

## oracle 定谳与语料统一规则（2026-11-08 续轮二，oracle 全 12 案跑通）

修正：N 族 census 表行映射有误，真形（multitest.js :2173-:2412，F6b 刀域 6 键 = :2178/:2183/:2188 中选 + :2398/:2403/:2408 尾选；:2173/:2393 引擎已绿作机制见证；:2283-:2298 前导选区四形引擎已绿）：

| setup | 语料期望 | ref oracle 输出 |
|---|---|---|
| sub[font2[bar]]（:2173 绿） | `<sub>a</sub>baz` | `<sub><font size=2>` 不符 |
| font2[sub[bar]]（:2178 红得 `<font size="2">` 未重映射） | `<font size="1"><sub>a</sub></font>baz` | `<sub><font size=1>`（重映射✓ 位置异） |
| sub[font3[bar]]（:2183 红得 font 丢） | `<font size="3"><sub>a</sub></font>baz` | `<sub><font size=3>`（幸存✓ 位置异） |
| font3[sub[bar]]（:2188 红得 font 保留） | `<sub>a</sub>baz` | `<sub>a</sub>` ✓ |
| 尾三形（:2398/:2403/:2408） | 同上三形尾版 | 同构（✓/位置异/✓） |

**ref 机制定谳**（implementation.js myExecCommand 实测 8/12 与语料同向）：外层 font 壳（包着 script）删除时剥离；内层 font 壳幸存原样；typing 重放桶=删除位 computed px（含 sub 0.83em 缩放）→ sizeMap 最近桶，与 caret 位（sub 缩放后）effective 桶比较，不同→包 font size=记录桶于 script 内层，相同→不包（N2178 2→1 重映射、N2188 等桶溶解均 ref 自证）。

**语料统一规则（12 案全合，ref 上加一条 editorial）**：打字段归属的 font，桶≠桶（script 缩放后 caret 位 effective）→ 放 script 壳**外**（editorial hoist，幸存内层 font3 壳按原 attr 提出、重放 font 按记录桶包外层）；桶等 → 完全溶解（幸存 font2、等桶重放皆消）。**例外 A 族**（armed typing script 壳、无前树）：font 留 script **内**（ref typing-style 通道原形，`sub,delete,inserttext` → `<sub><font size="3">a</font></sub>`）。位置分派判据=script 壳是否本次 armed 产物（A）或前树幸存（N/C→外）。

**C 族并入同规则**：指定值通道（fontsize 命令置值）重放比较=指定 px 桶 vs caret 位 script 缩放 effective 桶（C1: 13px b2 ≡ 13.3px b2 → 溶 ✓；C3: 16px b3 ≠ b2 → font size3 壳外 ✓）。

## 引擎实现面（三臂+公共机具，桶映射基座实证）

- 公共机具：`bucket(px)`（sizeMap {1:10,2:13,3:16,4:18,5:24,6:32,7:48} 最近档，part01d :180 静态表同源）+ `scriptContextOf(node)`（上行链探 sub/sup/幸存 font[size]，宿主止）。引擎 UA 表已有 `sub,sup{font-size:0.83em}`（style-system/src/lib.rs :2288），gCS computed px 天然含缩放，桶读数直算可行。
- 臂 C（值重放面 part01d restore→setSelectionValue）：fontsize 指定值重放前加桶等价门（等→跳过，替换现默认值误跳面——现 size3 被当缺省丢、size2 无检查多包）；序面 font 落 armed script 壳外。
- 臂 A（delete 捕获 part01f + typing insert part01 `_zwCeTypingInsert`）：delete 位在 armed/裸 script 上下文时记录 carry（bucket(computed px)）；insert 消费：桶≠caret 桶→font size=carry 桶包 armed script **内**（现 f2.carry font 内层位同构，扩展捕获门即可）。
- 臂 N（前树幸存面，删除捕获同臂扩展）：幸存内层 font 壳按 attr 桶与 caret 桶比较：不等→提出 script 壳外（attr 原值）；相等→溶解。幸存外层 font 壳剥离后按 computed 记录桶走重放（等→消、不等→包外）。A/N 位置分派按 script 壳 armed 产物判别。
- 引擎现状实测（census 6 红键 got）：外层 font 原样保留不剥离不重映射（:2178 得 size2）、内层 font3 直弃不提出（:2183 得无 font）、外层 font3 保留不溶（:2188 得 size3 保留）；A 族四键全丢 carry（`<sub>a</sub>bar` 无 font）；C 族无等价门+默认值误跳。

## 悬案与下一步

- N4-N6（尾选区）oracle SETUP-FAIL：probe 标记解析器未支持 `[bar` 无闭合 ] 形；尾版期望=闭版对应尾版（语料 line 2288+），实现时逐块核对语料原文即可。
- shim 侧待读：sub/sup 臂在 part06.js + part01f.js；fontsize 臂在 part01c/part01d/part06/part01f。F1'-B 刚动过同域组合机制（D4 拆壳臂/前向并壳/swcss span 吸收），新刀须避开回归其收口面（f1b 门禁锚 part01=a82df7cf/part01d=5ab2368e/part06=5e78d9a4）。
- 设计方向：删除侧 typing style 记录源改为删除前 effective 值 + spec px→legacy 桶重映射；组合序按「删除位壳内/壳外」上下文分派（font 外 editorial 序 vs font 内 typing-style 序）；指定值 vs typing style 双通道序差（C3/C4 font 外 vs A 族 font 内）。
- 验证：probe 十四案（含尾区三形）+ mt 全分母 judge（F2P 窗口以 14 键为预期）+ iol/iul/enginelib/battery 残余批，沿用 F1'-B 门禁批骨架（batch-f1p-b-typing-wrap-gate.sh 修正 WL_PAT v2 后的定长 grep -F 白名单门）。
