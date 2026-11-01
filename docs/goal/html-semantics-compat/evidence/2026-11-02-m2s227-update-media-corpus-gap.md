# M2 s227 — update-media 文件级 Timeout 勘根定性：语料 images/ 缺口（非引擎缺陷）+ 本地补图

日期：2026-11-02。切片号 s227（承 s226）。

## 入场定刀位

s226 下一步候选首选「update-media 选源响应面余项勘根」。靶案
`the-img-element/update-media.html` 自 s219 起文件级 Timeout（embedded-content
30T 之一），控制文档挂账为「选源响应面余项」。

## 勘根（探针七轮分型，最终定性为语料缺口）

探针文件 `zz-s227-probe.html`（本地 gitignored，验后已删）把该测试拆为
static 选源 / setAttribute / IDL media / srcset 臂 / 触发链计数 / fetch 半观测
八案，五轮收敛到「attr 触发链完整（attr=1/reval=1/queue=1 per flip）、currentSrc
翻转正确、但 img load 永不派发」。第六轮 p8 全链可视（fetch 响应到达时点 +
createImageBitmap 拒因捕获 + epoch 赋值陷井 + pending 队列长度 + avail 键 +
naturalWidth 前后对照）定谳：

- **每一笔图片 fetch 都 `status=404 ok=false`**（连 parse 期 fail.gif 也是）。
- `_zwAvailableImages` 全空、naturalWidth 恒 0、error 事件按链路正常派发。

即：`/images/smiley.png`、`/images/fail.gif` 在本地 wpt-data 语料树根本不存在。
404 → `!resp.ok` → error 代替 load → async_test 永不 done → 文件级 Timeout。
此前五轮观测到的「load 不派发」全部是 404 的下游症状，与 update-the-image-data
运行核（part03 fetch→blob→createImageBitmap→settle 全链）无关。

语料根因：wpt-data（leizongmin/zeroweb-wpt-data v1.10）`images/` 仅打包 11
文件，无 smiley.png / fail.gif；update-media.html 是 the-img-element 内唯一
引用者（embed-hidden-attribute.html 引用 fail.gif 但不在运行器文件选择内，
sizes-auto-rendering*.html 属 reftest 形态且不在 manifest）。

## 施工（零 shim、零 Rust——纯语料本地补入）

按 R5274/R5276 语料缺口惯例，从上游 wpt images/ 以 git sparse clone（blob:none
filter）取原件补入本地 `tests/wpt-runner/wpt-data/images/`（gitignored）：

- smiley.png：300x150 PNG，1852 B
- fail.gif：GIF89a 320x240，24480 B

并在 `scripts/update-wpt-data.sh` 已知缺口注释挂账（升级新 ref 后需重新补）。

## 验证

- 靶案 solo **Pass**（Timeout → Pass）。日志 s227-update-media-solo-afterimg.log。
- 探针 r7 复核链路语义全对：p4r/p5 load 正常；p7 `fetchSmiley=0` + naturalWidth
  300 + load 到达 = avail 缓存命中臂（同步 settle + 延步派 load）按设计工作；
  p2r/p6 失败为探针自身计数口径缺陷（eval 期挂监听捕到 parse 期 load、页面级
  计数把其他案的写入算入），非引擎问题。
- the-img-element 目录级 A/B（B = s226 基线日志）：**861P→862P、75F→75F、
  17T→16T**，位移恰 **1 Timeout→Pass**（靶案），零 P→F。日志
  s227-img-element-dir.log。
- embedded-content 全域 A/B（B = s226 基线日志）：**1077P→1078P、150F→150F、
  30T→29T**，位移恰 **1 Timeout→Pass**（靶案；B 侧 `path :: path` 文件级
  Timeout 条目 ↔ A 侧真实子测名，key 集单侧差异各 1 即此一事两态），零 P→F。
  日志 s227-embedded-content-wip.log。
- decode 目录零代码变动，且为 embedded-content 域子集，已由全域 A/B 覆盖
  （join 中 decode 面零位移）。
- zero-engine 不适用：本切片零 Rust、零 shim 变更。
- 勘根证据链：s227-probe1..5.log（逐轮收敛）→ s227-probe6.log（p8 404 定谳）→
  s227-probe7.log（补图后链路复核）。探针文件已删。

## 方法论教训（已并入经验）

语料缺资产与引擎 bug 在「文件级 Timeout」症状上不可区分：404 → error 代替
load → 等待 load 的 async_test 永不 done。勘根 imagery 类测试时**先验语料
资产存在性**（`ls wpt-data/images/...`），再进 shim/引擎分型；fetch 到达观测
（status/ok）是分辨二者的最短路径。

## 承袭挂账（不动）

not-broken 2F（renderer 流车道归因）；layout-engine 1px 钳（sizing.rs 渲染
流）；error.html 破图 alt 7F；sizes-auto Timeout 68 案；SVG data-URL intrinsic
3F + img naturalWidth SVG viewBox 族 22F（decoder 耦合 renderer 流 ③A）；
svg-img-density-vs-zoom 7F；lazy-loading 族 ~24F；part06
`_zwFieldsetDisableFocusReset`（s215）；part03 `_zwMElCheckValidity` FORM 短路
（s210）；onblur flake 族（s209~）；beforeinput.tentative；非 canvas 面一般渲染
门 FIXME(rendering)（s218）；flat tree slot FIXME(flat-tree)（s218）；modern API
img 树变更面余项（s221）；TreeMutated IMG 移除臂余项（s221）；mimesniff 裸空白
value 理论缺口（s222）；image-alternate-no-store known limitation（s225）；
用户门禁三项（GB-20261031 customizable select 177F / form-nameditem override
builtins / form owner table 缓行族）。update-media 本轮收口并删。
