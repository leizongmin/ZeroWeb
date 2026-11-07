# R5459 证据：viv 域 float 子树容器行内轴 fill 落码——R5458 机制假设勘误（R4428 无关）＋驱动案 vlr −1.44pp＋corpus 双窗 0 新增红

日期：2026-11-08。基线 HEAD：`738103481`（R5458）。驱动案：`float-lft-orthog-htb-in-vrl-002`＋同构 vlr-002（证据文档沿称 003）。

## 1. 归因定谳：R5458 机制假设勘误

R5458 证据文档第 3 节假设「R4428 `child_viv` 打包排除把 float 子树关出 viv → 交换帧 stretch 关闭 → 塌陷」。本轮 kill-switch 矩阵证伪：

| 变体（探针矩阵，800×600，BASE_CSS＝002 形态） | 修复前 | 修复后 | ZW_VIV_SIZING=0 |
|---|---|---|---|
| V1 纯文本子 | 784/ch=768（fill ✓） | 784/ch=768（不变） | fill 保持 |
| V2 文本+float（002 形态） | **56/ch=40（塌）** | **784/ch=768（fill）** | **仍塌** |
| V3 仅 float | **16/ch=0（塌）** | **784/ch=768（fill）** | **仍塌** |
| V4 块子 | 784/ch=768 | 784/ch=768（不变） | fill 保持 |
| V5 文本+abs（ref 臂形态） | 784/ch=768 | 784/ch=768（不变） | fill 保持 |

`ZW_VIV_SIZING=0` 下塌陷不恢复、V1/V5 fill 保持 → **fill 机制与 R4428 viv 打包排除无关**；塌陷在 taffy same-BFC float 排除本身（`is_in_same_bfc`＝block+非 table+非 abs+非 float+非 scroll 容器，block.rs:732-733）→ 含 float 子树的容器走 float contribution sizing 不 stretch（block.rs:999-1054 slot 分支），viv 域行内轴（物理 h）无任何 fill 臂覆盖。CSS Writing Modes §7.1：块级盒 auto 行内尺寸 = fill 包含块行内尺寸，float 子不阻断（chromium oracle：002 test 臂 .vp h=497 = body content height 精确 fill）。

## 2. 修复：`fill_viv_float_subtree_inline_axis`（ZW_VIV_INLINE_FILL default-on）

- 位置：vertical_block_flow.rs 新函数 + engine.rs 管线 **11.10**（`recenter_abspos_margin_auto_vertically` 后、`compute_final_inline_layouts` 前）——存 IFC 直接见正确 `max_depth`（= 容器 content_height），不依赖 paint Path B width_matches 失配重跑。
- gate：父 vertical + 自身 vertical + display Block/ListItem + height Auto + `viv_subtree_has_float`（非 float 子树已由 taffy stretch 正确 fill，不触碰）+ 非 float/abs/fixed/multicol + fill finite > 0.5。
- action：`fill = parent.content_height − margin_tb`；**only-grow**（`fill > c.height + 0.5` 才 fire——body 784 ≥ html ch 的既有态与 inf 子（R1972 域）不收缩不覆盖）；`c.height = fill; c.content_height = fill − frame_v`＋R1978 `inline_layout_width` 同步。
- 行内轴 fill ≠ float 流几何：R5457 右缘锚定与 R4428 restack 依赖 content_width（block 轴），不受影响（钉断言 float 贴缘保持）。

## 3. 验证

- 常驻钉 3：`r5459_viv_float_subtree_inline_fill.rs`（fill 784/ch=768 + body only-grow + float 贴缘 + 非 float 子树哨戒）＋`..._kill_switch.rs`（独立进程，`ZW_VIV_INLINE_FILL=0` 复原 56/40，R3858 教训）。
- 驱动案 solo：vrl-002 **3.26% → 3.25%**（−0.01pp 持平）；vlr-002 **4.43% → 2.99%**（−1.44pp 实质改善，无翻红）。
- corpus 双窗（同二进制 16594）：w1(ON) **691F** / w2(OFF) **695F**；**P→F = 0**；F→P 4 案（tao-127/146/147/149）全落 table-anonymous-objects 噪声指纹家族（R5457 同族先例），solo×3 全 ✓✓✓ ＝ 新增稳定红 0。
- 内测 reftest **725/725**；fmt/clippy（-p zero-layout-engine --all-targets -D warnings）清零；layout-engine 全腿 25 二进制 0 failed；product-smoke PASS（DC-13 含 @320 结构门）；bench-gate 定向 zero-layout-engine GATE PASS（静窗 loadavg 9.49，25 指标全预算内）。

## 4. 残差与挂账（未了结债）

- vrl-002 残余 3.25% 主体：html shrink-to-fit 含 float 溢出（html w=32 h=219 vs chromium 227×513）→ body/.vp fill 上游值全偏（.vp 784 vs chromium 497）；.vp extent 塌宽（w=16 vs 56，行盒列 40 贡献缺失，block_extent=content_width=0 债）。此两项为下一轮最短路径（html shrink 修一则 .vp 高即回 497 带）。
- R1099 vertical advance 开门（待视觉消费驱动案）；taffy absolute 交换帧 sizing 域（ref 臂 abs w=89 vs chromium 211.48，R5455 家族）；R5457 窄门 FIXME（多列容器块流位/VerticalLr 镜像）。

## 5. 取证命令存根

```bash
# 探针矩阵（V1-V5 ＋ kill 态复原）
./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  cargo test -p zero-layout-engine --test r5459_probe_tmp -- --nocapture
# 驱动案 solo（ON 态）
./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vlr-002
# corpus 双窗（剥 % 后 comm 互翻）
ZW_VIV_INLINE_FILL=0 ./target/release/zero-wpt-runner reftest-upstream
./target/release/zero-wpt-runner reftest-upstream
```
