# M2 切片 408 — detached 文档 style 元素 LinkStyle sheet + headEl 本地树三件套

日期：2026-11-12（承 s407 下一步：the-style-element 10F 未归因面勘根优先）
commit：见 git log s408 提交

## referent 与勘根

driving 单键（s407 收口时在册 F）：

- `document-metadata/the-style-element/non-parser-created-doc.html ::
  CSS API should work on non-parser-created documents`——唯一断言
  `assert_true(!!style.sheet)`，时序 `implementation.createHTMLDocument("") →
  createElement('style') → doc.head.appendChild(style) → 读 sheet`。

四层勘根（探针 P1–P7 逐层定位，探针勘根毕即删）：

1. **headEl 无本地树方法**：detached doc 的 headEl 字面只有 childNodes /
   hasChildNodes / firstChild / lastChild（R130 只接了原型链与 sibling/parent
   getter）；`doc.head.appendChild(style)` 落全局 Node.prototype——childNodes
   不插（len 恒 1 只剩 title 子）、parentNode 不设。docEl 同款缺口 R126 修过
   （Node-removeChild synthetic 变体），headEl 是同族漏网。
2. **plain 产物无 sheet 路由**：detached `createElement` 产物是 `_zwMEl`
   plain 节点（无 `__zwHandle`），main sheet getter 分支（part04 R3036）按
   sel/handle 代理匹配根本不命中 → `.sheet` 恒 undefined（R3036 注记的
   handle-only detached 形态返 null 即此缺口）。
3. **R5271b 门对 plain owner 恒 false**：`_makeStyleSheet` 的连接门
   `_zwIsConnected(owner.__zwSelector, owner.__zwHandle)` 只认主文档形态，
   plain owner 双 undefined → 尾部 rect probe 恒 false → null。spec LinkStyle：
   sheet 在**任意文档树**内即存在（detached doc 无 browsing context 只影响
   active，不妨碍 sheet 创建）；未挂载（createElement 未 append）才返 null
   （LinkStyle.html not-in-a-Document 案钉）。
4. **createElement 时直挂是既有先例**：R5256 dialog（showModal/show/close
   inactive-document DOMException）+ s69 img.decode（调用时活跃性判别）都在
   detached createElement 按 tagName 直挂 API——sheet 访问器循同位。

## 施工

- **part03 headEl 三件套**（headEl 字面 + R130 原型接线后）：appendChild /
  removeChild / insertBefore 镜像 docEl trio（R126）语义——appendChild 先移
  旧父 + `_r223SetParent`（plain 子走 defineProperty 遮蔽臂）+ push +
  `_r130WireSiblings`；removeChild 带 spec dom-node-pre-remove NotFound 校验 +
  live 迭代器/range 迁移守卫（R81 NodeIterator 面一致性）；insertBefore 同款。
  残差记档：跨文档 adopt 子树传播（R330 ownerDocument 重指）与 CE 连接态派发
  （WC-M1）未接线（FIXME(headEl-adopt)，corpus 无钉）；body 字面（R3017
  detached root 形态）无本地树方法，corpus 未驱动不动。
- **part03 detached STYLE 产物 `sheet` 访问器**（createElement 内，s69 img
  块后，`_isHtmlDoc && tag === 'style'`）：defineProperty get——type 门镜像
  part04 R3036 getter（s156/s160 精确匹配：非空且 ASCII-CI 非 text/css →
  null，缺省与空串不设门，无 trimming/parameters）；体走 `_makeStyleSheet(e)`
  统一门（身份缓存/指纹失效/R5271 家族单实现）。
- **part06 `_makeStyleSheet` 门扩 plain owner 臂**：owner 有 sel/handle 维持
  `_zwIsConnected` 不变；否则 parentNode 链 walk（64 步防环）到 nodeType 9
  → true（detached doc 树含）。出树再读 → gate false → null（门每读先跑，
  缓存条目留着但不可达）；回挂 → 指纹命中 → 同一 sheet 对象（R5271
  [SameObject] 缓存跨门读写一致）。
- **常驻钉**：`js_dom_bridge_tests/part108_s408_detached_style_sheet.rs` 双钉
  ——①driving 面镜像 + headEl 树接线（父链/子表 [title, style]）+
  [SameObject] 身份 + 未挂载 null/出树 null/回挂同对象门矩阵 + cssRules 恒空
  面（FIXME(style-detached-rules)：detached 树无 mutation 历史，textContent
  内联规则未接 CSSOM）；②detached type 门五案（jibberish→null / text/css→obj /
  TeXt/CsS→obj / 空串→obj / ` text/css `→null——s156/s160 语义镜像）。

