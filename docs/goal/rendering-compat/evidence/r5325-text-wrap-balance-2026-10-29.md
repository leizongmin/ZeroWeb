# R5325 — `text-wrap: balance` 断点重排（css-text-4 §text-wrap-balance）

- **日期**: 2026-10-29
- **驱动案**: `css/css-text/white-space/text-wrap-balance-002.html` ✗0.00% → **✓1.00%**、
  `text-wrap-balance-004.html` ✗0.00% → **✓0.00%**（连带 003 ✓2.10% → ✓0.00%）
- **corpus**: 811F → **809F**（16590 案；**FIXED=2**，REGRESSED=0），通过率
  15781/16590 = **95.1%**
- **域**: css-text white-space/text-wrap-balance 家族（25 案，基线 23P/2F）

## 定性

balance 家族 25 案（wpt_file_loader 全量走查发现；reftest-skip-list 无 balance 项），
基线仅 002/004 红。两案同为 mismatch 型：测试页带 `text-wrap: balance`，misref 页
无——两页应仅在 balance 效果上不同；ZW 渲染 0 像素差 = balance 完全未生效，与无
balance 的 misref 逐像素同像。

三层根因（逐层实证收敛，前两层修复后 004 仍 0 像素差）：

1. **规划面不存在**：layout 断行纯贪心，002（Ahem）渲染 ≡ misref。R2161 时代仅在
   tree.rs `container_has_balancing_text_wrap` 做 probe 抑制探测（注释自述「layout
   不 balance」），无断行语义。
2. **重测/绘制路径覆盖**：主路径注入 balance 后 002 即绿，004 仍红。探针（临时
   eprintln，已移除）实证两处：
   - 规划器 Br 分段键偏移——段内 `enumerate` 的 item_idx 未加段首基准，组 2 断点键
     落到组 1 词位（伪命中 ×6）；
   - 布局生效 IFC 构建点多于主路径（remeasure/multicol/postprocess float 重测），
     未注入 balance 的重测以贪心行形覆盖主路径断点（`n=8 greedy ×19` vs `n=9
     planned ×2`）。
3. **非 Ahem 容器 paint Path B 权威重跑**（决定层，截图定谳）：REFTEST_DUMP 截图
   实测 004 测试页三段（蓝/橙/品红）全部贪心断行——layout 期平衡行形根本没被绘制。
   compute_final store 门（`!is_pure_ahem && !allow_non_ahem_store` 即 return 不存
   储）使 **monospace 页（004）的行几何从不存储**，paint text.rs「非存储模式下运行
   IFC」臂自建无 balance 的 IFC 重排并绘制 = 权威几何。探针 `[PT] stored=false` ×
   全部 400px 宽盒旁证。002（Ahem）走存储路径，故主路径修复即绿——这也解释了两案
   对同一修复的不同响应。

## 实现（kill-switch `ZW_TEXT_WRAP_BALANCE=0` 回退纯贪心，default-on）

**算法**（css-text-4：行数不变、词界重排最小化行长参差；具体算法 UA 自定义）：

- 贪心断行得行数 N（不改行数的 spec 面）；N ≥ 2 时在同 N 行内 DP 求
  min-Σ(rendered²) 的词界划分（贪心 split 恒可行 → 必有解；行内溢出对行尾扩展单调
  → 内层提前终止）。
- **非严格更优不改断点**（`dp_cost + 0.01 ≥ greedy_cost` 回退）：贪心已参差最优时
  保持字节级旧行为——green 面零扰动。
- 强制断行（Br）分组独立均衡（004「Groups of lines separated by a forced line
  break are processed separately」）：每段单独贪心 → DP，断点键以段首在全 items
  向量的下标为基准（分段偏移 bug 的修复位）。

**规划面（新文件 `inline/balance_breaks.rs`，include! 并入 mod inline）三级 scope 门**，
门不满足整段回退贪心：

- IFC 级：无 float 排除带（行带宽恒 = container_width）、无 text-indent、容器
  white-space normal、word-break normal、非 break-word、items 仅 Text/Br；
- run 级：无水平 margin/padding advance、无 bidi（rtl/override/plaintext）、
  ws_override 为 None 或普通折叠；
- 词界级：相邻词对必须空白分隔（词尾空格 / 纯空白 run / run 前导空格）——跨 run
  粘连、ZWSP 合并、CJK per-char 词界回退贪心（R5161 GL 面不越界）。

贪心与 DP 共享同一 span 模拟（fit 判据 / 前导空格门 / 行尾尾随空白扣除与词臂逐项
同式），规划结果 = `(item_idx, word_idx)` 断点集合，由 break_items_into_lines 词臂
在贪心 fit 门之前消费（行非空时落成推行）。

