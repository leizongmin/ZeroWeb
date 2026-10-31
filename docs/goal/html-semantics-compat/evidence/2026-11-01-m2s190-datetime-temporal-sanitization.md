# M2-S190 date/time 消毒面——三兄弟 24F + trailing-zeros 1F + date 红利 1F 共 26F 翻绿，shim 2 文件零 rust

## 入场定刀位

s189 收口后次席候选：the-input-element date/time 消毒三兄弟
（month.html 8F / datetime-local.html 8F / input-seconds-leading-zeroes.html
8F）。勘根前语料全读 + 探针页（s190probe.html，验后删）钉态。

## 勘根（三根分立，探针 A/B/C 定谳）

1. **valueAsNumber/valueAsDate type 读快照盲区**（R136 同类）：
   part03 两 getter type 读用非 lw `__zw_get_attr`——同批
   `setAttribute('type', …)` 后 IDL 读恒 miss → `vasT=''` → 恒 NaN
   （input-seconds-leading-zeroes 8F 根因：探针 A `vasN=NaN` 而解析期
   属性对照 d2 `vasD2=946729800500` 正常——计算体健在，唯 type 读盲）。
2. **时间族 value getter 消毒链接缺位**：range 臂（s189）只覆盖 range；
   date/month/week/time/datetime-local 落 P1a 裸属性读——解析期/
   setAttribute 非法值原样透出（探针 C `month13=[2013-13]`、
   month.html 8F 全「expected "" but got raw」）。
3. **datetime-local 消毒缺 canonical 重序列化**：旧 case 正则合法即
   verbatim 回显。语料钉态反推 canonical 形：'T' 分隔（空格解析接受
   不出）；秒仅 sec>0 或 ms>0 输出（"00:00:00.000" → "T00:00"）；
   毫秒 3 位补零后剥尾零（".010"→".01"、".500"→".5"、".040"→".04"、
   ".003"→".003"——trailing-zeros 与 seconds-leading-zeroes 两语料
   round-trip 钉态联合唯一确定）；非法（≥4 位小数）→ ''。
   date/month/week/time 语料只钉合法 verbatim，保持原 case 不动。

## 施工（shim 2 文件 4 处零 rust）

- **part01 `case 'datetime-local'` 重写**：正则取分量 + 复用
  `_parseHtmlDateValue` 日历回读校验 → canonical 重序列化
  （`('00'+ms).slice(-3).replace(/0+$/,'')` 毫秒规则）。
- **part03 value getter 时间族消毒臂**：range 臂后同构扩展
  （脏缓存优先/属性 lw 回落 → `_zwSanitizeInputValue`）——读侧单点
  挂接，解析/setAttribute/IDL set 三写径天然覆盖。
- **part03 valueAsNumber/valueAsDate type 读换 lw 融合读**：s136/s148
  同形（`typeof __zw_get_attr_lw === 'function'` 守卫口径）。

## 验证

- **定向 the-input-element 目录二跑**：逐文件 diff 对 s189 基线恰 5
  文件翻面——month 8F→P（15/15）、datetime-local 8F→P（17/17）、
  input-seconds-leading-zeroes 8F→P（12/12）、
  datetime-local-trailing-zeros 1F→P（红利）、date.html 1F→P（红利，
  消毒臂搭车）、其余 bit-exact；探针 3F 已验后删。
- **forms 全树：4381P/319F/12T → 4407P/293F/12T**（ΔP=+26/ΔF=−26
  精确闭合）；逐文件 diff 恰上述 5 文件，~400 文件零变化。
- **邻接 bit-exact**：interactive-elements 439P/44F；engine lib
  **3641P/1F**（1F = ua_display_li 渲染流在册同键）。
- **shim 拼接序 node --check OK**；cargo fmt --all --check 过。
  clippy 按质量门禁实际影响原则跳过（纯 .js 改动，.rs 零触碰）。

## 残留与边界

- 首轮目录跑 focus-dynamic-type-change-on-blur 文件级 Timeout（week
  4 案未上报）——solo 复跑 80/80 + 目录二跑 80P 复现失败，定谳负载
  敏感暂态（s171 在册同形），非本刀回归。
- time.html stepUp/Down fractional-seconds 2F 既有（基线同在，非本刀
  位移；step 对齐时间族面待语料再勘）。
- value IDL setter 不消毒（写侧存原串、读侧统一清洗，观察面等价）；
  `_controlValue` 表单序列化仍读原始存储（s189 在册 FIXME 承袭，
  时间族提交编码面同）。
- date/month/week/time canonical 重序列化未做（spec 同 canonical 化，
  语料未钉改写面，不预防性扩面）。

## 下一步候选

customizable select 177F 挂起待拍板（GB-20261031 Feishu 征询在途）；
fix#19c override builtins 1F 挂待决承袭；s186-era FIXME（wrapper
select/textarea/output `.value` 家族）在册无语料；time stepUp/Down
fractional-seconds 2F 待勘。
