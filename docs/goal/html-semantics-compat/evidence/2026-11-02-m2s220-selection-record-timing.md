# M2 s220 — img 选源记录 currentSrc 时序面（selection-record timing）

日期：2026-11-02。切片号 s220（承 s219）。

## 入场定刀位

s219 下一步候选「选源响应面 update-the-image-data 簇/img-picture-ancestor
……另勘根」。本刀收口 selection 时序子面。

## 勘根

s219 embedded-content A/B F 分布下钻 + 语料单跑分型，16F 五簇：

- **selection 时序面 6F 定刀**（img-picture-ancestor 4F +
  current-request-microtask 2F）：spec current request 的 current URL
  更新是**微任务级**（relevant mutation → microtask → update-the-image-data
  的 select-an-image-source 半先落 URL，fetch 半才异步）；本引擎选中态
  （resourceState）对静态解析 img 耦合 window load settle、对 detached img
  延一步——解析期/微任务期读 currentSrc 拿不到已选源。
- fail-to-resolve 4F：URL 解析器未拒非法 host `src="//["`（期望原文返
  回，实际解析成功换 base）——`_zwResolveFetchUrl` 面挂账。
- not-broken 2F：动态插入 img 设 attr 尺寸后 clientWidth 0≠50——attr
  sizing/rect 面挂账。
- decode quick-attach 2F 挂账。
- update-the-source-set 1F：`type="image/gif;encodings="` spec 合法空
  parameter value 被 `__zwSourceSupportedType5277` regex 拒——挂账。

## 施工（shim 3 文件 +118/−1，语料零改动，零 Rust）

选中记录表与 settle 解耦三帮手（part03，`__zwSelectImageSource5277` 后）：

- `__zwImgSelectionRecord5277`：选源链 `__zwSelectImageSource5277` 纯读
  落 `_zwSelectedSources5277[key]`（绝对形 URL）；loading=lazy 不记录
  （spec 选择推迟视口门）；**有在队 selection 实例（pending>0）的 key
  跳过**（补刀二——扫描不得旁路在队微任务序）。
- `__zwImgSelectionScan5277`：首读懒扫全文档 img 一次（静态解析 img 无
  mutation 钩，唯一入表路径）；表/旗标/计数随每次 settle 扫描 per-document
  重置（persistent context 跨案键隔离）。FIXME: 随解析期
  update-the-image-data 微任务接线改为插入点记录后移除懒扫。
- `__zwImgSelectionQueue5277`：mutation 队列点入队（pending 计数 +
  queueMicrotask）；微任务内计数 `--` 后 >0 即早退（spec update-the-image-
  data 排空/cancel 语义——同一元素已有在队实例时先队实例让位最新实例；
  002「the first task should be canceled」断言面）。计数自增在
  queueMicrotask 存在性门**之后**（补刀二——缺 API 时不得泄漏计数永久
  压制 Record）。

接线三点：

- part03 `__zwImgAttrMutated5277` IMG src/srcset/sizes 分支（setAttribute
  及 IDL srcset/sizes 汇流点）→ Queue。
- part05 src IDL setter IMG 臂 → Queue（src 专用臂不走 1263 通用 trap
  分流，两臂各恰一队）。
- part04 currentSrc getter 无 resourceState 时回落记录表（`_zwResolveFetchUrl`
  解析为绝对形；首读触发懒扫；AUDIO/VIDEO 不回落）；`__zwFireStaticImgSettle`
  开头重置表/计数/旗标。

## 勘根后两轮补刀

- **补刀一（排空语义）**：首轮微任务直录记录，002 卡 `currentSrc should
  be "" got resolved`——src 后 srcset 连写，src 的微任务先把 srcset 已在
  场的选择落了。spec update-the-image-data：「同一元素已有在队更新实例时
  先队实例早退（return early）让位最新实例」→ pending 计数实现。
- **补刀二（扫描旁路）**：排空后 002 同形失败。复核二进制新鲜度（Makefile
  依赖 `zero-wpt-runner-release` 已重建，mtime 晚于 shim 编辑）排除陈旧
  二进制；002 全读 + 队列点覆盖核对（part05:1263 通用 trap `moAttr!=='src'`
  分流 → src/srcset 各恰排队一次）排除双队/漏队——定谳**懒扫旁路**：
  断言 A（首个 currentSrc 读）触发懒扫把 src=/srcset= 已在场的 img 以同
  步近似落记录，塌掉微任务序，B 见 resolved。修法 = Record 对 pending>0
  跳过（在队实例自己按序落）+ Queue 计数自增移到存在性门后。

## 验证

- shim 拼接 15 part → node --check SYNTAX_OK（每轮补刀后重跑）。
- 靶语料 solo 三面：img-picture-ancestor **4/4 全 Pass**；current-request-
  microtask-002 **F→P**（排空 + 扫描旁路两刀后）；current-request-microtask
  001 停断言 4（`after appending img to picture` expected data:,b got
  data:,c——缺 picture 插入钩，挂账不扩面）。
- zero-engine 全量 **3684P + 0F**（与 s219 基线持平，零回归）。日志
  evidence/logs/s220-zero-engine-full.log。
- embedded-content 域 A/B（A = s220 修复后、B = s219 基线日志）：共享键
  **1257/1257**，漂移恰 **5 F→P**（img-picture-ancestor 4 案 +
  current-request-microtask-002——全靶面零扩面），**P→F = 0**，其他位移
  （含 Timeout 摆动）**0**。日志 evidence/logs/s220-embedded-content-wip.log。
  方向口径：join A B 后 `$2=="Pass" && $3=="Fail"` = F→P 改善（$2=A 侧）。
- 本切片纯 JS（git diff --stat：shim 3 文件 +118/−1，零 Rust）——fmt/clippy
  不适用。

## 挂账新记

- **img-into-picture 插入钩**：appendChild trap IMG 直接父 PICTURE →
  微任务 re-record（001 断言 4 收口位——spec appendChild 入 picture 非
  relevant mutation 但改选源上下文，selected source 需重算）。
- fail-to-resolve 4F：`_zwResolveFetchUrl` 未拒非法 host（URL 解析器面，
  爆炸半径大独立勘根）。
- not-broken 2F：动态插入 attr 尺寸 clientWidth 0（attr sizing/rect 面）。
- decode quick-attach 2F。
- update-the-source-set 1F：`;encodings=` 空 parameter value 合法形被
  supported type regex 拒。

## 承袭挂账（不动）

update-media/source-relevant-mutations/update-the-source-set 选源响应面
余案；layout-engine 1px 钳（sizing.rs:2899-2900/3136，s219 移交渲染流）；
error.html 破图 alt 7F；sizes-auto Timeout 68 案；parse-a-srcset URL
serializer 1F；SVG data-URL intrinsic 3F + img naturalWidth SVG viewBox
族 22F（decoder 耦合 renderer 流 ③A）；svg-img-density-vs-zoom 7F；
decode 2F；lazy-loading 族 ~24F；part06 `_zwFieldsetDisableFocusReset`
（s215）；part03 `_zwMElCheckValidity` FORM 短路（s210）；onblur flake
族（s209~）；beforeinput.tentative；非 canvas 面一般渲染门
FIXME(rendering)（s218）；flat tree slot FIXME(flat-tree)（s218）；用户
门禁三项（GB-20261031 customizable select 177F / form-nameditem override
builtins / form owner table 缓行族）。
