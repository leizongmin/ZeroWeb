# DC-2 values 刀1：CSS Values 4 §10.5 型别表 + IEEE sign/除零语义（fd，2026-11-11）

提交 eef2600da（纯 css-parser 11 文件，481 非空新增行）。working-tree 细账与逐页翻转清单见
`.acceptance/wptfyi-css/G-VALUES1-NOTES-fd.md`（含 JSON/日志/transcript 工件路径，不入库）。

## 变化

- `parse_extended.rs`：`calc_expr_math_type` 型别表 + `calc_expr_is_number` /
  `calc_expr_is_length_percentage` 改为归类包装。sign(X) 恒 `<number>`；abs/round/mod/rem/hypot
  随同族实参；乘除维度算术（number·length=length、length/number=length）；sqrt/exp/log/pow 实参
  仅 number；asin/acos/atan/atan2 产出 angle。
- `types.rs`：sign(±0)→±0（IEEE 符号位）、sign(NaN)→NaN；除零按 §10.6 产出 ±∞ 非无效值；
  一元负号字面叶直取负（`-0` 不再被 `0-x` 折成 +0；长度叶 `0px-x` 保族）；
  hypot 变参 1+（§10.8，单参 ≡ abs，≥3 参嵌套表示）。
- `serialize_math.rs`：transform scale 族（scale/scalex/scaley/scalez/scale3d）number 槽常量折叠。

## 验证

- css-parser 3106 单测全绿（新增 `values_math_type_table_tests` 9 组；6 处旧除零断言按 §10.6 更新；
  r2280/2320 兼容核对）。
- make test 3875P/7F，7F 为既有欠账组（img/svg settle R5275/R5277 + fetch/service-worker/
  websocket，be6703d58 定谳），零新增失败。
- css-values 语料 256 页 **2,218 → 2,426 净 +208，12 页增益零回归**：signed-zero +80、
  signs-abs-computed +45、hypot-pow-sqrt-computed +27、minmax-number-serialize +20、
  signs-abs-serialize +12、hypot-pow-sqrt-serialize +8、round-mod-rem-serialize +6、
  exp-log-serialize +4、calc-infinity-nan-computed +2、calc-numbers +2、progress-serialize +1、
  typed_arithmetic +1。
- css-color 语料 10,948P/63F 零回归（运行树含 color 组E 在途批，归属见组E 账）。

## 残余（后续刀）

style-system 槽臂 invalid 门 ~70F（signs-abs-invalid font-weight 52F / hypot-pow-sqrt-invalid
outline-offset 14F / round-mod-rem-invalid height 32F）；opacity 槽 pow/sqrt/hypot 腿卡 apply.rs
IMPLEMENTED_VALUE_FUNCTIONS 集（4d 面）；used-value NaN/∞ 钳制（engine 面）；angle 叶 provenance
（acos-asin-atan-atan2-invalid 1F）。
