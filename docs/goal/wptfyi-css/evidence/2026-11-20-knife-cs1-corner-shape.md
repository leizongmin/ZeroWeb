# 刀 CS-1：corner-shape 属性族全文法（css-borders-4 §corner-shaping）

**2026-11-20 · 收账**。主靶 css-borders corner-shape 池 +462（gained=462 lost=0 regressed=1→flake 重分类），battery b0 `40c006cf8839f605df92feb9c1ba22c1` / b1 `7dd088deceb23725099d8891ec99be29`，FACE-MARKER `parse_corner_shape_specified`（Rust 符号 bin 内 grep -a，b0=0/b1=1）。

## 页级账（17 页增益，b0→b1 P 数）

| 页 | b0→b1 | Δ |
|---|---|---|
| corner-shape/corner-shape-valid | 110→217 | +107 |
| corner-shape/corner-shape-interpolation | 0→144 | +144 |
| corner-shape/corner-shape-invalid | 0→71 | +71 |
| corner-shape/corner-shape-computed | 0→33 | +33 |
| corner-shape/corners-invalid | 0→23 | +23 |
| corner-shape/corner-invalid | 0→14 | +14 |
| corner-shape/corner-valid | 4→9 | +5 |
| corner-shape/corner-computed | 0→8 | +8 |
| corner-shape/corner-top-valid | 2→14 | +12 |
| corner-shape/corner-top-left-valid | 3→11 | +8 |
| corner-shape/corner-top-left-invalid | 0→10 | +10 |
| corner-shape/corner-top-left-computed | 0→6 | +6 |
| corner-shape/corner-top-computed | 0→4 | +4 |
| corner-shape/corner-block-start-valid | 2→14 | +12 |
| corner-shape/corner-block-start-computed | 0→1 | +1 |
| corner-shape/corner-block-start-writing-modes | 0→3 | +3 |
| border-shape/border-shape-ignore-radius-computed | 0→1 | +1 |

哨兵 backgrounds/masking/shapes/transitions 页级零位移（backgrounds 的 position-y 案见 flake 节）。

## face（7 文件，939 insertions/1 deletion，纯语义面）

- **css-parser** `parse_extended_border.rs`：`canonical_corner_shape_token`（关键字保形/superellipse 数值 canonical/-0→0/±∞→infinity 关键字/NaN 拒/FIXME(cs1-calc)）、`canonical_corner_length_token`（非负 px/%/rem/em/bare 0）、`fold_corner_shape_list`（4 同→1/[A,B,A,B]→2/[A,B,C,B]→3）、`parse_corner_shape_specified`（1-4 值角序展开折叠）、`parse_corner_groups`（组形 `Vec<(Option<radius>, shape)>`、'normal' 独占组、radius+shape 齐备、slash 组数门）、`corner_group_to_specified`/`parse_corner_pair_specified`/`parse_corner_mega_specified`；`types.rs` `CornerShapeValue::with_corner`（All 升 PerCorner 单角替换）。
- **style-system**：`apply.rs` VALIDATED_PROPERTIES 71→106（+35 corner 名，唯一触碰既有行=数组长度字面）+ `corner_prop_validate`（pub，写面 validate 与 CSS.supports 同一解析单点）；`apply_advanced.rs` 8 shape longhand 臂（8 属性 4 臂按角配对 `with_corner`；逻辑角 horizontal-tb ltr 映射 FIXME(cs1-writing-modes)）。
- **engine**：`css_style_declaration.rs` `corner_family_expand_json`（corners 恒自清；corner-shape/edge/pair/mega 展开；'normal'→radius '0px'+shape 'round'；组→角镜像 1→g0×4/2→ABAB/3→ABCB）+ `corner_family_reconstruct`（'0px round'→'normal' 塌缩；pair 等值单组；mega 三规折叠）；`computed_style.rs` 30 臂+4 助手（`corner_kind_computed` se 全折叠、`corner_group_computed` 双轴等值单写+全零组 'normal' 塌缩+resolve 归一零判定 Rem(0)、`corner_group_join2` 等值折叠、mega 组三规折叠）；`js_dom_bridge.rs` decl_supported corner 臂（animation-range 先例同型；registry 无需条目定谳——matcher/apply 路径不经 known_properties 门）。

## 发射窗前静态对账（testvectors 全 oracle 提取 228 行 vs 实现）

