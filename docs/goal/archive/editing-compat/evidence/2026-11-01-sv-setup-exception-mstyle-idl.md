# style-with-value 簇 setup-exception 族收口——物化元素补 style IDL（2026-11-01）

## 0. 结论速览

| 门禁 | 本轮（最终二进制） | 轮前基线 |
|---|---|---|
| fontsize 真页 | **1,867P/555F**（+468P） | 1,399P/1,023F |
| forecolor 真页 | **1,880P/447F**（+492P） | 1,388P/939F |
| fontname 真页 | **1,667P/558F**（+265P） | 1,402P/823F |
| backcolor 真页 | **736P/290F**（+347P） | 389P/637F |
| hilitecolor 真页 | **891P/327F**（+413P） | 478P/740F |
| 五语料合计 | **7,041P/2,177F** | 5,056P/4,162F |

对基线逐语料 **F→P 恰等额（ΔP=−ΔF）、P→F=0、Timeout 0→0**；`reading 'color'` setup-exception 族五语料全清零（grep 计 0）。七位移语料（join 三案 7,518P/0F、ihost 880P/0F、wss 1,104P/0F、ilb 552P/0F、chd 420P/0F、void 72P/0F + 48P/0F）、wcomp 4,022P/1,146F/5T、engine lib 3,642P/1F（s24 已知红唯一）、webview lib 834P/0F、bench-gate PASS——门禁面零回归（gate3 终数见 §5）。

## 1. 现象与立项

style-with-value 五语料（fontsize/forecolor/fontname/backcolor/hilitecolor）大量子测试报：

```
assert_equals: Setup must not throw an exception expected null but got
  object "TypeError: Cannot read properties of undefined (reading 'color')"
```

抛点 = vendored `tests.js:5507` `normalizeSerializedStyle`：`wrapper.querySelectorAll("[style]")` 逐元素读 `node.style.color`——`node.style === undefined` 即抛，整测 setup-exception（每案 return-value 行侥幸 Pass、checks/compare 两行 Fail）。轮前该族占五语料失败主体（基线 4,162F 中约 2,000F，归因前按断言面无法细分）。

## 2. 归因链（探针页两轮，gitignored 复用不入库）

### 2.1 二值化排除（zz-probe-sv-stack VARIANT 开关）

- V-A：剔除 stylewithcss 命令仍败 → 排除 styleWithCSS 模式嫌疑；
- V-B：`xx-small`→`large` 仍败 → 排除字号值面嫌疑；
- V-C：`documentElement.cloneNode` 换浅拼装（绕过整树深克隆）仍败 → 排除克隆路径嫌疑。

### 2.2 逐阶段定位（t10–t13）

- t10：直接 `setupDiv`（容器级 querySelector 取 div）→ span `.style` 健康；
- t11：真实 `runConformanceTest` 透明 spy 逐 actor 扫健康度 → `setupDiv` 后即坏（1/1，格式 bad/total）；
- t13：t10 原样换 `document.querySelector` 取 div → 坏。**文档级与容器级查询是唯一差异**。

### 2.3 形态取证（t15/t16）

- t15：`document.querySelector` 与容器级返回**同一对象**，但 own properties 形状：物化命中 93 props vs pending 形 2 props；
- t16：坏 span `'style' in s === false`，域标记父链 `SPAN(M---)>DIV(M--P)>DIV(-HS-)>BODY(-HS-)>HTML(-HS-)`。

**根因定谳**：`_zwMEl` 物化 plain 形元素整族缺 style IDL——此前 style 仅 `_makeProxy` get trap 的 `'style'` 分支提供；`document.querySelector` 宿主物化命中及其 innerHTML 解析子树全为 M 形 → `normalizeSerializedStyle` 全体抛 TypeError。语料 runConformanceTest 的 doc 级克隆/查询路径恰好全量踩中。

## 3. 修复（part03.js 两 hunk）

1. **`HTMLElement.prototype` 单点 style accessor**（`_zwBuiltNodeChain` 自建链门内——native prototype 不可覆盖，R3270 边界）：get 返 `_zwMStyleProxy(this)`，SameObject 槽 `__zwMStyleObj` 以 `Object.defineProperty` non-enumerable 缓存（wrapper 安装面约定：`__zw*` 槽 gPN own enumerable 零进）；set 为 PutForwards=cssText 语义（`setAttribute('style', String(v))`）。子类原型经 `Object.create(HTMLElement.prototype)`（HTMLFormElement/接口 ctor 表区）全员继承；代理家族 get trap 先于原型链命中，读数不变。`enumerable: false`——同 scrollTop/scrollLeft 先例（R3046 expando 枚举回归钉：proto 面 for...in 零进）。
2. **`_zwMStyleProxy(node)` 帮助函数**（`_zwMEl` 工厂前，~98 行）：纯属性反射面板——读 `getAttribute('style')` 末次非空声明（R3213）、`_styleValueAndPriority` 剥 `!important`（R3193）、camelCase→kebab（`_stylePropName`）、写走同名移除后尾部追加、空值=仅移除、`--` 自定义属性大小写敏感；Proxy get/set 面：cssText/length/getPropertyValue/getPropertyPriority/setProperty/removeProperty/item/其余属性读。

