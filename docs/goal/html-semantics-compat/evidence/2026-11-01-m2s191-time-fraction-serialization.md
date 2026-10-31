# M2-S191 time/datetime-local 毫秒序列化——stepUp/Down fractional 2F 翻绿，shim 1 文件 2 处零 rust

## 入场定刀位

s190 在册残差候选：time.html「stepUp/Down on step value with fractional
seconds」2F（step=0.001 亚秒步进断言面）。

## 勘根

两案步进计算体健在（allowedStep = 0.001×1000 = 1ms、在格 ±1ms 正确，
valueAsNumber 侧无恙）——根因在写回序列化器 `_formatHtmlDateValue`
（part03，R3317 valueAsDate setter 用）：time/datetime-local 两分支
丢毫秒分量，秒条件输出后不再追 `.fff`（S=0/ms=1 → "12:00:00" 缺
".001"；S=59/ms=999 → "11:59:59.999" 缺 ".999"）。spec
HTML §4.10.5.18 time/local date and time string 毫秒非零即输出；
格式与 s190 part01 canonical 规则同源（3 位补零后剥尾零）。

回归面盘点：`_formatHtmlDateValue` 四消费点（valueAsNumber/valueAsDate
setter + stepUp/Down 写回）语料全核——input-valueasdate.html time
setter 3 案全 ms=0、两 stepping 语料 `.value` 断言均 startsWith（注释
明言容忍可选秒/毫秒）、trailing-zeros/ms=0 既有案不变——加毫秒输出
零波及。

## 施工（shim 1 文件 2 处零 rust）

part03 `_formatHtmlDateValue` time/datetime-local 分支各补：
ms>0 → `'.' + ('00'+ms).slice(-3).replace(/0+$/,'')`；秒条件
（sec>0‖ms>0）原样保持。

## 验证

- **定向 the-input-element 目录**：逐文件 diff 对 s190 基线恰
  time.html 2F→P（32/32 全 Pass）+ 探针删除项，其余 bit-exact；
  valueasdate-stepping 4P / valueasnumber-stepping 7P /
  trailing-zeros 1P / type-change-state 380P 保持。
- **forms 全树：4407P/293F/12T → 4409P/291F/12T**（ΔP=+2/ΔF=−2
  精确闭合）；逐文件 diff 恰 time.html。
- **邻接 bit-exact**：interactive-elements 439P/44F；engine lib
  **3641P/1F**（在册同键）。
- **shim 拼接序 node --check OK**；cargo fmt --all --check 过
  （clippy 按 .rs 零触碰跳过）。

## 残留与边界

- `_formatHtmlDateValue` date/week 分支无亚秒形态（类型域无毫秒），
  不涉及；month 分支同。
- time 分支毫秒非零但秒零时输出 ":00.fff"（"12:00:00.001" 形）——
  spec canonical 形即含秒位，语料钉态一致。
- customizable select 177F 挂起待拍板（GB-20261031 征询在途）；
  fix#19c 挂待决承袭；s186-era FIXME 无语料在册。

## 下一步候选

the-input-element 目录内可动面清零（61F 全为在册挂账域：email/url
IDN 面、list/datalist、focus 导航时序域、number 约束校验等分立小簇）。
下轮入场先全树失败集中度重勘定，无新单根面则转挂账条目或 interactive
44F 复读。
