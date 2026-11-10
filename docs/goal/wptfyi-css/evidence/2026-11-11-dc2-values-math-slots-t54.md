# DC-2 values 批：L4 数学函数文法槽位 + T54 inline style 写面值校验

**2026-11-11**。css-values 三角/数学桶收口第一刀，两提交落地：
`6e98ca835`（文法槽位 + 三写臂值校验）、`74a6fea79`（oracle 收窄零回归钉）。

## 结果账

values corpus（594 页 / 5,593 子测试口径不变）：

| 轮 | 树态 | Pass | 说明 |
|---|---|---:|---|
| baseline | 23:42（math 槽位实现前） | 1,379 | D 链实测起点 |
| v5d | 23:54（math 槽位后、写面校验前） | 1,540 | 解析臂 +161 |
| v5e | commit 6e98ca835（union 树） | 1,770 | 写面校验 +230，但 4 serialize 页 **-12 回归** |
| **v5f** | 74a6fea79（oracle 收窄） | **1,770** | **逐页不降 0 回归**，vs v5d 净增 +230 |

v5f 增益 9 页全为 invalid 族翻正：acos-asin-atan-atan2-invalid 0→63、
minmax-angle-invalid 0→52（实为 min()/max() 驱动）、sin-cos-tan-invalid 0→42、
minmax-percentage-invalid 0→42、minmax-length-percent-invalid 0→16、
clamp-integer-invalid 0→12、signs-abs-invalid 0→1、calc-invalid-parsing 0→1、
percentage-without-context 2→3。逐页对照脚本见 transcript
`/tmp/css-css-values-corpus.log`（v5f）vs `/tmp/css-css-values-corpus-v5d.log`。

## 机制账（三写臂 + 一陷阱）

WPT `el.style.x = v` 有**三条实现**，值校验须三处同形，缺一即 corpus 侧不翻：

1. native CSD `native_style_named_setter` / `native_style_set_property_invoke`
   （css_style_declaration.rs）——engine 侧 V8 拦截器；
2. shim `_zwMStyleProxy.writeDecl`（part03.js ~22.7k 行）——纯 shim 世界元素；
3. shim **sel/handle 代理 setProp**（part03.js ~30.2k 行）——**engine 物化元素的
   实际写臂，corpus 全落此路径**（createElement/解析元素均物化带 handle）。

裁决宿主回调 `__zw_validate_decl_value` → `zero_style_system::validate_decl_value`
（白名单六属性：transform/rotate/scale/translate/z-index/margin-left；表外属性、
shorthand、css-wide keyword、空值放行）。

**陷阱**：宿主回调返 JS **字符串**（shim 通用约定，`=== 'false'` 先例
part01c.js:336）——首刀 `=== false`（boolean）比较恒 false，T54 engine 测试
红。debug probe（t54_probe_debug，已删）定位：`d.getAttribute('style')` 为
null 而值已存 → 写走 sel/handle 臂而非 attr。

**oracle 教训**：`apply_property_value` 作文法 oracle 对 tentative 函数
（progress/calc-mix/random/sibling-index）恒拒——误拒 store+serialize 路径
本已正常的合法值（v5e progress/calc-mix/random/calc-sibling serialize 四页
-12）。收窄：值内出现实现面函数集（数学函数 + transform 族）之外 `fn(` 调用
即放行 store 原文（v5f 四页恢复、0 回归）。FIXME 面逐 parser 扩实现集。

## 残余桶（values 下批排序，v5f 口径）

- sin-cos-tan-serialize 230 / acos-asin-atan-atan2-serialize 62：specified
  值常量折叠序列化（`cos(0)` → `calc(1)`，CSS Values 4 §10.11）——css-parser
  canonicalize 面。
- signs-abs-computed 157 / round-mod-rem-computed 120 / signed-zero 101 /
  calc-infinity-nan 58：computed/used 值数学求值管线（-0 语义、clamp/mod
  取整策略）。
- random-computed 141 / progress-serialize 56 / calc-mix 113：tentative 函数
  实现面（引擎未接线，skip 口径既录）。
- line-break-ch-unit 194：ch 单位 × 行断（layout 面，非 values 解析）。
- attr-all-types 103：attr() 全型别文法（css-parser 面）。

## 验证与门禁

- 单测：css-parser 3068P（含 r2280 10 案）/ style-system 2509P（含 t54 4 案）/
  engine t54 2P（`test_style_proxy_invalid_value_rejected_t54` 三写臂钉）。
- spot check：acos-asin-atan-atan2-invalid 63/63 Pass（runner 单页 --json）。
- pre-commit guard：两提交均 PASS（隔离扫描，0 发现）。
- union 树 engine 全量 7F（img/svg settle + websocket 族）——与本批写面零交集
  （part85/part45 无 style 用法、part89 仅 display 归零写），归因待主链
  （peer 在途 shim hunks 或 main 漂移），已记 master.md 碰撞记录。

## 推送态

本地 main 领先 origin/main 3 提交（17f30543c + 本批两笔）；origin/main 另有
2 提交（f0a23470f 触 part03.js）待 rebase——工作树带 peer 在途 hunks 不满足
干净树条件，push 留待树静默窗口（peer Commit B 顺带携带或下轮 pull --rebase
后推）。