**版本演进**：v1 逐实例 defineProperty（弃——per-instance 成本落在刚被「wrapper 安装面」深水轮优化 −36% 的解析构造面，改 proto 级零逐实例成本）；v2 槽写法 `this.__zwMStyleObj = v` 产生可枚举 own 属性，违反安装面不变式（弃，TaskStop 杀在途门禁批、撤代码重建、全量重跑——证据绑最终工件）；v3 终案 = proto accessor（enumerable:false）+ defineProperty 槽。

## 4. 测试契约更新（part18.rs t8r4）

`typeof el.style === 'object'` 曾是「wrapper(trap-proxy)/代理形」判别惯用法；style IDL 全员化后该判别式失效。t8r4 两臂形判别改域标记，取证数据（临时插桩，已剥）：

| 臂 | 对象 | 取证 |
|---|---|---|
| ② noWire h1 | M 形+pending | `{styleT:object, mEl:true, sel:false, hnd:false, pend:true, matParsed:false, gpn:94}` |
| ① K3 m1 | trap-proxy | `{styleT:object, mEl:false, sel:true, hnd:true, pend:false, matParsed:true, gpn:2}` |

新判别式：`__zwIsMEl === true`（M 形=代理面）vs `__zwSelector/__zwHandle`（trap 应答=wrapper）。两臂数据在修复前后语义均自洽（h1 修复前 styleT=undefined 归 proxy、修复后靠 M 标记归 proxy，不变式不变）。其余 part18 十处 style-presence 探针断言「有 style」继续通过（style 现恒在，存在性检查弱化但 tagName/className 等其余判别保留），未顺手改。R3046 expando 枚举回归钉（for...in 恰含用户 expando 键）由 accessor `enumerable:false` 保真。

## 5. 门禁（最终二进制 gate3 全批；gate2 为 enumerable 修正前中途二进制）

- 五语料终数见 §0（gate2/gate3 逐位一致）；
- 七位移语料：join **7,518P/0F/0T**、ihost **880P/0F**、wss **1,104P/0F**、ilb **552P/0F**、chd **420P/0F**、void **72P/0F + 48P/0F**——对上轮终树电池逐位恒等；
- wcomp **4,022P/1,146F/5T** 计数级恒等；hsem **6,872P/693F/162T**（对上轮 6,869P/695F/163T：+3P/−2F/−1T，总数 7,727 恒等，正向小漂移，style IDL 全员化的合理外溢，无回归形态）；
- engine lib **3,642P/1F**（s24 已知红唯一，红名 identical）、webview lib **834P/0F**、bench-gate（zero-engine 定向）**PASS**；
- 中途二进制教训：enumerable:true 版曾在 engine lib 暴露 +2 新红（t8r4 判别式失效 + R3046 for...in 混入），修正后复绿——门禁红必须逐名定谳，不允许「总数近似」放行。

## 6. 残余失败族定性（对 gate2 五语料 2,177F 断言面普查）

| 族 | fontsize | forecolor | fontname | backcolor | hilitecolor | 合计 | 定性 |
|---|---|---|---|---|---|---|---|
| queryCommandValue before/after | 324 | 227 | 294 | 152 | 176 | **1,173** | effective command value + getLegacyFontSize 值面，下轮主刀 |
| compare innerHTML | 149 | 141 | 139 | 63 | 75 | 567 | 实应用臂缺口（wrap/格式化产物形状） |
| queryCommandIndeterm | 34 | 34 | 80 | 33 | 33 | 214 | indeterm 判定面 |
| queryCommandState | 25 | 26 | 25 | 24 | 25 | 125 | state 判定面 |
| return value | 17 | 13 | 14 | 12 | 12 | 68 | 含表 fixture setup-exception 的 return 行 |
| checks for modifications | 6 | 6 | 6 | 6 | 6 | 30 | 同上 |

IndexSizeError 明细行每语料恒 96（表 `<tr>/<tbody data-start=…>` fixture 的 setup 例外形，双 swc 变体全断言失败，五语料同 fixture 恒定重复）——独立面，与命令语义无关，另立收口。

## 7. 下一步

① queryCommandValue 值面（≈1,173F）：effective command value 立项（`getLegacyFontSize` 字号映射、forecolor/backcolor computed color 反射、fontname computed family）；② 五命令实应用臂 compare innerHTML（≈567F）；③ 表 fixture IndexSizeError setup 形（每语料恒定）；④ jc `?5001-6000` 墙钟异动归因（挂账不变）；⑤ merge FIXME 族（挂账不变）。
