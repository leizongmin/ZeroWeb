# DC-3 · 批二十四备料腿：speculative-parsing TO 面（83 键）根因定谳

2026-11-23。批十二/批十三遗留的 TO 面归因收口（无 Rust 码笔；4 个一次性
探针页实跑后即删，语料面 gitignored 零残留）。批十二已证 img×text/plain
载入不完成一形；本腿补 script 与 stylesheet 两车道探针，并给出 83 键逐键
资源类归因表。

## 新增探针（script / stylesheet 车道）

按批十二探针同款机制（iframe 助页 = `slow.py?delay=1500` +
`document.write('<plaintext>')` + 资源标签指向
`stash.py?action=put`，顶页 `expect_fetched_onload` 等自身 load 后 take），
两探针各 1 子测试，`FILTER=zzprobe` 经 Makefile 守卫入口实跑
（test-guard 包裹，release 二进制 = 本日树）：

- `zzprobe-script-src`（助页资源 = `<script src=stash put>`）→ **Timeout**
- `zzprobe-stylesheet`（助页资源 = `<link rel=stylesheet href=stash put>`）→ **Timeout**

两键 TO 签名与语料 TO 键同形
（`testharness completed without reporting registered tests (pending:1)`）
＝put 响应（200 / text/plain / 空体）的 script 与 stylesheet 载入同 img
一样永不完成 → 文档 load 链阻断 → take 不发生。对照组隐含在语料内：
同前导（slow.py + plaintext write）的 Pass 键（如 xmp-script-src）证明
前导不致 TO，差异只在资源标签本身。

## 逐键归因表（83 键 = page-load 38 + document-write 45）

静态正则读 helper 资源标签（page-load 读 `-framed.sub.html`、dw 读顶页
write 模板串，19 个正则不可见的 dw 键按同名 page-load 姊妹键镜像补全）：

| 车道 | page-load | dw | 小计 | 键 |
|---|---:|---:|---:|---|
| script | 15 | 22 | 37 | script-src 全变体（async/defer/crossorigin/integrity/module/referrerpolicy/type-ecmascript）、base-href-script-src、base-in-dsd、meta-charset-script-src、math-font-face-script-src、template-shadowrootmode-script-src、svg-script-href/xlinkhref、svg-script-src(dw)、math-script-src(dw)、math-font-script-src(dw)、nested-template-shadowrootmode-1/2(dw) |
| img | 13 | 13 | 26 | image-src、img-src 全变体、img-srcset、video-poster、picture-source-br-img、meta-csp-img-src-asterisk/in-dsd、meta-referrer-no-referrer-img-src、template-shadowrootmode-img-src、svg-image-href/xlinkhref |
| stylesheet | 6 | 6 | 12 | link-rel-stylesheet 全变体（crossorigin/integrity/referrerpolicy/type-text-css）、template-shadowrootmode-link-stylesheet |
| preload | 4 | 4 | 8 | link-rel-preload-as-image/script/style/font-crossorigin |

preload 4 键按 `as=` 目标归入对应车道同族（font 车道未单测探针，标注为
同族假设）；`svg-nav`（svg 树导航边）为注记不影响归道。

**结论**：TO 面根因收敛为一个 engine 载入管道形族——非标准响应的资源
载入永不完成（img/script/stylesheet 三车道探针齐了，font 车道假设）→
文档 load 永不触发 → `expect_fetched_onload` 不 resolve。与批九已挂账
「样式表载入链挂起」、批十二「img 载入挂起」同域，script 车道为新登记形。

## 四键非对称观察（dw 形独有 TO）

`math-script-src`、`math-font-script-src`、
`nested-template-shadowrootmode-1/2` 四键 page-load 形 **Pass**（math 树与
template 子树的 script 取数不阻断 load）而 dw 形 **TO**（嵌套 write 管道
同类标签阻断 load）——同资源标签跨管道载入阻断行为不对称，engine 碰头
输入新增一形。

另批十三勘误的 7 键形（expectation=false 假 Pass→TO：真实 take 体生效后
断言在 promise 链内失败、done 不达）独立于本形族，照旧挂账不重计。

## leg-2 修复链含义

扫描器刀（leg-2 主刀）+ plaintext kill 补全（iframe / 嵌套 write 管道，
批十二战役图两形）落地后，主解析器不再创建 load-blocking 资源元素，
本面 83 键预期按 chrome 118/127 重排（A/B 重记）；载入挂起形本身仍是
独立正确性 bug（真实网页 img/script/style 收到非 200 响应时 load 永挂），
维持批九/批十二刀位挂账不销。

[批十二证据](2026-11-22-dc2-batch12-syntax-speculative-parsing.md) ·
[批十三证据](2026-11-22-dc2-batch13-query-encoding-py-fixtures.md) ·
[逐文件 JSON](2026-11-23-dc3-batch24-speculative-to-face-rootcause-probes.json)
