# M2-S11b — Window.origin 自有 accessor 吞全局 `var` 重声明——referrer 6F 收口（2026-10-29）

**切片**: [M2-S11](2026-10-29-m2-s11-cors-preflight-cache-coverage.md) 记账 2
（cors-preflight-referrer 6F）专项。**6F 全收口 +6 零回归**，全通道
10989 → **10995**（对一期基线 9354 累计 +1641）。

## 归因纠正（M2-S11 记账 2 的引擎 `_zwRefOut` 假设不成立）

M2-S11 档假设「引擎 `_zwRefOut` origin 序列化带尾斜杠、期望无斜杠」。
本轮 wpt-data 一次性诊断用例（zz-m2s11-probe，验后即删）实证**两腿分离**：

- `get_host_info().HTTP_ORIGIN + '/'` 求值正常（`http://wpt.test:8000/`），
  排除 runner 语料副本差异 / 模板替换 / assert 参数序假设；
- 同页顶层 `var origin = get_host_info().HTTP_ORIGIN + "/";` **装载期即丢
  拼接值**（读到 `http://wpt.test:8000`），异步回调后复读同样丢。

根因：shim 把 `Window.origin` 定义为 globalThis 的**自有可配置 accessor**
（真浏览器挂 `Window.prototype`，全局 `var origin` 建自有数据属性自然遮蔽）。
按 spec（GlobalDeclarationInstantiation → CreateGlobalVarBinding）可配置
accessor 遇 `var` 重声明应转换为数据属性；**QuickJS 不落实该转换**——赋值
静默吞、读仍走 getter（返回 `location.origin` 无尾斜杠）。即：引擎 Referer
发送（`origin + '/'`）与测试期望（同形）本就一致，坏的是测试的全局变量求值。

## 实现面

- `part01.js` 原 `Object.defineProperty(globalThis, 'origin', {get})` 删除；
  改**数据属性快照** `globalThis.origin = location.origin || 'null'`，置于
  shim 安装尾（`part07.js` 闭合 IIFE 前）——part01 安装期 `_hist_current`
  依赖未就绪，eager 求值 `href()` 翻车（过程轮 r3 TypeError 定界）。
- 数据属性走 QuickJS 常规 var 重声明路径；runner 单文档页 origin 装载即定，
  快照不腐化（活性边界见记账 2）。
- https://html.spec.whatwg.org/multipage/window-object.html#dom-window-origin
- 可复用经验落
  [learnings/bugs](../../../learnings/bugs/2026-10/2026-10-29-quickjs-global-var-accessor-conversion.md)
  （定位手法：wpt-data 一次性诊断用例把可疑值直接 assert 进 runner JSON）。

## 验证

- 全通道 A/B（`netapi-m2s11-final.json` vs **直接前轮**
  `netapi-m2s11-r2.json`，逐 subtest）：**+6 增益 / 0 回归 / MISSING 0**
  （10989 → 10995）；per-file Δ 仅 cors-preflight-referrer（6/12 → 12/12
  收口），其余全通道逐文件零变化；stderr 0 panic（0 字节）。
- 对 S10 基线（`netapi-m2s10-r2.json`）同轮复核 +46/0/0（S11 +40 与 S11b
  +6 合并账）。
- 诊断用例删除后重跑定账（净通道分母 12078，无 zz 面）。

## 记账（未清偿）

1. cors-preflight-redirect「after preflight failed」×5（沿 M2-S11 记账 1）：
   本轮补上游 fixture 核对——runner redirect.py OPTIONS 分支与上游逐字
   等价（无 `redirect_preflight` → 200 + ACAO + ACAH 合法预检响应）；拒源
   疑在上游 redirect.py（双键 stash dict）与 preflight.py（六键 schema）
   跨 fixture stash 互写的 KeyError → 500 链。runner 域独立切片。
2. Window.origin 快照活性边界：同全局上下文跨源导航（现不存在）会读到
   装载值；支持时需改回惰性方案并另行解决 var 遮蔽面。
