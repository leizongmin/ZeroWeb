# M2 s295 — 余量无主 img 面收口（null-image-source shim 双层修 1 文件 Timeout→3P；update-src-complete 纯资产 Timeout→1P；端点仿真两面 F→P 转正；natural-size-orientation 转明 1P/2F EXIF 挂账；目录级 A/B P2F=0）

日期：2026-11-06。切片号 s295（承 s294 下一步候选④余量无主 img 面：image-alternate-
no-store EncodingError 1F、list-of-available-images coalesce 1F、natural-size-
orientation / null-image-source / not-rendered-dimension-getter / update-src-
complete Timeout ×4）。上一 session 施工中途 429 限额中断（testharness.rs 两端
点仿真臂未提交 WIP），本 session stash pop 恢复衔接收口。

## 勘根

**面 1 null-image-source.html**（首子测「img with empty src」挂 → 文件级
Timeout，2P 存证）：spec 原文核对（updating-the-image-data selected-source-
null 步：current request → broken、current URL 清空、error 门 =「element has
a src attribute or it uses srcset or picture」+ maybe-omit-events/previousURL
子句）。WPT 契约甄别：srcset=""/picture 父两臂 `unreached_func` 钉**不派**
error（pin 语料与上游 master 逐字一致；主套件非 tentative）——浏览器 de-facto
门 = **src 属性存在**（任意值含空串），与 spec 字面「uses srcset or picture
也派」分叉，按语料契约实现。根因两层：①part04 sweep 空 src 臂裸 `continue`
——静态 src="" img 到不了任何 settle 面；②part03 `__zwImgNullSelection5277`
`if (!_nkPrev) return` 无前请求早退。爆炸半径枚举：静态 src="" img 全树仅
3 页（本目标页；img.complete.html 两断言 broken→complete=true 同语义无移；
iframed.sub.html 仅 currentSrc 面）；动态 src='' 赋值面全在 media-elements
（归档通道 rule 9 零执行）与 the-img-element lazy 两页（目录 A/B 覆盖实证
零回归）。

**面 2 update-src-complete.html**（文件级 Timeout，state={loaded:true,
pending:1,phase:4,tests:1} 存证）——**s227 同型教训面**：先绕 shim 全链静态
勘根（fetch handler query 剥离 `split_once('?')` ✓；runner 窗序
`__zwReadyState='complete'` 先于 window load 派发，handler 内二次派发走直
派臂 ✓；RunUpdate restart/epoch/avail 链 ✓ 全健全）——后验语料：顶层
`the-img-element/image.png` 缺失（resources/ 与 srcset/resources/ 两同名副
本在场，③g 注记「image.png 已在本地」指字节出处非运行时消费路径，误导勘根
方向）。症状全链闭合：首载 404 → decode 失败 → error → resourceState
broken → complete getter 见 state 返 true（assert_true 假绿）→ 二次换源
`image.png?<random>` 亦 404 → error 非 load → 新 onload 永不到 → pending:1
Timeout。上游 pin 该路径 200，fetch_raw 补齐即绿——shim 零改动。

**面 3 natural-size-orientation.html**（Timeout → 转明 **1P/2F**）：真面 =
EXIF upright 方向重定向缺口（期望 144×240，实得原始存储维 240×144；第二臂
image-orientation:none 同红）——zero-media 解码器级特性缺口，本片挂账不定修
（s296+ 解码器特性片候选）。

**面 4/5 两端点仿真面**（image-alternate-no-store.window.js「Reassigning
the same src value…」1F、list-of-available-images-does-not-coalesce….
sub.tentative.html 1F）——上一 session testharness.rs WIP（端点臂内嵌字节
回体 + per-case 状态表），本 session rebased 二进制复验 PASS。

## 施工（3 文件）

- `crates/engine/src/js_dom_shim/part03.js`：`__zwImgNullSelection5277` 无
  前请求门收窄——`!_nkPrev` 时仅 src 属性在（任意值）放行 broken 写入 +
  error 派发（_nkHasLink 门照旧）；srcset/picture 祖先/无属性维持既有无状
  态早退面。has-prev 臂零改动（part88 (6) 走前态臂不受影响）。
- `crates/engine/src/js_dom_shim/part04.js` 两处：①sweep 空 src 臂路由
  NullSelection（`!_zwPrior` 幂等守卫——写 broken 后后续 sweep pass skip，
  防多轮重派 error → t.done() double-call 假红）；②IMG currentSrc 空串短
  路——null-selection broken 态（url=''）不进 absolutize（'' 对 base 解析
  得页面 URL；spec selected-source-null「Change the current request's
  current URL to the empty string」→ currentSrc 须空串）。
- `tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh`：③g 注记澄
  清（顶层 image.png 与两副本是不同路径，勿混读）+ ③i image.png 运行时消
  费补拉（含基准变更注记）。
- 语料卫生（不入提交——gitignored）：image.png 268B 本轮拉取；image-1.jpg
  / arrow-oriented-upright.jpg 上一 session ③h 行拉取。

## 中途回归与修正（目录冒烟 A/B 抓 1 枚 P→F）

首版 shim 修（s295-imgelem-dir-solo-fixed.log）目录级 A/B 出 **P2F=1**：
update-the-source-set.html「`<img src>`」子测（currentSrc 空串断言）——
①的 broken 写入使该 img 进入 currentSrc getter resourceState 臂，
`__zwAbsolutizeSourceUrl5277('')` 把空串对 base 解析成页面 URL。修正 =
②空串短路（fixed2）。修正后目录 A/B P2F=0（见验证）。

