# M2 s121 · a/area URL 分解组件三面收口（sel setter lw 读 + 失败 protocol ':' + opaque pathname no-op）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s120（fragment 解析 script 克隆 already-started 继承）

## 刀位与判据链

四候选入场勘察：

1. **the-dialog-element 43F 精确勘根**（探针页 A/B，gitignored corpus 验后删
   除）：16F popover-closedby-complex（shadow 变体）+3F focus-previous-outside
   +1F form-submit-dialog-shadow = **20F 堵声明式 shadow DOM 解析**——
   `<template shadowrootmode>` 在 parser/DOM 层完全未实现（全仓 grep
   `shadowrootmode` 零命中；探针 B 实证 `host.shadowRoot` 全 null，s29
   `_zwDialogShadowInclContains`/zero-dom `attachShadow` 均为 JS API 面），
   shadow 内元素无布局盒（rect 全零）——渲染流大基建，与用户已决策 parser
   批次缓行（2026-10-23）同档。~12F top-layer UA 定位/centering（abspos 6F、
   multiple-centered、scroll-height、non-modal-layout、top-layer-position、
   closedby-bounds-clicking 2F、showModal d11）= style-system/layout 跨流让
   位。余 2F overlay CSS 属性、2F default-color（CanvasText 已挂账）、3F
   inert/hit（渲染 inert 挂账）、1F selection、1F ESC re-entrant——**无 shim
   单根常规刀位**。
2. **execution-timing 余 F**：全部已挂账大基建（041/068/143 写流队列、056/063
   增量解析构成面、module 异步调度），无新单根。
3. **meta-element 24F**：全 CanvasText 系统色 = style-system 让位确认。
4. **新刀位：links-created-by-a-and-area 20F 单面簇**——attribute-getter-setter
   14F + non-parsable 4F + non-special-opaque 2F。

探针页 C/D 交叉定谳根因：

- **sel 形（querySelector）组件 setter 失败、handle 形（createElement）正常**
  ——R3070 URL 组件 setter 分支（part05）sel 路径读 `__zw_get_attr` **纯快
  照**：`a.href = v` IDL setter 经 R3069 落 pending SetAttr mutation，render
  apply 前不入快照，`_uhCur` 读 stale 旧值（空）→ 组件 set 静默 no-op。与
  R3202/M2-S9 stale-snapshot 模式同款（R3070 未随 M2-S9 迁 lw 读）。
- host 回调 `__zw_set_url_part` 存在且直调成功——断链仅在 shim 读侧。
- 判据面：WPT `non-parsable-url-getter-setter.window.js`（`http://test:test/`
  protocol get `:`、set `https`、origin set null）+ `non-special-opaque-path-
  url-getter-setter.window.js`（`non-special:opaque` pathname get `opaque`/
  setget `opaque`）+ HTML §4.6.3 api-for-a-and-area-elements（protocol getter
  步 2「If this's url is null, return ':'」；pathname setter 步 3「If url is
  null or url has an opaque path, then return」）。

## 施工（净变更 part03/part05 + js_dom_bridge.rs + pin）

1. **part05 R3070 sel 形读迁 `__zw_get_attr_lw`**（typeof 回退同 part03:25275
   M2-S9 款）——组件 setter 读最新 href，pending mutation 面闭合（14F）。
2. **part03 解析失败 getter 三态**——`__zw_parse_url` 失败（spec set the url
   后 url 仍 null）时：href 返原串；protocol 返 `:`；其余返 `''`。catch 臂同
   面（host 回调异常同 url-null 口径）。setter 面因解析失败返空串静默不写自
   动自愈（origin set null 期望即此形）（4F）。R5009 空 href 早退（解析前）
   不动——已收口径不扩面。
3. **js_dom_bridge.rs `set_url_part` pathname 臂 `cannot_be_a_base()` 守卫**
   ——opaque path 不执行 `set_path`，href 序列化不变（2F）。search/hash
   setter 无 opaque 守卫 = spec 正确（query/fragment 对 opaque 合法）；
   username/password/host/port 经 url crate 对 cannot-be-a-base 报错天然
   no-op，均不动。

## 验证

