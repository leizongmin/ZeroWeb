# M2-S173 html5ever 0.29.1 声明式 shadow 解析缺陷绕道 + register/ensure 惰性镜像（s168 遗留面收口）

**日期**: 2026-10-31
**刀位**: s168 施工后遗留的两案红（form-indexed-element-shadow / form-submit-dialog-shadow，s168 账面记 1F=M1 基线同态）+ s168 commit1 FIXME「shadowRoot 恒 null」面。勘根升级：不是物化通道 bug，是解析层出生即坏。

## 勘根定谳

**html5ever 0.29.1 `should_attach_declarative_shadow` 值精确匹配缺陷**：该函数对 `shadowrootmode` 只精确匹配 `"open"`/`"close"`（`"closed"` 不在匹配集——html5ever 自身 bug，spec 枚举是 `open`/`closed`）。命中时 template 元素**不入树**、其内容摊平进宿主元素。后果链：

- 解析产物里**根本不存在 template 元素** → s168 `materialize_declarative_shadow_roots` 对 `open` 形态从未生效（读侧永远找不到模板）；`closed` 形态反而正常（失配 → 正常入树 + materialize 物化）。
- `host.shadowRoot` 恒 null → WPT form-indexed-element-shadow / form-submit-dialog-shadow 红面（shadowRoot 通道断）。
- R2926（shadow 内容不渲染）在摊平形态下反而"看起来有内容"——4 个基线凑巧 Pass 键的来源（见 LOST 归因）。

## 施工（7 文件：rust 5 + shim 2 + 新模块 1）

1. **crates/dom/src/parser/declarative_shadow.rs**（新，323 行）：`restore_declarative_shadow_templates` 预变换——喂 html5ever 前把 `<template>` 标签内 `shadowrootmode` **值首字母大写**（`open`→`Open`）。spec shadowrootmode 枚举 **ASCII 大小写不敏感**（HTML §the-template-element 声明式 shadow root 步），值大小写对语义不可见；materialize 读侧 `to_ascii_lowercase` 归一后照常物化。扫描面镜像 tokenizer 状态：`<template` 名字边界校验 + 引号感知 + 属性名前界（`data-shadowrootmode`、值内子串不命中）+ RAWTEXT/RCDATA 原文区跳过（script/style/title 等，polyfill JS 内嵌模板串是真实场景）。`closed`/bogus 值不改写。6 单测（值改写/语法变体/幂等/非模板区不动/RAWTEXT 区不动/多模板）。FIXME(html5ever)：升级 0.38（完整 spec 流）后移除本预变换。
2. **crates/dom/src/parser.rs**：`parse_html_with_builder` 接线 prepass（`shadowrootmode` 不存在时零拷贝 fast path——`contains_ci` 门）。
3. **crates/engine/src/js_dom_bridge.rs**：`declarative_shadow_deep_json_doc` 生产函数——live 文档按 selector 定位宿主，把其（物化后的）shadow root 序列化为 deep JSON（mode/slotAssignment/delegatesFocus + 子树）。
4. **crates/engine/src/js_dom_bridge/callbacks.rs**：`__zw_declarative_shadow_deep` wire（query-live-aware 求值）+ `__zw_declare_fragment`——**给定句柄**的 fragment 绑定回调（shim 本地句柄在惰性镜像构建时补 engine 绑定，与自 mint 版 `__zw_create_document_fragment` 的差异仅在句柄来源）。
5. **crates/engine/src/dom_bridge_tests.rs**：钉测 `test_declarative_shadow_deep_json_s173`（宿主可寻址 + template 计数 0 + shadow_root Some + `"mode":"open"` + slot 在 deep JSON + closed 透传）。
6. **part04.js**：shadowRoot getter 调 `_zwDeclarativeShadowHydrate`。
7. **part05.js**（register/ensure 分离，净 +108）：
   - `_zwDeclarativeShadowHydrate` = **register-only 零 mutation**：`_zwDsRegMiss` 负缓存 + `_zwDsHandleCounter` mint **`__ds{n}`**（刻意避开 `__n{n}` 命名空间防 HANDLE_COUNTER 碰撞）→ 写 `_fragmentHandles`/`_shadowHandles`/`_shadowHandleMeta`（`{hostSel, hostHandle, mode, mirrored:false, declarative:true}`）+ `_shadowRoots[key]`，**不产生任何 DomMutation**。
   - `_zwDeclarativeShadowEnsureMirrors(rootHandle)`：仅 `meta.declarative && !meta.mirrored` 时构建（先置 mirrored 防重入）→ `__zw_declarative_shadow_deep(hostSel)` 重取 deep JSON + `__zw_declare_fragment(rootHandle)` engine 侧绑定 → `_dsBuildHandleChild` 递归建镜像（T→text/C→comment/E→element+attr+递归 append）→ `__zwDetailsScan4100` 收尾。
   - 钩点只在 R2928 `_handleQueryFirst`/`_handleQueryAll` 顶部——**真内容查询才付结构批代价**，通用枚举不触发。

