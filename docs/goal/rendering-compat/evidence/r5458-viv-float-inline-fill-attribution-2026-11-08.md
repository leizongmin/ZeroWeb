# R5458 证据：#33 slice-2 双切片证伪/零驱动回退＋新归因定谳——.vp 行内轴 fill 于 float 子树 viv 排除域塌陷

日期：2026-11-08。基线 HEAD：`faa819317`（R5457）。驱动案：`float-lft-orthog-htb-in-vrl-002`（✓3.26%）＋同构 003（✓4.43%）。本轮两切片均未落码，交付物＝勘误＋机制发现＋新归因。

## 1. 切片 A 勘误：R5457 挂账「行盒 inline 轴 float 排除」前提证伪

R5457 挂账断言「推压机制在但未生效——文字行盒与蓝盒重叠」。本轮像素取证（REFTEST_DUMP PNG，严格暗色过滤 r<110,g<110,b<110 排除 float 蓝盒内白字 AA 干扰）证伪：ZW test 臂黑文本字形已从 y≈50-56 起（float 蓝盒带 y 16..54 **下方**），与 chromium 行盒 y=56 一致——重叠不存在，行盒已让位。按 R5442 判例落码（vertical 列 float 排除后处理）后 solo 002 % 逐位不变（3.26）、kill-switch ON/OFF PNG 逐字节相同（md5 同）→ 无驱动缺陷域，完整回退（不为无缺陷域留投机抽象）。

## 2. 切片 B：R1099 vertical advance 开门——机械生效、视觉零 delta，回退

落码（试后已回退）：`configure_inline_fonts` 单点 `use_advance ∨= vertical_advance_enabled()`（ZW_VERTICAL_ADVANCE=0 kill）＋runtime_flags/mod.rs 包装。机械验证生效：

- 布局期 shaped 度量调用数：002 node 30v1（"VRL"/"parent" run）ON=13 vs OFF=5（ZW_SHAPED_ADVANCE_TRACE）；无 [zw-advance-fallback] 回退。
- 存面片段高实测变化："parent " estimate 113.6 → shaped 79.95（hmtx 真值）。

但 002/003 solo % 逐位不变（3.26/4.43）、REFTEST_DUMP ON/OFF test 臂 PNG md5 逐字节相同 → **视觉零 delta**，按准则回退不落投机变更。零 delta 根因（防后续轮重追）：

1. **垂直字形推进不消费存面片段高**：paint 侧垂直域逐字符 `measure_char_cached` 自测推进（text.rs `char_advance_is_y` 域），字形 y 位与 layout word_height 无关；paint Path B 注入本就无垂直门（`configure_paint_ifc_advance` text_shaping.rs:551 无条件 with_advance_source）。
2. **列断宽度不敏感域**：`break_items_into_columns` max_depth = IFC container_width = 容器 content_height；本案 = 0 → 任意词宽同判不 fit → 逐词断列与 estimate/shaped 无关。
3. text-only measure 臂（inline_finalization:3117）早已 allow=true，R5456 ortho 臂 3236 强制 vertical=false——开门仅新增布局主路径注入。

## 3. 新归因（本轮定谳，下一轮切片）：.vp 行内轴 fill 于 float 子树 viv 排除域塌陷

LAYOUT_DUMP（002，同二进制）：

| 臂 | 盒 | ZW | chromium |
|---|---|---|---|
| test | html | x=768 w=32 h=219 | [572.52, 0, 227.48, 513] |
| test | .vp | x=776 w=16 **h=16 ch=0** | [736, 8, 56, **497**] |
| test | float 子 | x=573 w=211 h=40（R5457 右锚 ✓） | [572.52, 16, 211.48, 40] |
| ref | 同位 div（无 float 子） | x=736 w=56 **h=784 ch=768**（fill ✓） | — |
| ref | span | [744, 56, 40, 40]（= chromium 逐位） | [744, 56, 40, 40] |

- **同构对照**：ref 臂 div（abs+span 子，无 float）行内轴 fill = body content height 768 ✓；test 臂 .vp（文本+float 子）塌 16（仅 border 8×2）✗。chromium float 子不阻断 fill（497 = body content box）。
- **残差链**：.vp ch=0 → 垂直 IFC `max_depth = container_width = 0`（break_items_into_columns:2433）→ 逐词断列（2 列 vs chromium 1 列）＝002/003 可见残差主项；兼 .vp extent 塌宽（w=16 vs 56）上游。
- **机制**：viv 域（父亦 vertical）行内轴 fill 依赖 taffy 交换帧自然 stretch；R4428 `child_viv = viv && !viv_subtree_has_float(child)` 把 float 子树整体关出 viv（swap+stretch+restack 打包排除）→ .vp taffy 原始取向，auto 物理 h = 流内子高 Σ = 0（float 排除、行内文本非 taffy 子）→ h=border。R4446 fill 臂仅正交域（parent htb）且显式排除 float 子树（clearance-calculations-vrl-008 翻红先例）；R4439 fill 臂仅 abs 容器。
- **下一轮切片**：viv 域 float 子树容器的行内轴 fill。须拆 R4428 打包排除中 swap/stretch 与 float restack 的语义（fill 容器自身行内轴 ≠ 动 float 流几何），或 postprocess 单臂对 viv 域 float 容器 fill；clearance-calculations-vrl-008 / ortho-htb-alongside-vrl-floats 族回归风险需全量 corpus kill-switch 双窗 A/B 判定。备选：taffy absolute 交换帧 sizing 域（ref 臂 abs w=89 vs chromium 211.48，R5455 家族）。

## 4. 取证命令存根

```bash
# solo（两态 % 逐位不变 3.26/4.43）
./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
# ON/OFF PNG 逐字节对比（REFTEST_DUMP 落 target/reftest-dump/；REFTEST_DUMP_PASS=1 才存通过案）
REFTEST_DUMP=1 ./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
# shaped 度量计数差（ON 13 / OFF 5，node 30v1）＋estimate 宽（OFF 态 [zw-advance-fallback]）
ZW_SHAPED_ADVANCE_TRACE=1 ./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
# 布局 dump（.vp h=16 ch=0 vs ref 臂 784/768）
LAYOUT_DUMP=1 ./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
```
