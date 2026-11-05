# M2 s299 — iframe 工厂 stub 收养图 src 恢复 + reftest harness `<base href>` 图路径（双面根修：document-adopt-base-url ✗16%→✓ 0.00%，document-base-url 同族 ✗16%→✓；全树 reftest A/B 净 +2P 零真回归，the-img-element 域 A/B 零漂移）

日期：2026-11-06。切片号 s299（承 s293 挂账① 钉面落地 + s298 候选①）。
双层根修：①engine shim R368 盖章点 own-property IDL src 恢复；②reftest
harness build_image_cache 的 `<base href>` 相对引用前缀。探针页 gitignored
语料区完测即删（日志 md5 留存）。

## 勘根（五轮插桩探针梯 + 管线读码闭环）

**钉面**：document-adopt-base-url.html（reftest ✗16.00%，s292 期 5 份日志
一致，零账面记录）。页形：`<base href="resources/">` + `<iframe></iframe>`
字面 + script 内 `iframe.contentDocument.createElement('img')` →
`i.src = "cat.jpg"` → `document.body.appendChild(i)` → `iframe.remove()`。
ref = document-base-url-ref.html（`<img src="resources/cat.jpg">`）。

**探针梯**（zz-s299-adopt-probe.html，the-img-element 域 FILTER solo；
v5 err 注 → v6 diag → v7 hookRan → v8 entry/plainArm/hseq/isMEl → v12
终态五测）定谳三事实：

1. `isMEl=undefined`——iframe-doc createElement 产物**不带** `__zwIsMEl`
   印章。读码闭环（part05）：`_zwMakeIframeDoc` 建 detached doc 后在
   :2804 **替换** doc.createElement → `_zwIframeCreateElement`（:3114，
   纯对象字面量工厂——仅 `id` 有 accessor，无 `_zwMEl` 盖章无 s294 src
   accessor）。此前评估误以 detached 工厂（part03 :25605 `_zwMEl`+s294
   setter）为 iframe doc createElement——两工厂并存，iframe 路径走替换版。
   因此 `img.src=` 赋值落 **own data property**（非 accessor 捕获），attr
   store 不落（getAttribute('src')=null、`i.src` own property 读回）。
2. `h2=__n2`——append 期**既有 R368 盖章臂**（part04 :7463）已为 stub 分配
   host handle（`__zw_create_element`）并接宿主树（handle-child 路径
   `__zw_append_child` wire）；R4004 属性快照回填源 = `getAttributeNames()`
   /`_attrs`——own property src 不在两源 → **host img 无 src 属性**。
3. s287 触发臂（:8441）`__zw_has_attr_handle(handle,'src')==='1'` 门不亮
   → 零 fetch/渲染。shim 序列化视图含 `<img>` 无 src → reftest 渲染空。

**初版方案否决存证**：收养点 `_wcRebuildAsHandle` 物化 hook（v5-v8 插桩
期实现于 plain 分支后）——门 `__zwIsMEl===true` 永假（事实 1），且若改门
放行会在 R368 已物化之上**双物化**（第二 host 元素）。hook 全删。真根修
= R368 盖章点单一恢复。

**第二层缺口**（probe 全绿后 reftest 仍 ✗16% 引出）：v11/v12 探针证 shim
序列化视图干净（`<img src="cat.jpg">` 在、`<iframe>` 已移除）→ 渲染面读
码：harness `build_image_cache`（reftest/resources.rs）文件查找一律
`base.join(url)`——**`<base href>` 不参与** → `cat.jpg` 找
the-img-element/cat.jpg（不存在）；ref 页无 base 直找 resources/cat.jpg
（存在）→ test 页图零加载。ImageKey=raw 串 hash（painter 对齐），路径面
与键面分离。

## 施工（双面各一处精准修改）

1. **engine**（part04.js R368 臂内、R4004 回填块后）：img + own property
   src（hasOwnProperty + string 非空）+ host attr 缺失门 →
   `__zw_set_attr_handle(handle,'src', raw)`。spec 链：reflect setter 写
   内容属性**原串**（set 期不解析），fetch 期按 node document base URL
   解析；adopt 不剥属性——收养后 node document = 主文档，原串按主文档
   base（含 `<base>`）解析（即钉面语义）。恢复点 = R368 身份盖章点（此后
   host 面 canonical）；accessor 族（s294 落 attributes）已由 R4004 回填，
   host 有 src 不覆盖。s287 触发臂既有通路自动点亮 fetch。
