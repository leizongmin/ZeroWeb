# M2 切片 413 — 声明式 CSS module parse 期 map 注册 + import 命中免 fetch + kind 固定（tentative/ 11 文件全绿收口）

日期：2026-11-12（承 s412 下一步首位：tentative/ import 簇——basic-import/fixed 2 file-T + 3 allowed 文件 6 NotRun）
commit：见 git log s413 提交

## referent 与勘根

driving 面：`document-metadata/the-style-element/tentative/` 11 文件余红——basic-import/fixed 2 file-T（loaded:false）+ 3 allowed 文件（csp-allowed/nonce-allowed/script-src-allowed）6 NotRun（t1/t2 `.done()` 在 module script 内、import 不解析则永不执行）。s412 已勘定底座厚度：s249 fetch-backed CSS module 全链在库（`__zw_mtype=css` 标记取回 → shim CSSOM → CSSStyleSheet default 导出）+ adoptedStyleSheets 汇总下发（computed_style_cache R-t2qq 槽）。本切片勘根细化四点：

1. **kind 固定的存活载体**：fixed.html 8 子测钉「type/specifier 移除、innerText 改写、removeChild+重插、普通 style 不可转 module」——语义为 fixed-at-parse-time。变异路径 `apply_dom_mutations_to_html` 逐批 re-parse 序列化快照（js_dom_bridge.rs:3180）、`render_after_deferred_batch` 同走 `render_html(&snapshot)`——**NodeId 键旗标在 re-parse 即死**。选 html TEXT 载体：`data-zw-declmod` 标记属性（load_html parse 期插入开标签）——跨 re-parse 免费存活、无 type=module 元素时零成本、单门点覆盖 collect_stylesheets 与 collect_interleaved_external 双路径；既有内部工件先例（zw-pseudo/zw-inline）。备选否决：内容清空（fixed t3 innerText 先重加红内容）、attribute 启发（t6 移除 specifier 后仍须认得）、selector 注册表（12+ 调用点穿参 + 无 id 结构选择器脆面）。
2. **级联门现状**：`style_sheet_if_eligible` ①type 门已排除 `type=module`（非空非 text/css → None）——初始不入级联免费（basic-import 初始 rgb(0,0,0) 面）；缺的是 **type 移除后不复活**（移除后元素表面转普通 style 形，type 门放行）→ 新增 ⓪ 标记门（先于 type 门）。
3. **`.sheet === undefined` 判字面**：shim 既有 type 门对非法 type 返 **null**（style_type_html 锚）；basic-import:19 显式钉 `undefined` → module 专用臂（trim + ASCII 大小写不敏感，与 s412 `is_module` 同判字风格），非法 type 其余值维持 null。
4. **map 键一致性与裸 specifier**：注册键 = `resolve_document_url(page_url, specifier)`；查找键 = `resolve_document_url(parent_url, import spec)`——语料 import 全部内联模块承载（parent = page URL）恒一致。语料 specifier 全裸值形态（`"foo"`）→ 命中臂置于 `collect_module_deps_recursive` 顶部、先于 s87 裸 specifier 检查（未命中裸值仍落 DepResolution，负例面不松动）。CSP-blocked module 不注册（阻止者不可 import——安全面；s412 blocked ordinals 复用）。

## 施工（四臂）

