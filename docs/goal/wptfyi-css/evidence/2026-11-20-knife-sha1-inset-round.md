# 刀 SH-A1 clip-path inset() round 组全模型收账（2026-11-20 11:4x）

## 收账结论

- **A/B 净 +284（主电池 gained=218 lost=0 regressed=15→补刀折叠修后 regressed=0，折叠修另 +66＝b1' 对账 gained=66 lost=0 pages_gain=4）**，face 落库 push `5fd4bda01`（e791760c5 于 GBX base d516f4e29 重放；8 文件 +169/−26
含折叠修；导出树 /tmp/sha1-verify `cargo check -p zero-css-parser -p zero-engine`
Finished 11.78s 编译核证绿——GBX 与本刀同域 8 文件文本级 3-way 合并后组合态
编译实证；tests/clip_path.rs 与 GBX geometry-box pins 并集解、两套测试共存）。
- 局部账累计 +7747（origin GBX 收账重锚 +7915）→ **+7915+284 = +8199**。
- 主靶 css-shapes `shape-outside-inset-003.html` P 286→520（b1 主电池 +182 + 折叠修 +52）；inset-002 P 44→80（+28+8）；inset-006 P 4→13（+4+5）；inset-function-computed P 6→10（+4）；computed-shape-001 P 3→4（折叠修 +1）。
- css-masking 双臂恒等零差（62 页 5259P，clip-path 语料 computed 面不受本刀影响＝哨兵臂）。

## 刀面机制（face 设计）

历史缺陷：`ClipPathValue::Inset.round: Option<ClipPathRadius>` 单值半径槽——`inset(10px round 5px 10px 15px 20px)` 解析期截断为首值，多值/slash 组全丢。face 换装全模型（8 文件 +164/−26）：

1. **css-parser types.rs**：`round: Option<ClipPathRadius>` → `Option<BorderRadiusValue>`（css-backgrounds-3 §border-radius 1-4 值 [/ 1-4 值]，parse 期镜像展开——与 rect()/xywh() round 槽同模型）。
2. **css-parser parse_basic_shape.rs**：`parse_border_radius` 升 `pub(crate)` 供 inset round 槽复用（负半径拒收在解析器内）；`parse_shape_len` 拒 thin/medium/thick 线宽关键字（CSS 2.1 §8.5.1 对形状坐标非法——token 层先拒，`is_nonnegative_keyword` 兜底）。
3. **css-parser parse_extended_border.rs**：`parse_clip_inset` round 槽改走共享 border-radius 解析器（css-shapes-1 §funcdef-inset round 参量 = <'border-radius'>；`inset(10px round closest-side)` lenient 钉面作废改拒）。
4. **css-parser tests/clip_path.rs**：4 案 pin 测试（2 值镜像展开、slash 椭圆组保形、4 值保形、负半径拒收）+ closest-side 拒收归正。
5. **engine computed_style.rs**：`border_radius_computed_to_css` computed 串化——h 组简写最小化（全等→1、对角等值→2、左右等值→3、否则 4）+ **v 组与 h 组序列化等值时省略**（css-backgrounds-3 computed 简写折叠；主电池 REGRESSED=15 定谳补刀）。
6. **engine clip_interp.rs**：`lerp_radius_opt`（单值）→ `lerp_radius_tuple`（h 四元组逐位 + v 组存在性一致；rect()/xywh() 同款），插值发射 4 值全形（parse 镜像展开幂等）。
7. **engine animation.rs**：`clip_value_has_calc` 扩 8 槽（h 四元组 + v 组逐位 calc 扫描）；插值 pin 测试换 4 值全形期望。
8. **engine border_shape.rs**：`uniform_round_radius` 提取（h 全等且 v 缺省或镜像等值 → h[0]；非均一 round 组回 None 沿方角回退）——R5184 判例 `inset(0 round R)` 单形行为不变；paint 测试换装 uniform_round 便利函数。

## 电池 knife-sha1（14 腿 = 7 corpus × b0/b1）

