# M3 R100：真实 Vue 3 端到端首切片（mount 落地 + 响应式 + 事件）

**日期**: 2026-08-18
**里程碑**: M3（真实 SPA / Web Components 端到端验收）
**DC-2 第一项**: SPA 框架（React/Vue/Svelte 之一）代表性页面可真实加载、渲染、交互

## 资产

- `tests/integration/fixtures/vue/vue.global.js` — vendored `vue@3.5.13` global build（jsdelivr 快照，MIT，~550KB）
- `tests/integration/src/e2e_vue_library.rs` — 两组常驻 e2e（`make test` 内运行）：
  - **组 A `vue_mount_lands`**：`createApp({data, template}).mount('#app')` → 模板编译 `{{ msg }}` → post-flush 断言真实 DOM `html:<p class="msg">Hello Vue!</p>`
  - **组 B `vue_reactive_and_event_lands`**：`@click="inc"` → post-drain dispatch click → `t0:0` → `text:1`（handler + 响应式 patch commit）

## 装载方式发现

bundle **不 inline 进 HTML**：HTML tokenizer 的 script data double-escaped 状态会把 bundle 字符串字面量中的 `<!--` + `<script` 组合解析成双转义模式，吞掉真正的 `</script>` 闭合（spec 行为，Chrome 同款；实证 extract 把 bundle + 后续 script 并成一个 563KB 脚本）。真实页面从不 inline 大 bundle（Vue 官方部署一律 `<script src>`）。装载序 = 外链 bundle 的真实执行序：① onerror 捕获脚本（inline，经 `run_page_scripts` 完成 shim 注入 + document 建立）→ ② bundle → ③ 页面脚本（均经 `execute_script_with_dom`）。

## 四层根因修复（探针驱动）

1. **`generate_dom_api_polyfill` 覆写 document**：每次 `execute_script_with_dom` 前置的最小虚拟 DOM stub 覆写 `globalThis.document`——execute 路径上 getElementById/body 视图全空（Vue mount 找不到宿主元素的根因）。改幂等安装：shim document 带 `__zwShimInstalled` 标记时不覆盖（`part06.js` + `dom_bridge.rs`）。
2. **execute 路径不应用 DOM 变更**：`__zw_*` 回调未注册 → JS 侧写静默丢失（mount 后 host 侧查询全 miss，无任何报错——静默丢弃比显式错误更隐蔽）。镜像 `run_page_scripts` 机制：注册回调 → 执行 → apply 变更 + 快照同步。**共享 append-only mutations 队列**（`WebView::shared_mutations`）：三注册路径同一 Arc，读回调天然看全量历史（HEAD「未清空 Vec」语义的显式化），apply 按 `applied_mutations` 游标只取新增尾部（重放会重复建节点）。
3. **跨 execute handle 身份**（SPA 事件链命门）：
   - `HANDLE_COUNTER` thread_local 单调：旧版每次注册从 0 重启 → 跨注册 `__n0` 碰撞 → `_wrapHandle` 经 `_proxyCache` 返回**旧元素 proxy**（lit awaited continuation 错挂旧 registry，首渲染插值丢失实证）。
   - `selector_handle_map` 持久反查表：apply 后 merge handle→selector（倒置），`__zw_handle_for_selector` 回调 + shim `_zwQueryWrapIdentity` 在 **query 返回点**（document.querySelector / 元素级 querySelector）命中时返回原 handle proxy——Vue `vm.$el` 持有的 proxy 与 `querySelector('button')` 返回的同一 identity，@click invoker（注册在 handle proxy listener key 下）可达。R77 验证的反查三件套形态，限定 query 返回点（R77 教训 #7：全局 `_wrapSelector` 前置反查波及一切 sel 包装路径）。
   - `MUTATION_HISTORY` 线程本地已应用历史 + `FORWARD_HANDLE_MAP` 正置表：读回调（`__zw_get_text_handle`/`__zw_get_tag_handle`）当前批 miss 时 latest-wins 历史回落 + handle→selector→快照回落（双层）。
   - `apply_dom_mutations_full` 持久 handle 解析：ephemeral map miss 的 handle 变体先查 `persistent_nodes`（NodeId 直达**预植**——`SetTextOnHandle` 等无 selector 翻译变体靠此解析）再查 persistent selector 表（8 变体翻译成 selector 形态重入队）——跨 execute 旧 handle 的写 mutation 不再 `unknown handle` 硬错中止整批。pipeline `persistent_handle_nodes` 跨 apply 存活；`render_html` 换代（slotmap 全灭）时按 `handle_selectors` 抢救重锚定；apply 失败先放回 `cached_doc`（旧 `?` 早退把 take 出的 Rc drop——一次坏 mutation 拖死整页后续脚本）。
