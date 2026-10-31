# M2-S189 range value sanitization——10F 钉态翻绿 + range-2 同根 3F 红利，shim 2 刀零 rust

## 入场定刀位

s188 收口后失败集中度勘定：customizable-select 簇挂拍板在途，最大可
动面 = the-input-element/range.html 10F（date/time 三兄弟 8F×3 次席）。

## 勘根

10F 同根：**range 状态 value 消毒算法缺位**。既有 `case 'range'`
（part01 `_zwSanitizeInputValue`）有夹取 + default 中点，但 ① 无
step 对齐；② 无 best-representation 序列化；③ **未挂 value getter
链**——value 模式 getter 落 P1a lazy-init 裸属性读，解析期属性
（"ppp"/-10/" 123"/缺省）原样透出。语料钉态逐案反推算法：

- 非法 fp/缺省 → default 值 max<min → min，否则 min+(max−min)/2，
  **default 仍走 step 对齐**（"ppp" min=0 max=5 → 2.5 → "3"，
  Math.round half-up）
- step 对齐 base=min nearest half-up（1.7→2、1.4→1、2.8→3、
  3.5→4——spec range sanitization 步 4 suffering step mismatch 面）
- k 封顶 floor((max−min)/step) 防对齐值越 max（spec 未钉越界折向、
  语料无面，保守不越界；stepup_beyond_max "12" 自洽）
- best representation 浮点尾差归一（5.3+1.5 → "6.8" 非
  "6.7999…98"，1e12 量化）

回归护栏勘定：stepUp/Down 四绿案全走已对齐值（幂等）；邻接
type-change-state 462P（自带 sanitizeRange 期望 "50" 与 default 中
点同源）/ input-valueasnumber-stepping 7P / clone 19P。

## 施工（shim 2 刀零 rust）

- **part01 `case 'range'` 重写**：step 属性读（非法/≤0 → 1）、
  非法 fp strict 正则 → default、min/max 夹取、k 封顶对齐、1e12
  best-representation。
- **part03 INPUT value getter**：type 捕获重构（`_rvTyG`）+ range
  消毒读臂（脏缓存优先 / 属性 lw 回落 → `_zwSanitizeInputValue`）——
  读侧单点挂接，解析/setAttribute/IDL set 三写径天然覆盖。

## 验证

- **定向 range.html：25/25 全 Pass**（10F→P 精确 + 15 既有零回归）。
- **forms 全树：4368P/332F/12T → 4381P/319F/12T**（ΔP=+13/ΔF=−13）；
  文件粒度 diff **range.html 10F 翻面 + range-2.html 3F 同根红利翻
  面**（3F+2P→5P），其余 409 文件零变化。
- **邻接 bit-exact**：type-change-state 462P、
  input-valueasnumber-stepping 7P（定向复跑）；interactive-elements
  483 键零位移（439P/44F）。
- **engine lib 3641P/1F**（1F = ua_display_li 渲染流在册同键）。
- **shim 拼接序 node --check OK**；cargo fmt --all --check 过 +
  guarded workspace clippy 零告警。无探针页（勘根经失败消息逐案
  反推 + 邻接面复跑闭合）。

## 残留与边界

- `_controlValue`（表单序列化）仍读 `_inputValues` 原始存储——range
  提交编码面未消毒（getter 消毒不回写存储；语料无 range 提交断言面，
  挂 FIXME 按语料再补）。
- value IDL setter 不消毒（脏缓存存原串，读侧统一清洗）——spec 是
  set 时清洗，观察面等价；若后续语料钉「set 后 value 属性内容」
  差异再补写侧。
- step 对齐 tie 折向（.5 恰中）仅 Math.round half-up 一案在册
  （"ppp" → "3"）；负 half（−0.5 → −0）不可达（k≥0 恒成立）。
- date/time 三兄弟（month/datetime-local/seconds-leading-zeroes
  各 8F）为次席候选——疑 value 消毒同族但根不同（日期算法面），
  待独立勘根。

## 下一步候选

date/time 消毒三兄弟（8F×3）在册；customizable select 177F 挂起待
拍板（GB-20261031 征询在途）；fix#19c override builtins 1F 挂待决
承袭；s186-era FIXME（wrapper select/textarea/output `.value` 家族）
在册无语料。