**注入位**（布局生效路径全量，度量路径不注入——balance 不影响 min/max-content）：
主构建 `.with_balance(style.text_wrap == Balance)`；fragment-segment remeasure、
remeasure_inline_only_containers 主/h_ctx/col_ctx、multicol 三处、postprocess float
重测——统一经 `resolve_balance_for_ifc`（inline_finalization）。**paint Path B**
（`crates/engine/src/paint/painter/text.rs` 非存储臂）：同源内联 match 注入——非
Ahem 容器的该重跑即权威绘制几何，漏注入即 004 的第三层根因。float 重测路径天然被
IFC 级 float 门挡住（双保险）。

## 回归测试

`crates/layout-engine/src/inline/tests/r5325_text_wrap_balance_tests.rs` +6 pin
（Ahem 精确字宽）：

1. `r5325_balance_redistributes_breaks_min_raggedness`——400px「123 567 901 345 789
   123」贪心 [380,60] → [220,220]（002 镜像，主判例）；
2. `r5325_balance_groups_forced_breaks_separately`——Br 分组独立均衡：组 1 贪心
   [220,40] → [100,160]，组 2 单词不跨组重排（004 语义镜像）；
3. `r5325_balance_multiline_reduces_raggedness`——5 行 Lorem 段（004 组 1 镜像，
   Ahem 8px ≡ monospace 8px/字）贪心 [400,384,384,392,256] → [312,392,352,376,384]
   且 Σ² 严格降；
4. `r5325_balance_noop_when_greedy_already_even`——贪心已参差最优：balance 开关
   两侧断行逐 fragment 全等（零扰动腿）；
5. `r5325_balance_scope_fallback_break_word`——break-word 语境回退贪心（scope 腿）；
6. `r5325_balance_single_line_stays_one_line`——单行段不新造断行（N=1 腿）。

**mutant**（`ZW_TEXT_WRAP_BALANCE=0`）：①②⑥ FAIL（三条驱动腿：重排 / Br 分组 /
多行降参差）、③④⑤ PASS（守卫腿两态均过属预期）——双向可判别。kill-switch 同门
规划与 paint 两层（规划器入口统一检查），单一 mutant 覆盖全链。

## 门禁

- fmt：本轮 9 文件 0 漂移（`cargo fmt --all --check` 现存 js_dom_bridge.rs 7 处 +
  testharness.rs 3 处 = 既有并行流活跃态，移交知悉）。
- clippy zero-layout-engine + zero-engine `--all-targets -D warnings` 0（systemd
  scope 守卫）。
- **zero-layout-engine lib 2248P/0F**（2242+6）；**zero-engine lib 10F** =
  js_dom_bridge fetch 契约案，clean HEAD worktree A/B 同红 = **既有记账**
  （R5320 起提请 net-api 流自查），非本轮修改面。
- **corpus 全量（最终树）**：16590 案 **809F**，A/B 对照 811F 旧账逐案 diff 恰
  {−balance-002, −balance-004}，**REGRESSED=0**；balance 家族 25/25。
- **本地 725 套件**：724/725，唯一红 = R5311 既有记账
  `float-shrink-inline-children-zw-001` ✗3.68%。
- **product-smoke**：welcome vs chromium **15.68%** ≤ 20%（与 R5324 零位移）+
  struct ×6 PASS；**legacy** 42 fixtures struct 0 失败（paint Path B 同为 legacy
  路径，balance 仅 text-wrap:balance 页激活，legacy 零触达）。
- **bench-gate（定向 zero-layout-engine + zero-engine）GATE PASS 25 指标 NEW=7
  全预算内**（welcome first_paint 17.07ms ≤ 59.63；peak_rss 192.52 ≤ 359.02MB）。

## 挂账

- **收口**：balance-002 / balance-004（balance 家族 25/25）。
- **新线索（本轮探针发现，与 R5324 同源待扫）**：规划面 scope 门外的多行 balance
  形态（margin/padding run、bidi run、CJK 粘连词界）现按贪心回退，corpus 无红案
  驱动；有 0.00% 段新案时逐门放行。
- **另案维持**：white-space-pre-001、非 preserve `<br>` 20px 地板换 strut、
  first-line-line-height-003/abspos-block-level-001 复评；block 断行后行首可折叠
  空格未塌（R5324 新线索）。
- **维持**：非 img inline-block 口径分裂、content-none-select-1 脆弱族、c44-003
  残差、va 族余案、c548 结构性案排除口径待用户决策、zero-engine js_dom_bridge
  fetch 10 案（本轮 A/B 复核 10F，R5320 记 8F 后并行流有新增，仍属 net-api 域）+
  details 闪断、js_dom_bridge/testharness fmt drift 移交。
- **下一步 R5326**：①809F 账本续扫择轻（0.00% 段 + 低 diff 段，避开 va 域与 OOF
  锚域）②css-multicol 103F / css-transforms 67F / css-flexbox 52F 域内挑轻 ③
  text-wrap 相邻域（text-wrap: pretty/stable 未实现，corpus 无驱动案，维持记录）。