4. **SVG 接口构造器缺失**：Vue runtime 挂载期 `resolveRootNamespace` 读 `container instanceof SVGElement`——ReferenceError 使 mount 中止。补 ~36 个 `SVG*Element` stub（`SVGElement`→`Element.prototype` 链 + 具体接口→`SVGElement.prototype`）。

## 顺带修复（A/B 捕获/解锁）

- `cloneNode(deep)` handle 容器递归复制 `_handleChildren` 子树（WPT nodes `Node-cloneNode "node with children"` Fail→Pass）
- `replaceChild` handle-handle 形态 JS registry 原位替换（splice + 反链 + childList record + 连接态传播）
- 结构伪类白名单解析+求值（`first/last/only-child`、`nth(-last)-child(an+b)`、`empty`、`not(simple)`、`checked`）——detached 容器子树 pseudo 查询此前全空
- 纯文本 innerHTML 不入 `_handleChildren`（`_zwRegisterTextEl` 已建本地视图，textContent 融合两源双计——`'Hello WorldHello World'` 实证）

## 验证

- **A/B（clean-HEAD `eef1337f0` 重建二进制）**：nodes 6568→**6569P（净 +1 零回归）**、collections 48=48、traversal 1595=1595、events 236=236 per-case 逐字节一致。master.md 旧记 6673P 为 R99 时点基线（并行流 landing 后基线漂移）；name-validation 输出含二进制控制字符致 grep 计数噪声已核实排除（5 次复跑稳定一致）。
- **单测矩阵**：engine v8 2205 / quickjs 1431 / integration 783（+2）/ webview v8 602 / quickjs 555 / script-sandbox 76 / wpt-runner 109 / renderer 134 全绿；fmt 无 diff；clippy 双矩阵零警告。
- **product-smoke**：23.61% = clean-HEAD 同值（ZRG-2026-08-17-01 已归因渲染流 hmtx 既存红灯，oracle re-capture 属用户决策）——非本切片回归，struct-check PASS。
- **webview 集成测试语义更新**：`execute_script_with_dom` 路径不再覆写 shim 后，setTimeout/setInterval 走 shim 真异步语义（spec：宏任务延迟执行）——3 个旧 stub 同步语义断言按 spec 纠正（回调经第二次 execute 读数）。

## 教训

1. execute 路径的「静默丢弃」类 bug（回调未注册）比显式错误更隐蔽——Vue mount 后 host 侧全 miss 无任何报错，探针对比 run_page_scripts vs execute 行为差异才定位。
2. 跨 execute 元素 identity 是 SPA 事件链的命门：selector↔handle 双向持久表（query 返回点反查 + apply 持久翻译）是 polyfill 架构下的最小可行形态。
3. stash A/B 必须重建二进制（R79 教训第四次实证——JS shim 嵌入产物）。
4. WPT 用例名可含原始控制字符（`\x0b`）——grep 管道计数需 `-a` 防 binary 误判。

## DC-2 进度

- 第一项（SPA 框架）：**实质达成**（Vue mount/响应式/事件三段，常驻资产断言可复现）
- 第二项（Web Components）：**实质达成**（R90-R99 lit 全链闭环）
- 剩余：hydration/reconciliation 深场景验收 + QuickJS feature 同页对齐（DC-7 联动）+ 资产化收尾
