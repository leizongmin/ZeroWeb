# R5462 证据：正交块级 htb 子 shrink-to-fit §7.1 重解——016 双案 20.95%/20.75%→0.00% 精确绿，vlr 族全绿，corpus 双窗回归 0 净 −2F

日期：2026-11-08。基线 HEAD：`fe9682b73`（R5461）。驱动案：`sizing-orthog-htb-in-vrl-016` ＋同构 vlr-016（R5461 收尾时「余账稳定红择优」第一名，同 writing-modes 族最大切片）。

## 1. 归因：taffy 交换帧把父当前收缩宽当正交块级行内约束，块轴 extent 塌 ICB 高 fill

R5461 收尾挂账开案（vrl-016 ✗20.95%，orthog-htb-in-vrl 族最大单案切片）。机制分解（探针矩阵六变体，临时已删）：

- **宽度塌陷**：taffy 交换帧对块级正交子的测量闭包以**父当前收缩宽**（循环首趟 cb 内容 60/20）当子行内约束 → 文本按父宽断行，子盒宽 = 断行 extent + frame（100 数塌 130 vs chromium 780）。与 R5461 abs 叶同病理（物理高反馈成水平约束的镜像形态：这里是父收缩宽反馈），块级域 R4437/R5456 收窄先例的同债。
- **块轴塌陷**：taffy 以物理行内轴当块轴分配 → 正交块高塌 ICB fill（784 vs 真实 Σ 行高 130）。R5460 viv 容器臂的同族域。
- **chromium oracle**（headless=new，.xht XML + getBoundingClientRect 注入 document.title 零流内污染）：`OBX=0 OBY=8 OBW=780 OBH=130`——3 行 × 20 + 70 border，vrl 块流起始右缘 780。
- **STF 约束轴定谳**：WPT ref 臂 `width:100vw + box-sizing:border-box` 渲染 = 视口宽 → **约束落 border-box**（正交盒 border-box = 视口宽、content = 730）；首版实现 available = ICB − margins 不扣 frame 得 w=870 即被 oracle + ref 臂双证否决，修正为先扣自身 frame 再与 content 基 intrinsic 比较（CSS Writing Modes §7.1 orthogonal-auto × css-sizing-3 §5 shrink-to-fit）。

## 2. 修复：`orthog_block_inline_axis_stf`（ZW_ORTHO_BLOCK_STF default-on）

- 位置：vertical_block_flow.rs 新 walk + engine.rs 管线 **11.12**（viv_mixed_content_block_extent 后、compute_final_inline_layouts 前——content_width 先落定使存储 IFC 以 stf 断行；height 须 pass 自写，compute_final 空容器守卫不回写非空盒高）。
- gate（最小切片）：块级（display Block/ListItem）；宽/高双 Auto；父 style 宽 Auto（**indefinite CB** 域，§7.1 约束 = ICB；definite-CB 另账 FIXME）；非 abs/fixed/multicol/float（float 域 R5456、abs 域 R5461 已修）；DOM 无元素子（纯文本叶；元素子域需子树重排 FIXME）；父非 vertical 不触。
- action：`available_border = ICB − margin_l/r`（resolve_margin_px，Px only，% FIXME）→ `available = available_border − frame_h`（border-box 约束口径）；`stf_content = min(maxc, max(minc, available))`（intrinsic_sizing content 基）；水平 IFC `with_vertical(false)` 以 stf_content 断行 → `width = stf_content + frame_h`、`height = Σ 行高 + frame_v`；min/max-width Px 钳（5.5 口径）；width_changed → c.width/content_width 写回 + `x −= delta`（按**父 WM** VerticalRl 判向——子恒 htb，右缘保持）；height_changed → c.height/content_height 写回。

## 3. 验证

