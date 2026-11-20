# 刀 BSHAPE-SERIAL · basic-shape CSSOM 数值 canonical（2026-11-21）

## 目标与机制

css-values-4 §8 CSSOM canonical 序列化：basic-shape computed 形的数值 token
此前 parse_basic_shape `norm_zero_tok` 保形直通，`+10px`/`10.00px`/`20.230px`
原样泄漏，WPT 要 `10px`/`10.23px`。修法单 helper 升级：数值 token 剥可选
`+` 前缀 + f64 最短往返（Rust Display）+ 单位保留，负号保留，bare 0→0px
旧规保持，非数值 token（关键字/calc() 组）保形；circle/ellipse radius join
与 at 位置 fallback 接入同臂，inset/polygon/xywh/rect 既有路径自动受益。

face `5e5a12929`：parse_basic_shape.rs（norm_zero_tok 数值臂 + FACE-MARKER
'BSHAPE cssom-number-canonical-sign-trailing-zero'）+ parse_basic_shape/tests.rs
（shape_numeric_token_cssom_canonical 七面钉 + path_valid_forms 逗号保形
既有红订正）2 文件 +86/−7。

## 电池

r1（01:31-01:39 会话 rate-limit 窗）判 gained=87 lost=0，但 S1/S3 ride
census MISMATCH——发射窗内 MATHFN fork 对 ride 文件（clip_interp/es_module）
在飞编辑，b0/b1 双臂 ride 同载假设被破坏 → r1 不收账，留作预判。

r2（03:21-03:30 systemd unit zw-bshape-r2-0322，树静默窗重发射）＝仲裁轮：

- 双臂：b0 `95569113`（HEAD~1 face 剥）/ b1 `62f1a382`（HEAD face 在树）。
- 判别：FACE-MARKER b0=0/b1=1；M2 异 md5；S1/S2/S3 ride md5 三窗恒
  （ride=clip_interp.rs+es_module.rs 双臂同载相消）。
- 8 腿：shapes/masking/ui/borders × b0/b1，rc=1（runner 有败测试正常）。

## RECONCILE r2（gained=82 lost=9 → 恰集收账 +82）

| 页 | P 变化 |
|---|---|
| shapes shape-outside-circle-006.html | 18→34 (+16) |
| shapes shape-outside-circle-009.html | 26→36 (+10) |
| shapes shape-outside-ellipse-006.html | 27→51 (+24) |
| shapes shape-outside-ellipse-009.html | 26→36 (+10) |
| shapes shape-outside-inset-004.html | 6→12 (+6) |
| shapes shape-outside-inset-006.html | 13→22 (+9) |
| shapes shape-outside-polygon-005.html | 7→12 (+5) |
| shapes shape-outside-shape-arguments-001.html | 2→4 (+2) |

shapes 全域 gross F2P=82 / P2F=0 纯单向，r1↔r2 逐页逐值复现＝确定性 face
效应（桶分布 ellipse 34/circle 26/inset 15/polygon 5/other 2 与 r1 全同）。

**corner-shape-interpolation 排除笔**：r1 134→139 (+5)／r2 145→136 (−9)，
gross 双向翻转 123 断言（同语义断言两臂互翻）＝CS-1c 已定谳的
corner-shape 插值 ±17~20 噪声带（行为等价构建同页互翻前例 69F→P/52P→F）。
flake 域不入账，基线计数漂移照 CS-1c 让渡约定不追。

## 哨兵

masking/ui/borders 页级 SET-IDENTICAL（r2 除 corner-shape-interpolation 噪
声页外零位移）；ASYM 0。

## 残面

- path_valid_forms 逗号保形钉测＝既有红（落后 canonicalize_path_data 空格
  制形改版），face 提交内订正并注明归属，非本刀域。
- shapes 路径面口径（非动画 281F）收 82，余量下次刀位 census 重估。