2. **harness**（reftest/resources.rs）：`extract_base_href_prefix(html)`
   （首个 `<base href>`，相对形态限定——`/` 开头/scheme/`#` 返 None 走既
   有站点根解析）+ build_image_cache 文件查找 base 前缀路径优先（落空回
   落 primary）。ImageKey 键面零变化；无 `<base>` 页短路零行为面外移。
   单测 `base_href_prefix_resolves_relative_img`（30/30 模块测试全绿）。

## 验证（全矩阵）

- 探针 v12 五测全绿：R368 盖章 ✓ / host src attr 回填且 raw 串保真
  （get_attr_handle='cat.jpg'）✓ / 收养图 settle（snapshot/currentSrc）✓
  / 序列化视图 img 在 iframe 无 ✓ / control 面 ✓。
- reftest solo：document-adopt-base-url **✗16.00% → ✓ 0.00%**。
- **全树 reftest A/B**（A=HEAD 15967P/1045F，B=with-fix 15969P/1043F）：
  真实 F→P 恰 2——document-adopt-base-url + document-base-url（同
  `<base>` 族，勘根期未列、harness 修复顺带收编）。翻转对
  table-anonymous-objects-127（F3.18%→P）/-150（P→F2.67%）经 solo 三连
  （B 二进制 3/3 全 ✓ 0.00%，两页均无 `<base>`）定谳时序 flake（s292
  跨跑状态矩阵先例），净零。**零真回归**。
- the-img-element 域 testharness A/B（A=1012P/19F/1T，B=1017P/19F/1T）：
  B 含探针 5P（A 侧探针已挪出）——1017−5=1012 恰合；F/T 计数与失例文件
  集全等（current-pixel-density/basic+error、sizes-auto、
  svg-img-density-vs-zoom、not-broken-while-load-task-scheduled——皆既有
  红线）。**零漂移**。
- 消费面保绿：Node-appendChild.html 11P/0F（Adoption orphan/non-orphan
  两案绿维持——R368 盖章后 R177 门 `!__zwHandle` 跳过，行为面本就经 R368
  传播臂）；scoped-registry-define-upgrade-criteria A=1P/13F=B（13F 既有
  红基线，钉面消费者 1P 保持）。
- harness 单测：reftest::resources 30P/0F（含新增 base 前缀测试）。

## 销案与残面

- **s293 挂账①（plain-src 运行时 img 触发缺口）销案**：其钉面即
  document-adopt-base-url（本轮落地）；假设的 detached 延步/触发臂挡形
  在 control 面已被 s293① 探针证伪，语料无其余钉形。
- 残面注记（不定修，语料钉形再立候选）：①工厂 stub own-property 族
  hazard 不止 img.src——`input.value`/`img.srcset` 同形（`_zwIframeCreate
  Element` 仅 id 有 accessor）零语料钉面不扩（准则 #5）；②`<base>` 仅接
  入 build_image_cache 图路径——CSS url()/video poster 同理未接（语料钉
  形再扩）；③reftest 全跑 table-anonymous-objects 族 2-3% 近阈值 flake
  对（127/150 本轮再证）常设注记承继。

## 下一步候选

1. PNG eXIf 方向标签支持（png crate exif_metadata 面，s296 FIXME 已注记，
   低优先）。
2. css image-orientation 属性渲染面移交 rendering-compat 候选（s296 账）。

## 证据

- 日志（evidence/logs/，gitignored 本地留存 + md5）：s299-probe12.log
  （v12 五测全绿；probe11 字节同——两版探针同一测试名全绿输出确定同形）、
  s299-imgelem-a/b.log（域 A/B）、s299-reftest-full-a/b.log（全树 A/B）、
  s299-reftest-solo2.log（钉面 ✓ 0.00%）、s299-node-appendchild.log、
  s299-scoped-registry-a.log。md5：probe12 5890559d、imgelem-a f025720a、
  imgelem-b 635da0bf、reftest-full-a 1df1cfe4、reftest-full-b 2fae9f9f、
  reftest-solo2 f9ddacd7、node-appendchild a24bd95e、scoped-registry-a
  c93deead。探针页 zz-s299-adopt-probe.html 完测即删不入库。
- 代码锚：js_dom_shim/part04.js R368 臂 s299 恢复块（R4004 回填后）；
  reftest/resources.rs extract_base_href_prefix + build_image_cache 前缀
  查找；part05.js:2804（createElement 替换点）、:3114（_zwIframeCreate
  Element）；part03.js:25605（detached 工厂——非 iframe 路径，勘根澄清）。
- s292/s296/s298 账本（承前挂账与 A/B 协议先例）；s292 日志（钉面 ✗16%
  五份一致存证）。
