# R5461 证据：正交 htb abs 叶 shrink-to-fit 真CB重解——002 残余 1.39%→0.58% / vlr 1.24%→0.26%，corpus 双窗 0 新增红净 −1F

日期：2026-11-08。基线 HEAD：`2d0e74743`（R5460）。驱动案：`float-lft-orthog-htb-in-vrl-002` ref 臂 abs＋同构 vlr-002。

## 1. 归因：taffy 交换帧 absolute sizing 两趟把物理高反馈成水平约束

R5460 挂账「002 残余 1.39% 主体：ref 臂 abs 宽 89 vs chromium 211.48（R5455 家族）」开案。机制分解（探针矩阵五变体，临时已删）：

- **塌陷链**：pass1 R5456 正交测量臂以静态父 content 区测得 `(taffy w = 物理块轴 extent 40, taffy h = 物理行内 advance)`（交换帧转置）；随后 R5293 纯块级定高回灌臂（taffy-local block.rs `height_content_derived`，文本叶无 taffy 子恒真）以 `known_width = Some(已测 taffy w)` 重排——该值是**物理高 40**，经测量闭包被 R5456 臂当水平 IFC 约束消费（交换帧下 taffy width 槽 = 物理块轴，槽语义劈叉）→ 文本按 40px 断行，行内 extent 塌最宽词。
- **内容无关实证**：单词「floated」与 16 字符全文塌后同宽 123.2（estimate 域）＝宽度不是 shrink-to-fit；真 CB=ICB 可用 784 ≥ max-content 应单行。
- **taffy 侧无解**：taffy CB=静态父（物理水平可用仅父内容宽 40）；真 CB=ICB 须按 §10.3.7 重解（R5455 家族同域：x 已由 11.5 右锚方程随 width 修正，宽度域本轮收口）。

chromium oracle（headless=new，.xht XML + getBoundingClientRect 注入 document.title 零流内污染）：`orthog-horiz` = **x=552.52 w=211.48 h=40**——单行、右缘 764（=ICB−16−chrome 壳滚动条带）、shrink-to-fit 宽（其字体度量；ZW 字体 max-content 262.4，self-source 双臂同字体不进判定）。

## 2. 修复：`resolve_orthog_abspos_shrink_to_fit`（ZW_ABSPOS_ORTHO_STF default-on）

- 位置：abspos_sizing.rs 新 walk + engine.rs 管线 **11.4b**（fix_rtl_abspos_static_position 后、11.5 视口重解析前——右锚方程消费修正后 width）。
- gate（最小切片）：viewport-CB abs（镜像 11.5 `is_abs_viewport_cb` 谓词链：父链 positioned 或本盒 is_abspos_cb 均排除，两 pass 判定逐位一致）；父 vertical + 自身非 vertical（交换帧域；水平域 R5104 臂 known.w=物理宽语义自洽不触）；非替换；物理宽 Auto；水平/垂直双 inset 均不全定长（stretch 域让位）；DOM 无元素子（纯文本叶；元素子域 taffy 子树同病理需子树重排，FIXME）。fixed 与 containment/视觉-CB 域不在本切片（FIXME）。
- action（CSS2 §10.3.7）：available = ICB 宽 − 定长 left/right − 定长 margin_l/r（auto/% 不计）；水平 IFC（R5456 臂同取向 with_vertical(false)）以 available 断行 → 单行 width = ceil(advance)（R5104 paint 复排口径：盒宽 ≥ advance 免 Path B 复排折行）、多行保持测量值；height = Σ 行高 + frame（显式 height 不越权）；content_* 同步；min/max-width Px 钳（5.5 口径）；|Δw| ≤ 0.5 零扰动。

## 3. 验证

