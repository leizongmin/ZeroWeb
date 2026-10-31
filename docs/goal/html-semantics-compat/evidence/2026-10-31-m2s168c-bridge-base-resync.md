# M2-S168c 桥基座失配收口（stylesheet-media-change-no-fetch 翻绿）

**日期**: 2026-10-31
**刀位**: s168 候选①——commit2（`bc767664e`）遗留挂账案 `stylesheet-media-change-no-fetch` gCS 全空勘根 + 修复。s168b 账本已证 88ca4f5db（commit 1）已然，非 commit2 回归。

## 症状与最小复现

动态 `createElement('link')`（id-less handle-only）+ rel=stylesheet + href 取回完成后，文档既有**动态创建**元素的计算样式整体丢失：`getComputedStyle(p).color === ''`（空串，非错值）。静态 link 变体 Pass；同步 gCS 读（drain 前）Pass——**drain 后窗口专属**。

探针矩阵（`zz-s168c-probe.html`，验后删）五案钉边界：p1 plain link append 同步读 P；p2 rel-only P；p3 sheet link 同步双读 P；p4 **+1 tick 读 F**（`'' `）；p5 drain 后 fresh 元素 P（mutation 尚在 pending，重放臂命中）。复现不依赖 link 语义本身——任何「drain 消费全部 mutation 后读动态元素 gCS」形态皆触发。

## 勘根链（插桩钉死，四步）

1. **engine 视图重建臂**（`computed_style_cache.rs` computed_style_by_handle）：drain 后重建打点 `applied=25 mlen=25 start=25 nodes=0`——重放区间空（游标==队尾），视图 = parse(基座)。
2. **基座内容**：miss 臂打点 `html_link=false html_p=false`，html 尾部即探针文件**自身源码**——基座是**初始页面源码**，不是 drain 后权威重序列化。handle miss → forward 正置表 selector `html:nth-child(1) > body:nth-child(2) > p:nth-child(9)`（位次型）在无 p 的视图 doc 上锚定 miss → `return String::new()`。
3. **webview apply 臂**（`apply_pending_shared_mutations`）：`ok tail=25 snap_some=true handles=8`——apply **成功**，`html_snapshot = outer_html(doc)` **含全部动态元素**，`self.cached_html = mutated` 已执行。
4. **失配定位**：gCS 回调读的是**回调闭包捕获的 `dom_html` Arc**——其内容停留在武装时点的初始源码。apply 只更新 `self.cached_html`，Arc 内容不换；基座刷新此前只发生在「重注册换新 Arc」站点，而 runner 探测环 `execute_script` 武装一次化（form-url-encoding CI 回归修，同日落地）后 **apply 到下一脚本之间无任何重注册**——s143 契约「共享队列注册站的查询基座 = 最新 cached_html（已含已应用前缀落地效果）」的前提（游标推进必经重注册，见 `MUT_APPLIED_CURSOR` 文档）被打破，失配窗口常态化。selector 基 gCS 不受影响的原因：元素在初始基座里本就存在，属性/样式面走 latest-wins 队列 overlay；只有 handle 元素（createElement 产物）必须靠基座物化。

## 修复（WebView 侧 2 文件，engine 零改动）

- `webview.rs`：新字段 `dom_html_bridge: Option<Arc<Mutex<String>>>`——最近一次 `register_dom_callbacks` 站点发布的 Arc（同名单重注册后者胜），6 个注册站登记；`sync_dom_html_bridge()` 内容差分就地写 + `bump_dom_view_gen`（R358「快照 Arc 无法换装、内容只能就地写 + 代际项进缓存键」模式的 apply 侧补全；内容相同跳过防空转换代）。
- 同步点四处，各配不双计论证：
  - `apply_pending_shared_mutations` 成功臂（游标发布配对——基座含 `[0..applied)` 效果与重放区间 `[applied..count)` 空同时成立）；
  - `apply_mutations_subset`（两步 apply 的结构/样式子批，同配对）;
  - `sync_render_after_native_dom`（native 写不经共享队列，无重放双计面）;
  - `refresh_if_html_changed` **门控**：仅队列无 pending（`applied >= len`）时同步——shim 序列化真相已含 pending 效果，pending 非空时重放会双计（take_probe 在 flush 后调用恒满足门）。
- perf 权衡：`bump_dom_view_gen` 只落 apply/native 边界，脚本 turn 内视图链式增量（R-baidu3 防重解析风暴）不受影响；每 apply 轮至多一次全量 parse，相对该轮已有的全量 render 为噪声。

## 验证

- check / fmt / guarded-clippy workspace 全绿；product-smoke 6/6 struct-check PASS。
- 探针 `zz-s168c-probe.html` 5P（p4 F→P），验后删。
- 靶面 `stylesheet-media-change-no-fetch` **F→P**。
- 目录 `the-link-element` **115P/0F**（基线 114P/1F，净 **+1P −1F**，P2F 0）。
- 目录 `document-metadata` **215P/1F/9T**（基线 214P/2F/9T；F/T 集合 diff 仅目标案 F→P，9 Timeout 逐案同上轮 tentative/csp 族既有）。
- 回归面 `implicit-submission` 3P/0F、`image-map` 9P/0F（s143/尾簇 13 视图缓存敏感面零回归）。
- engine lib **3612P/11F**：失败名单与 s168b 基线 11 案同集（fetch 族 9 + ua_display_li + style_sheet_from_handle_r113），零新增，净 +1P。
- webview lib **821P/2F**：双 F 既有——`test_document_scrolling_element_r3259`（无 doctype html 断言 CSS1Compat，spec 应 BackCompat）stash 复证 HEAD 同败；`proxy_mode_decodes_png_correctly` 为 image-decoder 进程 spawn 环境问题。

## FIXME 挂账（不变 + 新记）

- forward 正置表 selector 位次脆弱性（s166 已知）：跨批 handle 锚定依赖 parse∘serialize 往返位次稳定，后续结构漂移仍可 miss（本刀不扩大）。
- s168b 挂账不变：`__zw_query_all` 枚举排除 shadow 子树；`@handle` 形宿主 collector 代际文档锚定。
- webview `test_document_scrolling_element_r3259` 断言与 spec 相悖（无 doctype 应 quirks）——待勘根归位（归 shim compatMode 判定或改测试锚）。
