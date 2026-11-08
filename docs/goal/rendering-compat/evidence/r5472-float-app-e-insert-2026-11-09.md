# R5472 — floats flush 段逐容器 App E step4<step5 splice（c5525 六案 ✓ 全绿 0.00%）＋ corpus 净改善

日期：2026-11-09
范围：`crates/engine/src/paint/painter/mod.rs` + `collapse_phase.rs`

## 1. 根因链

R5471 后 c5525-fltmrgn 残余 ✓0.19%（r5469 §2 在册：test6 img mr−30 红格绘在行内墨上）定谳为纯 z 序：R5420 floats flush 在下降末施绘＝float 盖**全部**墨（App E step5 违例；step3 块装饰 < step4 floats < step5 墨）。ZERO_DRAW_ORDER 默认开（DC-10）＝发射序即绘制序，dump PNG 复核一致。

## 2. 修法三迭代（corpus 纪律裁决）

- **v1 宿主 pre-ink 位**：`float_pre_ink_pos[host]`（paint_inline_content_phase 入口记录）＋flush 后逐容器 splice。fltmrgn **✓0.00%** 但 fltblck **✗1.42%**（新回归）——dump PNG：float 是 `p.float`（宿主＝div），div 后继兄弟块 `p.flow` 红图案 bg 在 splice 位之后 DFS 发射 → 后继块 bg（step3）盖 float（step4）违例。
- **R5472b 墨主 pre-ink 位**（「首个 ≥ 宿主位的真墨主」）：fltblck ✓ 但 corpus 净 **+9F**（699F vs OFF 690F）——stack-floats-001 型定谳：`.inline` 墨（XXXXX glyphs）之后其内块 `.inline .block` 红 bg 才发射，「墨后随 bg」窗内 DFS 序**不可单点插入**，exact 序需三桶分离（FIXME 在册，架构级 sweep）。
- **终版 glyph 规则**（默认开）：目标位＝窗 `(宿主 pre-ink 位, flush)` 内**首个 Glyph op**；语义门——首墨位之后到 flush 仍有块级装饰 op（Fill/RoundedRect/PathFill/PathStroke/Stroke/Gradient/Image/Shadow）→ 该 float 放弃 splice 留 flush 原位（基线旧序）。不可移动段按 tree order 回填 flush 位（v1 重建曾丢弃＝漏绘隐患，本轮修复）。kill-switch `ZW_FLOAT_APP_E_INSERT=0`（OFF 反证走 corpus 独立进程，R3858 口径）。

guard 不变：影响域 [min_target..end) 含 BlendMode/Transform/Clip/Filter 状态性/位置性破坏 op 或存在 animation scope 时整体放弃保持旧序；op 身份（桶+索引）寻址，typed Vec 与 clip.rs position+pop-tail 重写兼容重排。

## 3. corpus A/B（同二进制 16599，release runner）

- 全量：**ON 688F / OFF 690F**。
- solo×3 定谳：**真修复 3**（delete-inline-in-blocks-middle-001 ✓16.07%、table-anonymous-objects-127 ✓3.18%、tao-146 ✓3.08%，ON solo 恒 0F）；**真回归 0**（run-in-table-between-003 批跑 ✗3.52% 但 ON solo ×3 恒 0F＝噪声家族新员）；留红 mover 3（column-scroll-marker-001/002 band 1.62%/2.04%→4.69%、inline-size-bfc-floats 7.67%→7.72%，双窗恒红、计数中性——近似域内代价，band 变宽记档维持观察）。

## 4. 验证矩阵

- 常驻 judgment test＝`engine/src/paint/tests/r5472_float_app_e_insert_tests.rs` 3 案（draw_order 真插入序断言：后继块 bg < float < 首墨；宿主 bg < float < 宿主墨；跨宿主 float 树序保持——R5420 驱动案语义）。
- kill-switch OFF 锚＝`engine/tests/r5472_float_app_e_insert_kill_switch.rs`（独立进程，断言 flush 原位旧序）。
- c5525 六案 **6/6**（fltblck/fltinln/fltmrgn/fltmult/fltwidth-001/003 全 ✓0.00%）；R5420 驱动案 overhanging-float-paint-order ✓。
- 内测 `make reftest` **725/725**。
- engine 全腿（test-guard compile-first，release）：**3819P/7F**——7F 全部 `js_dom_bridge`（websocket/fetch/iframe），`git stash` 回 HEAD 复跑同 FAIL＝预存红腿（对端流在册观察），非本轮引入。doc-tests 0 案。
- fmt clean；clippy `-p zero-engine --all-targets -D warnings` 清。
- `make product-smoke`：welcome **17.23%**（与 R5465-R5471 七轮同值零漂移）＋ struct-check 全 PASS。

## 5. 挂账更新

- **销账**：R5471 下一步①「c5525 残余 ✓0.19% paint 域择优」——test6 墨 z 序本轮收口，c5525 六案全绿。
- **维持/新增**：三桶分离 FIXME（App E 精确序 sweep 架构——stack-floats-001 型「墨后随 bg」窗在册，单点插入证伪×2）；噪声家族扩员 +1（run-in-table-between-003）；csm-001/002 band 变宽观察；**行盒归属缺陷**归因；fsa 单 owner 切片；缺陷 3＝待用户决策⑥；及控制面既有长账全维持。
- **下一步 R5473**：①行盒归属缺陷归因（display:inline 中间层 lines=0，floats-149 域）或 fsa 度量单 owner 切片择优 ②留红 mover band 变宽择机归因（csm-001/002）③重腿静窗纪律维持。
