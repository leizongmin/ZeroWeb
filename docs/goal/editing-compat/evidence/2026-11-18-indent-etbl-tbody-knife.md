# indent E-表 tbody 全覆盖整表升级刀（R20 收口档）

日期：2026-11-18 ｜ 刀体：`455db5508`（parent 2450bd0bf） ｜ 门禁批：r20b ALL-DONE 12:02:34 @bin `d87da808a92ebd50134bde160d424b5e`

## 根因与刀面

行组端点形选区（data-start/end 语义 `(tbody,0)-(tbody,1)`）经块扩展止于
`(table,0)-(table,1)`（两侧同容器＝R17 条件尾爬升守卫不破），contained 搜索的
顶层 tbody 被 wrap 过滤器（allowed-child of div/ol 判据）滤除，退化收集 cell
文本成员 → indent 逐 td 包壳。Chrome 录形＝整表外包裹单壳（spec 注记
"outermost possible location"——tr/td 壳不可落 tbody 内容模型）。

刀面（part01g.js indent nodeList 构建面，B 族容器升级臂之前插 E-表升级臂）：

- 判据＝nodeList 成员上溯行组（tbody/thead/tfoot）且父为 table → 行组整体含于
  newRange 且该表**全部**行组均覆盖 → 表内成员全数剔除（树序连续段），原首位插
  table；下游与 :147 表级端点形同态（`[table]` 走既有 wrap 面）。
- 不触发面：tr 部分覆盖（语料 :134）、单 cell 文本（:122）、表级端点（:147/:154）、
  多行组表的部分覆盖（allRg 门）。

探针证据（TEMP probe，读数后已删）：

- 刀前 got 形（/tmp/etbl-probe1.log）：t140（tbody 端点）逐 td 壳——红形；
  t147/t134/t122 现绿形。
- 刀后对拍：t140 → `<blockquote><table><tbody><tr>…</tbody></table></blockquote>`
  与语料 :140 期望形一致；t147/t134/t122 逐字节不变。

钉测：part141_indent_table_tbody_pins.rs 4 形（t140 升级形＋t147 对照＋
t134/t122 负断言＋防部分覆盖误升级断言）。

## 门禁批

判读锚＝r19c fresh 锚。批链：

- **r20**（10:27:30，脚本 md5 e97fc2d5…）：L1-L6 全 PASS（indent 1328/1 翻绿恰
  E-表键整行集合＋零新增红；fb/fs/img/event/ie SET-IDENTICAL）→ **L7 ABORT**
  ＝lib `animation::tests::test_border_image_width_lerp` 红（animation.rs:6902，
  got `""` ≠ exp `"10%"`）。HEAD 连跑 3/3 FAILED 0.00s＝确定性红。
- **归因 bisect**：测试出生 commit `b410cbcb3`（wptfyi-css 流刀 B1 border-image
  三长手动画族三通道）。window base 2450bd0bf（不含本刀）worktree 复测同测
  **FAILED 0.00s 同断言**＝窗内上游自带红，非本刀（本刀零 Rust 逻辑）。处置沿
  A1c ab7821b24 先例：skip＋留痕＋挂账，不越界改他流域码。
  - worktree 复测前置教训：`.cargo/rusty_v8/archive` 为 gitignored symlink
    （→ `~/.cache/zero-web/rusty_v8/`），fresh worktree 须重建该链接，否则
    v8 build.rs NotFound。
- **r20b**（10:50:29，脚本 md5 84a13809…，L7 skip 扩 1）：**ALL-DONE
  2026-11-18T12:02:34** 八腿全绿 @455db5508 bin `d87da808…` 全程恒一：

| 腿 | 终值 | wall |
|----|------|------|
| indent | 1328/1/0 翻绿恰 E-表键＋零新增红 | 18s |
| fb | 5043/3 SET-IDENTICAL | 729s |
| fs | 2419/3 SET-IDENTICAL | 51s |
| img | 962/0 | 12s |
| event | 176/4 SET-IDENTICAL | 3s |
| ie | 1552/1776/3 SET-IDENTICAL（零翻绿＋T 恒一） | 144s |
| lib | 4173 passed/0 failed，pin-in 10/10 | 17s |
| mt | 10090/2 X37 双键恒一（multiplier2） | 1728s |

## 结论与挂账

- indent 语料 1327/2 → **1328/1**（E-表 tbody 键翻绿恰 1 键），残 1F＝
  F 族 li 尾 br 双存键（`<ul><li>a<br>{<br>}</li><li>b</li></ul>`，占位 br＋
  选中 br 双存，待小刀）。
- 外域挂账新增（wptfyi-css 流 animation.rs 域，本流不越界改码）：
  ① `test_border_image_width_lerp` 自带红（b410cbcb3 带入，2450bd0bf 定谳，
  zero-engine lib skip 4 之一）；② animation.rs:690 fmt 违规（79b70dce2 带入，
  `cargo fmt --all -- --check` 红）。
- 窗内上游刀 A2（79b70dce2 vertical-align 插值）editing 域零位移实证
  （六腿 SET-IDENTICAL，ie/mt 含窗刀重锚后仍恒一）。

账本：`.acceptance/dps/r20-ledger.md`（批脚本 r20-etbl.sh / r20b-etbl.sh，
chmod 400 冻结＋发射器 exec 前 md5 断言）。
