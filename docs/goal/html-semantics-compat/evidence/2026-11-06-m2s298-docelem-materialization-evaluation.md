# M2 s298 — 宿主 documentElement 物化缺口评估销案（勘根定性修正：物化链健康——apply 正确落 html、forward map 有条目；残面 = 序列化往返 html→body 位置漂移，spec 解析行为、渲染位置等价、零语料钉面）

日期：2026-11-06。切片号 s298（承 s285 挂账「宿主树 documentElement 父插入
物化缺口根修」+ s293 挂账②并案评估；s297 后候选①）。纯评估零源码改动切片
（探针页 gitignored 语料区完测即删；探针日志 md5 留存）。

## 勘根（探针 v1-v3 + 管线读码）

**探针梯**（zz-s298-docelem-probe.html，the-img-element 域 FILTER solo；
v1 单 test 首败掩后续 → v2 逐断言拆独立 promise_test → v3 落点 dump）：

- shim 反链：`img.parentNode.nodeName === 'HTML'` ✓——s285「shim 树有」
  实为 child→parent 反链；父向 childNodes 视图**不含**该子（v2 Fail 面）
  ——'html' sel 视图 quirk，s130b 探针先例「documentElement 级元素 qSA
  异常返 0」同族。
- 宿主落点（v3 dump 定谳）：`body>img=[#zwprobe-img]`、`html>img=[]`、
  `head>img=[]`（div 同型）——节点**在宿主树**、**不在 html 之下**。
- forward map：`fwd-img=[#zwprobe-img]` 有条目——s285 死锁链第一环
  （「forward handle map 无条目」）在当前 HEAD **不成立**。
- body 控制组三面全绿（落点/forward map/查询）。

**机制链**（管线读码闭环）：首次 apply 正确——`AppendChild{parent_
selector:'html'}` → `find_by_selector('html')` 命中根元素（part01 嵌套
测试族钉裸名 `html` 键）→ forward map 同批发布；runner 逐拍结构性刷新以
**shim 序列化 html fresh-parse 重建**（testharness.rs:11865-11872「shim
序列化 html 驱动的结构性刷新 / render 每帧 fresh-parse」）——html 级子序
列化形 `<img>` 居 `</body>` 后，解析器 after-body 插入模式**收养进
body**（HTML 解析规范行为；真浏览器 outer_html 往返同型有损）→ 宿主稳态
视图 body>img。s285「宿主 apply 静默丢弃」定性修正：apply 不丢，丢的是
**跨重建的位置保真**。

**语料消费面盘点**：全树结构插入 documentElement 消费面仅 empty-src-no-
current-request.html 一页（2 面）——s296 全树 B 侧 **2P 绿**（s285 shim
判近臂 + 等待窗 drain-only 补偿维持）；其余 documentElement 消费皆
className/classList 属性面不入结构链。零语料面断言 documentElement 子
placement / childNodes 视图。

**残面定性**（均零语料钉面）：①html→body 位置漂移——宿主查询视图永不
可见 html 级子（`html > img` 恒空）；渲染位置等价（真浏览器 html 级图渲
染于 body 内容之后，body 尾子同位；empty-src 语义面只钉事件序列）；②
shim 'html' sel 视图 childNodes/qSA quirk（v2 Fail 面 + s130b 先例）。

**根修评估**：位置保真根修 = 管线重建架构面（保留 cached_doc 不做
serialize/parse 往返，或重建后按 shim 记忆重挂 html 级子）——共享渲染主
路径，爆炸半径全树，收益仅假想 future 语料（无钉面）。准则 #2（不做需求
外功能）/#5（精准修改）不入片。s293 挂账②前提（「根修随 s285 物化缺口
根修一并评估」）随定性修正消解——sweep 身份链 gap 为 shim 内部键位事项
（探针自创形，corpus 无此形态），同挂账①不定修注记处理。

## 结论

- s285 挂账「宿主 documentElement 物化缺口根修」**销案**——物化链健康
  + 残面零钉面 + 根修架构级不成比例；勘根定性修正入账（s285 勘根①
  「静默丢弃/forward map 无条目」两表述按本账修正）。
- s293 挂账②并案**销案**（前提消解，不定修注记承继）。
- 常设注记（不定修，后续语料钉此两形再立候选，落点=重建后重挂或 shim
  视图统一）：①宿主查询视图 html 级子永盲；②shim 'html' sel 视图
  childNodes/qSA quirk。

## 下一步候选

1. plain-src 运行时 img 触发缺口根修（s293 挂账①——需先立 corpus 钉面
   或确认 runner 生命周期 sweep 已覆盖实际断言域）。
2. PNG eXIf 方向标签支持（png crate exif_metadata 面——无语料消费面，
   s296 FIXME 已注记，低优先）。
3. css image-orientation 属性渲染面移交 rendering-compat 候选（s296 账）。

## 证据

- 探针三轮日志（evidence/logs/，gitignored 本地留存 + md5）：
  s298-probe1.log（v1 首败掩后续存证）、s298-probe2.log（v2 十断言拆分
  ——反链 ✓/父向视图 ✗/宿主 any 命中/html> 不命中/forward map 有）、
  s298-probe3.log（v3 落点 dump——body>img 定谳）。
- md5：probe1 49a40d2c、probe2 10d5d96f、probe3 113e0768。探针页
  zz-s298-docelem-probe.html 完测即删不入库。
- 代码锚：js_dom_bridge.rs:1717-1735（AppendChild apply lenient no-op
  臂——本次实证未触发）、pipeline/mod.rs:2256-2279（apply + outer_html
  snapshot）、testharness.rs:11840-11872（flush pending + shim 序列化刷
  新 + fresh-parse 注记）、js_dom_bridge_tests/part01.rs:280-311（裸名
  html 键钉测）。
- s285/s293 账本（挂账原文）；s296-fulltree-b.log（empty-src 2P 绿、
  the-img-element 域 F 线全清点）。
