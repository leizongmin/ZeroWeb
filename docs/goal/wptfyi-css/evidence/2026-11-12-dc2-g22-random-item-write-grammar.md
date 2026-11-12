# DC-2 g22 批：random-item 写面文法门（CSS Values 5 §funcdef-random-item 实参文法裁决）

日期：2026-11-12　刀：g21 播种求值之上的写面收口——random-item()
parse 期实参文法校验（`random-item( <random-key>? , <declaration-value>#
)`：key 节缺失即整参文法非法、项表非空、逐项声明值禁顶层 `!` `;` `]`
与块残片），三写臂共用门。探针 **random-item-invalid 0P/14F → 14P/0F
全页清零**，random-item-valid 21P 零回归，家族 7 页 ±0；六套件权威账
values 3,214 → **3,228 净 +14**，五套件 Pass 级零漂移。

## 1. 实现面

- **文法裁决器**（css-parser values/random.rs）：`random_item_args_valid`
  —— ① `random_scan_balanced`（引号/括号/花括号三层配平，负深即
  false）；② key 节判定：首节非 `{` 开头且非 var-key 且非空 → 缺
  key 槽即非法（WPT invalid 页 `", serif, sans-serif"` 形），首节为
  `{` 开头块或 var-key 时项表自次节起；③ 项表非空（零项非法，
  `"auto"` 单参 = key 无项表）；④ 逐项 `random_item_value_valid`：
  整项 `{…}` 块（`random_match_block` 配对偏移，块后残片非法）或
  无花括号裸项；⑤ 裸项 `random_scan_no_forbidden`（括号深 0 处禁
  `!` `;` `]`——块内豁免，声明值内部合法）。
- **求值臂接线**：`eval_random_item_call` 头部 `!random_item_args_valid
  → None`（var( 豁免检查之前）——compute 期同门，unresolvable 路径
  一致。
- **写面门**（style-system property/apply.rs `random_item_calls_valid`
  + validate_decl_value 白名单检查之前接线）：引号状态机扫描
  `random-item(` needle（ident 边界检查防前缀假阳）→ `match_paren`
  要求闭合（未闭合即拒，与替换器 extract 的 break 放行不同——写面
  fail-closed）→ 逐调用转 `random_item_args_valid`。三写臂共用：
  native CSD / shim writeDecl / shim sel-handle 代理（corpus 实际写臂
  走宿主回调 `__zw_validate_decl_value` → 本门）。
- **规范锚**：CSS Values 5 §funcdef-random-item 文法 + §9.4
  parse-time invalid 语义——任意替换函数 parse 期仅实参文法裁决。

## 2. 探针（results-g22probe，filter=random-item 8 页）

- random-item-invalid：0P/14F → **14P/0F 全页清零**（16 语料中 2 案
  g21 前已过；本刀 14F 形全为写面缺门——空参/空 key 槽/零项/`auto`
  混 `!;]`/块残片/`rgb(1, 2` 未闭合，恰为本门裁决面）。
- random-item-valid：21P/0F 保持（21 语料全矩阵正验门：空项/多块/
  块裸混排/裸 ident 首节/var key/负 fixed/空白变体全放行）。
- random-item-computed 8P/23F、nested 2P/2F、家族其余 4 页 ±0。
- 批面净 +14，恰中预测。

## 3. 验证面

- 单测：css-parser lib **3,173 全绿**（g21 3,172 +1：random 组
  `item_args_grammar_matrix` 新增 38 断言——15 invalid 形 + 21 valid
  形 + 2 豁免形，`"auto, rgb(1, !, 2)"` 块内 `!` 合法、`";"` 引号
  字面免疫）；style-system lib **2,526 全绿**（两 crate 在 doc 注释
  与 lint 修复终码上复跑 exit=0）；engine lib 3,909 过（7 失败逐名
  核对全为既有组：img settle 2/fetch 1/iframe-sw 1/response-body 1/
  websocket 2，与 g21 基线一致，零新增；本树 HEAD 43d476e64 已含
  peer editing bold 批——engine 计数同 g21，两流零交互）。
- clippy（css-parser/style-system/engine 定向，-D warnings）零告警
  （exit=0）：首轮两处机械告警已修——random.rs spec 链接行缺独立
  成段（doc_lazy_continuation，补空行）+ apply.rs 扫描器字符串切片
  比较（op_ref + sliced_string_as_bytes，改纯字节切片
  `bytes[i..j] != needle[..]`）；rustfmt 三 crate 零漂移（首轮漂移
  已机械收敛，git diff --stat 仅 3 个 WIP 文件）。
- **provenance**：corpus 链二进制 release（本树 HEAD 43d476e64 +
  g22 WIP 构建）；doc 注释与 apply.rs lint 修复不进二进制（编译期
  消除），corpus 结果对终码有效。

## 4. 权威账（results-g22，六套件全量，service 模式 test-guard 包裹
zw-g22-full-2100，release 二进制 = 本树 HEAD 43d476e64 + g22 WIP）

- **values：3,214 → 3,228 净 +14，零回归**（唯一变化页：
  random-item-invalid 0P/14F → **14P/0F**，其余 255 页 ±0——含
  random-item-valid 21P 保持、random-item-computed 8P/23F 持平）。
- 五套件 Pass 级零漂移：color 10,972 ±0、transitions 200 ±0、
  animations 238 ±0、easing 51 ±0、transforms 229 ±0。transitions
  F +1（2,279→2,280）与 animations F +1/T −1（716→717/8→7）为页面
  超时截断的子测试枚举差伪影（g21 同类，Pass/Timeout 双锚定）。

## 5. 残余桶（random 族更新）

- random-item-invalid **清零**（14F → 0F）。
- random-item-computed 23F：shim gCS raw 回显桶 + color 槽 shim 黑值
  桶 + color 单项 supports 2F（g21 挂账不变）。
- random-computed 46F、@规则通道 ~17F、unresolvable IACVT 5F、SVG
  supports 8F、custom property 2F（g21 挂账不变）。
- 既有挂账不变：tree-counting 家族残 89F、anchor 953F、独立大桶
  （line-break-ch 194F/attr-all-types 103F/viewport-units 84F/
  calc-size 62F）、serialize-resolution 29F。
