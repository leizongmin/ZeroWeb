# R5317 — va:text-* 原子盒 margin-box 围合 + 基线锚定 + img 原子 border-box 口径

- **日期**: 2026-10-29
- **驱动案**: `css/CSS2/css1/c44-ln-box-001.xht` ✗21.12% → **✓0.00%（像素级精确）**
- **corpus**: 825F → **824F**（16592 案，FIXED=1，REGRESSED=0；一次全量捕获 content-none-select-1
  +1 红经隔离 ✓0.00% + 全量复跑 824F 定谳负载脆弱非本轮）
- **R5316 挂账收口**: 「原子 text-* 行盒围合域」本轮关闭

## 定性

案形（c44-ln-box-001，chromium 对端模型已像素级验证）：div `font:10px/20px Ahem`，
img `width/height:3em; padding:.5em; border-width:1em; margin:1.5em`，va text-bottom/text-top。
strut 盒 [3,23] 基线 13，A_parent 8 / D_parent 2。chromium 判例：

- **行盒围合 = margin box 并集**：img1（text-bottom）mb 底锚 `B_pre + D_parent = 15` →
  [−75,15]；img2（text-top）mb 顶锚 `B_pre − A_parent = 5` → [5,95]；与 strut 并集 →
  行盒 **170**、扩展后基线 **88**（13+75）。
- **原子盒基线锚定**：img1 border-box 顶 = B+D_parent−mb_h+mt = **15**；img2 = B−A_parent+mt
  = **95**；后续行 restack（行 2 y = 20+170 = **190**，glyph 行 218-227）。
- **img 原子参排尺寸 = border box**（60×60 = content 30 + padding 10 + border 20）。

ZW 缺口四处：① R822 扩展原子 text-* 臂用 half_leading 校准不生长行盒（行盒 110）；②
原子 bot 合围缺席；③ 存面/测量 build 原子尺寸用 CSS content（30×30）与 layout 树 taffy
border-box（60×60）分裂——存面行盒 110 vs 170、glyph 行 y93 应 123；④ img 后文本 advance
少 padding+border（css-text-3 §4.1 空白序列跨原子盒被吞，x−10）。

## 实现（kill-switch `ZW_VA_TEXT_BASELINE_PIN` / `ZW_IMG_ATOM_BORDER_BOX`，均 default-on，`=0` 回退）

1. **R822 扩展原子臂**（inline/mod.rs）：text-* 原子（font_size=0）top 需求
   TextBottom = `max(0, −(B_pre + D_parent − mb_h))`、TextTop = `max(0, A_parent − B_pre)`，
   与 half_leading 校准值取 max；文本 run 臂逐字节不变（R5316 已收口域零扰动）。
2. **bot 合围原子臂**（PRE-extension 坐标系）：`va_bot_need = max(text-top 盒底,
   text-bottom 盒底钉位)`（top_extend 在差式中消去，不预减）。
3. **run.y 基线锚定**（Phase 3 后 fixup）：TextBottom `run.y = B_final + D_parent − mb_h +
   mt`、TextTop `run.y = B_final − A_parent + mt`。
4. **restack**：`va_lifts` 逐行累积（top_extend+bot_extend），与 R5298 shifts 合并循环施加
   于后续行 y。
5. **img 原子 border-box 口径**（collect 原子分支 + measure inline_block_sizes 表）：
   content 尺寸 + `img_atom_border_box_extra`（padding+border；`box-sizing:border-box`
   声明豁免；border-style none/hidden 侧宽不计；stored 表已给 taffy border-box 时
   `stored_inline_size.is_none()` 门防重补；measure 表 img 任一侧 auto 不入表防
   「纯 padding+border」伪尺寸覆盖 attr/固有回退）。
6. **原子盒中断空白序列**（break_lines InlineBlock 臂，css-text-3 §4.1 + §4.2）：
   `last_ended_collapsible_ws = false`——「a <img> b」双空格分居原子两侧均存活。

**路径曲折（复用价值）**：collect 有两条原子推进入口——img 专名臂与元素臂（is_replaced_inline
∪ inline_block_sizes 命中 ∪ display inline-block-*）。img 的 UA display 即 inline-block →
恒走元素臂，img 专名臂对本域为死代码。首轮修复落在专名臂 + measure 表（同误判「Path B 空
styles 走该表」——实证水平域存面 build 传 `HashMap::new()`，见 inline_finalization builder
`if is_vertical` 分支），`[ibA-diag]` 插桩三轮才定位：**存面 build 原子尺寸真源 = 元素臂
CSS 解析**（无表时）。教训：多入口共用语义域的修复，先插桩证可达再落笔。