- b0 = HEAD + ride（face 剥除，`git apply -R`）；b1 = HEAD + ride + face。
- patch：`knife-sha1-face.patch`（launch 时 `git diff` 8 文件提取 + optical 指纹 hunk 域拆：kept=22 dropped=4）。
- bins：b0=`d4dad043becec701b76e0d13f157964f` / b1=`de1fd6a3bd49fa458008b63f7232ae28`；M2 异 md5 过（face 无 ASCII strings marker，判别 = STRIP-CHECK + M2 + ride census）。
- RIDE 11 文件 S1/S2/S3 md5 零扰动。
- 腿时序：BUILD-ALL-DONE 11:00:12；b0 七腿 11:00:14–11:18:33（masking 79s/shapes 62s/transitions 8m22s/animations 3m/fonts 104s/pva 103s/typed-om 70s）；b1 七腿 11:18:33–11:35:59；RECONCILE + GATE-DONE 11:35:59 rc=0。
- 产物：`.acceptance/wptfyi-css/results-knife-sha1/`（b0/b1 七语料 JSON/err、s1/s2/s3-md5、ride-files、regressed-css-shapes.txt 15 案清单）。
- **官方账：gained=218 lost=0 regressed=15 pages_gain=4 pages_loss=0**；哨兵/监控六语料 b0/b1 逐页恒等（masking 5259/transitions 1051/animations 817/fonts 6538/pva 958/typed-om 12024）；ride 11 文件 S1==S3 零扰动。
- css-parser pin 腿：clip_path 62P/2F——2F 系预存红（`test_shape_outside_specified_{keyword_reorder,inset_round_collapse}`，SH-A1-COORD 备案 10:2x stash 窗硬证＝HEAD 无 face 即红，2fd814691 canonicalizer 非等值组改 Some 保形未同步 3788f6c99 代 pin，归 shapes-canonical 流修，本刀不碰）；face 新增 4 pin + 改装断言全绿。

## REGRESSED 定谳与补刀（主电池 forms 缺陷 → fix）

主电池 shapes 臂 gained=218 之外 **REGRESSED=15**（inset-002 ×2 + inset-003 ×13，全部 `round R / R` 等值 slash 形 computed 臂；清单 `regressed-css-shapes.txt`）。双证定谳＝face 缺陷非语料/环境：

1. WPT oracle：`parsing-utils.js` generateInsetRoundCases `lhs == rhs` 臂期望走 `serializedInsetRound(lhs, null)` ＝ computed 期望 **无 slash 形**（css-backgrounds-3 computed 简写折叠语义）。
2. b0 单半径模型把 `round R / R` 截断为 `R`，串化 `round R` 与折叠期望**偶合 Pass**；b1 保真串化 `round R / R` 反 Fail——等值组折叠缺失是 face 串化缺陷，非行为回退。

补刀（GATE-DONE 后 bins 冻结窗内入树，重建 b1' 补跑 shapes/masking 两语料）：`border_radius_computed_to_css` 加 `v_str != out` 门（6 行 + 注释更正）。指定值层 canonicalize_inset_inner 等值折叠既有（parse_basic_shape.rs:1483–1486）无需动。

## 补刀定谳（b1' 对账，2026-11-20 12:46，rally-cont）

b1' bins = FONT-1 电池 b1（基树 HEAD+optical+sha1 face+折叠修+font1 face+CS-1 face+es_module WIP；bins 12:22-12:24 冻结，md5 `238d288082364fe1acb0be66930c00e4`）。对账 = results-knife-sha1/b1 vs results-knife-font1/b1（脚本 sha1-b1prime-reconcile.py）：

- **REG 15/15 全翻 P，still_fail=0**（WPT oracle `serializedInsetRound(lhs, null)` 折叠期望逐案兑现）。
- 折叠修净效应 **+66 超 +15 预期**——折叠修翻的是全部等值折叠形 computed 断言（含 b0 双挂的非 REG 案 51 项）：inset-002 +8、inset-003 +52、inset-006 +5、computed-shape-001 +1。
- masking 哨兵 5259==5259 **零差**（折叠修 shapes 域封闭；ride FONT-1/CS-1/es_module 零扰动）；fonts +82 归 FONT-1 face 承载面（R3-LAUNCH 账）。
- transitions/animations/pva/typed-om b1 腿补核（GATE-DONE 12:58:58 后全七语料实跑）：masking/pva/typed-om **零差**（哨兵全净）；fonts +82（归 FONT-1，R3-LAUNCH 账）；animations +25（event-dispatch.tentative +2、animation-range-shorthand +23——FONT-1/CS-1 face 承载面，非 sha1 域，归彼账）；transitions +3 全在 `zw-probe-wa3b-census.html`（他 fork 探针页在 FONT-1 电池树内、sha1 电池树外的树态伪增益，不入账；同语料 properties-value-inherit-001/002 timeout→fail swap 属 CASE_WALL flake 域，P 计数无差）。**sha1 净 +284 持立（主电池 +218 + 折叠 +66）。**

## 主靶逐页（b0→b1'）

| 页 | b0 | b1（无折叠修） | b1'（含折叠修） | 净 |
|---|---|---|---|---|
| shape-outside-inset-003.html | 286 | 468 | 520 | +234 |
| shape-outside-inset-002.html | 44 | 72 | 80 | +36 |
| shape-functions/inset-function-computed.html | 6 | 10 | 10 | +4 |
| shape-outside-inset-006.html | 4 | 8 | 13 | +9 |
| shape-outside-computed-shape-001.html | 3 | 3 | 4 | +1 |
| **合计** | 343 | 561 | 627 | **+284** |