- 常驻钉 5＋kill-switch 锚 1：`r5462_orthog_block_stf.rs`（FULL 驱动形态 800×130/cw=730/ch=60/右缘 130 保持；SHORT 内容响应 336×90 单行；vlr 域 x=0 左缘保持；显式宽 470 不越权；definite-CB 域 90 不触；float 域 126 不触）＋`..._kill_switch.rs`（独立进程 set_var 模式，`=0` 复现旧塌陷 **130×784**＝旧行为失败证据，R3858 教训）。
- chromium oracle 对照：ZW 修后 800×130（border-box）vs oracle 780×130（其字体度量 770+10 差域）；self-source 双臂同字体，绝对宽差不进判定。
- 驱动案 solo×3 全稳 **✓0.00% 精确绿**：vrl-016 **20.95% → 0.00%**；vlr-016 **20.75% → 0.00%**。OFF solo 复红 20.95%/20.75%（kill-switch 因果实锤）。
- **族效果**：orthog-htb-in-vlr 族 **100.0% 全绿**（本轮首个全绿族）；orthog-htb-in-vrl 族 94.7%（仅 sizing-007 ✗5.02% definite-CB 域在册红）。连带改善无翻红：vrl-001 4.00→2.10、003 1.58→1.03、004 4.78→1.84、006 1.12→0.22、013 4.23→2.16、015 1.40→0.62、018 1.01→0.22。
- 内测 reftest **725/725**；fmt 清零；clippy（-p zero-layout-engine --all-targets -D warnings）清零；layout-engine 全腿 **lib 2444P/0F/2I ＋ 31 测试二进制 0F**（含新钉 6 测；临时 probe 提交前已删）。
- corpus kill-switch 双窗（同二进制 16594，env 经 make→test-guard 透传）：w1(ON) **690F（95.8%）** / w2(OFF) **697F**——comm 互翻（剥 ✗ % 后）：ON 独红 **0＝净回归 0**；OFF 独红 7 → solo 复核定谳：双 016 OFF 复红（20.95%/20.75%）＝真实修复 **净 −2F**；aspect-ratio-024 ＋ table-anonymous-objects-127/147/150/002-dynamic 五案 OFF solo 全绿＝全量窗噪声指纹家族（R5457/R5460/R5461 同族先例），非本轮效应。
- product-smoke PASS（welcome/morning struct-check 双站）；bench-gate 定向 zero-layout-engine 静窗（1min load 2.48 入窗）**GATE PASS 25 指标（NEW=0）**，morning total 53.21ms（基线 50.02，预算 2000ms）。

## 4. 残差与挂账（未了结债）

- **definite-CB 域**（父 style 宽 Px）：§7.1 约束 = 父 content 宽非 ICB，本切片排除；探针 VD 证该域仍塌 20px 断行。在册红 sizing-orthog-htb-in-vrl-007 ✗5.02%（007..012 批次）归此域，**R5463 首选**。
- **显式宽正交子的内容断行塌陷**：探针 VC 证 width:400px 时子盒宽不越权（470 正确）但内容仍按 60px 断行（cw=60/ch=714）——known_width 反馈污染断行的子域，FIXME。
- **inline 元素子 / DOM 有元素子**：需子树重排，FIXME（R5461 abs 元素子域同债）。
- **content 关键字宽**（fit-content 等）物宽门排除：FIXME。
- % margin resolve_margin_px 返回 0：FIXME（有驱动案再收）。
- **维持**：c5525-fltmrgn-000 ✗3.92%；sizing-orthog-htb-in-vrl-007 ✗5.02%＝在册稳定红（definite-CB 域，见上）；正交 inline-block 测量域（R4437 收窄面）；R5447 collapse 显式臂 ½border 同口径核查；㉗-㉘/R5432 销账面/deep intrinsic 48 变体矩阵/**待用户决策清单**①-⑤（无新增）；对端 js_dom_bridge 红腿观察（九案）；deep_nesting 基线漂移观察；R4213 行组参照系再对齐未镜像（FIXME）；inline_finalization 超 2000 行拆分挂账；revert-layer-014 构建/环境时代漂移观察；R5457 窄门 FIXME（多列容器块流位随行号左移/VerticalLr 镜像）；R1099 vertical advance 开门待视觉消费驱动案；definite-height 混合内容容器（探针 H 域）；html root sizing 链低优先。

## 5. 取证命令存根

```bash
# 驱动案 solo（ON 态 / OFF 态）
make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-016
ZW_ORTHO_BLOCK_STF=0 make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-016
# chromium oracle（.xht XML + title 注入，零流内污染）
google-chrome --headless=new --dump-dom file:///tmp/r5462-oracle/probe.xht | grep -oE 'OB[XYWH]=[0-9.-]+'
# corpus 双窗（剥 ✗ % 后 comm 互翻）
make reftest-upstream 2>&1 | grep '^  ✗' | sed 's/ ([0-9.]*%)$//' | sort > /tmp/w_on.txt
ZW_ORTHO_BLOCK_STF=0 make reftest-upstream 2>&1 | grep '^  ✗' | sed 's/ ([0-9.]*%)$//' | sort > /tmp/w_off.txt
```