## 交付

- corpus 同树 A/B（基线 R5316 交付态 825F fail-list）：**824F**，FIXED=1（c44-ln-box-001
  ✓0.00% 像素级：glyph 行 L0 y28 / 原子行 y123 x{28,178,228,378} / L2 y218 全对齐 chromium
  模型），REGRESSED=0。c44 族 000/002 ✓0.00%、003 ✓0.47%（< 严格 text 阈 0.5%；0.25→0.47
  为 margin-box 口径一致性提升后的同绿漂移，length-va 原子残差记账）。
- 一次全量跑捕获 `css-content/content-none-select-1` +1 红：隔离复跑 ✓0.00%、全量复跑
  824F 无此红——ZeroWeb-3 交叉流活跃窗（load 8-17）负载脆弱，入族记账（同
  intrinsic-size-009 先例）。

## 回归测试（inline/tests/r5317_va_atom_line_box_tests.rs +5）

- `r5317_text_bottom_top_atom_margin_box_union_line_box`：c44-001 几何全钉——3 行
  y/h/bl = 0/20/13、20/170/88、190/20/203 + 原子 run.y 15/95 + div 内容高 210
  （修复前行盒 110、基线 58、行 2 y=130）。
- `r5317_unmargined_atom_union_line_box_70`：c44-000 几何——行盒 70、基线 38、原子盒
  final 系 y0/y30。
- `r5317_img_atom_advance_is_border_box`：「a<img>b」b 片段 x=70（border box 60；旧
  content 口径 50）。
- `r5317_atomic_inline_breaks_collapsible_ws_sequence`：「a <img> b」b 字形 x=90（双空格
  存活；旧 80）。
- `r5317_img_atom_border_box_extra_unit`：(30,30) / border-box 声明 (0,0) / none 侧
  (30,20)。
- **mutant**：`ZW_IMG_ATOM_BORDER_BOX=0` → 4F（口径族+围合族）；`ZW_VA_TEXT_BASELINE_PIN=0`
  → 2F（围合+锚定钉）——双向可判别。

## 门禁

- fmt 0；clippy `zero-layout-engine --all-targets -- -D warnings` 0。
- **layout-engine lib 2217P/0F**（2212 + 本轮 5）。
- 本地 720 套件唯一红 = `float-shrink-inline-children-zw-001` ✗3.68%（R5311 既有记账）。
- product-smoke struct-check PASS（双 viewport 零 issue）。
- corpus 全量 ×2：825F→824F FIXED=1 REGRESSED=0（首跑负载伪影定谳见上）。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-layout-engine`：GATE FAIL 2-3/25 **归因环境，
  静窗复测挂账下轮收口（R5299/R5316 先例；零 relax 零基线改动）**。证据链：①三个 FAIL
  全落 layout 路径域，但同跑 parse/style/paint 相对基线同幅膨胀 +13~39%（仅预算余量未
  穿）——均匀环境通胀而非 layout 独回归；②**kill-switch 双关（字节旧行为）同态 A/B**：
  medium/first_paint 370.53 同样 FAIL（预算 362.35）、medium/layout 181.45 贴预算帽
  181.9（余量 0.45ms）——旧代码同处预算边缘，ON/OFF 分布重叠 Δ=0.3~2.4% ≪ 负载摆幅
  （load 3→14 区间 layout 181.45→194.79）；③incremental_layout 微基准 ON 首跑 4.20M
  FAIL、复跑翻回 PASS——run 间方差非回归。同期 ZeroWeb-3 encoding-legacy-mb rally 活跃
  窗（load 3.7-14 振荡，cargo/rustc 交替），真静窗本会话不可得。

## 挂账

- **收口**：R5316「原子 text-* 行盒围合域」→ 本轮关闭。
- **新增**：bench-gate 静窗复测（环境归因 A/B 证据在门禁节；下轮静窗收口，基线重录属
  用户决策不自行放宽）。
- **新增**：非 img inline-block 元素 content/border 推进口径分裂（同款 x 偏移域，元素臂
  CSS content 尺寸 vs taffy border-box）另案定性。
- **新增**：content-none-select-1 入负载脆弱族记账（隔离 ✓0.00% + 复跑无红证据）。
- c44-ln-box-003 0.47% 残差（length-va 原子 margin-box 参排，< 严格阈）。
- va 族余 122 / baseline-003 / intrinsic-size-009 负载脆弱 / R5314/R5315 挂账全维持。