- 探针 C 四臂全绿（hash/search/protocol/host setter round-trip + non-parsable
  protocol getter）；探针 D p1 分发路径判别（sel/handle/setAttribute 三形）。
- 定向 `FILTER=links-created-by-a-and-area`：**20F→0F**（14F
  attribute-getter-setter + 4F non-parsable + 2F opaque 全收口；4 既有
  Timeout——rel-attribute-clearing + target_blank 三页——不变）。
- **全量 A/B**（/tmp/s120-full.log → /tmp/s121-full.log，全行归一 diff）：20
  对 Fail→Pass 精确翻转 = 三刀判据面，**真翻红 = 0**。另三处方差逐一归因非
  翻转：①src-then-lazy-load 文件 Timeout 消失、7→8 子测全绿②
  focus-dynamic-type-change-on-blur 文件 Timeout 消失、68→80 子测全绿——两
  页即 s120 账本已判定的负载型页级超时 flake 面（8-job 并发负载相关，本轮负
  载轻完成更深）③modulepreload-cross-origin-referrerpolicy origin 子测本轮
  未及完成（文件两轮皆 Timeout；定向单页复跑同貌 2F+页级 T）——modulepreload
  referrerpolicy 挂账族行形漂移，与刀面无涉。
- bin pin `anchor_url_decomposition_s121`（sel 形 stale 面 + 失败 protocol +
  opaque pathname 三面）：新 shim **绿**；旧 shim stash 往返**红**（行为失败：
  `sel 形组件 setter 读最新 href: #somehash`，非编译失败）。
- engine lib **3570/0**；guard 内 clippy（runner/engine --all-targets）零告
  警；fmt 过；shim 拼接语法过（真实装配序 part01+01b+02..07，`node --check`）。
- 渲染面零依赖——reftest/product-smoke 未跑；make test 未跑（Rust 增量仅
  test 模块）。

## 挂账（本切片新增/更新）

- **declarative shadow DOM parser 批次明示**：`<template shadowrootmode>`
  解析（shadow root attach + template content 消费）+ shadow 内布局盒——与
  parser form element pointer 4F 打捆（用户决策 2026-10-23 缓行），M2 收口后
  尾段单独立项；dialog 43F 中 20F 归此。
- **dialog top-layer UA 定位/centering ~12F**（style-system/layout 跨流让位
  面）。
- **dialog 散项**：overlay CSS 属性 2F、inert/hit 3F（渲染 inert）、selection
  1F、ESC re-entrant 1F。
- 承袭：写流执行身份盖章断链 FIXME、part02 setAttribute 绕 notify 臂（gap 2）、
  execution-timing 构成面 056/063+051/052 单根（增量解析大基建）、041 nested
  write（写流队列重构）、lazy-loading 族 26F（viewport 基建大项）、meta
  color-scheme 21F（跨流 style-system）、css-module 族 corpus 门（待决须用户
  拍板）、www2 可达性、modulepreload referrerpolicy 族、module-import-referrer
  TLA、script-crossorigin-network cookies、redirect-relative-urls、
  export-from with FIXME、script-charset-02/03、143 write、defer 多写完成序、
  dynamic import() with 透传、classic 外链 SRI、接口对象基建面、FIXME 组、
  /tmp/zw-hb*.txt；**待决（须用户拍板）**：css/text 数据模块语料资源缺失、
  parser form element pointer 4F 缓行（2026-10-23，本切片并入 declarative
  shadow DOM parser 批次口径）。

## 遗留近似

- 解析失败 getter 三态以 shim 侧 `__zw_parse_url` 返空为 url-null 判据，与
  host 侧 `Url::parse` 失败一一对应；corpus 对 non-parsable 的组件 getter 期
  望全为 `''`（origin 条目仅 `set: null`，corpus 跳过其 setter 测试，getter
  期望默认 `''`）——本仓返 `''` 对齐，无 null/`''` 表达差异面。
- `set_url_part` opaque pathname no-op 臂 `ok=true` 仍回写序列化 href——
  `non-special:opaque` 序列化恒等，现语料无漂移面；若未来组件 setter 语料含
  非规范 href 输入（odd escapes）需复查该回写面。