修 7 缺陷（全部 vector 锚定）：mega computed 尾组截断→组折叠三规（:133/:140/:141/:152）；pair computed 等值折叠（:162/:186/:200）；pair expand 空串镜像→clone 镜像（:159）；mega expand 镜像序（:107-115）；pair reconstruct 等值折叠（:157/:169/:195）；mega reconstruct 三规折叠（:107/:109/:113）；`is_zero` Px(0)/% → resolve 归一零判定（:134 'round 0rem'→'normal' Rem(0) 塌缩）。

## 残面挂账

- **interpolation 期望断言**：corner-shape-interpolation 144 翻全为 CSS.supports 前置门翻面（`from`/`to` 支持性探针）；`at: 0.3/0.5/0.6` 中点期望需 superellipse 参数插值数学＝挂账 cs1-interp-math（part03 `_zwAnimatableProps` 同域，WAAPI 关键帧白名单只影响期望断言域）。
- **cs1-calc**：corner 值内 calc 简形。
- **cs1-writing-modes**：逻辑角物理映射仅 horizontal-tb ltr（corner-block-start-writing-modes 已翻 3 页部分形）。
- **cs1-explicit-trailing-normal**：销账（组折叠三规替代截断，:152 过）。

## 过程事故线

1. **首发自拒（12:59:26）**：电池脚本 systemctl unit 名自查撞自身 transient unit 激活竞态（`DOUBLE-LAUNCH-REFUSED`）——自匹配缺陷第 4 例；flock 原子守卫本已覆盖，systemctl 自查冗余删除。
2. **FACE-MARKER 判别臂脚本缺陷（13:29:27 rc=7）**：`grep -ac … || echo 0` 双追加（grep -c 恒输出计数、0 命中 exit 1，b0 合法 0 被拼 `$'0\n0'` 炸整数测试）。bin provenance 三角干净（M2 异 md5 + marker 实判 b0=0/b1=1 恰为期望判别值 + S1/S2 ride 零扰动）→ 按 legs-only salvage 先例采纳：`knife-cs1-legs-salvage.sh` 复用双 bin 跑 5 语料×2 腿+收账（S3 补课复验零扰动），脚本捕获同步修正防复发。
3. **13:42:52 七文件同秒触**：CS-1 面 7 文件 mtime 同秒被他流窗内全树操作（疑 cargo fmt）触碰；内容完整性 verdict＝13:27 发射 patch 反向 apply-check 过+9 marker 计数全符＝语义无损（raw diff 行数漂移=hunk 合并表示差异）。同窗树内出现新一批他流 unstaged WIP（at_rules/helpers/color/parse_basic/parse_misc/transition_registry/part03.js）＝活跃 fork 在树工作。
4. **恢复走 3-way**：checkout HEAD 后 forward apply 上下文失配（churn 区），`git apply -3` 全文件干净合入且**格式 churn 自动剥除**（staged=939/1 纯语义面）；churn 来源非本流（13:27 patch 内 ~190 行 rustfmt 重排，非我语义 hunks）。

## flake 重分类（regressed=1 → 非回归）

`css/css-backgrounds/parsing/background-computed.html` position-y `calc(10px - 0.5em), -20%, bottom, 15%` 案：电池 b0 Pass/b1 Fail。同**CONTROL 二进制**（HEAD+rides 重建）三次 solo 运行 **Fail/Fail/Pass 自翻**＝run-to-run 方差实锤（stale 值随进程漂移：`13px, 6px, -10px, 8px`/`12px, 50%, 30px, -7px`＝先前 background-position 简写的 y/x 分量泄漏）；面无 background-position-y 因果路径（纯新增 hunks，零既有行修改）。该页 4 个 position-y 断言两侧普遍 F（含 `center, top, bottom` 多层计数 `50%, 0%, 100%, 50%`≠`50%, 0%, 100%`＝现存多层 position-y 计数缺陷），非本刀域。

## 验证面

- 离线钉面：testvectors 228 行 oracle 提取全对账（valid/invalid/computed/shorthand 四面）。
- 电池：css-borders 主靶 + backgrounds/masking/shapes/transitions 四哨兵，b0/b1 双 build（FONT-1 face 同树 ride，其 b0/b1 双 build 反向实证 CS-1 face 全树编译绿）。
- 语义等价链：staged 面=13:27 电池验证面（churn 剥除后唯一差异=非我 hunks 区的格式还原，域不相交）。
