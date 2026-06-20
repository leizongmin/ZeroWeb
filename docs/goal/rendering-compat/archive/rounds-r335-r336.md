# 归档：R335–R336 轮次详细记录

**归档日期**：2026-06-22（R397 doc-maintenance 治理轮）
**归档原因**：R335/R336 全文详记已超出 master.md「最近 20 轮」窗口（当前 R396，窗口 = R377–R396），按文档治理规则迁出。逐轮结论摘要仍保留在 master.md 顶部「综合裁决」表（WM-1 / abspos 文本相关行）。

> 本文件为 master.md `## 最近轮次详细记录` 节的历史迁出，只追加、不修改。回溯 R334 及更早轮次见同目录其他归档（[`rounds-r314-r334.md`](./rounds-r314-r334.md) 等）。

---

### R335 — paint-IFC per-fragment 颜色探针 net-negative：WM-1 真阻塞 = Phase A 双路径终局确认（探针已回退，基线持平）

**承接**：R334 收尾 CONTINUE 指向探查 paint-IFC per-fragment 颜色修复可行性。R334 定位 WM-1 cluster（abs-pos-non-replaced-vrl/vlr）真阻塞 = 绿色 "X" glyph 完全未绘制（paint-IFC 用容器 `color:transparent` 绘全部 inline 子树，per-fragment 颜色覆盖仅 multicol 分支 text.rs:1028）。本轮实证该修复是否可 bounded 落地。

**探针实现（已 100% 回退）**：非多列 `render_fragment!` 宏（text.rs:1124，覆盖 use_stored + 非存储两 fragment 循环）镜像 multicol 分支（text.rs:1019-1032）——解析 fragment 所属 inline 元素（文本节点取父元素）的 color，fallback 容器 color；glyph add 改用 frag_color。生产 caller（mod.rs:263/452）传 `Some(styles)`，per-fragment 解析可生效。

**实证（abs-pos-non-replaced 子集 loose，探针 ON）**：**loose 14/14 → 12/14 净 -2**，多数 case diff 上升 1-1.5pp（vrl-002 1.33→2.67、vrl-012 3.67→5.00、vrl-130 5.03→6.33 新 loose FAIL、vlr-163 5.40 新 loose FAIL）。

**根因（Phase A 双路径，R334 推断实证确认）**：green X 现被 div 的 **paint IFC** 在 **paint-IFC 位置**（normal-flow，"X" 紧跟 "1 2 34"）绘制，≠ ref 期望的 **abspos 静态位置**（fix_vertical_mode_abs_pos 计算，R334 实证 vrl-012 y=80 正确）= goal doc gap #4 **Layout/Paint IFC 双路径**。per-fragment 颜色虽隔离正确，激活了**错误路径**（paint-IFC）绘制→离 ref 更远→diff 上升。

**裁决**：① per-fragment 颜色探针 **net-negative，已 100% 回退**（git checkout text.rs，子集复测 14/14 loose 恢复），avenue 关闭。② **WM-1 真阻塞 = Phase A 双路径**——R334（positioning）+ R335（color）两角度均收敛至此。per-fragment 颜色须先统一 layout/paint IFC（paint 复用 layout 存储行盒/abspos 位置）才能安全应用。③ WM-1 单会话 lever 彻底穷尽，剩余 forward motion = Phase A IFC 统一（多会话硬里程碑）。

**Phase A 设计补充**：paint IFC 把 abspos 后代的 inline 文本当正常流绘制（位置错）+ 抑制 abspos span 自身 paint_text 的 green 输出 = Phase A 须解决的具体机制之一（区别于 large-font 的 font_size 存储、welcome 的度量分歧）。建议 Phase A 设计文档补此表现。

**代码变更**：零（探针已回退，`git diff -- '*.rs'` 空）。基线 loose 438/490 / strict 295/490 持平。

### R336 — abspos 文本抑制机制精确定位 + refined skip 探针 net-neutral：WM-1 Phase A 第三角度确认（探针已回退，基线持平）

**承接**：R335 收尾 CONTINUE 指 per-fragment 颜色（R335 已证伪）。Phase A 设计文档 v1.2（R306）已证伪 baseline-alignment Wall ③ 为阻塞点，故本轮**不 pursuing baseline**，换第三角度——abspos 文本抑制机制——做实现轮。R334/R335 已定位 WM-1 green "X" 未绘制，本轮精确定位抑制源 + 测 refined skip。

**抑制机制精确定位（探针 PROBE_ABSPOS，env-gated）**：插桩 abspos span paint_text（text.rs:679）实证——span 的 paint_text **被调用**（fs=80、color=green、content_w=80、has_direct_text=true 全正确），但 **painted_contains=TRUE** → text.rs:690 守卫 `fragment_node_ids.is_none() && painted_inline_nodes.contains(&node_id)` return → **span 自身绘制被抑制**。探针 div IFC fragments：node_id=34(span) text="X"。**collect_inline_items（inline/mod.rs:1066）对 inline 元素用 `doc.text_content(child_id)` 收集文本，node_id=child_id=span，不检查 position**——abspos span 的 "X" 被收入 div IFC，render_fragment!（text.rs:1125）insert span 进 painted_inline_nodes → 抑制 span 自身 paint。CSS §9.8：abspos out-of-flow，文本不应参与父容器 inline 流——当前违反。

**refined skip 探针（已回退）**：非多列 fragment 循环加 skip——owner 为 abspos/fixed **且 owner≠self**（abspos 元素自身绘制时不跳过）的后代文本跳过。探针实证：div(box 32) 绘 "X"(owner=span≠div) → skip=true（正确）；span(box 34) 绘自身 "X"(owner=span=self) → skip=false（正确）；span painted_contains 翻 **false**（抑制解除）。**但 span IFC fragment fs=16**（paint-IFC 空 styles 默认，非 80）= Layout/Paint IFC double-path（gap #4）。

**净效应**：子集 loose 14/14 持平（vrl-002 1.33→1.28 微变）、strict 0/14 持平、全量 loose **438/490 持平** = skip **net-neutral**。即便抑制解除，span 自身 paint-IFC 因空 styles 产出 fs=16，green "X" 仍错——suppression 修复须**同时**解 double-path 才生效 = Phase A 整体。

**裁决**：① refined skip **net-neutral，已 100% 回退**（git checkout text.rs，基线恢复）。② collect_inline_items 不排除 abspos 是**有意为之**——layout 侧 fix_vertical_mode_abs_pos 依赖 IFC fragment 算 abspos 静态位置，故不能 collect 层排除；paint 层 skip 又受 double-path 阻塞。③ **WM-1 Phase A 第三角度确认**：R334 positioning → R335 color → R336 suppression，三角度一致指向 Layout/Paint IFC 双路径。WM-1 单会话 lever 彻底穷尽（三角度闭环）。

**Phase A 设计补充（v1.2 之上）**：WM-1 精确表现 = (a) paint IFC 把 abspos 后代 inline 文本当正常流绘制（位置错）+ 标记 painted_inline_nodes 抑制 abspos 自身 paint_text；(b) 即便解除抑制，abspos 自身 paint-IFC 空 styles 产出错误 font_size。Phase A 须同时：paint 复用 layout 存储的 abspos 位置/font_size + 容器不绘 abspos 后代文本。两者均非单点。

**代码变更**：零（探针已回退，`git diff -- '*.rs'` 空）。基线 loose 438/490 / strict 295/490 持平。
