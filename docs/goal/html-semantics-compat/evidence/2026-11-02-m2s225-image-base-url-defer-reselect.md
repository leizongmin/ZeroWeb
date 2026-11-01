# M2 s225 — image-base-url 延步重选（选源随 update microtask 生效）+ image-alternate-no-store 定谳不施工

日期：2026-11-02。切片号 s225（承 s224）。

## 入场定刀位

s224 下一步候选首选「image-base-url 1F + image-alternate-no-store 1F 勘根
（decode 纪元面延伸）」。

## 勘根

### Case 1：image-alternate-no-store 1F —— 定谳不施工

失败形态：同 src 重赋期望多次 fetch。勘根定谳——语料引用
`alternate.py`（nosniff/no-store 响应头仿真端点），.py 端点资源**刻意不
入语料清单**（157 行注释同族先例：runner 无 CGI 结构性边界，
testharness_fileapi.rs:77 `source.contains(".py")` 分道先例，cookie.py 同
族）。记 known limitation，与 runner 端点落地同批再议。

### Case 2：image-base-url 1F —— 施工收口

失败形态：`An image should not be refetched upon insertion asynchronously
after its Document's base URL changes`（首次 loadPromise error 拒绝，
unhandled rejection "The image must load"）。语料结构：`<img src=green.png>`
detached + `<base href="resources/">` 后插入——spec whatwg/html#7383：
update the image data 对**未连接 img 整体延步运行**，URL 解析随 update
microtask 生效，src 后 `<base>` 变更在延步内可见（image-base-url 断言面）。

根因：shim RunUpdate 对 detached img 的 defer 闭包**只延 fetch 不延选源**
——延步内仍用同步阶段冻结的旧候选（解析于 base 变更前 → `green.png`
相对文档基 URL 404）。前置核实：`_zwDocBaseURL`（part03:9315）已活读
文档基 URL（R5270 动态 base 面），缺的只是选源时机。

## 施工（shim 1 文件 +27/−3，语料清单 1 文件 +5，零 Rust）

- **part03 RunUpdate detached defer 闭包重做选源**：延步内
  `__zwSelectImageSource5277` 重跑（`_imViaDefer=true` 保持同一 update
  run 语义，s224 纪元不 bump）；空选中（延步内 src 已撤）走共享
  null-selection 面。
- **part03 `__zwImgRevalidate5277` 空选中臂抽取**为
  `globalThis.__zwImgNullSelection5277`（`_nkKey`/`_nkPrev` 门、broken
  写入、`_nkHasLink` error 门、pending-events 排队）——Revalidate 尾与
  新 defer 闭包共享，消重复。
- **语料清单 ③c 段**：`fetch_raw green.png`——image-base-url 语料经
  `<base href="resources/">` 重写引用 resources/green.png，重写后路径落
  在域内 resources/，缺席即假 Fail "The image must load"（utf-8.css 同坑
  变体：幂等快路径整目录跳过后补清单）。green.png（91 字节 100×100
  PNG）本地落盘 wpt-data（gitignore 本地语料不入库）。

## 验证

- shim 拼接 16 part → node --check SYNTAX_OK（120221 行）。
- image-base-url solo **Pass**（修复前 1F）。日志
  evidence/logs/s225-image-base-url-solo.log。
- decode 目录回归 **60P/0F/0T 全绿**（s224 面零回归）。日志
  evidence/logs/s225-decode-dir.log。
- the-img-element 目录级 **861P/76F/17T**（s223 基线 858P/79F/17T），
  join 位移恰 **3 F→P**（s224 quick-attach 2 + 本刀 image-base-url 1）
  **零 P→F**，lost 2 为 s223 Fail 行多行消息缩进续行伪影（两靶案转
  Pass 后消失）gained 0。日志 evidence/logs/s225-img-element-dir.log。
- embedded-content 全域 A/B（B = s224 基线日志）：**1077P/151F/30T**
  （s224 1076P/152F/30T），共享键位移恰 **1 F→P**（image-base-url 靶
  案）**零 P→F** Timeout 30→30 稳定，lost 1 为 s224 基线同案 Fail 多行
  消息续行伪影、gained 0。日志
  evidence/logs/s225-embedded-content-wip.log。
- zero-engine 全量 **3710P+0F** 持平 s224（--compile-first
  test-guard 包裹；注：编译期裸内存限三次触限被杀后按质量门禁文档
  `--compile-first` 编译阶段不设阈、只保护测试运行）。日志
  evidence/logs/s225-zero-engine-full.log。
- 纯 JS 切片（git diff --stat shim +27/−3、清单 +5）——fmt/clippy 不适
  用；release runner 重编译（28.87s）后跑靶（shim include_str 进二进
  制）。

## 挂账新记

- **image-alternate-no-store 1F known limitation**：runner 无 CGI 端点
  面（.py 资源刻意不入语料清单），同 utf-8.css/cookie.py 族，端点落地
  同批再议。
- green.png 语料基准变更注记：下轮全量 join 对应 GAINED 按语料变更
  归因非回归（③b 段先例同构）。

## 承袭挂账（不动）

image-base-url 1F **本轮收口**（s224 挂账新记该项并删）；image-
alternate-no-store 1F **本轮定谳 known limitation**（承袭项转段）。
承袭：not-broken 2F（renderer 流车道归因）；update-media/source-
relevant-mutations 选源响应面余案；layout-engine 1px 钳（sizing.rs:
2899-2900/3136 渲染流）；error.html 破图 alt 7F；sizes-auto Timeout 68
案；SVG data-URL intrinsic 3F + img naturalWidth SVG viewBox 族 22F
（decoder 耦合 renderer 流 ③A）；svg-img-density-vs-zoom 7F；lazy-
loading 族 ~24F；part06 `_zwFieldsetDisableFocusReset`（s215）；part03
`_zwMElCheckValidity` FORM 短路（s210）；onblur flake 族（s209~）；
beforeinput.tentative；非 canvas 面一般渲染门 FIXME(rendering)（s218）；
flat tree slot FIXME(flat-tree)（s218）；modern API img 树变更面余项
（s221）；TreeMutated IMG 移除臂（s221）；mimesniff 裸空白 value 理论
缺口（s222）；用户门禁三项（GB-20261031 customizable select 177F /
form-nameditem override builtins / form owner table 缓行族）。
