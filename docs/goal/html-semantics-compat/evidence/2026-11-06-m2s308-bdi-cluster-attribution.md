# M2 s308 — the-bdi-element 13 红页簇归属勘根：根因全在 layout-engine inline bidi 面 + style-system UA hints，整簇移交渲染流域（本流零施工）

日期：2026-11-06。切片号 s308（承 s307 下一步候选①，勘根-移交型切片）。

## 勘根

**簇面**：the-bdi-element 目录 16 对入通道（13✗ + 3✓），红页 1.05%–4.39%
小差幅——失败面局部化（非整幅缺失）。页面同构：test 用 `<bdi>`（默认
`unicode-bidi:isolate` + 隐式 `dir=auto`）包裹混排内容，ref 用 LRO/PDF
（U+202D/U+202C）控制符序列逐行模拟隔离语义；含 6 行结构页（3 LTR 段 +
3 RTL 段）与 5 行变体。HTML spec：bdi 按 U+FFFC OBJECT REPLACEMENT
参与段落级 UBA（isolate 语义，BD14-16/X9 + 隐式边界终止）。

**像素取证**（`REFTEST_DUMP=1 make reftest-upstream
FILTER=the-bdi-element` 全簇 solo dump + 逐列 diff）：

1. **LTR 段行逐列像素零差**：bdi-neutral-to-letter-following-1 行 1-3
   （`dir="ltr"` 容器）test 与 ref mismatch columns = 0——**bdi 隔离在
   LTR 段已正确**；该页全部 1.20% 差幅来自行 4-6（RTL 段）。
2. **RTL 段行两侧均错且互为镜像**：行 4 test 实测 `[a] < ...b`（bdi 块
   排行首、`>` 镜像为 `<`）；同页 ref（我们引擎渲染的 LRO 序列）实测
   `b... > [a]`——恰为 test 的水平镜像反序。spec 期望两者同为
   `...b < [a]`（省略号行首、`<` level 2 不镜像）。**两侧都是我们引擎
   渲染、两种路径输出互为镜像、且都不是 spec 序**。
3. **全行红页**（missing-pdf/nested/to-surrounding-run 5 行全错）：
   机制含 isolate 边界控制符作用域终止——missing-pdf 的 bdi 内未闭合
   RLO 必须在 isolate 边界隐式终止（否则越界翻转后继 `de`→`ed`），
   需要 isolate 显式 level 映射（X5a-c/X6/X7/X9）。
4. **dir=auto 页专属**（bdi-auto-dir-default）：三个 bdi 需逐个 first-strong
   定向（`[:)]` 无强字符→LTR；`[+- a ב]` a 先现→LTR；`[1 ג d]` ג 先现
   →RTL，ref 侧内容反序 `[d ג 1]`）。
5. **换行交互**（bdi-neutral-wrapped）：test 7 行 vs ref 8 行——isolate
   run 断行分歧。
6. **绿页 3**（`-2` 变体 letter/number-following-2、preceding-2）：bdi
   邻接强字符无中性跨界面，per-run 路径输出与 LRO ref 像素一致
   （✓≤strict）——负样本面佐证失败条件 = 中性字符跨界/组序/作用域终止。

**根因层（全部落在渲染流域 crate，引擎/dom 零足迹）**：

| # | 根因 | 位置 | 域 |
|---|------|------|----|
| ① | 段落级 UBA isolate 回退门：含 `is_isolated_bidi` run 的 IFC 整体回退 per-run 路径——跨 run 中性解析与 isolate 边界作用域终止不可达（R5337 保守设计，头注释明言「isolate 显式 level 映射前保守不混排」） | layout-engine/src/inline/para_bidi.rs:77（门）+ 头注释 FIXME | 渲染 |
| ② | per-run 路径无行级跨 run 组序重排：RTL 段 run 块序不按 level 反转（实测 test 侧 bdi 块排行首） | layout-engine/src/inline（per-run 校准路径） | 渲染 |
| ③ | ref 侧 LRO 混合内容行级 L2 与 spec 偏差：ref 实测 `b... > [a]` ≠ spec `...b < [a]`（para_bidi 精确性守卫「混合 level 片段保守维持逻辑序」未覆盖/偏离此形态） | layout-engine/src/inline/para_bidi.rs（行级 L2） | 渲染 |
| ④ | `dir=auto` first-strong 未实现：UA hints "auto" 臂仅设 `unicode-bidi:isolate`，direction 恒承父（代码注释自认 first-strong 缺） | style-system/src/lib.rs:3237 附近 UA 表 | 渲染 |
| ⑤ | isolate run 断行交互（换行数 7 vs 8） | layout-engine/src/inline/break_lines.rs 面 | 渲染 |

**归属判定**：整簇移交**渲染流域**。layout-engine、style-system 均属
渲染流 crate 清单（rally 规则 9 工作面不重叠），且渲染流当前活跃于
layout-engine（R5434 col used width，d8f313bfc）。移交落点 =
`docs/goal/rendering-compat/box-level-bidi-spec.md`（其 P1「inline-box
bidi level」项 = direction/unicode-bidi normal/embed/isolate/override
显式 level 分配——即本簇根因①④的正解路径；该 spec RFC v0.1 未实施，
本账本为其补充实证）+ 渲染流控制面。

## 施工

零（勘根-移交型切片；并行规则工作面不重叠，本流不触 layout-engine/
style-system）。

## 验证

- 全簇 16 对 `REFTEST_DUMP` solo 取证（13✗ dump 成功；3✓ 未 dump 与
  s307-full-tree 计数一致）。
- bdi-neutral-to-letter-following-1 行级逐列 diff：行 1-3 = 0 mismatch
  columns；行 4-6 = 89 mismatch columns/行（RTL 段）。
- bdi-neutral-separate 盒 offset 对齐全行差（单行页行带配对法不适用，
  已盒对齐复验）。
- 源码锚：`grep uniform_resolved_rtl|uba_rtl_enabled`（text_shaping.rs
  片段级 RTL shaping 既有面）；`para_bidi.rs` 头注释 isolate 回退门自述；
  UA 表 "auto" 臂 first-strong 缺自述。
- 全引擎 `dir=auto|first_strong|DirAuto` grep 零命中（④实锤）。

## 证据

产物（target/reftest-dump/，会话级不入库）：全簇 26 PNG（13✗ test/ref
对）+ 行级裁剪对照图 bdi1-l{1,4}-{t,r}.png。日志：本切片零 runner 落盘
（solo 复跑口径同 s307-full-tree.log 5df3a83127a22374aa32d385bb19ac3c
的 13✗+3✓ 计数）。

代码锚：layout-engine/src/inline/para_bidi.rs（段落级 UBA + isolate
回退门）；inline/collect_items.rs:644（is_isolated_bidi 旗标）；
engine/paint/painter/text/text_shaping.rs:634（片段级
uniform_resolved_rtl 消费）；style-system/src/lib.rs:3237（UA 表
dir="auto" 臂）。

## 下一步候选

1. ol 6 红页簇勘根（小簇最大头，本流域内）。
2. 小簇排队其余（textarea 4/meter 4/input 3/link 3/embed 3/object 2/
   option 1/iframe 1/br 1/wbr 1）。
3. 生产通道动画装载接线（s307 ③，engine 网络路径
   decode_image_animation——生产 img 动画 GIF 维持首帧静态）。
