# M2-S186 click 分派域合刀——mutTree wrapper 事件面三件 + proxy 拾取器激活消费，15F 恰靶翻绿，shim 2 零 rust

## 入场定刀位

s185 账本「下一步候选」首位：disabled click 8 + click 消费激活 7
（同 click 分派域可合刀）。

## 勘根（双靶三根）

1. **disabled-elements-01 8F（test3/test4 × button/input/select/textarea）**：
   `innerHTML` 解析元素是 mutTree `_zwMEl` plain wrapper（探针
   fingerprint：`getPrototypeOf(el).constructor.name === "HTMLInputElement"`
   ——R125 按 `__zwHtmlTagIface` 链接口原型；ownKeys 含 `__zwIsMEl`；无
   `__zwSelector`/`__zwHandle` 非 `_makeProxy` 产物；`root.childNodes[0] ===
   el` 同一对象）。三个缺口：
   - ⅰ) **INPUT `.value` 无 accessor**——proxy 侧值分派在 get trap
     （part03:26301 select/OUTPUT/tag-skip/TEXTAREA/INPUT-mode 臂），wrapper
     不经。语料首断言 `assert_equals(other.value, "no event dispatched")`
     got undefined，8 案全同形（`other` 恒为 innerHTML 解析 INPUT）。
   - ⅱ) **on* IDL 属性处理器不激活**——`elem.onclick = fn` 落 expando 死槽；
     wrapper dispatchEvent（R167 链派发）只激活注册 listener（`_zwMEvLs`
     本地存储）与 inline onclick **内容属性**（R155 编译）。test3 直接
     dispatchEvent 期望 handler 改写 `other.value`，handler 从不执行。
   - ⅲ) **wrapper `node.click()`（R151/R152/R155）无 disabled 门禁**——
     proxy 侧 `.click()` API 有 R313（part04:5682 可禁用族 + disabled 属性
     → return），wrapper 路径无镜像，test4 disabled 元素 click() 照常执行
     inline onclick。
   disabled 反射在 wrapper 可用（test1/test2 基线即 P）由
   `_zwMDefineBooleanReflected`（part03:20374 R154 布尔反射族）闭合——
   非本刀面。
2. **click-user-gesture 7F（loop B × 7 picker 类型）**：bless +
   `input.click()`（createElement detached proxy）+ blur 后
   `navigator.userActivation.isActive` 恒 true——`.click()` API 无激活消费
   臂（旧仅 fullscreen 消费 `_zwTransientActive`，part04:4923）。拾取器
   类型 = color/date/datetime-local/file/month/time/week（语料名单，7 案
   对位）。loop A（does not throw，无 isActive 断言）基线 7P 不动。

## 施工（shim 2 零 rust）

- **part03 wrapper dispatchEvent**：target 站（`station === node`）AT_TARGET
  bubble pass 尾（`captureOnly !== true` 每次派发恰一次）补 on* 属性处理器
  激活——`node['on' + ev.type]` typeof function 即设 currentTarget/eventPhase
  后 `call(node, ev)`，注册 listener 之后、任意 on\<type\> 通用。spec
  event handler IDL attributes：属性赋值等价注册 listener。
- **part03 factory INPUT `value` accessor**（`_zwMDefineBooleanReflected`
  调用后、R5000 title/lang/dir 同款 defineProperty 形）：attr-backed 近似
  ——getter 读 value 内容属性 absent → ''（default 值模式），setter 写回
  属性。真实 DOM value= 写 dirty 值不回写属性，wrapper 无 dirty 追踪，
  同 turn 读写回环优先（FIXME 记 select/textarea/output 值模式族按语料再补）。
- **part03 wrapper `node.click()` 顶部**：R313 门禁镜像——可禁用族
  button/input/select/textarea/fieldset/optgroup/option + disabled 属性存在
  → return（与 proxy 侧同口径；`node.hasAttribute` wrapper own 方法）。
- **part04 `.click()` API**：R313 门后 INPUT 拾取器类型消费
  `_zwTransientActive = false`——type 读 `handle ? __zw_get_attr_handle :
  __zw_get_attr`（R136 lw 融合读同款），7 类型小写归一匹配。preventDefault
  不豁免（Chrome 语义：click() 方法本体触发 picker 非默认动作）；disabled
  早退在上，不派发即不消费。

## 验证

- **定向 disabled-elements-01：16/16 全 Pass**（8F→P 精确，test1/2 基线
  8P 保持）。
- **定向 click-user-gesture：14/14 全 Pass**（loop B 7F→P 精确，loop A
  基线 7P 保持）。
- **forms 全树：4348P/352F/12T → 4363P/337F/12T**（ΔP=+15/ΔF=−15 精确
  闭合）。文件粒度状态表 diff 外科级：仅两靶标文件翻面（click-user-gesture
  7P+7F → 14P；disabled-elements-01 8P+8F → 16P），其余 411 文件零变化。
  对账方法注记：多行子测试名（wrapping round-trips 簇含换行）使 (key,state)
  行级 join 不可靠，本轮改文件粒度对账（P/F/T 计数 × 文件 diff）。
- **interactive-elements 483 键零位移**（439P/44F bit-exact）。
- **engine lib 3641P/1F**（1F = ua_display_li 渲染流在册同键）。
- **shim node --check SYNTAX_OK**；cargo fmt --all --check 过 + guarded
  workspace clippy 零告警。
- 勘根探针 6 页（zz-s186-probe1..6）验后删。

## 残留与边界

- wrapper INPUT `.value` 为 attr-backed 近似：真实 DOM `value=` 写 API 值
  （dirty）不回写属性、`getAttribute('value')` 保持初始值——同 turn 读写
  回环优先，差异面无语料形态。
- on* 激活序 = 注册 listener 之后（真实 DOM 为属性赋值时点注册序）——
  语料无同型竞争 listener 面。
- wrapper select/textarea/output `.value` 值模式族缺位（FIXME 在册，随
  语料需要再补）。
- wrapper `node.click()` 无拾取器激活消费（proxy 侧独有）——wrapper 路径
  为 detached/克隆树 headless 近似（R151 语义），无语料面。

## 下一步候选

color-attributes input.alpha/colorSpace 2F 小反射面；submission-checks
三兄弟（连接性面）在册；customizable select 177F 挂起待拍板
（GB-20261031 飞书征询在途）。