## 验证

- **前置**：concat node --check 干净（canonical 序 19 片，129626 行）；
  `cargo fmt --all --check` clean。
- **solo 目标面**（终版二进制）：null-image-source **3P/0F**（文件
  Timeout 消失；srcset=""/picture 两钉静默面维持）、update-src-complete
  **1P**（single_test）、natural-size-orientation **1P/2F**（挂账面转明）、
  image-alternate-no-store / coalesce / not-rendered-dimension-getter
  各 **1P**（PASS）。
- **目录级 A/B**（A = s294-fulltree-b2.log the-img-element 域行；B =
  s295-imgelem-dir-solo-fixed2.log）：**shared 1026 键，翻转恰 2 枚全
  F→P**（两端点仿真面——testharness.rs WIP 归因），**P2F=0**；LOST 4 =
  四页 file-Timeout 行（natural-size-orientation / not-rendered-dimension-
  getter / null-image-source / update-src-complete），GAINED 5 = 真面子测
  行（not-rendered 1P、null-image「empty src」1P、update-src 1P、
  natural-size ×2F EXIF 挂账）——全按语料变更/端点仿真/shim 修归因。
  首版 fixed1 同域 A/B（P2F=1 存证）与修正轮 fixed2 对照日志留存。
- **engine 哨兵静态核对**：part85 s68 (c)「无/空 src complete true/true」
  broken→true 同语义无移；part88 (6) srcset 移除走 has-prev 臂零改动。
- **make test**：**81 套件 9137P/5F** 对 s294 基线（80 套件 9130P/5F）——
  5F 与 s294 **精确同键**（p56/p99s/r2809/r2810/r3276 已知 CSSOM cssText
  域红），**零新增红**。Δ=+1 套件（1P）+engine 簇 +6P 合计 +7P，名字集差
  分定位 r5426 家族七测试（r5426_content177 + cdata_entity ×6）——归因
  e637f77e8（R5426 CDATA 段内实体字面保留，HEAD 第 6 前序提交，s294 会话
  后落库），非本片 diff（本片 Rust 测试零改动，git diff HEAD --stat 实
  证 4 文件全为 JS shim/脚本/runner 端点臂）。
- **全树 A/B**（A = s294-fulltree-b2.log；B = s295-fulltree-b.log，双方
  TIME_LIMIT=14400 RAF_FRAME_DRIVEN）：**8414 shared 键，翻转恰 2 枚全
  F→P**（list-of-available-images coalesce + image-alternate-no-store
  ——两端点仿真面，testharness.rs WIP 落地归因），**P2F=0**；LOST 4 =
  四页 file-Timeout 行（语料资产 + shim 修复转子测真面），GAINED 5 =
  真面子测行（not-rendered 1P、null-image「empty src」1P、update-src 1P
  落袋 + natural-size ×2F EXIF 挂账转明）——全按语料变更/端点仿真/shim
  修归因。行级 8418 键 7982P/379F/57T → 8419 键 **7987P/379F/53T**
  （ΔP=+5=2 翻转+3 真面落袋；ΔF=0=2 翻转+2 EXIF 相抵；ΔT=−4=四页
  Timeout 行转真面）——**第 27 个连续完美 A/B**。
- **guarded-clippy**：workspace -D warnings 干净。
- **影响面豁免**（quality-gates 按影响选择）：本片 JS shim DOM 语义 +
  runner 端点仿真 + 语料资产，无 layout/render/Rust 产品路径改动——
  `make reftest`、product-smoke、bench-gate 不跑。

## 下一步候选

1. EXIF 方向解码缺口根修（zero-media 解码器特性片——natural-size-
   orientation 2F 挂账面；需盘点 rust 解码器生态 image crate EXIF re-
   orientation 支持面）。
2. s294 候选 1/2/3 维持挂账（picture 父 IMG 臂 env 化 / 宿主
   documentElement 物化根修 / plain-src 运行时触发缺口）。

## 证据

- 本轮日志（evidence/logs/）：s295-alt-no-store-solo-rebased.log（1P）、
  s295-coalesce-solo.log（1P）、s295-not-rendered-dim-solo.log（1P）、
  s295-natural-size-orientation-solo.log（1P/2F 挂账面转明）、
  s295-update-src-complete-baseline.log（Timeout 存证）、
  s295-null-image-source-baseline.log（2P+文件 Timeout 存证）、
  s295-null-image-source-solo-fixed.log（3P 终态）、
  s295-update-src-complete-solo-fixed.log（1P 终态）、
  s295-imgelem-dir-solo-fixed.log（首版 P2F=1 存证）、
  s295-imgelem-dir-solo-fixed2.log（修正轮 P2F=0）、
  s295-fulltree-b.log（终版全树 B 侧）、s295-make-test.log（81 套件
  9137P/5F）、s295-clippy.log。
- 日志 md5：alt-no-store df260d27、coalesce 18ba52eb、not-rendered
  f77ddc61、natural-size c1765e9e、update-src-baseline e1ff935b、
  null-image-baseline 792495df、null-image-fixed 66175528、
  update-src-fixed 0df0aae1、dir-fixed1 6ec42a77、dir-fixed2 8daff567、
  fulltree-b 23826199、make-test 5ddc41fa、clippy d198b7d4。
