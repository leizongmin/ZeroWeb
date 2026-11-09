# M2 s333：audio lazy loading 门补面——the-audio-element loading=lazy 族 11F→11P 零位移

日期：2026-11-07 ｜ 承接：s332（`2026-11-07-m2s332-media-iface-illegal-constructor.md`）

## 目标与定谳

s330 挂账「audio loading=lazy 10F」（s332 域跑实测 **11F**）。定谳：s281 落 media lazy loading 门时只认 `video`——脚本门 `_zwMediaScheduleLoad` 与静态门 `_zwSettleResourceSelector` 的 tag 臂均为 `tag === 'video'`，audio 不进门、一律即时装载。而语料（Squarespace 署名的 audio lazy 族）期望 audio 与 video 同语义：`loading=lazy` 四态门（未连接/not-rendered/远端 defer，近端+window load 已派发装载）+ scrollIntoView 唤醒 + eager 化唤醒。另 audio 独有一面：**无 controls 的 audio 不渲染**（HTML rendering §audio「An audio element that does not have a controls attribute should not be rendered」），lazy intersection observer 对不渲染元素永不回调——即使 scrollIntoView 也不醒（`audio-loading-lazy-not-rendered` 三案断言面：无 controls / hidden / display:none 全无 controls，2s 内无 loadstart 亦无 error）。

spec 锚：https://html.spec.whatwg.org/multipage/urls-and-fetching.html#lazy-loading-attributes （lazy load resumption steps；s281 注释「lazy elements = img 与 video、audio 不在列」为旧面，语料按含 audio/video 的现行清单出题）；https://html.spec.whatwg.org/multipage/rendering.html#audio 。

## 施工（三文件）

- `part06.js` 脚本门：tag 臂 `video` → `video || audio`；新增 **audio not-rendered 臂**——`audio && lazy && 未 settle && 无 controls content 属性` 恒 defer 入册（置于旗标检查之前、无视旗标：零面积元素 scrollIntoView 不落位旗标不置，同 img s274 not-rendered 模型）。engine 布局对无 controls audio 仍建盒（UA `audio:not([controls]) { display:none }` 属 style-system 渲染域未落地），故门内资格判定显式补面。eager 化唤醒经冻结参重入：重入时 `loading='eager'` 使臂失效、落载。
- `part06.js` 静态门：同构 tag 臂扩 audio + not-rendered 臂（commit 参数册 `{kind:'commit',…}`，唤醒体重放）。非 lazy audio 不进任何 defer 分支，直落 settle——常规 audio 面零变化。
- `part04.js` scrollIntoView 臂：`_sivTag === 'VIDEO'` → `VIDEO || AUDIO`（旗标置位 + lazy 册 sweep；source-deferred / autoplay-when-visible 的唤醒面）。
- `part09.rs` 契约测试 `test_media_audio_lazy_gate_s333` 四臂：① 脚本车道无 controls lazy audio defer（readyState 0 无 loadstart）；② eager 化唤醒装载（readyState 4）；③ 静态车道宿主 commit 后 defer（`__zw_commit_resource_element_state` 路由）；④ 静态 eager 化唤醒 settle。

唤醒机具全部复用 s281 既有三入口（window load drain / loading mutation / 旗标 sweep），门内资格再判自愈——not-rendered 臂每次重入重判，无新增唤醒通路。controls 动态添加的唤醒面语料暂无——FIXME 挂账在码。

## 验证

| 面 | 方式 | 结果 |
|---|---|---|
| 契约四臂 | `cargo test -p zero-engine --lib test_media_audio_lazy_gate_s333` | 1/1 通过，`logs/s333-contract.log` |
| audio 域 | `make testharness-html-semantics FILTER=the-audio-element` | **24P/0F/0T exit 0**；audio-loading 族 13 键全 P——11F→11P（autoplay-deferred / lazy-autoplay-when-visible / lazy-disconnected / lazy-in-viewport / lazy-not-rendered / lazy-source-deferred / lazy-to-eager / lazy-window-onload / load-deferred / load-preload-auto-deferred / load-preload-metadata-deferred）+ eager、lazy-in-scroller 原绿保持。基线 `logs/s332-ec-after.log`（s332 HEAD，M1 shadow 面不触 js_dom_shim part06，且 M1 自证 WC 全量零新红），`logs/s333-audio-after.log` |
| video 域回归 | 同 FILTER=the-video-element | 21 键 (state,key) 归一 diff **零位移**（15P/1F/1T 族含既有红 `video-loading-lazy-load-when-visible`、`resize-during-playback`×2、`video_size_preserved` T 原样）——共享门面零回归，`logs/s333-video-after.log` |
| engine lib | `cargo test -p zero-engine --lib`（test-guard） | 3799P / **6R 恰为在册既有红集**（img settle×2 / fetch matrix / iframe SW / websocket×2，s330 记录原样）零新红，`logs/s333-lib.log` |
| fmt/clippy | `cargo fmt -p zero-engine -- --check`；clippy test-guard | fmt 零 diff；clippy `logs/s333-clippy.log` |

## 挂账与队列

- FIXME 在码：lazy audio 无 controls → 动态添加 controls 的唤醒面语料暂无（not-rendered 臂无视旗标，加 controls 后仍 defer，需 mutation 观察补面或语料证伪）。
- video-loading-lazy-load-when-visible 1F、resize-during-playback 2F、video_size_preserved 1T 既有残红承 s329/s332，本切片不动面。
- 下一候选：object width/height rendered-dim IDL、canvas 残红 3F+1T（承 s329）、CSP style-element 8T、microtasks 4F+2T、credentials 簇（移交 runner-net-infra）。