- **engine extract**（`pipeline/extract.rs`）：`StyleElementCsp` 增 `specifier`（缺省/空 → None）+ `open_tag_start`/`open_tag_end`（开标签原文偏移——标记插入锚点 + 幂等守卫面）三字段；既有 s412 单测扩 5 元素钉 specifier/偏移提取。
- **webview map**（`webview.rs`）：`declarative_css_modules: Arc<Vec<(String, String)>>` 进程内快照字段 + `csp_blocked_style_ordinals`（gate 三返回路径记录）；`load_html` CSP 门后 `prepare_declarative_style_modules`——is_module && 未 blocked && specifier 非空者入 map（first-wins 依文档序），逆序号插 ` data-zw-declmod=""`（`open_tag_end - 1`，`>` 前；幂等守卫已含标记者跳过）；map 穿四消费方：静态图收集（`collect_module_deps_recursive` 顶部 `type_attr == Some("css")` 命中臂——`registry.register` 合成体即返，零 fetch）、shim `__zw_compile_module` / `__zw_compile_inline_module`、V8 原生动态 resolver（with 子句 type 穿参命中——fixed t6 面）；合成体共享 `css_module_synth_source`（s249 `export default (function(){...replaceSync(literal)...` 复用，serde_json 字符串转义）。
- **engine 门**（`pipeline/mod.rs`）：`style_sheet_if_eligible` ⓪ `data-zw-declmod` 属性存在即 None（explainer：kind 固定元素永不为级联源；注释锚 re-parse 架构理由）。
- **shim**（`js_dom_shim/part04.js`）：`.sheet` getter module 臂返 `undefined`（先于既有 null type 门；非法 type 其余值 null 不变）。

## 过程陷阱

1. **单测无 fetcher 配置 → 动态 import 机制整体缺席**：`__zw_compile_module` 与 V8 原生 resolver 的注册面整个在 `if let Some(fetcher)`（R3091 门）内——缺省配置 `import('m', {with:{type:'css'}})` 落 V8 默认 reject `Error: Not supported`（v8_runtime state.resolver=None 臂），map 命中面根本测不到。探针烧 2 轮定位（bare/relative 双变体同红排除 specifier 因素）。修正：fetcher 兼探针——取回即 Err（map miss 快速失败），命中面 = 零取回 + resolve。
2. **adopted css 槽进程级全局互染**（R-t2qq D1 既有语义，非本切片引入）：两 `#[test]` 并行线程下 kind 固定相读到 map 相 adopt 的蓝（`rgb(0, 0, 255)` 污染初始黑断言，4/8 复现率）——`ACTIVE_ADOPTED_CSS` 槽无 load_html 复位点（runner 案级 render override 覆盖，webview 直载路径无）。修正：合并单 `#[test]` 双相顺序 + 相间 `set_active_adopted_css("")` 清槽（part56 复位法），8/8 稳。生产 in-process 导航跨文档 adopted 残留单列挂账（renderer 多进程快照恢复臂覆盖 adopted 槽，s158 FIXME 同族）。
3. **机械面（编译期抓住零漏网）**：collect 嵌套递归调用点 +1 形参漏穿（E0061）；Arc 进 move 闭包 E0382（显式 clone）；`*k == &key` E0277（String/&String 比较）。clippy `too_many_arguments` compile_module_contract 8 参（+1 触发）——s338 同法 `#[allow]` + 注记。

## 验证（/tmp/s413/ 全量日志，终稿代码）

- driving solo ×2（tentative-solo-1.log 裸跑 + battery tentative-solo.log guard 包裹）：**11 文件 37 子测全 Pass**——basic-import 2P、fixed 8P、9 个 csp-* 文件 27P（6 blocked 18P 零位移 + 3 allowed 9P 本轮新绿）；**0 Fail / 0 Timeout / 0 NotRun**。
- 单元（test-guard 包裹）：engine lib **3909P/7F**（+1P = 本切片 extract 扩钉；7F = s412 在册同名清单零位移，fetch/stream/websocket/img 域）；webview lib **852P/2F**（+1P = 本切片双相钉；2F = image_decoder/csp_connect_gate 在册对零新增——裸跑单飞偶现 image_decoder 绿为 favorable flake，guard 批回到在册对）。
- `cargo fmt --all -- --check` 净；`cargo clippy -p zero-engine -p zero-webview --all-targets -- -D warnings -A unused_variables` 净（css-parser random.rs 跨流红单列，s412 陷阱 3 同形）。
- 终局 runner freshness 门过；sweep 双跑（RAF_FRAME_DRIVEN，test-guard 包裹）+ 三 join（s292 协议，基线 /tmp/s412/pairs-*.txt = s412 终稿批 212F+8256P+19T）：见下。
- `make test` 全量（/tmp/s413/maketest.log，test-guard 包裹）：**154 腿 153 绿 + 仅 engine lib 在册红腿**（3909P/7F 与单元口径同清单一零位移；exit 101 → make exit 2，webview lib 腿结构性不在 workspace 腿清单——s412 同形，battery 独立腿 852P/2F 为权威记录）。compile-first 编译段 78m04s：ZeroSeed-2 项目验收环 26+ 实例 balance_dirty_pages 洪泛（本机 93% iowait、load 238）陪绑拖长，执行段 29.91s 正常——非本流负载单列。