- 常驻钉 4＋kill-switch 锚 1：`r5461_orthog_abs_shrink_to_fit.rs`（单行 263×40/content_height 同步/无框盒 content_width 同步；shrink-to-fit 内容响应「floated」124 vs 全文 263；嵌套 CB 域哨戒维持 taffy 既有 123；显式宽 300 不越权；双 inset stretch 域维持 784）＋`..._kill_switch.rs`（独立进程，`=0` 复现旧塌陷 123＝旧行为失败证据，R3858 教训）。
- chromium oracle 对照：ZW 修后 abs 263×40 单行右缘 784（=ICB−16 精确）；oracle 552.52/211.48/40 同形——宽度绝对值差 = 双方字体度量域，self-source 双臂同字体不进判定。
- 驱动案 solo×3 全稳：vrl-002 **1.39% → 0.58%**（−0.81pp）；vlr-002 **1.24% → 0.26%**（−0.98pp）；同族 float-rgt vrl-003 **4.43% → 3.68%**、vlr-003 2.17%（连带改善，无翻红）。
- 内测 reftest **725/725**；fmt 清零；clippy（-p zero-layout-engine --all-targets -D warnings）清零；layout-engine 全腿 **lib 2444P/0F/2I（基线逐位同）＋tests 30 二进制 0F**（含新钉 5 测与临时 probe 提交前删除）。
- corpus kill-switch 双窗（同二进制 16594，env 经 test-guard 透传）：w1(ON) **693F** / w2(OFF) **694F**——P→F 2 案（table-anonymous-objects-149/151）＋F→P 3 案（127/129/145）全在册 table 匿名对象噪声指纹家族（R5457/R5460 同族先例），movers 5 案 solo×3 全 ✓✓✓（0.00% 精确）＝**新增稳定红 0、净 −1F**。
- product-smoke PASS（welcome oracle diff 17.23% ≤20%＋struct 六站＋form-input-perf-gate）；bench-gate 定向静窗（1min load 2.48 入窗）**GATE PASS 25 指标（NEW=0）**。

## 4. 残差与挂账（未了结债）

- 002/003 残余 0.26~0.58%：进入亚像素/双臂共享债带（.vp 784 vs chromium 513/497 html root sizing 双臂自洽零 reftest 影响；字体度量域绝对宽度差不进 self-source 判定）。无新增可见切片。
- **元素子域 abs（DOM 有元素子）**：taffy 子树同病理（known_width 反馈污染子树断行），需子树重排，本轮 FIXME 不触；有驱动案再收。
- fixed 定位与 containment/视觉-CB 域：不在本切片（FIXME 有驱动案再探）。
- content 关键字域（fit-content 等）物宽门排除：FIXME。
- **维持**：c5525-fltmrgn-000 ✗3.92%；sizing-orthog-htb-in-vrl-007 ✗5.02%/016 ✗20.95%（vlr 同 016）＝在册稳定红，非本轮域；R5447 collapse 显式臂 ½border 同口径核查；㉗-㉘/R5432 销账面/deep intrinsic 48 变体矩阵/**待用户决策清单**①-⑤（无新增）；对端 js_dom_bridge 红腿观察维持（九案）；deep_nesting 基线漂移观察；R4213 行组参照系再对齐未镜像（FIXME）；inline_finalization 超 2000 行拆分挂账；revert-layer-014 构建/环境时代漂移观察；正交块级/inline-block htb 叶旧 vertical 测量路径 FIXME 维持；R5457 窄门 FIXME（多列容器块流位随行号左移/VerticalLr 镜像）维持；R1099 vertical advance 开门待视觉消费驱动案维持；definite-height 混合内容容器（探针 H 域）维持；html root sizing 链低优先维持。

## 5. 取证命令存根

```bash
# 驱动案 solo（ON 态）
./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vlr-002
# chromium oracle（.xht XML + title 注入，零流内污染）
google-chrome --headless=new --dump-dom file:///tmp/r5461-oracle/probe.xht | grep -oE 'ABS[XYWH]=[0-9.-]+'
# corpus 双窗（剥 ✗ 后 comm 互翻；env 经 test-guard 透传）
ZW_ABSPOS_ORTHO_STF=0 ./target/test-guard --time-limit 3600 -- ./target/release/zero-wpt-runner reftest-upstream
./target/test-guard --time-limit 3600 -- ./target/release/zero-wpt-runner reftest-upstream
```
