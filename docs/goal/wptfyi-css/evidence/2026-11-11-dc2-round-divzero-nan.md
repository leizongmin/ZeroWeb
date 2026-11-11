# DC-2 round 除零语义批（css-values 2,442 → 2,450 净 +8 零回归）

**日期**：2026-11-11 深夜
**树**：46b582b78（g8 起跑时 HEAD，含 f415be99c editing 面 + 9d243bc67 cqw 面
两个混在变量；本批 hunks = css-parser types.rs 四臂 + basic.rs 两测）
**基线**：results-g6/css-values.json 2,442P（fd2 2,426 + g6 +16 账）

## 改动面（css-parser 单域）

`types.rs` round 族四臂撤 `bv == 0.0 → av` 特判（af02f52bf 旧码
「round(x, 0) = x」），IEEE 路径自然产出：a/0=±∞、±∞*0=NaN。

- css-values-4 §round-func："If B is 0, the result is NaN."——WPT 配对
  `round(5, 0) ≡ calc(NaN)` 实证（g6b 定谳：该特判被 0061c7fff opacity 折叠
  白名单翻成 declared `calc(1)` 才暴露语义分裂，round-mod-rem-serialize
  computed 腿 1 回归同根因）。
- specified 序列化 `calc(NaN)` / computed censor-to-zero 由 serialize_math
  既有 NaN 管线承接（asin/acos 同道，r2320 批落地）。
- mod/rem 除零 None 臂不动（§10.5 同段 NaN 语义挂 g9 微批——本批只收 round
  四臂，g6b 原定范围）。

测试：`basic.rs` 新增 `test_parse_math_function_round_div_zero_nan`（四策略
B=0 全 NaN 钉）；`test_eval_calc_binary_math` 旧断言 `round(x, 0) = x` 改钉
NaN（af02f52bf 契约退役）。

## 结果（results-g8-round/，2026-11-11 23:0x）

- css-values 语料 256 页 **2,442 → 2,450 净 +8，逐页零回归**（漂移页恰为
  round 族三页，其余 253 页逐位一致——混在变量 f415be99c/9d243bc67 对
  values 语料零影响，顺带核清）：
  - round-function.html 167→169（+2：`round(5,0)`、`calc(-1*round(5,0))`
    used-value 等价类翻绿）
  - round-mod-rem-computed.html 177→181（+4：`round(±4,0)`、
    `round(±Infinity,0)` used-value 等价类翻绿）
  - round-mod-rem-serialize.html 6→8（+2：`calc(round(1,0))` specified
    `calc(NaN)` + `scale(calc(round(1,0)))` specified——0061c7fff 引入的
    computed 1 回归所在腿仍 F，归 computed 臂缺口见下）
- css-parser lib 3107P/0F（fmt 修正后）。
- make test 全量（unit zw-g8b-mktest-2312，mktest-g8b.log）：3877P/**9F**——
  7F 在册欠账组 + 2F f415be99c execCommand 契约回归（46b582b78 碰撞记录在
  案），失败清单与本批前 g7b 账逐项一致，本批零新增。编译期树含 ba 在途
  part11-color*.js hunks，engine 测试腿 814P/0F 全绿未受扰。

## 残余分桶（不阻提交）

- **serialize computed 两腿**：`calc(round(1,0))` computed 期望 `'0'`（opacity
  computed NaN censor 缺一行，§10.12）+ `scale(calc(round(1,0)))` computed
  matrix 形（经典 transform 未 matrix 化，g6b 在册 UNCLAIMED 桶——连
  `scale(mod(1,1))`=0 都不过，非 NaN 特有）。
- **mod/rem B=0 ×12 computed + ×4 serialize**：§10.5 同段 NaN 语义，g9 微批。
- **B=±∞ 有符号零桶 ×16+26**：FIXME types.rs:1420 在案（`0*∞` 需 spec 有符
  零建模，独立批）。
- **单参 round（B 缺省 =1）×2**：`round(±1.5)` 腿，nearest 平局朝 +∞
  （Rust `.round()` 半值远离零不等价，`(-1.5)→-1`），解析+求值双面，独立批。
