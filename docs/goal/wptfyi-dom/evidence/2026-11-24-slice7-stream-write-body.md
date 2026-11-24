# DC-2 切片 7：位置方法族 + stream 写入体收口（2026-11-24）

## 本刀范围

`crates/engine/src/js_dom_shim/part15-positional.js`（新文件，670 行）：WICG
[declarative-partial-updates](https://github.com/WICG/declarative-partial-updates)
位置方法族实现体——`appendHTML`/`prependHTML`/`beforeHTML`/`afterHTML`/
`replaceWithHTML` ± `Unsafe` 后缀 + `stream*HTML` 次序壳全写入体。挂接面：
part04 get trap / part05 has trap（late-bound `__zwPosHtml`/`__zwPosStream`，
R1835 先例形态）；消毒/解析消费 part13 导出；part12 `createParserOptions`
政策方法（`__zwTtParserOptions` 章）；part03 foreign-doc Comment 原型链
（CharacterData 分派可达）；js_dom_bridge.rs include_str 注册。

## 结果

domparsing 全腿（干净树、probe 已删）：**1094P/441F**（对比 11-23 日志
dc2-s7-positional-domparsing-full.log 797P/738F：**+297P/−297F，309 面修复、
0 真回归**——28 面新红中 16 为 zz-knife 诊断 probe（已删）、12 为空转型假绿
翻红（见下））。

分页（11-23 基线 → 本刀）：

| 页 | 基线 | 本刀 |
|---|---|---|
| positional-methods.html | 33P/1154F（切片 6 前基线，方法未实现态） | 348P/0F 全绿 |
| with-parser-options + with-trusted-types 合计 | 31F | 197P/0F 全绿 |
| stream-html.html | 12F（3P） | 15P/0F 全绿 |
| stream-html-unsafe.html | 13F | 15P/0F 全绿 |
| stream-html-with-trusted-types.html | 2F | 0F |
| stream-html-run-scripts-option.html | 4F | 8P/0F 全绿 |
| stream-html-script-safe.html | 3F | 6P/0F 全绿 |
| stream-html-sanitizer-safe.html | 30F | 2F |
| stream-append-html.html | 14F | 1F |
| stream-append-html-unsafe.html | 11F | 1F |
| stream-html-custom-element.html | 4F+页超时（37 案未报） | 4F（超时解除） |
| stream 家族合计 | 466F+超时 | 230F（401P） |

stream 残余 230F 全部挂账（见「挂账」）。

## 根因清单（本刀修）

1. **活数组 splice 移位**（positional 1154F 主根因）：解析 body 内部活数组被
   mock 容器 appendChild 预摘（splice）移位，逐索引消费把后续 kid 静默漏收。
   修 = `_p15ParseInert` 消费前 `kids.slice()` 快照。
2. **engine 动态启动**：解析产物 SCRIPT 物化进活文档被引擎插入臂自动执行
   （runScripts=false 案）。修 = s118 同章 `__zwFragNoExec` 递归盖章（part05
   _uS118 先例），执行只走 options 门。
3. **replaceWith ref 滞留**（18F）：`refNode.remove()` 经引擎 registry 寻父，
   mock 父形态引擎侧无父 → lenient no-op。修 = 容器原语 `removeChild` 主臂
   （mock 按身份 splice 语义等价）+ `remove()` 回落。
4. **Symbol 写入不拒**（stream-html）：本宿主 `String(Symbol)` 非抛（弱化面）
   → `_p15DomString` 独立 typeof 门（WebIDL DOMString ToString 语义）。
5. **style 流送不生效**：parse 产物 text/comment kid 落引擎代管父静默丢弃
   （probe12：appendChild 零抛、childNodes=0）。`__zwHandle` get trap 域外
   隐藏（probe14 双 null）→ 反转判据：非 mock 族容器一律经 document 原语重建
   （`_p15Materialize`，probe14 t2 实证落位）。
6. **Unsafe 流默认脚本门**（stream-html-unsafe「should execute scripts when
   connected」）：Unsafe 变体默认允许、`options.runScripts:false`/TT 覆盖
   false 显式关；safe 变体默认禁止、`runScripts:true` 显式开（TT 不能开——
   runScripts-cannot-be-turned-on 页持绿）。执行挂**逐冲刷**（三写案
   `</script>` 补全冲刷即执行，批次本地收集防跨冲刷重执行）+ close 时连接性门
   （disconnected 案；ShadowRoot face 走 shadow-including 宿主连接性）。
7. **RAWTEXT 未闭元素尾持**：`<script>` 开标签后数据到 `</script>` 前不入
   冲刷批次——否则引擎 inert 解析对未闭合 RAWTEXT 产空文本 script、执行域
   拿不到源（三写案 flag 恒 false 根因）。script/style/textarea/title 四名。
8. **SCRIPT 容器 safe inert**（script-safe 3F）：engine 对已连接 script 的
   文本子追加动态启动执行（probe20 t1 直 appendChild 实证，与流无关）——
   safe 变体对 SCRIPT 容器的 text kid 不物化（`_p15Materialize` safe 参数），
   源不落位即不执行。

## 假绿翻红说明（12 面，非回归）

`stream-html-sanitizer.html`「sanitizer shouldn't allow escaping template」
×12 方法：pre-knife 写入体为桩，内容不落位 → `querySelector("template")`
空过；本刀内容真落位后暴露 `replaceWithChildrenElements`（模板解包）消毒
语义缺失。同簇「sanitizer takes effect after foster parenting」等深配置面
一并挂账（见下）。

## 挂账（下一步）

- **engine 注册路由**（~200F 大头：stream-positional 36F、insertion-logic
  24F、TT 簇 18F、shadow-root 18F、sanitizer 深簇 120F）：parse 产物落
  engine 代管父只入 JS 视图（childNodes），engine 视图（innerHTML getter、
  querySelector）不可见（probe15 双臂实证）。需协议级
  `InsertAdjacentHtmlOnHandle` 突变（enum + apply 臂 + shim 侧）——
  zero-protocol + apps/renderer 两端同步；另 engine 现有 detached 无 selector
  的 `beforeend` IA 丢文本子（probe17 t1 `<b id=k1></b>`）。锚定校验（ref 摘
  除/换父 → write 拒 HierarchyRequestError，insertion-logic 12 案）随该刀。
- `TrustedParserOptions` 构造器 globalThis 曝光（pass-trusted 页 10F
  ReferenceError；spec TT 集成面）。
- sanitizer 深配置簇（120F）：replaceWithChildrenElements/foster parenting/
  adoption agency/模板逃逸/流中变异。
- defer/async 脚本通道（src/type FIXME，part15:306）、img-ctor CE 面三案。

## 验证

- 全量 domparsing 腿 ×2（probe 含入态 + 干净态），test-guard 包裹
  （--per-proc-mem 4 --total-mem 8 --time-limit 1800）。干净腿日志：
  `dc2-s7-clean-domparsing-full.log`。
- `cargo fmt --all -- --check` 零 diff。
- engine lib 测试 **4252P/0F** 持位（test-guard 包裹）；`cargo clippy --workspace
  --all-targets -- -D warnings` 零 warning（test-guard 包裹）。
- probe 页 zz-knife-s7-probe1..20 已删（诊断留证见本档根因清单与 probe 描述；
  原始页面字节存 .acceptance/dps/ 备份链）。
- 注意：本档洁净腿日志摄于 part15 注释订正前（行为中性，JS 注释语法），
  终版字节以提交 md5 为锚，另有 stream-html/-unsafe 两页终版字节复验针。
