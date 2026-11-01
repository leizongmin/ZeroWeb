# M2 s223 — source 候选 URL 解析失败保持原串（fail-to-resolve 4F 收口 + URL serializer 1F 意外收口）

日期：2026-11-02。切片号 s223（承 s222）。

## 入场定刀位

s222 挂账首选「fail-to-resolve URL 解析器面独立勘根（4F，爆炸半径大）」。

## 勘根

s221 基线失败形态：fail-to-resolve.html 四案（`src="//["` /
`srcset="//["` / `srcset="//[" src=合法` / `srcset="//[, 合法"`）全部
expected `"//["` but got `"https://wpt.test/["`。node 验证 WHATWG URL
parser 对 `//[`、`//[,` 均抛 Invalid URL（bracket host 无闭合）。

spec 定谳（WHATWG HTML §update the image data）：

- 候选 URL 在 create/select source set 层**不 parse**（raw string）；parse
  只发生在选源后步骤 12「encoding-parsing-and-serializing a URL」。
- 步骤 13 parse failure → current request state **broken** +
  **current URL = selected source 原始串**（步骤 13.4.1 queued task 臂）。
  currentSrc 返回 current URL → 四案期望原串 `//[`，不回落合法候选、
  不返回空串。案 3/4 双候选同密度去重（select an image source 步骤 1）
  恰选中 `//[`，spec 行为自洽。

根因链（shim 侧三处漏）：

1. `__zwCreateSourceSet5277` / `__zwUpdateSourceSet5277` source 分支的
   候选就地绝对化走 `_zwResolveFetchUrl`——其手写 fallback
   （part01:4175-4192）不验证 host，`//[` 被误拼成 `https://wpt.test/[`。
2. currentSrc 记录臂（part04）对记录值再过 `_zwResolveFetchUrl`（同误拼）。
3. settle 后四案 img 有 `_resourceStates[key]`（broken 臂，url = 选中源），
   getter 走 resourceState 臂——`_zwResolveFetchUrl` 第三次误拼。

关键定位：settle 静态扫描（`__zwFireStaticImgSettle`）的请求 URL 同样取
自 `__zwSelectImageSource5277`，broken 臂写 `url: _zwImSrc` → 选中源
raw 化后 state.url 自动随行。

## 施工（shim 2 文件 +34/−12，语料零改动，零 Rust）

- **part03** 新增 `__zwAbsolutizeSourceUrl5277`（WHATWG URL 解析失败返
  null，base 与 `_zwResolveFetchUrl` 同源）；create 与 source 分支两个
  绝对化点换用（null → 保持原串）；create 注记补 spec broken 臂说明。
- **part04** currentSrc 记录臂直接返回记录值（经新 helper 已是绝对形或
  解析失败原串）；resourceState IMG 臂换 `__zwAbsolutizeSourceUrl5277`
  （null → 原串）。AUDIO/VIDEO 臂不动。

合法 URL 行为零变化（`new URL().href` 与 `_zwResolveFetchUrl` 成功臂同
逻辑）；`data:` scheme 等绝对 URL 语义差：旧快路直返原串，新恒过 WHATWG
序列化（非 ASCII 百分号编码）——spec 步骤 12 serialization 语义。

消费方普查（`__zwSelectImageSource5277` 七处调用）：记录、settle、
revalidate 运行核、naturalDim 快照、lazy 视口、`__zwSelectUrl5277`——
合法 URL 不变；非法 URL 此前误拼后 404，现在原串 fetch 失败，结局同为
broken，currentSrc 之外无观测差。

## 验证

- shim 拼接 16 part（按 js_dom_bridge.rs include_str! 序提取）→
  node --check SYNTAX_OK（120072 行）。
- 离线 helper 矩阵：`//[` / `//[,` → null（保原串）；`/images/red.png`
  / `data:,a` / 相对 / 绝对 URL 形态全部不变。
- 靶语料 solo：fail-to-resolve **4/4 全绿**（修复前 4F）。日志
  evidence/logs/s223-fail-to-resolve-solo.log。
- the-img-element 目录级：858P/79F/17T，(key,state) join 对 s222 基线
  位移恰 **5 F→P**（4 靶案 + 1 意外），**P→F = 0**，lost/gained 0。
  日志 evidence/logs/s223-img-element-dir.log。
- embedded-content 全域 A/B（A = s223、B = s222 基线日志）：共享键
  **1354/1354**，位移恰 **5 F→P**、**P→F = 0**，Timeout 30→30 稳定，
  行级 1069P/159F/30T → 1074P/154F/30T。lost 2 为 s222 Fail 行多行消息
  缩进续行伪影（非测试键）。日志
  evidence/logs/s223-embedded-content-wip.log。
- zero-engine 全量 **3710P + 0F**（test-guard 包裹；s222 时 3684P，
  +26 为并行 main 流合入；零失败）。日志
  evidence/logs/s223-zero-engine-full.log。
- 纯 JS 切片（git diff --stat +34/−12）——fmt/clippy 不适用；release
  runner 重编译后跑靶（shim include_str 进二进制）。

## 意外收口

**parse-a-srcset URL serializer 1F**（承袭挂账）：
`parse-a-srcset-attribute.html :: "data:,a "` F→P——旧
`_zwResolveFetchUrl` 对 `data:` scheme 快路直返原串跳过序列化；新 helper
恒过 WHATWG URL parser，NBSP 规范百分号编码为 `%C2%A0`（spec 步骤 12）。
挂账该项删除。

## 挂账

无新记。承袭挂账不动：not-broken 2F（attr sizing/rect 面）；decode
quick-attach 2F；update-media/source-relevant-mutations 选源响应面余案；
layout-engine 1px 钳（sizing.rs:2899-2900/3136 渲染流）；error.html 破图
alt 7F；sizes-auto Timeout 68 案；SVG data-URL intrinsic 3F + img
naturalWidth SVG viewBox 族 22F（decoder 耦合 renderer 流 ③A）；
svg-img-density-vs-zoom 7F；decode 2F；lazy-loading 族 ~24F；part06
`_zwFieldsetDisableFocusReset`（s215）；part03 `_zwMElCheckValidity` FORM
短路（s210）；onblur flake 族（s209~）；beforeinput.tentative；非 canvas
面一般渲染门 FIXME(rendering)（s218）；flat tree slot FIXME(flat-tree)
（s218）；modern API img 树变更面余项（s221）；TreeMutated IMG 移除臂
（s221）；mimesniff 裸空白 value 理论缺口（s222，无语料不扩面）；用户
门禁三项（GB-20261031 customizable select 177F / form-nameditem override
builtins / form owner table 缓行族）。