## 口径

- sweep 双跑全等（join-cross stable=8504 state-diff=0 new=0）：**8275P/212F/17T + 5 NotRun**（s412 终稿批 8256P/212F/19T + 11 NotRun）。
- 本流域在册红：**212F+19T → 212F+17T**（−2T = basic-import/fixed file 键 Timeout→子测 Pass 键形迁移；212F 双基线 join state-diff=0 零位移）；NotRun **11 → 5**（−6 = 3 allowed 文件 `.done()` 达成，子测面 9P 全 Pass——pairs 基线不含 NotRun 键，GONE 侧仅 2 file-T 键对账）。
- join-b1/join-b2（对 s412 pairs-1/pairs-2）：**恰 19 NEW = tentative/ 19 子测 Pass**（basic 2 + fixed 8 + csp-allowed/nonce-allowed/script-src-allowed 各 3）+ **恰 2 GONE = basic-import/fixed file 键 Timeout**，stable=8485 **state-diff=0**——纯 driving 位移，零其他变化。
- tentative/ 目录口径：**11 文件 37 子测全 Pass、0F/0T/0NotRun**；the-style-element 目录余 1T = style-error-01.html（@import fetch 面，非 import 簇，先于本切片在册）。

## 残差与边界

- **外链模块基座 map 键错配（挂账）**：注册键 page URL 基 vs 查找键 parent URL 基——external module script 内 import 时（parent = 模块自身 URL ≠ page URL）未命中。语料 import 全部内联模块承载零案面；explainer 未钉该面；真外链承载出现时按 key 双写或 lookup 回退勘根。
- **`.sheet` kind 固定面不全（挂账）**：shim getter 按**现 type 属性**判字——type 移除后元素返 sheet（普通 style 形）而非 undefined。fixed.html 无 `.sheet` 断言（仅级联色面），explainer 语义 module 恒 undefined；语料零案面单列。完全对齐须 shim 侧标记感知（读 `data-zw-declmod`）。
- **标记属性外泄面（挂账，先例同形）**：`data-zw-declmod` 经 cloneNode/innerHTML 序列化外泄（zw-pseudo/zw-inline 内部工件同面，语料零断言）。
- **多进程 parity（挂账）**：declarative map 播种在 webview load_html 进程内；renderer/js_worker 多进程路径无此面（快照恢复臂不含 map/标记——SetDomSnapshot 恢复臂 adopted 槽已覆盖，s158 FIXME 同族挂账扩展）。
- css-parser random.rs unused-variable clippy 红承渲染流台账（本切片 strict 跑唯一 red，-A 单列复验）。

## 下一步

tentative/ 全绿收口；constructed-stylesheets 1T 同族勘根候选（s412 已提合并）；挂账池承 s404/s405/s411/s412（forward map 空返勘根/渲染流 5 面/form-submission-target 6T/sizes-auto 半边/ilf+modulepreload 观察池/适用态打满 reject 面/maxLength-minLength IDL 不适用态反射/unhandled-rejection 换算）；checkable 父 label 2F 渲染域移交承 s410；select/combobox（簇 D）与 A2+D 簇立项征询仍待用户决策。