### register/ensure 分离的根因（首刀回归勘根，7 探针钉链）

首刀 hydrate 即建镜像 → 镜像 mutation = structural batch → `render_with_dom_mutations_persistent`（pipeline/mod.rs:2036）**全快照重建**（slotmap 换代、旧 NodeId 全灭）发生在 **initial-script 时点** → dialog-popover-closedby-complex 的 light-dismiss/geometry 拿到代际偏斜视图 → **18 键 P2F**（423P/60F vs 基线 439P/44F）。探针 A（hydrate-off）与探针 E bit-exact 定谳回归源，拆分后 P2F=0。教训入账：**engine 侧结构性重建的时点敏感性**——getter 类 API 绝不能触发 structural batch。

## 验证

- **zero-dom lib 880P/0F**（6 新单测全绿；parser prepass 幂等——已大写形态原样输出）。
- **engine lib 3615P/10F**（+1 = s173 钉测；10F 与基线同键：fetch/request/response 契约族 9 + ua_display_li s24）。
- **webview lib 825P/0F**（基线持平）。
- **定向双靶 F→P**：form-indexed-element-shadow（s168 账面 1F）、form-submit-dialog-shadow（同）——`Pass html/... :: form.elements: indexed access reflects DOM order, not flat tree` 等。
- **interactive-elements 域终态 437P/26F/463 键**（FILTER=interactive-elements，探针已删后终态树重跑）对 s172 基线 439P/44F/483 键 (key,state) 对严格 join：
  - **P2F = 0**（首刀 18 键回归清零）；
  - **F2P = 2**：form-submit-dialog-shadow（靶案）+ dialog-focus-previous-outside「Focus restore should not occur when the focused element is in a shadowroot outside of the dialog.」（shadowRoot 不再恒 null 的连带翻绿）；
  - **LOST = 20**（**4P+16F** = dialog-popover-closedby-complex shadow-slots 全段）：宿主内容**枚举** deferred（childNodes 类查询对声明式根不展开镜像——钩点只挂 R2928 内容查询），段内元素收集 assert 自中断无行产出；基线 4 个 Pass 全为摊平 DOM 凑巧 Pass 的 modeless 组合（double-error 抵消），18F 为真红——本刀把假 Pass 清掉、真红维持（枚举落地后再翻）。归因明确非行为回归。
  - **GAINED = 0**。
- **forms 家族（FILTER=the-form-element）96P/12F**：12 F 全既有——form-elements-filter「form.elements must contain all listed elements」`git stash push` 双态对照 HEAD 同键 Fail（pre-existing 钉证）；其余 7 文件（nameditem/interfaces/matches/action-reflection）零 shadow 关联（prepass shadowrootmode 门 + shim declarative 门 inert by construction）；`form-indexed-element-shadow` 不在 F 列（家族级 Pass 复证）。
- fmt 全绿 + guarded clippy `--all-targets` 0 警告（两处即修：doc list 缩进 + collapsible_if→filter 等价改写）。
- shim 13-part 按装配序（part01,01b,02..07,08a,08,08c,08b,08d）拼接 **node --check SYNTAX_OK**（113158 行）。
- 探针验后删：zz-s173-probe + zz-s173a/b/e/f/g-probe + zz-s168b-shadow-cascade-probe 全清（9 月遗留 zz-probe5/6/7/8/9/15、zz-cueprobe 非 本次产物，只提及不清理）。

## FIXME 新记 / 挂账

- **声明式根内容枚举 deferred**（LOST 20 归因面）：`childNodes` 等通用枚举对声明式根不展开镜像，需把 ensure 钩扩到枚举通道且避开 getter 时点雷区——下轮候选。
- **shadow 内容不渲染**（R2926 承接）：render 管线走平铺 dom_html 跳过 shadow 树——保真度 defer。
- innerHTML/fragment-parse 路径不跑 materialize（解析 prepass 只挂 parse_html 主路径）。
- 嵌套 shadow 级联（s168 commit1 FIXME 承接）+ 镜像写穿 render 树——均 defer。
- FIXME(html5ever) 0.38 升级后移除 prepass（模块文档在案）。

## 下一步候选

①声明式根内容枚举（本刀 LOST 20 直接承接）；②webview 静态 blob 路径 entries 化（s167 挂账）；③ua_display_li（style-system 跨流）；④selection-pointer forms 域 22 案翻绿归因勘定（s172 记）。
