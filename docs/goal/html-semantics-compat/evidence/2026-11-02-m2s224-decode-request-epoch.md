# M2 s224 — img.decode() 请求代际面（quick-attach empty-picture 2F 收口）

日期：2026-11-02。切片号 s224（承 s223）。

## 入场定刀位

s223 下一步候选。先勘 not-broken attr sizing/rect 2F：失败形态
`clientWidth expected 50 but got 0`（appendChild 后同步读）。勘根定谳——
clientWidth = `_layoutRect` → RectBridge **快照查询**（rect reflects last
render，shim 注释明言 force-reflow-on-demand 为 follow-up 未做），动态插入
img 同步读时无布局项 → 0。handler 注入点 `apps/renderer/js_worker.rs:1613`
+ `apps/browser/tab_js_worker.rs:465`——sync reflow-on-demand 是 renderer
流车道（layout-engine/worker 环），按车道纪律不入侵，记账转
decode quick-attach 2F。

## 勘根

失败形态：image-decode-with-quick-attach.html 两案
`assert_unreached: Should have rejected: undefined`——expectReject=true
的 promise **resolve 了该 reject**（empty picture not cached / srcset
in empty picture）。同文件另五案（body attach 两案 resolve、picture
with source 两案 reject、empty picture cached resolve）全 Pass。

spec 定谳（HTML §dom-img-decode + update the image data relevant
mutations）：decode() 绑定**调用时的 current request**；attach 进 picture
成员 = relevant mutation → restart——中止 unfinished current request、
启动新请求（同 URL 也重启）。被取代请求的 decode promise →
EncodingError。cached 案 current request 已 completely decoded → attach
不 restart → resolve（上游 NOTE 实证同型）。

根因：shim decode 核是「调用时捕获候选 URL + 决断点复检」模型——候选
URL 比对只覆盖**选中源变化**（picture with source 案 green→blue 已被
拒）；同 URL 重启（empty picture 案）无信号，自驱 fetch 链照常 resolve。

## 施工（shim 2 文件 +24/−1，语料零改动，零 Rust）

请求代际模型三处：

- **part03 `__zwImgRunUpdate5277` fetch 臂**（lazy 门后）：非
  `_imViaDefer`（语义上**新一次** update run，defer 重入是同一 run 的
  detached 延步）bump `_zwImgDecodeEpoch5277[key]`；avail 同步 settle 臂
  无新请求不 bump。
- **part03 `__zwImgDecodeCore`**：executor（同步 = 调用时）捕获 `ep0`；
  微任务入口（候选复检后）+ bitmap 完成点（fetch 窗口内 restart）两处
  比对，不匹配 → EncodingError。plain/SVG 臂传 `undefined` key 不入表。
- **part04 `__zwFireStaticImgSettle`**：epoch 表随 s220 选源记录表同点
  per-document 重置（persistent context 跨案键隔离——stale 代际会假拒）。

七案逐案推演自洽（略详见控制文档）：两靶案 reject ✓；body attach 两案
（TreeMutated 非 picture 父不 Revalidate，epoch 不动）resolve ✓；cached
案（avail 臂不 bump）resolve ✓；picture with source 两案（候选变化既有
臂先拒）reject ✓。

## 验证

- shim 拼接 16 part → node --check SYNTAX_OK（120197 行）。
- decode 目录 solo：**60P/0F/0T 全绿**（修复前 2F）。日志
  evidence/logs/s224-decode-dir.log。
- embedded-content 全域 A/B（A = s224、B = s223 基线日志）：共享键
  **1353/1353**，位移恰 **2 F→P**（两靶案），**P→F = 0**，Timeout
  30→30 稳定，行级 1074P/154F/30T → 1076P/152F/30T。lost 1 为 s223 Fail
  行多行消息缩进续行伪影（非测试键）。日志
  evidence/logs/s224-embedded-content-wip.log。
- zero-engine 全量 **3710P + 0F**（test-guard 包裹，与 s223 持平）。日志
  evidence/logs/s224-zero-engine-full.log。
- 纯 JS 切片（git diff --stat +24/−1）——fmt/clippy 不适用；release
  runner 重编译后跑靶。

## 挂账新记

- **not-broken 2F 车道归因**：sync reflow-on-demand（几何查询强制同步
  重排）——RectBridge 快照查询 + handler 注入在 renderer 流车道
  （apps/renderer js_worker / browser tab_js_worker + layout-engine），
  本流不入侵；renderer 流推进 force-reflow-on-demand 时自然收口。
- 同源面：image-base-url.html 1F（"The image must load" unhandled
  rejection）与 image-alternate-no-store.window.js 1F（同 src 重赋多
  fetch）未动，留勘根。

## 承袭挂账（不动）

decode quick-attach 2F **本轮收口**，旧账目「decode 2F」与之为同项异名
（s218 响应式图片族分型残留，decode 目录 60P/0F 全绿实证无余面）——
两项并删。承袭：not-broken 2F（**本轮车道归因 renderer 流**，见挂账
新记）；update-media/source-relevant-mutations 选源响应面余案；
layout-engine 1px 钳（sizing.rs:2899-2900/3136 渲染流）；error.html 破图
alt 7F；sizes-auto Timeout 68 案；SVG data-URL intrinsic 3F + img
naturalWidth SVG viewBox 族 22F（decoder 耦合 renderer 流 ③A）；
svg-img-density-vs-zoom 7F；image-base-url 1F + image-alternate-no-store
1F（本轮新记勘根候选）；lazy-loading 族 ~24F；part06
`_zwFieldsetDisableFocusReset`（s215）；part03 `_zwMElCheckValidity`
FORM 短路（s210）；onblur flake 族（s209~）；beforeinput.tentative；非
canvas 面一般渲染门 FIXME(rendering)（s218）；flat tree slot
FIXME(flat-tree)（s218）；modern API img 树变更面余项（s221）；
TreeMutated IMG 移除臂（s221）；mimesniff 裸空白 value 理论缺口（s222）；
用户门禁三项（GB-20261031 customizable select 177F / form-nameditem
override builtins / form owner table 缓行族）。
