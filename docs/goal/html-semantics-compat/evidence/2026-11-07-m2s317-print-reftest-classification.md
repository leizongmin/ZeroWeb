# M2 s317 — runner print-reftest 分类增强：the-object-element 簇 2F→4/4 全绿（print 媒体仿真）

日期：2026-11-07。切片号 s317（承 s316 下一步候选：object 簇 for-print 面收口 + 簇余勘根）。

## 勘根/边界确认

- **上游分类规则核源**（sourcefile.py 拉取逐行）：`name_is_print_reftest` = markup_type
  存在且（文件名末 `-` 段 type_flag == "print" 或路径含 `print` 目录分量）——
  `object-image-only-for-print.html` rsplit 后 type_flag="print" → **上游 print
  reftest**（print 媒体仿真跑）。ZW runner 按 screen 跑 → `@media not print`
  隐藏臂把 `.print-only` object 永久隐藏、ref img 照常显绿 → 结构性恒红 ✗1.04%。
- **语料爆炸半径实测**（FILTER=-print 子串扫 70 案 = 66 flag 案 + 4 `.sub.html`
  尾 flag 案）：仅 object 对带 `@media` 分支；其余 68 案 test/ref 双侧媒体无关
  （grep @media 零命中），分类翻转对它们零实效——上游保真统一分类安全。
- **print 媒体近似口径**：上游 print reftest 走分页 A4 渲染；ZW 以 print 媒体
  @ 标准视口渲染（无分页）——driving 对（无分页依赖）像素契约成立；分页依赖案
  见下「假绿暴露」。

## 施工

1. **wpt_file_loader.rs**：
   - `is_print_reftest(relative_id)`：路径含 `print` 目录分量 或 文件名 stem
     `rsplit_once('-')` 尾段（点分 meta_flags 首段）== "print"（上游同规则）。
   - `FileReftestCase.is_print` 字段 + discovery 装配 + `to_config` 置
     `MediaType::Print`（per-case 分类，非全局开关）。
2. **main.rs CLI Option 化**：`--media` 值 `MediaType` → `Option<MediaType>`
   （None = 未显式指定 → per-case 分类生效；Some 时覆盖）。4 使用点同改
   （cmd_reftest / cmd_reftest_upstream / layout-dump / cmd_reftest_oracle——
   oracle 路径 struct literal 后置 if-let）。
3. 单测 1 案：分类矩阵（flag 臂 4 形态含 .sub/.tentative/.xht + print 目录臂 +
   3 反例）。

## 验证

- **the-object-element solo reftest 2F→4/4 全绿**（本切片首个全绿簇）：
  object-image-only-for-print ✗1.04% → **✓0.00%**、object-svg-only-for-print
  ✗1.04% → **✓0.00%**（print 媒体下 `.print-only` object 显影绘图、ref img 同图
  同尺寸）、param-url / display-none-loading 维持绿。日志
  [logs/s317-object-after.log](logs/s317-object-after.log)。
- **-print 全集 70 案翻转矩阵**（screen 基线 [logs/s317-print-screen.log](logs/s317-print-screen.log)
  60P/10F → print 终态 [logs/s317-print-after.log](logs/s317-print-after.log) 55P/15F）：
  - +2 真绿：object 对。
  - **7 案 screen 期「空转假绿」暴露为真红**（block/float/rowgroup/table
    page-break-inside-avoid、break-nested-float-flex-002、grid-fragmentation-001
    .tentative——screen 模式分页惰性双侧一致 → 0.00% 假通过；print 媒体激活
    break-inside 语义后 ZW print pagination 缺口显形 3.13%~13.63%）。
  - 5 案维持红（scope-implicit ×5 2.08% 同值、multicol-height-002、float-2、
    image-srcdoc-relative-uri——非媒体面，既有缺口维持）。
  - 7 案绿值 0.00%→0.29~0.60% 微移（分页激活亚阈差，仍绿）。
- 单测：runner 分类矩阵 1 案全绿。
- clippy（runner）：零告警。
- make test（guarded-test，后台全量）终态 **80 腿 ok + engine lib 3783P/9F**
  （与 s316 终态完全一致——9 键全为已知归因集，runner diff 零新失败）。
  日志 [logs/s317-make-test.log](logs/s317-make-test.log)。

## 跨流移交（记入待决）

**print pagination 缺口 7 键 → 渲染流域**（layout-engine print_pagination.rs 面；
screen 期假绿 = 分页惰性从未被测）：block-page-break-inside-avoid-7（3.84%）、
float-page-break-inside-avoid-5（10.34%）/6（13.63%）、rowgroup-…-avoid-7
（10.99%）、table-…-avoid-4（7.71%）、break-nested-float-in-flex-item-002
（3.13%）、grid-fragmentation-between-rows-001-print.tentative（7.47%）。
全为 break-inside:avoid 分页语义缺口，ZW 媒体口径（print @ 视口、无分页）下
test/ref 双侧分页应一致处 ZW 单侧失效。

## 语义边界（记录）

- print 媒体渲染仍为「print 媒体 @ 标准视口（无分页）」近似——上游分页 A4 口径
  未接（分页归渲染流域，runner 分页口径待其落地后对齐）。
- CLI `--media` 显式值优先级高于 case 分类（Option 化保持既有显式行为）。
- cmd_reftest（DC-14 内部集）与 oracle 路径不经 FileReftestCase.to_config 的
  print 分类（自建 config），行为不变。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：
- s317-print-screen.log（70 案 screen 基线 60P/10F）
- s317-print-after.log（print 终态 55P/15F + 翻转矩阵源）
- s317-object-after.log（the-object-element 4/4 全绿）
- s317-make-test.log（全量回归门禁）

代码锚：tests/wpt-runner/src/wpt_file_loader.rs（is_print_reftest + is_print 字段
+ to_config）；tests/wpt-runner/src/main.rs（--media Option 化 4 使用点）。

注记：本切片 diff 落 wpt-runner 2 文件；engine/css-parser/style-system/layout-engine
零触碰。语料 70 案翻转矩阵为归因证据不另建 skip 条目（渲染流域接分页后自然收敛）。

## 下一步候选

1. 小簇排队：option 1 / iframe 1 / wbr 1 逐簇勘根；br 1 勘根前先 solo 复核
   editing-compat br 刀（fe760dc57）外溢现势。
2. （挂账承前）embed fallback-image 与 print pagination 7 键跨流移交跟踪；
   range-intrinsic-size 移交渲染流域跟踪。