## 过程陷阱

1. **probe 污染目录 solo**：勘根用 s408-probe.html（7 独立 test() 块）勘根毕
   未即时删——目录 FILTER 跑出 `Fail s408-probe.html :: P6 element form`
   （P6 断言 `__zwHandle` 存在是旧架构假设，修复走 plain 访问器路线，探针
   断言自然过时恒红）。且勘根前 `ls` 报不存在是路径层级算错一级（crates/
   engine/src 上跳 4 层落到 /lzcapp/document/work 而非仓库根）——探针实际
   在 corpus 内污染。删后目录重跑归零。probe 卫生纪律（勘根毕即删）重申。
2. **钉首编 E0308**：`Arc::new("…".to_string())` 漏 `Mutex::new` 包裹
   （part107 原样是 `Arc::new(Mutex::new(…))`）——照抄锚点时丢了中间层。

## 验证（/tmp/s408/ 全量日志）

- 双钉全绿：`cargo test -p zero-engine --lib detached_` 28 passed 0 failed
  （含既有 detached 面 26 + 新 2）。
- driving solo（runner 重编含 WIP，token `_e408s` 命中钉证新鲜）：
  non-parser-created-doc.html `CSS API should work on non-parser-created
  documents` **F→Pass**。
- the-style-element 目录 solo（探针删后干净跑）：**34P / 0F / 9T / 6NotRun**
  ——driving 1F→P 后全目录零 Fail；9T = tentative/ CSP + constructed-
  stylesheet 机制面（CSSStyleSheet constructor + adoption + CSP 大机制，
  独立候选）；6NotRun = CSP securitypolicyviolation 面，与修复前基线态一致。
- 单元：engine lib **3886P/10F**（s407 基线 3884P/10F + 双钉；失败清单逐名
  = 9 在册 + r307 HEAD 红单列，零位移）；webview lib **849P/2F**（
  image_decoder proxy / csp_gate 在册对）。
- 全树 sweep ×2（同二进制，s292 协议）：run1 = 8491 keys 8249P/217F/25T
  （**含遗留探针 7 键 6P+1F——sweep-1 起跑时探针尚未删，单列**）；run2 =
  **8484 keys 8243P/215F/26T**。join-cross stable=8483 **state-diff=1**
  （customizable-select select-home-end-pagedown-pageup-detailed.optional
  Fail→Timeout 同二进制翻转 = s292 时序 flake，s407 在册 watchdog 边界翻转
  族同款）+ **new=0**（探针键全 A-only 单列）。
- join 对 s407 基线：join-1 = stable 8482 + driving Fail→Pass +
  select-home-end Timeout→Fail（同 watchdog 家族反向翻转）+ 探针 7 NEW 单列；
  join-2 = **恰 1 STATE-DIFF 全为 driving Fail→Pass、new=0**——双跑一致，
  真 new=0。
- fmt：本切片 Rust 文件 `rustfmt --check` 干净；`callbacks.rs` HEAD 既有 fmt
  脏承 s407 单列不代修。clippy `-p zero-engine --all-targets -- -D warnings`
  干净。
- smoke：`make product-smoke` + `make product-smoke-legacy` 双 PASS
  （struct-check 0 issues）。

## 口径

- 本流域在册红：**215F+24T → 214F+24T**（−1F = driving 键 F→P 双跑一致）。
  （sweep run2 8484 keys 8243P/215F/26T）
- engine lib 单测在册红 **9F + r307 HEAD 单列**（承 s407 口径不变）。

## 残差与边界

- detached style textContent 内联规则未接 CSSOM（`_makeSheetBuild` 空 wire
  臂，cssRules 恒空——FIXME(style-detached-rules) 记档，corpus 无钉）。
- detached LINK rel=stylesheet 产物仍无 `sheet` 访问器（LinkStyle 双接口之
  一，corpus 未驱动；扩面循同位直挂即可）。
- headEl 跨文档 adopt 子树传播 + CE 连接态派发未接线（FIXME(headEl-adopt)）；
  body 字面无本地树方法（the-body-element 面未驱动）。
- tentative/ 9T（CSS Modules via constructed stylesheets：CSSStyleSheet
  constructor + adoption + CSP 事件面）独立大机制，本切片不动。
- 自建 probe 恒红案族（s343 口径）继续单列，不入在册两池。

## 下一步

散单续勘（下轮候选，按池体检视）：links 族 4F / the-input-element 散单 /
tentative/ constructed-stylesheets 机制（体量大，勘根后按簇决策）；
select/combobox（簇 D）与 form-submission-target（M3 门控）仍待用户决策，
不动。
